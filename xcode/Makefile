SHELL := /bin/sh

SCRIPT_ROOT := ./scripts
SCRIPT_FILES := $(shell find $(SCRIPT_ROOT) -type f -name '*.sh' | sort)
SCRIPT_TARGETS := $(patsubst $(SCRIPT_ROOT)/%.sh,%,$(SCRIPT_FILES))
SCRIPT_VERBOSE := $(if $(VERBOSE),--verbose,)

.DEFAULT_GOAL := help
.PHONY: help list-scripts build-libp2p-framework $(SCRIPT_TARGETS)

help:
	@printf '%s\n' 'Usage: make <script-target> [ARGS="..."] [VERBOSE=1]'
	@printf '\n%s\n' 'Available targets:'
	@for t in $(SCRIPT_TARGETS); do printf '  %-36s ./scripts/%s.sh\n' "$$t" "$$t"; done
	@printf '  %-36s %s\n' help "Show this help"
	@printf '  %-36s %s\n' list-scripts "List script targets"
	@printf '  %-36s %s\n' build-libp2p-framework "Alias for libp2p/build-libp2p-framework"

list-scripts:
	@printf '%s\n' $(SCRIPT_TARGETS)

define RUN_SCRIPT
$1:
	@script="$(SCRIPT_ROOT)/$1.sh"; \
	exec bash "$$$$script" $(ARGS) $(SCRIPT_VERBOSE)
endef

$(foreach target,$(SCRIPT_TARGETS),$(eval $(call RUN_SCRIPT,$(target))))

build-libp2p-framework: libp2p/build-libp2p-framework