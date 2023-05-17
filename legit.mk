legit-help:### 	
	@awk 'BEGIN {FS = ":.*?####"} /^[a-zA-Z_-]+:.*?####/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
.PHONY: legit
.ONESHELL:
legit-install:#### 	install
	@. make-legit.sh
	$(MAKE) cargo-build
	$(MAKE) cargo-install
legit-commit:touch-time####	commit
	@echo $(GIT_STATUS)
	$(MAKE) git-add
	@test legit && legit . -p 000 -m "$(shell date +%s)" -m "$(GIT_STATUS)"