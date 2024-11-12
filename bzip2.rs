#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::ffi::{c_char, CStr, CString, OsStr};
use std::io::IsTerminal;
use std::mem::zeroed;
use std::path::{Path, PathBuf};
use std::ptr;

use libbzip2_rs_sys::{
    BZ2_bzRead, BZ2_bzReadClose, BZ2_bzReadGetUnused, BZ2_bzReadOpen, BZ2_bzWrite,
    BZ2_bzWriteClose64, BZ2_bzWriteOpen, BZ2_bzlibVersion,
};

use libc::{
    _exit, close, exit, fclose, fdopen, ferror, fflush, fgetc, fileno, fopen, fread, fwrite, open,
    perror, remove, rewind, signal, stat, strcat, strcmp, strlen, strncpy, ungetc, utimbuf, write,
    FILE,
};
extern "C" {
    static mut stdin: *mut FILE;
    static mut stdout: *mut FILE;
}
type Bool = libc::c_uchar;

type IntNative = libc::c_int;

static mut verbosity: i32 = 0;
static mut keep_input_files: bool = false;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DecompressMode {
    Fast = 0,
    Small = 1,
}

static mut decompress_mode: DecompressMode = DecompressMode::Fast;

static mut delete_output_on_interrupt: bool = false;
static mut force_overwrite: bool = false;
static mut test_fails_exists: bool = false;
static mut unz_fails_exist: bool = false;
static mut noisy: bool = false;
static mut numFileNames: i32 = 0;
static mut numFilesProcessed: i32 = 0;
static mut blockSize100k: i32 = 0;
static mut exitValue: i32 = 0;

/// source modes
///
/// - F = file
/// - I = stdin
/// - O = stdout
#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceMode {
    I2O = 1,
    F2O = 2,
    F2F = 3,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OperationMode {
    Zip = 1,
    Unzip = 2,
    Test = 3,
}

const FILE_NAME_LEN: usize = 1034;

static mut opMode: OperationMode = OperationMode::Zip;
static mut srcMode: SourceMode = SourceMode::I2O;

static mut longestFileName: i32 = 0;
static mut inName: [c_char; FILE_NAME_LEN] = [0; FILE_NAME_LEN];
static mut outName: [c_char; FILE_NAME_LEN] = [0; FILE_NAME_LEN];
static mut progName: *mut c_char = ptr::null_mut();
static mut progNameReally: [c_char; FILE_NAME_LEN] = [0; FILE_NAME_LEN];

// this should eventually be removed and just passed down into functions from the root
fn get_program_name() -> PathBuf {
    let program_path: PathBuf = std::env::args_os().next().unwrap().into();
    PathBuf::from(program_path.file_name().unwrap())
}

static mut outputHandleJustInCase: *mut FILE = ptr::null_mut();
static mut workFactor: i32 = 0;

/// Strictly for compatibility with the original bzip2 output
fn display_last_os_error() -> String {
    let mut error = std::io::Error::last_os_error().to_string();

    // now strip off the ` (os error x)` part
    if let Some(index) = error.find(" (os error") {
        error.truncate(index);
    }

    // the C version tries to open a file to check whether a path exists
    error.replace("Bad address", "No such file or directory")
}

unsafe fn myfeof(f: *mut FILE) -> Bool {
    let c: i32 = fgetc(f);
    if c == -1 as libc::c_int {
        return 1 as Bool;
    }
    ungetc(c, f);
    0 as Bool
}

unsafe fn compressStream(stream: *mut FILE, zStream: *mut FILE) {
    let mut ibuf: [u8; 5000] = [0; 5000];
    let mut nbytes_in_lo32: u32 = 0;
    let mut nbytes_in_hi32: u32 = 0;
    let mut nbytes_out_lo32: u32 = 0;
    let mut nbytes_out_hi32: u32 = 0;
    let mut bzerr: i32 = 0;
    let mut bzerr_dummy: i32 = 0;
    let mut ret: i32;

    // TODO set files to binary mode?

    if ferror(stream) != 0 {
        // diverges
        ioError()
    }
    if ferror(zStream) != 0 {
        // diverges
        ioError()
    }

    let bzf = BZ2_bzWriteOpen(&mut bzerr, zStream, blockSize100k, verbosity, workFactor);

    'errhandler: {
        if bzerr != libbzip2_rs_sys::BZ_OK {
            break 'errhandler;
        }

        if verbosity >= 2 {
            eprintln!();
        }

        loop {
            if myfeof(stream) != 0 {
                break;
            }

            let nIbuf = fread(
                ibuf.as_mut_ptr() as *mut libc::c_void,
                core::mem::size_of::<u8>() as libc::size_t,
                5000 as libc::c_int as libc::size_t,
                stream,
            ) as i32;
            if ferror(stream) != 0 {
                // diverges
                ioError()
            }
            if nIbuf > 0 as libc::c_int {
                BZ2_bzWrite(
                    &mut bzerr,
                    bzf,
                    ibuf.as_mut_ptr() as *mut libc::c_void,
                    nIbuf,
                );
            }
            if bzerr != libbzip2_rs_sys::BZ_OK {
                break 'errhandler;
            }
        }

        BZ2_bzWriteClose64(
            &mut bzerr,
            bzf,
            0 as libc::c_int,
            &mut nbytes_in_lo32,
            &mut nbytes_in_hi32,
            &mut nbytes_out_lo32,
            &mut nbytes_out_hi32,
        );

        if bzerr != libbzip2_rs_sys::BZ_OK {
            break 'errhandler;
        }

        if (ferror(zStream)) != 0 {
            // diverges
            ioError()
        }
        ret = fflush(zStream);
        if ret == libc::EOF {
            // diverges
            ioError()
        }

        if zStream != stdout {
            let fd = fileno(zStream);
            if fd < 0 {
                // diverges
                ioError()
            }
            applySavedFileAttrToOutputFile(fd);
            ret = fclose(zStream);
            outputHandleJustInCase = core::ptr::null_mut();
            if ret == libc::EOF {
                // diverges
                ioError()
            }
        }

        outputHandleJustInCase = core::ptr::null_mut();
        if ferror(stream) != 0 {
            // diverges
            ioError()
        }
        ret = fclose(stream);
        if ret == libc::EOF {
            // diverges
            ioError()
        }

        if verbosity >= 1 {
            if nbytes_in_lo32 == 0 && nbytes_in_hi32 == 0 {
                eprintln!(" no data compressed.");
            } else {
                let bytes_in = (nbytes_in_hi32 as u64) << 32 | nbytes_in_lo32 as u64;
                let bytes_out = (nbytes_out_hi32 as u64) << 32 | nbytes_out_lo32 as u64;

                let nbytes_in_d = bytes_in as f64;
                let nbytes_out_d = bytes_out as f64;

                eprintln!(
                    "{:6.3}:1, {:6.3} bits/byte, {:5.2}% saved, {} in, {} out.",
                    nbytes_in_d / nbytes_out_d,
                    8.0 * nbytes_out_d / nbytes_in_d,
                    100.0 * (1.0 - nbytes_out_d / nbytes_in_d),
                    bytes_in,
                    bytes_out,
                );
            }
        }

        return;
    }

    // errhandler:

    BZ2_bzWriteClose64(
        &mut bzerr_dummy,
        bzf,
        1,
        &mut nbytes_in_lo32,
        &mut nbytes_in_hi32,
        &mut nbytes_out_lo32,
        &mut nbytes_out_hi32,
    );

    match bzerr {
        libbzip2_rs_sys::BZ_CONFIG_ERROR => configError(),
        libbzip2_rs_sys::BZ_MEM_ERROR => outOfMemory(),
        libbzip2_rs_sys::BZ_IO_ERROR => ioError(),
        _ => panic_str("compress:unexpected error"),
    }
}

unsafe fn uncompressStream(zStream: *mut FILE, stream: *mut FILE) -> bool {
    let mut bzf = std::ptr::null_mut();
    let mut bzerr: i32 = 0;
    let mut bzerr_dummy: i32 = 0;
    let mut ret: i32;
    let mut nread: i32;
    let mut obuf: [u8; 5000] = [0; 5000];
    let mut unused: [u8; 5000] = [0; 5000];
    let mut unusedTmpV: *mut libc::c_void = std::ptr::null_mut::<libc::c_void>();

    let mut nUnused: libc::c_int = 0;
    let mut streamNo: libc::c_int = 0;

    enum State {
        Standard,
        CloseOk,
        TryCat,
        ErrHandler,
    }

    let mut state = State::Standard;

    // TODO: set the file into binary mode?

    if ferror(stream) != 0 || ferror(zStream) != 0 {
        // diverges
        ioError()
    }

    'outer: loop {
        match state {
            State::Standard => loop {
                bzf = BZ2_bzReadOpen(
                    &mut bzerr,
                    zStream,
                    verbosity,
                    decompress_mode as libc::c_int,
                    unused.as_mut_ptr() as *mut libc::c_void,
                    nUnused,
                );
                if bzf.is_null() || bzerr != 0 as libc::c_int {
                    state = State::ErrHandler;
                    continue 'outer;
                }
                streamNo += 1;

                while bzerr == 0 as libc::c_int {
                    nread = BZ2_bzRead(
                        &mut bzerr,
                        bzf,
                        obuf.as_mut_ptr() as *mut libc::c_void,
                        5000 as libc::c_int,
                    );
                    if bzerr == libbzip2_rs_sys::BZ_DATA_ERROR_MAGIC {
                        state = State::TryCat;
                        continue 'outer;
                    }
                    if (bzerr == libbzip2_rs_sys::BZ_OK || bzerr == libbzip2_rs_sys::BZ_STREAM_END)
                        && nread > 0
                    {
                        fwrite(
                            obuf.as_mut_ptr() as *const libc::c_void,
                            core::mem::size_of::<u8>() as libc::size_t,
                            nread as libc::size_t,
                            stream,
                        );
                    }
                    if ferror(stream) != 0 {
                        // diverges
                        ioError()
                    }
                }

                if bzerr != libbzip2_rs_sys::BZ_STREAM_END {
                    state = State::ErrHandler;
                    continue 'outer;
                }

                BZ2_bzReadGetUnused(&mut bzerr, bzf, &mut unusedTmpV, &mut nUnused);
                if bzerr != libbzip2_rs_sys::BZ_OK {
                    // diverges
                    panic_str("decompress:bzReadGetUnused")
                }

                let unusedTmp = unusedTmpV as *mut u8;
                for i in 0..nUnused {
                    unused[i as usize] = *unusedTmp.offset(i as isize);
                }

                BZ2_bzReadClose(&mut bzerr, bzf);
                if bzerr != libbzip2_rs_sys::BZ_OK {
                    // diverges
                    panic_str("decompress:bzReadGetUnused")
                }

                if nUnused == 0 && myfeof(zStream) != 0 {
                    state = State::CloseOk;
                    continue 'outer;
                }
            },
            State::CloseOk => {
                if ferror(zStream) != 0 {
                    // diverges
                    ioError()
                }

                if stream != stdout {
                    let fd: i32 = fileno(stream);
                    if fd < 0 {
                        // diverges
                        ioError()
                    }

                    applySavedFileAttrToOutputFile(fd);
                }

                ret = fclose(zStream);
                if ret == libc::EOF {
                    ioError()
                }

                if ferror(stream) != 0 {
                    // diverges
                    ioError()
                }

                ret = fflush(stream);
                if ret != 0 {
                    // diverges
                    ioError()
                }

                if stream != stdout {
                    ret = fclose(stream);
                    outputHandleJustInCase = core::ptr::null_mut();
                    if ret == libc::EOF {
                        ioError()
                    }
                }
                outputHandleJustInCase = core::ptr::null_mut();

                if verbosity >= 2 {
                    eprint!("\n    ");
                }

                return true;
            }
            State::TryCat => {
                if force_overwrite {
                    rewind(zStream);
                    loop {
                        if myfeof(zStream) != 0 {
                            break;
                        }
                        nread = fread(
                            obuf.as_mut_ptr() as *mut libc::c_void,
                            core::mem::size_of::<u8>() as libc::size_t,
                            5000 as libc::c_int as libc::size_t,
                            zStream,
                        ) as i32;
                        if ferror(zStream) != 0 {
                            // diverges
                            ioError()
                        }
                        if nread > 0 {
                            fwrite(
                                obuf.as_mut_ptr() as *const libc::c_void,
                                core::mem::size_of::<u8>() as libc::size_t,
                                nread as libc::size_t,
                                stream,
                            );
                        }
                        if ferror(stream) != 0 {
                            // diverges
                            ioError()
                        }
                    }

                    state = State::CloseOk;
                    continue 'outer;
                } else {
                    state = State::ErrHandler;
                    continue 'outer;
                }
            }
            State::ErrHandler => {
                BZ2_bzReadClose(&mut bzerr_dummy, bzf);

                match bzerr {
                    libbzip2_rs_sys::BZ_CONFIG_ERROR => configError(),
                    libbzip2_rs_sys::BZ_IO_ERROR => ioError(),
                    libbzip2_rs_sys::BZ_DATA_ERROR => crcError(),
                    libbzip2_rs_sys::BZ_MEM_ERROR => outOfMemory(),
                    libbzip2_rs_sys::BZ_UNEXPECTED_EOF => compressedStreamEOF(),
                    libbzip2_rs_sys::BZ_DATA_ERROR_MAGIC => {
                        if zStream != stdin {
                            fclose(zStream);
                        }
                        if stream != stdout {
                            fclose(stream);
                        }
                        if streamNo == 1 {
                            return false;
                        } else {
                            if noisy {
                                eprintln!(
                                    "\n{}: {}: trailing garbage after EOF ignored",
                                    CStr::from_ptr(progName).to_string_lossy(),
                                    CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
                                );
                            }
                            return true;
                        }
                    }
                    _ => panic_str("decompress:unexpected error"),
                }
            }
        }
    }
}

unsafe fn testStream(zStream: *mut FILE) -> bool {
    let mut bzf: *mut libc::c_void;
    let mut bzerr: i32 = 0;
    let mut i: i32;
    let mut obuf: [u8; 5000] = [0; 5000];
    let mut unused: [u8; 5000] = [0; 5000];

    let mut nUnused = 0;
    let mut streamNo = 0;

    'errhandler: {
        loop {
            bzf = BZ2_bzReadOpen(
                &mut bzerr,
                zStream,
                verbosity,
                decompress_mode as libc::c_int,
                unused.as_mut_ptr() as *mut libc::c_void,
                nUnused,
            );
            if bzf.is_null() || bzerr != 0 {
                // diverges
                ioError()
            }

            // there might be multiple files if the input stream is stdin
            streamNo += 1;

            while bzerr == 0 {
                BZ2_bzRead(
                    &mut bzerr,
                    bzf,
                    obuf.as_mut_ptr() as *mut libc::c_void,
                    5000,
                );
                if bzerr == libbzip2_rs_sys::BZ_DATA_ERROR_MAGIC {
                    break 'errhandler;
                }
            }

            if bzerr != libbzip2_rs_sys::BZ_STREAM_END {
                break 'errhandler;
            }

            let mut unusedTmpV = std::ptr::null_mut();
            BZ2_bzReadGetUnused(&mut bzerr, bzf, &mut unusedTmpV, &mut nUnused);
            if bzerr != libbzip2_rs_sys::BZ_OK {
                panic_str("test:bzReadGetUnused");
            }

            let unusedTmp = unusedTmpV as *mut u8;
            i = 0 as libc::c_int;
            while i < nUnused {
                unused[i as usize] = *unusedTmp.offset(i as isize);
                i += 1;
            }

            BZ2_bzReadClose(&mut bzerr, bzf);
            if bzerr != libbzip2_rs_sys::BZ_OK {
                panic_str("test:bzReadClose");
            }
            if nUnused == 0 && myfeof(zStream) != 0 {
                break;
            }
        }

        if ferror(zStream) != 0 {
            ioError() // diverges
        }
        if fclose(zStream) == libc::EOF {
            ioError() // diverges
        }

        if verbosity >= 2 {
            eprintln!()
        }

        return true;
    }

    // errhandler:

    BZ2_bzReadClose(&mut 0, bzf);
    if verbosity == 0 {
        eprintln!(
            "{}: {}: ",
            get_program_name().display(),
            CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
        );
    }
    match bzerr {
        libbzip2_rs_sys::BZ_CONFIG_ERROR => configError(),
        libbzip2_rs_sys::BZ_IO_ERROR => ioError(),
        libbzip2_rs_sys::BZ_DATA_ERROR => {
            eprintln!("data integrity (CRC) error in data");
            false
        }
        libbzip2_rs_sys::BZ_MEM_ERROR => outOfMemory(),
        libbzip2_rs_sys::BZ_UNEXPECTED_EOF => {
            eprintln!("file ends unexpectedly");
            false
        }
        libbzip2_rs_sys::BZ_DATA_ERROR_MAGIC => {
            if zStream != stdin {
                fclose(zStream);
            }
            if streamNo == 1 {
                eprintln!("bad magic number (file not created by bzip2)");
                false
            } else {
                if noisy {
                    eprintln!("trailing garbage after EOF ignored");
                }
                true
            }
        }
        _ => panic_str("test:unexpected error"),
    }
}

unsafe fn setExit(v: i32) {
    if v > exitValue {
        exitValue = v;
    }
}

unsafe fn cadvise() {
    if noisy {
        eprint!(concat!(
            "\n",
            "It is possible that the compressed file(s) have become corrupted.\n",
            "You can use the -tvv option to test integrity of such files.\n",
            "\n",
            "You can use the `bzip2recover' program to attempt to recover\n",
            "data from undamaged sections of corrupted files.\n",
            "\n",
        ));
    }
}

unsafe fn showFileNames() {
    if noisy {
        eprintln!(
            "\tInput file = {}, output file = {}",
            CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
            CStr::from_ptr(outName.as_ptr()).to_string_lossy(),
        );
    }
}

unsafe fn cleanUpAndFail(ec: i32) -> ! {
    let program_name = CStr::from_ptr(progName).to_string_lossy();

    let mut statBuf: stat = zeroed();
    if srcMode == SourceMode::F2F && opMode != OperationMode::Test && delete_output_on_interrupt {
        if stat(inName.as_mut_ptr(), &mut statBuf) == 0 {
            if noisy {
                eprintln!(
                    "{}: Deleting output file {}, if it exists.",
                    program_name,
                    CStr::from_ptr(outName.as_ptr()).to_string_lossy(),
                );
            }
            if !outputHandleJustInCase.is_null() {
                fclose(outputHandleJustInCase);
            }
            if remove(outName.as_mut_ptr()) != 0 {
                eprintln!(
                    "{}: WARNING: deletion of output file (apparently) failed.",
                    program_name,
                );
            }
        } else {
            eprintln!(
                "{}: WARNING: deletion of output file suppressed",
                program_name,
            );
            eprintln!(
                "{}:    since input file no longer exists.  Output file",
                program_name,
            );
            eprintln!(
                "{}:    `{}' may be incomplete.",
                program_name,
                CStr::from_ptr(outName.as_ptr()).to_string_lossy(),
            );
            eprintln!(
                "{}:    I suggest doing an integrity test (bzip2 -tv) of it.",
                program_name,
            );
        }
    }
    if noisy as libc::c_int != 0
        && numFileNames > 0 as libc::c_int
        && numFilesProcessed < numFileNames
    {
        eprint!(
            concat!(
                "{}: WARNING: some files have not been processed:\n",
                "{}:    {} specified on command line, {} not processed yet.\n",
                "\n",
            ),
            program_name,
            program_name,
            numFileNames,
            numFileNames - numFilesProcessed,
        );
    }
    setExit(ec);
    exit(exitValue);
}

unsafe fn panic(s: *const c_char) -> ! {
    panic_str(&CStr::from_ptr(s).to_string_lossy())
}

unsafe fn panic_str(s: &str) -> ! {
    eprint!(
        concat!(
            "\n",
            "{}: PANIC -- internal consistency error:\n",
            "\t{}\n",
            "\tThis is a BUG.  Please report it at:\n",
            "\thttps://github.com/trifectatechfoundation/libbzip2-rs/issues\n"
        ),
        CStr::from_ptr(progName).to_string_lossy(),
        s,
    );
    showFileNames();
    cleanUpAndFail(3 as libc::c_int);
}

unsafe fn crcError() -> ! {
    eprintln!(
        "\n{}: Data integrity error when decompressing.",
        CStr::from_ptr(progName).to_string_lossy(),
    );
    showFileNames();
    cadvise();
    cleanUpAndFail(2 as libc::c_int);
}

unsafe fn compressedStreamEOF() -> ! {
    if noisy {
        eprint!(
            concat!(
                "\n",
                "{}: Compressed file ends unexpectedly;\n",
                "\tperhaps it is corrupted?  *Possible* reason follows.\n"
            ),
            CStr::from_ptr(progName).to_string_lossy(),
        );
        perror(progName);
        showFileNames();
        cadvise();
    }
    cleanUpAndFail(2 as libc::c_int);
}
unsafe fn ioError() -> ! {
    eprintln!(
        "\n{}: I/O or other error, bailing out.  Possible reason follows.",
        CStr::from_ptr(progName).to_string_lossy(),
    );
    perror(progName);
    showFileNames();
    cleanUpAndFail(1 as libc::c_int);
}

unsafe extern "C" fn mySignalCatcher(_: libc::c_int) {
    eprintln!(
        "\n{}: Control-C or similar caught, quitting.",
        CStr::from_ptr(progName).to_string_lossy(),
    );
    cleanUpAndFail(1 as libc::c_int);
}

unsafe extern "C" fn mySIGSEGVorSIGBUScatcher(_: libc::c_int) {
    const COMPRESS_MSG: &str = concat!(
        ": Caught a SIGSEGV or SIGBUS whilst compressing.\n",
        "\n",
        "   Possible causes are (most likely first):\n",
        "   (1) This computer has unreliable memory or cache hardware\n",
        "       (a surprisingly common problem; try a different machine.)\n",
        "   (2) A bug in the compiler used to create this executable\n",
        "       (unlikely, if you didn't compile bzip2 yourself.)\n",
        "   (3) A real bug in bzip2-rs -- I hope this should never be the case.\n",
        "   The user's manual, Section 4.3, has more info on (1) and (2).\n",
        "   \n",
        "   If you suspect this is a bug in bzip2-rs, or are unsure about (1)\n",
        "   or (2), report it at: https://github.com/trifectatechfoundation/libbzip2-rs/issues\n",
        "   Section 4.3 of the user's manual describes the info a useful\n",
        "   bug report should have.  If the manual is available on your\n",
        "   system, please try and read it before mailing me.  If you don't\n",
        "   have the manual or can't be bothered to read it, mail me anyway.\n",
        "\n",
    );

    const DECOMPRESS_MSG: &str = concat!(
        ": Caught a SIGSEGV or SIGBUS whilst decompressing.\n",
        "\n",
        "   Possible causes are (most likely first):\n",
        "   (1) The compressed data is corrupted, and bzip2's usual checks\n",
        "       failed to detect this.  Try bzip2 -tvv my_file.bz2.\n",
        "   (2) This computer has unreliable memory or cache hardware\n",
        "       (a surprisingly common problem; try a different machine.)\n",
        "   (3) A bug in the compiler used to create this executable\n",
        "       (unlikely, if you didn't compile bzip2 yourself.)\n",
        "   (4) A real bug in bzip2-rs -- I hope this should never be the case.\n",
        "   The user's manual, Section 4.3, has more info on (2) and (3).\n",
        "   \n",
        "   If you suspect this is a bug in bzip2-rs, or are unsure about (2)\n",
        "   or (3), report it at: https://github.com/trifectatechfoundation/libbzip2-rs/issues\n",
        "   Section 4.3 of the user's manual describes the info a useful\n",
        "   bug report should have.  If the manual is available on your\n",
        "   system, please try and read it before mailing me.  If you don't\n",
        "   have the manual or can't be bothered to read it, mail me anyway.\n",
        "\n",
    );

    let msg = match opMode {
        OperationMode::Zip => COMPRESS_MSG,
        _ => DECOMPRESS_MSG,
    };

    let write_str = |s: &str| {
        write(
            libc::STDERR_FILENO,
            s.as_ptr() as *const libc::c_void,
            s.len() as _,
        )
    };

    write_str("\n");
    write(
        libc::STDERR_FILENO,
        progName as *const libc::c_void,
        strlen(progName) as _,
    );
    write_str(msg);

    write_str("\tInput file = ");
    write(
        libc::STDERR_FILENO,
        inName.as_mut_ptr() as *const libc::c_void,
        strlen(inName.as_mut_ptr()) as _,
    );
    write_str("\n");

    write_str("\tOutput file = ");
    write(
        libc::STDERR_FILENO,
        outName.as_mut_ptr() as *const libc::c_void,
        strlen(outName.as_mut_ptr()) as _,
    );
    write_str("\n");

    match opMode {
        OperationMode::Zip => setExit(3),
        _ => setExit(2),
    }

    _exit(exitValue);
}

unsafe fn outOfMemory() -> ! {
    eprintln!(
        "\n{}: couldn't allocate enough memory",
        CStr::from_ptr(progName).to_string_lossy(),
    );
    showFileNames();
    cleanUpAndFail(1 as libc::c_int);
}

unsafe fn configError() -> ! {
    const MSG: &str = concat!(
        "bzip2: I'm not configured correctly for this platform!\n",
        "\tI require Int32, Int16 and Char to have sizes\n",
        "\tof 4, 2 and 1 bytes to run properly, and they don't.\n",
        "\tProbably you can fix this by defining them correctly,\n",
        "\tand recompiling.  Bye!\n",
    );
    eprint!("{}", MSG);
    setExit(3 as libc::c_int);
    exit(exitValue);
}

unsafe fn pad(s: *mut c_char) {
    if strlen(s) as i32 >= longestFileName {
        return;
    }

    for _ in 1..=longestFileName - strlen(s) as i32 {
        eprint!(" ");
    }
}
unsafe fn copyFileName(to: *mut c_char, from: *const c_char) {
    if strlen(from) > (FILE_NAME_LEN - 10) {
        eprint!(
            concat!(
                "bzip2: file name\n",
                "`{}'\n",
                "is suspiciously (more than {} chars) long.\n",
                "Try using a reasonable file name instead.  Sorry! :-)\n",
            ),
            CStr::from_ptr(from).to_string_lossy(),
            FILE_NAME_LEN - 10
        );
        setExit(1 as libc::c_int);
        exit(exitValue);
    }
    strncpy(to, from, FILE_NAME_LEN - 10);
    *to.wrapping_add(FILE_NAME_LEN - 10) = '\0' as i32 as c_char;
}
unsafe fn fileExists(name: *mut c_char) -> Bool {
    let tmp: *mut FILE = fopen(name, b"rb\0" as *const u8 as *const libc::c_char);
    let exists: Bool = (tmp != std::ptr::null_mut::<libc::c_void>() as *mut FILE) as Bool;
    if !tmp.is_null() {
        fclose(tmp);
    }
    exists
}
unsafe fn fopen_output_safely(name: *mut c_char, mode: *const libc::c_char) -> *mut FILE {
    let fh = open(
        name,
        0o1 as libc::c_int | 0o100 as libc::c_int | 0o200 as libc::c_int,
        0o200 as libc::c_int | 0o400 as libc::c_int,
    );
    if fh == -1 as libc::c_int {
        return std::ptr::null_mut::<FILE>();
    }
    let fp = fdopen(fh, mode);
    if fp.is_null() {
        close(fh);
    }
    fp
}

fn not_a_standard_file(path: &Path) -> bool {
    let Ok(metadata) = path.symlink_metadata() else {
        return true;
    };

    !metadata.is_file()
}

#[cfg(unix)]
unsafe fn notAStandardFile(name: *mut c_char) -> Bool {
    let mut statBuf: stat = zeroed();
    let i = libc::lstat(name, &mut statBuf);
    if i != 0 as libc::c_int {
        return 1 as Bool;
    }
    if statBuf.st_mode & 0o170000 == 0o100000 {
        return 0 as Bool;
    }
    1 as Bool
}
#[cfg(not(unix))]
unsafe fn notAStandardFile(name: *mut c_char) -> Bool {
    let Ok(name) = std::ffi::CStr::from_ptr(name).to_str() else {
        return 1;
    };
    let Ok(metadata) = std::path::Path::new(name).symlink_metadata() else {
        return 1;
    };

    if metadata.file_type().is_file() {
        0
    } else {
        1
    }
}
#[cfg(unix)]
unsafe fn countHardLinks(name: *mut c_char) -> i32 {
    let mut statBuf: stat = zeroed();
    let i = libc::lstat(name, &mut statBuf);
    if i != 0 as libc::c_int {
        return 0 as libc::c_int;
    }
    (statBuf.st_nlink).wrapping_sub(1) as i32
}
#[cfg(not(unix))]
unsafe fn countHardLinks(name: *mut c_char) -> i32 {
    0 // FIXME
}

#[cfg(unix)]
fn count_hardlinks(path: &Path) -> u64 {
    use std::os::linux::fs::MetadataExt;

    let Ok(metadata) = path.metadata() else {
        return 0;
    };

    metadata.st_nlink().saturating_sub(1)
}

#[cfg(not(unix))]
unsafe fn count_hardlinks(path: &Path) -> u64 {
    0 // FIXME
}

static mut fileMetaInfo: stat = unsafe { zeroed() };
unsafe fn saveInputFileMetaInfo(srcName: *mut c_char) {
    let retVal = stat(srcName, core::ptr::addr_of_mut!(fileMetaInfo));
    if retVal != 0 as libc::c_int {
        ioError();
    }
}
#[cfg(unix)]
unsafe fn applySavedTimeInfoToOutputFile(dstName: *mut c_char) {
    let mut uTimBuf: utimbuf = utimbuf {
        actime: 0,
        modtime: 0,
    };
    uTimBuf.actime = fileMetaInfo.st_atime;
    uTimBuf.modtime = fileMetaInfo.st_mtime;
    let retVal = libc::utime(dstName, &uTimBuf);
    if retVal != 0 as libc::c_int {
        ioError();
    }
}
#[cfg(not(unix))]
unsafe fn applySavedTimeInfoToOutputFile(_dstName: *mut c_char) {}
#[cfg(unix)]
unsafe fn applySavedFileAttrToOutputFile(fd: IntNative) {
    let retVal = libc::fchmod(fd, fileMetaInfo.st_mode);
    if retVal != 0 as libc::c_int {
        ioError();
    }
    libc::fchown(fd, fileMetaInfo.st_uid, fileMetaInfo.st_gid);
}
#[cfg(not(unix))]
unsafe fn applySavedFileAttrToOutputFile(_fd: IntNative) {}

#[cfg(unix)]
unsafe fn contains_dubious_chars(_: *mut c_char) -> bool {
    // On unix, files can contain any characters and the file expansion is performed by the shell.
    false
}

#[cfg(not(unix))]
unsafe fn contains_dubious_chars(ptr: *mut c_char) -> bool {
    // On non-unix (Win* platforms), wildcard characters are not allowed in filenames.
    CStr::from_ptr(ptr)
        .to_bytes()
        .iter()
        .any(|c| *c == b'?' || *c == b'*')
}

#[cfg(unix)]
fn contains_dubious_chars_safe(_: &Path) -> bool {
    // On unix, files can contain any characters and the file expansion is performed by the shell.
    false
}

#[cfg(not(unix))]
fn contains_dubious_chars_safe(path: &Path) -> bool {
    // On non-unix (Win* platforms), wildcard characters are not allowed in filenames.
    for b in path.as_os_str().as_encoded_bytes() {
        match b {
            b'?' | b'*' => return true,
            _ => {}
        }
    }

    false
}

const BZ_N_SUFFIX_PAIRS: usize = 4;

const Z_SUFFIX: [&str; BZ_N_SUFFIX_PAIRS] = [".bz2", ".bz", ".tbz2", ".tbz"];
const UNZ_SUFFIX: [&str; BZ_N_SUFFIX_PAIRS] = ["", "", ".tar", ".tar"];

static mut zSuffix: [*const c_char; BZ_N_SUFFIX_PAIRS] = [
    b".bz2\0" as *const u8 as *const libc::c_char,
    b".bz\0" as *const u8 as *const libc::c_char,
    b".tbz2\0" as *const u8 as *const libc::c_char,
    b".tbz\0" as *const u8 as *const libc::c_char,
];

unsafe fn hasSuffix(s: *mut c_char, suffix: *const c_char) -> Bool {
    let ns: i32 = strlen(s) as i32;
    let nx: i32 = strlen(suffix) as i32;
    if ns < nx {
        return 0 as Bool;
    }
    if strcmp(s.offset(ns as isize).offset(-nx as isize), suffix) == 0 as libc::c_int {
        return 1 as Bool;
    }
    0 as Bool
}

unsafe fn compress(name: *mut c_char) {
    let inStr: *mut FILE;
    let outStr: *mut FILE;
    let mut n: i32 = 0;
    let mut statBuf: stat = zeroed();
    delete_output_on_interrupt = false;
    if name.is_null() && srcMode != SourceMode::I2O {
        panic(b"compress: bad modes\n\0" as *const u8 as *const libc::c_char);
    }
    match srcMode {
        SourceMode::I2O => {
            copyFileName(
                inName.as_mut_ptr(),
                b"(stdin)\0" as *const u8 as *const libc::c_char,
            );
            copyFileName(
                outName.as_mut_ptr(),
                b"(stdout)\0" as *const u8 as *const libc::c_char,
            );
        }
        SourceMode::F2O => {
            copyFileName(inName.as_mut_ptr(), name);
            copyFileName(
                outName.as_mut_ptr(),
                b"(stdout)\0" as *const u8 as *const libc::c_char,
            );
        }
        SourceMode::F2F => {
            copyFileName(inName.as_mut_ptr(), name);
            copyFileName(outName.as_mut_ptr(), name);
            strcat(
                outName.as_mut_ptr(),
                b".bz2\0" as *const u8 as *const libc::c_char,
            );
        }
    }
    if srcMode != SourceMode::I2O && contains_dubious_chars(inName.as_mut_ptr()) {
        if noisy {
            eprintln!(
                "{}: There are no files matching `{}'.",
                get_program_name().display(),
                CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
            );
        }
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode != SourceMode::I2O && fileExists(inName.as_mut_ptr()) == 0 {
        eprintln!(
            "{}: Can't open input file {}: {}.",
            std::env::args().next().unwrap(),
            CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
            display_last_os_error(),
        );
        setExit(1 as libc::c_int);
        return;
    }
    let mut i = 0 as libc::c_int;
    while i < 4 as libc::c_int {
        if hasSuffix(inName.as_mut_ptr(), zSuffix[i as usize]) != 0 {
            if noisy {
                eprintln!(
                    "{}: Input file {} already has {} suffix.",
                    std::env::args().next().unwrap(),
                    CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
                    CStr::from_ptr(zSuffix[i as usize]).to_string_lossy(),
                );
            }
            setExit(1 as libc::c_int);
            return;
        }
        i += 1;
    }
    if srcMode == SourceMode::F2F || srcMode == SourceMode::F2O {
        stat(inName.as_mut_ptr(), &mut statBuf);
        if statBuf.st_mode & 0o170000 == 0o40000 {
            eprintln!(
                "{}: Input file {} is a directory.",
                get_program_name().display(),
                CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
            );
            setExit(1 as libc::c_int);
            return;
        }
    }
    if srcMode == SourceMode::F2F
        && !force_overwrite
        && notAStandardFile(inName.as_mut_ptr()) as libc::c_int != 0
    {
        if noisy {
            eprintln!(
                "{}: Input file {} is not a normal file.",
                get_program_name().display(),
                CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
            );
        }
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode == SourceMode::F2F && fileExists(outName.as_mut_ptr()) as libc::c_int != 0 {
        if force_overwrite {
            remove(outName.as_mut_ptr());
        } else {
            eprintln!(
                "{}: Output file {} already exists.",
                get_program_name().display(),
                CStr::from_ptr(outName.as_ptr()).to_string_lossy(),
            );
            setExit(1 as libc::c_int);
            return;
        }
    }
    if srcMode == SourceMode::F2F && !force_overwrite && {
        n = countHardLinks(inName.as_mut_ptr());
        n > 0 as libc::c_int
    } {
        eprintln!(
            "{}: Input file {} has {} other link{}.",
            get_program_name().display(),
            CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
            n,
            if n > 1 as libc::c_int { "s" } else { "" },
        );
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode == SourceMode::F2F {
        saveInputFileMetaInfo(inName.as_mut_ptr());
    }
    match srcMode {
        SourceMode::I2O => {
            inStr = stdin;
            outStr = stdout;
            if std::io::stdout().is_terminal() {
                eprintln!(
                    "{}: I won't write compressed data to a terminal.",
                    get_program_name().display(),
                );
                eprintln!(
                    "{}: For help, type: `{} --help'.",
                    get_program_name().display(),
                    get_program_name().display(),
                );
                setExit(1 as libc::c_int);
                return;
            }
        }
        SourceMode::F2O => {
            inStr = fopen(
                inName.as_mut_ptr(),
                b"rb\0" as *const u8 as *const libc::c_char,
            );
            outStr = stdout;
            if std::io::stdout().is_terminal() {
                eprintln!(
                    "{}: I won't write compressed data to a terminal.",
                    get_program_name().display(),
                );
                eprintln!(
                    "{}: For help, type: `{} --help'.",
                    get_program_name().display(),
                    get_program_name().display(),
                );
                if !inStr.is_null() {
                    fclose(inStr);
                }
                setExit(1 as libc::c_int);
                return;
            }
            if inStr.is_null() {
                eprintln!(
                    "{}: Can't open input file {}: {}.",
                    get_program_name().display(),
                    CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
                    display_last_os_error(),
                );
                setExit(1 as libc::c_int);
                return;
            }
        }
        SourceMode::F2F => {
            inStr = fopen(
                inName.as_mut_ptr(),
                b"rb\0" as *const u8 as *const libc::c_char,
            );
            outStr = fopen_output_safely(
                outName.as_mut_ptr(),
                b"wb\0" as *const u8 as *const libc::c_char,
            );
            if outStr.is_null() {
                eprintln!(
                    "{}: Can't create output file {}: {}.",
                    std::env::args().next().unwrap(),
                    CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
                    display_last_os_error(),
                );
                if !inStr.is_null() {
                    fclose(inStr);
                }
                setExit(1 as libc::c_int);
                return;
            }
            if inStr.is_null() {
                eprintln!(
                    "{}: Can't open input file {}: {}.",
                    std::env::args().next().unwrap(),
                    CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
                    display_last_os_error(),
                );
                if !outStr.is_null() {
                    fclose(outStr);
                }
                setExit(1 as libc::c_int);
                return;
            }
        }
    }
    if verbosity >= 1 as libc::c_int {
        eprint!("  {}: ", CStr::from_ptr(inName.as_ptr()).to_string_lossy(),);
        pad(inName.as_mut_ptr());
    }
    outputHandleJustInCase = outStr;
    delete_output_on_interrupt = true;
    compressStream(inStr, outStr);
    outputHandleJustInCase = std::ptr::null_mut::<FILE>();
    if srcMode == SourceMode::F2F {
        applySavedTimeInfoToOutputFile(outName.as_mut_ptr());
        delete_output_on_interrupt = false;
        if !keep_input_files {
            let retVal: IntNative = remove(inName.as_mut_ptr());
            if retVal != 0 as libc::c_int {
                ioError();
            }
        }
    }
    delete_output_on_interrupt = false;
}

unsafe fn uncompress(name: Option<String>) {
    let inStr: *mut FILE;
    let outStr: *mut FILE;
    let n: u64;

    delete_output_on_interrupt = false;

    let mut cannot_guess = false;

    let in_name: PathBuf;
    let out_name: PathBuf;

    match (name, srcMode) {
        (_, SourceMode::I2O) => {
            in_name = PathBuf::from("(stdin)");
            out_name = PathBuf::from("(stdout)");

            copyFileName(
                inName.as_mut_ptr(),
                b"(stdin)\0" as *const u8 as *const libc::c_char,
            );
            copyFileName(
                outName.as_mut_ptr(),
                b"(stdout)\0" as *const u8 as *const libc::c_char,
            );
        }
        (Some(name), SourceMode::F2O) => {
            in_name = PathBuf::from(&name);
            out_name = PathBuf::from("(stdout)");

            let name_cstr = CString::new(name).unwrap();
            copyFileName(inName.as_mut_ptr(), name_cstr.as_ptr());
            copyFileName(
                outName.as_mut_ptr(),
                b"(stdout)\0" as *const u8 as *const libc::c_char,
            );
        }
        (Some(mut name), SourceMode::F2F) => {
            in_name = PathBuf::from(&name);

            let name_cstr = CString::new(name.clone()).unwrap();
            copyFileName(inName.as_mut_ptr(), name_cstr.as_ptr());

            cannot_guess = 'blk: {
                for (old, new) in Z_SUFFIX.iter().zip(UNZ_SUFFIX) {
                    if name.ends_with(old) {
                        name.truncate(name.len() - old.len());
                        name += new;
                        break 'blk false;
                    }
                }

                name += ".out";
                true
            };

            out_name = PathBuf::from(&name);

            let name_cstr = CString::new(name).unwrap();
            copyFileName(outName.as_mut_ptr(), name_cstr.as_ptr());
        }
        (None, SourceMode::F2O | SourceMode::F2F) => {
            panic(b"uncompress: bad modes\n\0" as *const u8 as *const libc::c_char)
        }
    }

    if srcMode != SourceMode::I2O && contains_dubious_chars_safe(&in_name) {
        if noisy {
            eprintln!(
                "%{}: There are no files matching `{}'.",
                get_program_name().display(),
                in_name.display(),
            );
        }
        setExit(1);
        return;
    }

    if srcMode != SourceMode::I2O && !in_name.exists() {
        eprintln!(
            "{}: Can't open input file {}: {}.",
            get_program_name().display(),
            in_name.display(),
            display_last_os_error(),
        );
        setExit(1);
        return;
    }

    if (srcMode == SourceMode::F2F || srcMode == SourceMode::F2O) && in_name.is_dir() {
        eprintln!(
            "{}: Input file {} is a directory.",
            get_program_name().display(),
            in_name.display(),
        );
        setExit(1);
        return;
    }

    if srcMode == SourceMode::F2F && !force_overwrite && not_a_standard_file(&in_name) {
        if noisy {
            eprintln!(
                "{}: Input file {} is not a normal file.",
                get_program_name().display(),
                in_name.display(),
            );
        }
        setExit(1);
        return;
    }

    if cannot_guess && noisy {
        // just a warning, no return
        eprintln!(
            "{}: Can't guess original name for {} -- using {}",
            get_program_name().display(),
            in_name.display(),
            out_name.display(),
        );
    }

    if srcMode == SourceMode::F2F && out_name.exists() {
        if force_overwrite {
            let _ = std::fs::remove_file(&out_name);
        } else {
            eprintln!(
                "{}: Output file {} already exists.",
                get_program_name().display(),
                out_name.display(),
            );
            setExit(1);
            return;
        }
    }

    if srcMode == SourceMode::F2F && !force_overwrite && {
        n = count_hardlinks(&in_name);
        n > 0
    } {
        eprintln!(
            "{}: Input file {} has {} other link{}.",
            get_program_name().display(),
            in_name.display(),
            n,
            if n > 1 { "s" } else { "" },
        );
        setExit(1 as libc::c_int);
        return;
    }

    if srcMode == SourceMode::F2F {
        // Save the file's meta-info before we open it.
        // Doing it later means we mess up the access times.
        unsafe { saveInputFileMetaInfo(inName.as_mut_ptr()) };
    }

    match srcMode {
        SourceMode::I2O => {
            inStr = stdin;
            outStr = stdout;
            if std::io::stdin().is_terminal() {
                eprint!(
                    concat!(
                        "{program_name}: I won't read compressed data from a terminal.\n",
                        "{program_name}: For help, type: `{program_name} --help'.\n",
                    ),
                    program_name = get_program_name().display(),
                );
                setExit(1);
                return;
            }
        }
        SourceMode::F2O => {
            inStr = fopen(
                inName.as_mut_ptr(),
                b"rb\0" as *const u8 as *const libc::c_char,
            );
            outStr = stdout;
            if inStr.is_null() {
                eprintln!(
                    "{}: Can't open input file {}: {}.",
                    get_program_name().display(),
                    in_name.display(),
                    display_last_os_error(),
                );
                if !inStr.is_null() {
                    // this is unreachable, but it exists in the original C source code
                    fclose(inStr);
                }
                setExit(1);
                return;
            }
        }
        SourceMode::F2F => {
            inStr = fopen(
                inName.as_mut_ptr(),
                b"rb\0" as *const u8 as *const libc::c_char,
            );
            outStr = fopen_output_safely(
                outName.as_mut_ptr(),
                b"wb\0" as *const u8 as *const libc::c_char,
            );

            if outStr.is_null() {
                eprintln!(
                    "{}: Can't create output file {}: {}.",
                    get_program_name().display(),
                    out_name.display(),
                    display_last_os_error(),
                );
                if !inStr.is_null() {
                    fclose(inStr);
                }
                setExit(1);
                return;
            }

            if inStr.is_null() {
                eprintln!(
                    "{}: Can't open input file {}: {}.",
                    get_program_name().display(),
                    in_name.display(),
                    display_last_os_error(),
                );
                if !outStr.is_null() {
                    fclose(outStr);
                }
                setExit(1);
                return;
            }
        }
    }

    if verbosity >= 1 {
        eprint!("  {}: ", in_name.display(),);
        pad(inName.as_mut_ptr());
    }

    /*--- Now the input and output handles are sane.  Do the Biz. ---*/
    outputHandleJustInCase = outStr;
    delete_output_on_interrupt = true;
    let magicNumberOK = uncompressStream(inStr, outStr);
    outputHandleJustInCase = std::ptr::null_mut::<FILE>();

    /*--- If there was an I/O error, we won't get here. ---*/
    if magicNumberOK {
        if srcMode == SourceMode::F2F {
            applySavedTimeInfoToOutputFile(outName.as_mut_ptr());
            delete_output_on_interrupt = false;
            if !keep_input_files {
                let retVal: IntNative = remove(inName.as_mut_ptr());
                if retVal != 0 {
                    ioError();
                }
            }
        }
    } else {
        unz_fails_exist = true;
        delete_output_on_interrupt = false;
        if srcMode == SourceMode::F2F {
            let retVal_0: IntNative = remove(outName.as_mut_ptr());
            if retVal_0 != 0 {
                ioError();
            }
        }
    }

    delete_output_on_interrupt = false;

    if magicNumberOK {
        if verbosity >= 1 {
            eprintln!("done");
        }
    } else {
        setExit(2);
        if verbosity >= 1 {
            eprintln!("not a bzip2 file.");
        } else {
            eprintln!(
                "{}: {} is not a bzip2 file.",
                get_program_name().display(),
                in_name.display(),
            );
        }
    };
}

unsafe fn testf(name: Option<String>) {
    let inStr: *mut FILE;
    delete_output_on_interrupt = false;

    copyFileName(
        outName.as_mut_ptr(),
        b"(none)\0" as *const u8 as *const libc::c_char,
    );

    let in_name;

    match (name, srcMode) {
        (_, SourceMode::I2O) => {
            in_name = PathBuf::from("(stdin)");
            copyFileName(
                inName.as_mut_ptr(),
                b"(stdin)\0" as *const u8 as *const libc::c_char,
            );
        }
        (Some(name), SourceMode::F2O) => {
            in_name = PathBuf::from(&name);
            let name = CString::new(name).unwrap();
            copyFileName(inName.as_mut_ptr(), name.as_ptr());
        }
        (Some(name), SourceMode::F2F) => {
            in_name = PathBuf::from(&name);
            let name = CString::new(name).unwrap();
            copyFileName(inName.as_mut_ptr(), name.as_ptr());
        }
        (None, SourceMode::F2O | SourceMode::F2F) => {
            panic_str("testf: bad modes");
        }
    }

    if srcMode != SourceMode::I2O && contains_dubious_chars_safe(&in_name) {
        if noisy {
            eprintln!(
                "{}: There are no files matching `{}'.",
                get_program_name().display(),
                in_name.display(),
            );
        }
        setExit(1);
        return;
    }
    if srcMode != SourceMode::I2O && !in_name.exists() {
        eprintln!(
            "{}: Can't open input {}: {}.",
            get_program_name().display(),
            in_name.display(),
            display_last_os_error(),
        );
        setExit(1);
        return;
    }
    if srcMode != SourceMode::I2O && in_name.is_dir() {
        eprintln!(
            "{}: Input file {} is a directory.",
            get_program_name().display(),
            in_name.display(),
        );
        setExit(1);
        return;
    }
    match srcMode {
        SourceMode::I2O => {
            if std::io::stdin().is_terminal() {
                eprintln!(
                    "{}: I won't read compressed data from a terminal.",
                    get_program_name().display(),
                );
                eprintln!(
                    "{}: For help, type: `{} --help'.",
                    get_program_name().display(),
                    get_program_name().display(),
                );
                setExit(1);
                return;
            }
            inStr = stdin;
        }
        SourceMode::F2O | SourceMode::F2F => {
            inStr = fopen(
                inName.as_mut_ptr(),
                b"rb\0" as *const u8 as *const libc::c_char,
            );
            if inStr.is_null() {
                eprintln!(
                    "{}: Can't open input {}: {}.",
                    get_program_name().display(),
                    in_name.display(),
                    display_last_os_error(),
                );
                setExit(1);
                return;
            }
        }
    }
    if verbosity >= 1 {
        eprint!("  {}: ", in_name.display());
        pad(inName.as_mut_ptr());
    }
    outputHandleJustInCase = std::ptr::null_mut::<FILE>();
    let allOK = testStream(inStr);
    if allOK && verbosity >= 1 {
        eprintln!("ok");
    }
    if !allOK {
        test_fails_exists = true;
    }
}

const BZLIB_VERSION: &str = unsafe {
    match CStr::from_ptr(BZ2_bzlibVersion()).to_str() {
        Ok(s) => s,
        Err(_) => panic!(),
    }
};

fn license() {
    print!(
        concat!(
            "bzip2, a block-sorting file compressor.  Version {}.\n",
            "   \n",
            "   Copyright (C) 1996-2010 by Julian Seward.\n",
            "   \n",
            "   This program is free software; you can redistribute it and/or modify\n",
            "   it under the terms set out in the LICENSE file, which is included\n",
            "   in the bzip2-1.0.6 source distribution.\n",
            "   \n",
            "   This program is distributed in the hope that it will be useful,\n",
            "   but WITHOUT ANY WARRANTY; without even the implied warranty of\n",
            "   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the\n",
            "   LICENSE file for more details.\n",
            "   \n"
        ),
        BZLIB_VERSION,
    );
}

fn usage(full_program_name: &Path) {
    print!(
        concat!(
            "bzip2, a block-sorting file compressor.  Version {}.\n",
            "\n",
            "   usage: {} [flags and input files in any order]\n",
            "\n",
            "   -h --help           print this message\n",
            "   -d --decompress     force decompression\n",
            "   -z --compress       force compression\n",
            "   -k --keep           keep (don't delete) input files\n",
            "   -f --force          overwrite existing output files\n",
            "   -t --test           test compressed file integrity\n",
            "   -c --stdout         output to standard out\n",
            "   -q --quiet          suppress noncritical error messages\n",
            "   -v --verbose        be verbose (a 2nd -v gives more)\n",
            "   -L --license        display software version & license\n",
            "   -V --version        display software version & license\n",
            "   -s --small          use less memory (at most 2500k)\n",
            "   -1 .. -9            set block size to 100k .. 900k\n",
            "   --fast              alias for -1\n",
            "   --best              alias for -9\n",
            "\n",
            "   If invoked as `bzip2', default action is to compress.\n",
            "              as `bunzip2',  default action is to decompress.\n",
            "              as `bzcat', default action is to decompress to stdout.\n",
            "\n",
            "   If no file names are given, bzip2 compresses or decompresses\n",
            "   from standard input to standard output.  You can combine\n",
            "   short flags, so `-v -4' means the same as -v4 or -4v, &c.\n",
            "\n"
        ),
        BZLIB_VERSION,
        full_program_name.display(),
    );
}

fn redundant(program_name: &Path, flag_name: &str) {
    eprintln!(
        "{}: {} is redundant in versions 0.9.5 and above",
        program_name.display(),
        flag_name,
    );
}

fn contains_osstr(haystack: impl AsRef<OsStr>, needle: impl AsRef<OsStr>) -> bool {
    let needle = needle.as_ref().as_encoded_bytes();
    let haystack = haystack.as_ref().as_encoded_bytes();

    haystack.windows(needle.len()).any(|h| h == needle)
}

unsafe fn main_0(program_path: &Path) -> IntNative {
    let program_name = Path::new(program_path.file_name().unwrap());

    outputHandleJustInCase = std::ptr::null_mut::<FILE>();
    decompress_mode = DecompressMode::Fast;
    keep_input_files = false;
    force_overwrite = false;
    noisy = true;
    verbosity = 0;
    blockSize100k = 9;
    test_fails_exists = false;
    unz_fails_exist = false;
    numFileNames = 0;
    numFilesProcessed = 0;
    workFactor = 30;
    delete_output_on_interrupt = false;
    exitValue = 0;

    signal(
        11,
        mySIGSEGVorSIGBUScatcher as unsafe extern "C" fn(libc::c_int) as usize,
    );
    signal(
        7,
        mySIGSEGVorSIGBUScatcher as unsafe extern "C" fn(libc::c_int) as usize,
    );

    copyFileName(
        inName.as_mut_ptr(),
        b"(none)\0" as *const u8 as *const libc::c_char,
    );
    copyFileName(
        outName.as_mut_ptr(),
        b"(none)\0" as *const u8 as *const libc::c_char,
    );

    let program_name_str = program_name.to_str().unwrap();
    core::ptr::copy(
        program_name_str.as_ptr().cast::<libc::c_char>(),
        progNameReally.as_mut_ptr(),
        program_name_str.len(),
    );
    progName = progNameReally.as_mut_ptr();

    let mut arg_list = Vec::with_capacity(16);

    if let Ok(val) = std::env::var("BZIP2") {
        arg_list.extend(val.split_ascii_whitespace().map(|s| s.to_owned()));
    }

    if let Ok(val) = std::env::var("BZIP") {
        arg_list.extend(val.split_ascii_whitespace().map(|s| s.to_owned()));
    }

    arg_list.extend(std::env::args().skip(1));

    longestFileName = 7;
    numFileNames = 0;
    let mut decode = true;

    for name in &arg_list {
        if name == "--" {
            decode = false;
        } else if !(name.starts_with('-') && decode) {
            numFileNames += 1;
            longestFileName = Ord::max(longestFileName, name.len() as i32);
        }
    }

    srcMode = match numFileNames {
        0 => SourceMode::I2O,
        _ => SourceMode::F2F,
    };
    opMode = OperationMode::Zip;
    if contains_osstr(program_name, "unzip") || contains_osstr(program_name, "UNZIP") {
        opMode = OperationMode::Unzip;
    }
    if contains_osstr(program_name, "z2cat")
        || contains_osstr(program_name, "Z2CAT")
        || contains_osstr(program_name, "zcat")
        || contains_osstr(program_name, "ZCAT")
    {
        opMode = OperationMode::Unzip;
        srcMode = match numFileNames {
            0 => SourceMode::F2O,
            _ => SourceMode::F2F,
        };
    }

    for flag_name in &arg_list {
        if flag_name == "--" {
            break;
        }

        // only `-h`, not `--help`
        if flag_name.as_bytes()[0] == b'-' && flag_name.as_bytes()[1] != b'-' {
            for c in &flag_name.as_bytes()[1..] {
                match c {
                    b'c' => srcMode = SourceMode::F2O,
                    b'd' => opMode = OperationMode::Unzip,
                    b'z' => opMode = OperationMode::Zip,
                    b'f' => force_overwrite = true,
                    b't' => opMode = OperationMode::Test,
                    b'k' => keep_input_files = true,
                    b's' => decompress_mode = DecompressMode::Small,
                    b'q' => noisy = false,
                    b'1' => blockSize100k = 1,
                    b'2' => blockSize100k = 2,
                    b'3' => blockSize100k = 3,
                    b'4' => blockSize100k = 4,
                    b'5' => blockSize100k = 5,
                    b'6' => blockSize100k = 6,
                    b'7' => blockSize100k = 7,
                    b'8' => blockSize100k = 8,
                    b'9' => blockSize100k = 9,
                    b'V' | b'L' => {
                        license();
                        exit(0);
                    }
                    b'v' => verbosity += 1,
                    b'h' => {
                        usage(program_name);
                        exit(0);
                    }
                    _ => {
                        eprintln!("{}: Bad flag `{}'", program_name.display(), flag_name,);
                        usage(program_name);
                        exit(1);
                    }
                }
            }
        }
    }

    for flag_name in &arg_list {
        match flag_name.as_str() {
            "--" => break,
            "--stdout" => srcMode = SourceMode::F2O,
            "--decompress" => opMode = OperationMode::Unzip,
            "--compress" => opMode = OperationMode::Zip,
            "--force" => force_overwrite = true,
            "--test" => opMode = OperationMode::Test,
            "--keep" => keep_input_files = true,
            "--small" => decompress_mode = DecompressMode::Small,
            "--quiet" => noisy = false,
            "--version" | "--license" => {
                license();
                exit(0);
            }
            "--exponential" => workFactor = 1,
            "--repetitive-best" => redundant(program_name, flag_name),
            "--repetitive-fast" => redundant(program_name, flag_name),
            "--fast" => blockSize100k = 1,
            "--best" => blockSize100k = 9,
            "--verbose" => verbosity += 1,
            "--help" => {
                usage(program_name);
                exit(0);
            }
            _ => {
                if flag_name.starts_with("--") {
                    eprintln!("{}: Bad flag `{}'", program_name.display(), flag_name);
                    usage(program_name);
                    exit(1);
                }
            }
        }
    }

    if verbosity > 4 as libc::c_int {
        verbosity = 4;
    }

    if opMode == OperationMode::Zip && decompress_mode == DecompressMode::Small && blockSize100k > 2
    {
        blockSize100k = 2;
    }
    if opMode == OperationMode::Test && srcMode == SourceMode::F2O {
        eprintln!(
            "{}: -c and -t cannot be used together.",
            program_name.display(),
        );
        exit(1);
    }
    if srcMode == SourceMode::F2O && numFileNames == 0 {
        srcMode = SourceMode::I2O;
    }
    if opMode != OperationMode::Zip {
        blockSize100k = 0;
    }
    if srcMode == SourceMode::F2F {
        signal(
            2,
            mySignalCatcher as unsafe extern "C" fn(IntNative) as usize,
        );
        signal(
            15,
            mySignalCatcher as unsafe extern "C" fn(IntNative) as usize,
        );
        signal(
            1,
            mySignalCatcher as unsafe extern "C" fn(IntNative) as usize,
        );
    }

    match opMode {
        OperationMode::Zip => {
            if srcMode == SourceMode::I2O {
                compress(std::ptr::null_mut());
            } else {
                decode = true;
                for name in arg_list {
                    if name == "--" {
                        decode = false;
                    } else if !(name.starts_with('-') && decode) {
                        numFilesProcessed += 1;
                        let name = CString::new(name).unwrap();
                        compress(name.as_ptr().cast_mut());
                    }
                }
            }
        }
        OperationMode::Unzip => {
            unz_fails_exist = false;
            if srcMode == SourceMode::I2O {
                uncompress(None);
            } else {
                decode = true;
                for name in arg_list {
                    if name == "--" {
                        decode = false;
                    } else if !(name.starts_with('-') && decode) {
                        numFilesProcessed += 1;
                        uncompress(Some(name));
                    }
                }
            }
            if unz_fails_exist {
                setExit(2);
                exit(exitValue);
            }
        }
        OperationMode::Test => {
            test_fails_exists = false;
            if srcMode == SourceMode::I2O {
                testf(None);
            } else {
                decode = true;
                for name in arg_list {
                    if name == "--" {
                        decode = false;
                    } else if !(name.starts_with('-') && decode) {
                        numFilesProcessed += 1;
                        testf(Some(name));
                    }
                }
            }
            if test_fails_exists {
                if noisy {
                    eprintln!(concat!(
                        "\n",
                        "You can use the `bzip2recover' program to attempt to recover\n",
                        "data from undamaged sections of corrupted files.\n",
                    ));
                }
                setExit(2);
                exit(exitValue);
            }
        }
    }

    exitValue
}

fn main() {
    let mut it = ::std::env::args_os();

    let program_name = PathBuf::from(it.next().unwrap());

    unsafe { ::std::process::exit(main_0(&program_name) as i32) }
}
