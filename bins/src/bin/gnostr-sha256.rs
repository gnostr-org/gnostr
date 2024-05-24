use std::io::Result;
use std::{env, process};

//time functions
extern crate chrono;
extern crate time;
#[cfg(debug_assertions)]
use std::path::PathBuf;
#[cfg(not(debug_assertions))]
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
#[allow(unused_imports)]
use gnostr_bins::run;
#[allow(unused_imports)]
use gnostr_bins::Config;

//main.rs functions
fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
#[cfg(debug_assertions)]
fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}
#[cfg(not(debug_assertions))]
#[allow(dead_code)]
fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}

#[allow(unused)] //remove later
#[allow(dead_code)]
fn strip_trailing_newline(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix("\n"))
        .unwrap_or(input)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let _appname = &args[0];
    //catch empty query first
    if args.len() == 1 {
        use sha256::digest;
        let query = digest("");
        print!("{}", query);
        process::exit(0);
    }

    let start = time::get_time();
    let _epoch = get_epoch_ms();
    let _system_time = SystemTime::now();
    let _datetime: DateTime<Utc> = _system_time.into();

    //let version = env!("CARGO_PKG_VERSION");
    //let name = env!("CARGO_PKG_NAME");
    //let crate_name = env!("CARGO_CRATE_NAME");
    //let author = env!("CARGO_PKG_AUTHORS");

    //println!("Program Name: {}", name);
    //println!("Crate Name: {}", crate_name.replace("_","-"));
    //println!("Program Version: {}", version);
    //println!("Program Autor: {}", author);

    #[cfg(debug_assertions)]
    let cwd = get_current_working_dir();
    #[cfg(debug_assertions)]
    println!("cwd={:#?}", cwd);

    if args[1] == "-h" || args[1] == "--help" {
        let crate_name = env!("CARGO_CRATE_NAME");
        print!("{}", crate_name.replace("_", "-"));
        print!("           gnostr-sha256 <file_path>\n");
        process::exit(0);
    }
    if args[1] == "-v" || args[1] == "--version" {
        print!("{}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    let config = gnostr_bins::Config::build(&args).unwrap_or_else(|_err| {
        println!("Usage: gnostr-sha256 <string>");
        process::exit(0);
    });

    //println!("{}", strip_trailing_newline(&config.query));
    //println!("{}", config.query);

    if let Err(e) = gnostr_bins::run(config) {
        println!("Application error: {e}");
        process::exit(1);
    }

    let _duration = time::get_time() - start;

    if cfg!(debug_assertions) {

        //#[cfg(not(debug_assertions))]
        //println!("Debugging disabled");
        //println!("start={:?}", start);
        //println!("_duration={:?}", _duration);
    } else {

        //#[cfg(debug_assertions)]
        //println!("Debugging enabled");
        //println!("start={:?}", start);
        //println!("_duration={:?}", _duration);
    }

    Ok(())
} //end main

#[cfg(test)]
mod tests {
    use sha256::digest;

    use super::*;

    #[test]
    fn strip_newline_works() {
        assert_eq!(strip_trailing_newline("Test0\r\n\r\n"), "Test0\r\n");
        assert_eq!(strip_trailing_newline("Test1\r\n"), "Test1");
        assert_eq!(strip_trailing_newline("Test2\n"), "Test2");
        assert_eq!(strip_trailing_newline("Test3"), "Test3");
    }

    #[test]
    fn empty_query() {
        let query = digest("");
        let contents = "\
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(strip_trailing_newline(&query), contents);
    }

    #[test]
    fn hello_query() {
        let query = digest("hello");
        let contents = "\
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert_eq!(strip_trailing_newline(&query), contents);
    }

    #[test]
    fn raw_byte_hello_query() {
        let query = digest(r#"hello"#);
        //let query = digest("hello");
        let contents = "\
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert_eq!(strip_trailing_newline(&query), contents);
    }

    #[test]
    fn byte_str_hello_query() {
        let query = digest(b"hello");
        let contents = "\
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert_eq!(strip_trailing_newline(&query), contents);
    }

    #[test]
    fn byte_query() {
        let query = digest(b"h");
        let contents = "\
aaa9402664f1a41f40ebbc52c9993eb66aeb366602958fdfaa283b71e64db123";
        assert_eq!(strip_trailing_newline(&query), contents);
    }

    #[test]
    fn raw_byte_query() {
        let query = digest(br#"hello"#);
        let contents = "\
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert_eq!(strip_trailing_newline(&query), contents);
    }

    #[test]
    #[should_panic]
    fn hello_panic_query() {
        let query = digest(r#"hello\n"#);
        let contents = "\
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824 ";
        assert_eq!(strip_trailing_newline(&query), contents);
    }

    //REF:shell test
    //$ 0 2>/dev/null | sha256sum | sed 's/-//g'
    #[test]
    #[should_panic]
    fn panic_query() {
        let query = digest("");
        let contents = "\
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 ";

        assert_eq!(strip_trailing_newline(&query), contents);
    }
}
