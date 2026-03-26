use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args[1] != "-c" {
        eprintln!("Interactive access disabled.");
        std::process::exit(1);
    }
    let cmd_parts: Vec<&str> = args[2].split_whitespace().collect();
    match cmd_parts[0] {
        "git-receive-pack" | "git-upload-pack" | "git-upload-archive" => {
            Command::new(cmd_parts[0]).args(&cmd_parts[1..]).status().unwrap();
        }
        _ => {
            eprintln!("Unauthorized command.");
            std::process::exit(1);
        }
    }
}
