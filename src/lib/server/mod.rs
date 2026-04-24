use std::{
    env,
    fs,
    path::PathBuf,
    process::{Command, ExitStatus},
    thread,
};

use which::which;

const BLOSSOM_VERSION: &str = "0.5.6";

fn default_blossom_data_dir() -> String {
    app_dirs()
        .map(|dirs| dirs.data_local_dir().join("blossom/blobs").to_string_lossy().into_owned())
        .unwrap_or_else(|| "/var/lib/blossom/blobs".to_string())
}

fn default_blossom_db_path() -> String {
    app_dirs()
        .map(|dirs| dirs.data_local_dir().join("blossom/blossom.db").to_string_lossy().into_owned())
        .unwrap_or_else(|| "/var/lib/blossom/blossom.db".to_string())
}

fn app_dirs() -> Option<directories::ProjectDirs> {
    directories::ProjectDirs::from("org", "gnostr", "gnostr")
}

fn blossom_server_args(
    mut args: Vec<String>,
) -> Result<(Vec<String>, bool, bool, String, Option<String>), Box<dyn std::error::Error>> {
    let mut detach = false;
    let mut advertise_service = false;
    let mut service_name = None;

    if let Ok(extra_args) = env::var("BLOSSOM_EXTRA_ARGS") {
        if !extra_args.trim().is_empty() {
            args.extend(shellwords::split(&extra_args)?);
        }
    }

    let bind = value_for_any(&args, &["-b", "--bind"])
        .or_else(|| env::var("BLOSSOM_BIND").ok())
        .unwrap_or_else(|| "0.0.0.0:3000".to_string());
    let bind = normalize_bind_addr(&bind)?;
    let base_url = value_for_any(&args, &["-u", "--base-url"])
        .or_else(|| env::var("BLOSSOM_BASE_URL").ok())
        .unwrap_or_else(|| default_base_url_from_bind(&bind));
    let data_dir = value_for_any(&args, &["-d", "--data-dir"])
        .or_else(|| env::var("BLOSSOM_DATA_DIR").ok())
        .unwrap_or_else(default_blossom_data_dir);
    let db_path = value_for_any(&args, &["--db-path"])
        .or_else(|| env::var("BLOSSOM_DB_PATH").ok())
        .unwrap_or_else(default_blossom_db_path);
    let log_level = value_for_any(&args, &["--log-level"])
        .or_else(|| env::var("BLOSSOM_LOG_LEVEL").ok())
        .unwrap_or_else(|| "info".to_string());

    let mut filtered_args = Vec::with_capacity(args.len());
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--detach" => {
                detach = true;
            }
            "--advertise-service" => {
                advertise_service = true;
            }
            "--name" => {
                if let Some(value) = args.get(i + 1) {
                    service_name = Some(value.clone());
                    i += 1;
                }
            }
            _ => filtered_args.push(args[i].clone()),
        }
        i += 1;
    }
    args = filtered_args;

    let mut defaults = Vec::new();
    if !has_any(&args, &["-b", "--bind"]) {
        defaults.extend(["--bind".to_string(), bind]);
    }
    if !has_any(&args, &["-u", "--base-url"]) {
        defaults.extend(["--base-url".to_string(), base_url.clone()]);
    }
    if !has_any(&args, &["-d", "--data-dir"]) {
        defaults.extend(["--data-dir".to_string(), data_dir.clone()]);
    }
    if !has_any(&args, &["--db-path"]) {
        defaults.extend(["--db-path".to_string(), db_path.clone()]);
    }
    if !has_any(&args, &["--log-level"]) {
        defaults.extend(["--log-level".to_string(), log_level]);
    }
    defaults.extend(args);
    args = defaults;

    fs::create_dir_all(&data_dir)?;
    if let Some(parent) = PathBuf::from(&db_path).parent() {
        fs::create_dir_all(parent)?;
    }

    Ok((args, detach, advertise_service, base_url, service_name))
}

fn normalize_bind_addr(bind: &str) -> Result<String, Box<dyn std::error::Error>> {
    if bind.parse::<std::net::SocketAddr>().is_ok() {
        return Ok(bind.to_string());
    }

    if let Ok(port) = bind.parse::<u16>() {
        return Ok(format!("0.0.0.0:{port}"));
    }

    if let Some(port) = bind.strip_prefix(':').and_then(|value| value.parse::<u16>().ok()) {
        return Ok(format!("0.0.0.0:{port}"));
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("invalid socket address: {bind}"),
    )
    .into())
}

fn default_base_url_from_bind(bind: &str) -> String {
    bind.rsplit_once(':')
        .and_then(|(_, port)| port.parse::<u16>().ok())
        .map(|port| format!("http://localhost:{port}"))
        .unwrap_or_else(|| "http://localhost:3000".to_string())
}

fn has_any(args: &[String], flags: &[&str]) -> bool {
    args.iter().any(|arg| flags.iter().any(|flag| arg == flag))
}

fn value_for_any(args: &[String], flags: &[&str]) -> Option<String> {
    let mut value = None;
    let mut i = 0usize;
    while i < args.len() {
        if flags.iter().any(|flag| args[i] == *flag) {
            if let Some(next) = args.get(i + 1) {
                value = Some(next.clone());
                i += 1;
            }
        }
        i += 1;
    }
    value
}

fn exit_with_status(status: ExitStatus) -> ! {
    match status.code() {
        Some(code) => std::process::exit(code),
        None => std::process::exit(1),
    }
}

fn cargo_bin_path(binary: &str) -> Option<PathBuf> {
    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        return Some(PathBuf::from(cargo_home).join("bin").join(binary));
    }

    directories::BaseDirs::new().map(|dirs| dirs.home_dir().join(".cargo/bin").join(binary))
}

fn ensure_blossom_server_installed() -> Result<(), Box<dyn std::error::Error>> {
    if which("blossom-server").is_ok() {
        return Ok(());
    }

    let install_spec = format!("blossom-server@{BLOSSOM_VERSION}");
    let mut installer = if which("cargo-binstall").is_ok() {
        let mut command = Command::new("cargo");
        command.args(["binstall", "--no-confirm", "--locked", install_spec.as_str()]);
        command
    } else {
        let mut command = Command::new("cargo");
        command.args(["install", "--locked", "blossom-server", "--version", BLOSSOM_VERSION]);
        command
    };

    let status = installer.status()?;
    if status.success() && which("blossom-server").is_ok() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "failed to install blossom-server {BLOSSOM_VERSION} (status: {status})"
        ))
        .into())
    }
}

fn blossom_server_binary() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(path) = which("blossom-server") {
        return Ok(path);
    }

    ensure_blossom_server_installed()?;

    if let Ok(path) = which("blossom-server") {
        return Ok(path);
    }

    if let Some(path) = cargo_bin_path("blossom-server") {
        if path.exists() {
            return Ok(path);
        }
    }

    Err(std::io::Error::other("blossom-server was installed but could not be located").into())
}

/// Print the upstream blossom-server help output.
pub fn print_help() -> Result<(), Box<dyn std::error::Error>> {
    let blossom_server = blossom_server_binary()?;
    let status = Command::new(blossom_server).arg("--help").status()?;
    if status.success() {
        Ok(())
    } else {
        exit_with_status(status)
    }
}

fn spawn_advertiser_thread(base_url: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        if let Err(e) = rt.block_on(crate::p2p::advertise_service(
            "blossom-server".to_string(),
            base_url,
        )) {
            eprintln!("gnostr-server advertiser failed: {e}");
        }
    })
}

/// Run the gnostr server wrapper around upstream `blossom-server`.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    run_with_args(env::args().skip(1).collect())
}

/// Run the gnostr server wrapper around upstream `blossom-server` with explicit args.
pub fn run_with_args(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let (args, detach, advertise_service, base_url, service_name) = blossom_server_args(args)?;
    let process_name = service_name
        .as_deref()
        .map(|name| format!("gnostr-server-{name}"));

    if advertise_service {
        let rt = tokio::runtime::Runtime::new()?;
        return rt
            .block_on(crate::p2p::advertise_service(
                "blossom-server".to_string(),
                base_url,
            ))
            .map(|_| ());
    }

    let blossom_server = blossom_server_binary()?;

    if detach {
        let _advertiser_pid = crate::utils::detach::spawn_detached_current_exe_named_with_env(
            process_name
                .as_deref()
                .map(|name| format!("{name}-advertiser")),
            vec!["--advertise-service".to_string()],
            [("BLOSSOM_BASE_URL", base_url.as_str())],
        )?;
        let pid = crate::utils::detach::spawn_detached_named(
            &blossom_server,
            process_name.as_deref(),
            args,
        )?;
        println!("gnostr-server: started background server (pid: {})", pid);
        Ok(())
    } else {
        let _advertiser = spawn_advertiser_thread(base_url);
        let status = Command::new(&blossom_server).args(args).status()?;
        if status.success() {
            Ok(())
        } else {
            exit_with_status(status)
        }
    }
}
