#!/usr/bin/env bash
set -euo pipefail

BASH_VERSION_CURRENT="${BASH_VERSION:-unknown}"
BASH_MAJOR="${BASH_VERSINFO[0]:-0}"
BASH_MINOR="${BASH_VERSINFO[1]:-0}"
if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

DRY_RUN=false
POSITIONAL_ARGS=()
while (($#)); do
    case "$1" in
        -n|--dry-run)
            DRY_RUN=true
            ;;
        --dry-run)
            DRY_RUN=true
            ;;
        *)
            POSITIONAL_ARGS+=("$1")
            ;;
    esac
    shift
done

ensure_taplo_installed() {
    if ! command -v taplo >/dev/null 2>&1; then
        echo "taplo-cli not found. Installing it..."
        cargo install taplo-cli
    fi
}

workspace_version() {
    perl -0ne 'print "$1\n" and exit if /\[workspace\.package\].*?^version\s*=\s*"([^"]+)"/ms' Cargo.toml
}

blockheight_dependency_version() {
    local blockheight
    # gnostr semver: blockheight = major, weeble = minor, wobble = patch.
    # This helper feeds the version bump used for path dependency rewriting.

    if blockheight="$(gnostr --blockheight 2>/dev/null)"; then
        printf '^%s.0\n' "$blockheight"
        return
    fi

    printf '^%s.0\n' "${WORKSPACE_VERSION%%.*}"
}

previous_blockheight_dependency_version() {
    local blockheight
    local previous

    if blockheight="$(gnostr --blockheight 2>/dev/null)"; then
        previous=$((blockheight - 1))
        if [ "$previous" -lt 1 ]; then
            previous=1
        fi
        printf '^%s.0\n' "$previous"
        return
    fi

    printf '^%s.0\n' "${WORKSPACE_VERSION%%.*}"
}

version_requirement_for_dependency() {
    local manifest="$1"
    local dep_name="$2"

    case "$manifest:$dep_name" in
        "$REPO_ROOT/Cargo.toml:gnostr-asyncgit"|\
        "$REPO_ROOT/Cargo.toml:gnostr-crawler")
            printf '>=%s\n' "$WORKSPACE_VERSION"
            return 0
            ;;
        "$REPO_ROOT/asyncgit/Cargo.toml:gnostr-crawler"|\
        "$REPO_ROOT/crawler/Cargo.toml:gnostr-asyncgit"|\
        "$REPO_ROOT/js/Cargo.toml:gnostr-asyncgit"|\
        "$REPO_ROOT/js/Cargo.toml:gnostr-crawler"|\
        "$REPO_ROOT/p2p/Cargo.toml:gnostr-asyncgit"|\
        "$REPO_ROOT/chat/Cargo.toml:gnostr-asyncgit"|\
        "$REPO_ROOT/chat/Cargo.toml:gnostr-crawler")
            previous_blockheight_dependency_version
            return 0
            ;;
    esac

    return 1
}

manifest_version() {
    local manifest="$1"
    local fallback="$2"

    if grep -q '^version\.workspace = true' "$manifest"; then
        printf '%s\n' "$fallback"
        return
    fi

    grep '^version =' "$manifest" | head -1 | awk -F'"' '{print $2}'
}

manifest_package_name() {
    local manifest="$1"

    perl -0ne 'print "$1\n" and exit if /\[package\].*?^name\s*=\s*"([^"]+)"/ms' "$manifest"
}

cargo_paths() {
    while IFS= read -r -d '' path; do
        case "$path" in
            ./vendor/*|vendor/*) continue ;;
        esac
        printf '%s\0' "$path"
    done < <(
        find . -path './vendor' -prune -o \( -name Cargo.lock -o -name Cargo.toml \) -print0
    )
}

stage_cargo_files() {
    local paths=()
    while IFS= read -r -d '' path; do
        paths+=("$path")
    done < <(cargo_paths)

    if [ "${#paths[@]}" -gt 0 ]; then
        git add -- "${paths[@]}"
    fi
}

managed_manifests() {
    python3 - <<'PY'
import json
import os
import subprocess

root = os.path.abspath(".")
data = json.loads(subprocess.check_output(
    ["cargo", "metadata", "--no-deps", "--format-version", "1"],
    text=True,
))
paths = {
    pkg["manifest_path"]
    for pkg in data["packages"]
    if os.path.abspath(pkg["manifest_path"]).startswith(root + os.sep)
    or os.path.abspath(pkg["manifest_path"]) == root
}

paths = {
    path for path in paths
    if "/vendor/" not in os.path.abspath(path).replace("\\", "/")
}

for rel_path in ["crawler/Cargo.toml", "asyncgit/src/lib/filehash/core/Cargo.toml"]:
    manifest_path = os.path.abspath(rel_path)
    if os.path.isfile(manifest_path) and "/vendor/" not in manifest_path.replace("\\", "/"):
        paths.add(manifest_path)

for path in sorted(paths):
    print(path)
PY
}

versioned_path_dependencies() {
    local manifest="$1"
    python3 - "$manifest" <<'PY'
import pathlib
import re
import sys

manifest = pathlib.Path(sys.argv[1])
section = None
name = None
body = []
capturing = False

def emit(current_section, dep_name, dep_body):
    if not current_section or not dep_name:
        return

    path_match = re.search(r'\bpath\s*=\s*"([^"]+)"', dep_body)
    version_match = re.search(r'\bversion\s*=\s*"([^"]+)"', dep_body)
    if path_match and version_match:
        print(f"{current_section}\t{dep_name}\t{path_match.group(1)}")

for line in manifest.read_text().splitlines():
    stripped = line.strip()
    section_match = re.match(r'^\[([^\]]+)\]$', stripped)
    if section_match and not capturing:
        section = section_match.group(1)
        continue

    if capturing:
        body.append(line)
        if re.search(r'\}\s*$', line):
            emit(section, name, "\n".join(body))
            capturing = False
            name = None
            body = []
        continue

    dep_match = re.match(r'^([A-Za-z0-9_-]+)\s*=\s*\{', line)
    if dep_match:
        name = dep_match.group(1)
        body = [line]
        if re.search(r'\}\s*$', line):
            emit(section, name, line)
            name = None
            body = []
        else:
            capturing = True
PY
}

resolve_dep_manifest() {
    local manifest="$1"
    local dep_path="$2"
    local manifest_dir
    local candidate

    manifest_dir="$(cd "$(dirname "$manifest")" && pwd)"
    candidate="$manifest_dir/$dep_path/Cargo.toml"
    if [ -f "$candidate" ]; then
        printf '%s\n' "$candidate"
        return 0
    fi

    candidate="$(pwd)/$dep_path/Cargo.toml"
    if [ -f "$candidate" ]; then
        printf '%s\n' "$candidate"
        return 0
    fi

    return 1
}

sync_root_package_to_workspace() {
    perl -0pi -e '
        s/(\[package\]\n(?:[^\[]*\n)*?)version\s*=\s*"[^"]+"/$1 . "version.workspace = true"/se
    ' Cargo.toml
}

sync_package_version() {
    local manifest="$1"
    local version="$2"

    if grep -q '^version\.workspace = true' "$manifest"; then
        return
    fi

    perl -0pi -e '
        my $v = $ENV{SYNC_VERSION};
        s/(\[package\]\n(?:[^\[]*\n)*?)version\s*=\s*"[^"]+"/$1 . qq(version = "$v")/se
    ' "$manifest"
}

sync_dependency_version() {
    local manifest="$1"
    local dep_name="$2"
    local version="$3"

    python3 - "$manifest" "$dep_name" "$version" <<'PY'
import pathlib
import re
import sys

manifest, dep_name, version = sys.argv[1:]
path = pathlib.Path(manifest)
text = path.read_text()
pattern = re.compile(rf'(?m)^{re.escape(dep_name)}\s*=\s*\{{')
replacement_version = version if version.startswith(("^", ">", "=", "~")) else f">={version}"

def find_block_end(source: str, brace_start: int) -> int:
    depth = 0
    in_string = False
    escape = False
    for idx in range(brace_start, len(source)):
        ch = source[idx]
        if in_string:
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == '"':
                in_string = False
            continue

        if ch == '"':
            in_string = True
        elif ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return idx + 1
    return -1

offset = 0
changed = False
while True:
    match = pattern.search(text, offset)
    if not match:
        break

    brace_start = text.find("{", match.start())
    block_end = find_block_end(text, brace_start)
    if block_end == -1:
        break

    block = text[match.start():block_end]
    if "path" not in block or "version" not in block:
        offset = block_end
        continue

    updated_block, count = re.subn(
        r'(\bversion\s*=\s*")[^"]*(")',
        lambda m: f"{m.group(1)}{replacement_version}{m.group(2)}",
        block,
        count=1,
    )
    if count:
        text = text[:match.start()] + updated_block + text[block_end:]
        offset = match.start() + len(updated_block)
        changed = True
    else:
        offset = block_end

if changed:
    path.write_text(text)
PY
}

ROOT_CARGO_TOML="./Cargo.toml"
if [ ! -f "$ROOT_CARGO_TOML" ]; then
    echo "Error: $ROOT_CARGO_TOML not found."
    exit 1
fi

WORKSPACE_VERSION="$(workspace_version)"
if [ -z "$WORKSPACE_VERSION" ]; then
    echo "Error: Could not find [workspace.package].version in $ROOT_CARGO_TOML."
    exit 1
fi

REPO_ROOT="$(pwd)"

echo "Workspace version found: $WORKSPACE_VERSION"

if [ "$DRY_RUN" = true ]; then
    echo "Dry run: would synchronize workspace package versions and local path dependency versions."
    while read -r manifest; do
        echo "  would update package versions in $manifest"
        while IFS=$'\t' read -r section dep_name dep_path; do
            [ -z "$dep_name" ] && continue
            if dep_version="$(version_requirement_for_dependency "$manifest" "$dep_name")"; then
                echo "    would sync $dep_name (blockheight version) -> $dep_version"
            elif dep_manifest="$(resolve_dep_manifest "$manifest" "$dep_path")"; then
                dep_version="$(manifest_version "$dep_manifest" "$WORKSPACE_VERSION")"
                dep_package="$(manifest_package_name "$dep_manifest")"
                if [ -n "$dep_package" ] && [ "$dep_package" != "$dep_name" ]; then
                    echo "    would sync $dep_name ($dep_package) -> $dep_version"
                else
                    echo "    would sync $dep_name -> $dep_version"
                fi
            else
                echo "    would warn: dependency Cargo.toml not found for $dep_name (path: $dep_path)"
            fi
        done < <(versioned_path_dependencies "$manifest")
    done < <(managed_manifests)
    echo "  would run cargo update --workspace"
    echo "  would create version commit/tag and publish crates"
    exit 0
fi

ensure_taplo_installed

sync_root_package_to_workspace

while read -r manifest; do
    taplo format "$manifest"
    if [ "$manifest" != "./Cargo.toml" ]; then
        SYNC_VERSION="$WORKSPACE_VERSION" sync_package_version "$manifest" "$WORKSPACE_VERSION"
    fi
done < <(managed_manifests)

echo "Package versions synchronized."

    while read -r manifest; do
        echo "Checking local dependencies in $manifest..."

        while IFS=$'\t' read -r section dep_name dep_path; do
            [ -z "$dep_name" ] && continue

            if dep_version="$(version_requirement_for_dependency "$manifest" "$dep_name")"; then
                dep_package="blockheight version requirement"
            else
                if ! dep_manifest="$(resolve_dep_manifest "$manifest" "$dep_path")"; then
                    echo "    Warning: Dependency Cargo.toml not found for $dep_name (path: $dep_path)."
                    continue
                fi

                dep_version="$(manifest_version "$dep_manifest" "$WORKSPACE_VERSION")"
                dep_package="$(manifest_package_name "$dep_manifest")"
            fi

            if [ -z "$dep_version" ]; then
                echo "    Warning: Could not determine version for $dep_name from $dep_manifest."
                continue
            fi

            sync_dependency_version "$manifest" "$dep_name" "$dep_version"
            if [ "$dep_package" = "blockheight version requirement" ]; then
                echo "    Synchronized $dep_name in $manifest to $dep_version"
            elif [ -n "$dep_package" ] && [ "$dep_package" != "$dep_name" ]; then
                echo "    Synchronized $dep_name ($dep_package) in $manifest to $dep_version"
            else
                echo "    Synchronized $dep_name in $manifest to $dep_version"
            fi
    done < <(versioned_path_dependencies "$manifest")

    taplo format "$manifest"
done < <(managed_manifests)

echo "Local path dependency versions synchronized."

SORT_CRATES=(
    git2-hooks
    grammar
    filetreelist
    asyncgit/src/lib/filehash/core
    scopetime
    asyncgit
    crawler
    tui
    git-helpers
    invalidstring
    legit
    ngit
    qr
    relay
    relay/extensions
    js
    p2p
    chat
    web
)

for crate in "${SORT_CRATES[@]}"; do
    sleep 1 && pushd "$crate" >/dev/null && cargo sort || true && popd >/dev/null
done

PUBLISH_CRATES=(
    invalidstring
    git2-hooks
    grammar
    filetreelist
    asyncgit/src/lib/filehash/core
    scopetime
    crawler
    asyncgit
    tui
    git-helpers
    legit
    ngit
    qr
    relay
    relay/extensions
    js
    p2p
    chat
    web
)

PUBLISH_NO_VERIFY_CRATES=(
    asyncgit
    crawler
    js
    p2p
    chat
    web
)

should_skip_verify() {
    local crate="$1"
    local candidate

    for candidate in "${PUBLISH_NO_VERIFY_CRATES[@]}"; do
        if [ "$candidate" = "$crate" ]; then
            return 0
        fi
    done

    return 1
}

tag_package_versions() {
    local version="$1"
    local crate
    local tag
    local tree
    local commit

    tree="$(git rev-parse HEAD^{tree})"
    for crate in "${PUBLISH_CRATES[@]}"; do
        tag="$crate/v$version"
        commit="$(printf '%s\n' "$tag" | git commit-tree "$tree" -p HEAD)"
        git tag -f "$tag" "$commit"
    done
}

manifest_paths=()
while IFS= read -r -d '' manifest_path; do
    manifest_paths+=("$manifest_path")
done < <(
    while read -r manifest_path; do
        printf '%s\0' "$manifest_path"
    done < <(managed_manifests)
)

git add -- "${manifest_paths[@]}"
stage_cargo_files

if [ -n "${VERSION_TAG:-}" ]; then
    cargo update --workspace
    stage_cargo_files

    git checkout -b "release/$VERSION_TAG"

    gnostr legit -m "$VERSION_TAG"
    git tag -f "$VERSION_TAG" HEAD
    tag_package_versions "$WORKSPACE_VERSION"
elif [ "${SKIP_VERSION_COMMIT:-0}" != "1" ]; then

    cargo update --workspace
    stage_cargo_files



    gnostr legit -m "v$WORKSPACE_VERSION" --prefix 000000
    tag_package_versions "$WORKSPACE_VERSION"
fi

if [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
    echo "Error: CARGO_REGISTRY_TOKEN is not set."
    echo "Please set the CARGO_REGISTRY_TOKEN environment variable before running this script."
    echo "You can get one from https://crates.io/settings/tokens"
    exit 1
fi

export CARGO_REGISTRY_TOKEN

for crate in "${PUBLISH_CRATES[@]}"; do
    if should_skip_verify "$crate"; then
        sleep 1 && pushd "$crate" >/dev/null && cargo publish -j8 --no-verify || true && popd >/dev/null
    else
        sleep 1 && pushd "$crate" >/dev/null && cargo publish -j8 || true && popd >/dev/null
    fi
done

if [ -n "$(git status --porcelain -- . ':(exclude)vendor/**' 2>/dev/null | grep -E '(^|/)(Cargo\.toml|Cargo\.lock)$' || true)" ]; then
    echo "Warning: Cargo manifests changed during publish; leaving tagged commits as-is."
fi

if [ -n "${VERSION_TAG:-}" ]; then
  git push origin "$VERSION_TAG:$VERSION_TAG"
fi
for crate in "${PUBLISH_CRATES[@]}"; do
    git push origin "$crate/v$WORKSPACE_VERSION:$crate/v$WORKSPACE_VERSION"
done
