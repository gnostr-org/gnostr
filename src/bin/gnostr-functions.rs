use std::{
    env,
    path::PathBuf,
    process::Command,
    fs,
};

fn pathappend(args: &[String]) {
    let mut current_path = env::var("PATH").unwrap_or_default();
    let mut path_components: Vec<PathBuf> = env::split_paths(&current_path).collect();
    let mut changed = false;

    for arg in args {
        let arg_path = PathBuf::from(arg);
        if arg_path.is_dir() && !path_components.contains(&arg_path) {
            path_components.push(arg_path);
            changed = true;
        }
    }

    if changed {
        current_path = env::join_paths(path_components).unwrap().to_string_lossy().into_owned();
        env::set_var("PATH", &current_path);
        println!("New PATH: {}", current_path);
    } else {
        println!("PATH not changed.");
    }
}

fn pathprepend(args: &[String]) {
    let mut current_path = env::var("PATH").unwrap_or_default();
    let mut path_components: Vec<PathBuf> = env::split_paths(&current_path).collect();
    let mut changed = false;

    // Iterate in reverse to prepend correctly
    for arg in args.iter().rev() {
        let arg_path = PathBuf::from(arg);
        if arg_path.is_dir() && !path_components.contains(&arg_path) {
            path_components.insert(0, arg_path); // Prepend
            changed = true;
        }
    }

    if changed {
        current_path = env::join_paths(path_components).unwrap().to_string_lossy().into_owned();
        env::set_var("PATH", &current_path);
        println!("New PATH: {}", current_path);
    } else {
        println!("PATH not changed.");
    }
}

fn rustup_clean() {
    println!("Running rustup-clean...");

    let output = Command::new("rustup")
        .arg("toolchain")
        .arg("list")
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let toolchains: Vec<String> = stdout
                    .lines()
                    .filter_map(|line| {
                        let trimmed_line = line.replace("(active,", "").replace("default", "").trim().to_string();
                        if !trimmed_line.is_empty() {
                            Some(trimmed_line)
                        } else {
                            None
                        }
                    })
                    .collect();

                if toolchains.is_empty() {
                    println!("No toolchains found to uninstall.");
                    return;
                }

                println!("Found toolchains: {:?}", toolchains);

                for toolchain in toolchains {
                    println!("Uninstalling toolchain: {}", toolchain);
                    let uninstall_output = Command::new("rustup")
                        .arg("toolchain")
                        .arg("uninstall")
                        .arg(&toolchain)
                        .output();

                    match uninstall_output {
                        Ok(uninstall_output) => {
                            if uninstall_output.status.success() {
                                println!("Successfully uninstalled {}", toolchain);
                            } else {
                                eprintln!("Failed to uninstall {}: {}", toolchain, String::from_utf8_lossy(&uninstall_output.stderr));
                            }
                        }
                        Err(e) => eprintln!("Failed to execute rustup uninstall for {}: {}", toolchain, e),
                    }
                }
                println!("rustup-clean completed.");
            } else {
                eprintln!("Failed to list rustup toolchains: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => eprintln!("Failed to execute rustup toolchain list: {}", e),
    }
}

fn get_ip_address(interface: &str) -> Option<String> {
    let output = Command::new("ifconfig")
        .arg(interface)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("inet ") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return Some(parts[1].to_string());
                        }
                    }
                }
                None
            } else {
                eprintln!("Failed to get info for {}: {}", interface, String::from_utf8_lossy(&output.stderr));
                None
            }
        }
        Err(e) => {
            eprintln!("Failed to execute ifconfig for {}: {}", interface, e);
            None
        }
    }
}

fn geten0() {
    match get_ip_address("en0") {
        Some(ip) => println!("{}", ip),
        None => eprintln!("Could not get IP for en0"),
    }
}

fn geten1() {
    match get_ip_address("en1") {
        Some(ip) => println!("{}", ip),
        None => eprintln!("Could not get IP for en1"),
    }
}

fn bitcoin_configure_disable_wallet_tests_bench() {
    println!("Running ./configure --disable-wallet --disable-tests --disable-bench...");
    let status = Command::new("sh")
        .arg("-c")
        .arg("./configure --disable-wallet --disable-tests --disable-bench")
        .status();

    match status {
        Ok(s) if s.success() => println!("./configure executed successfully."),
        Ok(s) => eprintln!("./configure exited with status: {}", s),
        Err(e) => eprintln!("Failed to execute ./configure: {}", e),
    }
    println!("bitcoin-configure-disable-wallet-tests-bench completed.");
}

fn bitcoin_configure_disable_tests_bench() {
    println!("Running ./configure --disable-tests --disable-bench...");
    let status = Command::new("sh")
        .arg("-c")
        .arg("./configure --disable-tests --disable-bench")
        .status();

    match status {
        Ok(s) if s.success() => println!("./configure executed successfully."),
        Ok(s) => eprintln!("./configure exited with status: {}", s),
        Err(e) => eprintln!("Failed to execute ./configure: {}", e),
    }
    println!("bitcoin-configure-disable-tests-bench completed.");
}

fn execute_bash_script(script_content: &str) -> Result<(), String> {
    println!("Executing bash script:\n{}", script_content);
    let output = Command::new("bash")
        .arg("-c")
        .arg(script_content)
        .output()
        .map_err(|e| format!("Failed to execute bash command: {}", e))?;

    if output.status.success() {
        println!("Script executed successfully.");
        Ok(())
    } else {
        Err(format!("Script exited with error:\nStdout: {}\nStderr: {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)))
    }
}

fn bitcoin_make_appbundle() {
    println!("Running bitcoin-make-appbundle...");
    let commands = vec![
        "rm -f src/bitcoind",
        "rm -rf Bitcoin-Qt.app",
        "rm -rf ~/Library/Saved\\ Application\\ State/org.bitcoinfoundation.Bitcoin-Qt.savedState",
        "rm -rf ~/Library/Preferences/org.bitcoin.Bitcoin-Qt.plist",
        "make appbundle",
        // "wait", // This command might behave differently in a direct subprocess call, may need adjustment
        "./Bitcoin-Qt.app/Contents/MacOS/Bitcoin-Qt -resetguisettings",
    ];

    for cmd_str in commands {
        println!("Executing: {}", cmd_str);
        let status = Command::new("sh")
            .arg("-c")
            .arg(cmd_str)
            .status();

        match status {
            Ok(s) if s.success() => println!("Command '{}' executed successfully.", cmd_str),
            Ok(s) => {
                eprintln!("Command '{}' exited with status: {}. Aborting.", cmd_str, s);
                return;
            },
            Err(e) => {
                eprintln!("Failed to execute command '{}': {}. Aborting.", cmd_str, e);
                return;
            },
        }
    }
    println!("bitcoin-make-appbundle completed.");
}

fn bitcoin_make_depends() {
    println!("Running make -C depends...");
    let status = Command::new("make")
        .arg("-C")
        .arg("depends")
        .status();

    match status {
        Ok(s) if s.success() => println!("make -C depends executed successfully."),
        Ok(s) => eprintln!("make -C depends exited with status: {}", s),
        Err(e) => eprintln!("Failed to execute make -C depends: {}", e),
    }
    println!("bitcoin-make-depends completed.");
}

fn cargo_dl_install_depends() {
    println!("Running bitcoin-dl-install-depends...");

    let os_type = env::var("OSTYPE").unwrap_or_default();
    let script_content = if os_type == "linux-gnu" {
        r#"
            sudo apt update
            sudo apt install -y linuxbrew-wrapper autoconf libdb4.8++-dev libboost-all-dev libevent-dev miniupnpc libdb4.8-dev qtbase5-dev libqrencode-dev univalue-dev libzmq3-dev build-essential libtool autotools-dev automake pkg-config bsdmainutils python3 librsvg2-dev

            ./contrib/install_db4.sh .
            ./autogen.sh && ./configure --disable-tests && make download install -C depends
        "#
    } else if os_type.starts_with("darwin") {
        r#"
            if type -P brew >/dev/null; then
                brew install wget curl autoconf automake berkeley-db4 libtool boost miniupnpc pkg-config python qt libevent qrencode librsvg
            else
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
                brew install wget curl autoconf automake berkeley-db4 libtool boost miniupnpc pkg-config python qt libevent qrencode librsvg
            fi
            // The original script had pushd/popd and repeated make/configure commands
            // which indicates it expects to be run from specific directories like ~/gui or ~/bitcoin.
            // For simplicity in a single Rust binary, we assume it's run from the project root
            // or that paths are relative to where the binary is executed. Adjust paths if necessary.
            make download install -C depends
            ./autogen.sh && ./configure --disable-tests && make download install -C depends
        "#
    } else {
        eprintln!("Unsupported OS: {}", os_type);
        return;
    };

    if let Err(e) = execute_bash_script(script_content) {
        eprintln!("bitcoin-dl-install-depends failed: {}", e);
    } else {
        println!("bitcoin-dl-install-depends completed.");
    }
}

fn cargo_clean_r() {
    println!("Running cargo-clean-r...");
    let original_dir = env::current_dir().expect("Failed to get current directory");

    match fs::read_dir(".") {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                        // Only process if it looks like a repo directory (not . or .. or target/)
                        if dir_name != "." && dir_name != ".." && dir_name != "target" {
                            println!("Entering directory: {}", path.display());
                            if let Err(e) = env::set_current_dir(&path) {
                                eprintln!("Failed to change directory to {}: {}", path.display(), e);
                                continue;
                            }

                            let status = Command::new("cargo")
                                .arg("clean")
                                .output();

                            match status {
                                Ok(output) => {
                                    if !output.status.success() {
                                        eprintln!("cargo clean failed in {}: {}\n{}", path.display(), String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
                                    } else {
                                        println!("cargo clean successful in {}.", path.display());
                                    }
                                }
                                Err(e) => eprintln!("Failed to execute cargo clean in {}: {}", path.display(), e),
                            }

                            // Change back to the original directory
                            if let Err(e) = env::set_current_dir(&original_dir) {
                                eprintln!("Failed to change back to original directory: {}", e);
                                // Critical error, might need to panic or exit
                            }
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to read current directory: {}", e),
    }
    println!("cargo-clean-r completed.");
}

fn cargo_sweep_r(args: &[String]) {
    println!("Running cargo-sweep-r...");
    let original_dir = env::current_dir().expect("Failed to get current directory");

    let time_arg = args.get(0).map_or("1", |s| s.as_str());

    match fs::read_dir(".") {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                        // Only process if it looks like a repo directory (not . or .. or target/)
                        if dir_name != "." && dir_name != ".." && dir_name != "target" {
                            println!("Entering directory: {}", path.display());
                            if let Err(e) = env::set_current_dir(&path) {
                                eprintln!("Failed to change directory to {}: {}", path.display(), e);
                                continue;
                            }

                            // Remove .deps
                            let _ = fs::remove_dir_all(".deps");
                            // Remove .venv
                            let _ = fs::remove_dir_all(".venv");
                            // Remove rust-toolchain.toml
                            let _ = fs::remove_file("rust-toolchain.toml");

                            let status_clean = Command::new("cargo")
                                .arg("clean")
                                .output();

                            match status_clean {
                                Ok(output) => {
                                    if !output.status.success() {
                                        eprintln!("cargo clean failed in {}: {}\n{}", path.display(), String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
                                    } else {
                                        println!("cargo clean successful in {}.", path.display());
                                    }
                                }
                                Err(e) => eprintln!("Failed to execute cargo clean in {}: {}", path.display(), e),
                            }

                            let status_sweep = Command::new("cargo")
                                .arg("sweep")
                                .arg("-v")
                                .arg("-t")
                                .arg(time_arg)
                                .output();

                            match status_sweep {
                                Ok(output) => {
                                    if !output.status.success() {
                                        eprintln!("cargo sweep failed in {}: {}\n{}", path.display(), String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
                                    } else {
                                        println!("cargo sweep successful in {}.", path.display());
                                    }
                                }
                                Err(e) => eprintln!("Failed to execute cargo sweep in {}: {}", path.display(), e),
                            }

                            // Change back to the original directory
                            if let Err(e) = env::set_current_dir(&original_dir) {
                                eprintln!("Failed to change back to original directory: {}", e);
                                // Critical error, might need to panic or exit
                            }
                        }
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to read current directory: {}", e),
    }

    // rm-rf-node_modules || true - this part is separate in the bash script, will implement as a separate Rust function

    println!("cargo-sweep-r completed.");
}

fn iftop() {
    println!("Running iftop. Press Ctrl+C to exit.");
    let status = Command::new("iftop")
        .status();

    match status {
        Ok(s) if s.success() => println!("iftop executed successfully."),
        Ok(s) => eprintln!("iftop exited with status: {}", s),
        Err(e) => eprintln!("Failed to execute iftop: {}. Please ensure iftop is installed and in your PATH.", e),
    }
    println!("iftop completed.");
}

fn install_gnumakefile() {
    println!("Running install-gnumakefile...");

    let output = Command::new("which").arg("gmake").output();
    match output {
        Ok(output) => {
            if output.status.success() {
                println!("GNU Make (gmake) is already installed: {}", String::from_utf8_lossy(&output.stdout).trim());
                println!("install-gnumakefile completed.");
                return;
            }
        }
        Err(e) => eprintln!("Failed to execute 'which gmake': {}", e),
    }

    let output = Command::new("which").arg("make").output();
    match output {
        Ok(output) => {
            if output.status.success() {
                println!("Make is installed: {}", String::from_utf8_lossy(&output.stdout).trim());
            }
        }
        Err(e) => eprintln!("Failed to execute 'which make': {}", e),
    }

    eprintln!("GNU Make (gmake) not found. On macOS, you can install it via Homebrew: brew install gnu-make");
    println!("install-gnumakefile completed.");
}

fn bitcoin_autogen() {
    println!("Running bitcoin-autogen (./autogen.sh)...");
    let status = Command::new("sh")
        .arg("-c")
        .arg("./autogen.sh")
        .status();

    match status {
        Ok(s) if s.success() => println!("./autogen.sh executed successfully."),
        Ok(s) => eprintln!("./autogen.sh exited with status: {}", s),
        Err(e) => eprintln!("Failed to execute ./autogen.sh: {}. Please ensure the script exists and is executable.", e),
    }
    println!("bitcoin-autogen completed.");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "pathappend" => pathappend(&args[2..]),
            "pathprepend" => pathprepend(&args[2..]),
            "rustup-clean" => rustup_clean(),
            "geten0" => geten0(),
            "geten1" => geten1(),
            "iftop" => iftop(),
            "install-gnumakefile" => install_gnumakefile(),
            "bitcoin-autogen" => bitcoin_autogen(),
            "bitcoin-configure-disable-tests-bench-fuzz" => bitcoin_configure_disable_tests_bench(),
            "bitcoin-configure-disable-wallet-tests-bench-fuzz" => bitcoin_configure_disable_wallet_tests_bench(),
            "bitcoin-configure-disable-wallet-tests-bench" => bitcoin_configure_disable_wallet_tests_bench(),
            "bitcoin-configure-disable-tests-bench" => bitcoin_configure_disable_tests_bench(),
            "bitcoin-make-appbundle" => bitcoin_make_appbundle(),
            "bitcoin-make-depends" => bitcoin_make_depends(),
            "bitcoin-dl-install-depends" => cargo_dl_install_depends(),
            "cargo-clean-r" => cargo_clean_r(),
            "cargo-sweep-r" => cargo_sweep_r(&args[2..]),
            _ => {
                println!("gnostr-functions binary will contain Rust equivalents of bash functions.");
                println!("Usage:");
                println!("  gnostr-functions pathappend <paths...>");
                println!("  gnostr-functions pathprepend <paths...>");
                println!("  gnostr-functions rustup-clean");
                println!("  gnostr-functions geten0");
                println!("  gnostr-functions geten1");
                println!("  gnostr-functions iftop");
                println!("  gnostr-functions install-gnumakefile");
                println!("  gnostr-functions bitcoin-autogen");
                println!("  gnostr-functions bitcoin-configure-disable-tests-bench-fuzz");
                println!("  gnostr-functions bitcoin-configure-disable-wallet-tests-bench-fuzz");
                println!("  gnostr-functions bitcoin-configure-disable-wallet-tests-bench");
                println!("  gnostr-functions bitcoin-configure-disable-tests-bench");
                println!("  gnostr-functions bitcoin-make-appbundle");
                println!("  gnostr-functions bitcoin-make-depends");
                println!("  gnostr-functions bitcoin-dl-install-depends");
                println!("  gnostr-functions cargo-clean-r");
                println!("  gnostr-functions cargo-sweep-r <time>");
            }
        }
    } else {
        println!("gnostr-functions binary will contain Rust equivalents of bash functions.");
        println!("Usage:");
        println!("  gnostr-functions pathappend <paths...>");
        println!("  gnostr-functions pathprepend <paths...>");
        println!("  gnostr-functions rustup-clean");
        println!("  gnostr-functions geten0");
        println!("  gnostr-functions geten1");
        println!("  gnostr-functions iftop");
        println!("  gnostr-functions install-gnumakefile");
        println!("  gnostr-functions bitcoin-autogen");
        println!("  gnostr-functions bitcoin-configure-disable-tests-bench-fuzz");
        println!("  gnostr-functions bitcoin-configure-disable-wallet-tests-bench-fuzz");
        println!("  gnostr-functions bitcoin-configure-disable-wallet-tests-bench");
        println!("  gnostr-functions bitcoin-configure-disable-tests-bench");
        println!("  gnostr-functions bitcoin-make-appbundle");
        println!("  gnostr-functions bitcoin-make-depends");
        println!("  gnostr-functions bitcoin-dl-install-depends");
    }
}