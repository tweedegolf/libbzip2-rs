#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::c_char;

use libc::{fprintf, FILE};

use libc::{
    __errno_location, close, exit, fclose, fdopen, fflush, fopen, free, malloc, open, perror,
    sprintf, strcat, strcpy, strlen, strncpy, strrchr,
};

extern "C" {
    static mut stderr: *mut FILE;
    fn getc(__stream: *mut FILE) -> libc::c_int;
    fn putc(__c: libc::c_int, __stream: *mut FILE) -> libc::c_int;
}
pub type MaybeUInt64 = libc::c_ulonglong;
pub type Bool = libc::c_uchar;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BitStream {
    pub handle: *mut FILE,
    pub buffer: i32,
    pub buffLive: i32,
    pub mode: u8,
}
pub static mut IN_FILENAME: [c_char; 2000] = [0; 2000];
pub static mut OUT_FILENAME: [c_char; 2000] = [0; 2000];
pub static mut PROGNAME: [c_char; 2000] = [0; 2000];
pub static mut BYTES_OUT: MaybeUInt64 = 0 as libc::c_int as MaybeUInt64;
pub static mut BYTES_IN: MaybeUInt64 = 0 as libc::c_int as MaybeUInt64;
unsafe fn readError() {
    fprintf(
        stderr,
        b"%s: I/O error reading `%s', possible reason follows.\n\0" as *const u8
            as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
        IN_FILENAME.as_mut_ptr(),
    );
    perror(PROGNAME.as_mut_ptr());
    fprintf(
        stderr,
        b"%s: warning: output file(s) may be incomplete.\n\0" as *const u8 as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
    );
    exit(1 as libc::c_int);
}
unsafe fn writeError() {
    fprintf(
        stderr,
        b"%s: I/O error reading `%s', possible reason follows.\n\0" as *const u8
            as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
        IN_FILENAME.as_mut_ptr(),
    );
    perror(PROGNAME.as_mut_ptr());
    fprintf(
        stderr,
        b"%s: warning: output file(s) may be incomplete.\n\0" as *const u8 as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
    );
    exit(1 as libc::c_int);
}
unsafe fn mallocFail(n: i32) {
    fprintf(
        stderr,
        b"%s: malloc failed on request for %d bytes.\n\0" as *const u8 as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
        n,
    );
    fprintf(
        stderr,
        b"%s: warning: output file(s) may be incomplete.\n\0" as *const u8 as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
    );
    exit(1 as libc::c_int);
}
unsafe fn tooManyBlocks(max_handled_blocks: i32) {
    fprintf(
        stderr,
        b"%s: `%s' appears to contain more than %d blocks\n\0" as *const u8 as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
        IN_FILENAME.as_mut_ptr(),
        max_handled_blocks,
    );
    fprintf(
        stderr,
        b"%s: and cannot be handled.  To fix, increase\n\0" as *const u8 as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
    );
    fprintf(
        stderr,
        b"%s: BZ_MAX_HANDLED_BLOCKS in bzip2recover.c, and recompile.\n\0" as *const u8
            as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
    );
    exit(1 as libc::c_int);
}
unsafe fn bsOpenReadStream(stream: *mut FILE) -> *mut BitStream {
    let bs: *mut BitStream = malloc(core::mem::size_of::<BitStream>()) as *mut BitStream;
    if bs.is_null() {
        mallocFail(::core::mem::size_of::<BitStream>() as libc::c_ulong as i32);
    }
    (*bs).handle = stream;
    (*bs).buffer = 0 as libc::c_int;
    (*bs).buffLive = 0 as libc::c_int;
    (*bs).mode = b'r';
    bs
}
unsafe fn bsOpenWriteStream(stream: *mut FILE) -> *mut BitStream {
    let bs: *mut BitStream = malloc(core::mem::size_of::<BitStream>()) as *mut BitStream;
    if bs.is_null() {
        mallocFail(::core::mem::size_of::<BitStream>() as libc::c_ulong as i32);
    }
    (*bs).handle = stream;
    (*bs).buffer = 0 as libc::c_int;
    (*bs).buffLive = 0 as libc::c_int;
    (*bs).mode = b'w';
    bs
}
unsafe fn bsPutBit(bs: *mut BitStream, bit: i32) {
    if (*bs).buffLive == 8 as libc::c_int {
        let retVal: i32 = putc((*bs).buffer as u8 as libc::c_int, (*bs).handle);
        if retVal == -1 as libc::c_int {
            writeError();
        }
        BYTES_OUT = BYTES_OUT.wrapping_add(1);
        (*bs).buffLive = 1 as libc::c_int;
        (*bs).buffer = bit & 0x1 as libc::c_int;
    } else {
        (*bs).buffer = (*bs).buffer << 1 as libc::c_int | bit & 0x1 as libc::c_int;
        (*bs).buffLive += 1;
    };
}
unsafe fn bsGetBit(bs: *mut BitStream) -> i32 {
    if (*bs).buffLive > 0 as libc::c_int {
        (*bs).buffLive -= 1;
        (*bs).buffer >> (*bs).buffLive & 0x1 as libc::c_int
    } else {
        let retVal: i32 = getc((*bs).handle);
        if retVal == -1 as libc::c_int {
            if *__errno_location() != 0 as libc::c_int {
                readError()
            } else {
                return 2;
            }
        }
        (*bs).buffLive = 7 as libc::c_int;
        (*bs).buffer = retVal;
        (*bs).buffer >> 7 as libc::c_int & 0x1 as libc::c_int
    }
}
unsafe fn bsClose(bs: *mut BitStream) {
    let mut retVal: i32;
    if (*bs).mode == b'w' {
        while (*bs).buffLive < 8 as libc::c_int {
            (*bs).buffLive += 1;
            (*bs).buffer <<= 1 as libc::c_int;
        }
        retVal = putc((*bs).buffer as u8 as libc::c_int, (*bs).handle);
        if retVal == -1 as libc::c_int {
            writeError();
        }
        BYTES_OUT = BYTES_OUT.wrapping_add(1);
        retVal = fflush((*bs).handle);
        if retVal == -1 as libc::c_int {
            writeError();
        }
    }
    retVal = fclose((*bs).handle);
    if retVal == -1 as libc::c_int {
        if (*bs).mode == b'w' {
            writeError();
        } else {
            readError();
        }
    }
    free(bs as *mut libc::c_void);
}
unsafe fn bsPutUChar(bs: *mut BitStream, c: u8) {
    let mut i: i32 = 7;
    while i >= 0 {
        bsPutBit(
            bs,
            (c as u32 >> i & 0x1 as libc::c_int as libc::c_uint) as i32,
        );
        i -= 1;
    }
}
unsafe fn bsPutUInt32(bs: *mut BitStream, c: u32) {
    let mut i: i32 = 31;
    while i >= 0 {
        bsPutBit(bs, (c >> i & 0x1 as libc::c_int as libc::c_uint) as i32);
        i -= 1;
    }
}
unsafe fn endsInBz2(name: *mut c_char) -> Bool {
    let n: i32 = strlen(name) as i32;
    if n <= 4 as libc::c_int {
        return 0 as Bool;
    }
    (*name.offset((n - 4 as libc::c_int) as isize) as libc::c_int == '.' as i32
        && *name.offset((n - 3 as libc::c_int) as isize) as libc::c_int == 'b' as i32
        && *name.offset((n - 2 as libc::c_int) as isize) as libc::c_int == 'z' as i32
        && *name.offset((n - 1 as libc::c_int) as isize) as libc::c_int == '2' as i32) as Bool
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
pub static mut B_START: [MaybeUInt64; 50000] = [0; 50000];
pub static mut B_END: [MaybeUInt64; 50000] = [0; 50000];
pub static mut RB_START: [MaybeUInt64; 50000] = [0; 50000];
pub static mut RB_END: [MaybeUInt64; 50000] = [0; 50000];
unsafe fn main_0(argc: i32, argv: *mut *mut c_char) -> i32 {
    strncpy(
        PROGNAME.as_mut_ptr(),
        *argv.offset(0 as libc::c_int as isize),
        (2000 as libc::c_int - 1 as libc::c_int) as usize,
    );
    PROGNAME[(2000 as libc::c_int - 1 as libc::c_int) as usize] = '\0' as i32 as c_char;
    OUT_FILENAME[0 as libc::c_int as usize] = 0 as libc::c_int as c_char;
    IN_FILENAME[0 as libc::c_int as usize] = OUT_FILENAME[0 as libc::c_int as usize];
    fprintf(
        stderr,
        b"bzip2recover 1.0.6: extracts blocks from damaged .bz2 files.\n\0" as *const u8
            as *const libc::c_char,
    );
    if argc != 2 as libc::c_int {
        fprintf(
            stderr,
            b"%s: usage is `%s damaged_file_name'.\n\0" as *const u8 as *const libc::c_char,
            PROGNAME.as_mut_ptr(),
            PROGNAME.as_mut_ptr(),
        );
        match core::mem::size_of::<MaybeUInt64>() as libc::c_ulong {
            8 => {
                fprintf(
                    stderr,
                    b"\trestrictions on size of recovered file: None\n\0" as *const u8
                        as *const libc::c_char,
                );
            }
            4 => {
                fprintf(
                    stderr,
                    b"\trestrictions on size of recovered file: 512 MB\n\0" as *const u8
                        as *const libc::c_char,
                );
                fprintf(
                    stderr,
                    b"\tto circumvent, recompile with MaybeUInt64 as an\n\tunsigned 64-bit int.\n\0"
                        as *const u8 as *const libc::c_char,
                );
            }
            _ => {
                fprintf(
                    stderr,
                    b"\tsizeof(MaybeUInt64) is not 4 or 8 -- configuration error.\n\0" as *const u8
                        as *const libc::c_char,
                );
            }
        }
        exit(1 as libc::c_int);
    }
    if strlen(*argv.offset(1 as libc::c_int as isize))
        >= (2000 as libc::c_int - 20 as libc::c_int) as usize
    {
        fprintf(
            stderr,
            b"%s: supplied filename is suspiciously (>= %d chars) long.  Bye!\n\0" as *const u8
                as *const libc::c_char,
            PROGNAME.as_mut_ptr(),
            strlen(*argv.offset(1 as libc::c_int as isize)) as libc::c_int,
        );
        exit(1 as libc::c_int);
    }
    strcpy(
        IN_FILENAME.as_mut_ptr(),
        *argv.offset(1 as libc::c_int as isize),
    );
    let mut inFile = fopen(
        IN_FILENAME.as_mut_ptr(),
        b"rb\0" as *const u8 as *const libc::c_char,
    );
    if inFile.is_null() {
        fprintf(
            stderr,
            b"%s: can't read `%s'\n\0" as *const u8 as *const libc::c_char,
            PROGNAME.as_mut_ptr(),
            IN_FILENAME.as_mut_ptr(),
        );
        exit(1 as libc::c_int);
    }
    let mut bsIn = bsOpenReadStream(inFile);
    fprintf(
        stderr,
        b"%s: searching for block boundaries ...\n\0" as *const u8 as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
    );
    let mut bitsRead = 0 as libc::c_int as MaybeUInt64;
    let mut buffLo = 0 as libc::c_int as u32;
    let mut buffHi = buffLo;
    let mut currBlock = 0 as libc::c_int;
    B_START[currBlock as usize] = 0 as libc::c_int as MaybeUInt64;
    let mut rbCtr = 0 as libc::c_int;
    loop {
        let b = bsGetBit(bsIn);
        bitsRead = bitsRead.wrapping_add(1);
        if b == 2 {
            if bitsRead >= B_START[currBlock as usize]
                && bitsRead.wrapping_sub(B_START[currBlock as usize])
                    >= 40 as libc::c_int as libc::c_ulonglong
            {
                B_END[currBlock as usize] =
                    bitsRead.wrapping_sub(1 as libc::c_int as libc::c_ulonglong);
                if currBlock > 0 as libc::c_int {
                    fprintf(
                        stderr,
                        b"   block %d runs from %Lu to %Lu (incomplete)\n\0" as *const u8
                            as *const libc::c_char,
                        currBlock,
                        B_START[currBlock as usize],
                        B_END[currBlock as usize],
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
                    B_END[currBlock as usize] = 0 as libc::c_int as MaybeUInt64;
                }
                if currBlock > 0 as libc::c_int
                    && (B_END[currBlock as usize]).wrapping_sub(B_START[currBlock as usize])
                        >= 130 as libc::c_int as libc::c_ulonglong
                {
                    fprintf(
                        stderr,
                        b"   block %d runs from %Lu to %Lu\n\0" as *const u8 as *const libc::c_char,
                        rbCtr + 1 as libc::c_int,
                        B_START[currBlock as usize],
                        B_END[currBlock as usize],
                    );
                    RB_START[rbCtr as usize] = B_START[currBlock as usize];
                    RB_END[rbCtr as usize] = B_END[currBlock as usize];
                    rbCtr += 1;
                }
                if currBlock >= 50000 as libc::c_int {
                    tooManyBlocks(50000 as libc::c_int);
                }
                currBlock += 1;
                B_START[currBlock as usize] = bitsRead;
            }
        }
    }
    bsClose(bsIn);
    if rbCtr < 1 as libc::c_int {
        fprintf(
            stderr,
            b"%s: sorry, I couldn't find any block boundaries.\n\0" as *const u8
                as *const libc::c_char,
            PROGNAME.as_mut_ptr(),
        );
        exit(1 as libc::c_int);
    }
    fprintf(
        stderr,
        b"%s: splitting into blocks\n\0" as *const u8 as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
    );
    inFile = fopen(
        IN_FILENAME.as_mut_ptr(),
        b"rb\0" as *const u8 as *const libc::c_char,
    );
    if inFile.is_null() {
        fprintf(
            stderr,
            b"%s: can't open `%s'\n\0" as *const u8 as *const libc::c_char,
            PROGNAME.as_mut_ptr(),
            IN_FILENAME.as_mut_ptr(),
        );
        exit(1 as libc::c_int);
    }
    bsIn = bsOpenReadStream(inFile);
    let mut blockCRC = 0 as libc::c_int as u32;
    let mut bsWr = std::ptr::null_mut::<BitStream>();
    bitsRead = 0 as libc::c_int as MaybeUInt64;
    let mut outFile = std::ptr::null_mut::<FILE>();
    let mut wrBlock = 0 as libc::c_int;
    loop {
        let b = bsGetBit(bsIn);
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
        if !outFile.is_null()
            && bitsRead >= RB_START[wrBlock as usize]
            && bitsRead <= RB_END[wrBlock as usize]
        {
            bsPutBit(bsWr, b);
        }
        bitsRead = bitsRead.wrapping_add(1);
        if bitsRead
            == (RB_END[wrBlock as usize]).wrapping_add(1 as libc::c_int as libc::c_ulonglong)
        {
            if !outFile.is_null() {
                bsPutUChar(bsWr, 0x17 as libc::c_int as u8);
                bsPutUChar(bsWr, 0x72 as libc::c_int as u8);
                bsPutUChar(bsWr, 0x45 as libc::c_int as u8);
                bsPutUChar(bsWr, 0x38 as libc::c_int as u8);
                bsPutUChar(bsWr, 0x50 as libc::c_int as u8);
                bsPutUChar(bsWr, 0x90 as libc::c_int as u8);
                bsPutUInt32(bsWr, blockCRC);
                bsClose(bsWr);
                outFile = std::ptr::null_mut::<FILE>();
            }
            if wrBlock >= rbCtr {
                break;
            }
            wrBlock += 1;
        } else if bitsRead == RB_START[wrBlock as usize] {
            let mut k = 0 as libc::c_int;
            while k < 2000 as libc::c_int {
                OUT_FILENAME[k as usize] = 0 as libc::c_int as c_char;
                k += 1;
            }
            strcpy(OUT_FILENAME.as_mut_ptr(), IN_FILENAME.as_mut_ptr());
            let mut split = strrchr(OUT_FILENAME.as_mut_ptr(), '/' as i32);
            if split.is_null() {
                split = OUT_FILENAME.as_mut_ptr();
            } else {
                split = split.offset(1);
            }
            let ofs = split.offset_from(OUT_FILENAME.as_mut_ptr()) as libc::c_long as i32;
            sprintf(
                split,
                b"rec%5d\0" as *const u8 as *const libc::c_char,
                wrBlock + 1 as libc::c_int,
            );
            let mut p = split;
            while *p != 0 {
                if *p as libc::c_int == ' ' as i32 {
                    *p = '0' as i32 as c_char;
                }
                p = p.offset(1);
            }
            strcat(
                OUT_FILENAME.as_mut_ptr(),
                IN_FILENAME.as_mut_ptr().offset(ofs as isize),
            );
            if endsInBz2(OUT_FILENAME.as_mut_ptr()) == 0 {
                strcat(
                    OUT_FILENAME.as_mut_ptr(),
                    b".bz2\0" as *const u8 as *const libc::c_char,
                );
            }
            fprintf(
                stderr,
                b"   writing block %d to `%s' ...\n\0" as *const u8 as *const libc::c_char,
                wrBlock + 1 as libc::c_int,
                OUT_FILENAME.as_mut_ptr(),
            );
            outFile = fopen_output_safely(
                OUT_FILENAME.as_mut_ptr(),
                b"wb\0" as *const u8 as *const libc::c_char,
            );
            if outFile.is_null() {
                fprintf(
                    stderr,
                    b"%s: can't write `%s'\n\0" as *const u8 as *const libc::c_char,
                    PROGNAME.as_mut_ptr(),
                    OUT_FILENAME.as_mut_ptr(),
                );
                exit(1 as libc::c_int);
            }
            bsWr = bsOpenWriteStream(outFile);
            bsPutUChar(bsWr, 0x42 as libc::c_int as u8);
            bsPutUChar(bsWr, 0x5a as libc::c_int as u8);
            bsPutUChar(bsWr, 0x68 as libc::c_int as u8);
            bsPutUChar(bsWr, (0x30 as libc::c_int + 9 as libc::c_int) as u8);
            bsPutUChar(bsWr, 0x31 as libc::c_int as u8);
            bsPutUChar(bsWr, 0x41 as libc::c_int as u8);
            bsPutUChar(bsWr, 0x59 as libc::c_int as u8);
            bsPutUChar(bsWr, 0x26 as libc::c_int as u8);
            bsPutUChar(bsWr, 0x53 as libc::c_int as u8);
            bsPutUChar(bsWr, 0x59 as libc::c_int as u8);
        }
    }
    fprintf(
        stderr,
        b"%s: finished\n\0" as *const u8 as *const libc::c_char,
        PROGNAME.as_mut_ptr(),
    );
    0 as libc::c_int
}
pub fn main() {
    let mut args: Vec<*mut libc::c_char> = Vec::new();
    for arg in ::std::env::args() {
        args.push(
            (::std::ffi::CString::new(arg))
                .expect("Failed to convert argument into CString.")
                .into_raw(),
        );
    }
    args.push(core::ptr::null_mut());
    unsafe { ::std::process::exit(main_0((args.len() - 1) as i32, args.as_mut_ptr())) }
}
