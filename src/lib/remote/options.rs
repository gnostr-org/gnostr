pub use clap::value_parser;
pub use clap::Parser;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Opt {
    #[arg(short, long, default_value = "warn")]
    /// Sets the level of verbosity.
    /// Supported levels: error, warn, info, debug, trace
    pub log_level: String,
    #[arg(short, long)]
    /// Run as remote runner (default). This is the machine where the executable(s) will be run.
    #[arg(long, default_value_t = false, value_parser = value_parser!(bool))]
    pub remote_runner: bool,
    #[arg(short, long, default_value_t = 8888)]
    /// Select a TCP port to talk over. Has to be same on both sides.
    pub port: u16,
    #[arg(short, long, default_value = "127.0.0.1")]
    /// The remote runner to connect to.
    pub target: Option<String>,
    #[arg(short, long)]
    /// The executable to run.
    pub filename: Option<String>,
}
