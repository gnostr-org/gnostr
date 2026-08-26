# gnostr-chat

`gnostr-chat` provides the chat transport crate used by the gnostr CLI.
The standalone binary is `gnostr-chat`; the root CLI subcommand remains `gnostr chat`.

It starts an auxiliary local relay peer for session bootstrap, and that peer now supports relay-client hole punching and Tor transport.
