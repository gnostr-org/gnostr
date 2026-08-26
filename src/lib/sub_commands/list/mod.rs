use anyhow::Result;

pub async fn launch() -> Result<()> {
    ngit::sub_commands::list::launch("open,draft".to_string(), false, None, false).await
}
