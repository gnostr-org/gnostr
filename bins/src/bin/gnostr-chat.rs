use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = gnostr_chat::ChatSubCommands::parse();
    gnostr_chat::chat(&args).await
}
