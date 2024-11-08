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
