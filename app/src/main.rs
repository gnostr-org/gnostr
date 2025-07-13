//! gnostr-relay
use clap::Parser;
#[macro_use]
extern crate clap;

use gnostr_app::*;
use gpui::*;

/// Cli
#[derive(Debug, Parser)]
#[command(name = "gnostr-app", about = "gnostr-app", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
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
