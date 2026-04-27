use std::{
    ffi::OsStr,
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    path::Path,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub fn spawn_detached<P, I, S>(program: P, args: I) -> anyhow::Result<u32>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    spawn_detached_named(program, None::<&OsStr>, args)
}

pub fn spawn_detached_named<P, N, I, S>(
    program: P,
    process_name: Option<N>,
    args: I,
) -> anyhow::Result<u32>
where
    P: AsRef<Path>,
    N: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program.as_ref());
    command.args(args);

    #[cfg(unix)]
    if let Some(process_name) = process_name.as_ref() {
        command.arg0(process_name.as_ref());
    }

    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());

    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            let pid = libc::setsid();
            if pid == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }

    let child = command.spawn()?;
    Ok(child.id())
}

pub fn spawn_detached_named_with_env<P, N, I, S, K, V, E>(
    program: P,
    process_name: Option<N>,
    args: I,
    envs: E,
) -> anyhow::Result<u32>
where
    P: AsRef<Path>,
    N: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let mut command = Command::new(program.as_ref());
    command.args(args).envs(envs);

    #[cfg(unix)]
    if let Some(process_name) = process_name.as_ref() {
        command.arg0(process_name.as_ref());
    }

    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());

    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            let pid = libc::setsid();
            if pid == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }

    let child = command.spawn()?;
    Ok(child.id())
}

pub fn spawn_detached_current_exe<I, S>(args: I) -> anyhow::Result<u32>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    spawn_detached(std::env::current_exe()?, args)
}

pub fn spawn_detached_current_exe_named<N, I, S>(
    process_name: Option<N>,
    args: I,
) -> anyhow::Result<u32>
where
    N: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    spawn_detached_named(std::env::current_exe()?, process_name, args)
}

pub fn spawn_detached_current_exe_named_with_env<N, I, S, K, V, E>(
    process_name: Option<N>,
    args: I,
    envs: E,
) -> anyhow::Result<u32>
where
    N: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    spawn_detached_named_with_env(std::env::current_exe()?, process_name, args, envs)
}

pub fn capture_detached_pid(name: &str, pid: u32) -> anyhow::Result<PathBuf> {
    let path = detached_pid_path(name);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(&path, format!("{pid}\n"))?;
    Ok(path)
}

pub fn detached_pid_path(name: &str) -> PathBuf {
    PathBuf::from(".gnostr").join(format!("{name}.pid"))
}

pub fn read_detached_pid(name: &str) -> anyhow::Result<Option<u32>> {
    let path = detached_pid_path(name);
    let Ok(raw) = fs::read_to_string(&path) else {
        return Ok(None);
    };

    let pid = raw.trim().parse::<u32>()?;
    Ok(Some(pid))
}

#[cfg(unix)]
pub fn pid_is_running(pid: u32) -> bool {
    let result = unsafe { libc::kill(pid as i32, 0) };
    if result == 0 {
        true
    } else {
        matches!(std::io::Error::last_os_error().raw_os_error(), Some(code) if code == libc::EPERM)
    }
}

#[cfg(not(unix))]
pub fn pid_is_running(_pid: u32) -> bool {
    false
}

pub fn existing_detached_pid(name: &str) -> anyhow::Result<Option<u32>> {
    let path = detached_pid_path(name);
    let Some(pid) = read_detached_pid(name)? else {
        return Ok(None);
    };

    if pid_is_running(pid) {
        return Ok(Some(pid));
    }

    let _ = fs::remove_file(path);
    Ok(None)
}

pub fn relay_port_is_listening(port: u16) -> bool {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok()
}

pub fn listener_pid_on_port(port: u16) -> anyhow::Result<Option<u32>> {
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-tiTCP:", &port.to_string(), "-sTCP:LISTEN"])
            .output();

        let Ok(output) = output else {
            return Ok(None);
        };

        let pid = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .and_then(|line| line.trim().parse::<u32>().ok());
        return Ok(pid);
    }

    #[cfg(not(unix))]
    {
        let _ = port;
        Ok(None)
    }
}

pub fn kill_process_by_pid(pid: u32) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        let result = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
        if result == 0 {
            return Ok(());
        }

        let err = std::io::Error::last_os_error();
        if matches!(err.raw_os_error(), Some(code) if code == libc::ESRCH) {
            return Ok(());
        }

        return Err(err.into());
    }

    #[cfg(windows)]
    {
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "taskkill failed").into())
        }
    }
}
