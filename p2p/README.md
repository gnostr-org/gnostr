## Attestation syndication

`p2p` mirrors the `asyncgit` attestation structure: deterministic fixture keys, chronological commit/event/note ordering, and `notes_ref` chaining for public attestations.

The p2p tests load crawler relay buckets and publish attestation events through the relay bridge so the syndication path stays wired to the crawler-discovered relay list.
