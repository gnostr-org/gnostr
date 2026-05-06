use chrono::TimeZone;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use sha2::{Digest, Sha256};

const NIP44_VECTORS_SHA256: &str = "269ed0f69e4c192512cc779e78c555090cebc7c785b609e338a62afc3ce25040";

fn main() {
    if let Err(e) = run() {
        eprintln!("An error occurred within the rgit build script:\n\n{:?}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    report_build_name();
    println!("cargo:rustc-check-cfg=cfg(gnostr_workspace_assets)");

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?);
    verify_nip44_vectors(&manifest_dir).context("Failed to verify NIP-44 test vectors")?;
    export_filehash_envs(&manifest_dir).context("Failed to export filehash build envs")?;
    export_git_envs(&manifest_dir).context("Failed to export git build envs")?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
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
        format!("{}-{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    } else {
        format!(
            "{}-{} {} ({})",
            env!("CARGO_PKG_NAME"),
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

fn export_filehash_envs(manifest_dir: &Path) -> anyhow::Result<()> {
    let build_rs = manifest_dir.join("build.rs");
    let cargo_toml = manifest_dir.join("Cargo.toml");
    let lib_mod = manifest_dir.join("src/lib/mod.rs");

    println!("cargo:rerun-if-changed={}", build_rs.display());
    println!("cargo:rerun-if-changed={}", cargo_toml.display());
    println!("cargo:rerun-if-changed={}", lib_mod.display());

    println!("cargo:rustc-env=BUILD_HASH={}", hash_file(&build_rs)?);
    println!("cargo:rustc-env=CARGO_TOML_HASH={}", hash_file(&cargo_toml)?);
    println!("cargo:rustc-env=LIB_HASH={}", hash_file(&lib_mod)?);
    Ok(())
}

fn export_git_envs(manifest_dir: &Path) -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed={}", manifest_dir.join(".git/HEAD").display());

    let git_commit_hash = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(manifest_dir)
        .output()
        .ok()
        .and_then(|output| output.status.success().then_some(output.stdout))
        .map(|stdout| String::from_utf8_lossy(&stdout).trim().to_string())
        .unwrap_or_default();

    let git_branch = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(manifest_dir)
        .output()
        .ok()
        .and_then(|output| output.status.success().then_some(output.stdout))
        .map(|stdout| String::from_utf8_lossy(&stdout).trim().to_string())
        .filter(|branch| !branch.is_empty())
        .unwrap_or_else(|| "HEAD".to_string());

    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_commit_hash);
    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);
    Ok(())
}

fn hash_file(path: &Path) -> anyhow::Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

fn verify_nip44_vectors(manifest_dir: &Path) -> anyhow::Result<()> {
    let vectors_path = manifest_dir.join("src/lib/types/nip44/nip44.vectors.json");
    println!("cargo:rerun-if-changed={}", vectors_path.display());

    let vectors = std::fs::read(&vectors_path).context("Failed to read NIP-44 vectors")?;
    // Normalize line endings to LF before hashing so the hash is consistent
    // across platforms (Windows checks out files with CRLF by default).
    let vectors: Vec<u8> = vectors.into_iter().filter(|&b| b != b'\r').collect();
    let actual = Sha256::digest(&vectors);
    let actual = format!("{:x}", actual);

    anyhow::ensure!(
        actual == NIP44_VECTORS_SHA256,
        "NIP-44 vector hash mismatch: expected {}, got {}",
        NIP44_VECTORS_SHA256,
        actual
    );

    Ok(())
}
