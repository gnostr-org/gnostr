use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

fn main() {
    let email = env::args()
        .nth(1)
        .unwrap_or_else(|| "gnostr@gnostr.org".to_string());

    let home_dir = env::var("HOME").expect("HOME environment variable not set");
    let ssh_dir = PathBuf::from(&home_dir).join(".ssh");
    let authorized_keys_file = ssh_dir.join("authorized_keys");

    println!("Starting SSH permissions setup...");

    // 1. Check if the ~/.ssh directory exists. If not, create it.
    if !ssh_dir.exists() {
        println!(
            "Directory '{}' does not exist. Creating it...",
            ssh_dir.display()
        );
        if let Err(e) = fs::create_dir_all(&ssh_dir) {
            eprintln!(
                "Error: Failed to create directory '{}'. {}",
                ssh_dir.display(),
                e
            );
            exit(1);
        }
    } else {
        println!("Directory '{}' already exists.", ssh_dir.display());
    }

    // Call ssh-keygen
    println!("Generating SSH key pair (gnostr-gnit-key)...");
    let key_file_name = "gnostr-gnit-key";
    let output = Command::new("ssh-keygen")
        .arg("-t")
        .arg("ed25519")
        .arg("-f")
        .arg(ssh_dir.join(key_file_name))
        .arg("-C")
        .arg(&email)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("SSH key generation successful.");
            } else {
                eprintln!("Error: ssh-keygen failed.");
                eprintln!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
                eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
                exit(1);
            }
        }
        Err(e) => {
            eprintln!(
                "Error: Could not execute ssh-keygen. Is it installed and in your PATH? {}",
                e
            );
            exit(1);
        }
    }

    // 2. Set permissions for the ~/.ssh directory to 700 (drwx------).
    println!("Setting permissions for '{}' to 700...", ssh_dir.display());
    if let Err(e) = set_permissions(&ssh_dir, 0o700) {
        eprintln!(
            "Error: Failed to set permissions for '{}'. {}",
            ssh_dir.display(),
            e
        );
        exit(1);
    } else {
        println!("Permissions for '{}' set to 700.", ssh_dir.display());
    }

    // 3. Set permissions for the authorized_keys file.
    if authorized_keys_file.exists() {
        println!("File '{}' found.", authorized_keys_file.display());
        println!(
            "Setting permissions for '{}' to 600...",
            authorized_keys_file.display()
        );
        if let Err(e) = set_permissions(&authorized_keys_file, 0o600) {
            eprintln!(
                "Error: Failed to set permissions for '{}'. {}",
                authorized_keys_file.display(),
                e
            );
            exit(1);
        } else {
            println!(
                "Permissions for '{}' set to 600.",
                authorized_keys_file.display()
            );
        }
    } else {
        println!(
            "File '{}' not found. No permissions to set for it.",
            authorized_keys_file.display()
        );
        println!("Note: If you plan to use SSH keys for authentication, you will need to");
        println!(
            "add your public key to '{}'.",
            authorized_keys_file.display()
        );
    }

    // 4. Set permissions for private SSH keys.
    let private_key_types = vec![
        "id_rsa",
        "id_dsa",
        "id_ecdsa",
        "id_ed25519",
        "gnostr-gnit-key",
    ];

    println!("Checking for and setting permissions for private SSH keys...");
    for key_type in private_key_types {
        let private_key_file = ssh_dir.join(key_type);
        if private_key_file.exists() {
            println!("Private key '{}' found.", private_key_file.display());
            println!(
                "Setting permissions for '{}' to 600...",
                private_key_file.display()
            );
            if let Err(e) = set_permissions(&private_key_file, 0o600) {
                eprintln!(
                    "Error: Failed to set permissions for '{}'. {}",
                    private_key_file.display(),
                    e
                );
                exit(1);
            } else {
                println!(
                    "Permissions for '{}' set to 600.",
                    private_key_file.display()
                );
            }
        } else {
            println!("Private key '{}' not found.", private_key_file.display());
        }
    }

    // 5. Set permissions for public SSH keys.
    let public_key_types = vec![
        "id_rsa.pub",
        "id_dsa.pub",
        "id_ecdsa.pub",
        "id_ed25519.pub",
        "gnostr-gnit-key.pub",
    ];

    println!("Checking for and setting permissions for public SSH keys...");
    for key_type in public_key_types {
        let public_key_file = ssh_dir.join(key_type);
        if public_key_file.exists() {
            println!("Public key '{}' found.", public_key_file.display());
            println!(
                "Setting permissions for '{}' to 644...",
                public_key_file.display()
            );
            if let Err(e) = set_permissions(&public_key_file, 0o644) {
                eprintln!(
                    "Error: Failed to set permissions for '{}'. {}",
                    public_key_file.display(),
                    e
                );
                exit(1);
            } else {
                println!(
                    "Permissions for '{}' set to 644.",
                    public_key_file.display()
                );
            }
        } else {
            println!("Public key '{}' not found.", public_key_file.display());
        }
    }
}

fn set_permissions(path: &Path, mode: u32) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(mode);
        fs::set_permissions(path, perms)
    }

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(mode);
        fs::set_permissions(path, perms)
    }

    #[cfg(target_os = "windows")]
    {
        println!("TODO:Running on Windows!");
        // Windows-specific code here
        Ok(())
    }
}
