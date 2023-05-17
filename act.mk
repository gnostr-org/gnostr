1:=cargo-check
2:=cargo-build
act-help:### 	
	@awk 'BEGIN {FS = ":.*?####"} /^[a-zA-Z_-]+:.*?####/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
.PHONY:github-workspace cargo-build cargo-check
automate:####	run act -vr
	@echo $@ $1 $2 $3
	@export $(cat ~/GH_TOKEN.txt) && act $(JOB) -v $(BIND) $(REUSE) \
	-W .github/workflows/$@.yml
static:#### 	run act -vr
	@export $(cat ~/GH_TOKEN.txt) && act -v $(BIND) $(REUSE) \
	-W .github/workflows/$@.yml