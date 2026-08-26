use std::env;
use std::path::Path;
use std::process::{Command, exit};

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    
    let mut flags = Vec::new();
    let mut command_args = Vec::new();
    
    for arg in raw_args {
        if arg == "-v" || arg == "-vv" {
            flags.push(arg);
        } else {
            command_args.push(arg);
        }
    }
    
    let verbosity = if flags.contains(&"-vv".to_string()) {
        2
    } else if flags.contains(&"-v".to_string()) {
        1
    } else {
        0
    };

    match command_args.get(0).map(|s| s.as_str()) {
        Some("build") => {
            let target = command_args.get(1).map(|s| s.as_str());
            run_build(target, verbosity, &flags);
        }
        Some("run-script") => {
            let script_name = command_args.get(1).map(|s| s.as_str());
            let script_args = if command_args.len() > 2 { &command_args[2..] } else { &[] };
            run_script(script_name, script_args, verbosity);
        }
        Some("xcode") => {
            let script_args = if command_args.len() > 1 { &command_args[1..] } else { &[] };
            if script_args.len() == 0 {
                print_usage();
                exit(1);
            }
            run_xcode_script(script_args, verbosity);
        }
        Some("help") | Some("--help") | None => {
            print_usage();
        }
        Some(cmd) => {
            eprintln!("Unknown command: {}", cmd);
            print_usage();
            exit(1);
        }
    }
}

fn get_host_target() -> String {
    let output = Command::new("rustc")
        .arg("-vV")
        .output()
        .expect("Failed to run rustc to detect host target");
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.starts_with("host: ") {
            return line.replace("host: ", "").trim().to_string();
        }
    }
    panic!("Could not detect host target triple");
}

fn print_usage() {
    println!("Usage: cargo xtask [-v|-vv] <command> [args]");
    println!("\nCommands:");
    println!("  build [target]           Builds the project (default: host target)");
    println!("  run-script <name> [args] Runs a script from the ./scripts directory");
    println!("  run-script --help        Lists available scripts");
    println!("  xcode <name> [args]      Runs a script from the ./xcode/scripts directory");
    println!("  xcode --help             Lists available xcode scripts");
    println!("\nExample:");
    println!("  cargo xtask build");
    println!("  cargo xtask run-script cargo-check.sh");
}

fn list_scripts() {
    println!("Available scripts in ./scripts:");
    if let Ok(entries) = std::fs::read_dir("scripts") {
        let mut scripts: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
            .filter(|name| !name.starts_with('.'))
            .collect();
        scripts.sort_by_key(|s| s.to_lowercase());
        for name in scripts {
            println!("  {}", name);
        }
    } else {
        eprintln!("Error: ./scripts directory not found.");
    }
}

fn list_xcode_scripts() {
    println!("Available scripts in ./xcode/scripts:");
    if let Ok(entries) = std::fs::read_dir("xcode/scripts") {
        let mut scripts: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
            .filter(|name| !name.starts_with('.'))
            .collect();
        scripts.sort_by_key(|s| s.to_lowercase());
        for name in scripts {
            println!("  {}", name);
        }
    } else {
        eprintln!("Error: ./xcode/scripts directory not found.");
    }
}

fn run_xcode_script(script_args: &[String], verbosity: u8) {
    if script_args.len() > 0 && (script_args[0] == "--help" || script_args[0] == "help") {
        list_xcode_scripts();
        exit(0);
    }

    let script_name = if script_args.len() > 0 {
        &script_args[0]
    } else {
        eprintln!("Error: Missing script name");
        print_usage();
        exit(1);
    };

    let script_path = Path::new("xcode/scripts").join(script_name);

    if !script_path.exists() {
        eprintln!("Error: Script not found: {}", script_path.display());
        exit(1);
    }

    if verbosity > 0 { println!("--- Running script: {} ---", script_path.display()); }

    let mut cmd = Command::new("bash");
    cmd.arg(&script_path);
    cmd.args(&script_args[1..]);

    let status = cmd.status().expect("Failed to execute script");

    if !status.success() {
        eprintln!("Script failed: {}", script_path.display());
        exit(1);
    } else {
        if verbosity > 0 { println!("--- Script executed successfully! ---"); }
    }
}


fn run_build(target_opt: Option<&str>, verbosity: u8, _flags: &[String]) {
    let final_target = target_opt.unwrap_or_else(|| {
        let host = get_host_target();
        if verbosity > 0 { println!("--- No target specified: Defaulting to {} ---", host); }
        "" 
    });

    let target = if final_target.is_empty() { get_host_target() } else { final_target.to_string() };

    let external_root = Path::new("/Volumes/DeepSpaceExtSDD");
    
    let target_dir = if external_root.exists() {
        let path = external_root.join("target").join(&target);
        if verbosity > 0 { println!("--- External volume detected: Using {} ---", path.display()); }
        path.to_string_lossy().into_owned()
    } else {
        format!("target/{}", target)
    };
    
    if verbosity > 0 { println!("--- Building for: {} ---", target); }

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
       .arg("--target")
       .arg(&target)
       .env("CARGO_TARGET_DIR", &target_dir);
    
    if verbosity > 0 { cmd.arg(if verbosity == 2 { "-vv" } else { "-v" }); }

    let status = cmd.status().expect("Failed to execute cargo build");

    if !status.success() {
        eprintln!("Build failed for target: {}", target);
        exit(1);
    } else {
        if verbosity > 0 { println!("--- Build successful! Artifacts located in: {} ---", target_dir); }
    }
}

fn run_script(script_name_opt: Option<&str>, script_args: &[String], verbosity: u8) {
    let script_name = match script_name_opt {
        Some("--help") | Some("help") => {
            list_scripts();
            exit(0);
        }
        Some(name) => name,
        None => {
            eprintln!("Error: Missing script name");
            print_usage();
            exit(1);
        }
    };

    let script_path = Path::new("scripts").join(script_name);

    if !script_path.exists() {
        eprintln!("Error: Script not found: {}", script_path.display());
        exit(1);
    }

    println!("--- Running script: {} ---", script_path.display());

    let mut cmd = Command::new("bash");
    cmd.arg(&script_path);
    cmd.args(script_args);

    let status = cmd.status().expect("Failed to execute script");

    if !status.success() {
        eprintln!("Script failed: {}", script_path.display());
        exit(1);
    } else {
        if verbosity > 0 { println!("--- Script executed successfully! ---"); }
    }
}
