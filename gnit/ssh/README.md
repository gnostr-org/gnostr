# gnostr-ssh

A self-hosted Git server that's easy to set up, use, and maintain, accessed over SSH.

## Features

*   **Self-Hosted:** Your own private Git server.
*   **SSH-Based:** Secure access using SSH public key authentication.
*   **Easy Setup:** Get started with a simple configuration file.
*   **Git-Powered Configuration:** Server configuration is managed in a Git repository.
*   **Access Control:** Differentiate between admin and regular users.
*   **On-the-fly Repository Creation:** Create repositories by pushing to them.
*   **Per-Repository Permissions:** Control read/write access for each repository.
*   **Static Site Generator:** Automatically generates a simple webpage for public repositories from their `README.md`.

## Getting Started

1.  **Installation:** Install via `cargo install gnostr-ssh` or build from source.
2.  **Configuration:** Create a `gnostr-ssh.toml` file (see example below) in an empty directory.
3.  **Run:** Execute the `gnostr-ssh` binary in that directory.

That's it! Your Git server is running.

## Server Configuration

`gnostr-ssh` is configured via a `gnostr-ssh.toml` file. On the first run, `gnostr-ssh` will create a `.config.git` repository and commit your `gnostr-ssh.toml` file to it. This repository is only accessible to admin users.

Here's a minimal `gnostr-ssh.toml`:

```toml
name = "gnostr-ssh Server"
port = 2222
hostname = "git.example.com"

# Admin user
[users.claudia]
is_admin = true
public_key = "ssh-rsa AAAAj74s..."

# Regular user who can create repositories
[users.alex]
can_create_repos = true
public_key = "ssh-rsa AAAAm8fd..."

# Optional welcome message for users upon connection.
# '%' is replaced with the username.
welcome_message = "Welcome, %!"
```

## Repositories

### Creating Repositories

You can create a new repository on the server by simply pushing an existing local repository.

For example, the user `alex` can create a repository named `my-project` under their personal namespace:
```sh
git remote add origin ssh://git.example.com:2222/alex/my-project.git
git push -u origin main
```

Non-admin users can only create repositories under their personal subdirectory (e.g., `/username/repo.git`). Admin users can create repositories anywhere.

### Repository Configuration

When a new repository is created, `gnostr-ssh` will insert an `gnostr-repo.toml` config file into it. You can pull the changes, edit the file, and push it back to configure your repository.

Here's an example `gnostr-repo.toml`:

```toml
name = "Example Repo"

# Anyone can read this repository
public = true

# Only 'alex' can write to this repository
members = ["alex"]

# Optional message shown to users who fail to push
failed_push_message = "Patches can be emailed to alex@example.com"
```

## Static Site Generator

`gnostr-ssh` includes a simple static site generator. It generates a webpage for any public repository that contains a `README.md` file.

The generated pages are saved to the `static` directory in the server's root directory. The pages reflect the repository path and name.

You can provide a custom [Tera](https://tera.netlify.app/) template for your repository's page by setting the `web_template` option in the repository's `gnostr-repo.toml` config file. The path should be relative to the root of the repository.

The following variables are available in the template:

*   `repo_name`: The name of the repository.
*   `content`: The HTML content of the `README.md` file.
*   `clone_url`: The SSH URL to clone the repository.

## Git Remote Configuration

To configure a `git remote` for `gnostr-ssh`, follow these steps:

1.  **Ensure SSH Key Setup:**
    Make sure your SSH client is configured with the appropriate SSH key that `gnostr-ssh` recognizes. This typically involves adding your public key to the `gnostr-ssh` server's configuration.

2.  **Add the Remote:**
    Navigate to your local Git repository and add `gnostr-ssh` as a remote using the `git remote add` command. The format is generally:

    ```bash
    git remote add <remote_name> ssh://<username>@<gnostr_ssh_hostname>:<port>/<repository_name>.git
    ```

    *   `<remote_name>`: A name for your remote (e.g., `gnostr`, `origin`).
    *   `<username>`: The username configured in `gnostr-ssh` for your public key.
    *   `<gnostr_ssh_hostname>`: The hostname or IP address where `gnostr-ssh` is running.
    *   `<port>`: The port `gnostr-ssh` is listening on (default is 2222, but can be configured).
    *   `<repository_name>.git`: The name of the repository on the `gnostr-ssh` server (must end with `.git`).

    **Example:**
    If `gnostr-ssh` is running on `localhost` at port `2222`, your username is `alice`, and your repository is `my_project.git`, you would run:

    ```bash
    git remote add gnostr ssh://alice@localhost:2222/my_project.git
    ```

3.  **Verify the Remote:**
    You can verify that the remote has been added correctly by listing your remotes:

    ```bash
    git remote -v
    ```

    This should show the `gnostr` remote with its fetch and push URLs.

4.  **Push to the Remote:**
    Once the remote is configured, you can push your local branches to `gnostr-ssh`:

    ```bash
    git push -u gnostr main
    ```

    This command pushes the `main` branch to the `gnostr` remote and sets it as the upstream tracking branch.
