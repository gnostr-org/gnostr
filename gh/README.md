# gh.rs
[GitHub](https://github.com/gh0st-work/gh.rs)

[![Lines Of Code](https://tokei.rs/b1/github/gh0st-work/gh.rs?category=code)](https://github.com/gh0st-work/gh.rs)

**CLI tool written in Rust & one-liner to provide extra GitHub capabilities: create repo, fork public repo as private**

`bash <(curl -s https://raw.githubusercontent.com/gh0st-work/gh.rs/main/install_prebuilt_binary.sh)`

## Motivation
Just wanted to fork one repo as private, 
but found that it's kinda complicated & not a one-liner & requires web GUI usage.

Then I wrote `gh.sh` script, 
which allowed me to do this, 
and also was creating repositories without GUI through the GitHub API.

It was ~400 LOC, 
with the same args parsing & `fork-private` and `new` commands.

But, you know, sh-scripts are quite unstable,
certainly not for 400 LOC, 
and the sh-syntax was simply not particularly suitable for such tasks, 
so I started rewriting it in Rust, 
realizing that this engineering solution would also 
allow to implement **search dream-TUI in the future**.

## Installation
**Linux only** for now. I provided some installation options to select from for your comfort.

### Script to install prebuilt binary
`bash <(curl -s https://raw.githubusercontent.com/gh0st-work/gh.rs/main/install_prebuilt_binary.sh)`

**Pros:**
- Fast

**Cons:**
- Can possibly not work for your machine

**Detailed steps:**
- Logs every command
- Checks required commands and permissions 
- Extracts your machine kernel info and architecture
- Creates temporary directory
- Fetches the latest release version number
- Downloads the tarball archive with the latest release version of the prebuilt binary 
- Unpacks the tarball archive
- Renames `gh` binary to `gh.rs` (as cargo does not allow to build binaries with dots in name, and I... don't give a fuck, especially about censorship)
- Gives execute (+x) permissions to `gh.rs` binary
- Tries to run `gh.rs --help` as test command to verify successful installation
- Moves `./gh.rs` to `/bin/gh.rs`
- Adds `/bin/gh.rs` to `PATH` (`/etc/profile.d/gh.rs.sh`)
- Cleans up

### Script to install from source
`bash <(curl -s https://raw.githubusercontent.com/gh0st-work/gh.rs/main/install_from_source.sh)`

**Pros:**
- Stable af

**Cons:**
- Long installation time

**Detailed steps:**
- Logs every command
- Checks required commands and permissions 
- Extracts your machine kernel info and architecture
- Creates temporary directory
- Installs Rust & cargo if not installed
- Fetches the latest release version number
- Downloads the tarball archive with the latest release version of the source code 
- Unpacks the tarball archive
- Builds binary from the source code with Rust
- Renames `gh` binary to `gh.rs` (as cargo does not allow to build binaries with dots in name, and I... don't give a fuck, especially about censorship)
- Gives execute (+x) permissions to `gh.rs` binary
- Tries to run `gh.rs --help` as test command to verify successful installation
- Moves `./gh.rs` to `/bin/gh.rs`
- Adds `/bin/gh.rs` to PATH (`/etc/profile.d/gh.rs.sh`)
- Cleans up

## Usage
`gh.rs help-full`
```
Usage: gh.rs <COMMAND>

Commands:
  new, n [OPTIONS]           Create new repo (local & GitHub)
    Options:
      -n, --name <name>                Set new repo name
      -d, --description <description>  Set new repo description [aliases: descr]
      -p, --public                     Make repo public [aliases: pub, is-public, make-public]
      -t, --token <access_token>       Set GitHub access token, filled automatically if stored by git [aliases: tok, access-token]
      -c, --cli-only                   CLI-only mode, no prompts, will error if something is not specified, all bools will be set to false automatically [aliases: co, np, no-prompt, no-prompts, no-prompting]
      -h, --help                       Print help
    
  publish, p, pub [OPTIONS]  Publish current directory to GitHub
    Options:
      -d, --description <description>  Set new repo description [aliases: descr]
      -p, --public                     Make repo public [aliases: pub, is-public, make-public]
      -t, --token <access_token>       Set GitHub access token, filled automatically if stored by git [aliases: tok, access-token]
      -c, --cli-only                   CLI-only mode, no prompts, will error if something is not specified, all bools will be set to false automatically [aliases: co, np, no-prompt, no-prompts, no-prompting]
      -h, --help                       Print help
    
  clone, c [OPTIONS]         Clone GitHub repo
    Options:
      -e, --external <repo_url>   Set external repo url
      -t, --token <access_token>  Set GitHub access token, filled automatically if stored by git [aliases: tok, access-token]
      -c, --cli-only              CLI-only mode, no prompts, will error if something is not specified, all bools will be set to false automatically [aliases: co, np, no-prompt, no-prompts, no-prompting]
      -h, --help                  Print help
    
  fork, f [OPTIONS]          Fork GitHub repo
    Options:
      -e, --external <repo_url>   Set external repo url
      -n, --name <name>           Set new repo name
      -p, --public                Make repo public [aliases: pub, is-public, make-public]
      -t, --token <access_token>  Set GitHub access token, filled automatically if stored by git [aliases: tok, access-token]
      -c, --cli-only              CLI-only mode, no prompts, will error if something is not specified, all bools will be set to false automatically [aliases: co, np, no-prompt, no-prompts, no-prompting]
      -h, --help                  Print help
    
  help-full                  Print help fully, describing every command

gh.rs GitHub: https://github.com/gh0st-work/gh.rs
```

### Examples
#### `new`:
- `gs.rs new --cli-only --name my-repo --description "My new repo description" --public --token ghp_...`
- `gs.rs n -c -n my-repo -d "My new repo description" -p -t ghp_...`
- `gs.rs new -c -n my-repo -d "My new repo description" -p`
- `gs.rs new -c -n my-repo -d "My new repo description" -p`
- `gs.rs new -c -n my-repo -d "My new repo description"`
- `gs.rs new -c -n my-repo -d "My new repo description"`
- ```
  gs.rs new -n my-repo -d "My new repo description"
  Make it public? y(es) / n(o) [no]: yes   
  ```
- ```
  gs.rs new -n my-repo
  GitHub new repo description: My new repo description
  Make it public? y(es) / n(o) [no]: y
  ```
- ```
  gs.rs new
  GitHub new repo name: my-repo
  GitHub new repo description: My new repo description
  Make it public? y(es) / n(o) [no]: n
  ```
- ```
  gs.rs new
  GitHub access token: ghp_...
  GitHub new repo name: my-repo
  GitHub new repo description: My new repo description
  Make it public? y(es) / n(o) [no]:
  ```
#### `publish`:
- `gs.rs publish --cli-only --description "My new repo description" --public --token ghp_...`
- `gs.rs publish -c -d "My new repo description" -p -t ghp_...`
- `gs.rs pub -c -d "My new repo description" -p -t ghp_...`
- `gs.rs p -c -d "My new repo description" -p -t ghp_...`
- `gs.rs pub -c -d "My new repo description" -p`
- `gs.rs pub -c -d "My new repo description" -p`
- `gs.rs pub -c -d "My new repo description"`
- `gs.rs pub -c -d "My new repo description"`
- ```
  gs.rs pub -d "My new repo description"
  Make it public? y(es) / n(o) [no]: no   
  ```
- ```
  gs.rs pub
  GitHub new repo description: My new repo description
  Make it public? y(es) / n(o) [no]: n
  ```
- ```
  gs.rs pub
  GitHub access token: ghp_...
  GitHub new repo description: My new repo description
  Make it public? y(es) / n(o) [no]:
  ```
#### `clone`:
- `gs.rs clone --cli-only --external https://github.com/gh0st-work/gh.rs.git --token ghp_...`
- `gs.rs clone -c -e gh0st-work/gh.rs -t ghp_...`
- `gs.rs c -c -e gh0st-work/gh.rs -t ghp_...`
- `gs.rs clone -c -e gh0st-work/gh.rs`
- ```
  gs.rs clone
  GitHub external repo url: gh0st-work/gh.rs
  ```
- ```
  gs.rs clone
  GitHub access token: ghp_...
  GitHub external repo url: gh0st-work/gh.rs
  ```
#### `fork`:
- `gs.rs fork --cli-only --external https://github.com/gh0st-work/gh.rs.git --name gh.rs-fork --public --token ghp_...`
- `gs.rs f -c -e gh0st-work/gh.rs -n gh.rs-fork -p -t ghp_...`
- `gs.rs fork -c -e gh0st-work/gh.rs -n gh.rs-fork -p -t ghp_...`
- `gs.rs fork -c -e gh0st-work/gh.rs -n gh.rs-fork -p`
- `gs.rs fork -c -e gh0st-work/gh.rs -n gh.rs-fork`
- `gs.rs fork -e gh0st-work/gh.rs -n gh.rs-fork`
- ```
  gs.rs fork -e gh0st-work/gh.rs -n gh.rs-fork
  Make it public? y(es) / n(o) [no]: yes 
  ```
- ```
  gs.rs fork -e gh0st-work/gh.rs
  GitHub new repo name [gh.rs]: gh.rs-fork 
  Make it public? y(es) / n(o) [no]: y
  ```
- ```
  gs.rs fork
  GitHub external repo url: gh0st-work/gh.rs
  GitHub new repo name [gh.rs]:
  Make it public? y(es) / n(o) [no]: no
  ```
- ```
  gs.rs fork
  GitHub external repo url: gh0st-work/gh.rs
  GitHub new repo name [gh.rs]:
  Make it public? y(es) / n(o) [no]: n
  ```
- ```
  gs.rs fork
  GitHub access token: ghp_...
  GitHub external repo url: gh0st-work/gh.rs
  GitHub new repo name [gh.rs]:
  Make it public? y(es) / n(o) [no]:
  ```

## Development

### Build on top of:
- [tokio](https://crates.io/crates/tokio), [async_std](https://crates.io/crates/async_std) — for async
- [clap](https://crates.io/crates/clap) — for CLI args parsing
- [octocrab](https://crates.io/crates/octocrab), [serde](https://crates.io/crates/serde), [serde_json](https://cretes.io/crates/serde_json), [url](https://crates.io/crates/url) — for GitHub API access
- [git2](https://crates.io/crates/git2) — for git repo management
- [configparser](https://crates.io/crates/configparser) — for parsing ~/.gitconfig
- [ssh_key](https://crates.io/crates/ssh_key), [rand_code](https://crates.io/crates/rand_core) — for SSH keys management
- [regex](https://crates.io/crates/regex), [home](https://crates.io/crates/home), [thiserror](https://crates.io/crates/thiserror), [chrono](https://crates.io/crates/chrono), [terminal_size](https://crates.io/crates/terminal_size), [anstream](https://crates.io/crates/anstream) — as other helpers  

### TODO (& devlog):
- [x] Concept-proof with `gh.sh` (same commands in shell script)
- [x] Create `gh.rs` project, start rewritting in Rust
- [x] Start trying to implement command `new`
- [x] Implement clap-like `cmd!` macro
- [x] Implement `get_github_token_from_machine` & `get_auth` auth (get `octocrab_client` & token verification)
- [x] Implement `git2_add_and_commit` & `get_github_signature_from_machine`
- [x] Implement `octocrab_repos_create`
- [x] Implement `git2_credentials` & `get_or_create_ssh_key` as credentials (containing ssh key) required to perform push
- [x] Implement `git2_push`
- [x] Implement & build & simple test command `new` & command `publish`
- [x] Start trying to implement command `fork` (private)
- [x] Implement `git2_clone`
- [x] Implement `git2_push` `mirror` option as it required to perform fork
- [x] Implement `git2_remote_recreate` & `git2_remote_delete` fix as [`git2`](https://crates.io/crates/git) -> [`Repository::remote_delete`](https://docs.rs/git2/latest/git2/struct.Repository.html#method.remote_delete) -> [`ligbgit2`](https://github.com/libgit2/libgit2) -> [`git_remote_delete`](https://github.com/libgit2/libgit2/blob/45fd9ed7ae1a9b74b957ef4f337bc3c8b3df01b5/src/libgit2/remote.c#L2842) -> [`git_config_rename_section`](https://github.com/libgit2/libgit2/blob/45fd9ed7ae1a9b74b957ef4f337bc3c8b3df01b5/src/libgit2/config.c#L1528) -> [`rename_config_entries_cb`](https://github.com/libgit2/libgit2/blob/45fd9ed7ae1a9b74b957ef4f337bc3c8b3df01b5/src/libgit2/config.c#L1505) -> [`:1523`](https://github.com/libgit2/libgit2/blob/45fd9ed7ae1a9b74b957ef4f337bc3c8b3df01b5/src/libgit2/config.c#L1523) fails when having multivar refspecs on remote in config (and they are required to perform push mirror)
- [x] Implement `git2_fetch_until_commit` as GitHub can be too slow for our blazingly fast application kekw
- [x] Implement & build & simple test command `fork` & command `clone`
- [x] Implement `GhRsError` & `GhRsResult` & handle errors correctly
- [x] Implement `cli-only` mode
- [x] Start implementing `install_prebuilt_binary.sh` script: to download & insert to PATH bin, that is automatically built by GitHub Action
- [x] Implement `git2_add_all_and_commit`
- [x] Implement `git2_set_branch_upstream` & `git2_default_branch_name`
- [x] Start implementing `install_from_source.sh` script, same as `install_prebuild_binary.sh`, but builds from sources
- [x] Fill initial `README.md`
- [x] Create this repo
- [x] Set up GitHub Actions to build Rust bins
- [x] Implement & simple test `install_prebuilt_binary.sh`
- [x] Implement & simple test `install_from_source.sh`
- [x] Compliment myself, bcuz... damn this is not bad for 4 days & first ever Rust project
- [x] Modify `--help` outputs & implement `help-full` command
- [ ] Write tests 
- [ ] Start implementing [lazyhub](https://github.com/ryo-ma/lazyhub)-like `search` command TUI with [ratatui](https://github.com/ratatui-org/ratatui)
- [ ] Start implementing [cliclack](https://github.com/fadeevab/cliclack)-like TUI for other commands


