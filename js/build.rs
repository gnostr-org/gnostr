use chrono::TimeZone;
use gnostr_asyncgit::types::{Client, EventBuilder, EventKind, Keys, Tag};
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=GNOSTR_JS_NSEC");
    println!("cargo:rerun-if-env-changed=GNOSTR_JS_RELAYS");
    println!("cargo:rerun-if-env-changed=NSEC");
    println!("cargo:rerun-if-env-changed=NOSTR_RELAYS");
    println!("cargo:rerun-if-env-changed=RELAYS");

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
    emit_patch_event(&build_name);
    watch_sources(Path::new("src/js"), &["js"]);
    watch_sources(Path::new("src/css"), &["css"]);
}

fn emit_patch_event(build_name: &str) {
    if std::env::var("PROFILE").as_deref() != Ok("release") {
        return;
    }

    let Some(private_key) = load_private_key() else {
        println!("cargo:warning=skipping Nostr patch event: no private key configured");
        return;
    };

    let relays = configured_relays();
    if relays.is_empty() {
        println!("cargo:warning=skipping Nostr patch event: no relays configured");
        return;
    }

    let Ok(runtime) = tokio::runtime::Runtime::new() else {
        println!("cargo:warning=skipping Nostr patch event: failed to start runtime");
        return;
    };

    let result = runtime.block_on(async move {
        let keys = Keys::new(private_key.clone());
        let mut client = Client::new(&keys, gnostr_asyncgit::types::client::Options::new());
        client
            .add_relays(relays)
            .await
            .map_err(|error| error.to_string())?;

        let head_commit = git_output(["rev-parse", "HEAD"]).unwrap_or_else(|| get_git_hash());
        let parent_commit = git_output(["rev-parse", "HEAD^"]).unwrap_or_default();
        let branch_name = git_output(["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_default()
            .replace("refs/heads/", "");
        let branch_name_without_id_or_prefix = if branch_name.is_empty()
            || branch_name == "HEAD"
            || branch_name == "main"
            || branch_name == "master"
        {
            safe_branch_name_for_pr(build_name)
        } else {
            safe_branch_name_for_pr(&branch_name)
        };
        let clone_url = git_output(["remote", "get-url", "origin"]).unwrap_or_default();
        let patch_body = git_output(["diff", "--binary", "--no-ext-diff", "--no-color", "HEAD"])
            .unwrap_or_default();
        let content = if patch_body.trim().is_empty() {
            format!("From {head_commit}\n\nBuild: {build_name}\n")
        } else {
            patch_body
        };

        let mut tags = vec![Tag::new_tag("commit", &head_commit)];
        if !parent_commit.is_empty() {
            tags.push(Tag::new_tag("parent-commit", &parent_commit));
        }
        if !clone_url.is_empty() {
            tags.push(Tag::new_tag("clone", &clone_url));
        }
        tags.push(Tag::new_tag("branch-name", &branch_name_without_id_or_prefix));
        tags.push(Tag::new_tag("build-name", build_name));

        let event = EventBuilder::new(EventKind::Patches, content, tags)
            .to_event(&private_key)
            .map_err(|error| error.to_string())?;

        client
            .send_event(event)
            .await
            .map_err(|error| error.to_string())?;

        Ok::<(), String>(())
    });

    if let Err(error) = result {
        println!("cargo:warning=failed to emit Nostr patch event: {error}");
    }
}

fn load_private_key() -> Option<gnostr_asyncgit::types::PrivateKey> {
    for key_var in ["GNOSTR_JS_NSEC", "GNOSTR_NSEC", "NSEC"] {
        let Ok(value) = std::env::var(key_var) else {
            continue;
        };

        if let Some(keys) = Keys::parse(value) {
            if let Ok(private_key) = keys.secret_key() {
                return Some(private_key);
            }
        }
    }

    None
}

fn configured_relays() -> Vec<String> {
    for relays_var in ["GNOSTR_JS_RELAYS", "NOSTR_RELAYS", "RELAYS"] {
        if let Ok(value) = std::env::var(relays_var) {
            let relays = value
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|relay| !relay.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            if !relays.is_empty() {
                return relays;
            }
        }
    }

    vec!["wss://relay.damus.io".to_string(), "wss://nos.lol".to_string()]
}

fn git_output<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn safe_branch_name_for_pr(s: &str) -> String {
    s.replace(' ', "-")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '/' {
                c
            } else {
                '-'
            }
        })
        .take(60)
        .collect()
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
