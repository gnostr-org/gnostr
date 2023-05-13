act-help:### 	
	@awk 'BEGIN {FS = ":.*?####"} /^[a-zA-Z_-]+:.*?####/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
automate:####	run act -vr
	@export $(cat ~/GH_TOKEN.txt) && act -vbr -W .github/workflows/$@.yml && popd
static:#### 	run act -vr
	@export $(cat ~/GH_TOKEN.txt) && act -vbr -W .github/workflows/$@.yml && popd