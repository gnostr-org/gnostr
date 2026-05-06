use anyhow::Result;

pub async fn run() -> Result<()> {
    ngit::git_remote_nostr::run().await
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
