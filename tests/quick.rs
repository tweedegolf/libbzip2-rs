use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

/// Useful to test with the C binary
fn bzip2_binary() -> &'static str {
    env!("CARGO_BIN_EXE_bzip2")
}

fn command() -> Command {
    match env::var("RUNNER") {
        Ok(runner) if !runner.is_empty() => {
            let mut runner_args = runner.split(' ');
            let mut cmd = Command::new(runner_args.next().unwrap());
            cmd.args(runner_args);
            cmd.arg(bzip2_binary());

            cmd
        }
        _ => Command::new(bzip2_binary()),
    }
}

fn run_test(compressed: &str, expected: &[u8]) {
    let mut cmd = command();

    let output = match cmd
        .arg("-d")
        .arg(compressed)
        .arg("-c")
        .stdout(Stdio::piped())
        .output()
    {
        Ok(output) => output,
        Err(err) => panic!("Running {cmd:?} failed with {err:?}"),
    };
    assert!(
        output.status.success(),
        "status: {:?} stderr: {:?}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected);
}

#[test]
fn sample1() {
    run_test(
        "tests/input/quick/sample1.bz2",
        include_bytes!("input/quick/sample1.ref"),
    );
}

#[test]
fn sample2() {
    run_test(
        "tests/input/quick/sample2.bz2",
        include_bytes!("input/quick/sample2.ref"),
    );
}

#[test]
fn sample3() {
    run_test(
        "tests/input/quick/sample3.bz2",
        include_bytes!("input/quick/sample3.ref"),
    );
}

#[test]
fn uncompress_stdin_to_stdout_unexpected_eof() {
    use std::io::Write;

    let compressed = include_bytes!("input/quick/sample1.bz2");

    let mut cmd = command();

    // Set up command to read from stdin, decompress, and output to stdout
    let mut child = cmd
        .arg("-d")
        .arg("-c")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start child process");

    // Write the compressed data to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&compressed[..1024])
            .expect("Failed to write to stdin");
    }

    // Wait for the child process to finish and capture output
    let output = child.wait_with_output().expect("Failed to read stdout");

    assert!(
        !output.status.success(),
        "status: {:?} stderr: {:?}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stderr).replace(bzip2_binary(), "bzip2"),
        format!(concat!(
            "\n",
            "bzip2: Compressed file ends unexpectedly;\n",
            "	perhaps it is corrupted?  *Possible* reason follows.\n",
            "bzip2: Inappropriate ioctl for device\n",
            "	Input file = (stdin), output file = (stdout)\n",
            "\n",
            "It is possible that the compressed file(s) have become corrupted.\n",
            "You can use the -tvv option to test integrity of such files.\n",
            "\n",
            "You can use the `bzip2recover' program to attempt to recover\n",
            "data from undamaged sections of corrupted files.\n",
            "\n",
        )),
    );
}

#[test]
fn uncompress_stdin_to_stdout_crc_error_i2o() {
    use std::io::Write;

    let compressed = include_bytes!("input/quick/sample1.bz2");

    let mut cmd = command();

    // Set up command to read from stdin, decompress, and output to stdout
    let mut child = cmd
        .arg("-d")
        .arg("-c")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start child process");

    let (left, right) = compressed.split_at(1024);

    // Write the compressed data to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(left).unwrap();
        stdin.write_all(b"garbage").unwrap();
        stdin.write_all(right).unwrap();
    }

    // Wait for the child process to finish and capture output
    let output = child.wait_with_output().expect("Failed to read stdout");

    assert!(
        !output.status.success(),
        "status: {:?} stderr: {:?}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stderr).replace(bzip2_binary(), "bzip2"),
        format!(concat!(
            "\n",
            "bzip2: Data integrity error when decompressing.\n",
            "\tInput file = (stdin), output file = (stdout)\n",
            "\n",
            "It is possible that the compressed file(s) have become corrupted.\n",
            "You can use the -tvv option to test integrity of such files.\n",
            "\n",
            "You can use the `bzip2recover' program to attempt to recover\n",
            "data from undamaged sections of corrupted files.\n",
            "\n",
        )),
    );
}

#[test]
fn uncompress_stdin_to_stdout_crc_error_f2f() {
    use std::io::Write;

    let compressed = include_bytes!("input/quick/sample1.bz2");

    let tmpdir = tempfile::tempdir().unwrap();
    let sample1 = tmpdir.path().join("sample1.bz2");

    {
        let mut f = std::fs::File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&sample1)
            .unwrap();

        let (left, right) = compressed.split_at(1024);

        f.write_all(left).unwrap();
        f.write_all(b"garbage").unwrap();
        f.write_all(right).unwrap();
    }

    let mut cmd = command();

    cmd.arg("-d").arg(sample1);

    let output = cmd.output().expect("Failed to read stdout");

    assert!(
        !output.status.success(),
        "status: {:?} stderr: {:?}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stderr).replace(bzip2_binary(), "bzip2"),
        format!(
            concat!(
                "\n",
                "bzip2: Data integrity error when decompressing.\n",
                "\tInput file = {tmp_dir}/sample1.bz2, output file = {tmp_dir}/sample1\n",
                "\n",
                "It is possible that the compressed file(s) have become corrupted.\n",
                "You can use the -tvv option to test integrity of such files.\n",
                "\n",
                "You can use the `bzip2recover' program to attempt to recover\n",
                "data from undamaged sections of corrupted files.\n",
                "\n",
                "bzip2: Deleting output file {tmp_dir}/sample1, if it exists.\n",
            ),
            tmp_dir = tmpdir.path().display(),
        ),
    );
}

#[test]
fn test_comp_decomp_sample_ref1() {
    let sample = Path::new("tests/input/quick/sample1.ref");

    for block_size in ["-1", "-2", "-3"] {
        let mut cmd = command();
        cmd.arg("--compress")
            .arg(block_size)
            .arg("--keep")
            .arg("--stdout")
            .arg(sample)
            .stdout(Stdio::piped());

        let output = cmd.output().unwrap();

        assert!(output.status.success());

        let tmpdir = tempfile::tempdir().unwrap();

        let tempfile_path = tmpdir
            .path()
            .with_file_name(sample.file_name().unwrap())
            .with_extension("bz2");

        std::fs::write(&tempfile_path, output.stdout).unwrap();

        let mut cmd = command();
        cmd.arg("--decompress")
            .arg("--stdout")
            .arg(tempfile_path)
            .stdout(Stdio::piped());

        let output = cmd.output().unwrap();

        assert!(output.status.success());

        let out_hash = crc32fast::hash(&output.stdout);
        let ref_file = std::fs::read(sample).unwrap();
        let ref_hash = crc32fast::hash(&ref_file);

        assert_eq!(out_hash, ref_hash);
    }
}

#[test]
fn test_comp_decomp_sample_ref2() {
    let sample = Path::new("tests/input/quick/sample2.ref");

    for block_size in ["-1", "-2", "-3"] {
        let mut cmd = command();
        cmd.arg("--compress")
            .arg(block_size)
            .arg("--keep")
            .arg("--stdout")
            .arg(sample)
            .stdout(Stdio::piped());

        let output = cmd.output().unwrap();

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );

        // let tmpdir = tempfile::tempdir().unwrap();
        let tmpdir_path = Path::new("/tmp/foo");

        let tempfile_path = tmpdir_path
            .with_file_name(sample.file_name().unwrap())
            .with_extension("bz2");

        std::fs::write(&tempfile_path, output.stdout).unwrap();

        let mut cmd = command();
        cmd.arg("--decompress")
            .arg("--stdout")
            .arg(tempfile_path)
            .stdout(Stdio::piped());

        let output = cmd.output().unwrap();

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );

        let out_hash = crc32fast::hash(&output.stdout);
        let ref_file = std::fs::read(sample).unwrap();
        let ref_hash = crc32fast::hash(&ref_file);

        assert_eq!(out_hash, ref_hash);
    }
}

#[test]
fn compression_stderr_output() {
    let sample = Path::new("tests/input/quick/sample3.ref");

    let mut cmd = command();
    cmd.arg("--compress")
        .arg("-1")
        .arg("--keep")
        .arg("--stdout")
        .arg("-v")
        .arg(sample)
        .stdout(Stdio::piped());

    let output = cmd.output().unwrap();

    assert!(output.status.success());

    assert_eq!(
        String::from_utf8_lossy(&output.stderr).replace(bzip2_binary(), "bzip2"),
        format!(
            "  {in_file}: 440.454:1,  0.018 bits/byte, 99.77% saved, 120244 in, 273 out.\n",
            in_file = sample.display(),
        ),
    );

    let mut cmd = command();
    cmd.arg("--compress")
        .arg("-1")
        .arg("--keep")
        .arg("--stdout")
        .arg("-vv")
        .arg(sample)
        .stdout(Stdio::piped());

    let output = cmd.output().unwrap();

    assert!(output.status.success());

    assert_eq!(
        String::from_utf8_lossy(&output.stderr).replace(bzip2_binary(), "bzip2"),
        format!(
            concat!(
                "  {in_file}: \n",
                "    block 1: crc = 0xbcd1d34c, combined CRC = 0xbcd1d34c, size = 99981\n",
                "    too repetitive; using fallback sorting algorithm\n",
                "    block 2: crc = 0xabd59416, combined CRC = 0xd276328f, size = 20263\n",
                "    too repetitive; using fallback sorting algorithm\n",
                "    final combined CRC = 0xd276328f\n",
                "   440.454:1,  0.018 bits/byte, 99.77% saved, 120244 in, 273 out.\n",
            ),
            in_file = sample.display(),
        ),
    );

    let mut cmd = command();
    cmd.arg("--compress")
        .arg("-1")
        .arg("--keep")
        .arg("--stdout")
        .arg("-vvv")
        .arg(sample)
        .stdout(Stdio::piped());

    let output = cmd.output().unwrap();

    assert!(output.status.success());

    assert_eq!(
        String::from_utf8_lossy(&output.stderr).replace(bzip2_binary(), "bzip2"),
        format!(
            concat!(
                "  {in_file}: \n",
                "    block 1: crc = 0xbcd1d34c, combined CRC = 0xbcd1d34c, size = 99981\n",
                "      901380 work, 99981 block, ratio  9.02\n",
                "    too repetitive; using fallback sorting algorithm\n",
                "      99981 in block, 292 after MTF & 1-2 coding, 32+2 syms in use\n",
                "      initial group 3, [0 .. 2], has 114 syms (39.0%)\n",
                "      initial group 2, [3 .. 9], has 85 syms (29.1%)\n",
                "      initial group 1, [10 .. 33], has 93 syms (31.8%)\n",
                "      pass 1: size is 296, grp uses are 2 0 4 \n",
                "      pass 2: size is 155, grp uses are 2 0 4 \n",
                "      pass 3: size is 155, grp uses are 2 0 4 \n",
                "      pass 4: size is 155, grp uses are 2 0 4 \n",
                "      bytes: mapping 19, selectors 3, code lengths 30, codes 155\n",
                "    block 2: crc = 0xabd59416, combined CRC = 0xd276328f, size = 20263\n",
                "      182372 work, 20263 block, ratio  9.00\n",
                "    too repetitive; using fallback sorting algorithm\n",
                "      20263 in block, 54 after MTF & 1-2 coding, 4+2 syms in use\n",
                "      initial group 2, [0 .. 1], has 48 syms (88.9%)\n",
                "      initial group 1, [2 .. 5], has 6 syms (11.1%)\n",
                "      pass 1: size is 11, grp uses are 0 2 \n",
                "      pass 2: size is 12, grp uses are 0 2 \n",
                "      pass 3: size is 12, grp uses are 0 2 \n",
                "      pass 4: size is 12, grp uses are 0 2 \n",
                "      bytes: mapping 11, selectors 2, code lengths 5, codes 12\n",
                "    final combined CRC = 0xd276328f\n",
                "   440.454:1,  0.018 bits/byte, 99.77% saved, 120244 in, 273 out.\n",
            ),
            in_file = sample.display(),
        ),
    );
}

#[test]
fn test_comp_decomp_sample_ref3() {
    let sample = Path::new("tests/input/quick/sample3.ref");

    for block_size in ["-1", "-2", "-3"] {
        let mut cmd = command();
        cmd.arg("--compress")
            .arg(block_size)
            .arg("--keep")
            .arg("--stdout")
            .arg(sample)
            .stdout(Stdio::piped());

        let output = cmd.output().unwrap();

        assert!(output.status.success());

        let tmpdir = tempfile::tempdir().unwrap();

        let tempfile_path = tmpdir
            .path()
            .with_file_name(sample.file_name().unwrap())
            .with_extension("bz2");

        std::fs::write(&tempfile_path, output.stdout).unwrap();

        let mut cmd = command();
        cmd.arg("--decompress")
            .arg("--stdout")
            .arg(tempfile_path)
            .stdout(Stdio::piped());

        let output = cmd.output().unwrap();

        assert!(output.status.success());

        let out_hash = crc32fast::hash(&output.stdout);
        let ref_file = std::fs::read(sample).unwrap();
        let ref_hash = crc32fast::hash(&ref_file);

        assert_eq!(out_hash, ref_hash);
    }
}

#[test]
fn redundant_flag() {
    {
        let mut cmd = command();
        cmd.arg("--repetitive-best");
        let output = cmd.output().unwrap();

        assert!(output.status.success());

        assert_eq!(
            String::from_utf8_lossy(&output.stderr).replace(bzip2_binary(), "bzip2"),
            "bzip2: --repetitive-best is redundant in versions 0.9.5 and above\n"
        );
    }

    {
        let mut cmd = command();
        cmd.arg("--repetitive-fast");
        let output = cmd.output().unwrap();

        assert!(output.status.success());

        assert_eq!(
            String::from_utf8_lossy(&output.stderr).replace(bzip2_binary(), "bzip2"),
            "bzip2: --repetitive-fast is redundant in versions 0.9.5 and above\n"
        );
    }
}
