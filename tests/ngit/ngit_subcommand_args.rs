use gnostr::cli::{NgitCli, NgitCommands};
use clap::Parser;

#[test]
fn test_ngit_subcommand_global_args() {
    // Test with all global arguments provided
    let args = vec![
        "ngit",
        "--nsec",
        "test_nsec",
        "--password",
        "test_password",
        "list",
    ];
    let cli_args = NgitCli::parse_from(args);

    assert_eq!(cli_args.nsec, Some("test_nsec".to_string()));
    assert_eq!(cli_args.password, Some("test_password".to_string()));

    // Test with no global arguments
    let args_no_globals = vec![
        "ngit",
        "list",
    ];
    let cli_args_no_globals = NgitCli::parse_from(args_no_globals);

    assert_eq!(cli_args_no_globals.nsec, None);
    assert_eq!(cli_args_no_globals.password, None);

    // Test with only nsec
    let args_only_nsec = vec![
        "ngit",
        "--nsec",
        "only_nsec",
        "list",
    ];
    let cli_args_only_nsec = NgitCli::parse_from(args_only_nsec);

    assert_eq!(cli_args_only_nsec.nsec, Some("only_nsec".to_string()));
    assert_eq!(cli_args_only_nsec.password, None);

    // Test with only password
    let args_only_password = vec![
        "ngit",
        "--password",
        "only_password",
        "list",
    ];
    let cli_args_only_password = NgitCli::parse_from(args_only_password);

    assert_eq!(cli_args_only_password.nsec, None);
    assert_eq!(cli_args_only_password.password, Some("only_password".to_string()));

    // Test with only disable_cli_spinners
    let args_only_spinners = vec![
        "ngit",
        "--disable-cli-spinners",
        "true",
        "list",
    ];
    let cli_args_only_spinners = NgitCli::parse_from(args_only_spinners);

    assert_eq!(cli_args_only_spinners.nsec, None);
    assert_eq!(cli_args_only_spinners.password, None);
    assert_eq!(cli_args_only_spinners.disable_cli_spinners, Some(true));
    assert!(matches!(cli_args_only_spinners.command, NgitCommands::List));
}
