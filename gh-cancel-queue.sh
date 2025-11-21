#for id in $(gh run list --limit 50 --jq ".[] | select (.status == \"queued\" ) | .databaseId" --json databaseId,status); do echo $id; gh run cancel $id; done
gh run list --status in_progress
for id in $(gh run list --status in_progress --limit 500 --jq ".[] | .databaseId" --json databaseId,status); do echo $id; gh run cancel $id; done
gh run list --status queued
for id in $(gh run list --status queued --limit 500 --jq ".[] | .databaseId" --json databaseId,status); do echo $id; gh run cancel $id; done
gh run list --status waiting
for id in $(gh run list --status waiting --limit 500 --jq ".[] | .databaseId" --json databaseId,status); do echo $id; gh run cancel $id; done
