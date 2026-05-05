-:docs
	. make-icns.sh
.PHONY: docs
docs:
	@mkdir -p docs
	@cd icons && ./make-sizes.sh && cd ..
	@./make-icns.sh
	@cp -R ./icons ./docs/
	@cd banner && ./make-sizes.sh && cd ..
	@cp -R ./banner ./docs/
	@install -v README.md docs/index.md
	@sed 's/images/.\/images/' README.md > docs/index.md
