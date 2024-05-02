JQ=$(shell which jq)
export JQ
XQ=$(shell which gnostr-xq)
export XQ
gnostr-verify-keypair:## 	with install sequence
##echo $(JQ)
##echo $(XQ)
	[ -x "$(shell which gnostr)" ]                || \
		$(MAKE) gnostr
	[ -x "$(shell which gnostr-xq)" ]             || \
		$(MAKE) gnostr-xq
	[ -x "$(shell which gnostr-sha256)" ]         || \
		cargo install --path ./bins --bin gnostr-sha256
	[ -x "$(shell which gnostr-weeble)" ]         || \
		cargo install --path ./bins --bin gnostr-weeble
	[ -x "$(shell which gnostr-wobble)" ]         || \
		cargo install --path ./bins --bin gnostr-wobble
	[ -x "$(shell which gnostr-blockheight)" ]    || \
		cargo install --path ./bins --bin gnostr-blockheight
	[ -x "$(shell which cargo)" ]                 || \
	$(MAKE) gnostr-bins
	[ -x "$(shell which gnostr-verify-keypair)" ] || \
		cargo install --path ./bins --bin gnostr-verify-keypair
##CASE 1
	echo CASE 1
	gnostr-verify-keypair $(shell gnostr --sec $(shell gnostr-sha256 $(shell gnostr-weeble)) | $(XQ) .pubkey | sed 's/\"//g') $(shell gnostr-sha256 $(shell gnostr-weeble)) || $(MAKE) bins
##CASE 2
	echo CASE 2
	gnostr-verify-keypair $(shell gnostr --sec $(shell gnostr --hash $(shell gnostr-weeble)) | $(XQ) .pubkey | sed 's/\"//g') $(shell gnostr-sha256 $(shell gnostr-weeble)) || $(MAKE) bins

# vim: set noexpandtab:
# vim: set setfiletype make
