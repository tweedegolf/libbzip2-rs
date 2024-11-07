#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const BZ_MAX_HANDLED_BLOCKS: usize = 50000;

enum Error {
    Reading(std::io::Error),
    Writing(std::io::Error),
    TooManyBlocks(usize),
}

#[repr(C)]
pub struct BitStream {
    pub handle: File,
    pub buffer: i32,
    pub buffLive: i32,
    pub mode: u8,
}
pub static mut BYTES_OUT: u64 = 0 as libc::c_int as u64;
pub static mut BYTES_IN: u64 = 0 as libc::c_int as u64;

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

unsafe fn bsPutBit(bs: &mut BitStream, bit: i32) -> Result<(), Error> {
    if bs.buffLive == 8 as libc::c_int {
        bs.handle
            .write_all(&[bs.buffer as u8])
            .map_err(Error::Writing)?;
        BYTES_OUT = BYTES_OUT.wrapping_add(1);
        bs.buffLive = 1 as libc::c_int;
        bs.buffer = bit & 0x1 as libc::c_int;
    } else {
        bs.buffer = bs.buffer << 1 as libc::c_int | bit & 0x1 as libc::c_int;
        bs.buffLive += 1;
    }

    Ok(())
}

fn bsGetBit(bs: &mut BitStream) -> Result<i32, Error> {
    if bs.buffLive > 0 as libc::c_int {
        bs.buffLive -= 1;

        Ok(bs.buffer >> bs.buffLive & 0x1 as libc::c_int)
    } else {
        let mut retVal = [0u8];
        let n = bs.handle.read(&mut retVal).map_err(Error::Reading)?;

        // EOF
        if n == 0 {
            return Ok(2);
        }

        bs.buffLive = 7 as libc::c_int;
        bs.buffer = retVal[0] as i32;

        Ok(bs.buffer >> 7 as libc::c_int & 0x1 as libc::c_int)
    }
}

unsafe fn bsClose(mut bs: BitStream) -> Result<(), Error> {
    if bs.mode == b'w' {
        while bs.buffLive < 8 as libc::c_int {
            bs.buffLive += 1;
            bs.buffer <<= 1 as libc::c_int;
        }
        bs.handle
            .write_all(&[bs.buffer as u8])
            .map_err(Error::Writing)?;
        BYTES_OUT = BYTES_OUT.wrapping_add(1);
        bs.handle.flush().map_err(Error::Writing)?;
    }

    Ok(())
}

unsafe fn bsPutUChar(bs: &mut BitStream, c: u8) -> Result<(), Error> {
    let mut i: i32 = 7;
    while i >= 0 {
        bsPutBit(
            bs,
            (c as u32 >> i & 0x1 as libc::c_int as libc::c_uint) as i32,
        )?;
        i -= 1;
    }

    Ok(())
}

unsafe fn bsPutUInt32(bs: &mut BitStream, c: u32) -> Result<(), Error> {
    let mut i: i32 = 31;
    while i >= 0 {
        bsPutBit(bs, (c >> i & 0x1 as libc::c_int as libc::c_uint) as i32)?;
        i -= 1;
    }

    Ok(())
}

pub static mut B_START: [u64; 50000] = [0; 50000];
pub static mut B_END: [u64; 50000] = [0; 50000];
pub static mut RB_START: [u64; 50000] = [0; 50000];
pub static mut RB_END: [u64; 50000] = [0; 50000];
unsafe fn main_0(program_name: &Path, in_filename: &Path) -> Result<ExitCode, Error> {
    let progname = program_name.display();

    if in_filename.as_os_str().len() >= (2000 - 20) as usize {
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
    let mut bitsRead = 0 as libc::c_int as u64;
    let mut buffLo = 0 as libc::c_int as u32;
    let mut buffHi = buffLo;
    let mut currBlock = 0 as libc::c_int;
    B_START[currBlock as usize] = 0 as libc::c_int as u64;
    let mut rbCtr = 0 as libc::c_int;
    loop {
        let b = bsGetBit(&mut bsIn)?;
        bitsRead = bitsRead.wrapping_add(1);
        if b == 2 {
            if bitsRead >= B_START[currBlock as usize]
                && bitsRead.wrapping_sub(B_START[currBlock as usize])
                    >= 40 as libc::c_int as libc::c_ulonglong
            {
                B_END[currBlock as usize] =
                    bitsRead.wrapping_sub(1 as libc::c_int as libc::c_ulonglong);
                if currBlock > 0 as libc::c_int {
                    eprintln!(
                        "   block {} runs from {} to {} (incomplete)",
                        currBlock, B_START[currBlock as usize], B_END[currBlock as usize],
                    );
                }
            }
            break;
        } else {
            buffHi = buffHi << 1 as libc::c_int | buffLo >> 31 as libc::c_int;
            buffLo = buffLo << 1 as libc::c_int | (b & 1 as libc::c_int) as libc::c_uint;
            if (buffHi & 0xffff as libc::c_int as libc::c_uint) == 0x3141 && buffLo == 0x59265359
                || (buffHi & 0xffff as libc::c_int as libc::c_uint) == 0x1772
                    && buffLo == 0x45385090
            {
                if bitsRead > 49 as libc::c_int as libc::c_ulonglong {
                    B_END[currBlock as usize] =
                        bitsRead.wrapping_sub(49 as libc::c_int as libc::c_ulonglong);
                } else {
                    B_END[currBlock as usize] = 0 as libc::c_int as u64;
                }
                if currBlock > 0 as libc::c_int
                    && (B_END[currBlock as usize]).wrapping_sub(B_START[currBlock as usize])
                        >= 130 as libc::c_int as libc::c_ulonglong
                {
                    eprintln!(
                        "   block {} runs from {} to {}",
                        rbCtr + 1 as libc::c_int,
                        B_START[currBlock as usize],
                        B_END[currBlock as usize],
                    );
                    RB_START[rbCtr as usize] = B_START[currBlock as usize];
                    RB_END[rbCtr as usize] = B_END[currBlock as usize];
                    rbCtr += 1;
                }
                if currBlock >= BZ_MAX_HANDLED_BLOCKS as libc::c_int {
                    return Err(Error::TooManyBlocks(BZ_MAX_HANDLED_BLOCKS));
                }
                currBlock += 1;
                B_START[currBlock as usize] = bitsRead;
            }
        }
    }
    bsClose(bsIn)?;
    if rbCtr < 1 as libc::c_int {
        eprintln!("{}: sorry, I couldn't find any block boundaries.", progname);

        return Ok(ExitCode::FAILURE);
    }
    eprintln!("{}: splitting into blocks", progname);

    let Ok(inFile) = std::fs::File::options().read(true).open(in_filename) else {
        eprintln!("{}: can't read `{}'", progname, in_filename.display());

        return Ok(ExitCode::FAILURE);
    };

    bsIn = bsOpenReadStream(inFile);
    let mut blockCRC = 0 as libc::c_int as u32;
    let mut bsWr: Option<BitStream> = None;
    bitsRead = 0 as libc::c_int as u64;
    let mut wrBlock = 0 as libc::c_int;
    loop {
        let b = bsGetBit(&mut bsIn)?;
        if b == 2 as libc::c_int {
            break;
        }
        buffHi = buffHi << 1 as libc::c_int | buffLo >> 31 as libc::c_int;
        buffLo = buffLo << 1 as libc::c_int | (b & 1 as libc::c_int) as libc::c_uint;
        if bitsRead
            == (47 as libc::c_int as libc::c_ulonglong).wrapping_add(RB_START[wrBlock as usize])
        {
            blockCRC = buffHi << 16 as libc::c_int | buffLo >> 16 as libc::c_int;
        }
        if bsWr.is_some()
            && bitsRead >= RB_START[wrBlock as usize]
            && bitsRead <= RB_END[wrBlock as usize]
        {
            bsPutBit(bsWr.as_mut().unwrap(), b)?;
        }
        bitsRead = bitsRead.wrapping_add(1);
        if bitsRead
            == (RB_END[wrBlock as usize]).wrapping_add(1 as libc::c_int as libc::c_ulonglong)
        {
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
        } else if bitsRead == RB_START[wrBlock as usize] {
            // we've been able to open this file, so there must be a file name
            let filename = in_filename.file_name().unwrap();

            let filename = format!("rec{:05}{}", wrBlock + 1, filename.to_string_lossy());

            let out_filename = in_filename.with_file_name(&filename).with_extension("bz2");

            let out_filename_cstr =
                CString::new(out_filename.to_string_lossy().as_bytes()).unwrap();

            eprintln!(
                "   writing block {} to `{}' ...",
                wrBlock + 1 as libc::c_int,
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

            drop(out_filename_cstr);
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
        match core::mem::size_of::<u64>() as libc::c_ulong {
            8 => {
                eprintln!("\trestrictions on size of recovered file: None");
            }
            4 => {
                eprintln!("\trestrictions on size of recovered file: 512 MB");
                eprintln!("\tto circumvent, recompile with u64 as an\n\tunsigned 64-bit int.");
            }
            _ => {
                eprintln!("\tsizeof::<u64> is not 4 or 8 -- configuration error.");
            }
        }

        return ExitCode::FAILURE;
    };

    match unsafe { main_0(&program_name, &in_filename) } {
        Ok(exit_code) => exit_code,
        Err(error) => match error {
            Error::Reading(io_error) => readError(&program_name, &in_filename, io_error),
            Error::Writing(io_error) => writeError(&program_name, &in_filename, io_error),
            Error::TooManyBlocks(handled) => tooManyBlocks(&program_name, &in_filename, handled),
        },
    }
}
