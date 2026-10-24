#!/usr/bin/env python3
"""Run focused legit metadata/chain/composition tests without Cargo.

Run from any directory; CARGO_TARGET_DIR is respected. Additional arguments are
forwarded to the Rust test runner. First populate the cache with your normal
gnostr-legit build workflow. Relay-list configuration smoke tests are excluded.
"""

import base64
import hashlib
import json
import os
from pathlib import Path
import re
import queue
import shutil
import socket
import subprocess
import sys
import tempfile
import threading


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
    path = json.dumps(str(ROOT / relative_path), ensure_ascii=False)
    return f"#[path = {path}] pub mod {name};"


def harness():
    main = (ROOT / "legit/src/main.rs").read_text()
    main = re.sub(
        r"^mod (\w+);$",
        lambda match: source_module(match[1], f"legit/src/{match[1]}.rs").replace("pub mod", "mod", 1),
        main,
        flags=re.MULTILINE,
    )
    return main + "\n" + "\n".join(
        [
            "extern crate self as gnostr_types;",
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
            source_module("event_builder", "types/src/nostr/event_builder.rs"),
            "pub use event_builder::EventBuilder;",
            source_module("client", "types/src/nostr/client.rs"),
            "pub use client::Client;",
            source_module("patch", "types/src/nostr/patch.rs"),
            "}",
        ]
    )


def read_exact(connection, length):
    data = bytearray()
    while len(data) < length:
        part = connection.recv(length - len(data))
        if not part:
            raise RuntimeError("Relay connection closed before the complete event arrived")
        data.extend(part)
    return bytes(data)


def receive_event(listener, results):
    try:
        connection, _ = listener.accept()
        with connection:
            connection.settimeout(5)
            request = bytearray()
            while not request.endswith(b"\r\n\r\n"):
                request.extend(read_exact(connection, 1))
                if len(request) > 16384:
                    raise RuntimeError("Oversized WebSocket handshake")
            headers = dict(
                line.split(": ", 1)
                for line in request.decode().split("\r\n")[1:]
                if ": " in line
            )
            key = next(value for name, value in headers.items() if name.lower() == "sec-websocket-key")
            accept = base64.b64encode(
                hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()
            ).decode()
            connection.sendall(
                ("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n"
                 "Connection: Upgrade\r\nSec-WebSocket-Accept: " + accept + "\r\n\r\n").encode()
            )
            header = read_exact(connection, 2)
            if header[0] != 0x81 or not header[1] & 0x80:
                raise RuntimeError("Expected a complete, masked text WebSocket frame")
            length = header[1] & 0x7F
            if length == 126:
                length = int.from_bytes(read_exact(connection, 2), "big")
            elif length == 127:
                length = int.from_bytes(read_exact(connection, 8), "big")
            if length > 65536:
                raise RuntimeError("Unexpectedly large smoke-test event")
            mask = read_exact(connection, 4)
            body = read_exact(connection, length)
            results.put(json.loads(bytes(value ^ mask[index % 4] for index, value in enumerate(body))))
    except (OSError, RuntimeError, ValueError, StopIteration) as error:
        results.put(error)


def run_cli(args, environment):
    result = subprocess.run(args, capture_output=True, text=True, timeout=15, env=environment)
    if result.stdout:
        print(result.stdout, end="", flush=True)
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr, flush=True)
    return result


def smoke_cli(binary, directory):
    print("========== CHECKING CLI JSON / NO-GIT DRY RUN / OPT-IN PUBLICATION ==========", flush=True)
    absent_repo = directory / "not-a-git-repository"
    environment = dict(
        os.environ,
        HOME=str(directory / "home"),
        XDG_CONFIG_HOME=str(directory / "config"),
        XDG_DATA_HOME=str(directory / "data"),
        APPDATA=str(directory / "config"),
        LOCALAPPDATA=str(directory / "data"),
    )
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        relay = f"ws://127.0.0.1:{listener.getsockname()[1]}"
        # This is a publicly known test fixture, never a user signing key.
        args = [
            str(binary), "--repo", str(absent_repo), "--nsec",
            "473a0f4c3be8a93681a267e3b1e9a7dcda1185436fe141f7749120a303721813",
            "compose", "--relay", relay, "grasp-list", "--server", "wss://grasp.example.com",
        ]
        dry_run = run_cli(args, environment)
        dry_run.check_returncode()
        event = json.loads(dry_run.stdout)
        if event["kind"] != 10317 or ["t", "gnostr"] not in event["tags"] or len(event["id"]) != 64:
            raise RuntimeError(f"Invalid composed event: {dry_run.stdout}")
        print(f"Dry-run JSON event ID: {event['id']}", flush=True)
        if any(tag[0] == "nonce" for tag in event["tags"]):
            raise RuntimeError("Default composition unexpectedly added NIP-13 proof")
        args[1:1] = ["--event-difficulty", "8"]
        mined_run = run_cli(args, environment)
        mined_run.check_returncode()
        mined = json.loads(mined_run.stdout)
        if (
            256 - int(mined["id"], 16).bit_length() < 8
            or len([tag for tag in mined["tags"] if tag[0] == "nonce"]) != 1
            or next(tag for tag in mined["tags"] if tag[0] == "nonce")[2] != "8"
        ):
            raise RuntimeError("Composed event did not meet its NIP-13 target")
        print(f"Mined dry-run JSON event ID: {mined['id']}", flush=True)
        if absent_repo.exists():
            raise RuntimeError("Composition unexpectedly created a Git repository")
        listener.settimeout(0.25)
        try:
            connection, _ = listener.accept()
        except socket.timeout:
            pass
        else:
            connection.close()
            raise RuntimeError("Dry-run composition unexpectedly contacted a relay")

        missing_relay = run_cli([str(binary), "compose", "--publish", "grasp-list"], environment)
        if missing_relay.returncode == 0 or "--relay" not in missing_relay.stderr:
            raise RuntimeError("--publish did not require an explicit relay")

        listener.settimeout(5)
        results = queue.Queue()
        receiver = threading.Thread(target=receive_event, args=(listener, results))
        receiver.start()
        publish_args = args.copy()
        publish_args.insert(publish_args.index("compose") + 1, "--publish")
        try:
            published = run_cli(publish_args, environment)
        finally:
            receiver.join(timeout=10)
        published.check_returncode()
        received = results.get(timeout=1)
        if isinstance(received, Exception):
            raise received
        event = json.loads(published.stdout)
        if received != ["EVENT", event]:
            raise RuntimeError("The relay did not receive exactly the event printed as JSON")
        print(f"Opt-in relay received JSON event ID: {event['id']}", flush=True)


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
        for name in ["clap", "chrono", "crypto", "env_logger", "gnostr_asyncgit", "time_0_3"]:
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
        print("========== COMPILING LEGIT METADATA / CHAIN / COMPOSITION TESTS ==========", flush=True)
        package = (ROOT / "Cargo.toml").read_text().split("[workspace.package]", 1)[1].split("\n[", 1)[0]
        version = re.search(r'^version\s*=\s*"([^"]+)"', package, re.MULTILINE)
        if version is None:
            sys.exit("Could not read workspace.package.version from Cargo.toml")
        environment = dict(
            os.environ,
            CARGO_PKG_NAME="gnostr-legit",
            CARGO_PKG_VERSION=version[1],
            CARGO_MANIFEST_DIR=str(ROOT / "legit"),
        )
        subprocess.run(command, check=True, env=environment)
        print("========== RUNNING LEGIT METADATA / CHAIN / COMPOSITION TESTS ==========", flush=True)
        subprocess.run(
            [
                str(binary), "--nocapture",
                "--skip", "nostr::patch::tests::test_get_relay_urls_",
                *sys.argv[1:],
            ],
            check=True,
        )
        print("========== COMPILING CLI SMOKE-TEST BINARY ==========", flush=True)
        cli_binary = directory / ("gnostr-legit.exe" if os.name == "nt" else "gnostr-legit")
        cli_command = command.copy()
        cli_command.remove("--test")
        cli_command[cli_command.index("-o") + 1] = str(cli_binary)
        subprocess.run(cli_command, check=True, env=environment)
        smoke_cli(cli_binary, directory)
    print("========== LEGIT METADATA / CHAIN / COMPOSITION TESTS PASSED ==========", flush=True)


if __name__ == "__main__":
    main()
