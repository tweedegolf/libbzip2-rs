#![no_main]
use libbzip2_rs_sys::BZ_OK;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|source: Vec<u8>| {
    let mut dest_c = vec![0u8; 1 << 16];
    let mut dest_rs = vec![0u8; 1 << 16];

    let mut dest_len_c = dest_c.len() as _;
    let mut dest_len_rs = dest_rs.len() as _;

    let err_c = unsafe {
        test_libbzip2_rs_sys::decompress_c(
            dest_c.as_mut_ptr(),
            &mut dest_len_c,
            source.as_ptr(),
            source.len() as _,
        )
    };

    let err_rs = unsafe {
        test_libbzip2_rs_sys::decompress_rs(
            dest_rs.as_mut_ptr(),
            &mut dest_len_rs,
            source.as_ptr(),
            source.len() as _,
        )
    };

    assert_eq!(err_c, err_rs);

    if err_c == BZ_OK {
        dest_c.truncate(dest_len_c as usize);
        dest_rs.truncate(dest_len_rs as usize);

        assert_eq!(dest_c, dest_rs);
    }
});
