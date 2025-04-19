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
fn install_pkg_config() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    println!("cargo:warning=Attempting to install pkg-config using Homebrew...");
    let install_result = Command::new("brew")
        .args(&["install", "pkg-config"])
        .status();

    match install_result {
        Ok(status) if status.success() => {
            println!("cargo:warning=Successfully installed openssl@3 via Homebrew.");
            // Instruct rustc to link against the OpenSSL libraries installed by Brew.
            // The exact paths might vary slightly based on Brew's configuration.
            // It's generally safer to rely on the `openssl` crate to handle linking.
            // However, if you need explicit linking:
            if target_os == "aarch64-apple-darwin" {
                println!("cargo:rustc-link-search=native=/opt/homebrew/opt/openssl@3/lib");
                // For Apple Silicon
            };
            if target_os == "x86_64-apple-darwin" {
                println!("cargo:rustc-link-search=native=/usr/local/opt/openssl@3/lib");
                // For Intel
            };
            println!("cargo:rustc-link-lib=dylib=ssl@3");
            println!("cargo:rustc-link-lib=dylib=crypto@3");
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


use chrono::TimeZone;

fn get_git_hash() -> String {
	use std::process::Command;

	// Allow builds from `git archive` generated tarballs if output of
	// `git get-tar-commit-id` is set in an env var.
	if let Ok(commit) = std::env::var("BUILD_GIT_COMMIT_ID") {
		return commit[..7].to_string();
	};
	let commit = Command::new("git")
		.arg("rev-parse")
		.arg("--short=7")
		.arg("--verify")
		.arg("HEAD")
		.output();
	if let Ok(commit_output) = commit {
		let commit_string =
			String::from_utf8_lossy(&commit_output.stdout);

		return commit_string.lines().next().unwrap_or("").into();
	}

	panic!("Can not get git commit: {}", commit.unwrap_err());
}

fn main() {
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
    
	let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    match target_arch.as_str() {
        "x86_64" => {
            println!("cargo:warning=Building for x86_64 architecture.");
            println!("cargo:rustc-cfg=target_arch_x86_64");
            // Add x86_64 specific build logic here

            if target_os == "linux" {}
            if target_os == "macos" {}
            if target_os == "windows" {}
        }
        "aarch64" => {
            println!("cargo:warning=Building for aarch64 architecture.");
            println!("cargo:rustc-cfg=target_arch_aarch64");
            // Add aarch64 specific build logic here
            if target_os == "linux" {}
            if target_os == "macos" {}
            if target_os == "windows" {}
        }
        "arm" => {
            println!("cargo:warning=Building for arm architecture.");
            println!("cargo:rustc-cfg=target_arch_arm");
            // Add arm specific build logic here
            if target_os == "linux" {}
            if target_os == "macos" {}
            if target_os == "windows" {}
        }
        "wasm32" => {
            println!("cargo:warning=Building for wasm32 architecture.");
            println!("cargo:rustc-cfg=target_arch_wasm32");
            // Add wasm32 specific build logic here
            if target_os == "linux" {}
            if target_os == "macos" {}
            if target_os == "windows" {}
        }
        "riscv64" => {
            println!("cargo:warning=Building for riscv64 architecture.");
            println!("cargo:rustc-cfg=target_arch_riscv64");
            // Add riscv64 specific build logic here
            if target_os == "linux" {}
            if target_os == "macos" {}
            if target_os == "windows" {}
        }
        _ => {
            println!(
                "cargo:warning=Building for an unknown architecture: {}",
                target_arch
            );
            // Handle unknown architectures or provide a default
            if target_os == "linux" {}
            if target_os == "macos" {}
            if target_os == "windows" {}
        }
    }

    if !if_windows() {
        //try
        musl_install_pkg_config();
        if if_linux_unknown() {
            linux_install_pkg_config();
        }
        if target_os == "aarch64-apple-darwin" || target_os == "x86_64-apple-darwin" {
            println!("cargo:warning=On macOS, openssl@3 is recommended for this crate.");

            if check_brew() {
                println!("cargo:warning=Homebrew detected.");
                install_pkg_config();
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
    }
    // Add other build logic here if needed
}

fn if_windows() -> bool {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

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
    let target = env::var("CARGO_CFG_TARGET_OS").unwrap();

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
                println!("cargo:warning=Could not find `libssl.so` and `libcrypto.so`. Ensure `libssl-dev` (or equivalent) is installed correctly.");
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
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target_os == "linux" {
        println!("cargo:warning=On Linux, the `pkgconfig` package (or equivalent providing `pkg-config`) is required for this crate.");
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
                println!("cargo:warning=`pkg-config` not found in your PATH. Ensure `pkg-config` (or equivalent) is installed and accessible.");
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
    let target = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target == "x86_64-unknown-linux-musl" {
        println!("cargo:warning=Building for x86_64-unknown-linux-musl (Musl libc).");
        println!("cargo:warning=This build process may require `pkg-config` to locate necessary libraries.");
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
                println!("cargo:warning=`pkg-config` not found in your PATH. Please ensure it is installed and accessible.");
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
