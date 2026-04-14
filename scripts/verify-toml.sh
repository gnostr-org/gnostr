#!/usr/bin/env bash
set -euo pipefail

taplo check Cross.toml
taplo check Cargo.toml
