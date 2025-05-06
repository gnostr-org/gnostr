use rexpect::error::Error;
use rexpect::spawn_bash;

fn main() -> Result<(), Error> {
    let mut p = spawn_bash(Some(1000))?;
    p.execute("ping gitworkshop.dev", "bytes")?;
    p.send_control('z')?;
    p.wait_for_prompt()?;
    p.execute("bg", "ping gitworkshop.dev")?;
    p.wait_for_prompt()?;
    p.send_line("sleep 0.5")?;
    p.wait_for_prompt()?;
    p.execute("fg", "ping gitworkshop.dev")?;
    p.send_control('c')?;
    p.exp_string("packet loss")?;
    Ok(())
}
