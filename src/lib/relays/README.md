# `gnostr-relays` Module

This module provides an interface for interacting with the `api.nostr.watch` REST API, specifically focusing on relay state aggregation. The API documentation can be found [here](https://api.nostr.watch/v2/).

## Base URL

The base URL for all API calls made by this module is `https://api.nostr.watch/v2/`.

## Implemented Functions

Each function in this module corresponds to a specific endpoint in the `api.nostr.watch` OpenAPI specification.

### `relays_by_nip(nip: &str) -> Result<String>`

Retrieves relays grouped by their supported NIP (Nostr Implementation Profile).

- **API Endpoint:** `/relays/by/nip/{nip}`
- **Description:** Get relays grouped by NIP support.
- **Parameters:**
  - `nip` (path parameter): The NIP number (e.g., "1", "9").

### `relays_all() -> Result<String>`

Retrieves a list of all relay states.

- **API Endpoint:** `/relays`
- **Description:** List all relay states with pagination (detailed format by default).
- **Note:** This function currently does not support pagination parameters (`limit`, `offset`, `sortBy`, `sortOrder`, `format`). It fetches all available relays with the default settings.

### `relays_online() -> Result<String>`

Retrieves a list of currently online relays.

- **API Endpoint:** `/relays/online` (POST request in OpenAPI, but this implementation uses GET for simplicity as `/relays/online` also works with GET in v2 by default)
- **Description:** Get currently online relays with optional label filtering.
- **Note:** This function currently does not support advanced filtering parameters (e.g., `onlineWindowSeconds`, `network`, `labels`) available in the POST request of the OpenAPI spec.

### `relays_paid() -> Result<String>`

Retrieves a list of paid relays.

- **API Endpoint:** `/relays/paid`
- **Description:** This endpoint is not explicitly listed in the provided OpenAPI `v2` YAML. This function might need adjustment if `v2` has a different endpoint or if it requires specific request body parameters for paid relays. The current implementation assumes a GET request to `/relays/paid` is still valid in `v2`.

### `relays_offline() -> Result<String>`

Retrieves a list of recently seen but currently offline relays.

- **API Endpoint:** `/relays/offline` (POST request in OpenAPI, but this implementation uses GET for simplicity as `/relays/offline` also works with GET in v2 by default)
- **Description:** Get recently seen but currently offline relays with optional label filtering.
- **Note:** This function currently does not support advanced filtering parameters (e.g., `offlineSeenSeconds`, `offlineThresholdSeconds`, `network`, `labels`) available in the POST request of the OpenAPI spec.

## Error Handling

All public functions in this module return `anyhow::Result<String>`, allowing for flexible error handling. The `String` contains the raw JSON response from the API. Any network or parsing errors are encapsulated within the `anyhow::Error` type.
