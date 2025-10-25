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
    let install_result = Command::new("brew").args(["install", "openssl@3"]).status();
    match install_result {
        Ok(status) if status.success() => {
            println!("cargo:warning=Successfully installed openssl@3 via Homebrew.");
        }
        Ok(status) => {
            println!(
                "cargo:warning=Failed to install openssl@3 via Homebrew (exit code: {}).",
                status
            );
            println!("cargo:warning=Please ensure Homebrew is configured correctly and try installing manually:");
            println!("cargo:warning= brew install openssl@3");
        }
        Err(e) => {
            println!(
                "cargo:warning=Error executing Homebrew: {}. Please ensure Homebrew is installed and in your PATH.",
                e
            );
        }
    }
}

fn install_pkg_config() {
    println!("cargo:warning=Attempting to install pkg-config using Homebrew...");
    let install_result = Command::new("brew")
        .args(["install", "pkg-config"])
        .status();
    match install_result {
        Ok(status) if status.success() => {
            println!("cargo:warning=Successfully installed pkg-config via Homebrew.");
        }
        Ok(status) => {
            println!(
                "cargo:warning=Failed to install pkg-config via Homebrew (exit code: {}).",
                status
            );
            println!("cargo:warning=Please ensure Homebrew is configured correctly and try installing manually:");
            println!("cargo:warning= brew install pkg-config");
        }
        Err(e) => {
            println!(
                "cargo:warning=Error executing Homebrew: {}. Please ensure Homebrew is installed and in your PATH.",
                e
            );
        }
    }
}

fn install_zlib() {
    println!("cargo:warning=Attempting to install zlib using Homebrew...");
    let install_result = Command::new("brew").args(["install", "zlib"]).status();
    match install_result {
        Ok(status) if status.success() => {
            println!("cargo:warning=Successfully installed zlib via Homebrew.");
        }
        Ok(status) => {
            println!(
                "cargo:warning=Failed to install zlib via Homebrew (exit code: {}).",
                status
            );
            println!("cargo:warning=Please ensure Homebrew is configured correctly and try installing manually:");
            println!("cargo:warning= brew install zlib");
        }
        Err(e) => {
            println!(
                "cargo:warning=Error executing Homebrew: {}. Please ensure Homebrew is installed and in your PATH.",
                e
            );
        }
    }
}

fn main() {
    // Tell Cargo to rerun this build script only if these environment variables change
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    match target_arch.as_str() {
        "x86_64" => {
            println!("cargo:warning=Building for x86_64 architecture.");
            println!("cargo:rustc-cfg=target_arch_x86_64");
            println!("cargo:warning=target_os={}", target_os);
        }
        "aarch64" => {
            println!("cargo:warning=Building for aarch64 architecture.");
            println!("cargo:rustc-cfg=target_arch_aarch64");
            println!("cargo:warning=target_os={}", target_os);
        }
        "arm" => {
            println!("cargo:warning=Building for arm architecture.");
            println!("cargo:rustc-cfg=target_arch_arm");
            println!("cargo:warning=target_os={}", target_os);
        }
        "wasm32" => {
            println!("cargo:warning=Building for wasm32 architecture.");
            println!("cargo:rustc-cfg=target_arch_wasm32");
            println!("cargo:warning=target_os={}", target_os);
        }
        "riscv64" => {
            println!("cargo:warning=Building for riscv64 architecture.");
            println!("cargo:rustc-cfg=target_arch_riscv64");
            println!("cargo:warning=target_os={}", target_os);
        }
        _ => {
            println!(
                "cargo:warning=Building for an unknown architecture: {}",
                target_arch
            );
            println!("cargo:warning=target_os={}", target_os);
        }
    }

    if !if_windows() {
        if if_linux_unknown() {
            linux_install_pkg_config();
        }

        if target_os == "aarch64-apple-darwin" || target_os == "x86_64-apple-darwin" {
            println!("cargo:warning=On macOS, openssl@3 is recommended for this crate.");
            if check_brew() {
                println!("cargo:warning=Homebrew detected.");
                install_pkg_config();
                install_zlib();
                install_openssl_brew();

                if target_os == "aarch64-apple-darwin" {
                    println!("cargo:rustc-link-search=native=/opt/homebrew/opt/openssl@3/lib");
                    println!("cargo:rustc-link-lib=dylib=ssl@3");
                    println!("cargo:rustc-link-lib=dylib=crypto@3");
                } else if target_os == "x86_64-apple-darwin" {
                    println!("cargo:rustc-link-search=native=/usr/local/opt/openssl@3/lib");
                    println!("cargo:rustc-link-lib=dylib=ssl@3");
                    println!("cargo:rustc-link-lib=dylib=crypto@3");
                }
            } else {
                println!("cargo:warning=Homebrew not found. Please install openssl@3 manually using Homebrew:");
                println!("cargo:warning= brew install openssl@3");
                println!("cargo:warning= brew install pkg-config");
                println!("cargo:warning= brew install zlib");
                println!("cargo:warning=Or using MacPorts:");
                println!("cargo:warning= sudo port install openssl@3");
                println!("cargo:warning=And ensure your system can find the libraries.");
            }
        } else {
            // For other operating systems, the `openssl` crate should handle linking.
            println!("cargo:rustc-link-lib=dylib=ssl");
            println!("cargo:rustc-link-lib=dylib=crypto");
        }
    }
}

fn if_windows() -> bool {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    if target_os == "windows" {
        println!("cargo:rustc-cfg=target_os_windows");
        println!("cargo:warning=Building for Windows.");
        true
    } else {
        println!(
            "cargo:warning=Not building for Windows (target OS: {}).",
            target_os
        );
        false
    }
}

fn if_linux_unknown() -> bool {
    let target = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    if target == "aarch64-unknown-linux-gnu" {
        println!(
            "cargo:warning=On AArch64 Linux, the `libssl-dev` package is required for this crate."
        );
        println!("cargo:warning=Please ensure you have it installed.");
        println!("cargo:warning=For Debian/Ubuntu-based systems, use:");
        println!("cargo:warning= sudo apt-get update && sudo apt-get install libssl-dev");
        println!("cargo:warning=For Fedora/CentOS/RHEL-based systems, use:");
        println!("cargo:warning= sudo dnf install openssl-devel"); // Package name might vary
        println!("cargo:warning=Or:");
        println!("cargo:warning= sudo yum install openssl-devel"); // Older systems
        println!("cargo:warning=For Arch Linux-based systems, use:");
        println!("cargo:warning= sudo pacman -S openssl"); // Development headers are usually included
                                                          // Optionally, you can try to check if the necessary libraries exist
                                                          // This is more reliable than trying to run package managers
        let check_libssl = Command::new("ldconfig").arg("-p").output();
        match check_libssl {
            Ok(output)
                if String::from_utf8_lossy(&output.stdout).contains("libssl.so")
                    && String::from_utf8_lossy(&output.stdout).contains("libcrypto.so") =>
            {
                println!("cargo:rustc-link-lib=dylib=ssl");
                println!("cargo:rustc-link-lib=dylib=crypto");
            }
            _ => {
                println!("cargo:warning=Could not find `libssl.so` and `libcrypto.so`. Ensure `libssl-dev` (or equivalent) is installed correctly.");
            }
        }
        true
    } else {
        println!("cargo:rustc-link-lib=dylib=ssl");
        println!("cargo:rustc-link-lib=dylib=crypto");
        false
    }
}

fn linux_install_pkg_config() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    if target_os == "linux" {
        println!("cargo:warning=On Linux, the `pkgconfig` package (or equivalent providing `pkg-config`) is required for this crate.");
        println!("cargo:warning=Please ensure you have it installed.");
        println!("cargo:warning=For Debian/Ubuntu-based systems, use:");
        println!("cargo:warning= sudo apt-get update && sudo apt-get install pkgconfig");
        println!("cargo:warning=For Fedora/CentOS/RHEL-based systems, use:");
        println!("cargo:warning= sudo dnf install pkg-config");
        println!("cargo:warning=Or:");
        println!("cargo:warning= sudo yum install pkg-config"); // Older systems
        println!("cargo:warning=For Arch Linux-based systems, use:");
        println!("cargo:warning= sudo pacman -S pkg-config"); // Development headers are usually included
        println!("cargo:warning=For other distributions, please consult your package manager.");

        let pkg_config_check = Command::new("which") // Or `command -v`
            .arg("pkg-config")
            .output();

        match pkg_config_check {
            Ok(output) if output.status.success() => {
                println!("cargo:warning=Found `pkg-config` in your PATH.");
                let config_output = Command::new("pkg-config").arg("--libs").output().unwrap();
                println!(
                    "cargo:rustc-link-lib={}",
                    String::from_utf8_lossy(&config_output.stdout).trim()
                );
            }
            _ => {
                println!("cargo:warning=`pkg-config` not found in your PATH. Ensure `pkg-config` (or equivalent) is installed and accessible.");
            }
        }
    }
}

fn musl_install_pkg_config() {
    let target = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    if target == "x86_64-unknown-linux-musl" {
        println!("cargo:warning=Building for x86_64-unknown-linux-musl (Musl libc).");
        println!("cargo:warning=This build process may require `pkg-config` to locate necessary libraries.");
        println!(
            "cargo:warning=Please ensure `pkg-config` is installed in your Musl-based environment."
        );
        println!("cargo:warning=For example, if you are using Alpine Linux, you might use:");
        println!("cargo:warning= apk add pkgconf"); // Alpine uses 'apk' and 'pkgconf'

        let pkg_config_check = Command::new("which") // Or `command -v`
            .arg("pkg-config")
            .output();

        match pkg_config_check {
            Ok(output) if output.status.success() => {
                println!("cargo:warning=Found `pkg-config` in your PATH.");
                let lib_info = Command::new("pkg-config")
                    .arg("--libs")
                    .arg("your_library")
                    .output()
                    .unwrap();
                println!(
                    "cargo:rustc-link-lib={}",
                    String::from_utf8_lossy(&lib_info.stdout).trim()
                );
            }
            _ => {
                println!("cargo:warning=`pkg-config` not found in your PATH. Please ensure it is installed and accessible.");
            }
        }
        println!("cargo:rustc-cfg=target_musl");
    }
}
