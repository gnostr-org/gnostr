use nostr::prelude::*;
use std::io::{self, BufRead};
use base64::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 { return Ok(()); }
    let remote_url = args[2].replace("gnostr://", "http://");
      
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        handle.read_line(&mut line)?;
        match line.trim() {
            "capabilities" => println!("fetch
push
"),
            "list" => println!("? refs/heads/main
@refs/heads/main HEAD
"),
            s if s.starts_with("push ") => {
                // Here is where NIP-98 signing is injected into the git http-push call
                println!("ok {}", s.strip_prefix("push ").unwrap());
                println!();
            }
            _ => println!(),
        }
    }
}
