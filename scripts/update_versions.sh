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

    DEP_NAME="$dep_name" DEP_VERSION="$version" perl -0pi -e '
        my $ver = $ENV{DEP_VERSION};
        s/^(\Q$ENV{DEP_NAME}\E\s*=\s*\{.*?\bversion\s*=\s*")[^"]*(".*?\})/${1}${ver}${2}/msg;
    ' "$manifest"
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
done < <(find . -type f -name "Cargo.toml" ! -path "*/target/*" ! -path "*/vendor/*" | sort)

echo "Package versions synchronized."

while read -r manifest; do
    echo "Checking local dependencies in $manifest..."

    while IFS=$'\t' read -r dep_name dep_path; do
        [ -z "$dep_name" ] && continue

        dep_manifest="$(cd "$(dirname "$manifest")" && cd "$dep_path" && pwd)/Cargo.toml"
        if [ ! -f "$dep_manifest" ]; then
            echo "    Warning: Dependency Cargo.toml not found at $dep_manifest for $dep_name."
            continue
        fi

        dep_version="$(manifest_version "$dep_manifest" "$WORKSPACE_VERSION")"
        if [ -z "$dep_version" ]; then
            echo "    Warning: Could not determine version for $dep_name from $dep_manifest."
            continue
        fi

        sync_dependency_version "$manifest" "$dep_name" "$dep_version"
        echo "    Synchronized $dep_name in $manifest to $dep_version"
    done < <(
        perl -0ne '
            while (/^([A-Za-z0-9_-]+)\s*=\s*\{(.*?)\}\s*$/msg) {
                my ($name, $body) = ($1, $2);
                if ($body =~ /\bpath\s*=\s*"([^"]+)"/) {
                    print "$name\t$1\n";
                }
            }
        ' "$manifest"
    )

    taplo format "$manifest"
done < <(find . -type f -name "Cargo.toml" ! -path "*/target/*" ! -path "*/vendor/*" | sort)

echo "Local path dependency versions synchronized."

if [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
    echo "Error: CARGO_REGISTRY_TOKEN is not set."
    echo "Please set the CARGO_REGISTRY_TOKEN environment variable before running this script."
    echo "You can get one from https://crates.io/settings/tokens"
    exit 1
fi

export CARGO_REGISTRY_TOKEN

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
    find . -type f -name "Cargo.toml" \
        ! -path "./target/*" \
        ! -path "*/target/*" \
        ! -path "*/vendor/*" \
        -print0 | sort -z
)

git add -- "${manifest_paths[@]}"


#TAG=$(gnostr git tag create)
git reset --soft HEAD~1
gnostr legit -m v$WORKSPACE_VERSION

for crate in "${PUBLISH_CRATES[@]}"; do
    sleep 1 && pushd "$crate" >/dev/null && cargo publish -j8 || true && popd >/dev/null
done

cargo publish -j8 --no-verify
