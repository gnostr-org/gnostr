use expectrl::session::Session;
use std::process::Command;
use std::io::Read;

/// The following code emits:
/// cat exited with code 0, all good!
/// cat exited with code 1
/// Output (stdout and stderr): cat: /this/does/not/exist: No such file or directory
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut p = Session::spawn(Command::new("cat").arg("/etc/passwd"))?;
    match p.child.wait() {
        _ => println!("cat exited with code >0, or it was killed"),
    }

    let mut p = Session::spawn(Command::new("cat").arg("/this/does/not/exist"))?;
    match p.child.wait() {
        Ok(Some(0)) => println!("cat succeeded"),
        Ok(Some(c)) => {
            println!("Cat failed with exit code {}", c);
            let mut output = String::new();
            p.child.read_to_end(&mut output)?;
            println!("Output (stdout and stderr): {}", output);
        }
        // for other possible return types of wait()
        // see here: https://doc.rust-lang.org/std/process/struct.ExitStatus.html
        _ => println!("cat was probably killed"),
    }

    Ok(())
}
