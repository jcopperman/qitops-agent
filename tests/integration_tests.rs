use std::process::Command;
use std::fs;
use std::path::Path;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(output.status.success());
    assert!(stdout.contains("QitOps Agent"));
    assert!(stdout.contains("USAGE:"));
    assert!(stdout.contains("OPTIONS:"));
    assert!(stdout.contains("SUBCOMMANDS:"));
}

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "version"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(output.status.success());
    assert!(stdout.contains("QitOps Agent v"));
}

#[test]
fn test_cli_llm_list() {
    let output = Command::new("cargo")
        .args(["run", "--", "llm", "list"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(output.status.success());
    assert!(stdout.contains("LLM Management") || stdout.contains("No LLM providers"));
}

#[test]
fn test_test_gen_invalid_path() {
    let output = Command::new("cargo")
        .args(["run", "--", "run", "test-gen", "--path", "nonexistent_file.rs"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!output.status.success());
    assert!(stderr.contains("File not found") || stderr.contains("path"));
}

#[test]
fn test_test_gen_valid_path() {
    // Create a temporary test file
    let test_dir = Path::new("test_files");
    if !test_dir.exists() {
        fs::create_dir_all(test_dir).unwrap();
    }
    
    let test_file = test_dir.join("temp_test.rs");
    fs::write(&test_file, "fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
    
    let output = Command::new("cargo")
        .args(["run", "--", "run", "test-gen", "--path", test_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Clean up
    fs::remove_file(test_file).unwrap();
    
    assert!(output.status.success() || stdout.contains("LLM providers"));
}

#[test]
fn test_pr_analyze_invalid_input() {
    let output = Command::new("cargo")
        .args(["run", "--", "run", "pr-analyze", "--pr", ""])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!output.status.success());
    assert!(stderr.contains("PR") || stderr.contains("empty"));
}

#[test]
fn test_risk_invalid_input() {
    let output = Command::new("cargo")
        .args(["run", "--", "run", "risk", "--diff", ""])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!output.status.success());
    assert!(stderr.contains("diff") || stderr.contains("empty"));
}

#[test]
fn test_test_data_invalid_input() {
    let output = Command::new("cargo")
        .args(["run", "--", "run", "test-data", "--schema", "", "--count", "0"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!output.status.success());
    assert!(stderr.contains("schema") || stderr.contains("count"));
}
