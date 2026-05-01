#[path = "gnostr_chat_tui_demo.rs"]
mod demo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    demo::run(demo::DemoPreset::RelayIo).await
}
