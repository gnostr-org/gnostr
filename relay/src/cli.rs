use std::path::Path;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct RelayCli {
    /// The logging level
    #[arg(short, long, default_value = "info")]
    pub logging: String,

    /// Path to the relay config file
    #[arg(long, default_value = "config/gnostr.toml")]
    pub config_file_path: String,
}

impl Default for RelayCli {
    fn default() -> Self {
        Self {
            logging: String::from("info"),
            config_file_path: String::from("config/gnostr.toml"),
        }
    }
}

impl RelayCli {
    pub fn config_path_always(&self) -> Option<&str> {
        Some(self.config_file_path.as_str())
    }

    pub fn config_path_if_exists(&self) -> Option<&str> {
        if Path::new(&self.config_file_path).exists() {
            Some(self.config_file_path.as_str())
        } else {
            None
        }
    }
}
