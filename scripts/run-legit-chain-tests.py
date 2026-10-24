#!/usr/bin/env python3
"""Run focused legit metadata/chain tests with cached dependencies, without Cargo.

Run from any directory; CARGO_TARGET_DIR is respected. Additional arguments are
forwarded to the Rust test runner. First populate the cache with your normal
gnostr-legit build workflow. Relay-list configuration smoke tests are excluded.
"""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parent.parent
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))
if not TARGET.is_absolute():
    TARGET = Path.cwd() / TARGET
FINGERPRINTS = TARGET / "debug/.fingerprint"
ARTIFACTS = TARGET / "debug/deps"


def dependency(name, manifest):
    fingerprint = next(dep[3] for dep in manifest["deps"] if dep[1] == name)
    actual = {"time_0_3": "time"}.get(name, name)
    for path in FINGERPRINTS.glob(f"*/lib-{actual}"):
        raw = path.read_text().strip()
        if int.from_bytes(bytes.fromhex(raw), "little") != fingerprint:
            continue
        suffix = path.parent.name.rsplit("-", 1)[1]
        library = ARTIFACTS / f"lib{actual}-{suffix}.rlib"
        if library.exists():
            return library
    raise RuntimeError(
        f"Missing matching cached dependency {name}; "
        "populate the cache with your normal gnostr-legit build workflow."
    )


def source_module(name, relative_path):
    path = json.dumps(str(ROOT / relative_path))
    return f"#[path = {path}] pub mod {name};"


def harness():
    return "\n".join(
        [
            "extern crate self as gnostr_types;",
            "#[macro_use] extern crate log;",
            "pub use cached_gnostr_types::test_serde;",
            "pub mod nostr {",
            "pub use cached_gnostr_types::nostr::*;",
            "pub static HRP_LNURL: std::sync::LazyLock<bech32::Hrp> =",
            'std::sync::LazyLock::new(|| bech32::Hrp::parse("lnurl").unwrap());',
            "pub mod private_key { pub mod content_encryption {",
            "pub use cached_gnostr_types::nostr::ContentEncryptionAlgorithm;",
            "}}",
            source_module("metadata_v1", "types/src/nostr/versioned/metadata.rs"),
            "pub use metadata_v1::MetadataV1 as Metadata;",
            source_module("client", "types/src/nostr/client.rs"),
            "pub use client::Client;",
            source_module("patch", "types/src/nostr/patch.rs"),
            "}",
            source_module("gitminer", "legit/src/gitminer.rs"),
            source_module("worker", "legit/src/worker.rs"),
        ]
    )


def main():
    libraries = list(ARTIFACTS.glob("libgnostr_legit-*.rlib"))
    if not libraries:
        sys.exit(
            f"No cached gnostr-legit library in {ARTIFACTS}; "
            "populate the cache with your normal build workflow first."
        )
    library = max(libraries, key=lambda path: path.stat().st_mtime)
    legit_hash = library.stem.rsplit("-", 1)[1]
    legit = json.loads(
        (FINGERPRINTS / f"gnostr-legit-{legit_hash}/lib-gnostr_legit.json").read_text()
    )
    types_library = dependency("gnostr_types", legit)
    types_hash = types_library.stem.rsplit("-", 1)[1]
    types = json.loads(
        (FINGERPRINTS / f"gnostr-types-{types_hash}/lib-gnostr_types.json").read_text()
    )

    with tempfile.TemporaryDirectory(prefix="gnostr-legit-chain-tests-") as directory:
        directory = Path(directory)
        source = directory / "legit-chain-tests.rs"
        binary = directory / ("legit-chain-tests.exe" if os.name == "nt" else "legit-chain-tests")
        source.write_text(harness())
        command = [
            "rustc", "--edition=2024", "--test", str(source),
            "-L", f"dependency={ARTIFACTS}",
            "--extern", f"cached_gnostr_types={types_library}",
            "-o", str(binary),
        ]
        for name in [
            "anyhow", "bech32", "directories", "futures", "futures_util", "log",
            "rand", "rustls", "secp256k1", "serde", "serde_json", "serde_yaml",
            "tempfile", "tokio", "tokio_tungstenite", "tracing", "url",
        ]:
            command.extend(["--extern", f"{name}={dependency(name, types)}"])
        for name in ["crypto", "gnostr_asyncgit", "time_0_3"]:
            command.extend(["--extern", f"{name}={dependency(name, legit)}"])
        if shutil.which("pkg-config"):
            native = subprocess.run(
                ["pkg-config", "--libs-only-L", "libgit2"],
                capture_output=True, text=True,
            )
            if native.returncode == 0:
                for flag in native.stdout.split():
                    command.extend(["-L", f"native={flag[2:]}"])
            else:
                print(
                    "Warning: system libgit2 not found; attempting cached native linkage.",
                    file=sys.stderr,
                )
        print("========== COMPILING FOCUSED METADATA / PARENT-CHAIN TESTS ==========", flush=True)
        subprocess.run(command, check=True)
        print("========== RUNNING FOCUSED METADATA / PARENT-CHAIN TESTS ==========", flush=True)
        subprocess.run(
            [
                str(binary), "--nocapture",
                "--skip", "nostr::patch::tests::test_get_relay_urls_",
                *sys.argv[1:],
            ],
            check=True,
        )
    print("========== METADATA / PARENT-CHAIN TESTS PASSED ==========", flush=True)


if __name__ == "__main__":
    main()
