#![cfg(feature = "coverage")]

use std::process::{Command, Stdio};
use std::str;
use std::env;
use std::path::Path;

/// Checks if a command is available in the system's PATH.
fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_or(false, |status| status.success())
}

/// A helper function to run a command and assert its success.
fn run_command(command: &mut Command) {
    println!("Running command: {:?}", command);
    let output = command.output().expect("Failed to execute command");

    println!("status: {}", output.status);
    println!("stdout: {}", str::from_utf8(&output.stdout).unwrap());
    eprintln!("stderr: {}", str::from_utf8(&output.stderr).unwrap());

    assert!(output.status.success());
}

#[test]
fn test_generate_code_coverage() {
    // 1. Check for llvm-tools-preview, or install it
    if !command_exists("llvm-profdata") || !command_exists("llvm-cov") {
        println!("llvm-tools-preview not found, installing...");
        run_command(Command::new("rustup").args(&["component", "add", "llvm-tools-preview"]));
    }

    // 2. Clean previous build artifacts
    println!("Cleaning project...");
    run_command(Command::new("cargo").arg("clean"));

    // 3. Build the project with coverage instrumentation
    println!("Building with coverage instrumentation...");
    let mut build_cmd = Command::new("cargo");
    build_cmd.env("RUSTFLAGS", "-C instrument-coverage");
    build_cmd.arg("build");
    run_command(&mut build_cmd);

    // 4. Run tests to generate coverage data
    println!("Running tests...");
    let mut test_cmd = Command::new("cargo");
    test_cmd.env("RUSTFLAGS", "-C instrument-coverage");
    test_cmd.arg("test");
    run_command(&mut test_cmd);

    // 5. Find the project name from Cargo.toml
    let cargo_toml: toml::Value = toml::from_str(
        &std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml")
    ).expect("Failed to parse Cargo.toml");

    let project_name = cargo_toml["package"]["name"].as_str().unwrap().replace("-", "_");

    // 6. Merge coverage data
    println!("Merging coverage data...");
    run_command(
        Command::new("llvm-profdata")
            .args(&["merge", "-sparse", "default.profraw", "-o", "default.profdata"])
    );

    // 7. Generate HTML report
    println!("Generating HTML report...");
    let binary_path = format!("./target/debug/{}", project_name);
    let mut objects: Vec<String> = vec!["-object".to_string(), binary_path];

    // Add dependencies to the object list
    let deps_path = Path::new("./target/debug/deps");
    if deps_path.exists() {
        for entry in std::fs::read_dir(deps_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.starts_with(&project_name) && !filename.ends_with(".d") {
                    objects.push("-object".to_string());
                    objects.push(path.to_str().unwrap().to_string());
                }
            }
        }
    }

    let mut html_report_cmd = Command::new("llvm-cov");
    html_report_cmd.arg("show")
        .args(&objects)
        .arg("--instr-profile=default.profdata")
        .arg("--format=html")
        .arg("--output-dir=coverage")
        .arg("--show-line-counts-or-regions")
        .arg("--show-instantiations")
        .arg("--show-missing-regions");

    if cfg!(target_os = "linux") {
        html_report_cmd.arg("--Xdemangler=rustfilt");
    }

    run_command(&mut html_report_cmd);

    // 8. Generate terminal summary
    println!("Generating terminal summary...");
    let mut summary_cmd = Command::new("llvm-cov");
    summary_cmd.arg("report")
        .args(&objects)
        .arg("--instr-profile=default.profdata");

    run_command(&mut summary_cmd);


    // 9. Assertions
    assert!(Path::new("./coverage").exists(), "Coverage directory not created.");
    assert!(Path::new("./coverage/index.html").exists(), "HTML report not generated.");
}
