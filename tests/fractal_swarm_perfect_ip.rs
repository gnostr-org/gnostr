use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn example_command() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("run")
        .arg("--quiet")
        .arg("-p")
        .arg("gnostr-p2p")
        .arg("--example")
        .arg("fractal_swarm_perfect_ip")
        .arg("--");
    cmd
}

#[test]
fn help_prints_usage() {
    let mut cmd = example_command();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: fractal_swarm_perfect_ip"))
        .stdout(predicate::str::contains("--recursive PATH"))
        .stdout(predicate::str::contains("--depth N"));
}

#[test]
fn depth_requires_recursive() {
    let mut cmd = example_command();
    cmd.arg("--depth").arg("1");

    cmd.assert().failure().stderr(predicate::str::contains(
        "--depth is only valid with --recursive",
    ));
}

#[test]
fn file_and_recursive_are_mutually_exclusive() {
    let dir = tempdir().expect("create tempdir");
    let file_path = dir.path().join("input.bin");
    fs::write(&file_path, b"abc").expect("write input");

    let mut cmd = example_command();
    cmd.arg("--file")
        .arg(&file_path)
        .arg("--recursive")
        .arg(dir.path());

    cmd.assert().failure().stderr(predicate::str::contains(
        "--file and --recursive are mutually exclusive",
    ));
}

#[test]
fn file_mode_round_trips_bytes() {
    let dir = tempdir().expect("create tempdir");
    let input_path = dir.path().join("input.bin");
    let output_path = dir.path().join("output.bin");
    let bytes = b"perfect-ip-round-trip\x00\x01\x02\n";
    fs::write(&input_path, bytes).expect("write input file");

    let mut cmd = example_command();
    cmd.arg("--file")
        .arg(&input_path)
        .arg("--out")
        .arg(&output_path);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("sha256 match: true"));

    let output = fs::read(&output_path).expect("read output file");
    assert_eq!(output, bytes);
}

#[test]
fn recursive_depth_limits_traversal() {
    let dir = tempdir().expect("create tempdir");
    let root = dir.path().join("root");
    fs::create_dir_all(root.join("sub")).expect("create dir structure");
    fs::write(root.join("top.txt"), b"top").expect("write top file");
    fs::write(root.join("sub").join("nested.txt"), b"nested").expect("write nested file");

    let mut depth_zero = example_command();
    depth_zero
        .arg("--recursive")
        .arg(&root)
        .arg("--depth")
        .arg("0");

    depth_zero
        .assert()
        .success()
        .stdout(predicate::str::contains("."))
        .stdout(predicate::str::contains("top.txt").not())
        .stdout(predicate::str::contains("sub/").not());

    let mut depth_one = example_command();
    depth_one
        .arg("--recursive")
        .arg(&root)
        .arg("--depth")
        .arg("1");

    depth_one
        .assert()
        .success()
        .stdout(predicate::str::contains("top.txt"))
        .stdout(predicate::str::contains("sub/"))
        .stdout(predicate::str::contains("nested.txt").not());
}
