use chrono::TimeZone;
use std::process::Command;

#[cfg(all(not(debug_assertions), feature = "nostr"))]
#[tokio::main]
async fn main() {
    report_build_name();
    use std::fs;
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let crate_src_path = manifest_dir.join("..").join("online_relays_gps.csv");

    // Only download if the file doesn't exist or is empty
    if !crate_src_path.exists() || fs::metadata(&crate_src_path).map(|m| m.len() == 0).unwrap_or(true) {
        println!("cargo:warning=Downloading online_relays_gps.csv...");
        let url = "https://raw.githubusercontent.com/permissionlesstech/bitchat/main/relays/online_relays_gps.csv";
        match reqwest::get(url).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(content) => {
                            fs::write(&crate_src_path, content).expect("Unable to write online_relays_gps.csv");
                            println!("cargo:warning=Successfully downloaded online_relays_gps.csv to {:?}", crate_src_path);
                        },
                        Err(e) => {
                            println!("cargo:warning=Failed to get text from response: {}", e);
                        }
                    }
                } else {
                    println!("cargo:warning=Failed to download online_relays_gps.csv: HTTP status {}", response.status());
                }
            },
            Err(e) => {
                println!("cargo:warning=Failed to fetch online_relays_gps.csv: {}", e);
            }
        }
    }
}

#[cfg(not(all(not(debug_assertions), feature = "nostr")))]
fn main() {
    report_build_name();
    // Placeholder for when the nostr feature is not enabled or in debug mode.
}

fn report_build_name() {
    let now = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(val) => chrono::Local
            .timestamp_opt(val.parse::<i64>().unwrap(), 0)
            .unwrap(),
        Err(_) => chrono::Local::now(),
    };
    let build_date = now.date_naive();
    let build_name = if std::env::var("GITUI_RELEASE").is_ok() {
        format!(
            "{}-{}",
            env!("CARGO_PKG_NAME").replace('_', "-"),
            env!("CARGO_PKG_VERSION")
        )
    } else {
        format!(
            "{}-{} {} ({})",
            env!("CARGO_PKG_NAME").replace('_', "-"),
            env!("CARGO_PKG_VERSION"),
            build_date,
            get_git_hash()
        )
    };

    println!("cargo:warning=buildname '{build_name}'");
    println!("cargo:rustc-env=GITUI_BUILD_NAME={build_name}");
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
