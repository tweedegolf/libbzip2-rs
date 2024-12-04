#![no_main]
use libbz2_rs_sys::BZ_OK;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: String| {
    let length = 8 * 1024;
    let mut deflated = vec![0; length as usize];
    let mut length = length as _;
    let error = unsafe {
        test_libbz2_rs_sys::compress_rs(
            deflated.as_mut_ptr().cast(),
            &mut length,
            data.as_ptr().cast(),
            data.len() as _,
        )
    };

    assert_eq!(error, BZ_OK);

    deflated.truncate(length as usize);

    let mut output = [0u8; 1 << 10];
    let mut output_len = output.len() as _;
    let error = unsafe {
        test_libbz2_rs_sys::decompress_rs(
            output.as_mut_ptr(),
            &mut output_len,
            deflated.as_ptr(),
            deflated.len() as _,
        )
    };
    assert_eq!(error, BZ_OK);
    let output = &output[..output_len as usize];

    if output != data.as_bytes() {
        let path = std::env::temp_dir().join("compressed.txt");
        std::fs::write(&path, &data).unwrap();
        eprintln!("saved input file to {path:?}");
    }

    assert_eq!(output, data.as_bytes());
});
