#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::fs::File;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const BZ_MAX_HANDLED_BLOCKS: usize = 50000;
const BZ_MAX_FILENAME: usize = 2000;

const BLOCK_HEADER_HI: u32 = 0x00003141u32;
const BLOCK_HEADER_LO: u32 = 0x59265359u32;

const BLOCK_ENDMARK_HI: u32 = 0x00001772u32;
const BLOCK_ENDMARK_LO: u32 = 0x45385090u32;

enum Error {
    Reading(std::io::Error),
    Writing(std::io::Error),
    TooManyBlocks(usize),
}

#[repr(C)]
struct BitStream {
    handle: File,
    buffer: i32,
    buffLive: i32,
    mode: u8,
}

fn readError(program_name: &Path, in_filename: &Path, io_error: std::io::Error) -> ExitCode {
    eprintln!(
        "{}: I/O error reading `{}', possible reason follows.",
        program_name.display(),
        in_filename.display(),
    );

    eprintln!("{}", io_error);

    eprintln!(
        "{}: warning: output file(s) may be incomplete.",
        program_name.display(),
    );

    ExitCode::FAILURE
}

fn writeError(program_name: &Path, in_filename: &Path, io_error: std::io::Error) -> ExitCode {
    eprintln!(
        "{}: I/O error writing `{}', possible reason follows.",
        program_name.display(),
        in_filename.display(),
    );

    eprintln!("{}", io_error);

    eprintln!(
        "{}: warning: output file(s) may be incomplete.",
        program_name.display()
    );

    ExitCode::FAILURE
}

fn tooManyBlocks(program_name: &Path, in_filename: &Path, max_handled_blocks: usize) -> ExitCode {
    let program_name = program_name.display();

    eprintln!(
        "{}: `{}' appears to contain more than {max_handled_blocks} blocks",
        program_name,
        in_filename.display(),
    );

    eprintln!("{program_name}: and cannot be handled.  To fix, increase");
    eprintln!("{program_name}: BZ_MAX_HANDLED_BLOCKS in bzip2recover.rs, and recompile.");

    ExitCode::FAILURE
}

fn bsOpenReadStream(stream: File) -> BitStream {
    BitStream {
        handle: stream,
        buffer: 0,
        buffLive: 0,
        mode: b'r',
    }
}

fn bsOpenWriteStream(stream: File) -> BitStream {
    BitStream {
        handle: stream,
        buffer: 0,
        buffLive: 0,
        mode: b'w',
    }
}

fn bsPutBit(bs: &mut BitStream, bit: i32) -> Result<(), Error> {
    if bs.buffLive == 8 {
        bs.handle
            .write_all(&[bs.buffer as u8])
            .map_err(Error::Writing)?;
        bs.buffLive = 1;
        bs.buffer = bit & 0x1;
    } else {
        bs.buffer = bs.buffer << 1 | bit & 0x1;
        bs.buffLive += 1;
    }

    Ok(())
}

fn bsGetBit(bs: &mut BitStream) -> Result<Option<bool>, Error> {
    if bs.buffLive > 0 {
        bs.buffLive -= 1;

        Ok(Some(bs.buffer >> bs.buffLive & 0x1 != 0))
    } else {
        let mut retVal = [0u8];
        let n = bs.handle.read(&mut retVal).map_err(Error::Reading)?;

        // EOF
        if n == 0 {
            return Ok(None);
        }

        bs.buffLive = 7;
        bs.buffer = retVal[0] as i32;

        Ok(Some(bs.buffer >> 7 & 0x1 != 0))
    }
}

fn bsClose(mut bs: BitStream) -> Result<(), Error> {
    if bs.mode == b'w' {
        while bs.buffLive < 8 {
            bs.buffLive += 1;
            bs.buffer <<= 1;
        }
        bs.handle
            .write_all(&[bs.buffer as u8])
            .map_err(Error::Writing)?;
        bs.handle.flush().map_err(Error::Writing)?;
    }

    Ok(())
}

fn bsPutUChar(bs: &mut BitStream, c: u8) -> Result<(), Error> {
    for i in (0..8).rev() {
        bsPutBit(bs, (c as u32 >> i & 0x1) as i32)?;
    }

    Ok(())
}

fn bsPutUInt32(bs: &mut BitStream, c: u32) -> Result<(), Error> {
    for i in (0..32).rev() {
        bsPutBit(bs, (c >> i & 0x1) as i32)?;
    }

    Ok(())
}

fn main_help(program_name: &Path, in_filename: &Path) -> Result<ExitCode, Error> {
    let mut B_START = [0u64; 50000];
    let mut B_END = [0u64; 50000];
    let mut RB_START = [0u64; 50000];
    let mut RB_END = [0u64; 50000];

    let progname = program_name.display();

    if in_filename.as_os_str().len() >= BZ_MAX_FILENAME - 20 {
        eprintln!(
            "{}: supplied filename is suspiciously (>= {} chars) long.  Bye!",
            program_name.display(),
            in_filename.as_os_str().len(),
        );

        return Ok(ExitCode::FAILURE);
    }

    let Ok(inFile) = std::fs::File::options().read(true).open(in_filename) else {
        eprintln!("{}: can't read `{}'", progname, in_filename.display());

        return Ok(ExitCode::FAILURE);
    };

    let mut bsIn = bsOpenReadStream(inFile);
    eprintln!("{}: searching for block boundaries ...", progname);

    let mut bitsRead: u64 = 0;
    let mut buffLo: u32 = 0;
    let mut buffHi = buffLo;
    let mut currBlock = 0;
    B_START[currBlock] = 0;
    let mut rbCtr = 0;

    loop {
        let b = bsGetBit(&mut bsIn)?;
        bitsRead = bitsRead.wrapping_add(1);
        match b {
            None => {
                if bitsRead >= B_START[currBlock] && bitsRead.wrapping_sub(B_START[currBlock]) >= 40
                {
                    B_END[currBlock] = bitsRead.wrapping_sub(1);
                    if currBlock > 0 {
                        eprintln!(
                            "   block {} runs from {} to {} (incomplete)",
                            currBlock, B_START[currBlock], B_END[currBlock],
                        );
                    }
                }
                break;
            }
            Some(b) => {
                buffHi = buffHi << 1 | buffLo >> 31;
                buffLo = buffLo << 1 | b as u32;
                if (buffHi & 0xffff) == BLOCK_HEADER_HI && buffLo == BLOCK_HEADER_LO
                    || (buffHi & 0xffff) == BLOCK_ENDMARK_HI && buffLo == BLOCK_ENDMARK_LO
                {
                    B_END[currBlock] = if bitsRead > 49 {
                        bitsRead.wrapping_sub(49)
                    } else {
                        0
                    };

                    if currBlock > 0 && (B_END[currBlock]).wrapping_sub(B_START[currBlock]) >= 130 {
                        eprintln!(
                            "   block {} runs from {} to {}",
                            rbCtr + 1,
                            B_START[currBlock],
                            B_END[currBlock],
                        );
                        RB_START[rbCtr] = B_START[currBlock];
                        RB_END[rbCtr] = B_END[currBlock];
                        rbCtr += 1;
                    }
                    if currBlock >= BZ_MAX_HANDLED_BLOCKS {
                        return Err(Error::TooManyBlocks(BZ_MAX_HANDLED_BLOCKS));
                    }
                    currBlock += 1;
                    B_START[currBlock] = bitsRead;
                }
            }
        }
    }

    bsClose(bsIn)?;

    /*-- identified blocks run from 1 to rbCtr inclusive. --*/

    if rbCtr < 1 {
        eprintln!("{}: sorry, I couldn't find any block boundaries.", progname);

        return Ok(ExitCode::FAILURE);
    }

    eprintln!("{}: splitting into blocks", progname);

    let Ok(inFile) = std::fs::File::options().read(true).open(in_filename) else {
        eprintln!("{}: can't read `{}'", progname, in_filename.display());

        return Ok(ExitCode::FAILURE);
    };
    bsIn = bsOpenReadStream(inFile);

    let mut blockCRC: u32 = 0;
    let mut bsWr: Option<BitStream> = None;
    let mut wrBlock = 0;

    bitsRead = 0;

    loop {
        let Some(b) = bsGetBit(&mut bsIn)? else {
            // EOF
            break;
        };

        buffHi = buffHi << 1 | buffLo >> 31;
        buffLo = buffLo << 1 | b as u32;

        if bitsRead == 47u64.wrapping_add(RB_START[wrBlock]) {
            blockCRC = buffHi << 16 | buffLo >> 16;
        }
        if bsWr.is_some() && bitsRead >= RB_START[wrBlock] && bitsRead <= RB_END[wrBlock] {
            bsPutBit(bsWr.as_mut().unwrap(), b as i32)?;
        }
        bitsRead = bitsRead.wrapping_add(1);
        if bitsRead == (RB_END[wrBlock]).wrapping_add(1) {
            if let Some(mut bsWr) = bsWr.take() {
                bsPutUChar(&mut bsWr, 0x17)?;
                bsPutUChar(&mut bsWr, 0x72)?;
                bsPutUChar(&mut bsWr, 0x45)?;
                bsPutUChar(&mut bsWr, 0x38)?;
                bsPutUChar(&mut bsWr, 0x50)?;
                bsPutUChar(&mut bsWr, 0x90)?;
                bsPutUInt32(&mut bsWr, blockCRC)?;
                bsClose(bsWr)?;
            }

            if wrBlock >= rbCtr {
                break;
            }
            wrBlock += 1;
        } else if bitsRead == RB_START[wrBlock] {
            // we've been able to open this file, so there must be a file name
            let filename = in_filename.file_name().unwrap();

            let filename = format!("rec{:05}{}", wrBlock + 1, filename.to_string_lossy());

            let out_filename = in_filename.with_file_name(&filename).with_extension("bz2");

            eprintln!(
                "   writing block {} to `{}' ...",
                wrBlock + 1,
                out_filename.display(),
            );

            let mut options = std::fs::File::options();
            options.write(true).create(true);

            #[cfg(unix)]
            options.mode(libc::S_IWUSR | libc::S_IRUSR);

            #[cfg(unix)]
            options.custom_flags(libc::O_EXCL);

            let Ok(outFile) = options.open(&out_filename) else {
                eprintln!("{}: can't write `{}'", progname, out_filename.display());

                return Ok(ExitCode::FAILURE);
            };

            bsWr = {
                let mut bsWr = bsOpenWriteStream(outFile);
                bsPutUChar(&mut bsWr, 0x42)?;
                bsPutUChar(&mut bsWr, 0x5a)?;
                bsPutUChar(&mut bsWr, 0x68)?;
                bsPutUChar(&mut bsWr, 0x30 + 9)?;
                bsPutUChar(&mut bsWr, 0x31)?;
                bsPutUChar(&mut bsWr, 0x41)?;
                bsPutUChar(&mut bsWr, 0x59)?;
                bsPutUChar(&mut bsWr, 0x26)?;
                bsPutUChar(&mut bsWr, 0x53)?;
                bsPutUChar(&mut bsWr, 0x59)?;
                Some(bsWr)
            }
        }
    }

    eprintln!("{}: finished", progname);

    Ok(ExitCode::SUCCESS)
}

pub fn main() -> ExitCode {
    let mut it = ::std::env::args_os();

    let program_name = PathBuf::from(it.next().unwrap());
    let opt_in_filename = it.next().map(PathBuf::from);

    eprintln!("bzip2recover 1.0.6: extracts blocks from damaged .bz2 files.");

    let Some(in_filename) = opt_in_filename else {
        eprintln!(
            "{program_name}: usage is `{program_name} damaged_file_name'.",
            program_name = program_name.display()
        );

        eprintln!("\trestrictions on size of recovered file: None");

        return ExitCode::FAILURE;
    };

    match main_help(&program_name, &in_filename) {
        Ok(exit_code) => exit_code,
        Err(error) => match error {
            Error::Reading(io_error) => readError(&program_name, &in_filename, io_error),
            Error::Writing(io_error) => writeError(&program_name, &in_filename, io_error),
            Error::TooManyBlocks(handled) => tooManyBlocks(&program_name, &in_filename, handled),
        },
    }
}
