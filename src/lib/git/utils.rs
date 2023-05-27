use std::path::Path;

use anyhow::Result;
use directories::UserDirs;
use git2::{Config, Error};

pub fn set_git_timeout() -> Result<(), Error> {
    let mut config = Config::open_default()?;
    config.set_i32("http.connecttimeout", 10)?; // 10 seconds
    config.set_i32("http.lowspeedtime", 30)?;   // 30 seconds
    Ok(())
}

pub fn check_ssh_keys() -> bool {
    // Get the user's home directory using the directories crate
    if let Some(user_dirs) = UserDirs::new() {
        let ssh_dir = user_dirs.home_dir().join(".ssh");
        let key_files = vec![
            "id_rsa",
            "id_ecdsa",
            "id_ed25519",
            "id_rsa.pub",
            "id_ecdsa.pub",
            "id_ed25519.pub",
        ];

        for key in key_files {
            if Path::new(&ssh_dir.join(key)).exists() {
                return true; // At least one key exists
            }
        }
    }
    false // No keys found
}
