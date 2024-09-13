use std::convert::TryInto;
use std::io::Read;
use std::{env, process};

use gnostr_types::Event;

static DEFAULT_RELAY_URL: &str = "wss://e.nos.lol";
fn main() {
    let mut relay_url = DEFAULT_RELAY_URL;
    if relay_url == DEFAULT_RELAY_URL {}
    let args_vector: Vec<String> = env::args().collect();

    #[allow(unreachable_code)]
    for i in 0..args_vector.len() {
        if i == args_vector.len() {
            process::exit(i.try_into().unwrap());
        } else {
            if args_vector.len() == 0 {
                print!("args_vector.len() = {}", 0);
            };
            if args_vector.len() == 1 {
                //no args case
                //no args case
                //no args case
                let mut s: String = String::new();
                std::io::stdin().read_to_string(&mut s).unwrap();
                let event: Event = serde_json::from_str(&s).unwrap();
                relay_url = DEFAULT_RELAY_URL;
                //always reprint s for further piping
                print!("{}\n", s);
                gnostr_bins::post_event(&relay_url, event);
            };
            if args_vector.len() == 2 {
                //catch help
                if args_vector[1] == "-h" {
                    print!(
                        "gnostr --sec <priv_key> | gnostr-post-event --relay {}",
                        DEFAULT_RELAY_URL
                    );
                    process::exit(0);
                }
                if args_vector[1] == "--help" {
                    print!(
                        "gnostr --sec <priv_key> | gnostr-post-event --relay {}",
                        DEFAULT_RELAY_URL
                    );
                    process::exit(0);
                }
                //catch version
                if args_vector[1] == "-v" {
                    const VERSION: &str = env!("CARGO_PKG_VERSION");
                    print!("v{}", VERSION);
                    process::exit(0);
                }
                if args_vector[1] == "--version" {
                    const VERSION: &str = env!("CARGO_PKG_VERSION");
                    print!("v{}", VERSION);
                    process::exit(0);
                }
                //catch missing url
                //because args_vector.len() == 2
                if args_vector[1] == "--relay" {
                    relay_url = &DEFAULT_RELAY_URL;
                    //pipe event from command line
                    //gnostr --sec <priv_key> | gnostr-post-event --relay>//
                    let mut s: String = String::new();
                    std::io::stdin().read_to_string(&mut s).unwrap();
                    let event: Event = serde_json::from_str(&s).unwrap();
                    //always reprint s for further piping
                    print!("{}\n", s);
                    gnostr_bins::post_event(relay_url, event);
                    process::exit(0);
                }
                //else assume the second arg is the relay url
                relay_url = &args_vector[1];
                //catch the stream
                //gnostr --sec <privkey> | gnostr-post-event <relay_url>
                let mut s: String = String::new();
                //this captures the stream when np --relay flag
                std::io::stdin().read_to_string(&mut s).unwrap();
                let event: Event = serde_json::from_str(&s).unwrap();
                //always reprint s for further piping
                print!("{}\n", s);
                gnostr_bins::post_event(relay_url, event);
                process::exit(0);
            };
            //this actually captures the stream when --relay flag
            if args_vector.len() == 3 {
                //and if
                if args_vector[1] == "--relay" {
                    relay_url = &args_vector[2];
                    let mut s: String = String::new();
                    std::io::stdin().read_to_string(&mut s).unwrap();
                    let event: Event = serde_json::from_str(&s).unwrap();
                    //always reprint s for further piping
                    print!("{}\n", s);
                    gnostr_bins::post_event(relay_url, event);
                    process::exit(0);
                }
                relay_url = &args_vector[3 - 1];
                let mut s: String = String::new();
                std::io::stdin().read_to_string(&mut s).unwrap();
                //always reprint s for further piping
                print!("{}\n", s);
                let event: Event = serde_json::from_str(&s).unwrap();
                gnostr_bins::post_event(relay_url, event);
            };
        }
    }
}
