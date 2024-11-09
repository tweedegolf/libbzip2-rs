#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::ffi::{c_char, CStr};
use std::mem::zeroed;
use std::ptr;

use libbzip2_rs_sys::{
    BZ2_bzRead, BZ2_bzReadClose, BZ2_bzReadGetUnused, BZ2_bzReadOpen, BZ2_bzWrite,
    BZ2_bzWriteClose64, BZ2_bzWriteOpen, BZ2_bzlibVersion,
};

use libc::{
    _exit, close, exit, fclose, fdopen, ferror, fflush, fgetc, fileno, fopen, fprintf, fread, free,
    fwrite, getenv, isatty, malloc, open, perror, remove, rewind, signal, size_t, stat, strcat,
    strcmp, strcpy, strlen, strncmp, strncpy, strstr, ungetc, utimbuf, write, FILE,
};
extern "C" {
    static mut stdin: *mut FILE;
    static mut stdout: *mut FILE;
    static mut stderr: *mut FILE;
}
type Bool = libc::c_uchar;
type IntNative = libc::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
struct UInt64 {
    b: [u8; 8],
}
#[derive(Copy, Clone)]
#[repr(C)]
struct zzzz {
    name: *mut c_char,
    link: *mut zzzz,
}
type Cell = zzzz;
static mut verbosity: i32 = 0;
static mut keepInputFiles: Bool = 0;
static mut smallMode: Bool = 0;
static mut deleteOutputOnInterrupt: Bool = 0;
static mut forceOverwrite: Bool = 0;
static mut testFailsExist: Bool = 0;
static mut unzFailsExist: Bool = 0;
static mut noisy: Bool = 0;
static mut numFileNames: i32 = 0;
static mut numFilesProcessed: i32 = 0;
static mut blockSize100k: i32 = 0;
static mut exitValue: i32 = 0;
static mut opMode: i32 = 0;
static mut srcMode: i32 = 0;
static mut longestFileName: i32 = 0;
static mut inName: [c_char; 1034] = [0; 1034];
static mut outName: [c_char; 1034] = [0; 1034];
static mut tmpName: [c_char; 1034] = [0; 1034];
static mut progName: *mut c_char = ptr::null_mut();
static mut progNameReally: [c_char; 1034] = [0; 1034];
static mut outputHandleJustInCase: *mut FILE = ptr::null_mut();
static mut workFactor: i32 = 0;
unsafe fn uInt64_from_UInt32s(n: *mut UInt64, lo32: u32, hi32: u32) {
    (*n).b[7 as libc::c_int as usize] =
        (hi32 >> 24 as libc::c_int & 0xff as libc::c_int as libc::c_uint) as u8;
    (*n).b[6 as libc::c_int as usize] =
        (hi32 >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint) as u8;
    (*n).b[5 as libc::c_int as usize] =
        (hi32 >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint) as u8;
    (*n).b[4 as libc::c_int as usize] = (hi32 & 0xff as libc::c_int as libc::c_uint) as u8;
    (*n).b[3 as libc::c_int as usize] =
        (lo32 >> 24 as libc::c_int & 0xff as libc::c_int as libc::c_uint) as u8;
    (*n).b[2 as libc::c_int as usize] =
        (lo32 >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint) as u8;
    (*n).b[1 as libc::c_int as usize] =
        (lo32 >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint) as u8;
    (*n).b[0 as libc::c_int as usize] = (lo32 & 0xff as libc::c_int as libc::c_uint) as u8;
}
unsafe fn uInt64_to_double(n: *mut UInt64) -> libc::c_double {
    let mut base: libc::c_double = 1.0f64;
    let mut sum: libc::c_double = 0.0f64;
    let mut i = 0 as libc::c_int;
    while i < 8 as libc::c_int {
        sum += base * (*n).b[i as usize] as libc::c_double;
        base *= 256.0f64;
        i += 1;
    }
    sum
}
unsafe fn uInt64_isZero(n: *mut UInt64) -> Bool {
    let mut i = 0 as libc::c_int;
    while i < 8 as libc::c_int {
        if (*n).b[i as usize] as libc::c_int != 0 as libc::c_int {
            return 0 as Bool;
        }
        i += 1;
    }
    1 as Bool
}
unsafe fn uInt64_qrm10(n: *mut UInt64) -> i32 {
    let mut rem = 0 as libc::c_int as u32;
    let mut i = 7 as libc::c_int;
    while i >= 0 as libc::c_int {
        let tmp = rem
            .wrapping_mul(256 as libc::c_int as libc::c_uint)
            .wrapping_add((*n).b[i as usize] as libc::c_uint);
        (*n).b[i as usize] = tmp.wrapping_div(10 as libc::c_int as libc::c_uint) as u8;
        rem = tmp.wrapping_rem(10 as libc::c_int as libc::c_uint);
        i -= 1;
    }
    rem as i32
}
unsafe fn uInt64_toAscii(outbuf: *mut libc::c_char, n: *mut UInt64) {
    let mut buf: [u8; 32] = [0; 32];
    let mut nBuf: i32 = 0 as libc::c_int;
    let mut n_copy: UInt64 = *n;
    loop {
        let q = uInt64_qrm10(&mut n_copy);
        buf[nBuf as usize] = (q + '0' as i32) as u8;
        nBuf += 1;
        if uInt64_isZero(&mut n_copy) != 0 {
            break;
        }
    }
    *outbuf.offset(nBuf as isize) = 0 as libc::c_int as libc::c_char;
    let mut i = 0 as libc::c_int;
    while i < nBuf {
        *outbuf.offset(i as isize) = buf[(nBuf - i - 1 as libc::c_int) as usize] as libc::c_char;
        i += 1;
    }
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
    let mut current_block: u64;
    let mut ibuf: [u8; 5000] = [0; 5000];
    let mut nbytes_in_lo32: u32 = 0;
    let mut nbytes_in_hi32: u32 = 0;
    let mut nbytes_out_lo32: u32 = 0;
    let mut nbytes_out_hi32: u32 = 0;
    let mut bzerr: i32 = 0;
    let mut bzerr_dummy: i32 = 0;
    let mut ret: i32;
    if ferror(stream) == 0 && ferror(zStream) == 0 {
        let bzf = BZ2_bzWriteOpen(&mut bzerr, zStream, blockSize100k, verbosity, workFactor);
        if bzerr != 0 as libc::c_int {
            current_block = 6224343814938714352;
        } else {
            if verbosity >= 2 as libc::c_int {
                fprintf(stderr, b"\n\0" as *const u8 as *const libc::c_char);
            }
            loop {
                if myfeof(stream) != 0 {
                    current_block = 9606288038608642794;
                    break;
                }
                let nIbuf = fread(
                    ibuf.as_mut_ptr() as *mut libc::c_void,
                    core::mem::size_of::<u8>() as libc::size_t,
                    5000 as libc::c_int as libc::size_t,
                    stream,
                ) as i32;
                if ferror(stream) != 0 {
                    current_block = 4037297614742950260;
                    break;
                }
                if nIbuf > 0 as libc::c_int {
                    BZ2_bzWrite(
                        &mut bzerr,
                        bzf,
                        ibuf.as_mut_ptr() as *mut libc::c_void,
                        nIbuf,
                    );
                }
                if bzerr != 0 as libc::c_int {
                    current_block = 6224343814938714352;
                    break;
                }
            }
            match current_block {
                4037297614742950260 => {}
                6224343814938714352 => {}
                _ => {
                    BZ2_bzWriteClose64(
                        &mut bzerr,
                        bzf,
                        0 as libc::c_int,
                        &mut nbytes_in_lo32,
                        &mut nbytes_in_hi32,
                        &mut nbytes_out_lo32,
                        &mut nbytes_out_hi32,
                    );
                    if bzerr != 0 as libc::c_int {
                        current_block = 6224343814938714352;
                    } else if ferror(zStream) != 0 {
                        current_block = 4037297614742950260;
                    } else {
                        ret = fflush(zStream);
                        if ret == -1 as libc::c_int {
                            current_block = 4037297614742950260;
                        } else {
                            if zStream != stdout {
                                let fd: i32 = fileno(zStream);
                                if fd < 0 as libc::c_int {
                                    current_block = 4037297614742950260;
                                } else {
                                    applySavedFileAttrToOutputFile(fd);
                                    ret = fclose(zStream);
                                    outputHandleJustInCase = std::ptr::null_mut::<FILE>();
                                    if ret == -1 as libc::c_int {
                                        current_block = 4037297614742950260;
                                    } else {
                                        current_block = 9828876828309294594;
                                    }
                                }
                            } else {
                                current_block = 9828876828309294594;
                            }
                            match current_block {
                                4037297614742950260 => {}
                                _ => {
                                    outputHandleJustInCase = std::ptr::null_mut::<FILE>();
                                    if ferror(stream) != 0 {
                                        current_block = 4037297614742950260;
                                    } else {
                                        ret = fclose(stream);
                                        if ret == -1 as libc::c_int {
                                            current_block = 4037297614742950260;
                                        } else {
                                            if verbosity >= 1 as libc::c_int {
                                                if nbytes_in_lo32
                                                    == 0 as libc::c_int as libc::c_uint
                                                    && nbytes_in_hi32
                                                        == 0 as libc::c_int as libc::c_uint
                                                {
                                                    fprintf(
                                                        stderr,
                                                        b" no data compressed.\n\0" as *const u8
                                                            as *const libc::c_char,
                                                    );
                                                } else {
                                                    let mut buf_nin: [c_char; 32] = [0; 32];
                                                    let mut buf_nout: [c_char; 32] = [0; 32];
                                                    let mut nbytes_in: UInt64 =
                                                        UInt64 { b: [0; 8] };
                                                    let mut nbytes_out: UInt64 =
                                                        UInt64 { b: [0; 8] };
                                                    uInt64_from_UInt32s(
                                                        &mut nbytes_in,
                                                        nbytes_in_lo32,
                                                        nbytes_in_hi32,
                                                    );
                                                    uInt64_from_UInt32s(
                                                        &mut nbytes_out,
                                                        nbytes_out_lo32,
                                                        nbytes_out_hi32,
                                                    );
                                                    let nbytes_in_d =
                                                        uInt64_to_double(&mut nbytes_in);
                                                    let nbytes_out_d =
                                                        uInt64_to_double(&mut nbytes_out);
                                                    uInt64_toAscii(
                                                        buf_nin.as_mut_ptr(),
                                                        &mut nbytes_in,
                                                    );
                                                    uInt64_toAscii(
                                                        buf_nout.as_mut_ptr(),
                                                        &mut nbytes_out,
                                                    );
                                                    fprintf(
                                                        stderr,
                                                        b"%6.3f:1, %6.3f bits/byte, %5.2f%% saved, %s in, %s out.\n\0"
                                                            as *const u8 as *const libc::c_char,
                                                        nbytes_in_d / nbytes_out_d,
                                                        8.0f64 * nbytes_out_d / nbytes_in_d,
                                                        100.0f64 * (1.0f64 - nbytes_out_d / nbytes_in_d),
                                                        buf_nin.as_mut_ptr(),
                                                        buf_nout.as_mut_ptr(),
                                                    );
                                                }
                                            }
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        match current_block {
            4037297614742950260 => {}
            _ => {
                BZ2_bzWriteClose64(
                    &mut bzerr_dummy,
                    bzf,
                    1 as libc::c_int,
                    &mut nbytes_in_lo32,
                    &mut nbytes_in_hi32,
                    &mut nbytes_out_lo32,
                    &mut nbytes_out_hi32,
                );
                match bzerr {
                    -9 => {
                        current_block = 8758779949906403879;
                        match current_block {
                            281677055667329941 => {
                                panic(
                                    b"compress:unexpected error\0" as *const u8
                                        as *const libc::c_char,
                                );
                            }
                            17408806085946970511 => {
                                outOfMemory();
                            }
                            _ => {
                                configError();
                            }
                        }
                    }
                    -3 => {
                        current_block = 17408806085946970511;
                        match current_block {
                            281677055667329941 => {
                                panic(
                                    b"compress:unexpected error\0" as *const u8
                                        as *const libc::c_char,
                                );
                            }
                            17408806085946970511 => {
                                outOfMemory();
                            }
                            _ => {
                                configError();
                            }
                        }
                    }
                    -6 => {}
                    _ => {
                        current_block = 281677055667329941;
                        match current_block {
                            281677055667329941 => {
                                panic(
                                    b"compress:unexpected error\0" as *const u8
                                        as *const libc::c_char,
                                );
                            }
                            17408806085946970511 => {
                                outOfMemory();
                            }
                            _ => {
                                configError();
                            }
                        }
                    }
                }
            }
        }
    }
    ioError();
}
unsafe fn uncompressStream(zStream: *mut FILE, stream: *mut FILE) -> Bool {
    let mut current_block: u64;
    let mut bzf: *mut libc::c_void;
    let mut bzerr: i32 = 0;
    let mut bzerr_dummy: i32 = 0;
    let mut ret: i32;
    let mut nread: i32;
    let mut i: i32;
    let mut obuf: [u8; 5000] = [0; 5000];
    let mut unused: [u8; 5000] = [0; 5000];
    let mut unusedTmpV: *mut libc::c_void = std::ptr::null_mut::<libc::c_void>();
    let mut nUnused: libc::c_int = 0;
    let mut streamNo: libc::c_int = 0;
    if ferror(stream) == 0 && ferror(zStream) == 0 {
        's_37: loop {
            bzf = BZ2_bzReadOpen(
                &mut bzerr,
                zStream,
                verbosity,
                smallMode as libc::c_int,
                unused.as_mut_ptr() as *mut libc::c_void,
                nUnused,
            );
            if bzf.is_null() || bzerr != 0 as libc::c_int {
                current_block = 12043696510059011606;
                break;
            }
            streamNo += 1;
            while bzerr == 0 as libc::c_int {
                nread = BZ2_bzRead(
                    &mut bzerr,
                    bzf,
                    obuf.as_mut_ptr() as *mut libc::c_void,
                    5000 as libc::c_int,
                );
                if bzerr == -5 as libc::c_int {
                    current_block = 16997752893149199978;
                    break 's_37;
                }
                if (bzerr == 0 as libc::c_int || bzerr == 4 as libc::c_int)
                    && nread > 0 as libc::c_int
                {
                    fwrite(
                        obuf.as_mut_ptr() as *const libc::c_void,
                        core::mem::size_of::<u8>() as libc::size_t,
                        nread as libc::size_t,
                        stream,
                    );
                }
                if ferror(stream) != 0 {
                    current_block = 6432526541220421294;
                    break 's_37;
                }
            }
            if bzerr != 4 as libc::c_int {
                current_block = 12043696510059011606;
                break;
            }
            BZ2_bzReadGetUnused(&mut bzerr, bzf, &mut unusedTmpV, &mut nUnused);
            if bzerr != 0 as libc::c_int {
                panic(b"decompress:bzReadGetUnused\0" as *const u8 as *const libc::c_char);
            }
            let unusedTmp = unusedTmpV as *mut u8;
            i = 0 as libc::c_int;
            while i < nUnused {
                unused[i as usize] = *unusedTmp.offset(i as isize);
                i += 1;
            }
            BZ2_bzReadClose(&mut bzerr, bzf);
            if bzerr != 0 as libc::c_int {
                panic(b"decompress:bzReadGetUnused\0" as *const u8 as *const libc::c_char);
            }
            if nUnused == 0 as libc::c_int && myfeof(zStream) as libc::c_int != 0 {
                current_block = 17487785624869370758;
                break;
            }
        }
        match current_block {
            6432526541220421294 => {}
            _ => {
                if current_block == 16997752893149199978 {
                    if forceOverwrite != 0 {
                        rewind(zStream);
                        loop {
                            if myfeof(zStream) != 0 {
                                current_block = 17487785624869370758;
                                break;
                            }
                            nread = fread(
                                obuf.as_mut_ptr() as *mut libc::c_void,
                                core::mem::size_of::<u8>() as libc::size_t,
                                5000 as libc::c_int as libc::size_t,
                                zStream,
                            ) as i32;
                            if ferror(zStream) != 0 {
                                current_block = 6432526541220421294;
                                break;
                            }
                            if nread > 0 as libc::c_int {
                                fwrite(
                                    obuf.as_mut_ptr() as *const libc::c_void,
                                    core::mem::size_of::<u8>() as libc::size_t,
                                    nread as libc::size_t,
                                    stream,
                                );
                            }
                            if ferror(stream) != 0 {
                                current_block = 6432526541220421294;
                                break;
                            }
                        }
                    } else {
                        current_block = 12043696510059011606;
                    }
                }
                match current_block {
                    6432526541220421294 => {}
                    _ => match current_block {
                        12043696510059011606 => {
                            BZ2_bzReadClose(&mut bzerr_dummy, bzf);
                            match bzerr {
                                -9 => {
                                    current_block = 13095676725198100986;
                                    match current_block {
                                        8365064614624041636 => {
                                            panic(
                                                b"decompress:unexpected error\0" as *const u8
                                                    as *const libc::c_char,
                                            );
                                        }
                                        13095676725198100986 => {
                                            configError();
                                        }
                                        14054180689323133469 => {
                                            crcError();
                                        }
                                        209928180524878488 => {
                                            outOfMemory();
                                        }
                                        13655413948940366317 => {
                                            compressedStreamEOF();
                                        }
                                        _ => {
                                            if zStream != stdin {
                                                fclose(zStream);
                                            }
                                            if stream != stdout {
                                                fclose(stream);
                                            }
                                            if streamNo == 1 as libc::c_int {
                                                return 0 as Bool;
                                            } else {
                                                if noisy != 0 {
                                                    fprintf(
                                                            stderr,
                                                            b"\n%s: %s: trailing garbage after EOF ignored\n\0"
                                                                as *const u8 as *const libc::c_char,
                                                            progName,
                                                            inName.as_mut_ptr(),
                                                        );
                                                }
                                                return 1 as Bool;
                                            }
                                        }
                                    }
                                }
                                -6 => {}
                                -4 => {
                                    current_block = 14054180689323133469;
                                    match current_block {
                                        8365064614624041636 => {
                                            panic(
                                                b"decompress:unexpected error\0" as *const u8
                                                    as *const libc::c_char,
                                            );
                                        }
                                        13095676725198100986 => {
                                            configError();
                                        }
                                        14054180689323133469 => {
                                            crcError();
                                        }
                                        209928180524878488 => {
                                            outOfMemory();
                                        }
                                        13655413948940366317 => {
                                            compressedStreamEOF();
                                        }
                                        _ => {
                                            if zStream != stdin {
                                                fclose(zStream);
                                            }
                                            if stream != stdout {
                                                fclose(stream);
                                            }
                                            if streamNo == 1 as libc::c_int {
                                                return 0 as Bool;
                                            } else {
                                                if noisy != 0 {
                                                    fprintf(
                                                            stderr,
                                                            b"\n%s: %s: trailing garbage after EOF ignored\n\0"
                                                                as *const u8 as *const libc::c_char,
                                                            progName,
                                                            inName.as_mut_ptr(),
                                                        );
                                                }
                                                return 1 as Bool;
                                            }
                                        }
                                    }
                                }
                                -3 => {
                                    current_block = 209928180524878488;
                                    match current_block {
                                        8365064614624041636 => {
                                            panic(
                                                b"decompress:unexpected error\0" as *const u8
                                                    as *const libc::c_char,
                                            );
                                        }
                                        13095676725198100986 => {
                                            configError();
                                        }
                                        14054180689323133469 => {
                                            crcError();
                                        }
                                        209928180524878488 => {
                                            outOfMemory();
                                        }
                                        13655413948940366317 => {
                                            compressedStreamEOF();
                                        }
                                        _ => {
                                            if zStream != stdin {
                                                fclose(zStream);
                                            }
                                            if stream != stdout {
                                                fclose(stream);
                                            }
                                            if streamNo == 1 as libc::c_int {
                                                return 0 as Bool;
                                            } else {
                                                if noisy != 0 {
                                                    fprintf(
                                                            stderr,
                                                            b"\n%s: %s: trailing garbage after EOF ignored\n\0"
                                                                as *const u8 as *const libc::c_char,
                                                            progName,
                                                            inName.as_mut_ptr(),
                                                        );
                                                }
                                                return 1 as Bool;
                                            }
                                        }
                                    }
                                }
                                -7 => {
                                    current_block = 13655413948940366317;
                                    match current_block {
                                        8365064614624041636 => {
                                            panic(
                                                b"decompress:unexpected error\0" as *const u8
                                                    as *const libc::c_char,
                                            );
                                        }
                                        13095676725198100986 => {
                                            configError();
                                        }
                                        14054180689323133469 => {
                                            crcError();
                                        }
                                        209928180524878488 => {
                                            outOfMemory();
                                        }
                                        13655413948940366317 => {
                                            compressedStreamEOF();
                                        }
                                        _ => {
                                            if zStream != stdin {
                                                fclose(zStream);
                                            }
                                            if stream != stdout {
                                                fclose(stream);
                                            }
                                            if streamNo == 1 as libc::c_int {
                                                return 0 as Bool;
                                            } else {
                                                if noisy != 0 {
                                                    fprintf(
                                                            stderr,
                                                            b"\n%s: %s: trailing garbage after EOF ignored\n\0"
                                                                as *const u8 as *const libc::c_char,
                                                            progName,
                                                            inName.as_mut_ptr(),
                                                        );
                                                }
                                                return 1 as Bool;
                                            }
                                        }
                                    }
                                }
                                -5 => {
                                    current_block = 17767854911385080736;
                                    match current_block {
                                        8365064614624041636 => {
                                            panic(
                                                b"decompress:unexpected error\0" as *const u8
                                                    as *const libc::c_char,
                                            );
                                        }
                                        13095676725198100986 => {
                                            configError();
                                        }
                                        14054180689323133469 => {
                                            crcError();
                                        }
                                        209928180524878488 => {
                                            outOfMemory();
                                        }
                                        13655413948940366317 => {
                                            compressedStreamEOF();
                                        }
                                        _ => {
                                            if zStream != stdin {
                                                fclose(zStream);
                                            }
                                            if stream != stdout {
                                                fclose(stream);
                                            }
                                            if streamNo == 1 as libc::c_int {
                                                return 0 as Bool;
                                            } else {
                                                if noisy != 0 {
                                                    fprintf(
                                                            stderr,
                                                            b"\n%s: %s: trailing garbage after EOF ignored\n\0"
                                                                as *const u8 as *const libc::c_char,
                                                            progName,
                                                            inName.as_mut_ptr(),
                                                        );
                                                }
                                                return 1 as Bool;
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    current_block = 8365064614624041636;
                                    match current_block {
                                        8365064614624041636 => {
                                            panic(
                                                b"decompress:unexpected error\0" as *const u8
                                                    as *const libc::c_char,
                                            );
                                        }
                                        13095676725198100986 => {
                                            configError();
                                        }
                                        14054180689323133469 => {
                                            crcError();
                                        }
                                        209928180524878488 => {
                                            outOfMemory();
                                        }
                                        13655413948940366317 => {
                                            compressedStreamEOF();
                                        }
                                        _ => {
                                            if zStream != stdin {
                                                fclose(zStream);
                                            }
                                            if stream != stdout {
                                                fclose(stream);
                                            }
                                            if streamNo == 1 as libc::c_int {
                                                return 0 as Bool;
                                            } else {
                                                if noisy != 0 {
                                                    fprintf(
                                                            stderr,
                                                            b"\n%s: %s: trailing garbage after EOF ignored\n\0"
                                                                as *const u8 as *const libc::c_char,
                                                            progName,
                                                            inName.as_mut_ptr(),
                                                        );
                                                }
                                                return 1 as Bool;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            if ferror(zStream) == 0 {
                                if stream != stdout {
                                    let fd: i32 = fileno(stream);
                                    if fd < 0 as libc::c_int {
                                        current_block = 6432526541220421294;
                                    } else {
                                        applySavedFileAttrToOutputFile(fd);
                                        current_block = 11459959175219260272;
                                    }
                                } else {
                                    current_block = 11459959175219260272;
                                }
                                match current_block {
                                    6432526541220421294 => {}
                                    _ => {
                                        ret = fclose(zStream);
                                        if ret != -1 as libc::c_int && ferror(stream) == 0 {
                                            ret = fflush(stream);
                                            if ret == 0 as libc::c_int {
                                                if stream != stdout {
                                                    ret = fclose(stream);
                                                    outputHandleJustInCase =
                                                        std::ptr::null_mut::<FILE>();
                                                    if ret == -1 as libc::c_int {
                                                        current_block = 6432526541220421294;
                                                    } else {
                                                        current_block = 3123434771885419771;
                                                    }
                                                } else {
                                                    current_block = 3123434771885419771;
                                                }
                                                match current_block {
                                                    6432526541220421294 => {}
                                                    _ => {
                                                        outputHandleJustInCase =
                                                            std::ptr::null_mut::<FILE>();
                                                        if verbosity >= 2 as libc::c_int {
                                                            fprintf(
                                                                stderr,
                                                                b"\n    \0" as *const u8
                                                                    as *const libc::c_char,
                                                            );
                                                        }
                                                        return 1 as Bool;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            }
        }
    }
    ioError();
}
unsafe fn testStream(zStream: *mut FILE) -> Bool {
    let mut current_block: u64;
    let mut bzf: *mut libc::c_void;
    let mut bzerr: i32 = 0;
    let mut bzerr_dummy: i32 = 0;
    let ret: i32;
    let mut i: i32;
    let mut obuf: [u8; 5000] = [0; 5000];
    let mut unused: [u8; 5000] = [0; 5000];
    let mut unusedTmpV: *mut libc::c_void = std::ptr::null_mut::<libc::c_void>();
    let mut nUnused = 0 as libc::c_int;
    let mut streamNo = 0 as libc::c_int;
    if ferror(zStream) == 0 {
        's_29: loop {
            bzf = BZ2_bzReadOpen(
                &mut bzerr,
                zStream,
                verbosity,
                smallMode as libc::c_int,
                unused.as_mut_ptr() as *mut libc::c_void,
                nUnused,
            );
            if bzf.is_null() || bzerr != 0 as libc::c_int {
                current_block = 5115833311404821621;
                break;
            }
            streamNo += 1;
            while bzerr == 0 as libc::c_int {
                BZ2_bzRead(
                    &mut bzerr,
                    bzf,
                    obuf.as_mut_ptr() as *mut libc::c_void,
                    5000 as libc::c_int,
                );
                if bzerr == -5 as libc::c_int {
                    current_block = 5115833311404821621;
                    break 's_29;
                }
            }
            if bzerr != 4 as libc::c_int {
                current_block = 5115833311404821621;
                break;
            }
            BZ2_bzReadGetUnused(&mut bzerr, bzf, &mut unusedTmpV, &mut nUnused);
            if bzerr != 0 as libc::c_int {
                panic(b"test:bzReadGetUnused\0" as *const u8 as *const libc::c_char);
            }
            let unusedTmp = unusedTmpV as *mut u8;
            i = 0 as libc::c_int;
            while i < nUnused {
                unused[i as usize] = *unusedTmp.offset(i as isize);
                i += 1;
            }
            BZ2_bzReadClose(&mut bzerr, bzf);
            if bzerr != 0 as libc::c_int {
                panic(b"test:bzReadGetUnused\0" as *const u8 as *const libc::c_char);
            }
            if nUnused == 0 as libc::c_int && myfeof(zStream) as libc::c_int != 0 {
                current_block = 5783071609795492627;
                break;
            }
        }
        match current_block {
            5783071609795492627 => {
                if ferror(zStream) == 0 {
                    ret = fclose(zStream);
                    if ret != -1 as libc::c_int {
                        if verbosity >= 2 as libc::c_int {
                            fprintf(stderr, b"\n    \0" as *const u8 as *const libc::c_char);
                        }
                        return 1 as Bool;
                    }
                }
            }
            _ => {
                BZ2_bzReadClose(&mut bzerr_dummy, bzf);
                if verbosity == 0 as libc::c_int {
                    fprintf(
                        stderr,
                        b"%s: %s: \0" as *const u8 as *const libc::c_char,
                        progName,
                        inName.as_mut_ptr(),
                    );
                }
                match bzerr {
                    -9 => {
                        current_block = 7033902699813040753;
                        match current_block {
                            14955303639559299169 => {
                                panic(
                                    b"test:unexpected error\0" as *const u8 as *const libc::c_char,
                                );
                            }
                            10567734636544821229 => {
                                fprintf(
                                    stderr,
                                    b"file ends unexpectedly\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                            17188754426950485343 => {
                                if zStream != stdin {
                                    fclose(zStream);
                                }
                                if streamNo == 1 as libc::c_int {
                                    fprintf(
                                        stderr,
                                        b"bad magic number (file not created by bzip2)\n\0"
                                            as *const u8
                                            as *const libc::c_char,
                                    );
                                    return 0 as Bool;
                                } else {
                                    if noisy != 0 {
                                        fprintf(
                                            stderr,
                                            b"trailing garbage after EOF ignored\n\0" as *const u8
                                                as *const libc::c_char,
                                        );
                                    }
                                    return 1 as Bool;
                                }
                            }
                            7033902699813040753 => {
                                configError();
                            }
                            12127014564286193091 => {
                                outOfMemory();
                            }
                            _ => {
                                fprintf(
                                    stderr,
                                    b"data integrity (CRC) error in data\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                        }
                    }
                    -6 => {}
                    -4 => {
                        current_block = 4900559648241656877;
                        match current_block {
                            14955303639559299169 => {
                                panic(
                                    b"test:unexpected error\0" as *const u8 as *const libc::c_char,
                                );
                            }
                            10567734636544821229 => {
                                fprintf(
                                    stderr,
                                    b"file ends unexpectedly\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                            17188754426950485343 => {
                                if zStream != stdin {
                                    fclose(zStream);
                                }
                                if streamNo == 1 as libc::c_int {
                                    fprintf(
                                        stderr,
                                        b"bad magic number (file not created by bzip2)\n\0"
                                            as *const u8
                                            as *const libc::c_char,
                                    );
                                    return 0 as Bool;
                                } else {
                                    if noisy != 0 {
                                        fprintf(
                                            stderr,
                                            b"trailing garbage after EOF ignored\n\0" as *const u8
                                                as *const libc::c_char,
                                        );
                                    }
                                    return 1 as Bool;
                                }
                            }
                            7033902699813040753 => {
                                configError();
                            }
                            12127014564286193091 => {
                                outOfMemory();
                            }
                            _ => {
                                fprintf(
                                    stderr,
                                    b"data integrity (CRC) error in data\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                        }
                    }
                    -3 => {
                        current_block = 12127014564286193091;
                        match current_block {
                            14955303639559299169 => {
                                panic(
                                    b"test:unexpected error\0" as *const u8 as *const libc::c_char,
                                );
                            }
                            10567734636544821229 => {
                                fprintf(
                                    stderr,
                                    b"file ends unexpectedly\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                            17188754426950485343 => {
                                if zStream != stdin {
                                    fclose(zStream);
                                }
                                if streamNo == 1 as libc::c_int {
                                    fprintf(
                                        stderr,
                                        b"bad magic number (file not created by bzip2)\n\0"
                                            as *const u8
                                            as *const libc::c_char,
                                    );
                                    return 0 as Bool;
                                } else {
                                    if noisy != 0 {
                                        fprintf(
                                            stderr,
                                            b"trailing garbage after EOF ignored\n\0" as *const u8
                                                as *const libc::c_char,
                                        );
                                    }
                                    return 1 as Bool;
                                }
                            }
                            7033902699813040753 => {
                                configError();
                            }
                            12127014564286193091 => {
                                outOfMemory();
                            }
                            _ => {
                                fprintf(
                                    stderr,
                                    b"data integrity (CRC) error in data\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                        }
                    }
                    -7 => {
                        current_block = 10567734636544821229;
                        match current_block {
                            14955303639559299169 => {
                                panic(
                                    b"test:unexpected error\0" as *const u8 as *const libc::c_char,
                                );
                            }
                            10567734636544821229 => {
                                fprintf(
                                    stderr,
                                    b"file ends unexpectedly\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                            17188754426950485343 => {
                                if zStream != stdin {
                                    fclose(zStream);
                                }
                                if streamNo == 1 as libc::c_int {
                                    fprintf(
                                        stderr,
                                        b"bad magic number (file not created by bzip2)\n\0"
                                            as *const u8
                                            as *const libc::c_char,
                                    );
                                    return 0 as Bool;
                                } else {
                                    if noisy != 0 {
                                        fprintf(
                                            stderr,
                                            b"trailing garbage after EOF ignored\n\0" as *const u8
                                                as *const libc::c_char,
                                        );
                                    }
                                    return 1 as Bool;
                                }
                            }
                            7033902699813040753 => {
                                configError();
                            }
                            12127014564286193091 => {
                                outOfMemory();
                            }
                            _ => {
                                fprintf(
                                    stderr,
                                    b"data integrity (CRC) error in data\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                        }
                    }
                    -5 => {
                        current_block = 17188754426950485343;
                        match current_block {
                            14955303639559299169 => {
                                panic(
                                    b"test:unexpected error\0" as *const u8 as *const libc::c_char,
                                );
                            }
                            10567734636544821229 => {
                                fprintf(
                                    stderr,
                                    b"file ends unexpectedly\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                            17188754426950485343 => {
                                if zStream != stdin {
                                    fclose(zStream);
                                }
                                if streamNo == 1 as libc::c_int {
                                    fprintf(
                                        stderr,
                                        b"bad magic number (file not created by bzip2)\n\0"
                                            as *const u8
                                            as *const libc::c_char,
                                    );
                                    return 0 as Bool;
                                } else {
                                    if noisy != 0 {
                                        fprintf(
                                            stderr,
                                            b"trailing garbage after EOF ignored\n\0" as *const u8
                                                as *const libc::c_char,
                                        );
                                    }
                                    return 1 as Bool;
                                }
                            }
                            7033902699813040753 => {
                                configError();
                            }
                            12127014564286193091 => {
                                outOfMemory();
                            }
                            _ => {
                                fprintf(
                                    stderr,
                                    b"data integrity (CRC) error in data\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                        }
                    }
                    _ => {
                        current_block = 14955303639559299169;
                        match current_block {
                            14955303639559299169 => {
                                panic(
                                    b"test:unexpected error\0" as *const u8 as *const libc::c_char,
                                );
                            }
                            10567734636544821229 => {
                                fprintf(
                                    stderr,
                                    b"file ends unexpectedly\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                            17188754426950485343 => {
                                if zStream != stdin {
                                    fclose(zStream);
                                }
                                if streamNo == 1 as libc::c_int {
                                    fprintf(
                                        stderr,
                                        b"bad magic number (file not created by bzip2)\n\0"
                                            as *const u8
                                            as *const libc::c_char,
                                    );
                                    return 0 as Bool;
                                } else {
                                    if noisy != 0 {
                                        fprintf(
                                            stderr,
                                            b"trailing garbage after EOF ignored\n\0" as *const u8
                                                as *const libc::c_char,
                                        );
                                    }
                                    return 1 as Bool;
                                }
                            }
                            7033902699813040753 => {
                                configError();
                            }
                            12127014564286193091 => {
                                outOfMemory();
                            }
                            _ => {
                                fprintf(
                                    stderr,
                                    b"data integrity (CRC) error in data\n\0" as *const u8
                                        as *const libc::c_char,
                                );
                                return 0 as Bool;
                            }
                        }
                    }
                }
            }
        }
    }
    ioError();
}

unsafe fn setExit(v: i32) {
    if v > exitValue {
        exitValue = v;
    }
}

unsafe fn cadvise() {
    if noisy != 0 {
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
    if noisy != 0 {
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
    if srcMode == 3 as libc::c_int
        && opMode != 3 as libc::c_int
        && deleteOutputOnInterrupt as libc::c_int != 0
    {
        if stat(inName.as_mut_ptr(), &mut statBuf) == 0 {
            if noisy != 0 {
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
    fprintf(
        stderr,
        b"\n%s: PANIC -- internal consistency error:\n\t%s\n\tThis is a BUG.  Please report it at:\n\thttps://gitlab.com/bzip2/bzip2/-/issues\n\0"
            as *const u8 as *const libc::c_char,
        progName,
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
    if noisy != 0 {
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
    fprintf(
        stderr,
        b"\n%s: I/O or other error, bailing out.  Possible reason follows.\n\0" as *const u8
            as *const libc::c_char,
        progName,
    );
    perror(progName);
    showFileNames();
    cleanUpAndFail(1 as libc::c_int);
}
unsafe extern "C" fn mySignalCatcher(_: IntNative) {
    fprintf(
        stderr,
        b"\n%s: Control-C or similar caught, quitting.\n\0" as *const u8 as *const libc::c_char,
        progName,
    );
    cleanUpAndFail(1 as libc::c_int);
}
unsafe fn mySIGSEGVorSIGBUScatcher(_: IntNative) {
    let mut msg: *const libc::c_char;
    if opMode == 1 {
        msg = b": Caught a SIGSEGV or SIGBUS whilst compressing.\n\n   Possible causes are (most likely first):\n   (1) This computer has unreliable memory or cache hardware\n       (a surprisingly common problem; try a different machine.)\n   (2) A bug in the compiler used to create this executable\n       (unlikely, if you didn't compile bzip2 yourself.)\n   (3) A real bug in bzip2 -- I hope this should never be the case.\n   The user's manual, Section 4.3, has more info on (1) and (2).\n   \n   If you suspect this is a bug in bzip2, or are unsure about (1)\n   or (2), report it at: https://gitlab.com/bzip2/bzip2/-/issues\n   Section 4.3 of the user's manual describes the info a useful\n   bug report should have.  If the manual is available on your\n   system, please try and read it before mailing me.  If you don't\n   have the manual or can't be bothered to read it, mail me anyway.\n\n\0"
            as *const u8 as *const libc::c_char;
    } else {
        msg = b": Caught a SIGSEGV or SIGBUS whilst decompressing.\n\n   Possible causes are (most likely first):\n   (1) The compressed data is corrupted, and bzip2's usual checks\n       failed to detect this.  Try bzip2 -tvv my_file.bz2.\n   (2) This computer has unreliable memory or cache hardware\n       (a surprisingly common problem; try a different machine.)\n   (3) A bug in the compiler used to create this executable\n       (unlikely, if you didn't compile bzip2 yourself.)\n   (4) A real bug in bzip2 -- I hope this should never be the case.\n   The user's manual, Section 4.3, has more info on (2) and (3).\n   \n   If you suspect this is a bug in bzip2, or are unsure about (2)\n   or (3), report it at: https://gitlab.com/bzip2/bzip2/-/issues\n   Section 4.3 of the user's manual describes the info a useful\n   bug report should have.  If the manual is available on your\n   system, please try and read it before mailing me.  If you don't\n   have the manual or can't be bothered to read it, mail me anyway.\n\n\0"
            as *const u8 as *const libc::c_char;
    }
    write(2, b"\n" as *const u8 as *const libc::c_void, 1);
    write(2, progName as *const libc::c_void, strlen(progName) as _);
    write(2, msg as *const libc::c_void, strlen(msg) as _);
    msg = b"\tInput file = \0" as *const u8 as *const libc::c_char;
    write(
        2 as libc::c_int,
        msg as *const libc::c_void,
        strlen(msg) as _,
    );
    write(
        2,
        inName.as_mut_ptr() as *const libc::c_void,
        strlen(inName.as_mut_ptr()) as _,
    );
    write(
        2,
        b"\n\0" as *const u8 as *const libc::c_char as *const libc::c_void,
        1,
    );
    msg = b"\tOutput file = \0" as *const u8 as *const libc::c_char;
    write(2, msg as *const libc::c_void, strlen(msg) as _);
    write(
        2,
        outName.as_mut_ptr() as *const libc::c_void,
        strlen(outName.as_mut_ptr()) as _,
    );
    write(2, b"\n" as *const u8 as *const libc::c_void, 1);
    if opMode == 1 {
        setExit(3);
    } else {
        setExit(2);
    }
    _exit(exitValue);
}
unsafe fn outOfMemory() -> ! {
    fprintf(
        stderr,
        b"\n%s: couldn't allocate enough memory\n\0" as *const u8 as *const libc::c_char,
        progName,
    );
    showFileNames();
    cleanUpAndFail(1 as libc::c_int);
}
unsafe fn configError() -> ! {
    fprintf(
        stderr,
        b"bzip2: I'm not configured correctly for this platform!\n\tI require Int32, Int16 and Char to have sizes\n\tof 4, 2 and 1 bytes to run properly, and they don't.\n\tProbably you can fix this by defining them correctly,\n\tand recompiling.  Bye!\n\0"
            as *const u8 as *const libc::c_char,
    );
    setExit(3 as libc::c_int);
    exit(exitValue);
}
unsafe fn pad(s: *mut c_char) {
    if strlen(s) as i32 >= longestFileName {
        return;
    }
    let mut i = 1 as libc::c_int;
    while i <= longestFileName - strlen(s) as i32 {
        fprintf(stderr, b" \0" as *const u8 as *const libc::c_char);
        i += 1;
    }
}
unsafe fn copyFileName(to: *mut c_char, from: *const c_char) {
    if strlen(from) > (1034 as libc::c_int - 10 as libc::c_int) as libc::size_t {
        fprintf(
            stderr,
            b"bzip2: file name\n`%s'\nis suspiciously (more than %d chars) long.\nTry using a reasonable file name instead.  Sorry! :-)\n\0"
                as *const u8 as *const libc::c_char,
            from,
            1034 as libc::c_int - 10 as libc::c_int,
        );
        setExit(1 as libc::c_int);
        exit(exitValue);
    }
    strncpy(
        to,
        from,
        (1034 as libc::c_int - 10 as libc::c_int) as libc::size_t,
    );
    *to.offset((1034 as libc::c_int - 10 as libc::c_int) as isize) = '\0' as i32 as c_char;
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
unsafe fn containsDubiousChars(_: *mut c_char) -> Bool {
    0
}
static mut zSuffix: [*const c_char; 4] = [
    b".bz2\0" as *const u8 as *const libc::c_char,
    b".bz\0" as *const u8 as *const libc::c_char,
    b".tbz2\0" as *const u8 as *const libc::c_char,
    b".tbz\0" as *const u8 as *const libc::c_char,
];
static mut unzSuffix: [*const c_char; 4] = [
    b"\0" as *const u8 as *const libc::c_char,
    b"\0" as *const u8 as *const libc::c_char,
    b".tar\0" as *const u8 as *const libc::c_char,
    b".tar\0" as *const u8 as *const libc::c_char,
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
unsafe fn mapSuffix(name: *mut c_char, oldSuffix: *const c_char, newSuffix: *const c_char) -> Bool {
    if hasSuffix(name, oldSuffix) == 0 {
        return 0 as Bool;
    }
    *name.add((strlen(name)).wrapping_sub(strlen(oldSuffix))) = 0 as libc::c_int as c_char;
    strcat(name, newSuffix);
    1 as Bool
}
unsafe fn compress(name: *mut c_char) {
    let inStr: *mut FILE;
    let outStr: *mut FILE;
    let mut n: i32 = 0;
    let mut statBuf: stat = zeroed();
    deleteOutputOnInterrupt = 0 as Bool;
    if name.is_null() && srcMode != 1 as libc::c_int {
        panic(b"compress: bad modes\n\0" as *const u8 as *const libc::c_char);
    }
    match srcMode {
        1 => {
            copyFileName(
                inName.as_mut_ptr(),
                b"(stdin)\0" as *const u8 as *const libc::c_char,
            );
            copyFileName(
                outName.as_mut_ptr(),
                b"(stdout)\0" as *const u8 as *const libc::c_char,
            );
        }
        3 => {
            copyFileName(inName.as_mut_ptr(), name);
            copyFileName(outName.as_mut_ptr(), name);
            strcat(
                outName.as_mut_ptr(),
                b".bz2\0" as *const u8 as *const libc::c_char,
            );
        }
        2 => {
            copyFileName(inName.as_mut_ptr(), name);
            copyFileName(
                outName.as_mut_ptr(),
                b"(stdout)\0" as *const u8 as *const libc::c_char,
            );
        }
        _ => {}
    }
    if srcMode != 1 as libc::c_int && containsDubiousChars(inName.as_mut_ptr()) as libc::c_int != 0
    {
        if noisy != 0 {
            fprintf(
                stderr,
                b"%s: There are no files matching `%s'.\n\0" as *const u8 as *const libc::c_char,
                progName,
                inName.as_mut_ptr(),
            );
        }
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode != 1 as libc::c_int && fileExists(inName.as_mut_ptr()) == 0 {
        eprintln!(
            "{}: Can't open input file {}: {}.",
            std::env::args().next().unwrap(),
            CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
            std::io::Error::last_os_error(),
        );
        setExit(1 as libc::c_int);
        return;
    }
    let mut i = 0 as libc::c_int;
    while i < 4 as libc::c_int {
        if hasSuffix(inName.as_mut_ptr(), zSuffix[i as usize]) != 0 {
            if noisy != 0 {
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
    if srcMode == 3 as libc::c_int || srcMode == 2 as libc::c_int {
        stat(inName.as_mut_ptr(), &mut statBuf);
        if statBuf.st_mode & 0o170000 == 0o40000 {
            fprintf(
                stderr,
                b"%s: Input file %s is a directory.\n\0" as *const u8 as *const libc::c_char,
                progName,
                inName.as_mut_ptr(),
            );
            setExit(1 as libc::c_int);
            return;
        }
    }
    if srcMode == 3 as libc::c_int
        && forceOverwrite == 0
        && notAStandardFile(inName.as_mut_ptr()) as libc::c_int != 0
    {
        if noisy != 0 {
            fprintf(
                stderr,
                b"%s: Input file %s is not a normal file.\n\0" as *const u8 as *const libc::c_char,
                progName,
                inName.as_mut_ptr(),
            );
        }
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode == 3 as libc::c_int && fileExists(outName.as_mut_ptr()) as libc::c_int != 0 {
        if forceOverwrite != 0 {
            remove(outName.as_mut_ptr());
        } else {
            fprintf(
                stderr,
                b"%s: Output file %s already exists.\n\0" as *const u8 as *const libc::c_char,
                progName,
                outName.as_mut_ptr(),
            );
            setExit(1 as libc::c_int);
            return;
        }
    }
    if srcMode == 3 as libc::c_int && forceOverwrite == 0 && {
        n = countHardLinks(inName.as_mut_ptr());
        n > 0 as libc::c_int
    } {
        fprintf(
            stderr,
            b"%s: Input file %s has %d other link%s.\n\0" as *const u8 as *const libc::c_char,
            progName,
            inName.as_mut_ptr(),
            n,
            if n > 1 as libc::c_int {
                b"s\0" as *const u8 as *const libc::c_char
            } else {
                b"\0" as *const u8 as *const libc::c_char
            },
        );
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode == 3 as libc::c_int {
        saveInputFileMetaInfo(inName.as_mut_ptr());
    }
    match srcMode {
        1 => {
            inStr = stdin;
            outStr = stdout;
            if isatty(fileno(stdout)) != 0 {
                fprintf(
                    stderr,
                    b"%s: I won't write compressed data to a terminal.\n\0" as *const u8
                        as *const libc::c_char,
                    progName,
                );
                fprintf(
                    stderr,
                    b"%s: For help, type: `%s --help'.\n\0" as *const u8 as *const libc::c_char,
                    progName,
                    progName,
                );
                setExit(1 as libc::c_int);
                return;
            }
        }
        2 => {
            inStr = fopen(
                inName.as_mut_ptr(),
                b"rb\0" as *const u8 as *const libc::c_char,
            );
            outStr = stdout;
            if isatty(fileno(stdout)) != 0 {
                fprintf(
                    stderr,
                    b"%s: I won't write compressed data to a terminal.\n\0" as *const u8
                        as *const libc::c_char,
                    progName,
                );
                fprintf(
                    stderr,
                    b"%s: For help, type: `%s --help'.\n\0" as *const u8 as *const libc::c_char,
                    progName,
                    progName,
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
                    std::io::Error::last_os_error(),
                );
                setExit(1 as libc::c_int);
                return;
            }
        }
        3 => {
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
                    std::io::Error::last_os_error(),
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
                    std::io::Error::last_os_error(),
                );
                if !outStr.is_null() {
                    fclose(outStr);
                }
                setExit(1 as libc::c_int);
                return;
            }
        }
        _ => {
            panic(b"compress: bad srcMode\0" as *const u8 as *const libc::c_char);
        }
    }
    if verbosity >= 1 as libc::c_int {
        fprintf(
            stderr,
            b"  %s: \0" as *const u8 as *const libc::c_char,
            inName.as_mut_ptr(),
        );
        pad(inName.as_mut_ptr());
        fflush(stderr);
    }
    outputHandleJustInCase = outStr;
    deleteOutputOnInterrupt = 1 as Bool;
    compressStream(inStr, outStr);
    outputHandleJustInCase = std::ptr::null_mut::<FILE>();
    if srcMode == 3 as libc::c_int {
        applySavedTimeInfoToOutputFile(outName.as_mut_ptr());
        deleteOutputOnInterrupt = 0 as Bool;
        if keepInputFiles == 0 {
            let retVal: IntNative = remove(inName.as_mut_ptr());
            if retVal != 0 as libc::c_int {
                ioError();
            }
        }
    }
    deleteOutputOnInterrupt = 0 as Bool;
}
unsafe fn uncompress(name: *mut c_char) {
    let current_block: u64;
    let inStr: *mut FILE;
    let outStr: *mut FILE;
    let n: i32;
    deleteOutputOnInterrupt = 0 as Bool;
    if name.is_null() && srcMode != 1 as libc::c_int {
        panic(b"uncompress: bad modes\n\0" as *const u8 as *const libc::c_char);
    }
    let mut cantGuess = 0 as Bool;
    match srcMode {
        1 => {
            copyFileName(
                inName.as_mut_ptr(),
                b"(stdin)\0" as *const u8 as *const libc::c_char,
            );
            copyFileName(
                outName.as_mut_ptr(),
                b"(stdout)\0" as *const u8 as *const libc::c_char,
            );
        }
        3 => {
            copyFileName(inName.as_mut_ptr(), name);
            copyFileName(outName.as_mut_ptr(), name);
            let mut i = 0 as libc::c_int;
            loop {
                if i >= 4 as libc::c_int {
                    current_block = 7651349459974463963;
                    break;
                }
                if mapSuffix(
                    outName.as_mut_ptr(),
                    zSuffix[i as usize],
                    unzSuffix[i as usize],
                ) != 0
                {
                    current_block = 4003995367480147712;
                    break;
                }
                i += 1;
            }
            match current_block {
                4003995367480147712 => {}
                _ => {
                    cantGuess = 1 as Bool;
                    strcat(
                        outName.as_mut_ptr(),
                        b".out\0" as *const u8 as *const libc::c_char,
                    );
                }
            }
        }
        2 => {
            copyFileName(inName.as_mut_ptr(), name);
            copyFileName(
                outName.as_mut_ptr(),
                b"(stdout)\0" as *const u8 as *const libc::c_char,
            );
        }
        _ => {}
    }
    if srcMode != 1 as libc::c_int && containsDubiousChars(inName.as_mut_ptr()) as libc::c_int != 0
    {
        if noisy != 0 {
            fprintf(
                stderr,
                b"%s: There are no files matching `%s'.\n\0" as *const u8 as *const libc::c_char,
                progName,
                inName.as_mut_ptr(),
            );
        }
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode != 1 as libc::c_int && fileExists(inName.as_mut_ptr()) == 0 {
        eprintln!(
            "{}: Can't open input file {}: {}.",
            std::env::args().next().unwrap(),
            CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
            std::io::Error::last_os_error(),
        );
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode == 3 as libc::c_int || srcMode == 2 as libc::c_int {
        let mut statBuf: stat = zeroed();
        stat(inName.as_mut_ptr(), &mut statBuf);
        if statBuf.st_mode & 0o170000 == 0o40000 {
            fprintf(
                stderr,
                b"%s: Input file %s is a directory.\n\0" as *const u8 as *const libc::c_char,
                progName,
                inName.as_mut_ptr(),
            );
            setExit(1 as libc::c_int);
            return;
        }
    }
    if srcMode == 3 as libc::c_int
        && forceOverwrite == 0
        && notAStandardFile(inName.as_mut_ptr()) as libc::c_int != 0
    {
        if noisy != 0 {
            fprintf(
                stderr,
                b"%s: Input file %s is not a normal file.\n\0" as *const u8 as *const libc::c_char,
                progName,
                inName.as_mut_ptr(),
            );
        }
        setExit(1 as libc::c_int);
        return;
    }
    if cantGuess != 0 && noisy != 0 {
        fprintf(
            stderr,
            b"%s: Can't guess original name for %s -- using %s\n\0" as *const u8
                as *const libc::c_char,
            progName,
            inName.as_mut_ptr(),
            outName.as_mut_ptr(),
        );
    }
    if srcMode == 3 as libc::c_int && fileExists(outName.as_mut_ptr()) as libc::c_int != 0 {
        if forceOverwrite != 0 {
            remove(outName.as_mut_ptr());
        } else {
            fprintf(
                stderr,
                b"%s: Output file %s already exists.\n\0" as *const u8 as *const libc::c_char,
                progName,
                outName.as_mut_ptr(),
            );
            setExit(1 as libc::c_int);
            return;
        }
    }
    if srcMode == 3 as libc::c_int && forceOverwrite == 0 && {
        n = countHardLinks(inName.as_mut_ptr());
        n > 0 as libc::c_int
    } {
        fprintf(
            stderr,
            b"%s: Input file %s has %d other link%s.\n\0" as *const u8 as *const libc::c_char,
            progName,
            inName.as_mut_ptr(),
            n,
            if n > 1 as libc::c_int {
                b"s\0" as *const u8 as *const libc::c_char
            } else {
                b"\0" as *const u8 as *const libc::c_char
            },
        );
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode == 3 as libc::c_int {
        saveInputFileMetaInfo(inName.as_mut_ptr());
    }
    match srcMode {
        1 => {
            inStr = stdin;
            outStr = stdout;
            if isatty(fileno(stdin)) != 0 {
                fprintf(
                    stderr,
                    b"%s: I won't read compressed data from a terminal.\n\0" as *const u8
                        as *const libc::c_char,
                    progName,
                );
                fprintf(
                    stderr,
                    b"%s: For help, type: `%s --help'.\n\0" as *const u8 as *const libc::c_char,
                    progName,
                    progName,
                );
                setExit(1 as libc::c_int);
                return;
            }
        }
        2 => {
            inStr = fopen(
                inName.as_mut_ptr(),
                b"rb\0" as *const u8 as *const libc::c_char,
            );
            outStr = stdout;
            if inStr.is_null() {
                eprintln!(
                    "{}: Can't open input file {}:{}.",
                    std::env::args().next().unwrap(),
                    CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
                    std::io::Error::last_os_error(),
                );
                if !inStr.is_null() {
                    fclose(inStr);
                }
                setExit(1 as libc::c_int);
                return;
            }
        }
        3 => {
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
                    std::io::Error::last_os_error(),
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
                    std::io::Error::last_os_error(),
                );
                if !outStr.is_null() {
                    fclose(outStr);
                }
                setExit(1 as libc::c_int);
                return;
            }
        }
        _ => {
            panic(b"uncompress: bad srcMode\0" as *const u8 as *const libc::c_char);
        }
    }
    if verbosity >= 1 as libc::c_int {
        fprintf(
            stderr,
            b"  %s: \0" as *const u8 as *const libc::c_char,
            inName.as_mut_ptr(),
        );
        pad(inName.as_mut_ptr());
        fflush(stderr);
    }
    outputHandleJustInCase = outStr;
    deleteOutputOnInterrupt = 1 as Bool;
    let magicNumberOK = uncompressStream(inStr, outStr);
    outputHandleJustInCase = std::ptr::null_mut::<FILE>();
    if magicNumberOK != 0 {
        if srcMode == 3 as libc::c_int {
            applySavedTimeInfoToOutputFile(outName.as_mut_ptr());
            deleteOutputOnInterrupt = 0 as Bool;
            if keepInputFiles == 0 {
                let retVal: IntNative = remove(inName.as_mut_ptr());
                if retVal != 0 as libc::c_int {
                    ioError();
                }
            }
        }
    } else {
        unzFailsExist = 1 as Bool;
        deleteOutputOnInterrupt = 0 as Bool;
        if srcMode == 3 as libc::c_int {
            let retVal_0: IntNative = remove(outName.as_mut_ptr());
            if retVal_0 != 0 as libc::c_int {
                ioError();
            }
        }
    }
    deleteOutputOnInterrupt = 0 as Bool;
    if magicNumberOK != 0 {
        if verbosity >= 1 as libc::c_int {
            fprintf(stderr, b"done\n\0" as *const u8 as *const libc::c_char);
        }
    } else {
        setExit(2 as libc::c_int);
        if verbosity >= 1 as libc::c_int {
            fprintf(
                stderr,
                b"not a bzip2 file.\n\0" as *const u8 as *const libc::c_char,
            );
        } else {
            fprintf(
                stderr,
                b"%s: %s is not a bzip2 file.\n\0" as *const u8 as *const libc::c_char,
                progName,
                inName.as_mut_ptr(),
            );
        }
    };
}
unsafe fn testf(name: *mut c_char) {
    let inStr: *mut FILE;
    deleteOutputOnInterrupt = 0 as Bool;
    if name.is_null() && srcMode != 1 as libc::c_int {
        panic(b"testf: bad modes\n\0" as *const u8 as *const libc::c_char);
    }
    copyFileName(
        outName.as_mut_ptr(),
        b"(none)\0" as *const u8 as *const libc::c_char,
    );
    match srcMode {
        1 => {
            copyFileName(
                inName.as_mut_ptr(),
                b"(stdin)\0" as *const u8 as *const libc::c_char,
            );
        }
        3 => {
            copyFileName(inName.as_mut_ptr(), name);
        }
        2 => {
            copyFileName(inName.as_mut_ptr(), name);
        }
        _ => {}
    }
    if srcMode != 1 as libc::c_int && containsDubiousChars(inName.as_mut_ptr()) as libc::c_int != 0
    {
        if noisy != 0 {
            fprintf(
                stderr,
                b"%s: There are no files matching `%s'.\n\0" as *const u8 as *const libc::c_char,
                progName,
                inName.as_mut_ptr(),
            );
        }
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode != 1 as libc::c_int && fileExists(inName.as_mut_ptr()) == 0 {
        eprintln!(
            "{}: Can't open input {}: {}.",
            std::env::args().next().unwrap(),
            CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
            std::io::Error::last_os_error(),
        );
        setExit(1 as libc::c_int);
        return;
    }
    if srcMode != 1 as libc::c_int {
        let mut statBuf: stat = zeroed();
        stat(inName.as_mut_ptr(), &mut statBuf);
        if statBuf.st_mode & 0o170000 == 0o40000 {
            fprintf(
                stderr,
                b"%s: Input file %s is a directory.\n\0" as *const u8 as *const libc::c_char,
                progName,
                inName.as_mut_ptr(),
            );
            setExit(1 as libc::c_int);
            return;
        }
    }
    match srcMode {
        1 => {
            if isatty(fileno(stdin)) != 0 {
                fprintf(
                    stderr,
                    b"%s: I won't read compressed data from a terminal.\n\0" as *const u8
                        as *const libc::c_char,
                    progName,
                );
                fprintf(
                    stderr,
                    b"%s: For help, type: `%s --help'.\n\0" as *const u8 as *const libc::c_char,
                    progName,
                    progName,
                );
                setExit(1 as libc::c_int);
                return;
            }
            inStr = stdin;
        }
        2 | 3 => {
            inStr = fopen(
                inName.as_mut_ptr(),
                b"rb\0" as *const u8 as *const libc::c_char,
            );
            if inStr.is_null() {
                eprintln!(
                    "{}: Can't open input file {}:{}.",
                    std::env::args().next().unwrap(),
                    CStr::from_ptr(inName.as_ptr()).to_string_lossy(),
                    std::io::Error::last_os_error(),
                );
                setExit(1 as libc::c_int);
                return;
            }
        }
        _ => {
            panic(b"testf: bad srcMode\0" as *const u8 as *const libc::c_char);
        }
    }
    if verbosity >= 1 as libc::c_int {
        fprintf(
            stderr,
            b"  %s: \0" as *const u8 as *const libc::c_char,
            inName.as_mut_ptr(),
        );
        pad(inName.as_mut_ptr());
        fflush(stderr);
    }
    outputHandleJustInCase = std::ptr::null_mut::<FILE>();
    let allOK = testStream(inStr);
    if allOK as libc::c_int != 0 && verbosity >= 1 as libc::c_int {
        fprintf(stderr, b"ok\n\0" as *const u8 as *const libc::c_char);
    }
    if allOK == 0 {
        testFailsExist = 1 as Bool;
    }
}
unsafe fn license() {
    fprintf(
        stdout,
        b"bzip2, a block-sorting file compressor.  Version %s.\n   \n   Copyright (C) 1996-2010 by Julian Seward.\n   \n   This program is free software; you can redistribute it and/or modify\n   it under the terms set out in the LICENSE file, which is included\n   in the bzip2-1.0.6 source distribution.\n   \n   This program is distributed in the hope that it will be useful,\n   but WITHOUT ANY WARRANTY; without even the implied warranty of\n   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the\n   LICENSE file for more details.\n   \n\0"
            as *const u8 as *const libc::c_char,
        BZ2_bzlibVersion(),
    );
}
unsafe fn usage(fullProgName: *mut c_char) {
    fprintf(
        stderr,
        b"bzip2, a block-sorting file compressor.  Version %s.\n\n   usage: %s [flags and input files in any order]\n\n   -h --help           print this message\n   -d --decompress     force decompression\n   -z --compress       force compression\n   -k --keep           keep (don't delete) input files\n   -f --force          overwrite existing output files\n   -t --test           test compressed file integrity\n   -c --stdout         output to standard out\n   -q --quiet          suppress noncritical error messages\n   -v --verbose        be verbose (a 2nd -v gives more)\n   -L --license        display software version & license\n   -V --version        display software version & license\n   -s --small          use less memory (at most 2500k)\n   -1 .. -9            set block size to 100k .. 900k\n   --fast              alias for -1\n   --best              alias for -9\n\n   If invoked as `bzip2', default action is to compress.\n              as `bunzip2',  default action is to decompress.\n              as `bzcat', default action is to decompress to stdout.\n\n   If no file names are given, bzip2 compresses or decompresses\n   from standard input to standard output.  You can combine\n   short flags, so `-v -4' means the same as -v4 or -4v, &c.\n\n\0"
            as *const u8 as *const libc::c_char,
        BZ2_bzlibVersion(),
        fullProgName,
    );
}
unsafe fn redundant(flag: *mut c_char) {
    fprintf(
        stderr,
        b"%s: %s is redundant in versions 0.9.5 and above\n\0" as *const u8 as *const libc::c_char,
        progName,
        flag,
    );
}
unsafe fn myMalloc(n: i32) -> *mut libc::c_void {
    let p: *mut libc::c_void = malloc(n as size_t);
    if p.is_null() {
        outOfMemory();
    }
    p
}
unsafe fn mkCell() -> *mut Cell {
    let c: *mut Cell = myMalloc(core::mem::size_of::<Cell>() as libc::c_ulong as i32) as *mut Cell;
    (*c).name = std::ptr::null_mut();
    (*c).link = std::ptr::null_mut::<zzzz>();
    c
}
unsafe fn snocString(root: *mut Cell, name: *mut c_char) -> *mut Cell {
    if root.is_null() {
        let tmp: *mut Cell = mkCell();
        (*tmp).name = myMalloc((5 as libc::c_int as libc::size_t).wrapping_add(strlen(name)) as i32)
            as *mut c_char;
        strcpy((*tmp).name, name);
        tmp
    } else {
        let mut tmp_0: *mut Cell = root;
        while !((*tmp_0).link).is_null() {
            tmp_0 = (*tmp_0).link;
        }
        (*tmp_0).link = snocString((*tmp_0).link, name);
        root
    }
}
unsafe fn addFlagsFromEnvVar(argList: *mut *mut Cell, varName: *const c_char) {
    let envbase = getenv(varName);
    if !envbase.is_null() {
        let mut p = envbase;
        let mut i = 0 as libc::c_int;
        loop {
            if *p.offset(i as isize) as libc::c_int == 0 as libc::c_int {
                break;
            }
            p = p.offset(i as isize);
            i = 0 as libc::c_int;
            while !(*p.offset(0 as libc::c_int as isize) as u8 as char).is_ascii_whitespace() {
                p = p.offset(1);
            }
            while *p.offset(i as isize) as libc::c_int != 0 as libc::c_int
                && !(*p.offset(i as isize) as u8 as char).is_ascii_whitespace()
            {
                i += 1;
            }
            if i > 0 as libc::c_int {
                let mut k = i;
                if k > 1034 as libc::c_int - 10 as libc::c_int {
                    k = 1034 as libc::c_int - 10 as libc::c_int;
                }
                let mut j = 0 as libc::c_int;
                while j < k {
                    tmpName[j as usize] = *p.offset(j as isize);
                    j += 1;
                }
                tmpName[k as usize] = 0 as libc::c_int as c_char;
                *argList = snocString(*argList, tmpName.as_mut_ptr());
            }
        }
    }
}
unsafe fn main_0(argc: IntNative, argv: *mut *mut c_char) -> IntNative {
    if ::core::mem::size_of::<i32>() as libc::c_ulong != 4 as libc::c_int as libc::c_ulong
        || ::core::mem::size_of::<u32>() as libc::c_ulong != 4 as libc::c_int as libc::c_ulong
        || ::core::mem::size_of::<i16>() as libc::c_ulong != 2 as libc::c_int as libc::c_ulong
        || ::core::mem::size_of::<u16>() as libc::c_ulong != 2 as libc::c_int as libc::c_ulong
        || ::core::mem::size_of::<i8>() as libc::c_ulong != 1 as libc::c_int as libc::c_ulong
        || ::core::mem::size_of::<u8>() as libc::c_ulong != 1 as libc::c_int as libc::c_ulong
    {
        configError();
    }
    outputHandleJustInCase = std::ptr::null_mut::<FILE>();
    smallMode = 0 as Bool;
    keepInputFiles = 0 as Bool;
    forceOverwrite = 0 as Bool;
    noisy = 1 as Bool;
    verbosity = 0 as libc::c_int;
    blockSize100k = 9 as libc::c_int;
    testFailsExist = 0 as Bool;
    unzFailsExist = 0 as Bool;
    numFileNames = 0 as libc::c_int;
    numFilesProcessed = 0 as libc::c_int;
    workFactor = 30 as libc::c_int;
    deleteOutputOnInterrupt = 0 as Bool;
    exitValue = 0 as libc::c_int;
    signal(
        11 as libc::c_int,
        mySIGSEGVorSIGBUScatcher as unsafe fn(IntNative) as usize,
    );
    signal(
        7 as libc::c_int,
        mySIGSEGVorSIGBUScatcher as unsafe fn(IntNative) as usize,
    );
    copyFileName(
        inName.as_mut_ptr(),
        b"(none)\0" as *const u8 as *const libc::c_char,
    );
    copyFileName(
        outName.as_mut_ptr(),
        b"(none)\0" as *const u8 as *const libc::c_char,
    );
    copyFileName(
        progNameReally.as_mut_ptr(),
        *argv.offset(0 as libc::c_int as isize),
    );
    progName = &mut *progNameReally
        .as_mut_ptr()
        .offset(0 as libc::c_int as isize);
    let mut tmp = progNameReally
        .as_mut_ptr()
        .offset(0 as libc::c_int as isize);
    while *tmp as libc::c_int != '\0' as i32 {
        if *tmp as libc::c_int == '/' as i32 {
            progName = tmp.offset(1 as libc::c_int as isize);
        }
        tmp = tmp.offset(1);
    }
    let mut argList = std::ptr::null_mut::<Cell>();
    addFlagsFromEnvVar(&mut argList, b"BZIP2\0" as *const u8 as *const libc::c_char);
    addFlagsFromEnvVar(&mut argList, b"BZIP\0" as *const u8 as *const libc::c_char);
    let mut i = 1 as libc::c_int;
    while i <= argc - 1 as libc::c_int {
        argList = snocString(argList, *argv.offset(i as isize));
        i += 1;
    }
    longestFileName = 7 as libc::c_int;
    numFileNames = 0 as libc::c_int;
    let mut decode = 1 as Bool;
    let mut aa = argList;
    while !aa.is_null() {
        if strcmp((*aa).name, b"--\0" as *const u8 as *const libc::c_char) == 0 as libc::c_int {
            decode = 0 as Bool;
        } else if !(*((*aa).name).offset(0 as libc::c_int as isize) as libc::c_int == '-' as i32
            && decode as libc::c_int != 0)
        {
            numFileNames += 1;
            if longestFileName < strlen((*aa).name) as i32 {
                longestFileName = strlen((*aa).name) as i32;
            }
        }
        aa = (*aa).link;
    }
    if numFileNames == 0 as libc::c_int {
        srcMode = 1 as libc::c_int;
    } else {
        srcMode = 3 as libc::c_int;
    }
    opMode = 1 as libc::c_int;
    if !(strstr(progName, b"unzip\0" as *const u8 as *const libc::c_char)).is_null()
        || !(strstr(progName, b"UNZIP\0" as *const u8 as *const libc::c_char)).is_null()
    {
        opMode = 2 as libc::c_int;
    }
    if !(strstr(progName, b"z2cat\0" as *const u8 as *const libc::c_char)).is_null()
        || !(strstr(progName, b"Z2CAT\0" as *const u8 as *const libc::c_char)).is_null()
        || !(strstr(progName, b"zcat\0" as *const u8 as *const libc::c_char)).is_null()
        || !(strstr(progName, b"ZCAT\0" as *const u8 as *const libc::c_char)).is_null()
    {
        opMode = 2 as libc::c_int;
        srcMode = if numFileNames == 0 as libc::c_int {
            1 as libc::c_int
        } else {
            2 as libc::c_int
        };
    }
    aa = argList;
    while !aa.is_null() {
        if strcmp((*aa).name, b"--\0" as *const u8 as *const libc::c_char) == 0 as libc::c_int {
            break;
        }
        if *((*aa).name).offset(0 as libc::c_int as isize) as libc::c_int == '-' as i32
            && *((*aa).name).offset(1 as libc::c_int as isize) as libc::c_int != '-' as i32
        {
            let mut j = 1 as libc::c_int;
            while *((*aa).name).offset(j as isize) as libc::c_int != '\0' as i32 {
                match *((*aa).name).offset(j as isize) as libc::c_int {
                    99 => {
                        srcMode = 2 as libc::c_int;
                    }
                    100 => {
                        opMode = 2 as libc::c_int;
                    }
                    122 => {
                        opMode = 1 as libc::c_int;
                    }
                    102 => {
                        forceOverwrite = 1 as Bool;
                    }
                    116 => {
                        opMode = 3 as libc::c_int;
                    }
                    107 => {
                        keepInputFiles = 1 as Bool;
                    }
                    115 => {
                        smallMode = 1 as Bool;
                    }
                    113 => {
                        noisy = 0 as Bool;
                    }
                    49 => {
                        blockSize100k = 1 as libc::c_int;
                    }
                    50 => {
                        blockSize100k = 2 as libc::c_int;
                    }
                    51 => {
                        blockSize100k = 3 as libc::c_int;
                    }
                    52 => {
                        blockSize100k = 4 as libc::c_int;
                    }
                    53 => {
                        blockSize100k = 5 as libc::c_int;
                    }
                    54 => {
                        blockSize100k = 6 as libc::c_int;
                    }
                    55 => {
                        blockSize100k = 7 as libc::c_int;
                    }
                    56 => {
                        blockSize100k = 8 as libc::c_int;
                    }
                    57 => {
                        blockSize100k = 9 as libc::c_int;
                    }
                    86 | 76 => {
                        license();
                        exit(0 as libc::c_int);
                    }
                    118 => {
                        verbosity += 1;
                    }
                    104 => {
                        usage(progName);
                        exit(0 as libc::c_int);
                    }
                    _ => {
                        fprintf(
                            stderr,
                            b"%s: Bad flag `%s'\n\0" as *const u8 as *const libc::c_char,
                            progName,
                            (*aa).name,
                        );
                        usage(progName);
                        exit(1 as libc::c_int);
                    }
                }
                j += 1;
            }
        }
        aa = (*aa).link;
    }
    aa = argList;
    while !aa.is_null() {
        if strcmp((*aa).name, b"--\0" as *const u8 as *const libc::c_char) == 0 as libc::c_int {
            break;
        }
        if strcmp(
            (*aa).name,
            b"--stdout\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            srcMode = 2 as libc::c_int;
        } else if strcmp(
            (*aa).name,
            b"--decompress\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            opMode = 2 as libc::c_int;
        } else if strcmp(
            (*aa).name,
            b"--compress\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            opMode = 1 as libc::c_int;
        } else if strcmp((*aa).name, b"--force\0" as *const u8 as *const libc::c_char)
            == 0 as libc::c_int
        {
            forceOverwrite = 1 as Bool;
        } else if strcmp((*aa).name, b"--test\0" as *const u8 as *const libc::c_char)
            == 0 as libc::c_int
        {
            opMode = 3 as libc::c_int;
        } else if strcmp((*aa).name, b"--keep\0" as *const u8 as *const libc::c_char)
            == 0 as libc::c_int
        {
            keepInputFiles = 1 as Bool;
        } else if strcmp((*aa).name, b"--small\0" as *const u8 as *const libc::c_char)
            == 0 as libc::c_int
        {
            smallMode = 1 as Bool;
        } else if strcmp((*aa).name, b"--quiet\0" as *const u8 as *const libc::c_char)
            == 0 as libc::c_int
        {
            noisy = 0 as Bool;
        } else if strcmp((*aa).name, c"--version".as_ptr()) == 0
            || strcmp((*aa).name, c"--license".as_ptr()) == 0
        {
            license();
            exit(0);
        } else if strcmp(
            (*aa).name,
            b"--exponential\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            workFactor = 1 as libc::c_int;
        } else if strcmp((*aa).name, c"--repetitive-best".as_ptr()) == 0
            || strcmp((*aa).name, c"--repetitive-fast".as_ptr()) == 0
        {
            redundant((*aa).name);
        } else if strcmp((*aa).name, b"--fast\0" as *const u8 as *const libc::c_char)
            == 0 as libc::c_int
        {
            blockSize100k = 1 as libc::c_int;
        } else if strcmp((*aa).name, b"--best\0" as *const u8 as *const libc::c_char)
            == 0 as libc::c_int
        {
            blockSize100k = 9 as libc::c_int;
        } else if strcmp(
            (*aa).name,
            b"--verbose\0" as *const u8 as *const libc::c_char,
        ) == 0 as libc::c_int
        {
            verbosity += 1;
        } else if strcmp((*aa).name, b"--help\0" as *const u8 as *const libc::c_char)
            == 0 as libc::c_int
        {
            usage(progName);
            exit(0 as libc::c_int);
        } else if strncmp(
            (*aa).name,
            b"--\0" as *const u8 as *const libc::c_char,
            2 as libc::c_int as libc::size_t,
        ) == 0 as libc::c_int
        {
            fprintf(
                stderr,
                b"%s: Bad flag `%s'\n\0" as *const u8 as *const libc::c_char,
                progName,
                (*aa).name,
            );
            usage(progName);
            exit(1 as libc::c_int);
        }
        aa = (*aa).link;
    }
    if verbosity > 4 as libc::c_int {
        verbosity = 4 as libc::c_int;
    }
    if opMode == 1 as libc::c_int
        && smallMode as libc::c_int != 0
        && blockSize100k > 2 as libc::c_int
    {
        blockSize100k = 2 as libc::c_int;
    }
    if opMode == 3 as libc::c_int && srcMode == 2 as libc::c_int {
        fprintf(
            stderr,
            b"%s: -c and -t cannot be used together.\n\0" as *const u8 as *const libc::c_char,
            progName,
        );
        exit(1 as libc::c_int);
    }
    if srcMode == 2 as libc::c_int && numFileNames == 0 as libc::c_int {
        srcMode = 1 as libc::c_int;
    }
    if opMode != 1 as libc::c_int {
        blockSize100k = 0 as libc::c_int;
    }
    if srcMode == 3 as libc::c_int {
        signal(
            2 as libc::c_int,
            mySignalCatcher as unsafe extern "C" fn(IntNative) as usize,
        );
        signal(
            15 as libc::c_int,
            mySignalCatcher as unsafe extern "C" fn(IntNative) as usize,
        );
        signal(
            1 as libc::c_int,
            mySignalCatcher as unsafe extern "C" fn(IntNative) as usize,
        );
    }
    if opMode == 1 as libc::c_int {
        if srcMode == 1 as libc::c_int {
            compress(std::ptr::null_mut());
        } else {
            decode = 1 as Bool;
            aa = argList;
            while !aa.is_null() {
                if strcmp((*aa).name, b"--\0" as *const u8 as *const libc::c_char)
                    == 0 as libc::c_int
                {
                    decode = 0 as Bool;
                } else if !(*((*aa).name).offset(0 as libc::c_int as isize) as libc::c_int
                    == '-' as i32
                    && decode as libc::c_int != 0)
                {
                    numFilesProcessed += 1;
                    compress((*aa).name);
                }
                aa = (*aa).link;
            }
        }
    } else if opMode == 2 as libc::c_int {
        unzFailsExist = 0 as Bool;
        if srcMode == 1 as libc::c_int {
            uncompress(std::ptr::null_mut());
        } else {
            decode = 1 as Bool;
            aa = argList;
            while !aa.is_null() {
                if strcmp((*aa).name, b"--\0" as *const u8 as *const libc::c_char)
                    == 0 as libc::c_int
                {
                    decode = 0 as Bool;
                } else if !(*((*aa).name).offset(0 as libc::c_int as isize) as libc::c_int
                    == '-' as i32
                    && decode as libc::c_int != 0)
                {
                    numFilesProcessed += 1;
                    uncompress((*aa).name);
                }
                aa = (*aa).link;
            }
        }
        if unzFailsExist != 0 {
            setExit(2 as libc::c_int);
            exit(exitValue);
        }
    } else {
        testFailsExist = 0 as Bool;
        if srcMode == 1 as libc::c_int {
            testf(std::ptr::null_mut());
        } else {
            decode = 1 as Bool;
            aa = argList;
            while !aa.is_null() {
                if strcmp((*aa).name, b"--\0" as *const u8 as *const libc::c_char)
                    == 0 as libc::c_int
                {
                    decode = 0 as Bool;
                } else if !(*((*aa).name).offset(0 as libc::c_int as isize) as libc::c_int
                    == '-' as i32
                    && decode as libc::c_int != 0)
                {
                    numFilesProcessed += 1;
                    testf((*aa).name);
                }
                aa = (*aa).link;
            }
        }
        if testFailsExist != 0 {
            if noisy != 0 {
                fprintf(
                    stderr,
                    b"\nYou can use the `bzip2recover' program to attempt to recover\ndata from undamaged sections of corrupted files.\n\n\0"
                        as *const u8 as *const libc::c_char,
                );
            }
            setExit(2 as libc::c_int);
            exit(exitValue);
        }
    }
    aa = argList;
    while !aa.is_null() {
        let aa2: *mut Cell = (*aa).link;
        if !((*aa).name).is_null() {
            free((*aa).name as *mut libc::c_void);
        }
        free(aa as *mut libc::c_void);
        aa = aa2;
    }
    exitValue
}
fn main() {
    let mut args: Vec<*mut libc::c_char> = Vec::new();
    for arg in ::std::env::args() {
        args.push(
            (::std::ffi::CString::new(arg))
                .expect("Failed to convert argument into CString.")
                .into_raw(),
        );
    }
    args.push(core::ptr::null_mut());
    unsafe { ::std::process::exit(main_0((args.len() - 1) as IntNative, args.as_mut_ptr()) as i32) }
}
