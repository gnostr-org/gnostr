use expectrl::session::Session;
use std::process::Command;
use std::io::Read;
use expectrl::process::Process;

/// The following code emits:
/// cat exited with code 0, all good!
/// cat exited with code 1
/// Output (stdout and stderr): cat: /this/does/not/exist: No such file or directory
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cat");
    cmd.arg("/etc/passwd");
    let mut p = Session::spawn(cmd)?;
    match p.wait() {
        Ok(Some(0)) => println!("cat exited with code 0, all good!"),
        _ => println!("cat exited with code >0, or it was killed"),
    }

    let mut cmd2 = Command::new("cat");
    cmd2.arg("/this/does/not/exist");
    let mut p = Session::spawn(cmd2)?;
    match p.wait() {
        Ok(Some(0)) => println!("cat succeeded"),
        Ok(Some(c)) => {
            println!("Cat failed with exit code {}", c);
            let mut output = Vec::new();
            p.read_to_end(&mut output)?;
            println!("Output (stdout and stderr): {}", String::from_utf8_lossy(&output));
        }
        // for other possible return types of wait()
        // see here: https://doc.rust-lang.org/std/process/struct.ExitStatus.html
        _ => println!("cat was probably killed"),
    }

    Ok(())
}
