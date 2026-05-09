#[cfg(test)]
mod tests {
    use std::{env, net::TcpListener, process::Command};

    use serial_test::serial;

    use crate::utils::find_available_port;

    #[test]
    #[serial]
    fn test_main_error_on_port_conflict() {
        let temp_dir = tempfile::tempdir().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        let port = find_available_port();
        // Occupy the port
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

        let binary_path = env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("gnostr-server");

        let output = Command::new(binary_path)
            .args(["--bind", &format!("127.0.0.1:{port}")])
            .output()
            .expect("failed to execute process");

        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Stderr: {}", stderr);

        assert!(!output.status.success());
        assert!(stderr.to_lowercase().contains("in use"));

        drop(listener);
    }
}
