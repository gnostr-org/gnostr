use std::env;

use futures::executor::block_on;
use gnostr_bins::{get_stripped_urls, get_watch_list, get_watch_list_json, print_watch_list};

pub fn handle_command(mut args: env::Args) -> Result<bool, Box<dyn std::error::Error>> {
    let _ = args.next(); // program name
    let command = args.next().unwrap(); // must be there or we would not have been called

    #[cfg(debug_assertions)]
    println!("\n*** gnostr-gui is running in command mode ***");
    #[cfg(debug_assertions)]
    println!("*** COMMAND = {} ***\n", command);

    match &*command {
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

    let crate_name = env!("CARGO_CRATE_NAME");
    let version = env!("CARGO_PKG_VERSION");
    print!("\n{} v{}\n\n", crate_name.replace('_', "-"), version);
    println!("{} get", crate_name.replace('_', "-"));
    println!("       <csv_relay_list>");
    println!("{} json", crate_name.replace('_', "-"));
    println!("       <json_relay_list>");
    println!("{} stripped", crate_name.replace('_', "-"));
    println!("       <string_relay_list> <int_length_last>");
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
    println!("{} v{}", crate_name.replace('_', "-"), version);
    //println!("Program Version: {}", version);
    //println!("Program Author: {}", author);

    process::exit(0);
}
fn main() {
    use std::process;
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
    process::exit(0);
}
