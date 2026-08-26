# Detachable services

`./p2p` and other service crates should stay **library-first**. The reusable detach logic belongs in `src/lib.rs`, and each binary should stay thin.

## Idiom

- Add a `--detach` flag to long-running service bins.
- When `--detach` is set, the bin relaunches its own executable with the same arguments minus `--detach`.
- The relaunch should send `stdin`, `stdout`, and `stderr` to `Stdio::null()`.
- The parent process exits after spawning the child and prints the child PID when useful.
- Foreground mode stays the default.

## Implementation shape

Use a shared helper in `src/lib.rs`:

```rust
pub fn spawn_detached_current_exe<I, S>(args: I) -> Result<u32, Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let current_exe = std::env::current_exe()?;
    let mut command = Command::new(current_exe);
    command.args(args);
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    let child = command.spawn()?;
    Ok(child.id())
}
```

Service binaries should:

1. Parse `--detach`.
2. If set, rebuild the current command line without `--detach`.
3. Call the shared helper.
4. Exit immediately in the parent process.

## Scope

Use this pattern for:

- `gnostr-p2p`
- `gnostr-p2p-relay-server`
- `gnostr-p2p-rendezvous-server`

Do not use it for one-shot client tools such as peer lookup.

## Reference

The crawler crate already follows this general pattern with `run_api_server_detached(...)`; `p2p` should mirror that style instead of adding ad hoc background logic in each binary.
