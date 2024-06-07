use std::env;
use std::io;
use std::process::Command;

//use std::env::args;

//use include_dir::{include_dir, Dir};
//use std::path::Path;
//use markdown::to_html;

//static PROJECT_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR");

fn help() {
    let event = Command::new("nostril")
        .arg("")
        .output()
        .expect("failed to execute process");

    let nostril_event = String::from_utf8(event.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();

    println!("{}", nostril_event);
    std::process::exit(0);
}

fn main() -> io::Result<()> {
    let args_vec: Vec<String> = env::args().collect();
    if args_vec.len() < 3 {
        help();
    }

    //capture git-nostril --sec <private_key>
    let mut _app: &String = &("").to_string();
    let mut _sec: &String = &("").to_string();
    let mut _private_key: &String = &("").to_string();
    _app = &args_vec[0]; //gnostr
    _sec = &args_vec[1]; //--sec
                         //println!("_app={}", &_app);
                         //println!("_sec={}", &_sec);

    if args_vec.len() >= 3 {
        _private_key = &args_vec[2];
    }
    //println!("_private_key={}", &_private_key);

    //skip gnostr --sec <private_key>
    //and capture everything else
    let args: Vec<String> = env::args().skip(1).collect();
    //print!("args={:}", format!("{:?}",&args));

    //let args = args.join(" ");  // Join with space as separator

    //println!("{}", args); // Output: hello world

    let which_nostril = Command::new("which")
        .arg("nostril")
        .output()
        .expect("failed to execute process");
    let _nostril = String::from_utf8(which_nostril.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();

    //print!("{}",_nostril);

    let event = Command::new("nostril")
        .args(&args)
        .output()
        .expect("failed to execute process");

    let nostril_event = String::from_utf8(event.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();

    println!("{}", nostril_event);
    Ok(())
}
