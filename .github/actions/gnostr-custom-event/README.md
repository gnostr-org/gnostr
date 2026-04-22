# gnostr-custom-event GitHub Action

This reusable composite action posts a custom Nostr event (kind-1) using the `gnostr` CLI, with flexible content and tags for CI/CD workflows.

## Usage

```yaml
- name: Post custom Nostr event
  uses: ./.github/actions/gnostr-custom-event
  with:
    status: "Success!"
    job_url: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}
    matrix_label: ${{ matrix.os }}, ${{ matrix.rust }}
    pow: "16" # optional
    damus: "wss://relay.damus.io" # optional
    nos: "wss://nos.lol" # optional
    njump_base: "http://njump.me" # optional
```

## Inputs
- `status`: Human-readable status label (required)
- `job_url`: URL to the current job (required)
- `matrix_label`: Matrix label, e.g. OS/version (required)
- `pow`, `damus`, `nos`, `njump_base`: Optional advanced settings

## Output
Prints the event and a human-friendly njump.me link.

## Requirements
- The `gnostr` CLI must be installed and available in the runner environment.
