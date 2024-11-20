#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::ffi::{c_char, c_int, CStr, OsStr};
use std::fs::Metadata;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};

use libbzip2_rs_sys::{
    BZ2_bzRead, BZ2_bzReadClose, BZ2_bzReadGetUnused, BZ2_bzReadOpen, BZ2_bzWrite,
    BZ2_bzWriteClose64, BZ2_bzWriteOpen, BZ2_bzlibVersion, BZFILE,
};

use libc::{
    fclose, ferror, fflush, fgetc, fileno, fread, rewind, signal, ungetc, FILE, SIGINT, SIGTERM,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum DecompressMode {
    Fast = 0,
    Small = 1,
}

// NOTE: we use Ordering::SeqCst to synchronize with the signal handler
static delete_output_on_interrupt: AtomicBool = AtomicBool::new(false);

static noisy: AtomicBool = AtomicBool::new(false);
static numFileNames: AtomicI32 = AtomicI32::new(0);
static numFilesProcessed: AtomicI32 = AtomicI32::new(0);
static exitValue: AtomicI32 = AtomicI32::new(0);

#[derive(Clone)]
struct Config {
    program_name: PathBuf,

    input: PathBuf,
    output: PathBuf,

    // general
    noisy: bool,
    verbosity: i32,
    force_overwrite: bool,
    keep_input_files: bool,

    // compress
    blockSize100k: i32,
    workFactor: i32,

    // uncompress
    decompress_mode: DecompressMode,
}

impl Config {
    fn with_input(
        &mut self,
        operation: OperationMode,
        name: Option<&str>,
        source_mode: SourceMode,
    ) {
        match operation {
            OperationMode::Zip => self.with_compress_input(name, source_mode),
            OperationMode::Unzip => self.with_uncompress_input(name, source_mode),
            OperationMode::Test => self.with_test_input(name, source_mode),
        }

        const FILE_NAME_LEN: usize = 1034;

        if self.input.as_os_str().len() >= FILE_NAME_LEN - 10 {
            eprint!(
                concat!(
                    "bzip2: file name\n",
                    "`{}'\n",
                    "is suspiciously (more than {} chars) long.\n",
                    "Try using a reasonable file name instead.  Sorry! :-)\n",
                ),
                self.input.display(),
                FILE_NAME_LEN - 10
            );

            exit(1);
        }

        if self.output.as_os_str().len() >= FILE_NAME_LEN - 10 {
            eprint!(
                concat!(
                    "bzip2: file name\n",
                    "`{}'\n",
                    "is suspiciously (more than {} chars) long.\n",
                    "Try using a reasonable file name instead.  Sorry! :-)\n",
                ),
                self.output.display(),
                FILE_NAME_LEN - 10
            );


            exit(1);
        }
    }

    fn with_compress_input(&mut self, name: Option<&str>, mode: SourceMode) {
        match (name, mode) {
            (_, SourceMode::I2O) => {
                self.input = Path::new("(stdin)").to_owned();
                self.output = Path::new("(stdout)").to_owned();
            }
            (Some(name), SourceMode::F2O) => {
                self.input = Path::new(name).to_owned();
                self.output = Path::new("(stdout)").to_owned();
            }
            (Some(name), SourceMode::F2F) => {
                self.input = Path::new(name).to_owned();
                self.output = PathBuf::from(format!("{name}.bz2"));
            }
            (None, SourceMode::F2O | SourceMode::F2F) => panic!("compress: bad modes"),
        }
    }

    fn with_uncompress_input(&mut self, name: Option<&str>, mode: SourceMode) {
        match (name, mode) {
            (_, SourceMode::I2O) => {
                self.input = Path::new("(stdin)").to_owned();
                self.output = Path::new("(stdout)").to_owned();
            }
            (Some(name), SourceMode::F2O) => {
                self.input = Path::new(name).to_owned();
                self.output = Path::new("(stdout)").to_owned();
            }
            (Some(name), SourceMode::F2F) => {
                self.input = Path::new(name).to_owned();

                let mut name = name.to_owned();

                'blk: {
                    for (old, new) in Z_SUFFIX.iter().zip(UNZ_SUFFIX) {
                        if name.ends_with(old) {
                            name.truncate(name.len() - old.len());
                            name += new;
                            break 'blk;
                        }
                    }

                    name += ".out";
                };

                self.output = PathBuf::from(name);
            }
            (None, SourceMode::F2O | SourceMode::F2F) => panic!("uncompress: bad modes"),
        }
    }

    fn with_test_input(&mut self, name: Option<&str>, mode: SourceMode) {
        self.output = Path::new("(none)").to_owned();

        match (name, mode) {
            (_, SourceMode::I2O) => {
                self.input = Path::new("(stdin)").to_owned();
            }
            (Some(name), SourceMode::F2O) => {
                self.input = Path::new(name).to_owned();
            }
            (Some(name), SourceMode::F2F) => {
                self.input = Path::new(name).to_owned();
            }
            (None, SourceMode::F2O | SourceMode::F2F) => panic!("testf: bad modes"),
        }
    }
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

static mut opMode: OperationMode = OperationMode::Zip;
static mut srcMode: SourceMode = SourceMode::I2O;

// NOTE: we use Ordering::SeqCst to synchronize with the signal handler
static LONGEST_FILENAME: AtomicUsize = AtomicUsize::new(0);

// this should eventually be removed and just passed down into functions from the root
fn get_program_name() -> PathBuf {
    let program_path: PathBuf = std::env::args_os().next().unwrap().into();
    PathBuf::from(program_path.file_name().unwrap())
}

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
    if c == -1 {
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
    config: &Config,
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

    set_binary_mode(config, zStream);

    if ferror(zStream) != 0 {
        // diverges
        ioError(config)
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
                Err(e) => exit_with_io_error(config, e),
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
            0,
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
            ioError(config)
        }
        ret = fflush(zStream);
        if ret == libc::EOF {
            // diverges
            ioError(config)
        }

        if let Some(metadata) = metadata {
            set_permissions(config, zStream, metadata);
            ret = fclose(zStream);
            if ret == libc::EOF {
                // diverges
                ioError(config)
            }
        }

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
        libbzip2_rs_sys::BZ_MEM_ERROR => outOfMemory(config),
        libbzip2_rs_sys::BZ_IO_ERROR => ioError(config),
        _ => panic_str(config, "compress:unexpected error"),
    }
}

unsafe fn uncompressStream(
    config: &Config,
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

    set_binary_mode(config, zStream);

    if ferror(zStream) != 0 {
        // diverges
        ioError(config)
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
                if bzf.is_null() || bzerr != 0 {
                    state = State::ErrHandler;
                    continue 'outer;
                }
                streamNo += 1;

                while bzerr == 0 {
                    nread = BZ2_bzRead(
                        &mut bzerr,
                        bzf,
                        obuf.as_mut_ptr() as *mut libc::c_void,
                        5000,
                    );
                    if bzerr == libbzip2_rs_sys::BZ_DATA_ERROR_MAGIC {
                        state = State::TryCat;
                        continue 'outer;
                    }
                    if (bzerr == libbzip2_rs_sys::BZ_OK || bzerr == libbzip2_rs_sys::BZ_STREAM_END)
                        && nread > 0
                    {
                        if let Err(e) = stream.write_all(&obuf[..nread as usize]) {
                            exit_with_io_error(config, e) // diverges
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
                    panic_str(config, "decompress:bzReadGetUnused")
                }

                let unusedTmp = unusedTmpV as *mut u8;
                for i in 0..nUnused {
                    unused[i as usize] = *unusedTmp.offset(i as isize);
                }

                BZ2_bzReadClose(&mut bzerr, bzf);
                if bzerr != libbzip2_rs_sys::BZ_OK {
                    // diverges
                    panic_str(config, "decompress:bzReadGetUnused")
                }

                if nUnused == 0 && myfeof(zStream) {
                    state = State::CloseOk;
                    continue 'outer;
                }
            },
            State::CloseOk => {
                if ferror(zStream) != 0 {
                    // diverges
                    ioError(config)
                }

                if let Some(metadata) = metadata {
                    if let OutputStream::File(file) = &stream {
                        set_permissions_rust(config, file, metadata);
                    }
                }

                if let libc::EOF = fclose(zStream) {
                    ioError(config)
                }

                if let Err(e) = stream.flush() {
                    exit_with_io_error(config, e) // diverges
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
                            5000,
                            zStream,
                        ) as i32;
                        if ferror(zStream) != 0 {
                            // diverges
                            ioError(config)
                        }
                        if nread > 0 {
                            if let Err(e) = stream.write_all(&obuf[..nread as usize]) {
                                exit_with_io_error(config, e) // diverges
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
                    libbzip2_rs_sys::BZ_IO_ERROR => ioError(config),
                    libbzip2_rs_sys::BZ_DATA_ERROR => crcError(config),
                    libbzip2_rs_sys::BZ_MEM_ERROR => outOfMemory(config),
                    libbzip2_rs_sys::BZ_UNEXPECTED_EOF => compressedStreamEOF(config),
                    libbzip2_rs_sys::BZ_DATA_ERROR_MAGIC => {
                        if zStream != STDIN!() {
                            fclose(zStream);
                        }

                        if streamNo == 1 {
                            return false;
                        } else {
                            if config.noisy {
                                eprintln!(
                                    "\n{}: {}: trailing garbage after EOF ignored",
                                    config.program_name.display(),
                                    config.input.display(),
                                );
                            }
                            return true;
                        }
                    }
                    _ => panic_str(config, "decompress:unexpected error"),
                }
            }
        }
    }
}

unsafe fn testStream(config: &Config, zStream: *mut FILE) -> bool {
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
                ioError(config)
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
                panic_str(config, "test:bzReadGetUnused");
            }

            let unusedTmp = unusedTmpV as *mut u8;
            i = 0;
            while i < nUnused {
                unused[i as usize] = *unusedTmp.offset(i as isize);
                i += 1;
            }

            BZ2_bzReadClose(&mut bzerr, bzf);
            if bzerr != libbzip2_rs_sys::BZ_OK {
                panic_str(config, "test:bzReadClose");
            }
            if nUnused == 0 && myfeof(zStream) {
                break;
            }
        }

        if ferror(zStream) != 0 {
            ioError(config) // diverges
        }
        if fclose(zStream) == libc::EOF {
            ioError(config) // diverges
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
            config.program_name.display(),
            config.input.display(),
        );
    }
    match bzerr {
        libbzip2_rs_sys::BZ_CONFIG_ERROR => configError(),
        libbzip2_rs_sys::BZ_IO_ERROR => ioError(config),
        libbzip2_rs_sys::BZ_DATA_ERROR => {
            eprintln!("data integrity (CRC) error in data");
            false
        }
        libbzip2_rs_sys::BZ_MEM_ERROR => outOfMemory(config),
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
                if config.noisy {
                    eprintln!("trailing garbage after EOF ignored");
                }
                true
            }
        }
        _ => panic_str(config, "test:unexpected error"),
    }
}

fn setExit(v: i32) {
    exitValue.fetch_max(v, Ordering::SeqCst);
}

fn cadvise() {
    if noisy.load(Ordering::SeqCst) {
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

fn showFileNames(config: &Config) {
    if noisy.load(Ordering::SeqCst) {
        eprintln!(
            "\tInput file = {}, output file = {}",
            config.input.display(),
            config.output.display(),
        );
    }
}

fn cleanUpAndFail(config: &Config, ec: i32) -> ! {
    if unsafe {
        srcMode == SourceMode::F2F
            && opMode != OperationMode::Test
            && delete_output_on_interrupt.load(Ordering::SeqCst)
    } {
        if config.input.exists() {
            if noisy.load(Ordering::SeqCst) {
                eprintln!(
                    "{}: Deleting output file {}, if it exists.",
                    config.program_name.display(),
                    config.output.display(),
                );
            }
            // This should work even on Windows as we opened the output file with FILE_SHARE_DELETE
            if std::fs::remove_file(&config.output).is_err() {
                eprintln!(
                    "{}: WARNING: deletion of output file (apparently) failed.",
                    config.program_name.display(),
                );
            }
        } else {
            eprintln!(
                "{}: WARNING: deletion of output file suppressed",
                config.program_name.display(),
            );
            eprintln!(
                "{}:    since input file no longer exists.  Output file",
                config.program_name.display(),
            );
            eprintln!(
                "{}:    `{}' may be incomplete.",
                config.program_name.display(),
                config.output.display(),
            );
            eprintln!(
                "{}:    I suggest doing an integrity test (bzip2 -tv) of it.",
                config.program_name.display(),
            );
        }
    }
    if noisy.load(Ordering::SeqCst)
        && numFileNames.load(Ordering::SeqCst) > 0
        && numFilesProcessed.load(Ordering::SeqCst) < numFileNames.load(Ordering::SeqCst)
    {
        eprint!(
            concat!(
                "{}: WARNING: some files have not been processed:\n",
                "{}:    {} specified on command line, {} not processed yet.\n",
                "\n",
            ),
            config.program_name.display(),
            config.program_name.display(),
            numFileNames.load(Ordering::SeqCst),
            numFileNames.load(Ordering::SeqCst) - numFilesProcessed.load(Ordering::SeqCst),
        );
    }
    setExit(ec);
    exit(exitValue.load(Ordering::SeqCst));
}

fn panic_str(config: &Config, s: &str) -> ! {
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
    showFileNames(config);
    cleanUpAndFail(config, 3);
}

fn crcError(config: &Config) -> ! {
    eprintln!(
        "\n{}: Data integrity error when decompressing.",
        get_program_name().display(),
    );
    showFileNames(config);
    cadvise();
    cleanUpAndFail(config, 2);
}

fn compressedStreamEOF(config: &Config) -> ! {
    if noisy.load(Ordering::SeqCst) {
        eprint!(
            concat!(
                "\n",
                "{}: Compressed file ends unexpectedly;\n",
                "\tperhaps it is corrupted?  *Possible* reason follows.\n"
            ),
            get_program_name().display(),
        );
        eprintln!(
            "{}: {}",
            config.program_name.display(),
            display_last_os_error()
        );
        showFileNames(config);
        cadvise();
    }
    cleanUpAndFail(config, 2);
}

fn exit_with_io_error(config: &Config, error: std::io::Error) -> ! {
    eprintln!(
        "\n{}: I/O or other error, bailing out.  Possible reason follows.",
        get_program_name().display(),
    );
    eprintln!("{}", display_os_error(error));
    showFileNames(config);
    cleanUpAndFail(config, 1);
}

fn ioError(config: &Config) -> ! {
    eprintln!(
        "\n{}: I/O or other error, bailing out.  Possible reason follows.",
        get_program_name().display(),
    );
    eprintln!(
        "{}: {}",
        config.program_name.display(),
        display_last_os_error()
    );
    showFileNames(config);
    cleanUpAndFail(config, 1);
}

fn setup_ctrl_c_handler(config: &Config) {
    static ABORT_PIPE: AtomicI32 = AtomicI32::new(0);
    static TRIED_TO_CANCEL: AtomicBool = AtomicBool::new(false);

    let mut pair = [0; 2];
    unsafe {
        #[cfg(windows)]
        assert_eq!(libc::pipe(&mut pair as *mut [i32; 2] as *mut i32, 1, 0), 0);

        #[cfg(not(windows))]
        assert_eq!(libc::pipe(&mut pair as *mut [i32; 2] as *mut i32), 0);
    }

    ABORT_PIPE.store(pair[1], Ordering::Relaxed);

    let config = config.clone();
    std::thread::Builder::new()
        .name("ctrl-c listener".to_owned())
        .spawn(move || {
            unsafe {
                libc::read(pair[0], &mut 0u8 as *mut u8 as *mut _, 1);
            }
            eprintln!(
                "\n{}: Control-C or similar caught, quitting.",
                config.program_name.display(),
            );
            cleanUpAndFail(&config, 1);
        })
        .unwrap();

    unsafe extern "C" fn signal_handler(_: libc::c_int) {
        if TRIED_TO_CANCEL.swap(true, Ordering::SeqCst) {
            // The previous ctrl-c usage didn't cause the process to exit yet. Exit immediately to
            // avoid the user from getting stuck.
            libc::exit(1);
        }

        unsafe {
            if libc::write(
                ABORT_PIPE.load(Ordering::Relaxed),
                &0u8 as *const u8 as *const _,
                1,
            ) != 1
            {
                libc::abort();
            }
        }
    }

    unsafe {
        signal(
            SIGINT,
            signal_handler as unsafe extern "C" fn(c_int) as usize,
        );
        signal(
            SIGTERM,
            signal_handler as unsafe extern "C" fn(c_int) as usize,
        );
        #[cfg(not(target_os = "windows"))]
        signal(
            libc::SIGHUP,
            signal_handler as unsafe extern "C" fn(c_int) as usize,
        );
    }
}

fn outOfMemory(config: &Config) -> ! {
    eprintln!(
        "\n{}: couldn't allocate enough memory",
        get_program_name().display(),
    );
    showFileNames(config);
    cleanUpAndFail(config, 1);
}

fn configError() -> ! {
    const MSG: &str = concat!(
        "bzip2: I'm not configured correctly for this platform!\n",
        "\tI require Int32, Int16 and Char to have sizes\n",
        "\tof 4, 2 and 1 bytes to run properly, and they don't.\n",
        "\tProbably you can fix this by defining them correctly,\n",
        "\tand recompiling.  Bye!\n",
    );
    eprint!("{}", MSG);
    setExit(3);
    exit(exitValue.load(Ordering::SeqCst));
}

fn pad(s: &Path) {
    let len = s.as_os_str().as_encoded_bytes().len();
    let longest_filename = LONGEST_FILENAME.load(Ordering::SeqCst);

    if len >= longest_filename {
        return;
    }

    for _ in 1..=longest_filename - len {
        eprint!(" ");
    }
}

fn fopen_input(name: impl AsRef<Path>) -> *mut FILE {
    use std::ffi::CString;

    unsafe {
        // The CString really only needs to live for the duration of the fopen
        #[allow(temporary_cstring_as_ptr)]
        libc::fopen(
            CString::new(name.as_ref().to_str().unwrap())
                .unwrap()
                .as_ptr(),
            b"rb\0".as_ptr().cast::<c_char>(),
        )
    }
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

    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use std::os::windows::io::IntoRawHandle;

        let mut opts = std::fs::File::options();

        opts.write(true).create_new(true);

        // Allow the ctrl-c handler to delete the file
        const FILE_SHARE_DELETE: u32 = 4u32;
        opts.share_mode(FILE_SHARE_DELETE);

        let Ok(file) = opts.open(name) else {
            return std::ptr::null_mut::<FILE>();
        };
        let handle = file.into_raw_handle();

        let fd = unsafe { libc::open_osfhandle(handle as isize, 0) };
        let mode = b"wb\0".as_ptr().cast::<c_char>();
        let fp = unsafe { libc::fdopen(fd, mode) };
        if fp.is_null() {
            unsafe { libc::close(fd) };
        }
        fp
    }

    #[cfg(not(any(unix, windows)))]
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
fn count_hardlinks(_path: &Path) -> u64 {
    0 // FIXME
}

fn apply_saved_time_info_to_output_file(dst_name: &Path, metadata: Metadata) -> io::Result<()> {
    let times = std::fs::FileTimes::new()
        .set_accessed(metadata.accessed()?)
        .set_modified(metadata.modified()?);
    std::fs::OpenOptions::new()
        .write(true)
        .open(dst_name)?
        .set_times(times)
}

unsafe fn set_permissions(_config: &Config, _handle: *mut FILE, _metadata: &Metadata) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        let fd = fileno(_handle);
        if fd < 0 {
            // diverges
            ioError(_config)
        }

        let retVal = libc::fchmod(fd, _metadata.mode() as libc::mode_t);
        if retVal != 0 {
            ioError(_config);
        }

        // chown() will in many cases return with EPERM, which can be safely ignored.
        libc::fchown(fd, _metadata.uid(), _metadata.gid());
    }
}

fn set_permissions_rust(config: &Config, file: &std::fs::File, metadata: &Metadata) {
    if let Err(error) = file.set_permissions(metadata.permissions()) {
        exit_with_io_error(config, error);
    }
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
unsafe fn set_binary_mode(config: &Config, file: *mut FILE) {
    use std::ffi::c_int;

    extern "C" {
        fn _setmode(fd: c_int, mode: c_int) -> c_int;
    }

    if _setmode(fileno(file), libc::O_BINARY) == -1 {
        ioError(config);
    }
}

#[cfg(not(windows))]
/// Prevent Windows from mangling the read data.
unsafe fn set_binary_mode(_config: &Config, _file: *mut FILE) {}

unsafe fn compress(config: &Config) {
    delete_output_on_interrupt.store(false, Ordering::SeqCst);

    if srcMode != SourceMode::I2O && contains_dubious_chars_safe(&config.input) {
        if config.noisy {
            eprintln!(
                "{}: There are no files matching `{}'.",
                config.program_name.display(),
                config.input.display(),
            );
        }
        setExit(1);
        return;
    }
    if srcMode != SourceMode::I2O && !config.input.exists() {
        eprintln!(
            "{}: Can't open input file {}: {}.",
            config.program_name.display(),
            config.input.display(),
            display_last_os_error(),
        );
        setExit(1);
        return;
    }
    if let Some(extension) = config.input.extension() {
        for bz2_extension in Z_SUFFIX {
            if extension == OsStr::new(&bz2_extension[1..]) {
                if config.noisy {
                    eprintln!(
                        "{}: Input file {} already has {} suffix.",
                        config.program_name.display(),
                        config.input.display(),
                        &bz2_extension[1..],
                    );
                }
                setExit(1);
                return;
            }
        }
    }
    if (srcMode == SourceMode::F2F || srcMode == SourceMode::F2O) && config.input.is_dir() {
        eprintln!(
            "{}: Input file {} is a directory.",
            config.program_name.display(),
            config.input.display(),
        );
        setExit(1);
        return;
    }

    if srcMode == SourceMode::F2F && !config.force_overwrite && not_a_standard_file(&config.input) {
        if config.noisy {
            eprintln!(
                "{}: Input file {} is not a normal file.",
                config.program_name.display(),
                config.input.display(),
            );
        }
        setExit(1);
        return;
    }
    if srcMode == SourceMode::F2F && config.output.exists() {
        if config.force_overwrite {
            let _ = std::fs::remove_file(&config.output);
        } else {
            eprintln!(
                "{}: Output file {} already exists.",
                config.program_name.display(),
                config.output.display(),
            );
            setExit(1);
            return;
        }
    }

    if srcMode == SourceMode::F2F && !config.force_overwrite {
        match count_hardlinks(&config.input) {
            0 => { /* fallthrough */ }
            n => {
                eprintln!(
                    "{}: Input file {} has {} other link{}.",
                    config.program_name.display(),
                    config.input.display(),
                    n,
                    if n > 1 { "s" } else { "" },
                );
                setExit(1);
                return;
            }
        }
    }

    // Save the file's meta-info before we open it.
    // Doing it later means we mess up the access times.
    let metadata = match srcMode {
        SourceMode::F2F => match std::fs::metadata(&config.input) {
            Ok(metadata) => Some(metadata),
            Err(error) => exit_with_io_error(config, error),
        },
        _ => None,
    };

    let input_stream;
    let outStr: *mut FILE;

    match srcMode {
        SourceMode::I2O => {
            input_stream = InputStream::Stdin(std::io::stdin());
            outStr = STDOUT!();
            if std::io::stdout().is_terminal() {
                eprintln!(
                    "{}: I won't write compressed data to a terminal.",
                    config.program_name.display(),
                );
                eprintln!(
                    "{}: For help, type: `{} --help'.",
                    config.program_name.display(),
                    config.program_name.display(),
                );
                setExit(1);
                return;
            }
        }
        SourceMode::F2O => {
            outStr = STDOUT!();
            if std::io::stdout().is_terminal() {
                eprintln!(
                    "{}: I won't write compressed data to a terminal.",
                    config.program_name.display(),
                );
                eprintln!(
                    "{}: For help, type: `{} --help'.",
                    config.program_name.display(),
                    config.program_name.display(),
                );
                setExit(1);
                return;
            }

            input_stream = match std::fs::File::open(&config.input) {
                Ok(file) => InputStream::File(file),
                Err(e) => {
                    eprintln!(
                        "{}: Can't open input file {}: {}.",
                        config.program_name.display(),
                        config.input.display(),
                        display_os_error(e),
                    );
                    setExit(1);
                    return;
                }
            };
        }
        SourceMode::F2F => {
            outStr = fopen_output_safely(&config.output);
            if outStr.is_null() {
                eprintln!(
                    "{}: Can't create output file {}: {}.",
                    config.program_name.display(),
                    config.input.display(),
                    display_last_os_error(),
                );
                setExit(1);
                return;
            }

            input_stream = match std::fs::File::open(&config.input) {
                Ok(file) => InputStream::File(file),
                Err(e) => {
                    eprintln!(
                        "{}: Can't open input file {}: {}.",
                        config.program_name.display(),
                        config.input.display(),
                        display_os_error(e),
                    );
                    setExit(1);
                    return;
                }
            };
        }
    }
    if config.verbosity >= 1 {
        eprint!("  {}: ", config.input.display());
        pad(&config.input);
    }
    delete_output_on_interrupt.store(true, Ordering::SeqCst);
    compressStream(config, input_stream, outStr, metadata.as_ref());

    if let Some(metadata) = metadata {
        if let Err(error) = apply_saved_time_info_to_output_file(&config.output, metadata) {
            exit_with_io_error(config, error);
        }
        delete_output_on_interrupt.store(false, Ordering::SeqCst);
        if !config.keep_input_files {
            if let Err(error) = std::fs::remove_file(&config.input) {
                exit_with_io_error(config, error)
            }
        }
    }
    delete_output_on_interrupt.store(false, Ordering::SeqCst);
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

unsafe fn uncompress(config: &Config) -> bool {
    delete_output_on_interrupt.store(false, Ordering::SeqCst);

    let cannot_guess = config.output.extension() == Some(OsStr::new("out"));

    if srcMode != SourceMode::I2O && contains_dubious_chars_safe(&config.input) {
        if config.noisy {
            eprintln!(
                "%{}: There are no files matching `{}'.",
                config.program_name.display(),
                config.input.display(),
            );
        }
        setExit(1);
        return true;
    }

    if srcMode != SourceMode::I2O && !config.input.exists() {
        eprintln!(
            "{}: Can't open input file {}: {}.",
            config.program_name.display(),
            config.input.display(),
            display_last_os_error(),
        );
        setExit(1);
        return true;
    }

    if (srcMode == SourceMode::F2F || srcMode == SourceMode::F2O) && config.input.is_dir() {
        eprintln!(
            "{}: Input file {} is a directory.",
            config.program_name.display(),
            config.input.display(),
        );
        setExit(1);
        return true;
    }

    if srcMode == SourceMode::F2F && !config.force_overwrite && not_a_standard_file(&config.input) {
        if config.noisy {
            eprintln!(
                "{}: Input file {} is not a normal file.",
                config.program_name.display(),
                config.input.display(),
            );
        }
        setExit(1);
        return true;
    }

    if cannot_guess && config.noisy {
        // just a warning, no return
        eprintln!(
            "{}: Can't guess original name for {} -- using {}",
            config.program_name.display(),
            config.input.display(),
            config.output.display(),
        );
    }

    if srcMode == SourceMode::F2F && config.output.exists() {
        if config.force_overwrite {
            let _ = std::fs::remove_file(&config.output);
        } else {
            eprintln!(
                "{}: Output file {} already exists.",
                config.program_name.display(),
                config.output.display(),
            );
            setExit(1);
            return true;
        }
    }

    if srcMode == SourceMode::F2F && !config.force_overwrite {
        match count_hardlinks(&config.input) {
            0 => { /* fallthrough */ }
            n => {
                eprintln!(
                    "{}: Input file {} has {} other link{}.",
                    config.program_name.display(),
                    config.input.display(),
                    n,
                    if n > 1 { "s" } else { "" },
                );
                setExit(1);
                return true;
            }
        }
    }

    // Save the file's meta-info before we open it.
    // Doing it later means we mess up the access times.
    let metadata = match srcMode {
        SourceMode::F2F => match std::fs::metadata(&config.input) {
            Ok(metadata) => Some(metadata),
            Err(error) => exit_with_io_error(config, error),
        },
        _ => None,
    };

    let inStr: *mut FILE;
    let output_stream;

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
                    program_name = config.program_name.display(),
                );
                setExit(1);
                return true;
            }
        }
        SourceMode::F2O => {
            inStr = fopen_input(&config.input);
            output_stream = OutputStream::Stdout(std::io::stdout());
            if inStr.is_null() {
                eprintln!(
                    "{}: Can't open input file {}: {}.",
                    config.program_name.display(),
                    config.input.display(),
                    display_last_os_error(),
                );
                if !inStr.is_null() {
                    // this is unreachable, but it exists in the original C source code
                    fclose(inStr);
                }
                setExit(1);
                return true;
            }
        }
        SourceMode::F2F => {
            inStr = fopen_input(&config.input);

            let mut options = std::fs::File::options();
            options.write(true).create_new(true);

            output_stream = match options.open(&config.output) {
                Ok(file) => OutputStream::File(file),
                Err(e) => {
                    eprintln!(
                        "{}: Can't create output file {}: {}.",
                        config.program_name.display(),
                        config.output.display(),
                        display_os_error(e),
                    );
                    if !inStr.is_null() {
                        fclose(inStr);
                    }
                    setExit(1);
                    return true;
                }
            };

            if inStr.is_null() {
                eprintln!(
                    "{}: Can't open input file {}: {}.",
                    config.program_name.display(),
                    config.input.display(),
                    display_last_os_error(),
                );
                setExit(1);
                return true;
            }
        }
    }

    if config.verbosity >= 1 {
        eprint!("  {}: ", config.input.display());
        pad(&config.input);
    }

    /*--- Now the input and output handles are sane.  Do the Biz. ---*/
    delete_output_on_interrupt.store(true, Ordering::SeqCst);
    let magicNumberOK = uncompressStream(config, inStr, output_stream, metadata.as_ref());

    /*--- If there was an I/O error, we won't get here. ---*/
    if magicNumberOK {
        if let Some(metadata) = metadata {
            if let Err(error) = apply_saved_time_info_to_output_file(&config.output, metadata) {
                exit_with_io_error(config, error);
            }
            delete_output_on_interrupt.store(false, Ordering::SeqCst);
            if !config.keep_input_files {
                if let Err(error) = std::fs::remove_file(&config.input) {
                    exit_with_io_error(config, error);
                }
            }
        }
    } else {
        delete_output_on_interrupt.store(false, Ordering::SeqCst);
        if srcMode == SourceMode::F2F {
            if let Err(error) = std::fs::remove_file(&config.output) {
                exit_with_io_error(config, error);
            }
        }
    }

    delete_output_on_interrupt.store(false, Ordering::SeqCst);

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
                config.program_name.display(),
                config.input.display(),
            );
        }
    };

    magicNumberOK
}

unsafe fn testf(config: &Config) -> bool {
    delete_output_on_interrupt.store(false, Ordering::SeqCst);

    if srcMode != SourceMode::I2O && contains_dubious_chars_safe(&config.input) {
        if config.noisy {
            eprintln!(
                "{}: There are no files matching `{}'.",
                config.program_name.display(),
                config.input.display(),
            );
        }
        setExit(1);
        return true;
    }
    if srcMode != SourceMode::I2O && !config.input.exists() {
        eprintln!(
            "{}: Can't open input {}: {}.",
            config.program_name.display(),
            config.input.display(),
            display_last_os_error(),
        );
        setExit(1);
        return true;
    }
    if srcMode != SourceMode::I2O && config.input.is_dir() {
        eprintln!(
            "{}: Input file {} is a directory.",
            config.program_name.display(),
            config.input.display(),
        );
        setExit(1);
        return true;
    }

    let inStr: *mut FILE;
    match srcMode {
        SourceMode::I2O => {
            if std::io::stdin().is_terminal() {
                eprintln!(
                    "{}: I won't read compressed data from a terminal.",
                    config.program_name.display(),
                );
                eprintln!(
                    "{}: For help, type: `{} --help'.",
                    config.program_name.display(),
                    config.program_name.display(),
                );
                setExit(1);
                return true;
            }
            inStr = STDIN!();
        }
        SourceMode::F2O | SourceMode::F2F => {
            inStr = fopen_input(&config.input);
            if inStr.is_null() {
                eprintln!(
                    "{}: Can't open input {}: {}.",
                    config.program_name.display(),
                    config.input.display(),
                    display_last_os_error(),
                );
                setExit(1);
                return true;
            }
        }
    }
    if config.verbosity >= 1 {
        eprint!("  {}: ", config.input.display());
        pad(&config.input);
    }
    let allOK = testStream(config, inStr);
    if allOK && config.verbosity >= 1 {
        eprintln!("ok");
    }

    allOK
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

unsafe fn main_0(program_path: &Path) -> c_int {
    let program_name = Path::new(program_path.file_name().unwrap());

    noisy.store(true, Ordering::SeqCst);
    numFileNames.store(0, Ordering::SeqCst);
    numFilesProcessed.store(0, Ordering::SeqCst);
    delete_output_on_interrupt.store(false, Ordering::SeqCst);

    exitValue.store(0, Ordering::SeqCst);

    // general config
    let mut verbosity = 0;
    let mut force_overwrite = false;
    let mut keep_input_files = false;

    // compress config
    let mut blockSize100k = 9;
    let mut workFactor = 30;

    // uncompress config
    let mut decompress_mode = DecompressMode::Fast;

    let mut arg_list = Vec::with_capacity(16);

    if let Ok(val) = std::env::var("BZIP2") {
        arg_list.extend(val.split_ascii_whitespace().map(|s| s.to_owned()));
    }

    if let Ok(val) = std::env::var("BZIP") {
        arg_list.extend(val.split_ascii_whitespace().map(|s| s.to_owned()));
    }

    arg_list.extend(std::env::args().skip(1));

    LONGEST_FILENAME.store(7, Ordering::SeqCst);
    numFileNames.store(0, Ordering::SeqCst);
    let mut decode = true;

    for name in &arg_list {
        if name == "--" {
            decode = false;
        } else if !(name.starts_with('-') && decode) {
            numFileNames.fetch_add(1, Ordering::SeqCst);
            LONGEST_FILENAME.fetch_max(name.len(), Ordering::SeqCst);
        }
    }

    srcMode = match numFileNames.load(Ordering::SeqCst) {
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
        srcMode = match numFileNames.load(Ordering::SeqCst) {
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
                    b'q' => noisy.store(false, Ordering::SeqCst),
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
            "--quiet" => noisy.store(false, Ordering::SeqCst),
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

    if verbosity > 4 {
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
    if srcMode == SourceMode::F2O && numFileNames.load(Ordering::SeqCst) == 0 {
        srcMode = SourceMode::I2O;
    }
    if opMode != OperationMode::Zip {
        blockSize100k = 0;
    }

    let arg_list = &arg_list;

    let mut config = Config {
        program_name: program_name.to_owned(),

        input: Path::new("(none)").to_owned(),
        output: Path::new("(none)").to_owned(),

        // general
        noisy: noisy.load(Ordering::SeqCst),
        verbosity,
        force_overwrite,
        keep_input_files,

        // compress
        blockSize100k,
        workFactor,

        // uncompress
        decompress_mode,
    };

    if srcMode == SourceMode::F2F {
        setup_ctrl_c_handler(&config);
    }

    match opMode {
        OperationMode::Zip => {
            if srcMode == SourceMode::I2O {
                config.with_input(opMode, None, srcMode);
                compress(&config);
            } else {
                decode = true;
                for name in arg_list {
                    if name == "--" {
                        decode = false;
                    } else if !(name.starts_with('-') && decode) {
                        numFilesProcessed.fetch_add(1, Ordering::SeqCst);
                        config.with_input(opMode, Some(name.as_str()), srcMode);
                        compress(&config);
                    }
                }
            }
        }
        OperationMode::Unzip => {
            let mut all_ok = true;
            if srcMode == SourceMode::I2O {
                config.with_input(opMode, None, srcMode);
                all_ok &= uncompress(&config);
            } else {
                decode = true;
                for name in arg_list {
                    if name == "--" {
                        decode = false;
                    } else if !(name.starts_with('-') && decode) {
                        numFilesProcessed.fetch_add(1, Ordering::SeqCst);
                        config.with_input(opMode, Some(name.as_str()), srcMode);
                        all_ok &= uncompress(&config);
                    }
                }
            }
            if !all_ok {
                setExit(2);
                exit(exitValue.load(Ordering::SeqCst));
            }
        }
        OperationMode::Test => {
            let mut all_ok = true;
            if srcMode == SourceMode::I2O {
                config.with_input(opMode, None, srcMode);
                all_ok &= testf(&config);
            } else {
                decode = true;
                for name in arg_list {
                    if name == "--" {
                        decode = false;
                    } else if !(name.starts_with('-') && decode) {
                        numFilesProcessed.fetch_add(1, Ordering::SeqCst);
                        config.with_input(opMode, Some(name.as_str()), srcMode);
                        all_ok &= testf(&config);
                    }
                }
            }

            if !all_ok {
                if config.noisy {
                    eprintln!(concat!(
                        "\n",
                        "You can use the `bzip2recover' program to attempt to recover\n",
                        "data from undamaged sections of corrupted files.\n",
                    ));
                }
                setExit(2);
                exit(exitValue.load(Ordering::SeqCst));
            }
        }
    }

    exitValue.load(Ordering::SeqCst)
}

fn main() {
    let mut it = std::env::args_os();

    let program_name = PathBuf::from(it.next().unwrap());

    unsafe { exit(main_0(&program_name) as i32) }
}
