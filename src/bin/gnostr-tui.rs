use gnostr::tui::tui::tui;
#[tokio::main]
async fn main() -> Result<(), ()> {
    let _ = tui().await;
    Ok(())
}
