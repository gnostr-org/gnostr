mod gnostr_dashboard {
    include!("bin/gnostr-dashboard.rs");
}

mod gnostr_tui {
    include!("bin/gnostr-tui.rs");
}

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = gnostr::cli::GnostrCli::parse();

    if cli.command.is_some() {
        gnostr_tui::run_with_cli(cli).await
    } else {
        gnostr_dashboard::run().await
    }
}
