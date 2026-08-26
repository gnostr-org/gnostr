use anyhow::Result;

pub async fn launch() -> Result<()> {
    let args = ngit::sub_commands::sync::SubCommandArgs {
        ref_name: None,
        force: false,
    };
    ngit::sub_commands::sync::launch(&args).await
}
