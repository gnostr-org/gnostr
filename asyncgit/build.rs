use std::path::{Path, PathBuf};

use anyhow::Context;
use sha2::{Digest, Sha256};

const NIP44_VECTORS_SHA256: &str = "269ed0f69e4c192512cc779e78c555090cebc7c785b609e338a62afc3ce25040";

#[derive(Copy, Clone)]
pub struct Paths<'a> {
    statics_in_dir: &'a Path,
    statics_out_dir: &'a Path,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("An error occurred within the rgit build script:\n\n{:?}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    println!("cargo:rustc-check-cfg=cfg(gnostr_workspace_assets)");

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?);
    verify_nip44_vectors(&manifest_dir).context("Failed to verify NIP-44 test vectors")?;
    export_filehash_envs(&manifest_dir).context("Failed to export filehash build envs")?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
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
