use expectrl::{session::Session, Expect, Regex, process::UnixProcess, PtyStream};
use std::process::Command;

struct EdSession {
    session: Session<UnixProcess, PtyStream>,
    prompt: String,
    quit_command: Option<String>,
}

impl EdSession {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut session = Session::spawn(Command::new("/bin/ed").arg("-p").arg("> ").clone())?;
        session.expect("> ")?;
        Ok(EdSession {
            session,
            prompt: "> ".to_string(),
            quit_command: Some("Q".to_string()),
        })
    }

    fn wait_for_prompt(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.session.expect(self.prompt.as_str())?;
        Ok(())
    }

    fn send_line(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.session.send_line(line)?;
        Ok(())
    }

    fn exp_string(&mut self, s: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.session.expect(s)?;
        Ok(())
    }

    fn exp_eof(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.session.expect(expectrl::Eof)?;
        Ok(())
    }
}

impl Drop for EdSession {
    fn drop(&mut self) {
        if let Some(ref cmd) = self.quit_command {
            self.session
                .send_line(cmd)
                .expect(&format!("could not run `{}` on child process", cmd));
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ed = EdSession::new()?;
    ed.send_line("a")?;
    ed.send_line("ed is the best editor evar")?;
    ed.send_line(".")?;
    ed.wait_for_prompt()?;
    ed.send_line(",l")?;
    ed.exp_string("ed is the best editor evar$")?;
    ed.send_line("Q")?;
    ed.exp_eof()?;
    Ok(())
}
