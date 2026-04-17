use crate::sub_commands::ngit::{NgitSubCommand, ngit};

#[tokio::test]
async fn test_ngit_customize_dispatch() {
    let sub_command_args = NgitSubCommand {
        command: None,
        bunker_uri: None,
        bunker_app_key: None,
        nsec: None,
        password: None,
        disable_cli_spinners: true,
        customize: true,
        defaults: false,
        interactive: false,
        force: false,
        verbose: false,
    };

    let result = ngit(sub_command_args).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ngit_help_dispatch() {
    let sub_command_args = NgitSubCommand {
        command: None,
        bunker_uri: None,
        bunker_app_key: None,
        nsec: None,
        password: None,
        disable_cli_spinners: true,
        customize: false,
        defaults: false,
        interactive: false,
        force: false,
        verbose: false,
    };

    let result = ngit(sub_command_args).await;
    assert!(result.is_ok());
}
