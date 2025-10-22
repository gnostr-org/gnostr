# gnostr-legit

`gnostr-legit` is a command-line tool that allows you to "legitimize" your Git commits by publishing them to the Nostr protocol.

## Features

- **Git Commit Mining**: Generate Git commit SHAs with a custom prefix.
- **Nostr Integration**: Derive Nostr keys from commit IDs and publish serialized Git commits as Nostr events.
- **Relay Support**: Connects to specified Nostr relays (e.g., `wss://relay.damus.io`, `wss://e.nos.lol`).

## Usage

```bash
gnostr-legit [repository-path] -p <prefix> -m <message> [-t <threads>]
```

### Arguments

- `<repository-path>`: Path to your Git repository (defaults to current directory if not provided).

### Options

- `-p`, `--prefix <prefix>`: Desired commit prefix (e.g., `00000`). This is required.
- `-m`, `--message <message>`: Commit message to use. This is required.
- `-t`, `--threads <threads>`: Number of worker threads to use (default is the number of available parallelisms).

## Example

```bash
gnostr-legit . -p 00000 -m "My first gnostr-legit commit"
```

This command will:
1. Discover the Git repository in the current directory.
2. Mine for a commit SHA starting with `00000`.
3. Use "My first gnostr-legit commit" as the commit message.
4. Publish the serialized commit as a Nostr event to the configured relays.

## Output

The tool will output information about the generated commit, the serialized commit content, and the details of the Nostr event sent, including the Event ID and the relays it was sent to.

---

# generate-server-config

`generate-server-config` is a utility that sets up SSH keys and generates a `server.toml` configuration file for a Gnostr server.

## Features

- **SSH Key Management**: Generates an `ed25519` SSH key pair (`gnostr-gnit-key`), sets secure permissions for SSH directories and files (`~/.ssh`, `authorized_keys`, private and public keys), and adds the private key to the SSH agent.
- **TOML Configuration Generation**: Creates a `server.toml` file with a default server name, port, hostname, and defines two users (`gnostr` and `gnostr-user`) with their respective public keys and repository creation permissions.
- **Key Archiving**: If an existing `gnostr-gnit-key` is found, it is renamed with a timestamp derived from Bitcoin block height, weeble, and wobble values to prevent overwriting.

## Usage

```bash
generate-server-config [email]
```

### Arguments

- `[email]`: Optional. The email address to use for the SSH key comment (defaults to `gnostr@gnostr.org`).

## Example

```bash
generate-server-config myemail@example.com
```

This command will:
1. Ensure the `~/.ssh` directory exists and has correct permissions.
2. Generate a new `gnostr-gnit-key` SSH key pair using `myemail@example.com` as the comment.
3. Set appropriate permissions for the generated SSH keys and `authorized_keys`.
4. Add the `gnostr-gnit-key` to the SSH agent.
5. Generate a `server.toml` file with default server settings and user configurations, including the newly generated public key.

## Output

The tool provides verbose output on the SSH key generation process, permission settings, and the content of the generated `server.toml` file. It also provides instructions to verify SSH permissions.

---

# git-ssh

`git-ssh` is a binary that starts a Gnostr SSH server, enabling Git operations over SSH with Nostr integration.

## Features

- **SSH Server**: Initiates an SSH server to handle Git connections.
- **Error Reporting**: Provides informative error messages, including example `server.toml` and `repo.toml` configurations, if the server fails to start.

## Usage

```bash
git-ssh
```

This command does not take any arguments.

## Example

```bash
git-ssh
```

This command will attempt to start the Gnostr SSH server. If successful, it will listen for incoming Git SSH connections. If it fails, it will print an error message along with example configuration files to help diagnose the issue.

## Configuration

`git-ssh` relies on `server.toml` and `repo.toml` for its configuration. Ensure these files are correctly set up in your project. Example configurations are provided in the error output if the server fails to start.

---

# gnostr-blockhash

`gnostr-blockhash` is a command-line tool that fetches and prints the current Bitcoin block hash.

## Features

- **Bitcoin Block Hash Retrieval**: Connects to a Bitcoin block explorer API (e.g., `mempool.space`) to get the latest block hash.
- **Timestamping Component**: Forms part of the "WEEBLE WOBBLE" decentralized timestamping method.

## Usage

```bash
gnostr-blockhash
```

This command does not take any arguments.

## Example

```bash
gnostr-blockhash
```

This command will print the current Bitcoin block hash to standard output.

## Output

The tool outputs the current Bitcoin block hash. In debug mode, it also provides timing information for the retrieval process.

---

# gnostr-blockheight

`gnostr-blockheight` is a command-line tool that fetches and prints the current Bitcoin block height.

## Features

- **Bitcoin Block Height Retrieval**: Connects to a Bitcoin block explorer API (e.g., `mempool.space`) to get the latest block height.
- **Timestamping Component**: Forms part of the "WEEBLE WOBBLE" decentralized timestamping method.

## Usage

```bash
gnostr-blockheight
```

This command does not take any arguments.

## Example

```bash
gnostr-blockheight
```

This command will print the current Bitcoin block height to standard output.

## Output

The tool outputs the current Bitcoin block height. In debug mode, it also provides timing information for the retrieval process.

---

# gnostr-blockhash

`gnostr-blockhash` is a command-line tool that fetches and prints the current Bitcoin block hash.

## Features

- **Bitcoin Block Hash Retrieval**: Connects to a Bitcoin block explorer API (e.g., `mempool.space`) to get the latest block hash.
- **Timestamping Component**: Forms part of the "WEEBLE WOBBLE" decentralized timestamping method.

## Usage

```bash
gnostr-blockhash
```

This command does not take any arguments.

## Example

```bash
gnostr-blockhash
```

This command will print the current Bitcoin block hash to standard output.

## Output

The tool outputs the current Bitcoin block hash. In debug mode, it also provides timing information for the retrieval process.
