use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_test_env(verbose: bool) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let config_home = temp_dir.path().canonicalize().unwrap();
    
    if verbose {
        eprintln!("Debug: Setting up test environment");
        eprintln!("Debug: Temp dir: {:?}", temp_dir.path());
        eprintln!("Debug: Canonicalized path: {:?}", config_home);
    }
    
    // Create the config directory structure
    let config_dir = config_home.join("q");
    if verbose {
        eprintln!("Debug: Creating config dir at: {:?}", config_dir);
    }
    fs::create_dir_all(&config_dir).unwrap();
    
    temp_dir
}

fn create_command(temp_dir: &TempDir, verbose: bool) -> Command {
    let config_home = temp_dir.path().canonicalize().unwrap();
    let mut cmd = Command::cargo_bin("q").unwrap();
    cmd.env("XDG_CONFIG_HOME", config_home.to_str().unwrap());
    if verbose {
        cmd.arg("--verbose");
    }
    cmd
}

#[test]
fn test_set_key_openai() {
    let verbose = true; // Enable verbose for this test
    let temp_dir = setup_test_env(verbose);
    let config_home = temp_dir.path().canonicalize().unwrap();

    let mut cmd = create_command(&temp_dir, verbose);
    cmd.args(["set-key", "openai", "sk-test1234567890abcdefghijklmnopqrstuvwxyz"])
        .assert()
        .success()
        .stdout(predicate::str::contains("API key for openai has been set successfully"));

    // Verify config file was created and contains the key
    let config_path = config_home.join("q/config.toml");
    if verbose {
        eprintln!("Debug: Checking config file at: {:?}", config_path);
    }
    assert!(config_path.exists(), "Config file does not exist at {:?}", config_path);
    let config_content = fs::read_to_string(&config_path).unwrap();
    if verbose {
        eprintln!("Debug: Config content:\n{}", config_content);
    }
    assert!(config_content.contains("sk-test1234567890abcdefghijklmnopqrstuvwxyz"));
}

#[test]
fn test_set_key_gemini() {
    let verbose = true; // Enable verbose for this test
    let temp_dir = setup_test_env(verbose);
    let config_home = temp_dir.path().canonicalize().unwrap();

    let mut cmd = create_command(&temp_dir, verbose);
    cmd.args(["set-key", "gemini", "test1234567890abcdefghijklmnopqrstuvwxyz"])
        .assert()
        .success()
        .stdout(predicate::str::contains("API key for gemini has been set successfully"));

    // Verify config file was created and contains the key
    let config_path = config_home.join("q/config.toml");
    if verbose {
        eprintln!("Debug: Checking config file at: {:?}", config_path);
    }
    assert!(config_path.exists(), "Config file does not exist at {:?}", config_path);
    let config_content = fs::read_to_string(&config_path).unwrap();
    if verbose {
        eprintln!("Debug: Config content:\n{}", config_content);
    }
    assert!(config_content.contains("test1234567890abcdefghijklmnopqrstuvwxyz"));
}

#[test]
fn test_set_key_invalid_provider() {
    let verbose = false; // Disable verbose for error tests
    let temp_dir = setup_test_env(verbose);
    let mut cmd = create_command(&temp_dir, verbose);
    cmd.args(["set-key", "invalid", "test-key"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown provider"));
}

#[test]
fn test_set_key_invalid_openai_key() {
    let verbose = false; // Disable verbose for error tests
    let temp_dir = setup_test_env(verbose);
    let mut cmd = create_command(&temp_dir, verbose);
    cmd.args(["set-key", "openai", "invalid-key"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OpenAI API key must start with 'sk-'"));
}

#[test]
fn test_set_key_invalid_gemini_key() {
    let verbose = false; // Disable verbose for error tests
    let temp_dir = setup_test_env(verbose);
    let mut cmd = create_command(&temp_dir, verbose);
    cmd.args(["set-key", "gemini", "short"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Gemini API key is too short"));
}
