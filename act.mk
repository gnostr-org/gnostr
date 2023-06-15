#NOTE: using -C for container context
#The action is run on the submodule .github as an example
alpine:docker-start## 	run act in .github
	@export $(cat ~/gh_token.txt) && act -vbr  -W $(PWD)/.github/workflows/$@.yml
ubuntu-matrix:docker-start## 	run act in .github
	#cant use -r reuse - blocks apt-get install
	@export $(cat ~/gh_token.txt) && act -vbr  -W $(PWD)/.github/workflows/$@.yml
