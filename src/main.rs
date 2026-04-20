mod gnostr_dashboard_portable_pty {
    include!("bin/gnostr-dashboard-portable-pty.rs");
}

mod gnostr_tui {
    include!("bin/gnostr-tui.rs");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gnostr_dashboard_portable_pty::run().await
}
