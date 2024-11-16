#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::ffi::{c_char, CStr, OsStr};
use std::fs::Metadata;
use std::io::{IsTerminal, Read, Write};
use std::mem::zeroed;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

use libbzip2_rs_sys::{
    BZ2_bzRead, BZ2_bzReadClose, BZ2_bzReadGetUnused, BZ2_bzReadOpen, BZ2_bzWrite,
    BZ2_bzWriteClose64, BZ2_bzWriteOpen, BZ2_bzlibVersion, BZFILE,
};

use libc::{
    _exit, exit, fclose, ferror, fflush, fgetc, fileno, fopen, fread, perror, remove, rewind,
    signal, stat, strlen, ungetc, write, FILE, SIGINT, SIGSEGV, SIGTERM,
};

// FIXME remove this
#[cfg(not(target_os = "windows"))]
extern "C" {
    #[cfg_attr(not(target_os = "macos"), link_name = "stdin")]
    #[cfg_attr(target_os = "macos", link_name = "__stdinp")]
    static mut stdin_handle: *mut FILE;
    #[cfg_attr(not(target_os = "macos"), link_name = "stdout")]
    #[cfg_attr(target_os = "macos", link_name = "__stdoutp")]
    static mut stdout_handle: *mut FILE;
}

#[cfg(all(target_os = "windows", target_env = "gnu"))]
extern "C" {
    fn __acrt_iob_func(idx: libc::c_uint) -> *mut FILE;
}

#[cfg(not(target_os = "windows"))]
macro_rules! STDIN {
    () => {
        stdin_handle
    };
}

#[cfg(all(target_os = "windows", target_env = "gnu"))]
macro_rules! STDIN {
    () => {
        __acrt_iob_func(0)
    };
}

#[cfg(not(target_os = "windows"))]
macro_rules! STDOUT {
    () => {
        stdout_handle
    };
}

#[cfg(all(target_os = "windows", target_env = "gnu"))]
macro_rules! STDOUT {
    () => {
        __acrt_iob_func(1)
    };
}

type IntNative = libc::c_int;

static mut keep_input_files: bool = false;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DecompressMode {
    Fast = 0,
    Small = 1,
}

// static mut decompress_mode: DecompressMode = DecompressMode::Fast;

static mut delete_output_on_interrupt: bool = false;
// static mut force_overwrite: bool = false;
static mut test_fails_exists: bool = false;
static mut unz_fails_exist: bool = false;
static mut noisy: bool = false;
static mut numFileNames: i32 = 0;
static mut numFilesProcessed: i32 = 0;
static mut exitValue: i32 = 0;

struct UncompressConfig {
    verbosity: i32,
    decompress_mode: DecompressMode,
    force_overwrite: bool,
}

struct CompressConfig {
    blockSize100k: i32,
    verbosity: i32,
    workFactor: i32,
    force_overwrite: bool,
}

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

static LONGEST_FILENAME: AtomicUsize = AtomicUsize::new(0);
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

/// Strictly for compatibility with the original bzip2 output
fn display_last_os_error() -> String {
    display_os_error(std::io::Error::last_os_error())
}

fn display_os_error(error: std::io::Error) -> String {
    let mut error = error.to_string();

    // now strip off the ` (os error x)` part
    if let Some(index) = error.find(" (os error") {
        error.truncate(index);
    }

    // the C version tries to open a file to check whether a path exists
    error.replace("Bad address", "No such file or directory")
}

unsafe fn myfeof(f: *mut FILE) -> bool {
    let c: i32 = fgetc(f);
    if c == -1 as libc::c_int {
        return true;
    }
    ungetc(c, f);
    false
}

enum InputStream {
    Stdin(std::io::Stdin),
    File(std::fs::File),
}

impl std::io::Read for InputStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            InputStream::Stdin(stdin) => stdin.read(buf),
            InputStream::File(file) => file.read(buf),
        }
    }
}

unsafe fn compressStream(
    config: &CompressConfig,
    mut stream: InputStream,
    zStream: *mut FILE,
    metadata: Option<&Metadata>,
) {
    let mut ibuf: [u8; 5000] = [0; 5000];
    let mut nbytes_in_lo32: u32 = 0;
    let mut nbytes_in_hi32: u32 = 0;
    let mut nbytes_out_lo32: u32 = 0;
    let mut nbytes_out_hi32: u32 = 0;
    let mut bzerr: i32 = 0;
    let mut ret: i32;

    set_binary_mode(zStream);

    if ferror(zStream) != 0 {
        // diverges
        ioError()
    }

    let bzf = BZ2_bzWriteOpen(
        &mut bzerr,
        zStream,
        config.blockSize100k,
        config.verbosity,
        config.workFactor,
    );

    'errhandler: {
        if bzerr != libbzip2_rs_sys::BZ_OK {
            break 'errhandler;
        }

        if config.verbosity >= 2 {
            eprintln!();
        }

        loop {
            let nIbuf = match stream.read(&mut ibuf) {
                Ok(0) => break, // EOF
                Ok(n) => n,
                Err(e) => exit_with_io_error(e),
            };

            BZ2_bzWrite(
                &mut bzerr,
                bzf,
                ibuf.as_mut_ptr() as *mut libc::c_void,
                nIbuf as i32,
            );

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

        if let Some(metadata) = metadata {
            set_permissions(zStream, metadata);
            outputHandleJustInCase = core::ptr::null_mut();
            ret = fclose(zStream);
            if ret == libc::EOF {
                // diverges
                ioError()
            }
        }

        outputHandleJustInCase = core::ptr::null_mut();

        if config.verbosity >= 1 {
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
        &mut 0,
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

unsafe fn uncompressStream(
    config: &UncompressConfig,
    zStream: *mut FILE,
    mut stream: OutputStream,
    metadata: Option<&Metadata>,
) -> bool {
    let mut bzf = std::ptr::null_mut();
    let mut bzerr: i32 = 0;
    let mut bzerr_dummy: i32 = 0;
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

    set_binary_mode(zStream);

    if ferror(zStream) != 0 {
        // diverges
        ioError()
    }

    'outer: loop {
        match state {
            State::Standard => loop {
                bzf = BZ2_bzReadOpen(
                    &mut bzerr,
                    zStream,
                    config.verbosity,
                    config.decompress_mode as libc::c_int,
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
                        if let Err(e) = stream.write_all(&obuf[..nread as usize]) {
                            exit_with_io_error(e) // diverges
                        }
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

                if nUnused == 0 && myfeof(zStream) {
                    state = State::CloseOk;
                    continue 'outer;
                }
            },
            State::CloseOk => {
                if ferror(zStream) != 0 {
                    // diverges
                    ioError()
                }

                if let Some(metadata) = metadata {
                    if let OutputStream::File(file) = &stream {
                        set_permissions_rust(file, metadata);
                    }
                }

                if let libc::EOF = fclose(zStream) {
                    ioError()
                }

                if let Err(e) = stream.flush() {
                    exit_with_io_error(e) // diverges
                }

                if config.verbosity >= 2 {
                    eprint!("\n    ");
                }

                return true;
            }
            State::TryCat => {
                if config.force_overwrite {
                    rewind(zStream);
                    loop {
                        if myfeof(zStream) {
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
                            if let Err(e) = stream.write_all(&obuf[..nread as usize]) {
                                exit_with_io_error(e) // diverges
                            }
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
                        if zStream != STDIN!() {
                            fclose(zStream);
                        }

                        if streamNo == 1 {
                            return false;
                        } else {
                            if noisy {
                                eprintln!(
                                    "\n{}: {}: trailing garbage after EOF ignored",
                                    get_program_name().display(),
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

unsafe fn testStream(config: &UncompressConfig, zStream: *mut FILE) -> bool {
    let mut bzf: *mut BZFILE;
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
                config.verbosity,
                config.decompress_mode as libc::c_int,
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
            if nUnused == 0 && myfeof(zStream) {
                break;
            }
        }

        if ferror(zStream) != 0 {
            ioError() // diverges
        }
        if fclose(zStream) == libc::EOF {
            ioError() // diverges
        }

        if config.verbosity >= 2 {
            eprintln!()
        }

        return true;
    }

    // errhandler:

    BZ2_bzReadClose(&mut 0, bzf);
    if config.verbosity == 0 {
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
            if zStream != STDIN!() {
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

unsafe fn panic_str(s: &str) -> ! {
    eprint!(
        concat!(
            "\n",
            "{}: PANIC -- internal consistency error:\n",
            "\t{}\n",
            "\tThis is a BUG.  Please report it at:\n",
            "\thttps://github.com/trifectatechfoundation/libbzip2-rs/issues\n"
        ),
        get_program_name().display(),
        s,
    );
    showFileNames();
    cleanUpAndFail(3 as libc::c_int);
}

unsafe fn crcError() -> ! {
    eprintln!(
        "\n{}: Data integrity error when decompressing.",
        get_program_name().display(),
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
            get_program_name().display(),
        );
        perror(progName);
        showFileNames();
        cadvise();
    }
    cleanUpAndFail(2 as libc::c_int);
}

unsafe fn exit_with_io_error(error: std::io::Error) -> ! {
    eprintln!(
        "\n{}: I/O or other error, bailing out.  Possible reason follows.",
        get_program_name().display(),
    );
    eprintln!("{}", display_os_error(error));
    showFileNames();
    cleanUpAndFail(1 as libc::c_int);
}

unsafe fn ioError() -> ! {
    eprintln!(
        "\n{}: I/O or other error, bailing out.  Possible reason follows.",
        get_program_name().display(),
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
        get_program_name().display(),
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

fn pad(s: &Path) {
    let len = s.as_os_str().as_encoded_bytes().len();
    let longest_filename = LONGEST_FILENAME.load(Ordering::Relaxed);

    if len >= longest_filename {
        return;
    }

    for _ in 1..=longest_filename - len {
        eprint!(" ");
    }
}

unsafe fn copy_filename(to: *mut c_char, from: &str) {
    if from.len() > (FILE_NAME_LEN - 10) {
        eprint!(
            concat!(
                "bzip2: file name\n",
                "`{}'\n",
                "is suspiciously (more than {} chars) long.\n",
                "Try using a reasonable file name instead.  Sorry! :-)\n",
            ),
            from,
            FILE_NAME_LEN - 10
        );
        setExit(1 as libc::c_int);
        exit(exitValue);
    }

    std::ptr::copy(from.as_ptr().cast::<c_char>(), to, from.len());
    *to.wrapping_add(from.len() + 1) = '\0' as i32 as c_char;

    // make sure that the final byte is always 0
    *to.wrapping_add(FILE_NAME_LEN - 10) = '\0' as i32 as c_char;
}

fn fopen_output_safely(name: impl AsRef<Path>) -> *mut FILE {
    #[cfg(unix)]
    {
        use std::os::fd::IntoRawFd;
        use std::os::unix::fs::OpenOptionsExt;

        let mut opts = std::fs::File::options();

        opts.write(true).create_new(true);

        #[allow(clippy::unnecessary_cast)]
        opts.mode((libc::S_IWUSR | libc::S_IRUSR) as u32);

        let Ok(file) = opts.open(name) else {
            return std::ptr::null_mut::<FILE>();
        };

        let fd = file.into_raw_fd();
        let mode = b"wb\0".as_ptr().cast::<c_char>();
        let fp = unsafe { libc::fdopen(fd, mode) };
        if fp.is_null() {
            unsafe { libc::close(fd) };
        }
        fp
    }

    #[cfg(not(unix))]
    unsafe {
        use std::ffi::CString;

        // The CString really only needs to live for the duration of the fopen
        #[allow(temporary_cstring_as_ptr)]
        libc::fopen(
            CString::new(name.as_ref().to_str().unwrap())
                .unwrap()
                .as_ptr(),
            b"wb\0".as_ptr().cast::<c_char>(),
        )
    }
}

fn not_a_standard_file(path: &Path) -> bool {
    let Ok(metadata) = path.symlink_metadata() else {
        return true;
    };

    !metadata.is_file()
}

#[cfg(unix)]
fn count_hardlinks(path: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt;

    let Ok(metadata) = path.metadata() else {
        return 0;
    };

    metadata.nlink().saturating_sub(1)
}

#[cfg(not(unix))]
unsafe fn count_hardlinks(_path: &Path) -> u64 {
    0 // FIXME
}

fn apply_saved_time_info_to_output_file(_dst_name: &CStr, _metadata: Metadata) {
    #[cfg(unix)]
    {
        use std::time::SystemTime;

        let convert = |x: SystemTime| {
            x.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as libc::time_t
        };

        let actime = convert(_metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH));
        let modtime = convert(_metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH));

        let buf = libc::utimbuf { actime, modtime };

        match unsafe { libc::utime(_dst_name.as_ptr(), &buf) } {
            0 => {}
            _ => unsafe { ioError() },
        }
    }
}

unsafe fn set_permissions(_handle: *mut FILE, _metadata: &Metadata) {
    #[cfg(unix)]
    {
        let fd = fileno(_handle);
        if fd < 0 {
            // diverges
            ioError()
        }

        set_permissions_fd(fd, _metadata)
    }
}

unsafe fn set_permissions_rust(_file: &std::fs::File, _metadata: &Metadata) {
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        set_permissions_fd(_file.as_raw_fd(), _metadata)
    }
}

#[cfg(unix)]
unsafe fn set_permissions_fd(fd: core::ffi::c_int, metadata: &Metadata) {
    use std::os::unix::fs::MetadataExt;

    let retVal = libc::fchmod(fd, metadata.mode() as libc::mode_t);
    if retVal != 0 as libc::c_int {
        ioError();
    }

    // chown() will in many cases return with EPERM, which can be safely ignored.
    libc::fchown(fd, metadata.uid(), metadata.gid());
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

#[cfg(windows)]
/// Prevent Windows from mangling the read data.
unsafe fn set_binary_mode(file: *mut FILE) {
    use std::ffi::c_int;

    extern "C" {
        fn _setmode(fd: c_int, mode: c_int) -> c_int;
    }

    if _setmode(fileno(file), libc::O_BINARY) == -1 {
        ioError();
    }
}

#[cfg(not(windows))]
/// Prevent Windows from mangling the read data.
unsafe fn set_binary_mode(_file: *mut FILE) {}

unsafe fn compress(config: &CompressConfig, name: Option<&str>) {
    let in_name;
    let out_name;

    let input_stream;
    let outStr: *mut FILE;

    let mut n: u64 = 0;
    delete_output_on_interrupt = false;

    match (name, srcMode) {
        (_, SourceMode::I2O) => {
            in_name = Path::new("(stdin)");
            out_name = PathBuf::from("(stdout)");

            copy_filename(inName.as_mut_ptr(), "(stdin)");
            copy_filename(outName.as_mut_ptr(), "(stdout)");
        }
        (Some(name), SourceMode::F2O) => {
            in_name = Path::new(name);
            out_name = PathBuf::from("(stdout)");

            copy_filename(inName.as_mut_ptr(), name);
            copy_filename(outName.as_mut_ptr(), "(stdout)");
        }
        (Some(name), SourceMode::F2F) => {
            in_name = Path::new(name);
            out_name = PathBuf::from(format!("{name}.bz2"));

            copy_filename(inName.as_mut_ptr(), in_name.to_str().unwrap());
            copy_filename(outName.as_mut_ptr(), out_name.to_str().unwrap());
        }
        (None, SourceMode::F2O | SourceMode::F2F) => {
            panic_str("compress: bad modes\n");
        }
    }

    if srcMode != SourceMode::I2O && contains_dubious_chars_safe(in_name) {
        if noisy {
            eprintln!(
                "{}: There are no files matching `{}'.",
                get_program_name().display(),
                in_name.display(),
            );
        }
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode != SourceMode::I2O && !in_name.exists() {
        eprintln!(
            "{}: Can't open input file {}: {}.",
            std::env::args().next().unwrap(),
            in_name.display(),
            display_last_os_error(),
        );
        setExit(1 as libc::c_int);
        return;
    }
    if let Some(extension) = in_name.extension() {
        for bz2_extension in Z_SUFFIX {
            if extension == OsStr::new(&bz2_extension[1..]) {
                if noisy {
                    eprintln!(
                        "{}: Input file {} already has {} suffix.",
                        std::env::args().next().unwrap(),
                        in_name.display(),
                        &bz2_extension[1..],
                    );
                }
                setExit(1 as libc::c_int);
                return;
            }
        }
    }
    if (srcMode == SourceMode::F2F || srcMode == SourceMode::F2O) && in_name.is_dir() {
        eprintln!(
            "{}: Input file {} is a directory.",
            get_program_name().display(),
            in_name.display(),
        );
        setExit(1 as libc::c_int);
        return;
    }

    if srcMode == SourceMode::F2F && !config.force_overwrite && not_a_standard_file(in_name) {
        if noisy {
            eprintln!(
                "{}: Input file {} is not a normal file.",
                get_program_name().display(),
                in_name.display(),
            );
        }
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode == SourceMode::F2F && out_name.exists() {
        if config.force_overwrite {
            let _ = std::fs::remove_file(&out_name);
        } else {
            eprintln!(
                "{}: Output file {} already exists.",
                get_program_name().display(),
                out_name.display(),
            );
            setExit(1 as libc::c_int);
            return;
        }
    }
    if srcMode == SourceMode::F2F && !config.force_overwrite && {
        n = count_hardlinks(in_name);
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

    // Save the file's meta-info before we open it.
    // Doing it later means we mess up the access times.
    let metadata = match srcMode {
        SourceMode::F2F => match std::fs::metadata(in_name) {
            Ok(metadata) => Some(metadata),
            Err(_) => ioError(),
        },
        _ => None,
    };

    match srcMode {
        SourceMode::I2O => {
            input_stream = InputStream::Stdin(std::io::stdin());
            outStr = STDOUT!();
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
            outStr = STDOUT!();
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

            input_stream = match std::fs::File::open(in_name) {
                Ok(file) => InputStream::File(file),
                Err(e) => {
                    eprintln!(
                        "{}: Can't open input file {}: {}.",
                        get_program_name().display(),
                        in_name.display(),
                        display_os_error(e),
                    );
                    setExit(1 as libc::c_int);
                    return;
                }
            };
        }
        SourceMode::F2F => {
            outStr = fopen_output_safely(&out_name);
            if outStr.is_null() {
                eprintln!(
                    "{}: Can't create output file {}: {}.",
                    std::env::args().next().unwrap(),
                    in_name.display(),
                    display_last_os_error(),
                );
                setExit(1 as libc::c_int);
                return;
            }

            input_stream = match std::fs::File::open(in_name) {
                Ok(file) => InputStream::File(file),
                Err(e) => {
                    eprintln!(
                        "{}: Can't open input file {}: {}.",
                        get_program_name().display(),
                        in_name.display(),
                        display_os_error(e),
                    );
                    setExit(1 as libc::c_int);
                    return;
                }
            };
        }
    }
    if config.verbosity >= 1 as libc::c_int {
        eprint!("  {}: ", in_name.display());
        pad(in_name);
    }
    outputHandleJustInCase = outStr;
    delete_output_on_interrupt = true;
    compressStream(config, input_stream, outStr, metadata.as_ref());
    outputHandleJustInCase = std::ptr::null_mut::<FILE>();

    if let Some(metadata) = metadata {
        apply_saved_time_info_to_output_file(CStr::from_ptr(outName.as_mut_ptr()), metadata);
        delete_output_on_interrupt = false;
        if !keep_input_files && std::fs::remove_file(in_name).is_err() {
            ioError();
        }
    }
    delete_output_on_interrupt = false;
}

enum OutputStream {
    Stdout(std::io::Stdout),
    File(std::fs::File),
}

impl std::io::Write for OutputStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            OutputStream::Stdout(stdout) => stdout.write(buf),
            OutputStream::File(file) => file.write(buf),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            OutputStream::Stdout(stdout) => stdout.write_all(buf),
            OutputStream::File(file) => file.write_all(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            OutputStream::Stdout(stdout) => stdout.flush(),
            OutputStream::File(file) => file.flush(),
        }
    }
}

unsafe fn uncompress(config: &UncompressConfig, name: Option<&str>) {
    let inStr: *mut FILE;
    let output_stream;
    let n: u64;

    delete_output_on_interrupt = false;

    let mut cannot_guess = false;

    let in_name: PathBuf;
    let out_name: PathBuf;

    match (name, srcMode) {
        (_, SourceMode::I2O) => {
            in_name = PathBuf::from("(stdin)");
            out_name = PathBuf::from("(stdout)");

            copy_filename(inName.as_mut_ptr(), "(stdin)");
            copy_filename(outName.as_mut_ptr(), "(stdout)");
        }
        (Some(name), SourceMode::F2O) => {
            in_name = PathBuf::from(&name);
            out_name = PathBuf::from("(stdout)");

            copy_filename(inName.as_mut_ptr(), name);
            copy_filename(outName.as_mut_ptr(), "(stdout)");
        }
        (Some(name), SourceMode::F2F) => {
            in_name = PathBuf::from(name);
            copy_filename(inName.as_mut_ptr(), name);

            let mut name = name.to_owned();

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

            copy_filename(outName.as_mut_ptr(), &name);

            out_name = PathBuf::from(name);
        }
        (None, SourceMode::F2O | SourceMode::F2F) => panic_str("uncompress: bad modes\n"),
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

    if srcMode == SourceMode::F2F && !config.force_overwrite && not_a_standard_file(&in_name) {
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
        if config.force_overwrite {
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

    if srcMode == SourceMode::F2F && !config.force_overwrite && {
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

    // Save the file's meta-info before we open it.
    // Doing it later means we mess up the access times.
    let metadata = match srcMode {
        SourceMode::F2F => match std::fs::metadata(&in_name) {
            Ok(metadata) => Some(metadata),
            Err(_) => ioError(),
        },
        _ => None,
    };

    match srcMode {
        SourceMode::I2O => {
            inStr = STDIN!();
            output_stream = OutputStream::Stdout(std::io::stdout());
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
            output_stream = OutputStream::Stdout(std::io::stdout());
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

            let mut options = std::fs::File::options();
            options.write(true).create_new(true);

            output_stream = match options.open(&out_name) {
                Ok(file) => OutputStream::File(file),
                Err(e) => {
                    eprintln!(
                        "{}: Can't create output file {}: {}.",
                        get_program_name().display(),
                        out_name.display(),
                        display_os_error(e),
                    );
                    if !inStr.is_null() {
                        fclose(inStr);
                    }
                    setExit(1);
                    return;
                }
            };

            if inStr.is_null() {
                eprintln!(
                    "{}: Can't open input file {}: {}.",
                    get_program_name().display(),
                    in_name.display(),
                    display_last_os_error(),
                );
                setExit(1);
                return;
            }
        }
    }

    if config.verbosity >= 1 {
        eprint!("  {}: ", in_name.display(),);
        pad(&in_name);
    }

    /*--- Now the input and output handles are sane.  Do the Biz. ---*/
    delete_output_on_interrupt = true;
    let magicNumberOK = uncompressStream(config, inStr, output_stream, metadata.as_ref());
    outputHandleJustInCase = std::ptr::null_mut::<FILE>();

    /*--- If there was an I/O error, we won't get here. ---*/
    if magicNumberOK {
        if let Some(metadata) = metadata {
            apply_saved_time_info_to_output_file(CStr::from_ptr(outName.as_mut_ptr()), metadata);
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
        if config.verbosity >= 1 {
            eprintln!("done");
        }
    } else {
        setExit(2);
        if config.verbosity >= 1 {
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

unsafe fn testf(config: &UncompressConfig, name: Option<&str>) {
    let inStr: *mut FILE;
    delete_output_on_interrupt = false;

    copy_filename(outName.as_mut_ptr(), "(none)");

    let in_name;

    match (name, srcMode) {
        (_, SourceMode::I2O) => {
            in_name = PathBuf::from("(stdin)");
            copy_filename(inName.as_mut_ptr(), "(stdin)");
        }
        (Some(name), SourceMode::F2O) => {
            in_name = PathBuf::from(&name);
            copy_filename(inName.as_mut_ptr(), name);
        }
        (Some(name), SourceMode::F2F) => {
            in_name = PathBuf::from(&name);
            copy_filename(inName.as_mut_ptr(), name);
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
            inStr = STDIN!();
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
    if config.verbosity >= 1 {
        eprint!("  {}: ", in_name.display());
        pad(&in_name);
    }
    outputHandleJustInCase = std::ptr::null_mut::<FILE>();
    let allOK = testStream(config, inStr);
    if allOK && config.verbosity >= 1 {
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
    keep_input_files = false;
    noisy = true;
    test_fails_exists = false;
    unz_fails_exist = false;
    numFileNames = 0;
    numFilesProcessed = 0;
    delete_output_on_interrupt = false;

    exitValue = 0;

    // general config
    let mut verbosity = 0;
    let mut force_overwrite = false;

    // compress config
    let mut blockSize100k = 9;
    let mut workFactor = 30;

    // uncompress config
    let mut decompress_mode = DecompressMode::Fast;

    signal(
        SIGSEGV,
        mySIGSEGVorSIGBUScatcher as unsafe extern "C" fn(libc::c_int) as usize,
    );
    #[cfg(not(target_os = "windows"))]
    signal(
        libc::SIGBUS,
        mySIGSEGVorSIGBUScatcher as unsafe extern "C" fn(libc::c_int) as usize,
    );

    copy_filename(inName.as_mut_ptr(), "(none)");
    copy_filename(outName.as_mut_ptr(), "(none)");

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

    LONGEST_FILENAME.store(7, Ordering::Relaxed);
    numFileNames = 0;
    let mut decode = true;

    for name in &arg_list {
        if name == "--" {
            decode = false;
        } else if !(name.starts_with('-') && decode) {
            numFileNames += 1;
            LONGEST_FILENAME.fetch_max(name.len(), Ordering::Relaxed);
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
            SIGINT,
            mySignalCatcher as unsafe extern "C" fn(IntNative) as usize,
        );
        signal(
            SIGTERM,
            mySignalCatcher as unsafe extern "C" fn(IntNative) as usize,
        );
        #[cfg(not(target_os = "windows"))]
        signal(
            libc::SIGHUP,
            mySignalCatcher as unsafe extern "C" fn(IntNative) as usize,
        );
    }

    match opMode {
        OperationMode::Zip => {
            let config = CompressConfig {
                blockSize100k,
                verbosity,
                workFactor,
                force_overwrite,
            };

            if srcMode == SourceMode::I2O {
                compress(&config, None);
            } else {
                decode = true;
                for name in arg_list {
                    if name == "--" {
                        decode = false;
                    } else if !(name.starts_with('-') && decode) {
                        numFilesProcessed += 1;
                        compress(&config, Some(name.as_str()));
                    }
                }
            }
        }
        OperationMode::Unzip => {
            let config = UncompressConfig {
                verbosity,
                decompress_mode,
                force_overwrite,
            };

            unz_fails_exist = false;
            if srcMode == SourceMode::I2O {
                uncompress(&config, None);
            } else {
                decode = true;
                for name in arg_list {
                    if name == "--" {
                        decode = false;
                    } else if !(name.starts_with('-') && decode) {
                        numFilesProcessed += 1;
                        uncompress(&config, Some(name.as_str()));
                    }
                }
            }
            if unz_fails_exist {
                setExit(2);
                exit(exitValue);
            }
        }
        OperationMode::Test => {
            let config = UncompressConfig {
                verbosity,
                decompress_mode,
                force_overwrite,
            };

            test_fails_exists = false;
            if srcMode == SourceMode::I2O {
                testf(&config, None);
            } else {
                decode = true;
                for name in arg_list {
                    if name == "--" {
                        decode = false;
                    } else if !(name.starts_with('-') && decode) {
                        numFilesProcessed += 1;
                        testf(&config, Some(name.as_str()));
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
