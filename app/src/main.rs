//! gnostr-relay

use clap::Parser;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
#[macro_use]
extern crate clap;

use anyhow::anyhow;
use gnostr::cli::setup_logging;
use gnostr_app::*;
use gpui::*;

use gnostr::Config;

//use simplelog::*;
//use simplelog::{Config, LevelFilter, WriteLogger};
use simplelog::WriteLogger;

use tracing::{debug, info, trace, warn};
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::FmtSubscriber;

/// Cli
#[derive(Debug, Parser)]
#[command(name = "gnostr-app", about = "gnostr-app", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// logging
    #[arg(long, value_name = "LOGGING")]
    pub logging: bool,

    /// logging level
    #[arg(long, value_name = "INFO")]
    pub info: bool,

    /// logging level
    #[arg(long, value_name = "DEBUG")]
    pub debug: bool,

    /// logging level
    #[arg(long, value_name = "TRACE")]
    pub trace: bool,

    /// logging level
    #[arg(long, value_name = "WARN")]
    pub warn: bool,
}

/// Commands
#[derive(Debug, Subcommand)]
enum Commands {
    /// Import data from jsonl file
    #[command(arg_required_else_help = true)]
    Import(ImportOpts),
    /// Export data to jsonl file
    #[command(arg_required_else_help = true)]
    Export(ExportOpts),
    /// Benchmark filter
    #[command(arg_required_else_help = true)]
    Bench(BenchOpts),
    /// Start nostr relay server
    Relay(RelayOpts),
    /// Delete data by filter
    Delete(DeleteOpts),
    /// Gui
    Gui(GuiOpts),
}

fn main() -> gnostr_app::Result<(), anyhow::Error> {
    let args = Cli::parse();

    if args.logging {
        //     let logging = gnostr::cli::setup_logging();
    };

    let level = if args.debug {
        debug!("debug={:?}", args.debug);
        LevelFilter::DEBUG
    } else if args.trace {
        trace!("trace={:?}", args.trace);
        LevelFilter::TRACE
    } else if args.info {
        info!("info={:?}", args.info);
        LevelFilter::INFO
    } else if args.warn {
        warn!("warn={:?}", args.warn);
        LevelFilter::WARN
    } else {
        LevelFilter::OFF
    };
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    //trace!("{:?}", app_cache);

    match args.command {
        Commands::Gui(opts) => {
            gui()?;
        }
        Commands::Import(opts) => {
            let total = import_opts(opts)?;
            println!("imported {} events", total);
        }
        Commands::Export(opts) => {
            export_opts(opts)?;
        }
        Commands::Bench(opts) => {
            bench_opts(opts)?;
        }
        Commands::Relay(opts) => {
            relay(&opts.config, opts.watch)?;
        }
        Commands::Delete(opts) => {
            let count = delete(&opts.path, &opts.filter, opts.dry_run)?;
            if opts.dry_run {
                println!("Would delete {} events", count);
            } else {
                println!("Deleted {} events", count);
            }
        }
    }
    Ok(())
}

struct GnostrApp {
    text: SharedString,
}

impl GnostrApp {
    fn get_blockheight(&self) -> String {
        gnostr::get_blockheight().expect("REASON")
    }
}
impl Render for GnostrApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(rgb(0x2e7d32))
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!(
                "{}:{}!",
                &self.text,
                GnostrApp::get_blockheight(&self)
            ))
    }
}

fn gui() -> anyhow::Result<()> {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_cx| GnostrApp {
                text: "blockheight".into(),
            })
        })
        .unwrap();
    });
    Ok(())
}
