act-all:
	echo "act-all"
ubuntu-git:submodules docker-start## 	run act in .github
	#we use -b to bind the repo to the act container
	#in the single dep instances we reuse (-r) the container
	@export $(cat ~/gh_token.txt) && act -vbr  -W $(PWD)/.github/workflows/$@.yml
ubuntu-jq:submodules docker-start## 	run act in .github
	#we use -b to bind the repo to the act container
	#in the single dep instances we reuse (-r) the container
	@export $(cat ~/gh_token.txt) && act -vbr  -W $(PWD)/.github/workflows/$@.yml
ubuntu-nostcat:submodules docker-start## 	run act in .github
	#we use -b to bind the repo to the act container
	#in the single dep instances we reuse (-r) the container
	@export $(cat ~/gh_token.txt) && act -vbr  -W $(PWD)/.github/workflows/$@.yml

	#the matrix/pre/release builds are for the resulting app builds
ubuntu-matrix:submodules docker-start## 	run act in .github
	@export $(cat ~/gh_token.txt) && act -vb  -W $(PWD)/.github/workflows/$@.yml
