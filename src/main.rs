mod gnostr_dashboard {
    include!("bin/gnostr-dashboard.rs");
}

mod gnostr_tui {
    include!("bin/gnostr-tui.rs");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gnostr_dashboard::run().await
}
