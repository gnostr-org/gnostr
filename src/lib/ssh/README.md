# Eejit

Eejit is self-hosted Git server that's easy to set up, use, and maintain.

## Getting Started

- Install via cargo || source.
- Create an empty directory, and create a starting `server.toml` file (minimal example below).
- Run Eejit.
- Done!

## Server Config

Eejit is configured via the `server.toml` file inside the `/config.git` repo, which is only accessible to admin users.
When starting Eejit for the first time, it will copy an adjacent config file into the newly created config repo.
Here's a minimal example:

```toml
name = "Eejit Server"
port = 2222

hostname = "example.com"

[users.claudia]
is_admin = true
public_key = "ssh-rsa AAAAj74s..."

[users.alex]
can_create_repos = true
public_key = "ssh-rsa AAAAm8fd..."

# Optional.
welcome_message = "Welcome, %!"
```

## Repositories

You can create a new repository on an Eejit server by simply pushing an existing one. Non-admin users can only create
repos under their personal subdirectory (so for example, the user Alex above could push to `ssh://127.0.0.1:2222/alex/repo.git`
to create it).

When a new repository is created, Eejit will insert an `eejit.toml` config file into it. There, the user can specify if the repo
is public, and which other members can write to it. Here's a minimal example:

```toml
name = "Example Repo"

# Anyone can read...
public = true

# But only Alex can write...
members = ["alex"]

# Anyone else will see this message (OPTIONAL)
failed_push_message = "Patches can be emailed to alex@alex.alex"
```

## Static Site Generator

Eejit comes with a simple static site generator, which generates a webpage out of any public repository with a `README.md` file.
The generated pages are saved to the `static` directory, and reflect the repo path/name. There's a default Tera template, or
you can define your own with the `web_template` option in the repo config.