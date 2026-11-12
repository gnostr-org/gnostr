include mk/header.mk
IPFSWATCH_BIN_$(d) := $(call go-curr-pkg-tgt)

CLEAN += $(IPFSWATCH_BIN_$(d))

# no version ldflags: ipfswatch does not consume build-time version info

$(IPFSWATCH_BIN_$(d)): $(d) $$(DEPS_GO) ALWAYS
	$(go-build-relative)

ipfswatch: $(IPFSWATCH_BIN_$(d))
.PHONY: ipfswatch

$(d)-install: $(d) $$(DEPS_GO) ALWAYS
	$(GOCC) install $(go-flags-with-tags) ./cmd/ipfswatch
.PHONY: $(d)-install

include mk/footer.mk