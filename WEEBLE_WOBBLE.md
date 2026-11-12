# Gnostr semantic versioning

`gnostr` uses descending semantic versioning for internal workspace and publish flows.

When versioning path dependencies, use a `<=` requirement against the current workspace version derived from `gnostr --weeble`, for example:

```toml
gnostr-chat = { path = "chat", version = "<=1875.947912.589750" }
```

The update scripts mirror this rule so generated manifests stay publishable while still matching the current workspace release line.

## WEEBLE WOBBLE timestamping

`gnostr-blockhash` and `gnostr-blockheight` are timestamping components in the WEEBLE WOBBLE method.

They provide the block-based inputs used to anchor generated identifiers and release metadata.
