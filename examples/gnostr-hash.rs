use gnostr::hash;
use std::io::Read;
use std::{env, process};
fn main() {
    let args: Vec<String> = env::args().collect();

    for i in 0..args.len() {
        if i == args.len() {
            process::exit(i.try_into().unwrap());
        } else {
            if args.len() == 0 {
                print!("args.len() = {}", 0);
            };
            if args.len() == 1 {
                let mut s: String = String::new();
                std::io::stdin().read_to_string(&mut s).unwrap();
                let res = hash::hash(&s);
                print!("{}", res.unwrap());
                std::process::exit(0);
            };
            if args.len() == 2 {
                let res = hash::hash(&args[1]);
                print!("{}", res.unwrap());
                std::process::exit(0);
            };
        }
    }
}
