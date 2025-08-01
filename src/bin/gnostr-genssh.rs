use std::env;
use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

//#[cfg(target_os = "windows")]
//use std::os::windows::fs::PermissionsExt;

fn main() {
    let email = env::args()
        .nth(1)
        .unwrap_or_else(|| "gnostr@gnostr.org".to_string());

    let home_dir = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .expect("HOME or USERPROFILE environment variable not set");
    let ssh_dir = PathBuf::from(&home_dir).join(".ssh");
    let authorized_keys_file = ssh_dir.join("authorized_keys");

    println!("Starting SSH permissions setup...");

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

    println!("Generating SSH key pair (gnostr-gnit-key)...");
    let key_file_name = "gnostr-gnit-key";
    let output = Command::new("ssh-keygen")
        .arg("-t")
        .arg("ed25519")
        .arg("-f")
        .arg(ssh_dir.join(key_file_name))
        .arg("-C")
        .arg(&email)
        .arg("-N")
        .arg("")
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

    // 2. Set permissions for the ~/.ssh directory.
    // On Windows, setting specific "mode" bits like 700 directly isn't equivalent.
    // We'll aim for "owner only" via readonly and then rely on ACLs for security.
    println!("Setting permissions for '{}'...", ssh_dir.display());
    if let Err(e) = set_directory_permissions(&ssh_dir) {
        eprintln!(
            "Error: Failed to set permissions for '{}'. {}",
            ssh_dir.display(),
            e
        );
        exit(1);
    } else {
        println!("Permissions for '{}' adjusted for OS.", ssh_dir.display());
    }

    // 3. Set permissions for the authorized_keys file.
    if authorized_keys_file.exists() {
        println!("File '{}' found.", authorized_keys_file.display());
        println!(
            "Setting permissions for '{}'...",
            authorized_keys_file.display()
        );
        if let Err(e) = set_file_permissions(&authorized_keys_file) {
            eprintln!(
                "Error: Failed to set permissions for '{}'. {}",
                authorized_keys_file.display(),
                e
            );
            exit(1);
        } else {
            println!(
                "Permissions for '{}' adjusted for OS.",
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
    let private_key_types = vec!["id_rsa", "id_dsa", "id_ecdsa", "id_ed25519", key_file_name];

    println!("Checking for and setting permissions for private SSH keys...");
    for key_type in private_key_types {
        let private_key_file = ssh_dir.join(key_type);
        if private_key_file.exists() {
            println!("Private key '{}' found.", private_key_file.display());
            println!(
                "Setting permissions for '{}'...",
                private_key_file.display()
            );
            if let Err(e) = set_file_permissions(&private_key_file) {
                eprintln!(
                    "Error: Failed to set permissions for '{}'. {}",
                    private_key_file.display(),
                    e
                );
                exit(1);
            } else {
                println!(
                    "Permissions for '{}' adjusted for OS.",
                    private_key_file.display()
                );
            }
        } else {
            println!("Private key '{}' not found.", private_key_file.display());
        }
    }

    // 5. Set permissions for public SSH keys.

    let binding = key_file_name.to_owned() + ".pub";
    let public_key_types = vec![
        "id_rsa.pub",
        "id_dsa.pub",
        "id_ecdsa.pub",
        "id_ed25519.pub",
        &binding,
    ];

    println!("Checking for and setting permissions for public SSH keys...");
    for key_type in public_key_types {
        let public_key_file = ssh_dir.join(key_type);
        if public_key_file.exists() {
            println!("Public key '{}' found.", public_key_file.display());
            println!("Setting permissions for '{}'...", public_key_file.display());
            if let Err(e) = set_public_key_permissions(&public_key_file) {
                eprintln!(
                    "Error: Failed to set permissions for '{}'. {}",
                    public_key_file.display(),
                    e
                );
                exit(1);
            } else {
                println!(
                    "Permissions for '{}' adjusted for OS.",
                    public_key_file.display()
                );
            }
        } else {
            println!("Public key '{}' not found.", public_key_file.display());
        }
    }
}

// Function to set permissions for directories
fn set_directory_permissions(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o700); // rwx------
        fs::set_permissions(path, perms)
    }

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o700); // rwx------
        fs::set_permissions(path, perms)
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, there's no direct equivalent to chmod for directories to make them owner-only
        // using just standard library `Permissions`.
        // The `SetFileAttributes` function (which `set_permissions` uses) primarily sets
        // flags like `FILE_ATTRIBUTE_READONLY`.
        // For true granular control (like owner-only), you need to work with ACLs.
        // For SSH, the critical part is that the user account running SSH *can* access these files,
        // and that other users *cannot*. Relying on default Windows permissions where only the
        // current user has full control is often sufficient for ~/.ssh.
        // If a stricter ACL is needed, a crate like `windows-permissions` or direct WinAPI calls
        // would be necessary. For this script, we'll ensure it's not world-writable via the
        // `set_readonly(true)` if it's a file, but for directories, `create_dir_all` often
        // inherits sensible permissions. We'll simply ensure it's not marked as readonly.
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_readonly(false); // Ensure it's not read-only
        fs::set_permissions(path, perms)?;
        println!("  (Windows: Relying on default ACLs for directory permissions.)");
        Ok(())
    }
}

// Function to set permissions for private files (like private keys, authorized_keys)
fn set_file_permissions(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600); // rw-------
        fs::set_permissions(path, perms)
    }

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600); // rw-------
        fs::set_permissions(path, perms)
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, setting a file to "600" (owner read/write only) means ensuring
        // it's not set as FILE_ATTRIBUTE_READONLY and that its ACL only grants
        // the current user full control. `set_readonly(true)` makes it *more* restricted.
        // For private keys, we want to ensure only the owner can read/write.
        // The `std::fs::set_permissions` function on Windows corresponds to `SetFileAttributes`.
        // Setting `set_readonly(true)` is the closest standard library equivalent to restrict
        // writes, but true owner-only access usually involves ACL manipulation.
        // SSH on Windows generally expects the private key file to *not* be accessible
        // by other users. The default file creation permissions often achieve this.
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_readonly(true); // Attempt to make it read-only for all, closest to 600
        fs::set_permissions(path, perms)?;
        println!("  (Windows: Set file to read-only attribute. For stronger security, consider manual ACL review.)");
        Ok(())
    }
}

// Function to set permissions for public files (like public keys)
fn set_public_key_permissions(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o644); // rw-r--r--
        fs::set_permissions(path, perms)
    }

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o644); // rw-r--r--
        fs::set_permissions(path, perms)
    }

    #[cfg(target_os = "windows")]
    {
        // For public keys (644), we want owner read/write, others read.
        // On Windows, this translates to ensuring it's not marked as readonly,
        // and relying on default ACLs that typically allow read by "Everyone"
        // and write by "Owner" (or administrators).
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_readonly(false); // Ensure it's not read-only
        fs::set_permissions(path, perms)?;
        println!("  (Windows: Ensured public key is not read-only. Default ACLs usually allow broader read access.)");
        Ok(())
    }
}
