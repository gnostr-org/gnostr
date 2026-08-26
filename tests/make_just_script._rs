use anyhow::Result;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_makefile_generation() -> Result<()> {
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();

    // Copy the script to the temporary directory
    let script_path = temp_path.join("make_just.sh");
    fs::copy("make_just.sh", &script_path)?;

    // Make the script executable
    Command::new("chmod").arg("+x").arg(&script_path).status()?;

    // Run the script
    let output = Command::new(&script_path)
        .current_dir(temp_path)
        .output()?;

    // Assert script ran successfully
    assert!(output.status.success());

    let makefile_path = temp_path.join("Makefile");
    assert!(makefile_path.exists());

    let makefile_content = fs::read_to_string(&makefile_path)?;

    // Assert key sections and targets are present in Makefile
    assert!(makefile_content.contains("all:\tbin"));
    assert!(makefile_content.contains("bin:\t"));
    assert!(makefile_content.contains("cargo-test-workspace:\t"));
    assert!(makefile_content.contains("gnostr-chat:\t"));
    assert!(makefile_content.contains("export TAG"));
    assert!(makefile_content.contains("cargo b --manifest-path Cargo.toml"));

    Ok(())
}

#[test]
fn test_justfile_generation() -> Result<()> {
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();

    // Copy the script to the temporary directory
    let script_path = temp_path.join("make_just.sh");
    fs::copy("make_just.sh", &script_path)?;

    // Make the script executable
    Command::new("chmod").arg("+x").arg(&script_path).status()?;

    // Run the script
    let output = Command::new(&script_path)
        .current_dir(temp_path)
        .output()?;

    // Assert script ran successfully
    assert!(output.status.success());

    let justfile_path = temp_path.join(".justfile");
    assert!(justfile_path.exists());

    let justfile_content = fs::read_to_string(&justfile_path)?;

    // Assert default recipe is present
    assert!(justfile_content.contains("default:"));
    assert!(justfile_content.contains("  just --choose"));

    // Assert some Makefile targets have corresponding just recipes
    assert!(justfile_content.contains("all:"));
    assert!(justfile_content.contains("  @make all"));
    assert!(justfile_content.contains("bin:"));
    assert!(justfile_content.contains("  @make bin"));
    assert!(justfile_content.contains("cargo-test-workspace:"));
    assert!(justfile_content.contains("  @make cargo-test-workspace"));

    Ok(())
}
