#!/usr/bin/env bash
set -euo pipefail

toml_files=()
while IFS= read -r file; do
  toml_files+=("$file")
done < <(
  find . -maxdepth 3 -type f -name '*.toml' -not -path '*/vendor/*' | sort
)

if ((${#toml_files[@]} == 0)); then
  echo "No TOML files found"
  exit 0
fi

taplo check "${toml_files[@]}"
