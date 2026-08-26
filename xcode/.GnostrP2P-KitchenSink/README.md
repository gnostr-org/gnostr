# MiniP2P Sample Application

This sample launches a deterministic libp2p node in SwiftUI using the local
`LibP2P-iOS` binary package.

Each runtime profile gets its own default port so iPhone, iPad, macOS, and Mac
Catalyst can run side by side:

- macOS: `10000`
- iPhone: `10001`
- iPad: `10002`
- Mac Catalyst: `10003`
- Made for iPad: `10004`

Set `P2P_LISTEN_PORT` to override the default port.
