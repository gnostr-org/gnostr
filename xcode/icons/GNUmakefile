.PHONY: docs
docs:
	@mkdir -p docs/icons docs/banner
	@./scripts/render-sizes.sh square ./1024x1024.png ./docs/icons
	@./scripts/render-sizes.sh banner ./banner/icon3072x1024.png ./docs/banner
	@./scripts/make-icns.sh
	@rm -f docs/icons/make-sizes.sh docs/banner/make-sizes.sh
	@sed 's/images/.\/images/' README.md > docs/index.md
