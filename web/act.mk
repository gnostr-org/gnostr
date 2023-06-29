push:## 	push.yml
### 	push
	$(MAKE) docker-start
	act $(VERBOSE) $(BIND) $(REUSE) -W .github/workflows/$@.yml
#NOTE: using -C for container context
ubuntu-matrix:## 	ubuntu-matrix
### 	ubuntu-matrix
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
macos-matrix:##
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vr -W $(PWD)/.github/workflows/$@.yml


alpine:## 	alpine act sequence
## alpine act sequence
## 	additional help example
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vb -W $(PWD)/.github/workflows/$@.yml
archlinux:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
centos:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
cygwin:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
debian:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
fedora:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
freebsd:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
gentoo:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
macos:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
manjaro:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
mint:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
msys2:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
netbsd:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
openbsd:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
opensuse-leap:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
static:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
ubuntu:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
voidlinux-glibc:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml
voidlinux-musl:##
	@touch ~/GITHUB_TOKEN.txt
	@export $(cat ~/GITHUB_TOKEN.txt) && act -C $(PWD) -vbr -W $(PWD)/.github/workflows/$@.yml

# vim: set noexpandtab:
# vim: set setfiletype make
