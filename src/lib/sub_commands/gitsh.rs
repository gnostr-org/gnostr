use clap::Parser;
use env_logger::Env;

#[cfg(not(test))]
use crate::ssh::start;

#[cfg(test)]
mod mock_ssh {
    // Modified to accept 0 arguments to match the actual start() function signature.
    // The remote_url is captured by GitshSubCommand but not directly passed to start().
    // Further investigation might be needed on how start() uses configuration.
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
pub struct GitshSubCommand {
    // This field captures the remote_url, but the start() function does not directly accept it.
    #[arg(index = 1)]
    remote_url: Option<String>,
}

pub async fn gitsh(_sub_command_args: &GitshSubCommand) -> Result<(), Box<dyn std::error::Error>> {
    //#[cfg(not(test))]
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    // The start() function takes no arguments. The remote_url is available in _sub_command_args.remote_url
    // but is not directly passed to start().
    let res = start().await;
    if let Err(e) = &res {
        // Use reference to res to avoid moving it
        println!("{}", e);
        println!("EXAMPLE:server.toml\n{}", SERVER_TOML);
        println!("check the port in your server.toml is available!\n");
        println!("EXAMPLE:repo.toml\n{}", REPO_TOML);
    }
    res.map_err(|e| e.into())
}

static REPO_TOML: &str = r###"#
name = "gnostr-gnit-server"
public = true
members = ["gnostr", "gnostr-user"]
failed_push_message = "Issues and patches can be emailed to admin@gnostr.org"
"###;

static SERVER_TOML: &str = r#"
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
"#;
