
DEPTH=${1:-50}

gh run list --status in_progress --limit $DEPTH
for id in $(gh run list --status in_progress --limit $DEPTH --jq ".[] | .databaseId" --json databaseId,status); do echo $id; gh run cancel $id; done
gh run list --status queued --limit $DEPTH
for id in $(gh run list --status queued --limit $DEPTH --jq ".[] | .databaseId" --json databaseId,status); do echo $id; gh run cancel $id; done
gh run list --status waiting --limit $DEPTH
for id in $(gh run list --status waiting --limit $DEPTH --jq ".[] | .databaseId" --json databaseId,status); do echo $id; gh run cancel $id; done
