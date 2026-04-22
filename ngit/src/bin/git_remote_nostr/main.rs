#![cfg_attr(not(test), warn(clippy::pedantic))]
#![allow(clippy::large_futures, clippy::module_name_repetitions)]
#![allow(dead_code)]
#![cfg_attr(not(test), warn(clippy::expect_used))]

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    gnostr_ngit::git_remote_nostr::run().await
}
