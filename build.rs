use std::env;
use std::fs;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use std::process::Command;

// try:
// cargo build --features memory_profiling -j8

fn check_sscache() {
    if Command::new("sccache").arg("--version").output().is_ok() {
        println!("cargo:warning=sscache detected, setting RUSTC_WRAPPER.");
        env::set_var("RUSTC_WRAPPER", "sscache");
        println!("cargo:rerun-if-env-changed=RUSTC_WRAPPER");
    } else {
        println!("cargo:warning=sscache not found - trying to install.");
        install_sccache();
    }
}

fn check_brew() -> bool {
    Command::new("brew")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn install_sccache() {
    // Check if the target is a Linux environment.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "linux" {
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:warning=Detected Linux OS. Attempting to install sccache.");

        let output = Command::new("sh")
            .arg("-c")
            .arg("if command -v apt-get &> /dev/null; then sudo apt-get update && sudo apt-get install -y sscache; else echo 'apt-get not found, trying yum'; if command -v yum &> /dev/null; then sudo yum install -y sccache; else echo 'Neither apt-get nor yum found. Please install sccache manually.'; fi; fi")
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("cargo:warning=Failed to install dependencies: {}", stderr);
                    // Exit the build process with an error
                    panic!("Failed to install required Linux dependencies.");
                } else {
                    println!("cargo:warning=Successfully installed xcb dependencies.");
                }
            }
            Err(e) => {
                println!(
                    "cargo:warning=Failed to run dependency installation command: {}",
                    e
                );
                // Exit the build process with an error
                panic!("Failed to run dependency installation command.");
            }
        }
    }
    if target_os == "macos" {
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:warning=Detected macOS. Attempting to install 'sccache' using Homebrew.");

        // We use a shell command to first check if 'brew' is installed,
        // and then run the installation command.
        let output = Command::new("sh")
            .arg("-c")
            .arg("if command -v brew >/dev/null 2>&1; then brew install sccache; else echo 'Homebrew is not installed. Please install Homebrew at https://brew.sh to continue.'; exit 1; fi")
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("cargo:warning=Failed to install dependencies: {}", stderr);
                    // Exit the build process with a panick, since the build cannot continue.
                    panic!("Failed to install required macOS dependencies.");
                } else {
                    println!("cargo:warning=Successfully installed sccache dependency.");
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to run Homebrew command: {}", e);
                // Exit the build process with a panick.
                panic!("Failed to run Homebrew command.");
            }
        }
    }
    if target_os == "windows" {
        println!("cargo:warning=Detected Windows. Trying to install sccache.");
        install_windows_dependency("sccache", "scoop install sccache");
        println!("cargo:rerun-if-changed=build.rs");

        println!("cargo:warning=Detected Windows. No XCB libraries are required for this build.");
    }
}
fn install_windows_dependency(name: &str, install_command: &str) {
    // Check if the dependency is already installed using the Windows 'where' command.
    let check_command = format!("where.exe {} >nul 2>nul", name);

    // Command::new("cmd") is the standard way to run shell commands on Windows.
    let output = Command::new("cmd").arg("/C").arg(&check_command).status();

    match output {
        Ok(status) => {
            if status.success() {
                println!("cargo:warning=Dependency '{}' already found.", name);
                return;
            }
        }
        Err(e) => {
            // A non-zero exit from the 'where.exe' check is expected if the command isn't found,
            // but a generic error here means 'cmd' itself couldn't run.
            println!("cargo:warning=Failed to check for '{}': {}", name, e);
        }
    }

    // Dependency not found (or check failed), proceed with installation.
    println!(
        "cargo:warning=Attempting to install dependency '{}' using: {}",
        name, install_command
    );

    let output = Command::new("cmd")
        .arg("/C") // Run the command string and then terminate
        .arg(install_command)
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);

                println!("cargo:warning=Failed to install {}: {}", name, stderr);
                println!("cargo:warning=Stdout: {}", stdout);

                // Exit the build process with a panick, since the build cannot continue.
                panic!(
                    "Failed to install required Windows dependency: {}. Ensure Scoop or Winget is installed and on your PATH.",
                    name
                );
            } else {
                println!("cargo:warning=Successfully installed {} dependency.", name);
            }
        }
        Err(e) => {
            println!(
                "cargo:warning=Failed to run installation command for {}: {}",
                name, e
            );
            // Exit the build process with a panick.
            panic!("Failed to run installation command for {}.", name);
        }
    }
}

fn install_xcb_deps() {
    // Check if the target is a Linux environment.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "linux" {
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:warning=Detected Linux OS. Attempting to install xcb dependencies.");

        let output = Command::new("sh")
            .arg("-c")
            .arg("if command -v apt-get &> /dev/null; then sudo apt-get update && sudo apt-get install -y libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev; else echo 'apt-get not found, trying yum'; if command -v yum &> /dev/null; then sudo yum install -y libxcb libxcb-devel libxcb-render-devel libxcb-shape-devel libxcb-xfixes-devel; else echo 'Neither apt-get nor yum found. Please install libxcb development libraries manually.'; fi; fi")
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("cargo:warning=Failed to install dependencies: {}", stderr);
                    // Exit the build process with an error
                    panic!("Failed to install required Linux dependencies.");
                } else {
                    println!("cargo:warning=Successfully installed xcb dependencies.");
                }
            }
            Err(e) => {
                println!(
                    "cargo:warning=Failed to run dependency installation command: {}",
                    e
                );
                // Exit the build process with an error
                panic!("Failed to run dependency installation command.");
            }
        }
    }
    if target_os == "macos" {
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:warning=Detected macOS. Attempting to install 'libxcb' using Homebrew.");

        // We use a shell command to first check if 'brew' is installed,
        // and then run the installation command.
        let output = Command::new("sh")
            .arg("-c")
            .arg("if command -v brew >/dev/null 2>&1; then brew install libxcb; else echo 'Homebrew is not installed. Please install Homebrew at https://brew.sh to continue.'; exit 1; fi")
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("cargo:warning=Failed to install dependencies: {}", stderr);
                    // Exit the build process with a panick, since the build cannot continue.
                    panic!("Failed to install required macOS dependencies.");
                } else {
                    println!("cargo:warning=Successfully installed xcb dependencies.");
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to run Homebrew command: {}", e);
                // Exit the build process with a panick.
                panic!("Failed to run Homebrew command.");
            }
        }
    }
    if target_os == "windows" {
        println!("cargo:rerun-if-changed=build.rs");

        // This project uses conditional compilation to handle Windows dependencies.
        // No external package manager is needed for `libxcb` because it's an X11 library.
        // The linker will automatically use the correct platform-specific APIs.
        println!("cargo:warning=Detected Windows. No XCB libraries are required for this build.");
    }
}

fn install_openssl_brew() {
    println!("cargo:warning=Attempting to install openssl@3 using Homebrew...");
    let install_result = Command::new("brew").args(["install", "openssl@3"]).status();

    match install_result {
        Ok(status) if status.success() => {
            println!("cargo:warning=Successfully installed openssl@3 via Homebrew.");
            // Instruct rustc to link against the OpenSSL libraries installed by Brew.
            // The exact paths might vary slightly based on Brew's configuration.
            // It's generally safer to rely on the `openssl` crate to handle linking.
            // However, if you need explicit linking:
            // The corrected paths are used conditionally in the main function.
        }
        Ok(status) => {
            println!(
                "cargo:warning=Failed to install openssl@3 via Homebrew (exit code: {}).",
                status
            );
            println!(
                "cargo:warning=Please ensure Homebrew is configured correctly and try installing manually:"
            );
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
fn install_pkg_config() {
    println!("cargo:warning=Attempting to install pkg-config using Homebrew...");
    let install_result = Command::new("brew")
        .args(["install", "pkg-config"])
        .status();

    match install_result {
        Ok(status) if status.success() => {
            println!("cargo:warning=Successfully installed pkg-config via Homebrew.");
            // Linking will be handled by the `openssl` crate or via pkg-config.
        }
        Ok(status) => {
            println!(
                "cargo:warning=Failed to install pkg-config via Homebrew (exit code: {}).",
                status
            );
            println!(
                "cargo:warning=Please ensure Homebrew is configured correctly and try installing manually:"
            );
            println!("cargo:warning=  brew install pkg-config");
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
            // Linking will be handled via pkg-config.
        }
        Ok(status) => {
            println!(
                "cargo:warning=Failed to install zlib via Homebrew (exit code: {}).",
                status
            );
            println!(
                "cargo:warning=Please ensure Homebrew is configured correctly and try installing manually:"
            );
            println!("cargo:warning=  brew install zlib");
        }
        Err(e) => {
            println!(
                "cargo:warning=Error executing Homebrew: {}. Please ensure Homebrew is installed and in your PATH.",
                e
            );
        }
    }
}

use chrono::TimeZone;

fn get_git_hash() -> String {
    use std::process::Command;

    // Allow builds from `git archive` generated tarballs if output of
    // `git get-tar-commit-id` is set in an env var.
    if let Ok(commit) = std::env::var("BUILD_GIT_COMMIT_ID") {
        return commit[..7].to_string();
    };
    let commit_output = Command::new("git")
        .arg("rev-parse")
        .arg("--short=7")
        .arg("--verify")
        .arg("HEAD")
        .output()
        .expect("Failed to execute git command to get commit hash");
    let commit_string = String::from_utf8_lossy(&commit_output.stdout);

    commit_string.lines().next().unwrap_or("").into()
}

fn main() {

    println!("cargo:rerun-if-changed=src/empty");
    make_empty();

    if env::var("RUSTC_WRAPPER").is_ok() {
        println!("cargo:warning=RUSTC_WRAPPER is already set, skipping sccache check.");
    } else {
        check_sscache();
    }
    // Tell Cargo to rerun this build script only if the Git HEAD or index changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");

    // Tell Cargo to rerun this build script only if these environment variables change
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    println!("cargo:rerun-if-env-changed=GITUI_RELEASE");

    make_empty();

    let now = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(val) => chrono::Local
            .timestamp_opt(val.parse::<i64>().unwrap(), 0)
            .unwrap(),
        Err(_) => chrono::Local::now(),
    };
    let build_date = now.date_naive();

    let build_name = if std::env::var("GITUI_RELEASE").is_ok() {
        format!(
            "{} {} ({})",
            env!("CARGO_PKG_VERSION"),
            build_date,
            get_git_hash()
        )
    } else {
        format!("nightly {} ({})", build_date, get_git_hash())
    };

    println!("cargo:warning=buildname '{}'", build_name);
    println!("cargo:rustc-env=GITUI_BUILD_NAME={}", build_name);

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    match target_arch.as_str() {
        "x86_64" => {
            println!("cargo:warning=Building for x86_64 architecture.");
            println!("cargo:rustc-cfg=target_arch_x86_64");
            // Add x86_64 specific build logic here

            println!("cargo:warning=target_os={}", target_os);
            if target_os == "windows" {}
        }
        "aarch64" => {
            println!("cargo:warning=Building for aarch64 architecture.");
            println!("cargo:rustc-cfg=target_arch_aarch64");
            // Add aarch64 specific build logic here
            println!("cargo:warning=target_os={}", target_os);
            if target_os == "windows" {}
        }
        "arm" => {
            println!("cargo:warning=Building for arm architecture.");
            println!("cargo:rustc-cfg=target_arch_arm");
            // Add arm specific build logic here
            println!("cargo:warning=target_os={}", target_os);
            if target_os == "windows" {}
        }
        "wasm32" => {
            println!("cargo:warning=Building for wasm32 architecture.");
            println!("cargo:rustc-cfg=target_arch_wasm32");
            // Add wasm32 specific build logic here
            println!("cargo:warning=target_os={}", target_os);
            if target_os == "windows" {}
        }
        "riscv64" => {
            println!("cargo:warning=Building for riscv64 architecture.");
            println!("cargo:rustc-cfg=target_arch_riscv64");
            // Add riscv64 specific build logic here
            println!("cargo:warning=target_os={}", target_os);
            if target_os == "windows" {}
        }
        _ => {
            println!(
                "cargo:warning=Building for an unknown architecture: {}",
                target_arch
            );
            // Handle unknown architectures or provide a default
            println!("cargo:warning=target_os={}", target_os);
            if target_os == "windows" {}
        }
    }

    if !if_windows() {
        //try
        musl_install_pkg_config();
        install_xcb_deps();
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

                // Instruct rustc to link against the OpenSSL libraries.
                // The `openssl` crate generally handles finding these libraries.
                // If you need explicit linking (less recommended):
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
                println!(
                    "cargo:warning=Homebrew not found. Please install openssl@3 manually using Homebrew:"
                );
                println!("cargo:warning=  brew install openssl@3");
                println!("cargo:warning=  brew install pkg-config");
                println!("cargo:warning=  brew install zlib");
                println!("cargo:warning=Or using MacPorts:");
                println!("cargo:warning=  sudo port install openssl@3");
                println!("cargo:warning=And ensure your system can find the libraries.");
            }
        } else {
            // For other operating systems, the `openssl` crate should handle linking.
            println!("cargo:rustc-link-lib=dylib=ssl");
            println!("cargo:rustc-link-lib=dylib=crypto");
        }
    }
    // Add other build logic here if needed
}

fn if_windows() -> bool {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    if target_os == "windows" {
        println!("cargo:rustc-cfg=target_os_windows");
        println!("cargo:warning=Building for Windows.");
        // Add Windows-specific build logic here
        // For example, linking against specific Windows libraries
        // println!("cargo:rustc-link-lib=user32");
        true
    } else {
        println!(
            "cargo:warning=Not building for Windows (target OS: {}).",
            target_os
        );
        // Add logic for other operating systems if needed
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
        println!("cargo:warning=  sudo apt-get update && sudo apt-get install libssl-dev");
        println!("cargo:warning=For Fedora/CentOS/RHEL-based systems, use:");
        println!("cargo:warning=  sudo dnf install openssl-devel"); // Package name might vary
        println!("cargo:warning=Or:");
        println!("cargo:warning=  sudo yum install openssl-devel"); // Older systems
        println!("cargo:warning=For Arch Linux-based systems, use:");
        println!("cargo:warning=  sudo pacman -S openssl"); // Development headers are usually included

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
                println!(
                    "cargo:warning=Could not find `libssl.so` and `libcrypto.so`. Ensure `libssl-dev` (or equivalent) is installed correctly."
                );
                // You might choose to fail the build here if it's strictly necessary
                // std::process::exit(1);
            }
        }
        true
    } else {
        // Logic for other target platforms
        println!("cargo:rustc-link-lib=dylib=ssl");
        println!("cargo:rustc-link-lib=dylib=crypto");
        false
    }

    // Add other build logic here if needed
}
fn linux_install_pkg_config() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    if target_os == "linux" {
        println!(
            "cargo:warning=On Linux, the `pkgconfig` package (or equivalent providing `pkg-config`) is required for this crate."
        );
        println!("cargo:warning=Please ensure you have it installed.");
        println!("cargo:warning=For Debian/Ubuntu-based systems, use:");
        println!("cargo:warning=  sudo apt-get update && sudo apt-get install pkgconfig");
        println!("cargo:warning=For Fedora/CentOS/RHEL-based systems, use:");
        println!("cargo:warning=  sudo dnf install pkg-config");
        println!("cargo:warning=Or:");
        println!("cargo:warning=  sudo yum install pkg-config"); // Older systems
        println!("cargo:warning=For Arch Linux-based systems, use:");
        println!("cargo:warning=  sudo pacman -S pkg-config"); // Development headers are usually included
        println!("cargo:warning=For other distributions, please consult your package manager.");

        // Optionally, you can try to find `pkg-config` in the PATH
        let pkg_config_check = Command::new("which") // Or `command -v`
            .arg("pkg-config")
            .output();

        match pkg_config_check {
            Ok(output) if output.status.success() => {
                println!("cargo:warning=Found `pkg-config` in your PATH.");
                // You can now use `pkg-config` to get build information
                // For example:
                let config_output = Command::new("pkg-config").arg("--libs").output().unwrap();
                println!(
                    "cargo:rustc-link-lib={}",
                    String::from_utf8_lossy(&config_output.stdout).trim()
                );
            }
            _ => {
                println!(
                    "cargo:warning=`pkg-config` not found in your PATH. Ensure `pkg-config` (or equivalent) is installed and accessible."
                );
                // You might choose to fail the build here if it's strictly necessary
                // std::process::exit(1);
            }
        }
    } else {
        // Logic for other operating systems
    }

    // Common build logic can go here
}

fn musl_install_pkg_config() {
    let target = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");

    if target == "x86_64-unknown-linux-musl" {
        println!("cargo:warning=Building for x86_64-unknown-linux-musl (Musl libc).");
        println!(
            "cargo:warning=This build process may require `pkg-config` to locate necessary libraries."
        );
        println!(
            "cargo:warning=Please ensure `pkg-config` is installed in your Musl-based environment."
        );

        // How to install pkg-config in a typical Musl environment might vary.
        // You might need to instruct the user to install it via their
        // chosen base image or package manager within that environment.
        println!("cargo:warning=For example, if you are using Alpine Linux, you might use:");
        println!("cargo:warning=  apk add pkgconf"); // Alpine uses 'apk' and 'pkgconf'

        // Optionally, you can check if `pkg-config` is in the PATH
        let pkg_config_check = Command::new("which") // Or `command -v`
            .arg("pkg-config")
            .output();

        match pkg_config_check {
            Ok(output) if output.status.success() => {
                println!("cargo:warning=Found `pkg-config` in your PATH.");
                // Now you can use `pkg-config` to get information about libraries
                // For example:
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
                println!(
                    "cargo:warning=`pkg-config` not found in your PATH. Please ensure it is installed and accessible."
                );
                // You might choose to fail the build here if `pkg-config` is strictly necessary
                // std::process::exit(1);
            }
        }

        println!("cargo:rustc-cfg=target_musl");
    } else {
        println!(
            "cargo:warning=Not building for x86_64-unknown-linux-musl (current target: {}).",
            target
        );
        // Logic for other target platforms
    }

    // Common build logic can go here
}

fn make_empty() {
    let target_path = Path::new("src/empty");

    // 1. Clean up the target path if it exists as a FILE or a DIRECTORY.
    // We try to remove the path as a file first, and if that fails, we try as a directory.
    if target_path.exists() {
        if target_path.is_file() {
            println!("cargo:warning=Found file at target path. Removing src/empty.");
            if let Err(e) = fs::remove_file(target_path) {
                panic!(
                    "Failed to remove existing file {}: {}",
                    target_path.display(),
                    e
                );
            }
        } else if target_path.is_dir() {
            // If it exists as a directory, we can skip removal, as create_dir_all is idempotent,
            // but if the goal is absolute cleanup, we use remove_dir_all.
            // For simplicity, we let create_dir_all handle the existing directory case.
        }
    }

    // 2. Create the directory `./src/empty` (using create_dir_all for robustness).
    // This function creates all necessary parent directories and succeeds if the directory already exists.
    //println!("cargo:warning=Creating directory: ./src/empty");
    //if let Err(e) = fs::create_dir_all(target_path) {
    //    panic!("Failed to create directory {}: {}", target_path.display(), e);
    //}

    let dir_path = Path::new("src/empty");
    let readme_path = dir_path.join("README.md");

    println!("cargo:rerun-if-changed=build.rs");

    // --- 1. Remove the Directory (if it exists) ---
    if dir_path.exists() {
        match fs::remove_dir_all(dir_path) {
            Ok(_) => println!(
                "Build: Successfully removed directory: {}",
                dir_path.display()
            ),
            Err(e) => {
                panic!(
                    "Build: Failed to remove directory {}: {}",
                    dir_path.display(),
                    e
                );
            }
        }
    } else {
        println!(
            "Build: Directory {} does not exist, skipping removal.",
            dir_path.display()
        );
    }

    // --- 2. Create the Directory ---
    match fs::create_dir_all(dir_path) {
        Ok(_) => println!(
            "Build: Successfully created directory: {}",
            dir_path.display()
        ),
        Err(e) => {
            panic!(
                "Build: Failed to create directory {}: {}",
                dir_path.display(),
                e
            );
        }
    }

    let content = r###"### gnostr-lfs/src/empty

This directory is intentionally kept minimal and serves as a placeholder for the initial
empty tree object in the Git repository history. The first commit creates the project's
root using a known epoch date for historical consistency.

GIT_AUTHOR_NAME=gnostr-vfs

GIT_AUTHOR_EMAIL=admin@gnostr.org

GIT_COMMITTER_NAME=gnostr_dev

GIT_COMMITTER_EMAIL=admin@gnostr.org

GIT_AUTHOR_DATE="Thu, 01 Jan 1970 00:00:00 +0000"

GIT_COMMITTER_DATE="Thu, 01 Jan 1970 00:00:00 +0000"

git commit --allow-empty -m "initial commit"

"###;

    match fs::File::create(&readme_path) {
        Ok(mut file) => match file.write_all(content.as_bytes()) {
            Ok(_) => println!("Build: Successfully wrote to {}", readme_path.display()),
            Err(e) => panic!(
                "Build: Failed to write content to {}: {}",
                readme_path.display(),
                e
            ),
        },
        Err(e) => panic!(
            "Build: Failed to create file {}: {}",
            readme_path.display(),
            e
        ),
    }

    // --- 3. Run 'git init' inside src/empty ---
    println!(
        "Build: Initializing Git repository in {}",
        dir_path.display()
    );

    let output = Command::new("git")
        .arg("init")
        .current_dir(dir_path) // Crucial: executes 'git init' inside the target folder
        .output()
        .expect("Failed to execute 'git init'");

    if output.status.success() {
        println!("Build: git init successful.");
    } else {
        panic!(
            "Build: git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    println!("Build: Adding README.md to the index...");

    let output = Command::new("git")
        .arg("-C")
        .arg(dir_path)
        .arg("add")
        .arg(".") // Use '.' to add all files in the current directory (src/empty)
        //.current_dir(dir_path) // Executes 'git add .' inside src/empty
        .output()
        .expect("Failed to execute 'git add'");

    if output.status.success() {
        println!("Build: git add successful.");
    } else {
        panic!(
            "Build: git add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    println!("Build: Adding README.md to the index...");

    //let output = Command::new("git")
    //    .arg("-C")
    //    .arg(dir_path)
    //    .arg("commit")
    //    .arg("-m") // Use '.' to add all files in the current directory (src/empty)
    //    .arg("READM.md") // Use '.' to add all files in the current directory (src/empty)
    //    //.current_dir(dir_path) // Executes 'git add .' inside src/empty
    //    .output()
    //    .expect("Failed to execute 'git add'");

    //if output.status.success() {
    //    println!("Build: git commit successful.");
    //} else {
    //    panic!(
    //        "Build: git add failed: {}",
    //        String::from_utf8_lossy(&output.stderr)
    //    );
    //}

    git_commit(dir_path);
    // Good practice: Rerun build script if the script itself changes.
    println!("cargo:rerun-if-changed=src/empty");
}

fn git_commit(dir_path: &Path) -> Result<(), io::Error> {
    // 1. Convert Path to &str safely using .ok_or_else()
    // This converts the Option<&str> to a Result<&str, io::Error>.
    // If the path is invalid UTF-8 (None), it generates a custom io::Error
    // with kind InvalidInput, which is compatible with the function's return type.
    let dir_path_str = dir_path.to_str().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("Path '{}' is not valid UTF-8", dir_path.display())
        )
    })?; // Now the '?' operator works, returning an io::Error on failure.

    // 2. Start the command and set environment variables (as in previous answer)
    let mut command = Command::new("git");

    // Setting the environment variables:
    command
        .env("GIT_AUTHOR_NAME", "gnostr-vfs")
        .env("GIT_AUTHOR_EMAIL", "admin@gnostr.org")
        .env("GIT_COMMITTER_NAME", "gnostr_dev")
        .env("GIT_COMMITTER_EMAIL", "admin@gnostr.org")
        .env("GIT_AUTHOR_DATE", "Thu, 01 Jan 1970 00:00:00 +0000")
        .env("GIT_COMMITTER_DATE", "Thu, 01 Jan 1970 00:00:00 +0000");


    // 3. Set the arguments. Note the use of the safe dir_path_str variable.
    command.args(&[
        "-C", // Use -C to run the git command from the specified directory
        dir_path_str,
        "commit",
        "--allow-empty",
        "-m",
        "initial commit",
    ]);

    println!("Executing command: git -C {} commit ...", dir_path_str);

    // 4. Execute the command. The '?' handles the *process execution* I/O error.
    let output = command.output()?;

    // 5. Check command success and return Result accordingly.
    if output.status.success() {
        println!("\n✅ Git command successful!");
        println!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        // The success block must return the success value, which is Ok(())
        Ok(())
    } else {
        println!("\n❌ Git command failed!");
        eprintln!("Status: {}", output.status);
        eprintln!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        
        // The failure block must return the error value, which is Err(io::Error).
        // We create a new io::Error here to indicate the child process failed.
        Err(Error::new(
            ErrorKind::Other,
            format!("'git commit' failed with status: {}", output.status)
        ))
    }
}
