#[cfg(test)]
mod tests {
    use super::*;
    use assert_cmd::Command;
    use std::process::Command as StdCommand;

    #[test]
    fn test_gnostr_sha256_basic() {
        let input_string = "hello world";
        let expected_hash = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";

        let mut cmd = Command::cargo_bin("gnostr").unwrap();
        cmd.arg("--hash").arg(input_string);

        let output = cmd.output().expect("Failed to execute gnostr --hash command");

        assert!(output.status.success(), "Command failed with status: {}", output.status);
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), expected_hash);
    }

    #[test]
    fn test_gnostr_sha256_empty_string() {
        let input_string = "";
        let expected_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        let mut cmd = Command::cargo_bin("gnostr").unwrap();
        cmd.arg("--hash").arg(input_string);

        let output = cmd.output().expect("Failed to execute gnostr --hash command");

        assert!(output.status.success(), "Command failed with status: {}", output.status);
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), expected_hash);
    }
}
