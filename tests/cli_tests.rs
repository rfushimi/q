use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("q").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("CLI tool for querying LLMs"))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("q").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("q").unwrap();
    cmd.arg("--invalid-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn test_set_key_command() {
    let mut cmd = Command::cargo_bin("q").unwrap();
    cmd.args(["set-key", "openai", "sk-test1234567890abcdefghijklmnopqrstuvwxyz"])
        .assert()
        .success()
        .stdout(predicate::str::contains("API key for openai has been set successfully"));
}

#[test]
fn test_direct_prompt() {
    let mut cmd = Command::cargo_bin("q").unwrap();
    cmd.arg("test prompt")
        .assert()
        .success()
        .stdout(predicate::str::contains("Query handling will be implemented"));
}
