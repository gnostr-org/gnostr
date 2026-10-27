pub mod agent;
pub mod cli;
pub mod docker;
pub mod sub_commands;
pub mod mcp;
pub mod openai;

pub mod app_data;
pub mod app_error;
pub mod config;
pub mod docker_data;
pub mod exec;
pub mod input_handler;
pub mod ui;

use app_data::AppData;
use app_error::AppError;
use bollard::{API_DEFAULT_VERSION, Docker};
use config::Config;
use docker_data::{DockerData, DockerMessage};
use parking_lot::Mutex;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{Level, error, info};
use ui::{GuiState, Rerender, Status, Ui};

pub const ENTRY_POINT: &str = "/app/oxker";
pub const ENV_KEY: &str = "OXKER_RUNTIME";
pub const ENV_VALUE: &str = "container";
pub const DOCKER_HOST: &str = "DOCKER_HOST";

pub fn setup_tracing() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
}

fn read_docker_host(config: &Config) -> Option<String> {
    if let Some(x) = &config.host {
        Some(x.to_string())
    } else if let Ok(env) = std::env::var(DOCKER_HOST)
        && !env.trim().is_empty()
    {
        Some(env)
    } else {
        None
    }
}

async fn docker_init(
    app_data: &Arc<Mutex<AppData>>,
    docker_rx: Receiver<DockerMessage>,
    docker_tx: Sender<DockerMessage>,
    gui_state: &Arc<Mutex<GuiState>>,
) {
    let host = read_docker_host(&app_data.lock().config);

    if let Ok(docker) = host
        .as_ref()
        .map_or_else(Docker::connect_with_defaults, |host| {
            Docker::connect_with_socket(host, 120, API_DEFAULT_VERSION)
        })
        && docker.ping().await.is_ok()
    {
        tokio::spawn(DockerData::start(
            Arc::clone(app_data),
            docker,
            docker_rx,
            docker_tx,
            Arc::clone(gui_state),
        ));
    } else {
        app_data.lock().set_error(
            AppError::DockerConnect,
            gui_state,
            Status::DockerConnect(host),
        );
    }
}

fn handler_init(
    app_data: &Arc<Mutex<AppData>>,
    docker_sx: &Sender<DockerMessage>,
    gui_state: &Arc<Mutex<GuiState>>,
    input_rx: Receiver<input_handler::InputMessages>,
    is_running: &Arc<AtomicBool>,
) {
    tokio::spawn(input_handler::InputHandler::start(
        Arc::clone(app_data),
        docker_sx.clone(),
        Arc::clone(gui_state),
        Arc::clone(is_running),
        input_rx,
    ));
}

/// Run the TUI with a pre-built [`Config`] — use this when embedding in another binary.
pub async fn run_with_config(config: Config) {
    let redraw = Arc::new(Rerender::new());
    let app_data = Arc::new(Mutex::new(AppData::new(config.clone(), &redraw)));
    let gui_state = Arc::new(Mutex::new(GuiState::new(&redraw, config.show_logs)));
    let is_running = Arc::new(AtomicBool::new(true));
    let (docker_tx, docker_rx) = tokio::sync::mpsc::channel(32);

    docker_init(&app_data, docker_rx, docker_tx.clone(), &gui_state).await;

    if config.gui {
        let (input_tx, input_rx) = tokio::sync::mpsc::channel(32);
        handler_init(&app_data, &docker_tx, &gui_state, input_rx, &is_running);
        Ui::start(app_data, gui_state, input_tx, is_running, redraw).await;
    } else {
        info!("in debug mode\n");
        let mut now = std::time::Instant::now();
        while is_running.load(Ordering::SeqCst) {
            let err = app_data.lock().get_error();
            if let Some(err) = err {
                error!("{}", err);
                std::process::exit(1);
            }
            if let Some(Ok(to_sleep)) = u128::from(config.docker_interval_ms)
                .checked_sub(now.elapsed().as_millis())
                .map(u64::try_from)
            {
                tokio::time::sleep(std::time::Duration::from_millis(to_sleep)).await;
            }
            let containers = app_data
                .lock()
                .get_container_items()
                .iter()
                .map(|i| format!("{i}"))
                .collect::<Vec<_>>();
            if !containers.is_empty() {
                for item in containers {
                    info!("{item}");
                }
                println!();
            }
            now = std::time::Instant::now();
        }
    }
}

/// Run the TUI standalone — parses its own CLI args via [`Config::new()`].
pub async fn run() {
    let config = Config::new();
    run_with_config(config).await;
}

/// Run the TUI embedded inside another binary — uses default config, does NOT parse process args.
pub async fn run_embedded() {
    let mut config = Config::from(&config::parse_args::Args::default());
    config.gui = true; // Args::default() sets gui=true meaning "disable gui"; flip it for embedded use
    run_with_config(config).await;
}

/// Run the TUI with args forwarded from a parent binary (e.g. `["--help"]`, `["-d", "500"]`).
/// Prepends a dummy program name so clap parses correctly.
/// Pass `filter` to pre-seed the container name filter (hides unrelated containers).
pub async fn run_with_args(args: &[String], filter: Option<String>) {
    use clap::Parser;
    use crossterm::event::KeyCode;
    let argv = std::iter::once("oxker").chain(args.iter().map(String::as_str));
    let parsed = crate::config::parse_args::Args::parse_from(argv);
    let mut config = Config::from(&parsed);
    if !parsed.gui {
        config.gui = true;
    }
    config.initial_filter = filter;
    // Bind 't' (primary) and 'T' (secondary) as exec-into-container keys
    config.keymap.exec = (KeyCode::Char('t'), Some(KeyCode::Char('T')));
    run_with_config(config).await;
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod tests {
    use std::{str::FromStr, sync::Arc};
    use bollard::service::{ContainerSummary, PortSummary};
    use crate::{
        app_data::{
            AppData, ContainerId, ContainerItem, ContainerPorts, ContainerStatus, Filter,
            RunningState, State, StatefulList,
        },
        config::{AppColors, Config, Keymap},
        ui::Rerender,
    };

    pub fn gen_config() -> Config {
        Config {
            app_colors: AppColors::new(),
            color_logs: false,
            exec_command: "sh".to_owned(),
            dir_save: None,
            dir_config: None,
            docker_interval_ms: 1000,
            gui: true,
            host: None,
            in_container: false,
            initial_filter: None,
            keymap: Keymap::new(),
            log_search_case_sensitive: true,
            raw_logs: false,
            show_logs: true,
            show_self: false,
            show_std_err: false,
            show_timestamp: false,
            timestamp_format: "HH:MM:SS.NNNNN dd-mm-yyyy".to_owned(),
            timezone: None,
            use_cli: false,
        }
    }

    pub fn gen_item(id: &ContainerId, index: usize) -> ContainerItem {
        ContainerItem::new(
            u64::try_from(index).unwrap(),
            id.clone(),
            format!("image_{index}"),
            false,
            format!("container_{index}"),
            vec![ContainerPorts {
                ip: None,
                private: u16::try_from(index).unwrap_or(1) + 8000,
                public: None,
            }],
            State::Running(RunningState::Healthy),
            ContainerStatus::from(format!("Up {index} hour")),
        )
    }

    pub fn gen_appdata(containers: &[ContainerItem]) -> AppData {
        AppData {
            containers: StatefulList::new(containers.to_vec()),
            hidden_containers: vec![],
            current_sorted_id: vec![],
            inspect_data: None,
            error: None,
            sorted_by: None,
            rerender: Arc::new(Rerender::new()),
            filter: Filter::new(),
            config: gen_config(),
        }
    }

    pub fn gen_containers() -> (Vec<ContainerId>, Vec<ContainerItem>) {
        let ids = (1..=3)
            .map(|i| ContainerId::from(format!("{i}").as_str()))
            .collect::<Vec<_>>();
        let containers = ids
            .iter()
            .enumerate()
            .map(|(index, id)| gen_item(id, index + 1))
            .collect::<Vec<_>>();
        (ids, containers)
    }

    pub fn gen_container_summary(index: usize, state: &str) -> ContainerSummary {
        ContainerSummary {
            image_manifest_descriptor: None,
            health: None,
            id: Some(format!("{index}")),
            names: Some(vec![format!("container_{}", index)]),
            image: Some(format!("image_{index}")),
            image_id: Some(format!("{index}")),
            command: None,
            created: Some(i64::try_from(index).unwrap()),
            ports: Some(vec![PortSummary {
                ip: None,
                private_port: u16::try_from(index).unwrap_or(1) + 8000,
                public_port: None,
                typ: None,
            }]),
            size_rw: None,
            size_root_fs: None,
            labels: None,
            state: Some(bollard::models::ContainerSummaryStateEnum::from_str(state).unwrap()),
            status: Some(format!("Up {index} hour")),
            host_config: None,
            network_settings: None,
            mounts: None,
        }
    }
}
