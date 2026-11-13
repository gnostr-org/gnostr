use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ffi_dir = manifest_dir.join("../../ffi");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib_path = out_dir.join("libkubo_ffi.a");

    let status = Command::new("go")
        .args(["build", "-buildmode=c-archive", "-o"])
        .arg(&lib_path)
        .arg(".")
        .current_dir(&ffi_dir)
        .status()
        .expect("failed to run go");
    assert!(status.success(), "go build of ffi failed");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=kubo_ffi");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=resolv");
    }
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dylib=dl");

    println!("cargo:rerun-if-changed={}", ffi_dir.display());
}
