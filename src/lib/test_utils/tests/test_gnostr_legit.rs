use clap::Parser;

use crate::cli::{GnostrCli, GnostrCommands, LegitCommands};

fn parse_legit(args: &[&str]) -> crate::cli::LegitSubCommand {
    let cli = GnostrCli::try_parse_from(args).expect("legit command should parse");
    match cli.command {
        Some(GnostrCommands::Legit(command)) => command,
        other => panic!("expected legit command, got {other:?}"),
    }
}

#[test]
fn parses_legit_subcommands() {
    let cases = [
        (
            ["gnostr", "legit", "init"].as_slice(),
            |command: &crate::cli::LegitSubCommand| {
                matches!(command.command, Some(LegitCommands::Init(_)))
            },
        ),
        (
            ["gnostr", "legit", "fetch"].as_slice(),
            |command: &crate::cli::LegitSubCommand| {
                matches!(command.command, Some(LegitCommands::Fetch(_)))
            },
        ),
        (
            ["gnostr", "legit", "list"].as_slice(),
            |command: &crate::cli::LegitSubCommand| matches!(command.command, Some(LegitCommands::List)),
        ),
        (
            ["gnostr", "legit", "pull"].as_slice(),
            |command: &crate::cli::LegitSubCommand| matches!(command.command, Some(LegitCommands::Pull)),
        ),
        (
            ["gnostr", "legit", "push"].as_slice(),
            |command: &crate::cli::LegitSubCommand| {
                matches!(command.command, Some(LegitCommands::Push(_)))
            },
        ),
        (
            ["gnostr", "legit", "send"].as_slice(),
            |command: &crate::cli::LegitSubCommand| {
                matches!(command.command, Some(LegitCommands::Send(_)))
            },
        ),
        (
            ["gnostr", "legit", "login"].as_slice(),
            |command: &crate::cli::LegitSubCommand| {
                matches!(command.command, Some(LegitCommands::Login(_)))
            },
        ),
        (
            ["gnostr", "legit", "mine"].as_slice(),
            |command: &crate::cli::LegitSubCommand| {
                matches!(command.command, Some(LegitCommands::Mine))
            },
        ),
    ];

    for (args, predicate) in cases {
        let command = parse_legit(args);
        assert!(predicate(&command), "unexpected command for {args:?}: {command:?}");
    }
}

#[test]
fn parses_legit_verbose_count_flags() {
    let command = parse_legit(["gnostr", "legit", "-v", "--verbose", "mine"].as_slice());
    assert_eq!(command.verbose, 2);
}

#[test]
fn defaults_to_mine_without_subcommand() {
    let command = parse_legit(["gnostr", "legit"].as_slice());
    assert!(command.command.is_none());
    assert_eq!(command.verbose, 0);
}
