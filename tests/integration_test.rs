use std::process::Command;
use std::time::{Duration, Instant};

fn ogle_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ogle"))
}

#[test]
fn test_help() {
    let output = ogle_bin().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ogle"));
    assert!(stdout.contains("--period"));
    assert!(stdout.contains("--until-success"));
    assert!(stdout.contains("--until-failure"));
}

#[test]
fn test_version() {
    let output = ogle_bin().arg("--version").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_no_args_fails() {
    let output = ogle_bin().output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_true_succeeds() {
    let output = ogle_bin().args(["-z", "--", "true"]).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_false_fails() {
    let output = ogle_bin().args(["-e", "--", "false"]).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_command_output_appears() {
    let output = ogle_bin()
        .args(["-z", "--", "echo", "integration_test_marker"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("integration_test_marker"));
}

fn run_with_timeout(args: &[&str], timeout: Duration) -> bool {
    let mut child = ogle_bin()
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            return true;
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn test_until_success_keeps_running_on_failure() {
    let exited = run_with_timeout(
        &["-p", "1", "-z", "--", "/bin/sh", "-c", "exit 1"],
        Duration::from_secs(3),
    );
    assert!(
        !exited,
        "ogle should keep running when command fails with -z"
    );
}

#[test]
fn test_until_failure_keeps_running_on_success() {
    let exited = run_with_timeout(&["-p", "1", "-e", "--", "true"], Duration::from_secs(3));
    assert!(
        !exited,
        "ogle should keep running when command succeeds with -e"
    );
}
