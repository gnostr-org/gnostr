use expectrl::{session::Session, Expect, Regex};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut command_builder = Command::new("ftp");
    command_builder.arg("speedtest.tele2.net");

    let cmd = command_builder; // (Or maybe Command::new(...).arg(...).into())

    let mut p = Session::spawn(cmd)?;

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
