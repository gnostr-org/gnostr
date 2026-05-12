# `gnostr-js`

`gnostr-js` provides the JS/web entry points for the `gnostr` workspace.

## Docs

- [`src/bin/embedded_js.md`](src/bin/embedded_js.md)

## Browser nostr interface

The app expects a browser extension or injected provider that exposes `window.nostr`.
That interface is used as the login and signing boundary for the web app.
The adapter now lives in its own `browser-nostr.js` asset so the app code no longer touches `window.nostr` directly.

Required methods:

- `window.nostr.getPublicKey()` for login / account discovery
- `window.nostr.signEvent(event)` for signing Nostr events
- `window.nostr.nip04.encrypt(pubkey, plaintext)` and `window.nostr.nip04.decrypt(pubkey, ciphertext)` for DMs

Startup flow:

1. `gnostr_web_init()` waits for `window.nostr` to appear.
2. `gnostr_web_init_ready()` calls `get_pubkey(false)` to auto-detect an account.
3. `signin()` calls `get_pubkey()` to prompt the browser signer, then loads the app.

This is a browser plugin-style interface, not a custom login system. The web UI reads the pubkey from the provider and uses it to sign and decrypt on behalf of the user.
