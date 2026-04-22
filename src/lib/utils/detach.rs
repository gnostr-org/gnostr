use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub fn spawn_detached<P, I, S>(
    program: P,
    args: I,
) -> anyhow::Result<u32>
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

pub fn spawn_detached_current_exe<I, S>(
    args: I,
) -> anyhow::Result<u32>
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
