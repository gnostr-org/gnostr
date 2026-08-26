
DEPTH=${1:-50}

cancel_status() {
  local status="$1"
  local id

  while IFS= read -r id; do
    [ -z "$id" ] && continue
    echo "$id"
    gh run cancel "$id"
  done < <(gh run list --status "$status" --limit "$DEPTH" --json createdAt,databaseId,status --jq 'sort_by(.createdAt, .databaseId) | .[] | .databaseId')
}

cancel_status in_progress
cancel_status queued
cancel_status waiting
