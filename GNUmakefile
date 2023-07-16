SHELL                                   := /bin/bash
PWD 									?= pwd_unknown
TIME 									:= $(shell date +%s)
export TIME
OS                                      :=$(shell uname -s)
export OS
OS_VERSION                              :=$(shell uname -r)
export OS_VERSION
ARCH                                    :=$(shell uname -m)
export ARCH
ifeq ($(ARCH),x86_64)
TRIPLET                                 :=x86_64-linux-gnu
export TRIPLET
endif
ifeq ($(ARCH),arm64)
TRIPLET                                 :=aarch64-linux-gnu
export TRIPLET
endif
ifeq ($(ARCH),arm64)
TRIPLET                                 :=aarch64-linux-gnu
export TRIPLET
endif
LEGIT									:= $(which legit)
export LEGIT
CARGO_PATH								:=$(HOME)/.cargo
export CARGO_PATH
#PATH									:=$(shell sudo -su $(USER) $(CARGO_PATH))/bin:$(PATH)
GIT_STATUS								:= $(shell  git status --ignore-submodules=dirty --porcelain=2 -s)
export GIT_STATUS
ifeq ($(project),)
PROJECT_NAME							:= $(notdir $(PWD))
else
PROJECT_NAME							:= $(project)
endif
export PROJECT_NAME

ifeq ($(reuse),true)
REUSE                                   :=-r
else
REUSE                                   :=	
endif
export REUSE
ifeq ($(bind),true)
BIND                                   :=-b
else
BIND                                   :=      
endif
export BIND
ifneq ($(job),)
JOB                                     :=-j $(job)
else
JOB                                   :=	
endif
export JOB


ifeq ($(port),)
PORT									:= 0
else
PORT									:= $(port)
endif
export PORT

#GIT CONFIG
GIT_USER_NAME							:= $(shell git config user.name)
export GIT_USER_NAME
GH_USER_NAME							:= $(shell git config user.name)
#MIRRORS
GH_USER_REPO    						:= $(GH_USER_NAME).github.io
KB_USER_REPO   	        				:= $(GH_USER_NAME).keybase.pub
#GITHUB RUNNER CONFIGS
ifneq ($(ghuser),)
GH_USER_NAME := $(ghuser)
GH_USER_REPO := $(ghuser).github.io
endif
ifneq ($(kbuser),)
KB_USER_NAME := $(kbuser)
KB_USER_REPO := $(kbuser).keybase.pub
endif
export GIT_USER_NAME
export GH_USER_REPO
export KB_USER_REPO

GIT_USER_EMAIL							:= $(shell git config user.email)
export GIT_USER_EMAIL
GIT_SERVER								:= https://github.com
export GIT_SERVER
GIT_SSH_SERVER							:= git@github.com
export GIT_SSH_SERVER
GIT_PROFILE								:= $(shell git config user.name)
export GIT_PROFILE
GIT_BRANCH								:= $(shell git rev-parse --abbrev-ref HEAD)
export GIT_BRANCH
GIT_HASH								:= $(shell git rev-parse --short HEAD)
export GIT_HASH
GIT_PREVIOUS_HASH						:= $(shell git rev-parse --short master@{1})
export GIT_PREVIOUS_HASH
GIT_REPO_ORIGIN							:= $(shell git remote get-url origin)
export GIT_REPO_ORIGIN
GIT_REPO_NAME							:= $(PROJECT_NAME)
export GIT_REPO_NAME
GIT_REPO_PATH							:= $(HOME)/$(GIT_REPO_NAME)
export GIT_REPO_PATH

BASENAME := $(shell basename -s .git `git config --get remote.origin.url`)
export BASENAME

# Force the user to explicitly select public - public=true
# export KB_PUBLIC=public && make keybase-public
ifeq ($(public),true)
KB_PUBLIC  := public
else
KB_PUBLIC  := private
endif
export KB_PUBLIC

ifeq ($(libs),)
LIBS  := ./libs
else
LIBS  := $(libs)
endif
export LIBS

SPHINXOPTS            =
SPHINXBUILD           = sphinx-build
PAPER                 =
BUILDDIR              = _build
PRIVATE_BUILDDIR      = _private_build

# Internal variables.
PAPEROPT_a4           = -D latex_paper_size=a4
PAPEROPT_letter       = -D latex_paper_size=letter
ALLSPHINXOPTS         = -d $(BUILDDIR)/doctrees $(PAPEROPT_$(PAPER)) $(SPHINXOPTS) .
PRIVATE_ALLSPHINXOPTS = -d $(PRIVATE_BUILDDIR)/doctrees $(PAPEROPT_$(PAPER)) $(SPHINXOPTS) .
# the i18n builder cannot share the environment and doctrees with the others
I18NSPHINXOPTS  = $(PAPEROPT_$(PAPER)) $(SPHINXOPTS) .

HOMEBREW_NO_ENV_HINTS=0
export HOMEBREW_NO_ENV_HINTS

.PHONY: init
init:
	@awk 'BEGIN {FS = ":.*?##"} /^[a-zA-Z_-]+:.*?##/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: help
help:## 	verbose help
	@echo ""
	@echo verbose $@
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' |  sed -e 's/^/ /'
	@echo ""

.PHONY: report
report:## 	report
	@echo ''
	@echo '	[ARGUMENTS]	'
	@echo '      args:'
	@echo '        - TIME=${TIME}'
	@echo '        - PROJECT_NAME=${PROJECT_NAME}'
	@echo ''
	@echo '        - LEGIT=${LEGIT}'
	@echo '        - PATH=${PATH}'
	@echo '        - CARGO_PATH=${CARGO_PATH}'
	@echo ''
	@echo '        - GIT_USER_NAME=${GIT_USER_NAME}'
	@echo '        - GH_USER_REPO=${GH_USER_REPO}'
	@echo '        - GIT_USER_EMAIL=${GIT_USER_EMAIL}'
	@echo '        - GIT_SERVER=${GIT_SERVER}'
	@echo '        - GIT_PROFILE=${GIT_PROFILE}'
	@echo '        - GIT_BRANCH=${GIT_BRANCH}'
	@echo '        - GIT_HASH=${GIT_HASH}'
	@echo '        - GIT_PREVIOUS_HASH=${GIT_PREVIOUS_HASH}'
	@echo '        - GIT_REPO_ORIGIN=${GIT_REPO_ORIGIN}'
	@echo '        - GIT_REPO_NAME=${GIT_REPO_NAME}'
	@echo '        - GIT_REPO_PATH=${GIT_REPO_PATH}'
	@echo ''
	@echo '        - HOMEBREW_NO_ENV_HINTS=${HOMEBREW_NO_ENV_HINTS}'

.PHONY: git-add
.ONESHELL:
git-add:## 	git-add
	git config advice.addIgnoredFile false
	git add --ignore-errors GNUmakefile
	git add --ignore-errors legit.mk
	git add --ignore-errors cargo.mk
	git add --ignore-errors README.md
	git add --ignore-errors *.html
	git add --ignore-errors TIME
	git add --ignore-errors CNAME
	git add --ignore-errors .gitignore
	git add --ignore-errors .github
	git add --ignore-errors *.sh
	git add --ignore-errors *.yml

.PHONY: push
.ONESHELL:
push: touch-time git-add## 	push
	test legit && legit . -p 00000 -m "make: push - $(shell date +%s)"
	@git push -f origin	+master:master

.PHONY: branch
.ONESHELL:
branch: docs touch-time touch-block-time## 	branch
	git add --ignore-errors GNUmakefile TIME GLOBAL .github *.sh *.yml
	git add --ignore-errors .github
	git commit -m 'make branch by $(GIT_USER_NAME) on $(TIME)'
	git branch $(TIME)
	git push -f origin $(TIME)

.PHONY: touch-time
.ONESHELL:
touch-time: remove## 	touch-time
	@echo $(TIME) $(shell git rev-parse HEAD) > TIME

commit: touch-time git-add## 	commit
	#@./automate.sh
	@test legit && legit . -p 00000 -m "$(shell date +%s):make automate"

.PHONY: docs
docs: touch-time git-add## 	docs
	bash -c "if pgrep MacDown; then pkill MacDown; fi"
	bash -c "if hash pandoc 2>/dev/null; then echo; fi || brew install pandoc"
	#bash -c 'pandoc -s README.md -o index.html  --metadata title="$(PROJECT_NAME)" '
	#bash -c 'pandoc -s README.md -o index.html  --metadata title="" '
	pandoc -f markdown -t html README.md -o index.html \
		                        -s --metadata title=" "
	#	--css=github-pandoc.css -s --metadata title=" "
	#$(MAKE) git-add
	#test legit && legit . -p 000000 -m "make: docs - $(shell date +%s)"
	#git ls-files -co --exclude-standard | grep '\.md/$\' | xargs git

.PHONY: legit
.ONESHELL:
install:legit-install
legit:## 	legit
	@$(MAKE) -f legit.mk
tag:
	@git tag $(OS)-$(OS_VERSION)-$(ARCH)-$(shell date +%s)
	@git push -f --tags

.PHONY: clean
.ONESHELL:
remove:## 	remove
	bash -c "rm -rf TIME"
clean: touch-time## 	clean
	bash -c "rm -rf TIME"
	bash -c "rm -rf $(BUILDDIR)"

.PHONY: failure
failure:
	@-/bin/false && ([ $$? -eq 0 ] && echo "success!") || echo "failure!"
.PHONY: success
success:
	@-/bin/true && ([ $$? -eq 0 ] && echo "success!") || echo "failure!"

cargo:## 	
	$(MAKE) -f cargo.mk
-include cargo.mk
act:## 	
	$(MAKE) -f act.mk
-include act.mk
-include legit.mk
# vim: set noexpandtab:
# vim: set setfiletype make
