use std::env;
use std::process::{exit, Command};
use std::time::{SystemTime, UNIX_EPOCH};

/// Orchestrates unescaping in the established sequence
fn unescape_sequences(s: &str) -> String {
    let mut result = s.to_string();
    result = unescape_backslashes(&result);
    result = unescape_newlines(&result);
    result = unescape_tabs(&result);
    result = unescape_carriage_returns(&result);
    result = unescape_hex(&result);
    result
}

fn unescape_backslashes(s: &str) -> String {
    s.replace("\\\\", "\\")
}
fn unescape_newlines(s: &str) -> String {
    s.replace("\\n", "\n")
}
fn unescape_tabs(s: &str) -> String {
    s.replace("\\t", "\t")
}
fn unescape_carriage_returns(s: &str) -> String {
    s.replace("\\r", "\n")
}
fn unescape_hex(s: &str) -> String {
    s.replace("\\x1b", "\x1b")
}

fn ensure_gnostr_exists() -> bool {
    if Command::new("gnostr").arg("--version").output().is_ok() {
        return true;
    }
    println!("gnostr not found. Attempting: cargo install gnostr...");
    let install_status = Command::new("cargo").args(["install", "gnostr"]).status();
    matches!(install_status, Ok(s) if s.success())
}

fn get_gnostr_metadata(flag: &str) -> String {
    if !ensure_gnostr_exists() {
        return fallback_metadata(flag);
    }
    match Command::new("gnostr").arg(flag).output() {
        Ok(out) if out.status.success() => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() || val == "unknown" {
                fallback_metadata(flag)
            } else {
                val
            }
        }
        _ => fallback_metadata(flag),
    }
}

fn fallback_metadata(flag: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    match flag {
        "--blockheight" => "0000000".to_string(),
        _ => format!("{:x}", now),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut messages: Vec<String> = Vec::new();
    let mut pow: Option<String> = None;
    let mut commit_prefix: Option<String> = None;
    let mut threads: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        if (args[i] == "-m" || args[i] == "--message") && i + 1 < args.len() {
            messages.push(unescape_sequences(&args[i + 1]));
            i += 2;
        } else if args[i] == "--pow" && i + 1 < args.len() {
            pow = Some(args[i + 1].clone());
            i += 2;
        } else if args[i] == "--prefix" && i + 1 < args.len() {
            commit_prefix = Some(args[i + 1].clone());
            i += 2;
        } else if (args[i] == "-t" || args[i] == "--threads") && i + 1 < args.len() {
            threads = Some(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    let mut summary = if messages.is_empty() {
        "gnostr legit commit".to_string()
    } else {
        messages.join("\n\n")
    };

    // IDEMPOTENCY: Strip existing Sovereign prefix
    if let Some(pos) = summary.find(':') {
        let prefix_part = &summary[..pos];
        if prefix_part.contains('/') {
            summary = summary[pos + 1..].to_string();
        }
    }

    let weeble = get_gnostr_metadata("--weeble");
    let blockheight = get_gnostr_metadata("--blockheight");
    let wobble = get_gnostr_metadata("--wobble");

    let metadata_prefix = format!("{}/{}/{}", weeble, blockheight, wobble);
    let final_message = format!("{}:{}", metadata_prefix, summary);

    if ensure_gnostr_exists() {
        let mut command = Command::new("gnostr");
        command.arg("legit").arg("-m").arg(&final_message);

        if let Some(pow) = pow.as_ref() {
            command.args(["--pow", pow]);
        }

        if let Some(prefix) = commit_prefix.as_ref() {
            command.args(["--prefix", prefix]);
        }

        if let Some(threads) = threads.as_ref() {
            command.args(["--threads", threads]);
        }

        let status = command.status();

        if let Ok(s) = status {
            if s.success() {
                return;
            }
        }
    }
    exit(1);
}
