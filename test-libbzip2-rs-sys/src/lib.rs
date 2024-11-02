#![allow(dead_code, unused_imports, unused_macros, non_snake_case)]

use std::{
    ffi::{c_char, c_void, CStr},
    path::{Path, PathBuf},
};

mod chunked;

const SAMPLE1_REF: &[u8] = include_bytes!("../../tests/input/quick/sample1.ref");
const SAMPLE1_BZ2: &[u8] = include_bytes!("../../tests/input/quick/sample1.bz2");

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
            use compress_c as compress;
            use decompress_c as decompress;

            $tt
        };

        #[allow(clippy::macro_metavars_in_unsafe)]
        #[allow(unused_braces)]
        #[allow(unused_unsafe)]
        let _rs = unsafe {
            use compress_rs as compress;
            use decompress_rs as decompress;
            use libbzip2_rs_sys::bzlib::*;

            $tt
        };

        #[cfg(not(miri))]
        assert_eq!(_rs, _ng);

        _rs
    }};
}

macro_rules! assert_eq_decompress {
    ($input:literal) => {
        let input = include_bytes!($input);

        assert_eq_rs_c!({
            let mut dest = vec![0; 2 * input.len()];
            let mut dest_len = dest.len() as core::ffi::c_uint;

            decompress(
                dest.as_mut_ptr(),
                &mut dest_len,
                input.as_ptr(),
                input.len() as core::ffi::c_uint,
            );

            dest.truncate(dest_len as usize);

            dest
        });
    };
}

macro_rules! assert_eq_compress {
    ($input:literal) => {
        let input = include_bytes!($input);

        assert_eq_rs_c!({
            let mut dest = vec![0; 2 * input.len()];
            let mut dest_len = dest.len() as core::ffi::c_uint;

            compress(
                dest.as_mut_ptr(),
                &mut dest_len,
                input.as_ptr(),
                input.len() as core::ffi::c_uint,
            );

            dest.truncate(dest_len as usize);

            dest
        });
    };
}

#[test]
fn version() {
    let ptr = libbzip2_rs_sys::bzlib::BZ2_bzlibVersion();
    let cstr = unsafe { core::ffi::CStr::from_ptr(ptr) };
    let string = cstr.to_str().unwrap();

    assert!(string.starts_with("1.1.0"));
}

#[test]
fn buff_to_buff_compress_small() {
    let verbosity = 0;
    let blockSize100k = 9;
    let workFactor = 30;

    let input = b"lang is it ompaad";

    let mut dest = vec![0u8; 1024];
    let mut dest_len = dest.len() as core::ffi::c_uint;

    let err = unsafe {
        libbzip2_rs_sys::bzlib::BZ2_bzBuffToBuffCompress(
            dest.as_mut_ptr().cast::<core::ffi::c_char>(),
            &mut dest_len,
            input.as_ptr() as *mut _,
            input.len() as _,
            blockSize100k,
            verbosity,
            workFactor,
        )
    };

    assert_eq!(err, 0);
}

#[test]
fn buff_to_buff_compress() {
    let verbosity = 0;
    let blockSize100k = 9;
    let workFactor = 30;

    let mut dest = vec![0; 2 * SAMPLE1_REF.len()];
    let mut dest_len = dest.len() as core::ffi::c_uint;

    let err = unsafe {
        libbzip2_rs_sys::bzlib::BZ2_bzBuffToBuffCompress(
            dest.as_mut_ptr().cast::<core::ffi::c_char>(),
            &mut dest_len,
            SAMPLE1_REF.as_ptr() as *mut _,
            SAMPLE1_REF.len() as _,
            blockSize100k,
            verbosity,
            workFactor,
        )
    };

    assert_eq!(err, 0);
}

#[test]
fn buff_to_buff_decompress() {
    let mut dest = vec![0; SAMPLE1_REF.len()];
    let mut dest_len = dest.len() as core::ffi::c_uint;

    let err = unsafe {
        libbzip2_rs_sys::bzlib::BZ2_bzBuffToBuffDecompress(
            dest.as_mut_ptr().cast::<core::ffi::c_char>(),
            &mut dest_len,
            SAMPLE1_BZ2.as_ptr() as *mut _,
            SAMPLE1_BZ2.len() as _,
            0,
            0,
        )
    };

    assert_eq!(err, 0);
}

fn buff_to_buff_decompress_helper(input: &[u8], buffer_size: usize, is_small: bool) {
    let mut dest = vec![0; buffer_size];
    let mut dest_len = dest.len() as core::ffi::c_uint;

    let err = unsafe {
        libbzip2_rs_sys::bzlib::BZ2_bzBuffToBuffDecompress(
            dest.as_mut_ptr().cast::<core::ffi::c_char>(),
            &mut dest_len,
            input.as_ptr() as *mut _,
            input.len() as _,
            is_small as _,
            0,
        )
    };

    assert_eq!(err, 0);
}

#[test]
fn miri_buff_to_buff_decompress_fast() {
    let input: &[u8] = &[
        66u8, 90, 104, 57, 49, 65, 89, 38, 83, 89, 164, 38, 196, 174, 0, 0, 5, 17, 128, 64, 0, 36,
        167, 204, 0, 32, 0, 49, 3, 64, 208, 34, 105, 128, 122, 141, 161, 22, 187, 73, 99, 176, 39,
        11, 185, 34, 156, 40, 72, 82, 19, 98, 87, 0,
    ];

    buff_to_buff_decompress_helper(input, 1024, false)
}

#[test]
fn miri_buff_to_buff_decompress_small() {
    let input: &[u8] = &[
        66u8, 90, 104, 0x39, 49, 65, 89, 38, 83, 89, 164, 38, 196, 174, 0, 0, 5, 17, 128, 64, 0,
        36, 167, 204, 0, 32, 0, 49, 3, 64, 208, 34, 105, 128, 122, 141, 161, 22, 187, 73, 99, 176,
        39, 11, 185, 34, 156, 40, 72, 82, 19, 98, 87, 0,
    ];

    buff_to_buff_decompress_helper(input, 1024, true)
}

#[test]
fn buff_to_buff_decompress_fast_randomized() {
    let input = include_bytes!("../../tests/input/randomized-blocks.bin");

    buff_to_buff_decompress_helper(input, 256 * 1024, false)
}

#[test]
fn buff_to_buff_decompress_small_randomized() {
    let input = include_bytes!("../../tests/input/randomized-blocks.bin");

    buff_to_buff_decompress_helper(input, 256 * 1024, true)
}

#[test]
fn decompress_sample1() {
    assert_eq_decompress!("../../tests/input/quick/sample1.bz2");
}

#[test]
fn decompress_sample2() {
    assert_eq_decompress!("../../tests/input/quick/sample2.bz2");
}

#[test]
fn decompress_sample3() {
    assert_eq_decompress!("../../tests/input/quick/sample3.bz2");
}

#[test]
fn compress_sample1() {
    assert_eq_compress!("../../tests/input/quick/sample1.bz2");
}

#[test]
fn compress_sample2() {
    assert_eq_compress!("../../tests/input/quick/sample2.bz2");
}

#[test]
fn compress_sample3() {
    assert_eq_compress!("../../tests/input/quick/sample3.bz2");
}

pub fn decompress_c<'a>(
    dest: *mut u8,
    dest_len: *mut libc::c_uint,
    source: *const u8,
    source_len: libc::c_uint,
) -> i32 {
    use bzip2_sys::*;

    pub unsafe fn BZ2_bzBuffToBuffDecompress(
        dest: *mut libc::c_char,
        dest_len: *mut libc::c_uint,
        source: *mut libc::c_char,
        source_len: libc::c_uint,
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
            || dest_len.is_null()
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
        strm.avail_in = source_len;
        strm.avail_out = *dest_len;
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
            *dest_len = (*dest_len).wrapping_sub(strm.avail_out);
            BZ2_bzDecompressEnd(&mut strm);
            return 0 as libc::c_int;
        }
    }

    unsafe {
        BZ2_bzBuffToBuffDecompress(
            dest.cast::<core::ffi::c_char>(),
            dest_len,
            source as *mut _,
            source_len,
            0,
            0,
        )
    }
}

pub fn decompress_rs<'a>(
    dest: *mut u8,
    dest_len: *mut libc::c_uint,
    source: *const u8,
    source_len: libc::c_uint,
) -> i32 {
    use libbzip2_rs_sys::bzlib::*;

    pub unsafe fn BZ2_bzBuffToBuffDecompress(
        dest: *mut libc::c_char,
        dest_len: *mut libc::c_uint,
        source: *mut libc::c_char,
        source_len: libc::c_uint,
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
            || dest_len.is_null()
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
        strm.avail_in = source_len;
        strm.avail_out = *dest_len;
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
            *dest_len = (*dest_len).wrapping_sub(strm.avail_out);
            BZ2_bzDecompressEnd(&mut strm);
            return 0 as libc::c_int;
        }
    }

    unsafe {
        BZ2_bzBuffToBuffDecompress(
            dest.cast::<core::ffi::c_char>(),
            dest_len,
            source as *mut _,
            source_len,
            0,
            0,
        )
    }
}

pub fn compress_c<'a>(
    dest: *mut u8,
    dest_len: *mut libc::c_uint,
    source: *const u8,
    source_len: libc::c_uint,
) -> i32 {
    use bzip2_sys::*;
    pub unsafe fn BZ2_bzBuffToBuffCompress(
        dest: *mut libc::c_char,
        dest_len: *mut libc::c_uint,
        source: *mut libc::c_char,
        source_len: libc::c_uint,
        block_size_100k: libc::c_int,
        verbosity: libc::c_int,
        mut work_factor: libc::c_int,
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
            || dest_len.is_null()
            || source.is_null()
            || block_size_100k < 1 as libc::c_int
            || block_size_100k > 9 as libc::c_int
            || verbosity < 0 as libc::c_int
            || verbosity > 4 as libc::c_int
            || work_factor < 0 as libc::c_int
            || work_factor > 250 as libc::c_int
        {
            return -2 as libc::c_int;
        }
        if work_factor == 0 as libc::c_int {
            work_factor = 30 as libc::c_int;
        }
        strm.bzalloc = None;
        strm.bzfree = None;
        strm.opaque = std::ptr::null_mut::<libc::c_void>();
        ret = BZ2_bzCompressInit(&mut strm, block_size_100k, verbosity, work_factor);
        if ret != 0 as libc::c_int {
            return ret;
        }
        strm.next_in = source;
        strm.next_out = dest;
        strm.avail_in = source_len;
        strm.avail_out = *dest_len;
        ret = BZ2_bzCompress(&mut strm, 2 as libc::c_int);
        if ret == 3 as libc::c_int {
            BZ2_bzCompressEnd(&mut strm);
            -8 as libc::c_int
        } else if ret != 4 as libc::c_int {
            BZ2_bzCompressEnd(&mut strm);
            return ret;
        } else {
            *dest_len = (*dest_len).wrapping_sub(strm.avail_out);
            BZ2_bzCompressEnd(&mut strm);
            return 0 as libc::c_int;
        }
    }

    let verbosity = 0;
    let block_size_100k = 9;
    let work_factor = 30;

    unsafe {
        BZ2_bzBuffToBuffCompress(
            dest.cast::<core::ffi::c_char>(),
            dest_len,
            source as *mut _,
            source_len,
            block_size_100k,
            verbosity,
            work_factor,
        )
    }
}

pub fn compress_rs<'a>(
    dest: *mut u8,
    dest_len: *mut libc::c_uint,
    source: *const u8,
    source_len: libc::c_uint,
) -> i32 {
    use libbzip2_rs_sys::bzlib::*;

    pub unsafe fn BZ2_bzBuffToBuffCompress(
        dest: *mut libc::c_char,
        dest_len: *mut libc::c_uint,
        source: *mut libc::c_char,
        source_len: libc::c_uint,
        block_size_100k: libc::c_int,
        verbosity: libc::c_int,
        mut work_factor: libc::c_int,
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
            || dest_len.is_null()
            || source.is_null()
            || block_size_100k < 1 as libc::c_int
            || block_size_100k > 9 as libc::c_int
            || verbosity < 0 as libc::c_int
            || verbosity > 4 as libc::c_int
            || work_factor < 0 as libc::c_int
            || work_factor > 250 as libc::c_int
        {
            return -2 as libc::c_int;
        }
        if work_factor == 0 as libc::c_int {
            work_factor = 30 as libc::c_int;
        }
        strm.bzalloc = None;
        strm.bzfree = None;
        strm.opaque = std::ptr::null_mut::<libc::c_void>();
        ret = BZ2_bzCompressInit(&mut strm, block_size_100k, verbosity, work_factor);
        if ret != 0 as libc::c_int {
            return ret;
        }
        strm.next_in = source;
        strm.next_out = dest;
        strm.avail_in = source_len;
        strm.avail_out = *dest_len;
        ret = BZ2_bzCompress(&mut strm, 2 as libc::c_int);
        if ret == 3 as libc::c_int {
            BZ2_bzCompressEnd(&mut strm);
            -8 as libc::c_int
        } else if ret != 4 as libc::c_int {
            BZ2_bzCompressEnd(&mut strm);
            return ret;
        } else {
            *dest_len = (*dest_len).wrapping_sub(strm.avail_out);
            BZ2_bzCompressEnd(&mut strm);
            return 0 as libc::c_int;
        }
    }

    let verbosity = 0;
    let block_size_100k = 9;
    let work_factor = 30;

    unsafe {
        BZ2_bzBuffToBuffCompress(
            dest.cast::<core::ffi::c_char>(),
            dest_len,
            source as *mut _,
            source_len,
            block_size_100k,
            verbosity,
            work_factor,
        )
    }
}

#[rustfmt::skip]
mod bzip2_testfiles {
    #![allow(non_snake_case, unused_imports)]

    use super::*;

    #[test] fn pyflate_765B() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/pyflate/765B.bz2"); }
    #[test] fn pyflate_45MB_00() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/pyflate/45MB-00.bz2"); }
    #[test] fn pyflate_hello_world() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/pyflate/hello-world.bz2"); }
    #[test] fn pyflate_510B() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/pyflate/510B.bz2"); }
    #[test] fn pyflate_aaa() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/pyflate/aaa.bz2"); }
    #[test] fn pyflate_empty() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/pyflate/empty.bz2"); }
    #[test] fn pyflate_45MB_fb() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/pyflate/45MB-fb.bz2"); }
    #[test] fn commons_compress_bla_xml() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/commons-compress/bla.xml.bz2"); }
    #[test] fn commons_compress_bla_tar() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/commons-compress/bla.tar.bz2"); }
    #[test] fn commons_compress_bla_txt() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/commons-compress/bla.txt.bz2"); }
    #[test] fn commons_compress_COMPRESS_131() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/commons-compress/COMPRESS-131.bz2"); }
    #[test] fn commons_compress_multiple() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/commons-compress/multiple.bz2"); }
    #[test] fn commons_compress_zip64support() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/commons-compress/zip64support.tar.bz2"); }
    #[test] fn go_compress_e() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/go/compress/e.txt.bz2"); }
    #[test] fn go_compress_Isaac() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/go/compress/Isaac.Newton-Opticks.txt.bz2"); }
    #[test] fn go_compress_pass_sawtooth() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/go/compress/pass-sawtooth.bz2"); }
    #[test] fn go_compress_pass_random1() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/go/compress/pass-random1.bz2"); }
    #[test] fn go_compress_pass_random2() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/go/compress/pass-random2.bz2"); }
    #[test] fn go_compress_random() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/go/compress/random.data.bz2"); }
    #[test] fn go_regexp_re2_exhaustive() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/go/regexp/re2-exhaustive.txt.bz2"); }
    #[test] fn go_crypto_pss_vect() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/go/crypto/pss-vect.txt.bz2"); }
    #[test] fn go_crypto_SigVer() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/go/crypto/SigVer.rsp.bz2"); }
    #[test] fn lbzip2_incomp_1() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/incomp-1.bz2"); }
    #[test] fn lbzip2_trash() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/trash.bz2"); }
    #[test] fn lbzip2_incomp_2() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/incomp-2.bz2"); }
    #[test] fn lbzip2_fib() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/fib.bz2"); }
    #[test] fn lbzip2_ch255() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/ch255.bz2"); }
    #[test] fn lbzip2_32767() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/32767.bz2"); }
    #[test] fn lbzip2_empty() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/empty.bz2"); }
    #[test] fn lbzip2_concat() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/concat.bz2"); }
    #[test] fn lbzip2_idx899999() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/idx899999.bz2"); }
    #[test] fn lbzip2_repet() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/repet.bz2"); }
    #[test] fn lbzip2_codelen20() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/codelen20.bz2"); }
    #[test] fn lbzip2_gap() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/gap.bz2"); }
    #[test] fn lbzip2_rand() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/lbzip2/rand.bz2"); }
    #[test] fn dotnetzip_sample1() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/dotnetzip/sample1.bz2"); }
    #[test] fn dotnetzip_sample2() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/dotnetzip/sample2.bz2"); }
    #[test] fn dotnetzip_dancing_color() { assert_eq_decompress!("../../tests/input/bzip2-testfiles/dotnetzip/dancing-color.ps.bz2"); }
}

#[test]
fn high_level_read() {
    use libbzip2_rs_sys::bzlib::*;

    let p = std::env::current_dir().unwrap();

    let input = std::fs::read(p.join("../tests/input/quick/sample1.bz2")).unwrap();
    let mut expected = vec![0u8; 256 * 1024];
    let mut expected_len = expected.len() as _;
    let err = decompress_c(
        expected.as_mut_ptr(),
        &mut expected_len,
        input.as_ptr(),
        input.len() as _,
    );
    assert_eq!(err, 0);

    let p = p.join("../tests/input/quick/sample1.bz2\0");
    let input_file = unsafe {
        libc::fopen(
            p.display().to_string().as_mut_ptr().cast::<c_char>(),
            b"rb\0".as_ptr().cast::<c_char>(),
        )
    };

    assert!(!input_file.is_null());

    let mut bzerror = 0;
    let bz_file =
        unsafe { BZ2_bzReadOpen(&mut bzerror, input_file, 0, 0, core::ptr::null_mut(), 0) };

    let mut output = Vec::<u8>::with_capacity(1024);

    const BUFFER_SIZE: usize = 1024;
    let mut buffer = [0u8; BUFFER_SIZE];
    while bzerror == 0 {
        let bytes_read = unsafe {
            BZ2_bzRead(
                &mut bzerror,
                bz_file,
                buffer.as_mut_ptr().cast(),
                BUFFER_SIZE as i32,
            )
        };

        if bzerror == bzip2_sys::BZ_OK || bzerror == bzip2_sys::BZ_STREAM_END {
            output.extend(&buffer[..bytes_read as usize]);
        }
    }

    // make sure to clean up resources even if there was an error
    let after_read = bzerror;

    unsafe { BZ2_bzReadClose(&mut bzerror, bz_file) };

    unsafe { libc::fclose(input_file) };

    assert_eq!(after_read, bzip2_sys::BZ_STREAM_END);

    assert_eq!(bzerror, bzip2_sys::BZ_OK);

    assert_eq!(&expected[..expected_len as usize], output);
}

#[test]
fn high_level_write() {
    use libbzip2_rs_sys::bzlib::*;

    let block_size = 9; // Maximum compression level (1-9)
    let verbosity = 0; // Quiet operation
    let work_factor = 30; // Recommended default value

    let p = std::env::temp_dir().join("high_level_write.bz2");

    let _ = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&p)
        .unwrap();

    let output_file = unsafe {
        let p = p.with_extension("bz2\0");
        libc::fopen(
            p.display().to_string().as_mut_ptr().cast::<c_char>(),
            b"wb\0".as_ptr().cast::<c_char>(),
        )
    };

    assert!(!output_file.is_null());

    let mut bzerror = 0;
    let bz_file = unsafe {
        BZ2_bzWriteOpen(
            &mut bzerror,
            output_file,
            block_size,
            verbosity,
            work_factor,
        )
    };

    for chunk in SAMPLE1_BZ2.chunks(1024) {
        unsafe {
            BZ2_bzWrite(
                &mut bzerror,
                bz_file,
                chunk.as_ptr().cast_mut().cast(),
                chunk.len() as _,
            )
        };
        assert_eq!(bzerror, 0);
    }

    unsafe {
        BZ2_bzWriteClose(
            &mut bzerror,
            bz_file,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )
    };

    unsafe { libc::fclose(output_file) };

    assert_eq!(bzerror, bzip2_sys::BZ_OK);

    let mut expected = vec![0u8; 256 * 1024];
    let mut expected_len = expected.len() as _;
    let err = compress_c(
        expected.as_mut_ptr(),
        &mut expected_len,
        SAMPLE1_BZ2.as_ptr(),
        SAMPLE1_BZ2.len() as _,
    );
    assert_eq!(err, 0);

    assert_eq!(
        std::fs::read(p).unwrap(),
        &expected[..expected_len as usize]
    );
}

#[test]
fn error_messages() {
    use libbzip2_rs_sys::bzlib::{bzFile, bz_stream, BZ2_bzerror};

    let mut bz_file = bzFile {
        handle: std::ptr::null_mut(),
        buf: [0; 5000],
        bufN: 0,
        writing: 0,
        strm: bz_stream::zeroed(),
        lastErr: 0,
        initialisedOk: 0,
    };

    for i in -17..1 {
        bz_file.lastErr = i;

        let mut errnum = 0;
        let ptr = unsafe { BZ2_bzerror(&bz_file as *const _ as *const c_void, &mut errnum) };
        assert!(!ptr.is_null());
        let cstr = unsafe { CStr::from_ptr(ptr) };

        match i {
            1 => assert_eq!(cstr.to_str(), Ok("OK")),
            0 => assert_eq!(cstr.to_str(), Ok("OK")),
            -1 => assert_eq!(cstr.to_str(), Ok("SEQUENCE_ERROR")),
            -2 => assert_eq!(cstr.to_str(), Ok("PARAM_ERROR")),
            -3 => assert_eq!(cstr.to_str(), Ok("MEM_ERROR")),
            -4 => assert_eq!(cstr.to_str(), Ok("DATA_ERROR")),
            -5 => assert_eq!(cstr.to_str(), Ok("DATA_ERROR_MAGIC")),
            -6 => assert_eq!(cstr.to_str(), Ok("IO_ERROR")),
            -7 => assert_eq!(cstr.to_str(), Ok("UNEXPECTED_EOF")),
            -8 => assert_eq!(cstr.to_str(), Ok("OUTBUFF_FULL")),
            -9 => assert_eq!(cstr.to_str(), Ok("CONFIG_ERROR")),
            _ => assert_eq!(cstr.to_str(), Ok("???")),
        }

        assert_eq!(i, errnum);
    }
}
