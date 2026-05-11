use chrono::TimeZone;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let now = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(val) => chrono::Local
            .timestamp_opt(val.parse::<i64>().unwrap(), 0)
            .unwrap(),
        Err(_) => chrono::Local::now(),
    };
    let build_date = now.date_naive();
    let build_name = if std::env::var("GITUI_RELEASE").is_ok() {
        format!("{}@{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    } else {
        format!(
            "{}@{} {} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            build_date,
            get_git_hash()
        )
    };

    println!("cargo:warning=buildname '{build_name}'");
    println!("cargo:rustc-env=GITUI_BUILD_NAME={build_name}");
    let app_metadata = json!({
        "name": "gnostr",
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "website": env!("CARGO_PKG_HOMEPAGE"),
        "repository": env!("CARGO_PKG_REPOSITORY"),
        "build_name": build_name,
        "kind": 31990,
    });
    println!("cargo:rustc-env=GITUI_APP_NAME=gnostr");
    println!("cargo:rustc-env=GITUI_APP_METADATA_JSON={}", app_metadata.to_string());
    write_nip89_app_asset(&build_name);
    watch_sources(Path::new("src/js"), &["js"]);
    watch_sources(Path::new("src/css"), &["css"]);
    watch_sources(Path::new("src"), &["html"]);
}

fn write_nip89_app_asset(build_name: &str) {
    let out_dir = match std::env::var("OUT_DIR") {
        Ok(dir) => dir,
        Err(error) => {
            println!("cargo:warning=skipping NIP-89 app asset: {error}");
            return;
        }
    };

    let description = env!("CARGO_PKG_DESCRIPTION");
    let metadata = serde_json::json!({
        "kind": 31990,
        "pubkey": "",
        "content": {
            "name": "gnostr",
            "about": if description.is_empty() { "git+nostr workflow utility" } else { description },
            "website": env!("CARGO_PKG_HOMEPAGE"),
            "repository": env!("CARGO_PKG_REPOSITORY"),
            "picture": "/images/logo.svg",
            "version": env!("CARGO_PKG_VERSION"),
            "build_name": build_name,
        },
        "tags": [
            ["d", "gnostr"],
            ["k", "0"],
            ["k", "1"],
            ["k", "3"],
            ["k", "4"],
            ["k", "10002"],
            ["k", "31989"],
            ["k", "31990"],
            ["web", "/settings"],
        ]
    });

    let path = Path::new(&out_dir).join("nip89-app.json");
    let data = match serde_json::to_vec_pretty(&metadata) {
        Ok(data) => data,
        Err(error) => {
            println!("cargo:warning=skipping NIP-89 app asset: {error}");
            return;
        }
    };

    if let Err(error) = fs::write(&path, data) {
        println!("cargo:warning=failed to write NIP-89 app asset: {error}");
    } else {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

fn get_git_hash() -> String {
    if let Ok(commit) = std::env::var("BUILD_GIT_COMMIT_ID") {
        return commit[..7].to_string();
    }

    let commit = Command::new("git")
        .arg("rev-parse")
        .arg("--short=7")
        .arg("--verify")
        .arg("HEAD")
        .output();

    if let Ok(commit_output) = commit {
        let commit_string = String::from_utf8_lossy(&commit_output.stdout);
        return commit_string.lines().next().unwrap_or("").into();
    }

    panic!("Can not get git commit: {}", commit.unwrap_err());
}

fn watch_sources(dir: &Path, extensions: &[&str]) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            panic!("failed to read {}: {}", dir.display(), err);
        }
    };

    for entry in entries {
        let entry = entry.unwrap_or_else(|err| {
            panic!("failed to read entry in {}: {}", dir.display(), err);
        });
        let path = entry.path();
        if path.is_dir() {
            watch_sources(&path, extensions);
            continue;
        }

        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| extensions.contains(&ext))
        {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
