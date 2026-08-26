use gnostr_qr::*;

fn main() {
    use std::env;
    let _package_name = env!("CARGO_PKG_NAME");
    let _crate_name = env!("CARGO_CRATE_NAME");
    let _version = env!("CARGO_PKG_VERSION");
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    if args.len() == 1 {
        for arg in &args {
            if arg == "-p" || arg == "--png" {
                help();
                std::process::exit(0);
            }
            if arg == "-h" || arg == "--help" {
                help();
            }
            if arg == "-v" || arg == "--version" {
                version();
            }
            gnostr_qr::render(&arg, true);
            println!("{}", &arg);
            std::process::exit(0);
        }
        std::process::exit(0);
    }
    if args.len() > 1 {
        let mut next: bool = false;
        for arg in &args {
            if arg == "-h" || arg == "--help" {
                help();
                std::process::exit(0);
            }
            if arg == "-v" || arg == "--version" {
                version();
                std::process::exit(0);
            }
            if arg == "-p" || arg == "--png" {
                next = true;
            } else if next {
                if arg.len() < 1 {
                    gnostr_qr::help();
                }
                gnostr_qr::render(&arg, true);
                println!("{}", &arg);
            } else {
                gnostr_qr::render(&arg, false);
                println!("{}", &arg);
            }
        }
        std::process::exit(0);
    } else {
        help();
        std::process::exit(0);
    }
}
