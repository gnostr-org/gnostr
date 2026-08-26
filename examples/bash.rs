use expectrl::{session::Session, Expect, Regex};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut p = Session::spawn(Command::new("bash"))?;
    p.expect(Regex(".*"))?;
    p.send_line("ping 8.8.8.8")?;
    p.expect("bytes")?;
    p.send_line("\x1A")?;
    p.expect(Regex(".*"))?;
    // bash writes 'ping 8.8.8.8' to stdout again to state which job was put into background
    p.send_line("bg")?;
    p.expect("ping 8.8.8.8")?;
    p.expect(Regex(".*"))?;
    p.send_line("sleep 0.5")?;
    p.expect(Regex(".*"))?;
    // bash writes 'ping 8.8.8.8' to stdout again to state which job was put into foreground
    p.send_line("fg")?;
    p.expect("ping 8.8.8.8")?;
    p.send_line("\x03")?;
    p.expect("packet loss")?;
    Ok(())
}
