use clap::Parser;

use crate::cli::{GnostrCli, GnostrCommands, LegitCommands};
use crate::sub_commands::legit::LegitSubCommand;

fn parse_legit(args: &[&str]) -> LegitSubCommand {
    let cli = GnostrCli::try_parse_from(args).expect("legit command should parse");
    match cli.command {
        Some(GnostrCommands::Legit(command)) => command,
        _ => panic!("expected legit command"),
    }
}

fn is_init(command: &LegitSubCommand) -> bool {
    matches!(command.command, Some(LegitCommands::Init(_)))
}

fn is_fetch(command: &LegitSubCommand) -> bool {
    matches!(command.command, Some(LegitCommands::Fetch(_)))
}

fn is_list(command: &LegitSubCommand) -> bool {
    matches!(command.command, Some(LegitCommands::List))
}

fn is_pull(command: &LegitSubCommand) -> bool {
    matches!(command.command, Some(LegitCommands::Pull))
}

fn is_push(command: &LegitSubCommand) -> bool {
    matches!(command.command, Some(LegitCommands::Push(_)))
}

fn is_send(command: &LegitSubCommand) -> bool {
    matches!(command.command, Some(LegitCommands::Send(_)))
}

fn is_login(command: &LegitSubCommand) -> bool {
    matches!(command.command, Some(LegitCommands::Login(_)))
}

fn is_mine(command: &LegitSubCommand) -> bool {
    matches!(command.command, Some(LegitCommands::Mine))
}

#[test]
fn parses_legit_subcommands() {
    let cases: [(&[&str], fn(&LegitSubCommand) -> bool); 8] = [
        (["gnostr", "legit", "init"].as_slice(), is_init),
        (["gnostr", "legit", "fetch"].as_slice(), is_fetch),
        (["gnostr", "legit", "list"].as_slice(), is_list),
        (["gnostr", "legit", "pull"].as_slice(), is_pull),
        (["gnostr", "legit", "push"].as_slice(), is_push),
        (["gnostr", "legit", "send"].as_slice(), is_send),
        (["gnostr", "legit", "login"].as_slice(), is_login),
        (["gnostr", "legit", "mine"].as_slice(), is_mine),
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
