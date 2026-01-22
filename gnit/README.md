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
