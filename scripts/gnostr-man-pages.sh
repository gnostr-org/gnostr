for cmd in award-badge bech32-to-any broadcast-events chat convert-key create-badge create-public-channel crawler
custom-event delete-event delete-profile dm fetch-by-id generate-keypair git ngit hide-public-channel-message legit
list-events mute-public-key nip34 note privkey-to-bech32 profile-badges publish-contact-list-csv query react relay
server send-channel-message set-channel-metadata set-metadata set-user-status sniper tui vanity xor; do
   cargo run --bin gnostr -- "$cmd" --help >"/tmp/gnostr-help/$cmd.txt" 2>&1 || true
