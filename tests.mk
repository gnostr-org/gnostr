gnostr-verify-keys:gnostr gnostr-install
	gnostr-verify-keypair $(shell gnostr --sec $(shell gnostr --hash $(shell gnostr-weeble)) | jq .pubkey | sed 's/\"//g') $(shell gnostr --hash $(shell gnostr-weeble)) || $(MAKE) bins
gnostr-verify-keys-with-gnostr-sha256:gnostr gnostr-sha256 gnostr-install
	gnostr-verify-keypair $(shell gnostr --sec $(shell gnostr-sha256 $(shell gnostr-weeble)) | jq .pubkey | sed 's/\"//g') $(shell gnostr-sha256 $(shell gnostr-weeble)) || $(MAKE) bins
