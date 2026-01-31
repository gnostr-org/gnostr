#!/usr/bin/env bash

if [ -z ${REFRESH_INTERVAL+x} ];
then 
	./bin/gnostr-gnit -b 8000 -s /git -d /tmp/gnostr-gnit-cache.db;
else
	./bin/gnostr-gnit -b 8000 -s /git -d /tmp/gnostr-gnit-cache.db --refresh-interval "$REFRESH_INTERVAL";
fi
