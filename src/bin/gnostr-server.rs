use std::{
    env,
    fs,
    path::PathBuf,
    process::{Command, ExitStatus},
    thread,
};

fn env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

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

fn blossom_server_args() -> Result<(Vec<String>, bool, bool, String), Box<dyn std::error::Error>> {
    let bind = env_or_default("BLOSSOM_BIND", "0.0.0.0:3000");
    let mut base_url = env_or_default("BLOSSOM_BASE_URL", "http://localhost:3000");
    let data_dir = env::var("BLOSSOM_DATA_DIR").unwrap_or_else(|_| default_blossom_data_dir());
    let db_path = env::var("BLOSSOM_DB_PATH").unwrap_or_else(|_| default_blossom_db_path());
    let log_level = env_or_default("BLOSSOM_LOG_LEVEL", "info");
    let mut detach = false;
    let mut advertise_service = false;

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

    let cli_args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < cli_args.len() {
        match cli_args[i].as_str() {
            "--detach" => {
                detach = true;
            }
            "--advertise-service" => {
                advertise_service = true;
            }
            "--base-url" => {
                if let Some(value) = cli_args.get(i + 1) {
                    base_url = value.clone();
                    args.push(cli_args[i].clone());
                    args.push(value.clone());
                    i += 1;
                } else {
                    args.push(cli_args[i].clone());
                }
            }
            _ => args.push(cli_args[i].clone()),
        }
        i += 1;
    }

    if let Ok(extra_args) = env::var("BLOSSOM_EXTRA_ARGS") {
        if !extra_args.trim().is_empty() {
            let extra_args = shellwords::split(&extra_args)?;
            let mut i = 0;
            while i < extra_args.len() {
                match extra_args[i].as_str() {
                    "--detach" => {
                        detach = true;
                    }
                    "--advertise-service" => {
                        advertise_service = true;
                    }
                    "--base-url" => {
                        if let Some(value) = extra_args.get(i + 1) {
                            base_url = value.clone();
                            args.push(extra_args[i].clone());
                            args.push(value.clone());
                            i += 1;
                        } else {
                            args.push(extra_args[i].clone());
                        }
                    }
                    _ => args.push(extra_args[i].clone()),
                }
                i += 1;
            }
        }
    }

    Ok((args, detach, advertise_service, base_url))
}

fn exit_with_status(status: ExitStatus) -> ! {
    match status.code() {
        Some(code) => std::process::exit(code),
        None => std::process::exit(1),
    }
}

fn spawn_advertiser_thread(base_url: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        if let Err(e) = rt.block_on(gnostr::p2p::advertise_service(
            "blossom-server".to_string(),
            base_url,
        )) {
            eprintln!("gnostr-server advertiser failed: {e}");
        }
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (args, detach, advertise_service, base_url) = blossom_server_args()?;

    if advertise_service {
        let rt = tokio::runtime::Runtime::new()?;
        return rt
            .block_on(gnostr::p2p::advertise_service(
                "blossom-server".to_string(),
                base_url,
            ))
            .map(|_| ());
    }

    if detach {
        let _advertiser_pid = gnostr::utils::detach::spawn_detached_current_exe_named_with_env(
            Some("gnostr-server-advertiser"),
            vec!["--advertise-service".to_string()],
            [("BLOSSOM_BASE_URL", base_url.as_str())],
        )?;
        let pid = gnostr::utils::detach::spawn_detached("blossom-server", args)?;
        println!("gnostr-server: started background server (pid: {})", pid);
        Ok(())
    } else {
        let _advertiser = spawn_advertiser_thread(base_url);
        let status = Command::new("blossom-server").args(args).status()?;
        if status.success() {
            Ok(())
        } else {
            exit_with_status(status)
        }
    }
}
