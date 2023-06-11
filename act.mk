#NOTE: using -C for container context
#The action is run on the submodule .github as an example
alpine:docker-start## 	run act in .github
	@export $(cat ~/GH_TOKEN.txt) && act -C $(PWD) -vr -W $(PWD)/.github/workflows/$@.yml
