//! Pure utility functions — no App or UI dependencies.

use crate::nip19;

/// Format a Unix timestamp as a compact date string (YYYY-MM-DD HH:MM).
pub fn chrono_fmt_unix(ts: u64) -> String {
    let secs = ts as i64;
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hh = time_of_day / 3600;
    let mm = (time_of_day % 3600) / 60;
    let mut y = 1970i64;
    let mut d = days_since_epoch;
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let months = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut mo = 0usize;
    for &dim in &months {
        if d < dim {
            break;
        }
        d -= dim;
        mo += 1;
    }
    format!("{y}-{:02}-{:02} {:02}:{:02}", mo + 1, d + 1, hh, mm)
}

/// Guess a MIME type from the file extension.
pub fn mime_from_path(path: &std::path::Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("ogg") => "audio/ogg",
        Some("pdf") => "application/pdf",
        Some("txt") | Some("md") => "text/plain",
        Some("json") => "application/json",
        Some("html") | Some("htm") => "text/html",
        _ => "application/octet-stream",
    }
    .into()
}

/// Human-readable byte size.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes < KB {
        format!("{bytes} B")
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

/// Format a Unix timestamp as `YYYY-MM-DD HH:MM`.
pub fn format_unix_ts(ts: u64) -> String {
    let secs = ts;
    let mins = secs / 60;
    let hours = mins / 60;
    let days_total = hours / 24;
    let minute = mins % 60;
    let hour = hours % 24;
    let (year, month, day) = days_to_ymd(days_total);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
}

/// Convert days since 1970-01-01 to (year, month, day).
/// Algorithm from <https://howardhinnant.github.io/date_algorithms.html>
pub fn days_to_ymd(d: u64) -> (u64, u64, u64) {
    let z = d + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    (year, month, day)
}

/// Encode a hex secret key as `nsec1` bech32.
pub fn encode_nsec(hex_key: &str) -> Result<String, String> {
    nip19::seckey_to_nsec(hex_key).map_err(|e| e.to_string())
}

/// Decode a secret key from hex or `nsec1` bech32.
pub fn decode_secret_key(input: &str) -> Result<String, String> {
    if input.starts_with("nsec1") {
        nip19::nsec_to_seckey(input).map_err(|e| e.to_string())
    } else {
        if input.len() != 64 || !input.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("invalid hex key: expected 64 hex characters".into());
        }
        Ok(input.to_string())
    }
}
