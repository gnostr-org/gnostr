use gnostr::blockheight::blockheight_sync;
use gnostr::weeble::weeble_sync;
use gnostr::wobble::wobble_sync;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;


use std::path::{Path, PathBuf};
use std::process::{exit, Command};

// --- Structs for TOML configuration ---
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    name: String,
    port: u16,
    hostname: String,
    users: HashMap<String, User>,
    welcome_message: WelcomeMessage,
    extra: Extra,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    #[serde(default)] // Use default value (false) if not specified in TOML
    is_admin: bool,
    can_create_repos: bool,
    public_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct WelcomeMessage {
    welcome_message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Extra {
    extra: String,
}

fn move_gnostr_gnit_key() -> io::Result<()> {
    let home_dir = env::var("HOME").expect("HOME environment variable not set");
    let ssh_dir = PathBuf::from(&home_dir).join(".ssh");
    debug!("{}", ssh_dir.display());
    let gnostr_gnit_key_path = PathBuf::from(&home_dir)
        .join(".ssh")
        .join("gnostr-gnit-key");
    let gnostr_gnit_key_path_weeble_blockheight_wobble =
        PathBuf::from(&home_dir).join(".ssh").join(format!(
            "gnostr-gnit-key-{}-{}-{}",
            &weeble_sync().unwrap().to_string(),
            blockheight_sync(),
            &wobble_sync().unwrap().to_string()
        ));

    println!(
        "Attempting to rename/move '{}' to '{}'",
        gnostr_gnit_key_path.display(),
        gnostr_gnit_key_path_weeble_blockheight_wobble.display()
    );

    // Rename the file
    match fs::rename(
        gnostr_gnit_key_path,
        gnostr_gnit_key_path_weeble_blockheight_wobble,
    ) {
        Ok(_) => {
            println!("File renamed successfully!");
        }
        Err(e) => {
            eprintln!("Error renaming file: {}", e);
        }
    }

    // Clean up (optional)
    // fs::remove_file(to_path)?;
    // println!("Cleaned up '{}'", to_path.display());

    Ok(())
}

// --- Helper function for setting file permissions ---
fn set_permissions(path: &Path, mode: u32) -> std::io::Result<()> {
    #[cfg(unix)] // Applies to macOS and Linux
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(mode);
        fs::set_permissions(path, perms)
    }

    #[cfg(windows)]
    {
        // On Windows, `ssh-keygen` is expected to handle appropriate permissions.
        // Direct `set_mode` for Unix-style permissions is not applicable.
        Ok(())
    }
}

fn main() -> io::Result<()> {
    // --- SSH Key Generation and Permissions Setup ---
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

    let private_key_types = vec![
        //"id_rsa",
        //"id_dsa",
        //"id_ecdsa",
        //"id_ed25519",
        "gnostr-gnit-key",
    ];

    println!("Checking for and setting permissions for private SSH keys...");
    for key_type in private_key_types {
        let private_key_file = ssh_dir.join(key_type);
        if private_key_file.exists() {
            let _ = move_gnostr_gnit_key();
        }
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
        //"id_rsa",
        //"id_dsa",
        //"id_ecdsa",
        //"id_ed25519",
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

                println!("Generating SSH key add (gnostr-gnit-key)...");
                let key_file_name = "gnostr-gnit-key";
                let output = Command::new("ssh-add")
                    .arg(ssh_dir.join(key_file_name))
                    .output();

                match output {
                    Ok(output) => {
                        if output.status.success() {
                            println!("SSH key-add successful.");
                        } else {
                            eprintln!("Error: ssh-add failed.");
                            eprintln!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
                            eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
                            exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!(
                        "Error: Could not execute ssh-add. Is it installed and in your PATH? {}",
                        e
                    );
                        exit(1);
                    }
                }
            }
        } else {
            println!("Private key '{}' not found.", private_key_file.display());
        }
    }

    // 5. Set permissions for public SSH keys.
    let public_key_types = vec![
        //"id_rsa.pub",
        //"id_dsa.pub",
        //"id_ecdsa.pub",
        //"id_ed25519.pub",
        "gnostr-gnit-key.pub",
    ];

    let mut gnostr_gnit_pubkey: String = "".to_string();
    println!("Checking for and setting permissions for public SSH keys...");
    for key_type in public_key_types {
        let public_key_file = ssh_dir.join(key_type);
        if public_key_file.exists() {
            let pubkey_file_name = "gnostr-gnit-key.pub";
            #[cfg(unix)]
            let output = Command::new("cat")
                .arg(ssh_dir.join(pubkey_file_name))
                .output();

            #[cfg(windows)]
            let output = Command::new("type")
                .arg(ssh_dir.join(pubkey_file_name))
                .output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        gnostr_gnit_pubkey = String::from_utf8_lossy(&output.stdout)
                            .trim()
                            .to_string();
                        println!("gnostr-gnit-key.pub:\n{}", gnostr_gnit_pubkey);
                    } else {
                        eprintln!("Error: Failed to read public key.");
                        eprintln!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
                        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
                        exit(1);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Error: Could not execute command to read public key. Is it installed and in your PATH? {}",
                        e
                    );
                    exit(1);
                }
            }

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

    println!("SSH permissions setup complete.");
    println!("You can verify permissions with:");
    println!("ls -ld ~/.ssh");
    println!("ls -l ~/.ssh/");

    // --- TOML Configuration Generation ---
    println!("\nStarting TOML configuration generation...");

    let mut users = HashMap::new();

    println!("{}", gnostr_gnit_pubkey.clone().to_string());
    users.insert(
        "gnostr".to_string(),
        User {
            is_admin: true,
            can_create_repos: true,
            public_key: gnostr_gnit_pubkey.clone(),
        },
    );

    users.insert(
        "gnostr-user".to_string(),
        User {
            is_admin: false, // Explicitly set to false as it's not an admin
            can_create_repos: true,
            public_key: gnostr_gnit_pubkey.clone(),
        },
    );

    let config = Config {
        name: "gnostr.org".to_string(),
        port: 2222,
        hostname: "gnostr.org".to_string(),
        users,
        welcome_message: WelcomeMessage {
            welcome_message: "welcome to gnostr.org!".to_string(),
        },
        extra: Extra {
            extra: "extra toml content!".to_string(),
        },
    };

    let toml_string = toml::to_string(&config).expect("Failed to serialize config to TOML");

    println!("Generated server.toml content:\n{}", toml_string);

    fs::write("server.toml", toml_string)?;

    println!("server.toml generated successfully!");

    Ok(())
}
