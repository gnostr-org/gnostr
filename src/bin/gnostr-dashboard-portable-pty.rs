use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = gnostr::cli::GnostrCli::parse();
    gnostr::dashboard::run_dashboard(args.commands).await
}
