commit:touch-time
	@echo $(GIT_STATUS)
	$(MAKE) git-add
	@test legit && legit . -p 000 -m "$(shell date +%s)" -m "$(GIT_STATUS)"