use anyhow::{bail, Context, Result};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::Command;

pub const IMAGE: &str = "ghcr.io/github/github-mcp-server";

/// Resolve the MCP server image to use, allowing override via `GITHUB_MCP_IMAGE`.
pub fn image() -> String {
    std::env::var("GITHUB_MCP_IMAGE").unwrap_or_else(|_| IMAGE.to_string())
}
pub const CONTAINER_NAME: &str = "github-mcp-server";

/// Locate the `docker` binary cross-platform (PATH + known fallback locations).
pub fn find_docker() -> Option<PathBuf> {
    let exe = if cfg!(windows) {
        "docker.exe"
    } else {
        "docker"
    };

    // Search PATH first
    if let Ok(path_var) = std::env::var("PATH") {
        let sep = if cfg!(windows) { ';' } else { ':' };
        for dir in path_var.split(sep) {
            let p = Path::new(dir).join(exe);
            if p.is_file() {
                return Some(p);
            }
        }
    }

    // Platform-specific fallbacks
    #[cfg(target_os = "macos")]
    let extras: &[&str] = &[
        "/usr/local/bin/docker",
        "/opt/homebrew/bin/docker",
        "/Applications/Docker.app/Contents/Resources/bin/docker",
    ];
    #[cfg(target_os = "linux")]
    let extras: &[&str] = &[
        "/usr/bin/docker",
        "/usr/local/bin/docker",
        "/snap/bin/docker",
    ];
    #[cfg(windows)]
    let extras: &[&str] = &[
        r"C:\Program Files\Docker\Docker\resources\bin\docker.exe",
        r"C:\ProgramData\DockerDesktop\version-bin\docker.exe",
    ];

    extras
        .iter()
        .map(Path::new)
        .find(|p| p.is_file())
        .map(PathBuf::from)
}

/// Find docker, verify the daemon is up, pull the MCP image if absent.
/// Returns the resolved path to the docker binary.
pub async fn ensure_ready() -> Result<PathBuf> {
    let docker = find_docker().with_context(|| {
        format!(
            "Docker binary not found.\n  \
             Install Docker Desktop: https://www.docker.com/get-started\n  \
             Then make sure `docker` is on your PATH."
        )
    })?;

    println!("🐳 Docker: {}", docker.display());
    check_daemon(&docker).await?;
    ensure_image(&docker).await?;
    Ok(docker)
}

async fn check_daemon(docker: &Path) -> Result<()> {
    let ok = Command::new(docker)
        .args(["info", "--format", "{{.ServerVersion}}"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false);

    if ok {
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    bail!(
        "Docker daemon is not running.\n  \
         Start it with: open -a Docker\n  \
         Or launch Docker Desktop from your Applications folder."
    );
    #[cfg(target_os = "windows")]
    bail!(
        "Docker daemon is not running.\n  \
         Start Docker Desktop from the Start menu or system tray."
    );
    #[cfg(not(any(target_os = "macos", windows)))]
    bail!(
        "Docker daemon is not running.\n  \
         Try:  sudo systemctl start docker\n  \
         Or:   sudo service docker start"
    );
}

async fn ensure_image(docker: &Path) -> Result<()> {
    let img = image();
    let present = Command::new(docker)
        .args(["image", "inspect", &img, "--format", "{{.Id}}"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false);

    if present {
        println!("✅ Image {img} already present");
        return Ok(());
    }

    // Default image: pull from registry.
    if img == IMAGE {
        println!("📦 Pulling {img} …");
        let status = Command::new(docker)
            .args(["pull", &img])
            .status()
            .await
            .context("Failed to run docker pull")?;

        if !status.success() {
            bail!(
                "Failed to pull {img}.\n  \
                 Check your internet connection, then try:\n  \
                 docker pull {img}"
            );
        }
        println!("✅ Image ready");
        return Ok(());
    }

    // Custom image: build it from the embedded Dockerfile if missing.
    println!("🔨 Building {img} …");
    let tmp = std::env::temp_dir();
    let dockerfile_path = tmp.join("Dockerfile.mcp");
    std::fs::write(&dockerfile_path, include_str!("../Dockerfile.mcp"))
        .context("Failed to write Dockerfile")?;
    let context_dir = tmp.join("mcp_build_context");
    let _ = std::fs::create_dir_all(&context_dir);

    let status = Command::new(docker)
        .args([
            "build",
            "-t",
            &img,
            "-f",
            dockerfile_path.to_str().unwrap_or("Dockerfile.mcp"),
            context_dir.to_str().unwrap_or("."),
        ])
        .status()
        .await
        .context("Failed to run docker build")?;

    if !status.success() {
        bail!(
            "Failed to build {img}.\n  \
             Try manually:\n  \
             docker build -f Dockerfile.mcp -t {img} ."
        );
    }
    println!("✅ Image ready");
    Ok(())
}

/// Check whether a container with `CONTAINER_NAME` is already running.
async fn container_running(docker: &Path) -> bool {
    Command::new(docker)
        .args(["inspect", "--format", "{{.State.Running}}", CONTAINER_NAME])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .await
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true")
        .unwrap_or(false)
}

/// Start the GitHub MCP server container in the background so it is visible in the docker TUI.
/// Uses `tail -f /dev/null` as entrypoint to keep it alive.
/// No-ops if a container named `CONTAINER_NAME` already exists and is running.
pub async fn start_mcp_server(docker: &Path, github_token: &str) -> Result<()> {
    if container_running(docker).await {
        println!("✅ Container '{CONTAINER_NAME}' already running");
        return Ok(());
    }

    // Remove any stopped container with the same name so we can start fresh.
    let _ = Command::new(docker)
        .args(["rm", "-f", CONTAINER_NAME])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    println!("🚀 Starting container '{CONTAINER_NAME}' …");
    let status = Command::new(docker)
        .args([
            "run",
            "-d",
            "-i",
            "--name",
            CONTAINER_NAME,
            "-e",
            &format!("GITHUB_PERSONAL_ACCESS_TOKEN={github_token}"),
            &image(),
        ])
        .status()
        .await
        .context("Failed to start MCP server container")?;

    if !status.success() {
        bail!("Could not start container '{CONTAINER_NAME}'");
    }
    println!("✅ Container '{CONTAINER_NAME}' started");
    Ok(())
}
