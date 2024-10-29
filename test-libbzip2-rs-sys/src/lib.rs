#[cfg(test)]
#[macro_export]
macro_rules! assert_eq_rs_c {
    ($tt:tt) => {{
        #[cfg(not(miri))]
        #[allow(clippy::macro_metavars_in_unsafe)]
        #[allow(unused_braces)]
        #[allow(unused_unsafe)]
        let _ng = unsafe {
            use bzip2_sys::*;

            $tt
        };

        #[allow(clippy::macro_metavars_in_unsafe)]
        #[allow(unused_braces)]
        #[allow(unused_unsafe)]
        let _rs = unsafe {
            use libbzip2_rs_sys::bzlib::{
                bz_stream, BZ2_bzDecompress, BZ2_bzDecompressEnd, BZ2_bzDecompressInit,
            };

            $tt
        };

        #[cfg(not(miri))]
        assert_eq!(_rs, _ng);

        _rs
    }};
}

#[test]
fn decode_sample1() {
    let input = include_bytes!("../../tests/input/quick/sample1.bz2");

    assert_eq_rs_c!({
        pub unsafe fn BZ2_bzBuffToBuffDecompress(
            mut dest: *mut libc::c_char,
            mut destLen: *mut libc::c_uint,
            mut source: *mut libc::c_char,
            mut sourceLen: libc::c_uint,
            mut small: libc::c_int,
            mut verbosity: libc::c_int,
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
            let mut ret: libc::c_int = 0;
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

        let mut dest = vec![0; 1024];
        let mut dest_len = dest.len() as core::ffi::c_uint;

        BZ2_bzBuffToBuffDecompress(
            dest.as_mut_ptr(),
            &mut dest_len,
            input.as_ptr() as *mut core::ffi::c_char,
            input.len() as core::ffi::c_uint,
            0,
            0,
        );

        dest.truncate(dest_len as usize);

        dest
    });
}
