use std::io::{Error, ErrorKind};
use std::process::{Command, Output};

struct SystemCommand {
    command: String,
    args: Vec<String>,
}

impl SystemCommand {
    fn new(command: &str) -> Self {
        SystemCommand {
            command: command.to_string(),
            args: Vec::new(),
        }
    }

    fn arg(&mut self, arg: &str) -> &mut Self {
        self.args.push(arg.to_string());
        self
    }

    fn args(&mut self, args: &[&str]) -> &mut Self {
        for arg in args {
            self.args.push(arg.to_string());
        }
        self
    }

    fn execute(&self) -> Result<Output, Error> {
        let output = Command::new(&self.command).args(&self.args).output();

        match output {
            Ok(output) => Ok(output),
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("Failed to execute command: {}", e),
            )),
        }
    }

    fn execute_and_get_stdout(&self) -> Result<String, Error> {
        let output = self.execute()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Command failed with status {}: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            ))
        }
    }

    fn execute_and_get_stderr(&self) -> Result<String, Error> {
        let output = self.execute()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stderr).trim().to_string())
        } else {
            Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Command failed with status {}: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            ))
        }
    }

    fn execute_and_check_status(&self) -> Result<bool, Error> {
        let output = self.execute()?;
        Ok(output.status.success())
    }
}

pub fn system_command_test() {

    let mut git_status_s = SystemCommand::new("git");
    git_status_s.arg("status").arg("-s");

    match git_status_s.execute_and_get_stdout() {
        Ok(output) => println!("GitStatusS:\n{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
    let mut git_diff = SystemCommand::new("git");
    git_diff.arg("diff");
    git_diff.arg("HEAD");

    match git_diff.execute_and_get_stdout() {
        Ok(output) => println!("GitDiff:\n{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }

    //let mut grep_command = SystemCommand::new("cat");
    //grep_command.args(&["README.md"]);
    //match grep_command.execute_and_get_stdout() {
    //    Ok(output) => println!("CatREADME_md:\n{}", output),
    //    Err(e) => eprintln!("Error: {}", e),
    //}




    let test_string = "This is a Test string\nAnother line with test";

    // You'd typically pipe the output of another command into grep.
    // However, for this example, we'll simulate the input by writing to stdin.
    use std::io::Write;
    use std::process::Stdio;

    let mut grep_process = Command::new("grep")
        .args(&["-i", "test"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(mut stdin) = grep_process.stdin.take() {
        stdin.write_all(test_string.as_bytes()).unwrap();
    }

    let grep_output = grep_process.wait_with_output().unwrap();

    if grep_output.status.success() {
        println!(
            "Grep output: {}",
            String::from_utf8_lossy(&grep_output.stdout)
        );
    } else {
        println!("Grep failed");
    }

    //let mut false_command = SystemCommand::new("false");
    //match false_command.execute_and_check_status() {
    //    Ok(status) => println!("False command status: {}", status),
    //    Err(e) => eprintln!("Error: {}", e),
    //}

    //let mut error_command = SystemCommand::new("nonexistent_command");
    //match error_command.execute_and_get_stdout() {
    //    Ok(output) => println!("Output: {}", output),
    //    Err(e) => eprintln!("Error: {}", e),
    //}
}
