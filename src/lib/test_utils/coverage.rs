//! # Code Coverage Generation
//!
//! This module provides a test function to generate code coverage reports for the project
//! using Rust's built-in source-based coverage tools (`-C instrument-coverage`).
//!
//! This entire module is conditionally compiled and is only available when the "coverage"
//! feature is enabled.
//!
//! ## Usage
//!
//! To generate a code coverage report, run the following command from the root of the project:
//!
//! ```bash
//! cargo test --features coverage -- --nocapture
//! ```
//!
//! - `--features coverage`: Enables this test module.
//! - `-- --nocapture`: Ensures that the output from the test (including command outputs and progress) is displayed in the console.
//!
//! Upon successful execution, a detailed HTML report will be available in the `coverage/`
//! directory, and a summary will be printed to the terminal.
//!
//! ## Quick Mode
//!
//! For faster runs when you haven't made significant changes, you can skip the `cargo clean`
//! step by passing the `--quick` flag:
//!
//! ```bash
//! cargo test --features coverage -- --nocapture --quick
//! ```
//!
//! ## Dependencies
//!
//! This script relies on `llvm-profdata` and `llvm-cov`, which are part of the `llvm-tools-preview`
//! rustup component. The test will attempt to install this component automatically if it's not
//! already present.

#![cfg(feature = "coverage")]

use std::process::{Command, Stdio};
use std::str;
use std::env;
use std::path::Path;

/// Checks if a command is available in the system's PATH by trying to run it
/// with the `--version` flag.
fn command_exists(command: &str) -> bool {
    println!("Checking for command: '{}'...", command);
    let result = Command::new(command)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_or(false, |status| status.success());

    if result {
        println!("'{}' found.", command);
    } else {
        println!("'{}' not found.", command);
    }
    result
}

/// A helper function to execute a shell command, print its output, and panic if it fails.
///
/// This provides visibility into the execution of external tools during the test run.
fn run_command(command: &mut Command) {
    println!("Running command: {:?}", command);
    let output = command.output().expect("Failed to execute command");

    println!("status: {}", output.status);
    println!("stdout: {}", str::from_utf8(&output.stdout).unwrap());
    eprintln!("stderr: {}", str::from_utf8(&output.stderr).unwrap());

    assert!(output.status.success(), "Command execution failed!");
}

/// The main test function that orchestrates the entire code coverage generation process.
///
/// This test is NOT a unit test for the application logic, but rather an integration test
/// for the code coverage workflow. It performs the following steps:
///
/// 1. Ensures `llvm-tools-preview` is installed.
/// 2. Cleans the project (can be skipped with `--quick`).
/// 3. Builds the project with coverage instrumentation.
/// 4. Runs tests to generate raw coverage data.
/// 5. Parses `Cargo.toml` to find the binary name.
/// 6. Merges the raw coverage data using `llvm-profdata`.
/// 7. Gathers all relevant binaries (the main crate and tests).
/// 8. Generates a detailed HTML report using `llvm-cov`.
/// 9. Prints a summary report to the console using `llvm-cov`.
/// 10. Asserts that the report was created successfully.
#[test]
fn test_generate_code_coverage() {
    // Check for a "--quick" command-line argument passed to the test binary.
    let args: Vec<String> = env::args().skip_while(|val| val != "--").skip(1).collect();
    let quick_mode = args.iter().any(|arg| arg == "--quick");

    // Step 1: Check for llvm-tools-preview, or install it if not present.
    // This is required for `llvm-profdata` and `llvm-cov`.
    if !command_exists("llvm-profdata") || !command_exists("llvm-cov") {
        println!("llvm-tools-preview not found, installing...");
        run_command(Command::new("rustup").args(&["component", "add", "llvm-tools-preview"]));
    }

    // Step 2: Clean previous build artifacts, unless in quick mode.
    if !quick_mode {
        println!("Cleaning project...");
        run_command(Command::new("cargo").arg("clean"));
    } else {
        println!("Skipping 'cargo clean' due to --quick flag.");
    }

    // Step 3: Build the project with coverage instrumentation enabled.
    // The `-C instrument-coverage` flag tells rustc to generate coverage data.
    println!("Building with coverage instrumentation...");
    let mut build_cmd = Command::new("cargo");
    build_cmd.env("RUSTFLAGS", "-C instrument-coverage");
    build_cmd.arg("build");
    run_command(&mut build_cmd);

    // Step 4: Run tests to generate the raw coverage data file (`default.profraw`).
    println!("Running tests...");
    let mut test_cmd = Command::new("cargo");
    test_cmd.env("RUSTFLAGS", "-C instrument-coverage");
    test_cmd.arg("test");
    run_command(&mut test_cmd);

    // Step 5: Find the project name from Cargo.toml to locate the binaries.
    let cargo_toml: toml::Value = toml::from_str(
        &std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml")
    ).expect("Failed to parse Cargo.toml");

    let project_name = cargo_toml["package"]["name"].as_str().unwrap().replace("-", "_");

    // Step 6: Merge the raw profile data into a single `.profdata` file.
    println!("Merging coverage data...");
    run_command(
        Command::new("llvm-profdata")
            .args(&["merge", "-sparse", "default.profraw", "-o", "default.profdata"])
    );

    // Step 7: Gather all relevant binaries to include in the report.
    // This includes the main library/binary and all test executables.
    println!("Gathering binaries for the report...");
    let binary_path = format!("./target/debug/{}", project_name);
    let mut objects: Vec<String> = vec!["-object".to_string(), binary_path];

    let deps_path = Path::new("./target/debug/deps");
    if deps_path.exists() {
        for entry in std::fs::read_dir(deps_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            // Filter for executable files related to the project.
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.starts_with(&project_name) && !filename.ends_with(".d") {
                    objects.push("-object".to_string());
                    objects.push(path.to_str().unwrap().to_string());
                }
            }
        }
    }

    // Step 8: Generate a detailed HTML report.
    println!("Generating HTML report...");
    let coverage_dir = env::current_dir().unwrap().join("coverage");
    let mut html_report_cmd = Command::new("llvm-cov");
    html_report_cmd.arg("show")
        .args(&objects)
        .arg("--instr-profile=default.profdata")
        .arg("--format=html")
        .arg("--output-dir").arg(&coverage_dir)
        .arg("--show-line-counts-or-regions")
        .arg("--show-instantiations")
        .arg("--show-missing-regions");

    // On Linux, `rustfilt` can demangle Rust symbols for a cleaner report.
    if cfg!(target_os = "linux") {
        html_report_cmd.arg("--Xdemangler=rustfilt");
    }

    run_command(&mut html_report_cmd);

    // Step 9: Generate and print a summary report to the terminal.
    println!("Generating terminal summary...");
    let mut summary_cmd = Command::new("llvm-cov");
    summary_cmd.arg("report")
        .args(&objects)
        .arg("--instr-profile=default.profdata");

    run_command(&mut summary_cmd);


    // Step 10: Final assertions to confirm the report was generated.
    assert!(coverage_dir.exists(), "Coverage directory was not created.");
    assert!(coverage_dir.join("index.html").exists(), "HTML report (index.html) was not generated.");
}
