EVENT=$(gnostr --nsec "$(gnostr --hash '$GITHUB_REPOSITORY_ID')" \
    -d 16 \
    -r wss://relay.damus.io \
    -r wss://nos.lol \
    custom-event \
    -k 1 \
    -c "[$GITHUB_REPOSITORY_ID]() (windows-latest, nightly) Failed!" \
    -t "$GITHUB_REPOSITORY" \
    -t "$GITHUB_WORKFLOW" \
    -t "$GITHUB_WORKFLOW_SHA" \
    -t "$GITHUB_WORKFLOW_REF" \
    -t "$GITHUB_REF" \
    -t "$GITHUB_RUN_ID" \
    -t "gnostr" \
    -t "gnostr-bot" \
    -t "$GITHUB_REPOSITORY_OWNER" \
    -t "$(date +%s)" \
    -t "$RUNNER_ENVIRONMENT" \
    -t "$RUNNER_OS" \
    -t "$(gnostr --weeble)" \
    -t "$(gnostr --blockheight)" \
    -t "$(gnostr --wobble)" \
    --hex || true) && \
    export EVENT=$EVENT && \
    NOTE=$(gnostr convert-key -p note -k $EVENT || true) || true && \
    export NOTE=$NOTE;
    gnostr query -i $EVENT -r wss://relay.damus.io;
    echo && echo "http://nostr.band/$NOTE"
