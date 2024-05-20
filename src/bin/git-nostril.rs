 use std::process::Command;
 use std::io;
use std::{env, fs};


use std::env::args;


use include_dir::{include_dir, Dir};
//use std::path::Path;
use markdown::to_html;

//static PROJECT_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR");

fn main() -> io::Result<()> {


    let args_vec: Vec<String> = env::args().collect();

    let app = &args_vec[0];
    let sec = &args_vec[1];
    let private_key = &args_vec[2];

    println!("app={}", app);
    println!("sec={}", sec);
    println!("private_key={}", private_key);


  let mut args_string = String::new();

  //Need push sequence here to format string correctly
  for arg in args().skip(1) {
  //print!("{}",arg);
    args_string.push_str(&arg);
    args_string.push(' ');
  }
  // Remove trailing space (if any)
  args_string.pop();

  //print!("{:?}",Some(args()));
  //print!("\n{:}\n",format!("{:?}",args()));
  //print!("{}\n",format!("{:?}",args_vec[0]));
  //print!("{}\n",format!("{:?}",args_vec[1]));
  //print!("{}\n",format!("{:?}",args_vec[2]));
  //print!("{:}\n",format!("{:?}",args_string));
  //print!("{:}\n",format!("{:?}",&args_string.replace("\"","")));
  //print!("{}\n",args_string);


    //let _out_dir = env::var("OUT_DIR").unwrap();
    //
    let which_nostril = Command::new("which")
        .arg("nostril")
        .output()
        .expect("failed to execute process");


    let nostril = String::from_utf8(which_nostril.stdout)
    .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
    .unwrap();
    println!("{}", nostril);

    let event = Command::new("nostril")
        //.arg("--sec")
        .args(&[
            "--sec",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ])
        //.args(&mut args())
        //.arg(args_vec[1])
        //.arg(args_vec[2])
        //.arg(private_key)
        //.arg(&args_string)
        .output()
        .expect("failed to execute process");


    let nostril_event = String::from_utf8(event.stdout)
    .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
    .unwrap();


    println!("{}", nostril_event);
    Ok(())

}
