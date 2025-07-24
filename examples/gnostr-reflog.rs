extern crate getopts;
use std::{env, process};

use getopts::Options;
use gnostr::{get_pwd, hash_list, hash_list_padded, hash_list_w_commit_message};

mod std_input {

    extern crate getopts;
    use std::{env, io, process};

    use ascii::AsciiChar;
    use getopts::Options;

    #[allow(dead_code)]
    pub fn parse_input() {
        let args: Vec<String> = env::args().collect();
        let _program = args[0].clone();

        let mut opts = Options::new();
        opts.optopt("o", "output", "set output file name", "NAME");
        opts.optopt(
            "r",
            "ref",
            "Specify the Git reference (default: HEAD)",
            "REF",
        );
        opts.optopt(
            "n",
            "number",
            "Specify the maximum number of commits to show (default: 10)",
            "NUMBER",
        );

        opts.optopt("s", "sec", "use following privkey", "SEC");

        opts.optflag("h", "help", "print this help menu");
        opts.optflag("m", "messages", "print reflog with commit messages");

        let mut input = String::new();
        //capture input from prompt
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read line");

        //check if hex
        let mut count = 0;
        #[allow(unused_mut)]
        let mut key_maybe = true;
        //REF: https://docs.rs/ascii/latest/ascii/enum.AsciiChar.html#method.is_ascii_hexdigit
        for (_i, c) in input.trim().chars().enumerate() {
            if c.is_ascii() {
                //TODO:more validation
                assert!(ascii::AsciiChar::Space.is_ascii_blank());
                assert!(ascii::AsciiChar::Space.is_ascii_blank());
                assert!(ascii::AsciiChar::Tab.is_ascii_blank());
                assert!(!ascii::AsciiChar::VT.is_ascii_blank());
                assert!(!ascii::AsciiChar::LineFeed.is_ascii_blank());
                assert!(!ascii::AsciiChar::CarriageReturn.is_ascii_blank());
                assert!(!ascii::AsciiChar::FF.is_ascii_blank());

                assert!(AsciiChar::Space.is_ascii_blank());
                assert!(AsciiChar::Tab.is_ascii_blank());
                assert!(!AsciiChar::VT.is_ascii_blank());
                assert!(!AsciiChar::LineFeed.is_ascii_blank());
                assert!(!AsciiChar::CarriageReturn.is_ascii_blank());
                assert!(!AsciiChar::FF.is_ascii_blank());

                #[cfg(debug_assertions)]
                if c.is_ascii_control() {
                    println!("{}.is_asscii_control()={}", c, c.is_ascii_control());
                }
                #[cfg(debug_assertions)]
                if c.is_ascii() {
                    println!("{}.is_asscii()={}", c, c.is_ascii());
                }
                #[cfg(debug_assertions)]
                if c.is_ascii_graphic() {
                    println!("{}.is_asscii_graphic()={}", c, c.is_ascii_graphic());
                }
                #[cfg(debug_assertions)]
                if c.is_whitespace() {
                    println!("{}.is_whitespace()={}", c, c.is_whitespace());
                }
                #[cfg(debug_assertions)]
                if c.is_ascii_whitespace() {
                    println!("{}.is_ascii_whitespace()={}", c, c.is_ascii_whitespace());
                }
                #[cfg(debug_assertions)]
                if !c.is_ascii_hexdigit() {
                    key_maybe = false;
                }
                #[cfg(debug_assertions)]
                println!("{}:{}={}", _i, c, c.is_ascii_hexdigit());
                #[cfg(debug_assertions)]
                println!("{}", c);
                count = count + 1;
            } //end is_ascii
        } //end for loop

        #[cfg(debug_assertions)]
        println!("count={:?}", count);
        #[cfg(debug_assertions)]
        println!("key_maybe={:?}", key_maybe);

        //we assume the input is a key
        //we assume it is a privkey for now
        if count == 64 && key_maybe == true {
            println!("handle key");
        } else {
            //println!("test for sub commands");
            if input.trim() == "input" {
                println!("input={}", input);
            }
            if input.trim() == "install" {
                println!("input={}", input);
            }
            if input.contains("PRIVKEY") {
                println!("PRIVKEY={}", input);
            }
            if input.trim() == "PRIVKEY" {
                println!("PRIVKEY={}", input);
            }
            process::exit(0);
        } //end else
    } //end if args.len() == 1
} //end parse_input

pub fn print_usage(_program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options]", _program);
    print!("{}", opts.usage(&brief));
    process::exit(0);
}
pub fn sec(_program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options]", _program);
    print!("sec:\n{}", opts.usage(&brief));
    process::exit(0);
}
pub fn hash(_program: &str, opts: &Options) {
    //hash a following value
    //TODO detect stream or file
    let brief = format!("Usage: {} [options]", _program);
    print!("hash:\n{}", opts.usage(&brief));
    process::exit(0);
}

pub fn main() -> Result<(), git2::Error> {
    let _this_pwd = get_pwd();
    //println!("this_pwd={}", this_pwd.unwrap());

    //gnostr::hash_list();
    //process::exit(0);
    // COMMAND CONTEXT:
    // for m in $(gnostr-reflog -p);do echo $m; for n in $(gnostr-reflog);do echo
    // $n;done;done for m in $(gnostr-reflog -p); do gnostr --sec  $m --content
    // "$(for n in $(gnostr-reflog); do echo $n;done)";done
    // for m in $(gnostr-reflog -p); do gnostr --sec  $m --content "$(for n in
    // $(gnostr-reflog); do echo $n;done)";done for m in $(gnostr-reflog -p); do
    // gnostr --sec  $m --content "$(for n in $(gnostr-reflog); do echo $n;done)" |
    // gnostr-xq ;done for m in $(gnostr-reflog -p); do gnostr --sec  $m
    // --content "$(for n in $(gnostr-reflog); do echo $n;done)" | gnostr-post-event
    // --relay wss://relay.damus.io ;done

    let args: Vec<String> = env::args().collect();
    let _program = args[0].clone();
    //REF: https://docs.rs/getopts/latest/getopts/struct.Options.html
    let mut opts = Options::new();

    opts.optflag("h", "help", "print this help menu");
    opts.optflag("p", "padded", "padded commit hashes");
    opts.optflag("m", "msgs", "print reflog with commit messages");

    if args.len() >= 1 {
        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                println!("Error: {}", f.to_string());
                panic!("{}", f.to_string())
            }
        };
        if matches.opt_present("h") {
            print_usage(&_program, &opts);
            process::exit(0);
        }
        if matches.opt_present("p") {
            hash_list_padded();
            process::exit(0);
        }
        if matches.opt_present("m") {
            hash_list_w_commit_message();
            process::exit(0);
        } else {
            hash_list();
            process::exit(0);
        }
    };

    Ok(())
}
