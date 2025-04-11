use std::env;
use std::process::Command;

fn check_brew() -> bool {
    Command::new("brew")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn install_openssl_brew() {
    println!("cargo:warning=Attempting to install openssl@3 using Homebrew...");
    let install_result = Command::new("brew")
        .args(&["install", "openssl@3"])
        .status();

    match install_result {
        Ok(status) if status.success() => {
            println!("cargo:warning=Successfully installed openssl@3 via Homebrew.");
            // Instruct rustc to link against the OpenSSL libraries installed by Brew.
            // The exact paths might vary slightly based on Brew's configuration.
            // It's generally safer to rely on the `openssl` crate to handle linking.
            // However, if you need explicit linking:
            // println!("cargo:rustc-link-search=native=/opt/homebrew/opt/openssl@3/lib"); // For Apple Silicon
            // println!("cargo:rustc-link-search=native=/usr/local/opt/openssl@3/lib");   // For Intel
            // println!("cargo:rustc-link-lib=dylib=ssl@3");
            // println!("cargo:rustc-link-lib=dylib=crypto@3");
        }
        Ok(status) => {
            println!(
                "cargo:warning=Failed to install openssl@3 via Homebrew (exit code: {}).",
                status
            );
            println!("cargo:warning=Please ensure Homebrew is configured correctly and try installing manually:");
            println!("cargo:warning=  brew install openssl@3");
        }
        Err(e) => {
            println!(
                "cargo:warning=Error executing Homebrew: {}. Please ensure Homebrew is installed and in your PATH.",
                e
            );
        }
    }
}


fn fetch_all(){
    Command::new("git")
        .args(&["fetch", "--all"])
        .status()
        .unwrap();
}
    fn fetch_all_tags(){
    Command::new("git")
        .args(&["fetch", "--all", "--tags", "--force"])
        .status()
        .unwrap();
}



fn update_submodules(){
     Command::new("git")
         .args(&[
             "submodule",
             "update",
             "--init",
             "--recursive",
         ])
         .status()
         .unwrap();
}

fn main() {

    update_submodules();

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "macos" {
        println!("cargo:warning=On macOS, openssl@3 is recommended for this crate.");

        if check_brew() {
            println!("cargo:warning=Homebrew detected.");
            install_openssl_brew();
        } else {
            println!("cargo:warning=Homebrew not found. Please install openssl@3 manually using Homebrew:");
            println!("cargo:warning=  brew install openssl@3");
            println!("cargo:warning=Or using MacPorts:");
            println!("cargo:warning=  sudo port install openssl@3");
        }

        // Instruct rustc to link against the OpenSSL libraries.
        // The `openssl` crate generally handles finding these libraries.
        // Ensure you have the `openssl` crate as a dependency in your Cargo.toml.
        println!("cargo:rustc-link-lib=dylib=ssl");
        println!("cargo:rustc-link-lib=dylib=crypto");
    } else {
        // For other operating systems, you might have different dependencies or approaches
        println!("cargo:rustc-link-lib=dylib=ssl");
        println!("cargo:rustc-link-lib=dylib=crypto");
    }

    // Add other build logic here if needed
}

