// Copyright 2020 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env;
use std::process::{ExitCode, *};

use futures::executor::block_on;
use gnostr_bins::{get_stripped_urls, get_watch_list, get_watch_list_json, print_watch_list};
use jj_cli::cli_util::CliRunner;

pub fn handle_command(mut args: env::Args) -> Result<bool, Box<dyn std::error::Error>> {
    let _ = args.next(); // program name
    let command = args.next().unwrap(); // must be there or we would not have been called

    #[cfg(debug_assertions)]
    println!("\n*** gnostr-gui is running in command mode ***");
    #[cfg(debug_assertions)]
    println!("*** COMMAND = {} ***\n", command);

    match &*command {
        //sec nostr private ley
        "sec" => gnostr_sec(),
        "--sec" => gnostr_sec(),

        //gnostr-gui
        "gui" => gnostr_gui(),
        "--gui" => gnostr_gui(),

        //gnostr-tui
        "tui" => gnostr_tui(),
        "--tui" => gnostr_tui(),

        //json
        "-j" => json(),
        "--json" => json(),
        "json" => json(),
        //print
        "-p" => print(),
        "--print" => print(),
        "print" => print(),
        //get
        "-g" => get(),
        "--get" => get(),
        "get" => get(),
        //stripped
        "-s" => stripped(),
        "--stripped" => stripped(),
        "stripped" => stripped(),

        //version
        "-V" => version(),
        //support help2man
        "-v" => version(),
        "--version" => version(),
        "version" => version(),
        //help
        "-h" => help(),
        "--help" => help(),
        "help" => help(),
        //other
        other => println!("Unknown command {}", other),
    }
    Ok(true)
}
fn gnostr_sec() {
    print!("sec called");
}
fn gnostr_gui() {
    print!("gui called");
}
fn gnostr_tui() {
    print!("tui called");
}
fn default() {
    json();
    use std::process;
    process::exit(0);
}
fn json() {
    let future = get_watch_list_json();
    let _ = block_on(future);
}
fn print() {
    let future = print_watch_list();
    let _ = block_on(future);
}
fn get() {
    let future = get_watch_list();
    let _ = block_on(future);
}
//TODO: return length in watch_list?
fn stripped() {
    let future = get_stripped_urls();
    let length = block_on(future);
    //print!("{}",format!("{:?}",length.unwrap()));
    print!("{}", format!("{:?}", length.expect("REASON").len()));
}
fn help() {
    use std::process;

    let package_name = env!("CARGO_PKG_NAME");
    let crate_name = env!("CARGO_CRATE_NAME");
    let version = env!("CARGO_PKG_VERSION");
    print!(
        "\n1:{} v{}\n\n",
        package_name.replace("jj-cli", "gnostr"),
        version
    );
    print!("\n{} v{}\n\n", crate_name.replace("git_", ""), version);
    print!("3:{} get\n", crate_name.replace("git_", ""));
    print!("4:       <csv_relay_list>\n");
    print!("5:{} json\n", crate_name.replace("git_", ""));
    print!("6:       <json_relay_list>\n");
    print!("7:{} stripped\n", crate_name.replace("git_", "-"));
    print!("8:       <string_relay_list> <int_length_last>\n");
    process::exit(0);
}
fn version() {
    use std::process;

    print!("");

    let version = env!("CARGO_PKG_VERSION");
    let crate_name = env!("CARGO_CRATE_NAME");
    //let name = env!("CARGO_PKG_NAME");
    //let author = env!("CARGO_PKG_AUTHORS");

    //println!("Program Name: {}", name);
    //println!("Program Version: {}", version);
    println!(
        "{} v{}",
        crate_name.replace("git_gnostr", "gnostr"),
        version
    );
    //println!("Program Version: {}", version);
    //println!("Program Author: {}", author);

    process::exit(0);
}

fn gnostr_git() -> std::process::ExitCode {
    //CliRunner::init().version(env!("JJ_VERSION")).run()

    let exit: ExitCode = 0.into();
    exit
}
fn main() {
    use std::process;
    print!("main\n");
    print!("gnostr\n");
    print!("git-gnostr\n");
    // If we were handed a command, execute the command and return
    let args = env::args();
    if args.len() > 1 {
        let _ = handle_command(env::args());
    } else {
        default();
    }
    //if args.len() > 1 {
    //    for arg in args {
    //        if arg == "json" {json();process::exit(0)}
    //        if arg == "get" {get();process::exit(0)}
    //        if arg == "print" {print();process::exit(0)} else {
    //           let _ = handle_command(env::args()); process::exit(0);
    //        }
    //    }
    //}

    let _exit_code: std::process::ExitCode = gnostr_git();
    _exit_code;
}
