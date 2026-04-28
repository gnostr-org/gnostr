use chrono::TimeZone;
use std::fs;
use std::path::Path;

fn get_git_hash() -> String {
    use std::process::Command;

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
        env!("CARGO_PKG_VERSION").to_string()
    } else {
        format!(
            "gnostr-js-{} {} ({})",
            env!("CARGO_PKG_VERSION"),
            build_date,
            get_git_hash()
        )
    };
    println!("cargo:rustc-env=GITUI_BUILD_NAME={build_name}");
    watch_js_sources(Path::new("src/js"));
}

fn watch_js_sources(dir: &Path) {
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
            watch_js_sources(&path);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) == Some("js") {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
