#!/usr/bin/env bash
if hash nostril; then
MESSAGE_HASH=$(nostril --hash $1)
export $MESSAGE_HASH

fi
nostril --sec $(nostril --hash 0) --created-at $(date +%s) --envelope --content "$MESSAGE_HASH:$1"
if hash nostcat; then
nostril --sec $(nostril --hash 0) --created-at $(date +%s) --envelope --content "$MESSAGE_HASH:$1" | nostcat -u wss://relay.damus.io
else
	echo "try:\nmake nostcat"
fi

nostril --hash 0