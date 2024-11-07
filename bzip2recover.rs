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

impl BitStream {
    fn open_read_stream(stream: File) -> Self {
        Self {
            handle: stream,
            buffer: 0,
            buffLive: 0,
            mode: b'r',
        }
    }

    fn open_write_stream(stream: File) -> Self {
        Self {
            handle: stream,
            buffer: 0,
            buffLive: 0,
            mode: b'w',
        }
    }

    fn close(mut self) -> Result<(), Error> {
        if self.mode == b'w' {
            while self.buffLive < 8 {
                self.buffLive += 1;
                self.buffer <<= 1;
            }
            self.handle
                .write_all(&[self.buffer as u8])
                .map_err(Error::Writing)?;
            self.handle.flush().map_err(Error::Writing)?;
        }

        Ok(())
    }

    fn get_bit(&mut self) -> Result<Option<bool>, Error> {
        if self.buffLive > 0 {
            self.buffLive -= 1;

            Ok(Some(self.buffer >> self.buffLive & 0x1 != 0))
        } else {
            let mut retVal = [0u8];
            let n = self.handle.read(&mut retVal).map_err(Error::Reading)?;

            // EOF
            if n == 0 {
                return Ok(None);
            }

            self.buffLive = 7;
            self.buffer = retVal[0] as i32;

            Ok(Some(self.buffer >> 7 & 0x1 != 0))
        }
    }

    fn put_bit(&mut self, bit: i32) -> Result<(), Error> {
        if self.buffLive == 8 {
            self.handle
                .write_all(&[self.buffer as u8])
                .map_err(Error::Writing)?;
            self.buffLive = 1;
            self.buffer = bit & 0x1;
        } else {
            self.buffer = self.buffer << 1 | bit & 0x1;
            self.buffLive += 1;
        }

        Ok(())
    }

    fn put_u8(&mut self, c: u8) -> Result<(), Error> {
        for i in (0..8).rev() {
            self.put_bit((c as u32 >> i & 0x1) as i32)?;
        }

        Ok(())
    }

    fn put_u32(&mut self, c: u32) -> Result<(), Error> {
        for i in (0..32).rev() {
            self.put_bit((c >> i & 0x1) as i32)?;
        }

        Ok(())
    }
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

fn main_help(program_name: &Path, in_filename: &Path) -> Result<ExitCode, Error> {
    let mut b_start = vec![0u64; BZ_MAX_HANDLED_BLOCKS];
    let mut b_end = vec![0u64; BZ_MAX_HANDLED_BLOCKS];
    let mut rb_start = vec![0u64; BZ_MAX_HANDLED_BLOCKS];
    let mut rb_end = vec![0u64; BZ_MAX_HANDLED_BLOCKS];

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

    let mut bsIn = BitStream::open_read_stream(inFile);
    eprintln!("{}: searching for block boundaries ...", progname);

    let mut bitsRead: u64 = 0;
    let mut buffLo: u32 = 0;
    let mut buffHi = buffLo;
    let mut currBlock = 0;
    b_start[currBlock] = 0;
    let mut rbCtr = 0;

    loop {
        let b = bsIn.get_bit()?;
        bitsRead = bitsRead.wrapping_add(1);
        match b {
            None => {
                if bitsRead >= b_start[currBlock] && bitsRead.wrapping_sub(b_start[currBlock]) >= 40
                {
                    b_end[currBlock] = bitsRead.wrapping_sub(1);
                    if currBlock > 0 {
                        eprintln!(
                            "   block {} runs from {} to {} (incomplete)",
                            currBlock, b_start[currBlock], b_end[currBlock],
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
                    b_end[currBlock] = if bitsRead > 49 {
                        bitsRead.wrapping_sub(49)
                    } else {
                        0
                    };

                    if currBlock > 0 && (b_end[currBlock]).wrapping_sub(b_start[currBlock]) >= 130 {
                        eprintln!(
                            "   block {} runs from {} to {}",
                            rbCtr + 1,
                            b_start[currBlock],
                            b_end[currBlock],
                        );
                        rb_start[rbCtr] = b_start[currBlock];
                        rb_end[rbCtr] = b_end[currBlock];
                        rbCtr += 1;
                    }
                    if currBlock >= BZ_MAX_HANDLED_BLOCKS {
                        return Err(Error::TooManyBlocks(BZ_MAX_HANDLED_BLOCKS));
                    }
                    currBlock += 1;
                    b_start[currBlock] = bitsRead;
                }
            }
        }
    }

    bsIn.close()?;

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
    bsIn = BitStream::open_read_stream(inFile);

    let mut blockCRC: u32 = 0;
    let mut bsWr: Option<BitStream> = None;
    let mut wrBlock = 0;

    bitsRead = 0;

    loop {
        let Some(b) = bsIn.get_bit()? else {
            // EOF
            break;
        };

        buffHi = buffHi << 1 | buffLo >> 31;
        buffLo = buffLo << 1 | b as u32;

        if bitsRead == 47u64.wrapping_add(rb_start[wrBlock]) {
            blockCRC = buffHi << 16 | buffLo >> 16;
        }
        if bitsRead >= rb_start[wrBlock] && bitsRead <= rb_end[wrBlock] {
            if let Some(bsWr) = bsWr.as_mut() {
                bsWr.put_bit(b as i32)?;
            }
        }
        bitsRead = bitsRead.wrapping_add(1);
        if bitsRead == (rb_end[wrBlock]).wrapping_add(1) {
            if let Some(mut bsWr) = bsWr.take() {
                bsWr.put_u8(0x17)?;
                bsWr.put_u8(0x72)?;
                bsWr.put_u8(0x45)?;
                bsWr.put_u8(0x38)?;
                bsWr.put_u8(0x50)?;
                bsWr.put_u8(0x90)?;
                bsWr.put_u32(blockCRC)?;
                bsWr.close()?;
            }

            if wrBlock >= rbCtr {
                break;
            }
            wrBlock += 1;
        } else if bitsRead == rb_start[wrBlock] {
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
                let mut bsWr = BitStream::open_write_stream(outFile);
                bsWr.put_u8(0x42)?;
                bsWr.put_u8(0x5a)?;
                bsWr.put_u8(0x68)?;
                bsWr.put_u8(0x30 + 9)?;
                bsWr.put_u8(0x31)?;
                bsWr.put_u8(0x41)?;
                bsWr.put_u8(0x59)?;
                bsWr.put_u8(0x26)?;
                bsWr.put_u8(0x53)?;
                bsWr.put_u8(0x59)?;
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
