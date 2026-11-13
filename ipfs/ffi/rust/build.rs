use std::env;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let ffi_dir = manifest_dir.join("..");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let target = env::var("TARGET").expect("TARGET not set");
    let host = env::var("HOST").expect("HOST not set");

    // Cross-compilation: allow same-family builds, bail out early for
    // unsupported host/target pairs with a clear message.
    let host_parts: Vec<&str> = host.split('-').collect();
    let target_parts: Vec<&str> = target.split('-').collect();
    let host_os = host_parts.get(2).copied().unwrap_or("unknown");
    let target_os = target_parts.get(2).copied().unwrap_or("unknown");
    let host_vendor = host_parts.get(1).copied().unwrap_or("unknown");
    let target_vendor = target_parts.get(1).copied().unwrap_or("unknown");

    let both_darwin = host_vendor == "apple" && target_vendor == "apple";
    let both_linux = host_os == "linux" && target_os == "linux";
    let both_windows = host_os == "windows" && target_os == "windows";

    if target != host && !both_darwin && !both_linux && !both_windows {
        panic!(
            "kubo-ffi build script does not support cross-compilation \
             for this target/host pair. TARGET ({}) != HOST ({}). \
             To cross-compile, set up a C cross-toolchain and \
             manually compile go/ffi/ with CGO_ENABLED=1.",
            target, host
        );
    }

    // Find Go binary.
    let go = env::var("GO").unwrap_or_else(|_| "go".to_string());
    match Command::new(&go).args(["version"]).output() {
        Ok(output) if output.status.success() => {}
        Ok(_) => panic!("`go version` failed. Make sure Go is installed and in PATH."),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            panic!(
                "Go toolchain not found: set GO or add `go` to PATH. \
                 Kubo requires Go >= 1.21."
            );
        }
        Err(err) => panic!("failed to run `go version` with `{go}`: {err}"),
    }

    // Pin GOTOOLCHAIN to the Go version declared in go.mod so that the
    // build is reproducible even when the host has a newer Go installed.
    let go_mod_text =
        std::fs::read_to_string(ffi_dir.join("go.mod")).expect("failed to read go.mod");
    let toolchain = go_mod_text
        .lines()
        .find_map(|line| {
            line.strip_prefix("go ")
                .map(|ver| format!("go{}", ver.trim()))
        })
        .expect("no 'go' directive found in go.mod");

    // Map Rust target to Go os/arch.
    let (goos, goarch) = parse_target(&target);

    // Go's c-archive naming depends on the platform toolchain.
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let env_kind = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let (archive_name, _header_name) = if os == "windows" && env_kind == "msvc" {
        ("kubo_ffi.lib", "kubo_ffi.h")
    } else {
        ("libkubo_ffi.a", "kubo_ffi.h")
    };
    let archive = out_dir.join(archive_name);

    // Only rebuild when Go sources change.
    for src in &[
        "kubo.go",
        "libp2p.go",
        "git.go",
        "nostr.go",
        "error.go",
        "command.go",
        "main.go",
    ] {
        println!("cargo:rerun-if-changed={}", ffi_dir.join(src).display());
    }
    println!(
        "cargo:rerun-if-changed={}",
        ffi_dir.join("go.mod").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        ffi_dir.join("go.sum").display()
    );

    let mut go_build = Command::new(&go);
    go_build
        .current_dir(&ffi_dir)
        .env("CGO_ENABLED", "1")
        .env("GOOS", goos)
        .env("GOARCH", goarch)
        .env("GOTOOLCHAIN", &toolchain)
        .args([
            "build",
            "-buildmode=c-archive",
            "-o",
            archive.to_str().unwrap(),
            ".",
        ]);

    // Prevent Apple deployment-target env vars from leaking into the Go build.
    go_build.env_remove("IPHONEOS_DEPLOYMENT_TARGET");
    go_build.env_remove("MACOSX_DEPLOYMENT_TARGET");

    // When cross-compiling for Apple platforms, the Go toolchain needs an
    // Apple-specific C cross-compiler.
    if let Some((sdk, cc, cxx)) = apple_compiler(&target) {
        go_build.env("CC", &cc).env("CXX", &cxx).env("SDKROOT", sdk);

        let deployment_target = env::var("IPHONEOS_DEPLOYMENT_TARGET")
            .or_else(|_| env::var("MACOSX_DEPLOYMENT_TARGET"))
            .unwrap_or_else(|_| "14.0".to_string());
        go_build.env("IPHONEOS_DEPLOYMENT_TARGET", &deployment_target);
    }

    let status = go_build.status().expect("failed to run go");
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
}

fn parse_target(target: &str) -> (&str, &str) {
    let parts: Vec<&str> = target.split('-').collect();
    let arch = parts[0];

    let os = if parts.len() >= 3 && parts[1] == "apple" && parts[2].starts_with("ios") {
        "ios"
    } else {
        parts.get(2).copied().unwrap_or("unknown")
    };

    let goarch = match arch {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "i686" => "386",
        "arm" => "arm",
        "armv7" => "arm",
        "riscv64" => "riscv64",
        "wasm32" => panic!("wasm32 is not supported by CGO"),
        _ => arch,
    };

    (os, goarch)
}

fn apple_compiler(target: &str) -> Option<(String, String, String)> {
    if std::env::consts::OS != "macos" {
        return None;
    }

    let parts: Vec<&str> = target.split('-').collect();
    if parts.len() < 3 || parts[1] != "apple" {
        return None;
    }

    let arch = parts[0];
    let os = parts[2];
    let variant = parts.get(3).copied();

    let (sdk, target_flag): (&str, &str) = match (arch, os, variant) {
        ("aarch64", "ios", None) => ("iphoneos", "-target arm64-apple-ios"),
        ("x86_64", "ios", None) => ("iphonesimulator", "-target x86_64-apple-ios-simulator"),
        ("aarch64", "ios", Some("sim")) => ("iphonesimulator", "-target arm64-apple-ios-simulator"),
        ("aarch64", "ios", Some("macabi")) => ("macosx", "-target arm64-apple-ios-macabi"),
        ("x86_64", "ios", Some("macabi")) => ("macosx", "-target x86_64-apple-ios-macabi"),
        (_, "darwin", _) => return None,
        _ => return None,
    };

    let cc = format!("xcrun -sdk {} clang {}", sdk, target_flag)
        .trim()
        .to_string();
    let cxx = format!("xcrun -sdk {} clang++ {}", sdk, target_flag)
        .trim()
        .to_string();

    Some((sdk.to_string(), cc, cxx))
}
