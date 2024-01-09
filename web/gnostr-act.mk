gnostr-act-install:## 	install gnostr-act from deps/gnostr-act/install.sh -b
#	@git submodule update --init --recursive  deps/gnostr-act
#	./deps/gnostr-act/install.sh -b /usr/local/bin && exec bash
gnostr-act-gnostr:submodules docker-start## 	run gnostr-act in .github
	#we use -b to bind the repo to the gnostr-act container
	#in the single dep instances we reuse (-r) the container
	@type -P gnostr-act && GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && gnostr-act $(VERBOSE) $(BIND) $(REUSE) -W $(PWD)/.github/workflows/gnostr.yml || $(MAKE) gnostr-act-install
gnostr-act-alpine:submodules docker-start## 	run gnostr-act in .github
	#we use -b to bind the repo to the gnostr-act container
	#in the single dep instances we reuse (-r) the container
	@type -P gnostr-act && GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && gnostr-act $(VERBOSE) $(BIND) $(REUSE) -W $(PWD)/.github/workflows/gnostr-alpine.yml || $(MAKE) gnostr-act-install
