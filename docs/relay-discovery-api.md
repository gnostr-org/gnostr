# Relay discovery API

`GET http://127.0.0.1:3030/api/relay/discovery`

Returns aggregated relay metadata as JSON.

## Request

- Method: `GET`
- Headers: optional `Accept: application/json`
- Query params: none

## Response

`200 OK`

JSON array of relay entries:

```json
[
  {
    "url": "wss://relay.example",
    "contact": "ops@example.com",
    "description": "Relay description",
    "name": "Relay name",
    "software": "strfry",
    "version": "1.0.0",
    "supported_nips": [1, 11, 50],
    "supported_nip_extensions": ["nip50-search"],
    "source_nips": [1, 11, 50]
  }
]
```

## Fields

| Field | Type | Meaning |
| --- | --- | --- |
| `url` | string | Relay websocket URL, normalized as `wss://<host>` |
| `contact` | string/null | Relay contact value from relay metadata |
| `description` | string/null | Human-readable relay description |
| `name` | string/null | Relay name |
| `software` | string/null | Relay software name |
| `version` | string/null | Relay software version |
| `supported_nips` | array<number> | Union of all NIPs reported for that relay |
| `supported_nip_extensions` | array<string> | Union of all reported NIP extensions |
| `source_nips` | array<number> | NIP directories that contributed metadata for the relay |

## Behavior

- Reads relay metadata from the crawler cache on disk.
- Merges duplicate relay entries by URL.
- Preserves the first non-empty `contact`, `description`, `name`, `software`, and `version` values.
- Sorts relays by:
  1. number of supported NIPs, descending
  2. number of source NIP directories, descending
  3. URL, ascending

## Example

```bash
curl -s http://127.0.0.1:3030/api/relay/discovery | jq '.[0]'
```
