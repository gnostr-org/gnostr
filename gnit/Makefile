all: fake
	@cargo run

example-dev-mode:
	@cd examples/dev_mode && cargo run

tags: fake
	find js -name '*.js' | xargs ctags > "$@"

emojiregex: fake
	@curl -sL 'https://raw.githubusercontent.com/mathiasbynens/emoji-test-regex-pattern/main/dist/latest/javascript.txt'

.PHONY: fake
