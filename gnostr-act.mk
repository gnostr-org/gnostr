gnostr-act-install:## 	install gnostr-act from deps/gnostr-act/install.sh -b
	@git submodule update --init --recursive  deps/gnostr-act
	./deps/gnostr-act/install.sh -b /usr/local/bin && exec bash
ubuntu-git:submodules docker-start## 	run gnostr-act in .github
	#we use -b to bind the repo to the gnostr-act container
	#in the single dep instances we reuse (-r) the container
	@type -P gnostr-act && GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && gnostr-act $(VERBOSE) $(BIND) $(REUSE) -W $(PWD)/.github/workflows/$@.yml || $(MAKE) gnostr-act-install
.PHONY:deps/jq/.github/workflows/linux ubuntu-jq
#ubuntu-jq:submodules docker-start jq/.github/workflows/linux## 	run gnostr-act for deps/jq/.github/workflows/linux.yml
ubuntu-gnostr-cat:submodules docker-start## 	run gnostr-act in .github
	#we use -b to bind the repo to the gnostr-act container
	#in the single dep instances we reuse (-r) the container
	type -P gnostr-act && GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && gnostr-act $(VERBOSE) $(BIND) $(REUSE) -W $(PWD)/.github/workflows/$@.yml || $(MAKE) gnostr-act-install
	#the matrix/pre/release builds are for the resulting app builds
ubuntu-gnostr-legit:submodules docker-start## 	run gnostr-act in .github
	#we use -b to bind the repo to the gnostr-act container
	#in the single dep instances we reuse (-r) the container
	type -P gnostr-act && GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && gnostr-act $(VERBOSE) $(BIND) $(REUSE) -W $(PWD)/.github/workflows/$@.yml || $(MAKE) gnostr-act-install
	#the matrix/pre/release builds are for the resulting app builds
ubuntu-matrix:docker-start## 	run gnostr-act in .github
##ubuntu-matrix
## 	make ubuntu-matrix reuse=true bind=true verbose=true
	@bash -c "export GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && echo $(GITHUB_TOKEN)"
	type -P gnostr-act && GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && gnostr-act -v         -b      $(REUSE) -W $(PWD)/.github/workflows/$@.yml || $(MAKE) gnostr-act-install
alpine-matrix:docker-start## 	run gnostr-act in .github
	@bash -c "export GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && echo $(GITHUB_TOKEN)"
	type -P gnostr-act && GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && gnostr-act $(VERBOSE) $(BIND) $(REUSE) -W $(PWD)/.github/workflows/$@.yml || $(MAKE) gnostr-act-install
##alpine-matrix
## 	make alpine-matrix reuse=true bind=true verbose=true
##	
linux-matrix:docker-start ubuntu-matrix## 	linux-matrix
	$(MAKE) alpine-matrix ubuntu-secp ubuntu-gnostr ubuntu-gnostr-relay verbose=true bind=true reuse=true
ubuntu-secp:docker-start## 	ubuntu-secp
	@bash -c "export GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && echo $(GITHUB_TOKEN)"
	type -P gnostr-act && GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && gnostr-act $(VERBOSE) $(BIND) $(REUSE) -W $(PWD)/.github/workflows/$@.yml || $(MAKE) gnostr-act-install
ubuntu-gnostr:docker-start## 	ubuntu-gnostr
	@bash -c "export GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && echo $(GITHUB_TOKEN)"
	type -P gnostr-act && GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && gnostr-act $(VERBOSE) $(BIND) $(REUSE) -W $(PWD)/.github/workflows/$@.yml || $(MAKE) gnostr-act-install
ubuntu-gnostr-relay:docker-start## 	ubuntu-gnostr-relay
	@bash -c "export GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && echo $(GITHUB_TOKEN)"
	type -P gnostr-act && GITHUB_TOKEN=$(shell cat ~/GITHUB_TOKEN.txt) && gnostr-act $(VERBOSE) $(BIND) $(REUSE) -W $(PWD)/.github/workflows/$@.yml || $(MAKE) gnostr-act-install
##linux-matrix
## 	make alpine-matrix ubuntu-matrix verbose=true bind=true reuse=true
##[act.mk developement notes]
## 	use reuse=false to force a rebuild of the docker image
##[command example]
##make alpine-matrix ubuntu-matrix verbose=true bind=true reuse=true
