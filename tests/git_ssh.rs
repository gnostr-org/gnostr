
#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::net::TcpListener;
    use std::path::Path;
    use gag::BufferRedirect;
    use std::io::Read;

    // This is the main function from src/bin/git-ssh.rs
    async fn run_git_ssh_main() {
        env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
        if let Err(e) = gnostr::ssh::start().await {
            println!("{}", e);
            println!("EXAMPLE:server.toml\n{}", SERVER_TOML_CONTENT);
            println!("check the port in your server.toml is available!");
            println!("check the port in your server.toml is available!");
            println!("check the port in your server.toml is available!\n");
            println!("EXAMPLE:repo.toml\n{}", REPO_TOML_CONTENT);
        }
    }

    const REPO_TOML_CONTENT: &str = r#" 
name = "gnostr-gnit-server"
public = true
members = ["gnostr", "gnostr-user"]
failed_push_message = "Issues and patches can be emailed to admin@gnostr.org"
"#;

    const SERVER_TOML_CONTENT: &str = r#" 
name = "gnostr.org"
port = 2222
hostname = "gnostr.org"
[users.gnostr]
is_admin = true
public_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDaBogLsfsOkKIpZEZYa3Ee+wFaaxeJuHps05sH2rZLf+KEE6pWX5MT2iWMgP7ihmm6OqbAPkWoBUGEO5m+m/K1S0MgQXUvaTsTI0II3MDqJT/RXA6Z9c+ZIDROEAkNIDrfeU2n8hQXfMHwG6aJjwv3Zky9jR/ey2rSgKLMcTOLrMeAyop6fYhjIHqp0dTagHo1j+XHAbVsrjw6oxC0ohTkp8rzH6cYJyjK4TOKApEgCALJUOA2rbHNxr68wAIe2RS36dRQobD3ops2+HoOGk7pkBQazBAlZp/H4monWRrq7tTEw8FkGMX5udZQX6BNEI0vJZqtdkSpG7jSS3aL7GXcuOYKpsTKxuGm5BWsrRPiphsc25U02oe/y3+qM0ceP/njJp3ZvXQ/a2QGPU4+P8WSD+J0oKS+TiRKrpiTR4ChJk8zWupg4PI5zflN3yyK7MrGXg1n0DsvHxPXcqpvVRz4i8ORt6IlKGkve1tC0Wd9pVy4044LDethMORRZFjWAdS/caN1EMgTrrGMxi0DLVw6ahedGUgZj2WYWfsrEg8Kzbfk3fn32sO/lMnNyz5hmavMBiNORGlIi2Qe2RjQEtcJHn89B7UtyEfnj87V+jZYcFf4nnNQigT2eQ3NlB1YzZS4Zk/OxQeYypclzYFaiYc7RZv2yxKVOy0KvEpldyUKeQ== randy.lee.mcmillan@gmail.com"
[users.gnostr-user]
can_create_repos = true
public_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDaBogLsfsOkKIpZEZYa3Ee+wFaaxeJuHps05sH2rZLf+KEE6pWX5MT2iWMgP7ihmm6OqbAPkWoBUGEO5m+m/K1S0MgQXUvaTsTI0II3MDqJT/RXA6Z9c+ZIDROEAkNIDrfeU2n8hQXfMHwG6aJjwv3Zky9jR/ey2rSgKLMcTOLrMeAyop6fYhjIHqp0dTagHo1j+XHAbVsrjw6oxC0ohTkp8rzH6cYJyjK4TOKApEgCALJUOA2rbHNxr68wAIe2RS36dRQobD3ops2+HoOGk7pkBQazBAlZp/H4monWRrq7tTEw8FkGMX5udZQX6BNEI0vJZqtdkSpG7jSS3aL7GXcuOYKpsTKxuGm5BWsrRPiphsc25U02oe/y3+qM0ceP/njJp3ZvXQ/a2QGPU4+P8WSD+J0oKS+TiRKrpiTR4ChJk8zWupg4PI5zflN3yyK7MrGXg1n0DsvHxPXcqpvVRz4i8ORt6IlKGkve1tC0Wd9pVy4044LDethMORRZFjWAdS/caN1EMgTrrGMxi0DLVw6ahedGUgZj2WYWfsrEg8Kzbfk3fn32sO/lMnNyz5hmavMBiNORGlIi2Qe2RjQEtcJHn89B7UtyEfnj87V+jZYcFf4nnNQigT2eQ3NlB1YzZS4Zk/OxQeYypclzYFaiYc7RZv2yxKVOy0KvEpldyUKeQ== randy.lee.mcmillan@gmail.com"
"#;

    #[tokio::test]
    async fn test_main_error_on_port_conflict() {
        let temp_dir = tempfile::tempdir().unwrap();
        env::set_current_dir(&temp_dir).unwrap();

        let port = 2222;
        // Occupy the port
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

        // Create a dummy server.toml
        let mut file = File::create("server.toml").unwrap();
        writeln!(file, "{}", SERVER_TOML_CONTENT).unwrap();
        drop(file);

        let mut buf = BufferRedirect::stdout().unwrap();

        tokio::time::timeout(std::time::Duration::from_secs(5), run_git_ssh_main())
            .await
            .expect("Test timed out");

        let mut output = String::new();
        buf.read_to_string(&mut output).unwrap();

        assert!(output.contains("check the port in your server.toml is available!"));
        assert!(output.contains(SERVER_TOML_CONTENT));
        assert!(output.contains(REPO_TOML_CONTENT));

        drop(listener);
    }
}
