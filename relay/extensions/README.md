# gnostr-extensions

gostr-relay extensions.

This crate provides several extensions for [gostr-relay](https://github.com/gnostr-org/gnostr/tree/master/relay), a Nostr relay implementation in Rust.

## Extensions

- [Auth](#auth)
- [Metrics](#metrics)
- [Rate Limiter](#rate-limiter)
- [Count](#count)
- [Search](#search)

---

## Auth

Implements NIP-42 for client authentication. This allows the relay to require authentication for certain actions, such as subscribing to events or publishing new events.

### Configuration

The `auth` extension is configured in the relay's settings file.

```json
{
    "auth": {
        "enabled": true,
        "req": {
            "ip_whitelist": ["127.0.0.1"],
            "pubkey_whitelist": ["<pubkey>"],
            "ip_blacklist": [],
            "pubkey_blacklist": []
        },
        "event": {
            "event_pubkey_whitelist": ["<pubkey>"],
            "event_pubkey_blacklist": [],
            "allow_mentioning_whitelisted_pubkeys": false
        }
    }
}
```

**Options:**

- `enabled`: A boolean to enable or disable the extension.
- `req`: Permissions for `REQ` messages (subscriptions).
- `event`: Permissions for `EVENT` messages (publishing).

**Permission Options:**

- `ip_whitelist`: A list of IP addresses that are allowed to connect.
- `pubkey_whitelist`: A list of public keys that are allowed to authenticate.
- `ip_blacklist`: A list of IP addresses that are not allowed to connect.
- `pubkey_blacklist`: A list of public keys that are not allowed to authenticate.
- `event_pubkey_whitelist`: A list of public keys that are allowed to publish events.
- `event_pubkey_blacklist`: A list of public keys that are not allowed to publish events.
- `allow_mentioning_whitelisted_pubkeys`: A boolean that, if true, allows events that mention a whitelisted public key to be published.

---

## Metrics

Exposes Prometheus metrics for monitoring the relay.

### Configuration

```json
{
    "metrics": {
        "enabled": true,
        "auth": "your_secret_auth_key"
    }
}
```

**Options:**

- `enabled`: A boolean to enable or disable the extension.
- `auth`: A secret key to protect the metrics endpoint.

### Usage

The metrics are available at the `/metrics` endpoint.

```bash
curl http://127.0.0.1:8080/metrics?auth=your_secret_auth_key
```

---

## Rate Limiter

Provides rate limiting for incoming events based on IP address.

### Configuration

```json
{
    "rate_limiter": {
        "enabled": true,
        "event": [{
            "name": "default",
            "description": "Default rate limit",
            "period": 60,
            "limit": 100,
            "kinds": [[0, 10000]],
            "ip_whitelist": []
        }]
    }
}
```

**Options:**

- `enabled`: A boolean to enable or disable the extension.
- `event`: A list of rate limiting rules.

**Rule Options:**

- `name`: A name for the rule (used in metrics).
- `description`: A description of the rule (sent to clients when the rate limit is exceeded).
- `period`: The time period in seconds.
- `limit`: The maximum number of events allowed in the period.
- `kinds`: A list of event kinds to apply the rule to. This can be a list of single kinds or ranges.
- `ip_whitelist`: A list of IP addresses to exclude from rate limiting.

---

## Count

Implements NIP-45 for counting events on the relay.

### Configuration

```json
{
    "count": {
        "enabled": true
    }
}
```

**Options:**

- `enabled`: A boolean to enable or disable the extension.

---

## Search

Implements NIP-50 for searching events on the relay.

### Configuration

```json
{
    "search": {
        "enabled": true
    }
}
```

**Options:**

- `enabled`: A boolean to enable or disable the extension.

### Usage

The search functionality is available via `REQ` messages. Here are some examples using `curl` to send search requests to the relay:

**Search for events by a specific author:**

```bash
curl -H "Content-Type: application/json" -X POST -d '[
  "REQ",
  "some_subscription_id",
  {
    "authors": ["<pubkey>"],
    "search": "some query"
  }
]' http://127.0.0.1:8080
```

**Search for events of a specific kind:**

```bash
curl -H "Content-Type: application/json" -X POST -d '[
  "REQ",
  "some_subscription_id",
  {
    "kinds": [1],
    "search": "some query"
  }
]' http://127.0.0.1:8080
```

**Search for events since a specific time, with a limit:**

```bash
curl -H "Content-Type: application/json" -X POST -d '[
  "REQ",
  "some_subscription_id",
  {
    "since": 1648848000,
    "limit": 10,
    "search": "some query"
  }
]' http://127.0.0.1:8080
```