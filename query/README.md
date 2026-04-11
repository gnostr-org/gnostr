## gnostr-query

### construct nostr queries

```sh
Usage: gnostr-query [OPTIONS]

Options:
  -a, --authors <authors>        Comma-separated list of authors
  -p, --mentions <mentions>      Comma-separated list of mentions
  -e, --references <references>  Comma-separated list of references
  -t, --hashtag <hashtag>        Comma-separated list of hashtags
  -i, --ids <ids>                Comma-separated list of ids
  -k, --kinds <kinds>            Comma-separated list of kinds (integers)
  -g, --generic <tag> <value>    Generic tag query: #<tag>: value
  -l, --limit <limit>            Limit the number of results
  -h, --help                     Print help
```

## Install gnostr-query 0.0.4

### Install prebuilt binaries via shell script

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/gnostr-org/gnostr-query/releases/download/v0.0.4/gnostr-query-installer.sh | sh
```

### Install prebuilt binaries via powershell script

```sh
powershell -ExecutionPolicy Bypass -c "irm https://github.com/gnostr-org/gnostr-query/releases/download/v0.0.4/gnostr-query-installer.ps1 | iex"
```

### Install prebuilt binaries via Homebrew

```sh
brew install gnostr-org/gnostr-org/gnostr-query
```

## Download gnostr-query 0.0.4

|  File  | Platform | Checksum |
|--------|----------|----------|
| [gnostr-query-aarch64-apple-darwin.tar.xz](https://github.com/gnostr-org/gnostr-query/releases/download/v0.0.4/gnostr-query-aarch64-apple-darwin.tar.xz) | Apple Silicon macOS | [checksum](https://github.com/gnostr-org/gnostr-query/releases/download/v0.0.4/gnostr-query-aarch64-apple-darwin.tar.xz.sha256) |
| [gnostr-query-x86_64-apple-darwin.tar.xz](https://github.com/gnostr-org/gnostr-query/releases/download/v0.0.4/gnostr-query-x86_64-apple-darwin.tar.xz) | Intel macOS | [checksum](https://github.com/gnostr-org/gnostr-query/releases/download/v0.0.4/gnostr-query-x86_64-apple-darwin.tar.xz.sha256) |
| [gnostr-query-x86_64-pc-windows-msvc.zip](https://github.com/gnostr-org/gnostr-query/releases/download/v0.0.4/gnostr-query-x86_64-pc-windows-msvc.zip) | x64 Windows | [checksum](https://github.com/gnostr-org/gnostr-query/releases/download/v0.0.4/gnostr-query-x86_64-pc-windows-msvc.zip.sha256) |
| [gnostr-query-x86_64-unknown-linux-gnu.tar.xz](https://github.com/gnostr-org/gnostr-query/releases/download/v0.0.4/gnostr-query-x86_64-unknown-linux-gnu.tar.xz) | x64 Linux | [checksum](https://github.com/gnostr-org/gnostr-query/releases/download/v0.0.4/gnostr-query-x86_64-unknown-linux-gnu.tar.xz.sha256) |


Usage:

```
gnostr-query -i 9f832fda858fdddb86c5c79dcc271767804fb3562ed5eea8c8be4f19be8e9cdc
```
