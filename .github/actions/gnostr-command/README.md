# gnostr-command GitHub Action

Runs `gnostr` with safe, reusable argument handling.

## Inputs

- `command`: subcommand name
- `help_command`: optional command path to show `-h` for first
- `nsec`: optional nsec, including the default `$(gnostr --hash "$GITHUB_REPOSITORY_ID")` placeholder
- `pow`: optional `-d` value
- `relays`: newline-delimited relay URLs
- `args`: newline-delimited argv items; each line becomes one argument
- `hex`: append `--hex`
- `convert_note`: convert extracted event id to a note id
- `query_relays`: newline-delimited relay URLs to query after execution
- `njump_base`: optional base URL for note links
- `ignore_failure`: ignore non-zero exit status from `gnostr`

## Outputs

- `raw_output`
- `event_id`
- `note`
- `njump_url`
