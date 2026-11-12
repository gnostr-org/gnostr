# TODO — p2p/fractal

Findings from the review of `714634c8` (`feat(p2p/example): add --clone to
fractal_swarm_perfect_ip`), in severity order.

**Status: every gap-fill is implemented as a v2 stack, committed in
`f7d77fcb8c` (types) and `56c261fde6` (p2p).** The v1 implementation is
untouched; v2 lives beside it. This file records exactly what was built per
finding.

## 1. Empty-placeholder slice is a correctness bug — FIXED in v2

`run_fractal_engine` answers a missing slice with
`ProtocolSlice { data: Vec::new(), is_parity: false }`. `Header` carries no
length field, so a receiver cannot distinguish "peer doesn't have it" from
"a zero-length leaf" — a placeholder counted as received silently corrupts
`reconstruct_payload`.

### What v2 does (plan 1 — explicit absence)

- The v2 repair response type is `Result<ProtocolSliceV2, String>`
  (`RepairResponseV2`, p2p/src/fractal/fractal_ip_core_v2.rs). A missing id
  answers `Err("not held: <id>")`; there is no placeholder path at all.
- `HeaderV2` (types/src/utils/protocol_slice_v2.rs) adds a `len` field, so a
  packet is self-describing and header/data length consistency is checkable.
- Receiver side (`request_slices_v2`, p2p/src/fractal/fractal_ip_core_v2.rs):
  an `Err` response is collected into `RepairFetchV2.errors` and counted as
  answered-with-absence — never inserted into `IntegrityManagerV2` or any
  received map.
- Verified: `fractal_v2_p2p_missing_slice_returns_err_and_parity_recovers`
  (networked, `#[ignore]`) asserts the dropped id is absent from `received`,
  present in `errors`, then parity recovery completes the payload
  byte-exactly. `fractal_v2_p2p_sync_reconstructs_git_repo` asserts every
  received slice has `header.len == data.len` (no placeholder shapes).

## 2. Wire encoding: JSON → binary — FIXED in v2

`request_response::json::Behaviour` costs ~2×+ wire overhead plus parse
cost on every ~730-byte slice.

### What v2 does (plan 2 — postcard codec beside JSON)

- `PostcardCodec` (p2p/src/fractal/fractal_ip_core_v2.rs) implements
  `request_response::Codec` with postcard. libp2p-request-response 0.27 ships
  JSON and CBOR only, so the codec is ours — same size limits and
  read-to-end/write-all shape as the upstream JSON codec. `postcard` 1.x
  added to `gnostr-p2p` along with `async-trait` for the trait impl.
- Both behaviours are mounted on every v2 swarm, newtyped
  (`RepairBinV2`/`RepairJsonV2`) so the composite behaviour can tell their
  events apart: postcard on `/fractal/repair/1.1.0/postcard`, JSON on
  `/fractal/repair/2.0.0/json`. `WireCodecV2` selects the outbound codec;
  default is binary.
- Protocol-id note: the original plan kept v2's JSON on v1's
  `/fractal/repair/1.0.0/json`, which would let a v1 and v2 node negotiate
  the debug stream and then fail every request on the response-type mismatch
  (serde cannot parse `Result<ProtocolSliceV2, String>` into v1's
  `ProtocolSlice`). It failed rather than corrupted, but v2's JSON behaviour
  now uses its own id, `/fractal/repair/2.0.0/json`, so the two stacks never
  negotiate a shared debug codec at all. The binary protocol id is already
  v2-only (`1.1.0/postcard`).
- Measured and pinned by `postcard_wire_size_beats_json_on_real_packet_batch`:
  a real 4096-byte payload, 15 packets — JSON 36546B vs postcard 10517B,
  **71.2% smaller** on the wire (test asserts >50%).
- Round-trip verified by `postcard_codec_round_trips_requests_and_responses`
  (request, `Ok` response, `Err` response).
- The `--json` flag on the v2 example (`fractal_swarm_perfect_ip_v2`)
  selects the JSON behaviour for outbound requests; the default is binary.

## 3. Per-slice authentication — FIXED in v2

Slices were trusted by manifest membership; `verify_integrity` only caught
bad parities when siblings were present.

### What v2 does (plan 3 — manifest v2: {id, offset, len, sha256})

- `ManifestEntry { id, offset, len, sha256 }`
  (types/src/utils/protocol_slice_v2.rs); offset/len come from the same tree
  walk that emits the ids, so sha256 requires the payload — the manifest is
  generated from data, not from length alone. `generate_manifest_v2` and
  `packetize_v2` emit both batch and manifest from one walk.
- `IntegrityManagerV2::record_slice` verifies in order: id must be in the
  manifest, `header.len == data.len == entry.len`, then
  `sha256_hex(&slice.data) == entry.sha256`. Any mismatch is pushed to the
  `rejected: Vec<RejectedSlice>` list and the slice is NOT stored.
- `verify_integrity` keeps the v1 parity check (XOR of children equals the
  parity frame when both children are present) and adds the per-slice hash
  re-check, so a store that was tampered with after recording still fails.
- Reconstruction (`reconstruct_directory_v2`) routes every packet through
  `record_slice` and aborts if anything was rejected — tampered input fails
  closed before a byte is written.
- CID upgrade path: still not built (unchanged). The manifest sha256 is the
  raw-leaf multihash digest, so a `cid` field can be added later without a
  format break.
- Verified: `record_slice_rejects_bad_hash_unknown_id_and_bad_length`,
  `verify_integrity_fails_when_slice_is_tampered_after_recording`,
  `load_from_disk_rejects_tampered_persisted_state` (types),
  `fractal_v2_reconstruct_rejects_tampered_packets` (p2p integration).

## 4. One-loss-per-pair ceiling — DOCUMENTED and PINNED in v2

XOR-tree recovers exactly one missing sibling per pair; a pair losing
children *and* parity is unrecoverable.

### What v2 does (plan 4)

- Module docs (types/src/utils/protocol_slice_v2.rs header) state the
  ceiling plainly: one loss per pair; two losses in a pair = subtree
  unrecoverable; repair-assist layer, not a primary transport.
- `IntegrityManagerV2::recover_missing` walks every parity trio
  `{base.0, base.1, base.P}`: exactly one missing member is rebuilt via XOR
  and re-verified through `record_slice` (a rebuild that fails hash
  verification counts as unrecoverable, never stored); two or three missing
  members are reported in `RecoveryReport.unrecoverable`. Recovery never
  fabricates silent garbage.
- Subtlety found while implementing: a parity frame whose children are
  internal nodes (e.g. `ROOT.P` at multi-level trees) has no slice trio —
  internal children emit no slices. Recovery only attempts trios where all
  three members exist in the manifest; such parities are still
  hash-verified on record, just not used for pair repair.
- Reed-Solomon/RaptorQ: still deferred, as the plan says — manifest v2's
  `(offset, len)` keying is the seam a k-of-n scheme plugs into.
- Verified: `recover_missing_rebuilds_single_loss_byte_exactly`,
  `recover_missing_reports_unrecoverable_when_pair_loses_two` (drops a
  sibling and its parity, asserts both land in `unrecoverable` and
  reconstruction errors).

## 5. `reconstruct_payload` invariant assertion — FIXED in v2 (subsumed by 3)

v1 sorts by `seq_num` and concatenates, relying on "seq order == byte order
across the recursive tree."

### What v2 does (plan 5)

- `IntegrityManagerV2::reconstruct_payload` orders data leaves by manifest
  `offset` (tie-broken by id) and asserts the assembled length equals the
  sum of the data-leaf lengths (`ReconstructErrorV2::LengthMismatch`); it
  never reads `seq_num`. Missing leaves fail with
  `ReconstructErrorV2::MissingSlices` instead of silently producing a short
  payload.
- Verified: `reconstruct_orders_by_offset_regardless_of_arrival_order`
  (records packets in reverse emission order; offsets still win),
  `reconstruct_fails_closed_with_missing_slices`,
  `packetize_v2_manifest_matches_tree_and_offsets_are_ordered` (data leaves
  are contiguous and sum to the payload length).

## 6. `--clone` verification: add `git fsck --strict` — FIXED in v2

The v1 check (clone → HEAD match → clean tree) proves the reconstruction is
a valid repo but can tolerate object-store corruption.

### What v2 does (plan 6)

- `git_fsck_strict` (p2p/src/fractal/fractal_ip_core_v2.rs) runs
  `git fsck --strict` on the reconstructed directory and surfaces fsck's
  stdout/stderr on failure.
- `fsck_clone_and_verify_reconstructed_repo`
  (p2p/src/fractal/fractal_ip_v2.rs) runs fsck first, then the existing
  clone → HEAD-match → clean-tree flow.
- Verified: `fractal_swarm_perfect_ip_v2_reconstructs_git_repo` (integration)
  runs fsck on the reconstructed `web/crlf.git` fixture before cloning; the
  networked sync test does the same on wire-received data. Example binary
  verified end-to-end against the fixture.

## 7. Persist format versioning — FIXED in v2

`IntegrityManager::persist` format was unversioned.

### What v2 does (plan 7)

- `IntegrityManagerV2::persist` writes JSON
  `{"v": 1, "manifest": [...], "received_slices": {...}}` (v1 used bincode;
  v2 uses JSON so the envelope is inspectable and the manifest travels with
  the state).
- `load_from_disk` refuses any `v` it doesn't know with a clear
  `InvalidData` error naming both versions, and routes every loaded slice
  back through `record_slice`, so tampered persisted state lands in
  `rejected` instead of being trusted. Unlike v1, the manifest is embedded
  in the file — loaders don't need it passed in.
- Verified: `persist_round_trips_and_marks_version` (asserts `"v": 1` in the
  file), `load_from_disk_refuses_unknown_version` (v: 99 rejected),
  `load_from_disk_rejects_tampered_persisted_state`.

## v2 stack map (for orientation)

- Data plane: `types/src/utils/protocol_slice_v2.rs` —
  `HeaderV2`, `ProtocolSliceV2`, `PacketBatchV2`, `ManifestEntry`,
  `RejectedSlice`, `RecoveryReport`, `ReconstructErrorV2`,
  `IntegrityManagerV2`, `generate_manifest_v2`, `packetize_v2`,
  `DirectoryWalkV2`, `run_directory_walk_v2`, `reconstruct_directory_v2`.
  Re-exported as `gnostr_types::perfect_ip_v2` and from the `gnostr-p2p`
  crate root.
- Wire plane: `p2p/src/fractal/fractal_ip_core_v2.rs` —
  `PostcardCodec`, `RepairBinV2`, `RepairJsonV2`, `FractalBehaviourV2`,
  `WireCodecV2`, `RepairResponseV2`, `build_fractal_swarm_v2`,
  `run_fractal_engine_v2`, `request_slices_v2`, `git_fsck_strict`.
- CLI/example: `p2p/src/fractal/fractal_ip_v2.rs` (`run_v2`) +
  `p2p/examples/fractal_swarm_perfect_ip_v2.rs`. Flags: `--clone`,
  `--recursive`, `--depth`, `--logging`, `--json` (text wire protocol),
  `--fetch <MULTIADDR>` (repair-by-id demo: manifest from a local walk,
  payload fetched slice-by-slice over the selected codec).
- Nostr carrier: `types/src/utils/protocol_slice_nostr.rs` —
  `slice_event_v1`/`slice_event_v2` wrap a slice as a signed kind-9261
  regular event (content = the exact pinned slice JSON; tags `["d", root,
  slice]`, `["perfect_ip", "1"|"2"]`, `["t","perfect_ip"]`);
  `manifest_event_v1`/`manifest_event_v2` publish the manifest the same way
  (`["d", root, "manifest"]`) so a receiver needs only the root id;
  `event_to_slice_v1`/`event_to_slice_v2` validate the envelope strictly —
  the version tag is checked before content is parsed, so the nostr plane
  is strict in both directions where raw serde silently drops v2's `len`.
  Kind 9261 is a regular kind on purpose: the vendored gnostr-relay keys
  replaceable storage by `(pubkey, kind)` only, so a 30xxx kind would have
  every slice after the first rejected `replaced: have newer event`.
  This is the perfect_ip-over-nostr transport: the packet tree becomes a
  relay-carryable event set, verified on record by `IntegrityManagerV2`.
  End-to-end test: `slice_events_round_trip_through_live_relay` spins a
  loopback gnostr-relay (isolated temp data dir), publishes manifest +
  slices with confirmed sends, fetches by `{"kinds":[9261], "#t":
  ["perfect_ip"]}`, and reconstructs the payload byte-exact from zero prior
  knowledge beyond the root id.
- Tests: unit tests in both modules; integration in
  `p2p/tests/fractal_clone_v2.rs`; networked `#[ignore]` tests in
  `fractal_ip_core_v2.rs` (full sync + missing-slice/repair); byte-level
  v1↔v2 wire compat matrix in `p2p/tests/fractal_wire_compat.rs` (golden
  JSON/postcard frames both directions, including the proof that v1's
  empty-placeholder bytes are unparseable by v2); networked v1↔v2
  round-trip matrix in `p2p/tests/fractal_wire_compat_networked.rs`
  (real-swarm negotiation refusals, the shared-id failure modes that
  motivated the protocol-id split, and a v2↔v2 both-codecs control).

## 8. Slices as raw-leaf CIDs — FIXED in v2

Trustless serving + gateway backfill: any slice must be fetchable from an
IPFS gateway by CID and verifiable without trusting the gateway.

### What v2 does

- `raw_leaf_cid_from_sha256` / `ManifestEntry::cid` /
  `verify_slice_against_cid`
  (types/src/utils/protocol_slice_v2.rs) derive a `CIDv1(raw, sha2-256)`
  — the exact `ipfs add --raw-leaves` shape — from the manifest sha256,
  which already IS the raw multihash digest. Hand-rolled base32/varint
  (all varints single-byte), no new dependencies, and no manifest format
  change: the CID is computed on demand instead of stored, so persisted
  state and the nostr carrier are untouched.
- `verify_slice_against_cid` re-checks gateway-fetched bytes against the
  digest encoded in the CID itself, so backfill works with zero manifest
  knowledge; it understands exactly this one CID shape and refuses every
  other rather than guessing.
- Verified: `raw_leaf_cid_matches_known_vectors` (empty-block and
  `b"hello world"` vectors, malformed digests refused),
  `manifest_entry_cids_address_their_slices` (every packetized slice's CID
  verifies against its bytes; one flipped byte fails),
  `verify_slice_against_cid_refuses_other_shapes` (uppercase multibase,
  mutated header bytes, truncated digests).

## Still open / later

- Reed-Solomon/RaptorQ evaluation for lossy links: EVALUATED, integration
  still deferred. `xor_repair_loss_rate_evaluation`
  (types/src/utils/protocol_slice_v2.rs) is a seeded Monte Carlo of full
  payload recovery under independent per-slice loss, tabulated against a
  Reed-Solomon baseline at lower wire overhead (k + 25% parity vs the XOR
  tree's ~100%). Measured: the XOR ceiling collapses with scale — at 256 KiB
  (1023 slices) full recovery is 8.6% at 5% loss and 0% at ≥10%, while RS(25%)
  stays at 100% through 10% loss and degrades gracefully to its 20%
  theoretical ceiling. Verdict: keep XOR repair as the cheap assist for
  scattered single losses; plug a systematic code (reed-solomon-erasure or
  similar) in at the manifest `(offset, len)` seam when a lossy-link
  transport is actually built. RaptorQ has no mature maintained Rust crate;
  not recommended as a dependency today.
- `Repair-by-id as the git-transport loss path` (primary = stream, repair =
  slice requests by id) is what `--fetch` demos; production integration
  (manifest distribution to fetchers) is not built.
- Nostr carrier relay interop: built — see
  `slice_events_round_trip_through_live_relay` above. Not built on top of
  it: live-subscription sync (REQ streaming while slices arrive) and
  bridging the carrier into `RelayBridgeSession`.
