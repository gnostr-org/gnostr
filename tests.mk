gnostr-verify-keypair:gnostr gnostr-sha256 gnostr-install
##CASE 1
	echo CASE 1
	gnostr-verify-keypair $(shell gnostr --sec $(shell gnostr-sha256 $(shell gnostr-weeble)) | jq .pubkey | sed 's/\"//g') $(shell gnostr-sha256 $(shell gnostr-weeble)) || $(MAKE) bins
##CASE 2
	echo CASE 2
	gnostr-verify-keypair $(shell gnostr --sec $(shell gnostr --hash $(shell gnostr-weeble)) | jq .pubkey | sed 's/\"//g') $(shell gnostr-sha256 $(shell gnostr-weeble)) || $(MAKE) bins
