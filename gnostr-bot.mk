gnostr-bot:## 	gnostr-act automated sequence
## --container-architecture linux/amd64
	[[ -x "$(shell which gnostr-act)" ]] && echo $(shell which gnostr-act)
	[[ -x "$(shell which gnostr-act)" ]] && \
		$(shell which gnostr-act) \
		-vbr \
		-W \
		./gnostr-bot.yml \
		--container-architecture linux/amd64
