use gnostr::verify_keypair::is_valid;
use std::{env, process};

fn main() {
    let mut args = env::args();

    if args.len() != 3 {
        println!("Usage:  verify_keypair <public> <private>");
        process::exit(1);
    }

    args.next().unwrap(); // the program name

    let verifying_key_string = args.next().unwrap();

    let signing_key_string = args.next().unwrap();

    if is_valid(verifying_key_string, signing_key_string) {
        println!("valid");
    }
}
