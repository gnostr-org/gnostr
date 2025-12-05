#![cfg(feature = "coverage")]

use std::process::Command;
use std::str;

#[test]
fn test_code_coverage_script() {
    // Execute the coverage script
    let output = Command::new("bash")
        .arg("./coverage.sh")
        .output()
        .expect("Failed to execute coverage script");

    // Print stdout and stderr for debugging
    println!("status: {}", output.status);
    println!("stdout: {}", str::from_utf8(&output.stdout).unwrap());
    eprintln!("stderr: {}", str::from_utf8(&output.stderr).unwrap());

    // Assert that the script executed successfully
    assert!(output.status.success(), "Coverage script failed to execute");

    // Assert that the coverage directory was created
    assert!(
        std::fs::metadata("./coverage").is_ok(),
        "Coverage directory was not created"
    );

    // Assert that the main HTML report was generated
    assert!(
        std::fs::metadata("./coverage/index.html").is_ok(),
        "Main HTML report was not generated"
    );

    // Assert that the report is not empty (has a reasonable file size)
    let metadata = std::fs::metadata("./coverage/index.html").unwrap();
    assert!(
        metadata.len() > 100,
        "HTML report seems to be empty or incomplete"
    );
}
