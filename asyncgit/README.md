## gnit:gnostr git server

cargo install --bins --path .
gnostr-gnit
gnostr-gnit-server

### remotes example

```
local	ssh://127.0.0.1:2222/gnostr-gnit.git (fetch)
local	ssh://127.0.0.1:2222/gnostr-gnit.git (push)
origin	git@github.com:gnostr-org/gnostr-gnit.git (fetch)
origin	git@github.com:gnostr-org/gnostr-gnit.git (push)
```

git push ssh://127.0.0.1:2222/.gnostr/$(gnostr-weeble)/$(gnostr-blockheight)/$(gnostr-wobble)

## Testing

Run the full asyncgit test suite, including the Nostr event paths and the full NIP-34 matrix:

```sh
./scripts/gnostr-asyncgit-tests.sh --nocapture
```

That matrix now covers plain/mined commits, plain/mined notes, and plain/PoW events, and each case is also replayed as a NIP-44 DM to the shared default recipient key.

Note: `nostr_sdk` is only used in test code here; production asyncgit paths stay on the repo's own types.

Useful variants:

```sh
./scripts/gnostr-asyncgit-tests.sh --ignored --nocapture
./scripts/asyncgit-tests.sh --nocapture
cargo test -p gnostr-asyncgit --all-targets --features nostr -- --nocapture
```
