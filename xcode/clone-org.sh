set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

repos="$(gh repo list swift-libp2p --limit 1000 --json nameWithOwner --jq '.[].nameWithOwner')"
count="$(printf '%s\n' "$repos" | sed '/^$/d' | wc -l | tr -d ' ')"

printf 'Found %s repositories\n' "$count"
printf '%s\n' "$repos" | while IFS= read -r repo; do
[ -z "$repo" ] && continue

name=${repo##*/}
if [ -d "$name/.git" ]; then
printf 'Skipping existing %s\n' "$name"
continue
fi

printf 'Cloning %s\n' "$repo"
gh repo clone "$repo" "$name"
done
