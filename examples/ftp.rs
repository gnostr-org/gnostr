use expectrl::{session::Session, Expect, Regex};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut p = Session::spawn(Command::new("ftp").arg("speedtest.tele2.net"))?;
    p.expect(Regex("Name \\(.*\\):"))?;
    p.send_line("anonymous")?;
    p.expect("Password")?;
    p.send_line("test")?;
    p.expect("ftp>")?;
    p.send_line("cd upload")?;
    p.expect("successfully changed.\r\nftp>")?;
    p.send_line("pwd")?;
    p.expect(Regex("[0-9]+ \"/upload\""))?;
    p.send_line("exit")?;
    p.expect(expectrl::Eof)?;
    Ok(())
}
