use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    name: String,
    port: u16,
    hostname: String,
    users: HashMap<String, User>,
    welcome_message: WelcomeMessage,
    extra: Extra,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    #[serde(default)] // Use default value (false) if not specified in TOML
    is_admin: bool,
    can_create_repos: bool,
    public_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct WelcomeMessage {
    welcome_message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Extra {
    extra: String,
}

fn main() -> io::Result<()> {
    let mut users = HashMap::new();

    users.insert(
        "gnostr".to_string(),
        User {
            is_admin: true,
            can_create_repos: true,
            public_key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDWBy03xeN9LL4ZwqAOVcdrDBv26JQXPoIdQ+ZzCaf6LsW4DNNZlSn5GQwBZ340zC9os098ArH2dz5Hbih2x6tAAKdNRraG/CCc8JYe5ogitbPZlMaWcoeJkMLEiaZhJ8ZKBiTVw8tRHxIEGuuEEKXspsicE2WA7vf/Xv5jSKYEO5KUriz+JeOHTDD5C65AFh8odKI5Yb+sYRXT3tAdRTyOEJLfAdLQLRITyZ57eEBH3Ikkcpk3Ixoc/CBFGB45AQsi3X61djiRkAULilvAdTPfvgk2If0ldbEzHdLiHcbkanhW//xwrZ4GU6hjGjviSOq+n3Qki/InNxdJmh2jr7nJ4mdevctvtD3YLyVU+Ku99Y83lyMWWZ2LlRYK3OxK0fc9d7xJQVl9f4kPG3C6ZUcJ1BZbl/mCKOqegrTLnaTLj3wx3+NQSjzw9unhkVmcf7dofL+zYf2GLCiDKQrgVX9f4ZQr7mWi53QDwrZm0BMxDvERj7qJmwAmb1nUkBP6aJU= randymcmillan@DeepSpaceMBPro.local".to_string(),
        },
    );

    users.insert(
        "gnostr-user".to_string(),
        User {
            is_admin: false, // Explicitly set to false as it's not an admin
            can_create_repos: true,
            public_key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDaBogLsfsOkKIpZEZYa3Ee+wFaaxeJuHps05sH2rZLf+KEE6pWX5MT2iWMgP7ihmm6OqbAPkWoBUGEO5m+m/K1S0MgQXUvaTsTI0II3MDqJT/RXA6Z9c+ZIDROEAkNIDrfeU2n8hQXfMHbLw6ahedGUgZj2WYWfsrEg8Kzbfk3fn32sO/lMnNyz5hmavMBiNORGlIi2Qe2RjQEtcJHn89B7UtyEfnj87V+jZYcFf4nnNQigT2eQ3NlB1YzZS4Zk/OxQeYypclzYFaiYc7RZv2yxKVOy0KvEpldyUKeQ== randy.lee.mcmillan@gmail.com".to_string(),
        },
    );

    let config = Config {
        name: "gnostr.org".to_string(),
        port: 2222,
        hostname: "gnostr.org".to_string(),
        users,
        welcome_message: WelcomeMessage {
            welcome_message: "welcome to gnostr.org!".to_string(),
        },
        extra: Extra {
            extra: "extra toml content!".to_string(),
        },
    };

    let toml_string = toml::to_string_pretty(&config).expect("Failed to serialize config to TOML");

    println!("Generated server.toml content:\n{}", toml_string);

    fs::write("server.toml", toml_string)?;

    println!("server.toml generated successfully!");

    Ok(())
}
