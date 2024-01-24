gnostr-verify-keys:
	gnostr-verify-keypair $(shell gnostr --sec $(shell gnostr-sha256 $(shell gnostr-weeble)) | jq .pubkey | sed 's/\"//g') $(shell gnostr-sha256 $(shell gnostr-weeble)) || $(MAKE) bins
