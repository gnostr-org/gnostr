use clap::Parser;
use env_logger::Env;
use anyhow::Result;
use std::process::Command;
use clap::ArgAction;
use which::which;
use std::io::Error;

#[cfg(not(test))]
use crate::ssh::start;

#[cfg(test)]
mod mock_ssh {
    pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
        // In test environment, always return an error for now
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Mock SSH Start Error",
        )))
    }
}

#[cfg(test)]
use mock_ssh::start;

#[derive(Parser, Debug, Clone)]
pub struct GitSubCommand {
    /// Starts the gnostr git SSH server (gitweb).
    #[arg(long)]
    pub gitweb: bool,
    /// Creates a git tag with an optional suffix.
    #[arg(long, action = ArgAction::Set, num_args = 0..=1, default_value_if("tag", clap::builder::ArgPredicate::IsPresent, Some("")))]
    pub tag: Option<String>,
    /// Displays local git information (version, path).
    #[arg(long)]
    pub info: bool,
}

pub async fn git(sub_command_args: &GitSubCommand) -> Result<(), Box<dyn std::error::Error>> {
    if sub_command_args.gitweb {
        env_logger::init_from_env(Env::default().default_filter_or("info"));
        let res = start().await;
        if let Err(e) = &res {
            println!("{}", e);
            println!("EXAMPLE:server.toml\n{}", SERVER_TOML);
            println!("check the port in your server.toml is available!\n");
            println!("EXAMPLE:repo.toml\n{}", REPO_TOML);
        }
        return res.map_err(|e| e.into());
    }

    if sub_command_args.tag.is_some() {
        let suffix = sub_command_args.tag.as_ref().map_or("".to_string(), |s| s.clone());
        run_git_tag(suffix).map_err(Into::<Box<dyn std::error::Error>>::into)?;
        return Ok(());
    }

    if sub_command_args.info {
        println!("{}", get_git_info());
        return Ok(());
    }

    let git_info = get_git_info();
    println!("The 'git' subcommand requires a flag to specify functionality.");
    println!("For example, use '--gitweb' to start the SSH server.");
    println!("Or, use '--tag [SUFFIX]' to create a git tag.");
    println!("Or, use '--info' to display local git information.");
    println!("{}", git_info);
    Ok(())
}

fn get_git_info() -> String {
    let git_path = match which("git") {
        Ok(path) => format!("Git path: {}", path.display()),
        Err(_) => "Git not found in PATH.".to_string(),
    };

    let git_version = match Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        },
        _ => "Git version: Not available.".to_string(),
    };

    format!("\nLocal Git Info:\n{}\n{}", git_path, git_version)
}

fn run_git_tag(suffix: String) -> Result<()> {
    let weeble_output = Command::new("gnostr-weeble").output()?.stdout;
    let weeble_cmd_output = String::from_utf8_lossy(&weeble_output).trim().to_string();

    let blockheight_output = Command::new("gnostr-blockheight").output()?.stdout;
    let blockheight_cmd_output = String::from_utf8_lossy(&blockheight_output).trim().to_string();

    let wobble_output = Command::new("gnostr-wobble").output()?.stdout;
    let wobble_cmd_output = String::from_utf8_lossy(&wobble_output).trim().to_string();

    let weeble = weeble_cmd_output.trim().to_string();
    let blockheight = blockheight_cmd_output.trim().to_string();
    let wobble = wobble_cmd_output.trim().to_string();

    let mut tag_name = format!("{}.{}.{}",
        if weeble.is_empty() { "0" } else { &weeble },
        if blockheight.is_empty() { "0" } else { &blockheight },
        if wobble.is_empty() { "0" } else { &wobble },
    );

    if !suffix.is_empty() {
        tag_name = format!("{}-{}", tag_name, suffix);
    }

    let output = Command::new("git").arg("tag").arg("-f").arg(&tag_name).output()?;

    if !output.status.success() {
        eprintln!("Error creating tag: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Failed to create tag");
    }
    print!("{}", tag_name);

    Ok(())
}

static REPO_TOML: &str = r###"#
name = "gnostr-gnit-server"
public = true
members = ["gnostr", "gnostr-user"]
failed_push_message = "Issues and patches can be emailed to admin@gnostr.org"
"###;

static SERVER_TOML: &str = r###"#
name = "gnostr.org"
port = 2222

hostname = "gnostr.org"

[users.gnostr]
is_admin = true
public_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDaBogLsfsOkKIpZEZYa3Ee+wFaaxeJuHps05sH2rZLf+KEE6pWX5MT2iWMgP7ihmm6OqbAPkWoBUGEO5m+m/K1S0MgQXUvaTsTI0II3MDqJT/RXA6Z9c+ZIDROEAkNIDrfeU2n8hQXfMHwG6aJjwv3Zky9jR/ey2rSgKLMcTOLrMeAyop6fYhjIHqp0dTagHo1j+XHAbVsrjw6oxC0ohTkp8rzH6cYJyjK4TOKApEgCALJUOA2rbHNxr68wAIe2RS36dRQobD3ops2+HoOGk7pkBQazBAlZp/H4monWRrq7tTEw8FkGMX5udZQX6BNEI0vJZqtdkSpG7jSS3aL7GXcuOYKpsTKxuGm5BWsrRPiphsc25U02oe/y3+qM0ceP/njJp3ZvXQ/a2QGPU4+P8WSD+J0oKS+TiRKrpiTR4ChJk8zWupg4PI5zflN3yyK7MrGXg1n0DsvHxPXcqpvVRz4i8ORt6IlKGkve1tC0Wd9pVy4044LDethMORRZFjWAdS/caN1EMgTrrGMxi0DLVw6ahedGUgZj2WYWfsrEg8Kzbfk3fn32sO/lMnNyz5hmavMBiNORGlIi2Qe2RjQEtcJHn89B7UtyEfnj87V+jZYcFf4nnNQigT2eQ3NlB1YzZS4Zk/OxQeYypclzYFaiYc7RZv2yxKVOy0KvEpldyUKeQ== randy.lee.mcmillan@gmail.com"

[users.gnostr-user]
can_create_repos = true
public_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDaBogLsfsOkKIpZEZYa3Ee+wFaaxeJuHps05sH2rZLf+KEE6pWX5MT2iWMgP7ihmm6OqbAPkWoBUGEO5m+m/K1S0MgQXUvaTsTI0II3MDqJT/RXA6Z9c+ZIDROEAkNIDrfeU2n8hQXfMHwG6aJjwv3Zky9jR/ey2rSgKLMcTOLrMeAyop6fYhjIHqp0dTagHo1j+XHAbVsrjw6oxC0ohTkp8rzH6cYJyjK4TOKApEgCALJUOA2rbHNxr68wAIe2RS36dRQobD3ops2+HoOGk7pkBQazBAlZp/H4monWRrq7tTEw8FkGMX5udZQX6BNEI0vJZqtdkSpG7jSS3aL7GXcuOYKpsTKxuGm5BWsrRPiphsc25U02oe/y3+qM0ceP/njJp3ZvXQ/a2QGPU4+P8WSD+J0oKS+TiRKrpiTR4ChJk8zWupg4PI5zflN3yyK7MrGXg1n0DsvHxPXcqpvVRz4i8ORt6IlKGkve1tC0Wd9pVy4044LDethMORRZFjWAdS/caN1EMgTrrGMxi0DLVw6ahedGUgZj2WYWfsrEg8Kzbfk3fn32sO/lMnNyz5hmavMBiNORGlIi2Qe2RjQEtcJHn89B7UtyEfnj87V+jZYcFf4nnNQigT2eQ3NlB1YzZS4Zk/OxQeYypclzYFaiYc7RZv2yxKVOy0KvEpldyUKeQ== randy.lee.mcmillan@gmail.com"

// Optional.
welcome_message = "welcome to gnostr.org!"
"###;