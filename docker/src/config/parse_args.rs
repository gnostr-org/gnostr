use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug, Clone, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
#[command(version, about)]
pub struct Args {
    /// Docker update interval in ms, minimum effectively 1000
    #[arg(short = 'd', value_name = "ms", default_value_t = 1000)]
    pub docker_interval: u32,

    /// Remove timestamps from Docker logs
    #[arg(short = 't')]
    pub timestamp: bool,

    /// Attempt to colorize the logs, conflicts with "-r"
    #[arg(short = 'c', conflicts_with = "raw")]
    pub color: bool,

    /// Show raw logs, default is to remove ansi formatting, conflicts with "-c"
    #[arg(short = 'r', conflicts_with = "color")]
    pub raw: bool,

    /// Show self when running as a docker container
    #[arg(short = 's')]
    pub show_self: bool,

    /// Don't draw gui - for debugging - mostly pointless
    #[arg(short = 'g')]
    pub gui: bool,

    /// Docker host, defaults to `/var/run/docker.sock`
    #[arg(long, short = None)]
    pub host: Option<String>,

    /// Do not include stderr output in logs
    #[arg(long = "no-stderr")]
    pub no_std_err: bool,

    /// Display the container logs timestamp with a given timezone, default is UTC
    #[arg(long="timezone", short = None)]
    pub timezone: Option<String>,

    /// Directory for saving exported logs, defaults to `$HOME`
    #[arg(long="save-dir", short = None)]
    pub save_dir: Option<String>,

    /// Path to a config file, readable as TOML, JSONC, or JSON
    #[arg(long="config-file", short = None)]
    pub config_file: Option<String>,

    /// Force use of docker cli when execing into containers
    #[arg(long="use-cli", short = None)]
    pub use_cli: bool,

    /// Shell/command to use when execing into containers (default: sh)
    #[arg(long="exec-command", short = None)]
    pub exec_command: Option<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            docker_interval: 1000,
            timestamp: true,
            color: false,
            raw: false,
            show_self: false,
            gui: true,
            host: None,
            no_std_err: true,
            timezone: None,
            save_dir: None,
            config_file: None,
            use_cli: false,
            exec_command: None,
        }
    }
}
