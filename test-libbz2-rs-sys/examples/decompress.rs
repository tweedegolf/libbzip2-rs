use core::ffi::c_uint;

use test_libbz2_rs_sys::{decompress_c, decompress_rs};

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
