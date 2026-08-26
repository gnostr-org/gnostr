use std::path::PathBuf;

use anyhow::Context;
use sha2::{Digest, Sha256};

const NIP44_VECTORS_SHA256: &str = "269ed0f69e4c192512cc779e78c555090cebc7c785b609e338a62afc3ce25040";

fn main() {
    if let Err(e) = run() {
        eprintln!("An error occurred within the gnostr-types build script:\n\n{:?}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?);

    verify_nip44_vectors(&manifest_dir).context("Failed to verify NIP-44 test vectors")?;
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn verify_nip44_vectors(manifest_dir: &std::path::Path) -> anyhow::Result<()> {
    let vectors_path = manifest_dir.join("src/nostr/nip44/nip44.vectors.json");
    println!("cargo:rerun-if-changed={}", vectors_path.display());

    let vectors = std::fs::read(&vectors_path).context("Failed to read NIP-44 vectors")?;
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
