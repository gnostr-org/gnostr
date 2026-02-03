use std::process::Command;

pub fn kill_gnostr_asyncgit_process() {
    #[cfg(unix)]
    {
        let output = Command::new("pkill")
            .arg("gnostr-asyncgit")
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    println!("Successfully killed gnostr-asyncgit process.");
                } else {
                    eprintln!("Failed to kill gnostr-asyncgit process: {}", String::from_utf8_lossy(&out.stderr));
                }
            },
            Err(e) => eprintln!("Could not run pkill command: {}", e),
        }
    }

    #[cfg(windows)]
    {
        println!("Killing gnostr-asyncgit process not implemented for Windows without external crates.");
        // For Windows, a more robust solution would involve `taskkill /IM gnostr-asyncgit.exe /F`
        // but would require parsing `tasklist` to ensure the process exists,
        // or handling errors gracefully if it doesn't.
        // This is left unimplemented due to the constraints of not adding external crates
        // and the complexity of reliable cross-platform process management from scratch.
    }
}
