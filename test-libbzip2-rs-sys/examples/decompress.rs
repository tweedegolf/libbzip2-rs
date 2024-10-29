use core::ffi::c_uint;

fn main() {
    let mut it = std::env::args();

    let _ = it.next().unwrap();

    match it.next().unwrap().as_str() {
        "c" => {
            let path = it.next().unwrap();
            let input = std::fs::read(&path).unwrap();

            let mut dest_vec = vec![0u8; 1 << 28];

            let mut dest_len = dest_vec.len() as c_uint;
            let dest = dest_vec.as_mut_ptr();

            let source = input.as_ptr();
            let source_len = input.len() as _;

            let err = unsafe { decompress_c(dest, &mut dest_len, source, source_len) };

            if err != 0 {
                panic!("error {err}");
            }

            dest_vec.truncate(dest_len as usize);

            drop(dest_vec)
        }
        "rs" => {
            let path = it.next().unwrap();
            let input = std::fs::read(&path).unwrap();

            let mut dest_vec = vec![0u8; 1 << 28];

            let mut dest_len = dest_vec.len() as std::ffi::c_uint;
            let dest = dest_vec.as_mut_ptr();

            let source = input.as_ptr();
            let source_len = input.len() as _;

            let err = unsafe { decompress_rs(dest, &mut dest_len, source, source_len) };

            if err != 0 {
                panic!("error {err}");
            }

            dest_vec.truncate(dest_len as usize);

            drop(dest_vec)
        }
        other => panic!("invalid option '{other}', expected one of 'c' or 'rs'"),
    }
}

fn decompress_c<'a>(
    dest: *mut u8,
    destLen: *mut libc::c_uint,
    source: *const u8,
    sourceLen: libc::c_uint,
) -> i32 {
    use bzip2_sys::*;

    pub unsafe fn BZ2_bzBuffToBuffDecompress(
        dest: *mut libc::c_char,
        destLen: *mut libc::c_uint,
        source: *mut libc::c_char,
        sourceLen: libc::c_uint,
        small: libc::c_int,
        verbosity: libc::c_int,
    ) -> libc::c_int {
        let mut strm: bz_stream = bz_stream {
            next_in: std::ptr::null_mut::<libc::c_char>(),
            avail_in: 0,
            total_in_lo32: 0,
            total_in_hi32: 0,
            next_out: std::ptr::null_mut::<libc::c_char>(),
            avail_out: 0,
            total_out_lo32: 0,
            total_out_hi32: 0,
            state: std::ptr::null_mut::<libc::c_void>(),
            bzalloc: None,
            bzfree: None,
            opaque: std::ptr::null_mut::<libc::c_void>(),
        };
        let mut ret: libc::c_int;
        if dest.is_null()
            || destLen.is_null()
            || source.is_null()
            || small != 0 as libc::c_int && small != 1 as libc::c_int
            || verbosity < 0 as libc::c_int
            || verbosity > 4 as libc::c_int
        {
            return -(2 as libc::c_int);
        }
        strm.bzalloc = None;
        strm.bzfree = None;
        strm.opaque = std::ptr::null_mut::<libc::c_void>();
        ret = BZ2_bzDecompressInit(&mut strm, verbosity, small);
        if ret != 0 as libc::c_int {
            return ret;
        }
        strm.next_in = source;
        strm.next_out = dest;
        strm.avail_in = sourceLen;
        strm.avail_out = *destLen;
        ret = BZ2_bzDecompress(&mut strm);
        if ret == 0 as libc::c_int {
            if strm.avail_out > 0 as libc::c_int as libc::c_uint {
                BZ2_bzDecompressEnd(&mut strm);
                -(7 as libc::c_int)
            } else {
                BZ2_bzDecompressEnd(&mut strm);
                -(8 as libc::c_int)
            }
        } else if ret != 4 as libc::c_int {
            BZ2_bzDecompressEnd(&mut strm);
            return ret;
        } else {
            *destLen = (*destLen).wrapping_sub(strm.avail_out);
            BZ2_bzDecompressEnd(&mut strm);
            return 0 as libc::c_int;
        }
    }

    unsafe {
        BZ2_bzBuffToBuffDecompress(
            dest.cast::<core::ffi::c_char>(),
            destLen,
            source as *mut _,
            sourceLen,
            0,
            0,
        )
    }
}

fn decompress_rs<'a>(
    dest: *mut u8,
    destLen: *mut libc::c_uint,
    source: *const u8,
    sourceLen: libc::c_uint,
) -> i32 {
    use libbzip2_rs_sys::bzlib::*;

    pub unsafe fn BZ2_bzBuffToBuffDecompress(
        dest: *mut libc::c_char,
        destLen: *mut libc::c_uint,
        source: *mut libc::c_char,
        sourceLen: libc::c_uint,
        small: libc::c_int,
        verbosity: libc::c_int,
    ) -> libc::c_int {
        let mut strm: bz_stream = bz_stream {
            next_in: std::ptr::null_mut::<libc::c_char>(),
            avail_in: 0,
            total_in_lo32: 0,
            total_in_hi32: 0,
            next_out: std::ptr::null_mut::<libc::c_char>(),
            avail_out: 0,
            total_out_lo32: 0,
            total_out_hi32: 0,
            state: std::ptr::null_mut::<libc::c_void>(),
            bzalloc: None,
            bzfree: None,
            opaque: std::ptr::null_mut::<libc::c_void>(),
        };
        let mut ret: libc::c_int;
        if dest.is_null()
            || destLen.is_null()
            || source.is_null()
            || small != 0 as libc::c_int && small != 1 as libc::c_int
            || verbosity < 0 as libc::c_int
            || verbosity > 4 as libc::c_int
        {
            return -(2 as libc::c_int);
        }
        strm.bzalloc = None;
        strm.bzfree = None;
        strm.opaque = std::ptr::null_mut::<libc::c_void>();
        ret = BZ2_bzDecompressInit(&mut strm, verbosity, small);
        if ret != 0 as libc::c_int {
            return ret;
        }
        strm.next_in = source;
        strm.next_out = dest;
        strm.avail_in = sourceLen;
        strm.avail_out = *destLen;
        ret = BZ2_bzDecompress(&mut strm);
        if ret == 0 as libc::c_int {
            if strm.avail_out > 0 as libc::c_int as libc::c_uint {
                BZ2_bzDecompressEnd(&mut strm);
                -(7 as libc::c_int)
            } else {
                BZ2_bzDecompressEnd(&mut strm);
                -(8 as libc::c_int)
            }
        } else if ret != 4 as libc::c_int {
            BZ2_bzDecompressEnd(&mut strm);
            return ret;
        } else {
            *destLen = (*destLen).wrapping_sub(strm.avail_out);
            BZ2_bzDecompressEnd(&mut strm);
            return 0 as libc::c_int;
        }
    }

    unsafe {
        BZ2_bzBuffToBuffDecompress(
            dest.cast::<core::ffi::c_char>(),
            destLen,
            source as *mut _,
            sourceLen,
            0,
            0,
        )
    }
}
