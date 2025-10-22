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

# gnostr-cube

`gnostr-cube` is a TUI (Text User Interface) application that provides various functionalities, including chat and system command execution, leveraging a global Tokio runtime for asynchronous operations.

## Features

- **Text User Interface (TUI)**: Built with `ratatui` for an interactive command-line experience.
- **Asynchronous Task Management**: Utilizes a global Tokio runtime to manage concurrent tasks, such as chat and system command tests.
- **Argument Parsing**: Uses `clap` for flexible command-line argument handling.

## Usage

```bash
gnostr-cube [OPTIONS]
```

### Options

- `-n`, `--name <NAME>`: Name of the person to greet (default: `user`).
- `-c`, `--count <COUNT>`: Number of times to greet (default: `1`).
- `-t`, `--tui`: Enable the Text User Interface. (default: `false`).
- `--chat`: Enable chat functionality. (default: `false`).
- `--cfg <CONFIG>`: Configuration string (default: `""`).

## Example

```bash
gnostr-cube --tui --chat -n Alice -c 5
```

This command will launch the `gnostr-cube` TUI with chat functionality enabled, greet "Alice" 5 times, and execute asynchronous tasks for chat and system command tests.

## Output

The tool outputs messages related to asynchronous task execution and, if the TUI is enabled, provides an interactive interface. Error messages are printed to `stderr` if any issues occur during execution.

---

# gnostr-genssh

`gnostr-genssh` is a utility for generating SSH keys and setting appropriate file permissions, with specific handling for different operating systems.

## Features

- **SSH Key Generation**: Creates an `ed25519` SSH key pair named `gnostr-gnit-key`.
- **Permission Management**: Sets secure permissions for the `~/.ssh` directory, `authorized_keys` file, and both private and public SSH keys. It includes OS-specific logic for macOS, Linux, and Windows.

## Usage

```bash
gnostr-genssh [email]
```

### Arguments

- `[email]`: Optional. The email address to use for the SSH key comment (defaults to `gnostr@gnostr.org`).

## Example

```bash
gnostr-genssh myuser@example.com
```

This command will generate an SSH key pair, set the necessary permissions, and print messages indicating the progress and success of each step. If any command fails, it will print an error message and exit.

## Output

The tool provides detailed output on the creation of the `~/.ssh` directory, SSH key generation, and the application of permissions to various SSH-related files. It also offers verification commands for checking permissions.

---

# gnostr-kvs

`gnostr-kvs` is a distributed key-value store application built on `libp2p`, leveraging Kademlia DHT and Gossipsub for peer-to-peer communication. It can publish Git repository data (commit messages and diffs) to the network.

## Features

- **Distributed Key-Value Store**: Implements a distributed KVS using `libp2p`'s Kademlia DHT for content routing and storage.
- **Gossipsub Messaging**: Utilizes Gossipsub for broadcasting messages (e.g., Bitcoin alerts) to subscribed peers.
- **Git Integration**: Scans a local Git repository to publish commit messages and diffs as records in the DHT.
- **Multi-network Support**: Can connect to various `libp2p` networks, including IPFS, Kusama, Polkadot, and Ursa, using their respective bootnodes.
- **Peer-to-Peer Communication**: Supports mDNS for local peer discovery, identify protocol for peer information exchange, and rendezvous for peer discovery.

## Usage

```bash
gnostr-kvs [OPTIONS] [arg_commit...]
```

### Options

- `--secret <SECRET>`: Optional. A `u8` seed for generating the ED25519 keypair.
- `--peer <PEER_ID>`: Optional. Peer ID to lookup (implies DHT lookup, default network is IPFS).
- `--multiaddr <MULTIADDR>`: Optional. Multiaddress to directly connect to a peer.
- `--network <NETWORK>`: Optional. Specifies the `libp2p` network to connect to (choices: `kusama`, `polkadot`, `ipfs`, `ursa`; default: `ipfs`).
- `--flag-topo-order`: Sort commits in topological order.
- `--flag-date-order`: Sort commits by date.
- `--flag-reverse`: Reverse the order of commits.
- `--flag-author <AUTHOR>`: Filter commits by author.
- `--flag-committer <COMMITTER>`: Filter commits by committer.
- `--grep <GREP_PATTERN>`: Filter commit messages by a regex pattern.
- `--git-dir <GIT_DIR>`: Path to the Git repository (defaults to current directory).
- `--skip <SKIP_COUNT>`: Number of commits to skip.
- `-n`, `--max-count <MAX_COUNT>`: Maximum number of commits to display.
- `--flag-merges`: Include merge commits.
- `--flag-no-merges`: Exclude merge commits.
- `--flag-no-min-parents`: Ignore minimum parent count filter.
- `--flag-no-max-parents`: Ignore maximum parent count filter.
- `--flag-max-parents <MAX_PARENTS>`: Maximum number of parents for a commit.
- `--flag-min-parents <MIN_PARENTS>`: Minimum number of parents for a commit.
- `-p`, `--flag-patch`: Show patch output for each commit.

### Arguments

- `[arg_commit...]`: Optional. Specific commit references (e.g., SHA, branch name) to start the revision walk from.
- `[arg_spec...]`: Optional. Additional arguments for revision walking (last argument).

## Example

```bash
gnostr-kvs --network ipfs --git-dir /path/to/my/repo -n 10 --flag-patch HEAD
```

This command will start the `gnostr-kvs` application, connect to the IPFS network, scan the Git repository at `/path/to/my/repo`, publish the latest 10 commit messages and their patches to the DHT, and subscribe to the "bitcoin_alert_system" topic. It will then enter an interactive loop to handle user commands like `GET`, `PUT`, `TOPIC`, and `QUIT`.

## Interactive Commands

Once `gnostr-kvs` is running, you can interact with it via standard input:

- `TOPIC <topic_string>`: Subscribe to a Gossipsub topic.
- `GET <key>`: Retrieve a record from the Kademlia DHT.
- `GET_PROVIDERS <key>`: Find providers for a given key in the Kademlia DHT.
- `PUT <key> <value>`: Store a key-value pair in the Kademlia DHT and start providing it. Also subscribes to a Gossipsub topic named after the key.
- `PUT_PROVIDER <key>`: Start providing a key in the Kademlia DHT.
- `QUIT` or `Q` or `EXIT`: Terminate the application.

## Output

The tool provides extensive logging (info, debug, warn) about network events, peer discovery, record storage/retrieval, and Gossipsub messages. It prints received Kademlia records in JSON format and Gossipsub messages to standard output.

---

# gnostr-query

`gnostr-query` is a command-line tool for querying Nostr relays. It allows users to construct complex filters to retrieve events based on various criteria.

## Features

- **Nostr Relay Querying**: Connects to Nostr relays (specified or bootstrap relays) to fetch events.
- **Flexible Filtering**: Supports filtering by:
    - **Authors**: `authors <public_key1,public_key2,...>`
    - **Event IDs**: `ids <event_id1,event_id2,...>`
    - **Kinds**: `kinds <kind1,kind2,...>` (e.g., NIP-0034 kinds like `1630,1632,...`)
    - **Hashtags**: `hashtag <tag1,tag2,...>`
    - **Mentions**: `mentions <public_key1,public_key2,...>`
    - **References**: `references <event_id1,event_id2,...>`
    - **Generic Tags**: `generic <tag_name> <value1,value2,...>`
    - **Search**: `search <search_term>`
- **Limit Control**: Specify the maximum number of events to retrieve using `limit <count>`.

## Usage

```bash
gnostr-query [OPTIONS]
```

### Options

- `--authors <AUTHORS>`: Comma-separated list of author public keys.
- `--ids <IDS>`: Comma-separated list of event IDs.
- `--limit <LIMIT>`: Maximum number of events to fetch.
- `--generic <TAG_NAME> <VALUES>`: Generic tag filter (e.g., `--generic d <some_value>`).
- `--hashtag <HASHTAGS>`: Comma-separated list of hashtags.
- `--mentions <MENTIONS>`: Comma-separated list of mentioned public keys.
- `--references <REFERENCES>`: Comma-separated list of referenced event IDs.
- `--kinds <KINDS>`: Comma-separated list of event kinds (integers).
- `--search <SEARCH_TERM>`: Search term for event content.
- `--relay <RELAY_URL>`: Specify a single relay URL to connect to (defaults to bootstrap relays).

## Example

```bash
gnostr-query --kinds 1,7 --limit 10 --hashtag nostr --relay wss://relay.damus.io
```

This command will query `wss://relay.damus.io` for the latest 10 events of kind 1 (text notes) and 7 (reactions) that contain the hashtag "nostr".

## Output

The tool outputs the retrieved Nostr events in JSON format to standard output. It also provides debug logging to `stderr` about the query construction and relay communication.

---

# gnostr-genssh

`gnostr-genssh` is a utility for generating SSH keys and setting appropriate file permissions, with specific handling for different operating systems.

## Features

- **SSH Key Generation**: Creates an `ed25519` SSH key pair named `gnostr-gnit-key`.
- **Permission Management**: Sets secure permissions for the `~/.ssh` directory, `authorized_keys` file, and both private and public SSH keys. It includes OS-specific logic for macOS, Linux, and Windows.

## Usage

```bash
gnostr-genssh [email]
```

### Arguments

- `[email]`: Optional. The email address to use for the SSH key comment (defaults to `gnostr@gnostr.org`).

## Example

```bash
gnostr-genssh myuser@example.com
```

This command will generate an SSH key pair, set the necessary permissions, and print messages indicating the progress and success of each step. If any command fails, it will print an error message and exit.

## Output

The tool provides detailed output on the creation of the `~/.ssh` directory, SSH key generation, and the application of permissions to various SSH-related files. It also offers verification commands for checking permissions.

---

# gnostr-cube

`gnostr-cube` is a TUI (Text User Interface) application that provides various functionalities, including chat and system command execution, leveraging a global Tokio runtime for asynchronous operations.

## Features

- **Text User Interface (TUI)**: Built with `ratatui` for an interactive command-line experience.
- **Asynchronous Task Management**: Utilizes a global Tokio runtime to manage concurrent tasks, such as chat and system command tests.
- **Argument Parsing**: Uses `clap` for flexible command-line argument handling.

## Usage

```bash
gnostr-cube [OPTIONS]
```

### Options

- `-n`, `--name <NAME>`: Name of the person to greet (default: `user`).
- `-c`, `--count <COUNT>`: Number of times to greet (default: `1`).
- `-t`, `--tui`: Enable the Text User Interface. (default: `false`).
- `--chat`: Enable chat functionality. (default: `false`).
- `--cfg <CONFIG>`: Configuration string (default: `""`).

## Example

```bash
gnostr-cube --tui --chat -n Alice -c 5
```

This command will launch the `gnostr-cube` TUI with chat functionality enabled, greet "Alice" 5 times, and execute asynchronous tasks for chat and system command tests.

## Output

The tool outputs messages related to asynchronous task execution and, if the TUI is enabled, provides an interactive interface. Error messages are printed to `stderr` if any issues occur during execution.

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
