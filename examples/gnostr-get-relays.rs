use std::env;

use futures::executor::block_on;
use gnostr::watch_list::{parse_json, parse_urls, stripped_urls};
use gnostr::{
    get_relays_by_nip, get_stripped_urls, get_watch_list, get_watch_list_json, print_watch_list,
};
pub fn handle_command() -> Result<bool, Box<dyn std::error::Error>> {
    let mut args = env::args().peekable();
    let relays: String;
    let output_type: String;
    let _ = args.next().unwrap();
    //gnostr-get-relays --nip <nip> ?
    if args.len() == 3 && args.peek().unwrap() == "--nip" || args.peek().unwrap() == "-n" {
        let _ = String::from(args.next().unwrap());
        relays = get_relays_by_nip(&args.next().unwrap())?;
        output_type = args.next().expect("REASON");
        if output_type == "-h" || output_type == "--help" {
            help("");
        }
        if output_type == "-j" {
            let relays_json = parse_json(&relays);
            let _ = block_on(relays_json);
        }
        if output_type == "-p" {
            let relays_json = parse_urls(&relays);
            let _ = block_on(relays_json);
        }
        if output_type == "-s" {
            let relays_json = stripped_urls(&relays);
            let _ = block_on(relays_json);
        } else {
            let relays_json = parse_json(&relays);
            let _ = block_on(relays_json);
        }
        std::process::exit(0);
    } else if args.len() == 2 && args.peek().unwrap() == "--nip" || args.peek().unwrap() == "-n" {
        let _ = String::from(args.next().unwrap());
        relays = get_relays_by_nip(&args.next().unwrap())?;
        println!("{}", relays);
        std::process::exit(0);
    }
    let flag = args.next().unwrap(); // must be there or we would not have been called
    #[cfg(debug_assertions)]
    println!("flag={}\n", flag);

    match &*flag {
        //nip
        "-n" => by_nip(&String::from("1")),
        "--nip" => by_nip(&String::from("1")),
        "nip" => by_nip(&String::from("1")),
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
        "-h" => help(""),
        "--help" => help(""),
        "help" => help(""),
        //other
        //other => println!("Unknown command {}", other),
        other => help(other),
    }
    Ok(true)
}
fn default() {
    json();
}
fn json() {
    let future = get_watch_list_json();
    let result = block_on(future);
    log::info!("{:?}", result.unwrap());
}
fn print() {
    let future = print_watch_list();
    let result = block_on(future);
    log::info!("{:?}", result.unwrap());
}
fn get() {
    let future = get_watch_list();
    let result = block_on(future);
    log::info!("{:?}", result.unwrap());
}
fn by_nip(nip: &str) {
    let relays = get_relays_by_nip(&nip).clone();
    print!("{}", relays.unwrap());
    std::process::exit(0);
}
fn stripped() {
    let future = get_stripped_urls();
    let length = block_on(future);
    print!(
        "{}",
        format!("{:?}", length.expect("stripped relay list length").len())
    );
}
fn help(other: &str) {
    use std::process;
    let crate_name = env!("CARGO_CRATE_NAME");
    let version = env!("CARGO_PKG_VERSION");
    println!("{} v{}", crate_name.replace('_', "-"), version);
    println!("{} --nip <int>", crate_name.replace('_', "-"));
    println!("{} --nip <int> [-j, -s, -p]", crate_name.replace('_', "-"));
    println!(
        "{} get [-g, -p, --get, --print]",
        crate_name.replace('_', "-")
    );
    println!("       <csv_relay_list>");
    println!("{} json [-j, --json]", crate_name.replace('_', "-"));
    println!("       <json_relay_list>");
    println!("{} stripped [-s, --stripped]", crate_name.replace('_', "-"));
    println!("       <stripped_relay_list> <list_length>");
    if other.len() > 0 {
        print!("\n{} {} error!", crate_name, other);
    }
    process::exit(0);
}
fn version() {
    use std::process;
    print!("");

    let version = env!("CARGO_PKG_VERSION");
    let crate_name = env!("CARGO_CRATE_NAME");
    //let name = env!("CARGO_PKG_NAME");
    //let author = env!("CARGO_PKG_AUTHORS");
    println!("{} v{}", crate_name.replace('_', "-"), version);
    process::exit(0);
}
fn main() {
    use std::process;
    // If we were handed a command, execute the command and return
    let args = env::args().peekable();
    if args.len() > 1 {
        let _ = handle_command();
    } else {
        default();
    }
    process::exit(0);
}
/// cargo       test --bin gnostr-get-relays -- --nocapture
/// cargo +nightly t --bin gnostr-get-relays -- --nocapture
#[test]
fn gnostr_get_relays_default() {
    //default();
    println!();
}
#[test]
fn gnostr_get_relays_json() {
    //json();
    println!();
}
#[test]
fn gnostr_get_relays_print() {
    //print();
    println!();
}
#[test]
fn gnostr_get_relays_stripped() {
    //stripped();
    println!();
}
#[test]
fn gnostr_get_relays_handle_command() {
    let args = env::args();
    if args.len() > 1 {
        let _ = handle_command();
    }
    println!();
}
