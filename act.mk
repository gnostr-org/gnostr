act-all:
	echo "act-all"

#we use -b to bind the repo to the act container
#in the single dep instances we reuse (-r) the container

ubuntu-git:submodules initialize docker-start## 	make ubuntu-git
	@export $(cat ~/gh_token.txt) && act -vbr  -W $(PWD)/.github/workflows/$@.yml

ubuntu-jq:submodules initialize docker-start## 	make ubuntu-jq
	@export $(cat ~/gh_token.txt) && act -vbr  -W $(PWD)/.github/workflows/$@.yml

ubuntu-nostcat:submodules initialize docker-start## 	make ubuntu-nostcat
	@export $(cat ~/gh_token.txt) && act -vbr -C deps/nostcat -W $(PWD)/.github/workflows/$@.yml

ubuntu-secp:submodules initialize docker-start## 	make ubuntu-secp
	@export $(cat ~/gh_token.txt) && act -vb  -W $(PWD)/.github/workflows/$@.yml

ubuntu-matrix:submodules initialize docker-start## 	make ubnutu-matrix
	@export $(cat ~/gh_token.txt) && act -vb  -W $(PWD)/.github/workflows/$@.yml
