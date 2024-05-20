 use std::process::Command;
 use std::io;
use std::{env, fs};

use include_dir::{include_dir, Dir};
//use std::path::Path;
use markdown::to_html;

//static PROJECT_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR");

fn main() -> io::Result<()> {
    //let _out_dir = env::var("OUT_DIR").unwrap();

    let event = Command::new("nostril")
        .args(&[
            "--sec",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ])
            .output()
            .expect("failed to execute process");


    let nostril_event = String::from_utf8(event.stdout)
    .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
    .unwrap();


    println!("{}", nostril_event);
    Ok(())

}
