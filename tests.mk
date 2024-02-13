gnostr-verify-keypair:## 	with install sequence
	[ -x "$(shell which gnostr)" ]                || \
    $(MAKE) gnostr
	[ -x "$(shell which gnostr-weeble)" ]         || \
    $(MAKE) gnostr-install
	[ -x "$(shell which gnostr-sha256)" ]         || \
    $(MAKE) gnostr-sha256                            || \
       [ -x "$(shell which cargo)" ]                 && \
    cargo install -q gnostr-sha256
	[ -x "$(shell which gnostr-verify-keypair)" ] || \
    $(MAKE) gnostr-bins
##CASE 1
	echo CASE 1
	gnostr-verify-keypair $(shell gnostr --sec $(shell gnostr-sha256 $(shell gnostr-weeble)) | jq .pubkey | sed 's/\"//g') $(shell gnostr-sha256 $(shell gnostr-weeble)) || $(MAKE) bins
##CASE 2
	echo CASE 2
	gnostr-verify-keypair $(shell gnostr --sec $(shell gnostr --hash $(shell gnostr-weeble)) | jq .pubkey | sed 's/\"//g') $(shell gnostr-sha256 $(shell gnostr-weeble)) || $(MAKE) bins
