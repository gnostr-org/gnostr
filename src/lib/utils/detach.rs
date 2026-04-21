use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub fn spawn_detached<P, I, S>(program: P, args: I) -> Result<u32, Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program.as_ref());
    command.args(args);
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

pub fn spawn_detached_current_exe<I, S>(args: I) -> Result<u32, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    spawn_detached(std::env::current_exe()?, args)
}
