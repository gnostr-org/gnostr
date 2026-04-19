#!/usr/bin/env bash
set -euo pipefail

ensure_taplo_installed() {
    if ! command -v taplo >/dev/null 2>&1; then
        echo "taplo-cli not found. Installing it..."
        cargo install taplo-cli
    fi
}

workspace_version() {
    perl -0ne 'print "$1\n" and exit if /\[workspace\.package\].*?^version\s*=\s*"([^"]+)"/ms' Cargo.toml
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

managed_manifests() {
    cargo metadata --no-deps --format-version 1 | python3 -c '
import json
import os
import sys

root = os.path.abspath(".")
data = json.load(sys.stdin)
paths = sorted({
    pkg["manifest_path"]
    for pkg in data["packages"]
    if os.path.abspath(pkg["manifest_path"]).startswith(root + os.sep) or os.path.abspath(pkg["manifest_path"]) == root
})
for path in paths:
    print(path)
'
}

versioned_path_dependencies() {
    local manifest="$1"
    perl -ne '
        our ($name, $body, $capture);

        sub emit_dep {
            my ($dep_name, $dep_body) = @_;
            return unless defined $dep_name && length $dep_name;
            my ($path) = $dep_body =~ /\bpath\s*=\s*"([^"]+)"/;
            my ($version) = $dep_body =~ /\bversion\s*=\s*"([^"]+)"/;
            if (defined $path && defined $version) {
                print "$dep_name\t$path\n";
            }
        }

        if (!$capture) {
            if (/^([A-Za-z0-9_-]+)\s*=\s*\{/) {
                $name = $1;
                $body = $_;
                if (/\}\s*$/) {
                    emit_dep($name, $body);
                    $name = undef;
                    $body = q{};
                } else {
                    $capture = 1;
                }
            }
        } else {
            $body .= $_;
            if (/\}\s*$/) {
                emit_dep($name, $body);
                $name = undef;
                $body = q{};
                $capture = 0;
            }
        }
    ' "$manifest"
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
        lambda m: f"{m.group(1)}{version}{m.group(2)}",
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

ensure_taplo_installed

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

echo "Workspace version found: $WORKSPACE_VERSION"
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

    while IFS=$'\t' read -r dep_name dep_path; do
        [ -z "$dep_name" ] && continue

        if ! dep_manifest="$(resolve_dep_manifest "$manifest" "$dep_path")"; then
            echo "    Warning: Dependency Cargo.toml not found for $dep_name (path: $dep_path)."
            continue
        fi

        dep_version="$(manifest_version "$dep_manifest" "$WORKSPACE_VERSION")"
        if [ -z "$dep_version" ]; then
            echo "    Warning: Could not determine version for $dep_name from $dep_manifest."
            continue
        fi

        sync_dependency_version "$manifest" "$dep_name" "$dep_version"
        echo "    Synchronized $dep_name in $manifest to $dep_version"
    done < <(versioned_path_dependencies "$manifest")

    taplo format "$manifest"
done < <(managed_manifests)

echo "Local path dependency versions synchronized."

SORT_CRATES=(
    git2-hooks
    grammar
    filetreelist
    scopetime
    asyncgit
    tui
    crawler
    git-helpers
    invalidstring
    legit
    ngit
    qr
    relay
    relay/extensions
)

for crate in "${SORT_CRATES[@]}"; do
    sleep 1 && pushd "$crate" >/dev/null && cargo sort || true && popd >/dev/null
done

PUBLISH_CRATES=(
    invalidstring
    git2-hooks
    grammar
    filetreelist
    scopetime
    asyncgit
    tui
    crawler
    git-helpers
    legit
    ngit
    qr
    relay
    relay/extensions
)

manifest_paths=()
while IFS= read -r -d '' manifest_path; do
    manifest_paths+=("$manifest_path")
done < <(
    while read -r manifest_path; do
        printf '%s\0' "$manifest_path"
    done < <(managed_manifests)
)

git add -- "${manifest_paths[@]}"
git add Cargo.toml */Cargo.toml */*/Cargo.toml */*/*/Cargo.toml 2>/dev/null || true

if [ -n "${VERSION_TAG:-}" ]; then
    git reset --soft HEAD~1
    gnostr legit -m "$VERSION_TAG"
elif [ "${SKIP_VERSION_COMMIT:-0}" != "1" ]; then
    gnostr legit -m "v$WORKSPACE_VERSION" --prefix 000000
fi

if [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
    echo "Error: CARGO_REGISTRY_TOKEN is not set."
    echo "Please set the CARGO_REGISTRY_TOKEN environment variable before running this script."
    echo "You can get one from https://crates.io/settings/tokens"
    exit 1
fi

export CARGO_REGISTRY_TOKEN

for crate in "${PUBLISH_CRATES[@]}"; do
    sleep 1 && pushd "$crate" >/dev/null && cargo publish -j8 || true && popd >/dev/null
done

cargo publish -j8 --no-verify
