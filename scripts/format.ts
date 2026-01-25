#!/usr/bin/env bun

import { $ } from "bun"
import { unlink } from "node:fs/promises"

const ignorePatterns = [
  "./gnit/src/*/",
  "**/debug/**",
  "target/**",
  "**/target/**",
  ".gnostr/**",
  "**/coverage/coverage/**",
  "**/.fingerprint/**",
].join("\n")

// Write a temp ignore file
await Bun.write(".tmp-prettierignore", ignorePatterns)

try {
  await $`bun run prettier --ignore-path .tmp-prettierignore --write .`
} finally {
  // Clean up
  await unlink(".tmp-prettierignore")
}

// Format Rust files with cargo fmt
const rustProjects = ["packages/desktop/src-tauri"]

for (const project of rustProjects) {
  const cargoPath = `${project}/Cargo.toml`

  try {
    await $`test -f ${cargoPath}`
    await $`cargo fmt --manifest-path ${cargoPath}`
    console.log(`Formatted Rust files in ${project}`)
  } catch (error) {
    console.log(`No Rust project found at ${project}, skipping...`)
  }
}
