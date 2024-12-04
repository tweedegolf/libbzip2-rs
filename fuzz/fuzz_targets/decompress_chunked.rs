#![no_main]
use libbz2_rs_sys::{BZ_FINISH, BZ_OK, BZ_STREAM_END};
use libfuzzer_sys::fuzz_target;

fn compress_c(data: &[u8]) -> Vec<u8> {
    // first, deflate the data using the standard zlib
    let length = 8 * 1024;
    let mut deflated = vec![0u8; length as usize];

    let mut stream = bzip2_sys::bz_stream {
        next_in: data.as_ptr() as *mut _,
        avail_in: data.len() as _,
        total_in_lo32: 0,
        total_in_hi32: 0,
        next_out: deflated.as_mut_ptr() as *mut _,
        avail_out: deflated.len() as _,
        total_out_lo32: 0,
        total_out_hi32: 0,
        state: std::ptr::null_mut(),
        bzalloc: None,
        bzfree: None,
        opaque: std::ptr::null_mut(),
    };

    unsafe {
        let err = bzip2_sys::BZ2_bzCompressInit(&mut stream, 9, 0, 250);
        assert_eq!(err, BZ_OK);
    };

    let error = unsafe { bzip2_sys::BZ2_bzCompress(&mut stream, BZ_FINISH) };

    assert_eq!(error, BZ_STREAM_END);

    deflated.truncate(
        ((u64::from(stream.total_out_hi32) << 32) + u64::from(stream.total_out_lo32))
            .try_into()
            .unwrap(),
    );

    unsafe {
        let err = bzip2_sys::BZ2_bzCompressEnd(&mut stream);
        assert_eq!(err, BZ_OK);
    }

    deflated
}

fuzz_target!(|input: (String, usize)| {
    let (data, chunk_size) = input;

    if chunk_size == 0 {
        return;
    }

    let deflated = compress_c(data.as_bytes());

    let mut stream = libbz2_rs_sys::bz_stream::zeroed();

    unsafe {
        let err = libbz2_rs_sys::BZ2_bzDecompressInit(&mut stream, 0, 0);
        assert_eq!(err, BZ_OK);
    };

    let mut output = vec![0u8; 1 << 15];
    stream.next_out = output.as_mut_ptr() as *mut _;
    stream.avail_out = output.len() as _;

    for chunk in deflated.as_slice().chunks(chunk_size) {
        stream.next_in = chunk.as_ptr() as *mut _;
        stream.avail_in = chunk.len() as _;

        let err = unsafe { libbz2_rs_sys::BZ2_bzDecompress(&mut stream) };
        match err {
            BZ_OK => continue,
            BZ_STREAM_END => continue,
            _ => {
                panic!("{err}");
            }
        }
    }

    output.truncate(
        ((u64::from(stream.total_out_hi32) << 32) + u64::from(stream.total_out_lo32))
            .try_into()
            .unwrap(),
    );
    let output = String::from_utf8(output).unwrap();

    unsafe {
        let err = libbz2_rs_sys::BZ2_bzDecompressEnd(&mut stream);
        assert_eq!(err, BZ_OK);
    }

    if output != data {
        let path = std::env::temp_dir().join("deflate.txt");
        std::fs::write(&path, &data).unwrap();
        eprintln!("saved input file to {path:?}");
    }

    assert_eq!(output, data);
});
