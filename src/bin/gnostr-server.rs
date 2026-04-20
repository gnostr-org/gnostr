use std::{
    env,
    fs,
    path::PathBuf,
    process::{Command, ExitStatus},
};

fn env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn default_blossom_data_dir() -> String {
    dirs::home_dir()
        .map(|home| home.join(".gnostr/blossom/blobs").to_string_lossy().into_owned())
        .unwrap_or_else(|| "/var/lib/blossom/blobs".to_string())
}

fn default_blossom_db_path() -> String {
    dirs::home_dir()
        .map(|home| home.join(".gnostr/blossom/blossom.db").to_string_lossy().into_owned())
        .unwrap_or_else(|| "/var/lib/blossom/blossom.db".to_string())
}

fn blossom_server_args() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let bind = env_or_default("BLOSSOM_BIND", "0.0.0.0:3000");
    let base_url = env_or_default("BLOSSOM_BASE_URL", "http://localhost:3000");
    let data_dir = env::var("BLOSSOM_DATA_DIR").unwrap_or_else(|_| default_blossom_data_dir());
    let db_path = env::var("BLOSSOM_DB_PATH").unwrap_or_else(|_| default_blossom_db_path());
    let log_level = env_or_default("BLOSSOM_LOG_LEVEL", "info");

    fs::create_dir_all(&data_dir)?;
    if let Some(parent) = PathBuf::from(&db_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let mut args = vec![
        "--bind".to_string(),
        bind,
        "--base-url".to_string(),
        base_url,
        "--data-dir".to_string(),
        data_dir,
        "--db-path".to_string(),
        db_path,
        "--log-level".to_string(),
        log_level,
    ];

    args.extend(env::args().skip(1));

    if let Ok(extra_args) = env::var("BLOSSOM_EXTRA_ARGS") {
        if !extra_args.trim().is_empty() {
            args.extend(shellwords::split(&extra_args)?);
        }
    }

    Ok(args)
}

fn exit_with_status(status: ExitStatus) -> ! {
    match status.code() {
        Some(code) => std::process::exit(code),
        None => std::process::exit(1),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = blossom_server_args()?;
    let status = Command::new("blossom-server").args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        exit_with_status(status)
    }
}
