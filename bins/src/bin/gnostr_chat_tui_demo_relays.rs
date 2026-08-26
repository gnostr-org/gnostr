#![allow(dead_code)]

#![allow(dead_code)]

#![allow(dead_code)]

#[path = "gnostr_chat_tui_demo.rs"]
mod demo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    demo::run(demo::DemoPreset::RelaysFirst).await
}
