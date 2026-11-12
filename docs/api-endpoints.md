# API endpoints

## Web app relay API

Base: `http://127.0.0.1:3030/api/relay`

| Method | Path | Response |
| --- | --- | --- |
| `GET` | `/status` | `RelayProcessState` JSON |
| `GET` | `/discovery` | Relay discovery JSON array |
| `POST` | `/start` | `RelayProcessState` JSON |
| `POST` | `/stop` | `RelayProcessState` JSON |

`RelayProcessState`:

```json
{
  "running": true,
  "pid": 1234,
  "message": "relay already running with pid 1234"
}
```

### `GET /status`

Returns the current embedded relay state.

### `GET /discovery`

Returns the aggregated relay discovery feed. Each entry contains:

- `url`
- `contact`
- `description`
- `name`
- `software`
- `version`
- `supported_nips`
- `supported_nip_extensions`
- `source_nips`

### `POST /start`

Starts the embedded relay process if it is not already running.

### `POST /stop`

Stops the embedded relay process if it is running.

## Crawler API

Base: `http://127.0.0.1:3030`

| Method | Path | Response |
| --- | --- | --- |
| `GET` | `/query` | HTML query page |
| `GET` | `/relays.yaml` | Relay list YAML |
| `GET` | `/relays.json` | Relay list JSON |
| `GET` | `/relays.txt` | Relay list plain text |
| `GET` | `/:nip` | NIP index HTML page |
| `GET` | `/:nip/query` | NIP-scoped HTML query page |
| `GET` | `/:nip/relays.yaml` | NIP-scoped relay list YAML |
| `GET` | `/:nip/relays.json` | NIP-scoped relay list JSON |
| `GET` | `/:nip/relays.txt` | NIP-scoped relay list plain text |
| `GET` | `/:nip/:relay.json` | NIP-scoped relay metadata JSON |

## Notes

- The crawler query endpoints accept query parameters such as `relay`, `authors`, `ids`, `limit`, `kinds`, `search`, `generic_tag`, `generic_value`, `hashtag`, `mentions`, and `references`.
- The relay discovery payload is what the app uses to detect `supported_nips` when choosing NIP-aware relays.
