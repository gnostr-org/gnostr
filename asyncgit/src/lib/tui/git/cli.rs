use clap::{Parser, Subcommand};

/// Command-line arguments for the asyncgit TUI.
#[derive(Debug, Clone, Default, Parser)]
pub struct Args {
    /// Print version or help-related output and exit.
    #[arg(long)]
    pub version: bool,

    /// Enable debug logging.
    #[arg(long)]
    pub log: bool,

    /// Run without entering the alternate screen.
    #[arg(long)]
    pub print: bool,

    /// Replay a key sequence before entering the event loop.
    #[arg(long)]
    pub keys: Option<String>,

    /// TUI command to run.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Supported asyncgit TUI subcommands.
#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Show a specific commit or reference.
    Show {
        /// Reference to display.
        reference: String,
    },

    /// Inspect git notes.
    Notes,
}
