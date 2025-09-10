#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::unused_self,
    clippy::future_not_send
)]

mod app;
mod cube;
mod global_rt;
mod system_command;
mod ui;

use crate::global_rt::global_rt;
use clap::{Arg, ArgAction, Command, Parser};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use cube_tui::chat::chat;
use cube_tui::system_command::system_command_test;
use cube_tui::local_git::local_git_test;
use cube_tui::CompleteConfig;
use cube_tui::WrapErr;
use std::{error::Error, io};
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "user")]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
    #[arg(short = 't', long)]
    tui: bool,
    #[arg(long)]
    chat: bool,
    #[arg(long = "cfg", default_value = "")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let global_rt_result = global_rt().spawn(async move {
	let _ = chat();
        println!("global_rt async task!");
        String::from("global_rt async task!")
    });
    println!("global_rt_result={:?}", global_rt_result.await);
    let global_rt_result = global_rt().spawn(async move {
        system_command_test();
        String::from("global_rt async task!")
    });
    println!("global_rt_result={:?}", global_rt_result.await);

    for args in 0..args.count {
        let global_rt_result = global_rt()
            .spawn(async move {
                println!("global_rt async task! {}", &args.clone());
            })
            .await;
        println!("global_rt_result={:?}!", global_rt_result);
    }

    let cmd = Command::new("gnostr-chat")
        .arg(
            Arg::new("name")
                .long("name")
                .short('n')
                //.required(true)
                .action(ArgAction::Set)
                .default_value("-"),
        )
        .arg(
            Arg::new("count")
                .long("count")
                .short('c')
                //.required(true)
                .action(ArgAction::Set)
                .default_value("0"),
        )
        .arg(
            Arg::new("tui")
                .long("tui")
                .short('t')
                //.required(true)
                .action(ArgAction::SetTrue)
                .default_value("false"),
        )
        .arg(
            Arg::new("chat")
                .long("chat")
                //.required(true)
                .action(ArgAction::SetTrue)
                .default_value("false"),
        )
        .arg(Arg::new("config").long("cfg").action(ArgAction::Set))
        .get_matches();

    assert!(cmd.clone().contains_id("tui"));

    let matches = cmd.clone();
    assert!(matches.contains_id("tui"));

    if let Some(c) = matches.get_one::<bool>("tui") {
        if matches.get_flag("tui") {
            println!("Value for --tui: {c}");
            assert_eq!(matches.get_flag("tui"), true);
        }
    }
    if let Some(c) = matches.get_one::<bool>("chat") {
        if matches.get_flag("chat") {
            let global_rt_result = global_rt()
                .spawn(async move {
                    println!("global_rt async task!");
                    //evt_loop(input_rx, peer_tx, topic).await.unwrap();
                    String::from("global_rt async task!");
                })
                .await;
            println!("global_rt_result={:?}", global_rt_result?);
            println!("Value for --chat: {c}");
            assert_eq!(matches.get_flag("tui"), true);
        }
    }
    color_eyre::install().unwrap();

    let config = CompleteConfig::new()
        .wrap_err("Configuration error.")
        .unwrap();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = ui::run(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err)
    }

    Ok(())
}
