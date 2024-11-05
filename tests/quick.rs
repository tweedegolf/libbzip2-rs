use std::env;
use std::process::{Command, Stdio};

fn run_test(compressed: &str, expected: &[u8]) {
    let mut cmd;
    match env::var("RUNNER") {
        Ok(runner) if !runner.is_empty() => {
            let mut runner_args = runner.split(' ');
            cmd = Command::new(runner_args.next().unwrap());
            cmd.args(runner_args);
            cmd.arg(env!("CARGO_BIN_EXE_bzip2"));
        }
        _ => cmd = Command::new(env!("CARGO_BIN_EXE_bzip2")),
    }
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
