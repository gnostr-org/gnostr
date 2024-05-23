//static PROJECT_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR");
use std::env;
use std::path::Path;
use std::process::Command; //, fs};

//use which::which;
//use include_dir::{include_dir};//, Dir};
//use std::path::Path;
//use markdown::to_html;

fn main() -> std::io::Result<()> {
    let _out_dir = env::var("OUT_DIR").unwrap();

    if Path::new("nostril.c").exists() {
        println!("cargo:rerun-if-changed=nostril.c");
    }
    if !Path::new("nostril").exists() {
        println!("cargo:rerun-if-changed=nostril");
    }
    if Path::new("nostril").exists() {
        println!("cargo:rerun-if-changed=nostril");
    }
    if !Path::new("/usr/local/bin/nostril").exists() {
        println!("cargo:rerun-if-changed=nostril");
    }
    if Path::new("/usr/local/bin/nostril").exists() {
        println!("cargo:rerun-if-changed=nostril");
    }

    if Path::new(".git/HEAD").exists() {
        println!("cargo:rerun-if-changed=.git/HEAD");
    }
    if Path::new("../.git/HEAD").exists() {
        println!("cargo:rerun-if-changed=../.git/HEAD");
    }

    // let library_path = Path::new("libsecp256k1.a");
    // cc::Build::new()
    //
    //     .file("nostril.c")
    //     .include(library_path)
    //     .include("deps/secp256k1/include/secp256k1.h")
    //     .compile("nostril");

    Command::new("git")
        .args(&["submodule", "update", "--init", "--recursive"])
        .spawn()
        //.status()
        .unwrap();
    Command::new("git")
        .args(&[
            "remote",
            "add",
            "randymcmillan/nostril",
            "git@github.com:randymcmillan/nostril.git",
        ])
        .spawn()
        //.status()
        .unwrap();
    Command::new("git")
        .args(&[
            "remote",
            "add",
            "jb55/nostril",
            "git@github.com:jb55/nostril.git",
        ])
        .spawn()
        //.status()
        .unwrap();
    Command::new("git")
        .args(&["fetch", "--all"])
        .spawn()
        //.status()
        .unwrap();
    Command::new("git")
        .args(&["fetch", "--all", "--tags", "--force"])
        .spawn()
        //.status()
        .unwrap();
    Command::new("cmake").args(&["`pwd`"]).status().unwrap();
    Command::new("make").args(&["&"]).status().unwrap();
    //Command::new("make")
    //    .args(&["nostril"])
    //    .status()
    //    .unwrap();
    //Command::new("make")
    //    .args(&["install"])
    //    .status()
    //    .unwrap();

    let script_name = "./script.sh";

    // Build the command
    let mut _command = Command::new(script_name);

    // Add arguments if needed (optional)
    // command.arg("argument1");
    // command.arg("argument2");

    Command::new(script_name)
        .current_dir(".")
        .spawn()
        .expect("script.sh command failed to start");
    Ok(())
}
