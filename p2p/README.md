## Attestation syndication

`p2p` mirrors the `asyncgit` attestation structure: deterministic fixture keys, chronological commit/event/note ordering, and `notes_ref` chaining for public attestations.

The p2p tests load crawler relay buckets and publish attestation events through the relay bridge so the syndication path stays wired to the crawler-discovered relay list.

## Testing help

- Use `--nocapture` so the printed attestation payload and relay destinations stay visible.
- `pretty_print_attestations` keeps the same commit → event → note shape as `asyncgit`.
- Real-event coverage must not assume buckets are pre-primed; the test should fetch the live crawler relay list and seed a temp bucket from that output before broadcasting.
- If the crawler server is not already running, start it first:

```sh
gnostr crawler serve --port 8080 --detach
```

- The live crawler relay list is served from `http://127.0.0.1:8080/relays.yaml`.
