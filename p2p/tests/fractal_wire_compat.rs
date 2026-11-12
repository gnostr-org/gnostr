//! Byte-level cross-version wire compatibility tests: v1 -> v2 and v2 -> v1.
//!
//! These tests pin the EXACT differences between the v1 and v2 wire formats
//! for the same logical packet, in both directions:
//!
//! - **v1 -> v2**: the packet trees are identical (ids, data, parity flags,
//!   sequence numbers); the only structural delta is `HeaderV2.len`. v1's
//!   JSON bytes upgrade to v2 with exactly one inserted key, and a v1 slice
//!   verifies cleanly against the v2 manifest once upgraded. v1 bytes fed
//!   raw to a v2 decoder are REFUSED (`missing field `len``) — including the
//!   empty-placeholder slice that was the v1 ship-blocker, which can no
//!   longer poison a v2 receiver at all.
//! - **v2 -> v1**: a v2 slice read as v1 decodes but SILENTLY DROPS `len`
//!   (serde ignores the unknown field) — v1 survives the encounter but
//!   loses the self-describing length. A v2 RESPONSE
//!   (`Result<ProtocolSliceV2, String>`) cannot be parsed as v1 at all —
//!   neither the `{"Ok":{...}}` nor the `{"Err":"..."}` shape. v2 postcard
//!   bytes are garbage to the v1 JSON codec and vice versa.
//!
//! This matrix is why the v2 JSON behaviour sits on its own protocol id
//! (`/fractal/repair/2.0.0/json`): the only silent direction (v2 slice read
//! by v1) is a request body, which v1's responder only reads `id` from, so
//! no information v1 acts on is ever lost.

use futures::io::Cursor;
use gnostr_p2p::{
    packetize, packetize_v2, Header, HeaderV2, IntegrityManagerV2, PostcardCodec, ProtocolSlice,
    ProtocolSliceV2, RepairResponseV2,
};
use libp2p::request_response::Codec as _;
use libp2p::StreamProtocol;

/// The canonical slice used across the matrix: id "R", seq 1/3, two data
/// bytes, not parity. Small enough that every golden byte fits in one line.
fn v1_slice() -> ProtocolSlice {
    ProtocolSlice {
        id: "R".to_string(),
        header: Header {
            seq_num: 1,
            total_packets: 3,
        },
        data: vec![0xDE, 0xAD],
        is_parity: false,
    }
}

/// The same logical slice in v2 form.
fn v2_slice() -> ProtocolSliceV2 {
    ProtocolSliceV2 {
        id: "R".to_string(),
        header: HeaderV2 {
            seq_num: 1,
            total_packets: 3,
            len: 2,
        },
        data: vec![0xDE, 0xAD],
        is_parity: false,
    }
}

// ---------------------------------------------------------------------------
// v1 -> v2
// ---------------------------------------------------------------------------

#[test]
fn v1_to_v2_same_tree_same_payload() {
    // Constructing the same payload in v1 and v2 must produce the identical
    // packet tree; the only difference is the added `len` header field.
    let payload: Vec<u8> = (0..2500).map(|i| (i % 251) as u8).collect();
    let v1 = packetize("ROOT".to_string(), payload.clone());
    let v2 = packetize_v2("ROOT".to_string(), payload.clone());

    assert_eq!(v1.total_packets, v2.batch.total_packets);
    for (v1_packet, v2_packet) in v1.packets.iter().zip(v2.batch.packets.iter()) {
        assert_eq!(v1_packet.id, v2_packet.id, "tree shape diverged");
        assert_eq!(v1_packet.data, v2_packet.data, "payload diverged");
        assert_eq!(v1_packet.is_parity, v2_packet.is_parity);
        assert_eq!(
            v1_packet.header.seq_num, v2_packet.header.seq_num,
            "sequence diverged"
        );
        assert_eq!(
            v1_packet.header.total_packets,
            v2_packet.header.total_packets
        );
        // The exact and only header difference:
        assert_eq!(v2_packet.header.len as usize, v2_packet.data.len());
    }
    // Reconstruction is byte-identical across versions.
    let v1_reconstructed = gnostr_p2p::reconstruct_payload(&v1.packets);
    let mut manager = IntegrityManagerV2::new(v2.manifest.clone());
    for packet in &v2.batch.packets {
        manager.record_slice(packet.clone());
    }
    assert_eq!(
        v1_reconstructed,
        manager.reconstruct_payload().expect("v2 payload")
    );
}

#[test]
fn v1_json_upgrades_to_v2_with_exact_byte_delta() {
    // v1 JSON bytes for the canonical slice — key order and compaction
    // pinned byte-for-byte (this is exactly what the v1 JSON codec writes).
    let expected_v1 = "{\"id\":\"R\",\"header\":{\"seq_num\":1,\"total_packets\":3},\
                       \"data\":[222,173],\"is_parity\":false}"
        .replace(' ', "");
    let actual_v1 = serde_json::to_string(&v1_slice()).expect("encode v1");
    assert_eq!(actual_v1, expected_v1);

    // v2 JSON bytes differ by exactly one inserted key: "len":2 in the header.
    let expected_v2 = "{\"id\":\"R\",\"header\":{\"seq_num\":1,\"total_packets\":3,\"len\":2},\
                       \"data\":[222,173],\"is_parity\":false}"
        .replace(' ', "");
    let actual_v2 = serde_json::to_string(&v2_slice()).expect("encode v2");
    assert_eq!(actual_v2, expected_v2);
    assert_eq!(
        actual_v2,
        actual_v1.replace("\"total_packets\":3}", "\"total_packets\":3,\"len\":2}")
    );

    // The upgrade itself: v1 slice + computed len verifies against the v2
    // manifest and records cleanly.
    let upgraded = ProtocolSliceV2 {
        id: v1_slice().id,
        header: HeaderV2 {
            seq_num: v1_slice().header.seq_num,
            total_packets: v1_slice().header.total_packets,
            len: v1_slice().data.len() as u32,
        },
        data: v1_slice().data.clone(),
        is_parity: v1_slice().is_parity,
    };
    assert_eq!(upgraded, v2_slice());

    let manifest = gnostr_p2p::generate_manifest_v2("R".to_string(), &v1_slice().data);
    let mut manager = IntegrityManagerV2::new(manifest);
    manager.record_slice(upgraded);
    assert!(manager.rejected.is_empty(), "upgraded v1 slice must verify");
    assert_eq!(
        manager.reconstruct_payload().expect("payload"),
        v1_slice().data
    );
}

#[test]
fn v1_wire_bytes_are_unrepresentable_in_v2() {
    // Any v1 slice JSON (no `len` field) is a hard decode error for v2 —
    // serde refuses, so v1-shaped bytes can never enter a v2 receiver.
    let v1_bytes = serde_json::to_vec(&v1_slice()).expect("encode v1");
    let error =
        serde_json::from_slice::<ProtocolSliceV2>(&v1_bytes).expect_err("v2 must refuse v1");
    assert!(
        error.to_string().contains("missing field `len`"),
        "unexpected error: {error}"
    );

    // The v1 ship-blocker itself: the empty placeholder. On the v2 wire it
    // is unrepresentable — the exact bytes that silently corrupted v1
    // reconstruction cannot even be parsed by v2.
    let placeholder = ProtocolSlice {
        id: "R".to_string(),
        header: Header {
            seq_num: 0,
            total_packets: 3,
        },
        data: Vec::new(),
        is_parity: false,
    };
    let placeholder_bytes = serde_json::to_vec(&placeholder).expect("encode placeholder");
    let error =
        serde_json::from_slice::<ProtocolSliceV2>(&placeholder_bytes).expect_err("v2 must refuse");
    assert!(
        error.to_string().contains("missing field `len`"),
        "unexpected error: {error}"
    );
}

// ---------------------------------------------------------------------------
// v2 -> v1
// ---------------------------------------------------------------------------

#[test]
fn v2_json_read_by_v1_silently_drops_len() {
    // The one silent direction: v2's JSON slice decodes as v1's ProtocolSlice
    // because serde ignores the unknown `len` field. id/data/is_parity
    // survive; the self-describing length is dropped on the floor.
    let v2_bytes = serde_json::to_vec(&v2_slice()).expect("encode v2");
    let as_v1: ProtocolSlice = serde_json::from_slice(&v2_bytes).expect("v1 reads v2 slice JSON");
    assert_eq!(as_v1.id, "R");
    assert_eq!(as_v1.data, vec![0xDE, 0xAD]);
    assert!(!as_v1.is_parity);
    assert_eq!(as_v1.header.seq_num, 1);
    // And there is nowhere to put `len`: v1 cannot even represent it.
    let re_encoded = serde_json::to_string(&as_v1).expect("re-encode as v1");
    assert!(!re_encoded.contains("\"len\""));
    assert_eq!(
        re_encoded,
        serde_json::to_string(&v1_slice()).expect("v1 bytes")
    );
}

#[test]
fn v2_response_wrapper_fails_v1_parse() {
    // v2 answers repairs with Result<ProtocolSliceV2, String>. Neither
    // variant parses as v1's plain ProtocolSlice — the upgrade is one-way.
    let ok: RepairResponseV2 = Ok(v2_slice());
    let ok_bytes = serde_json::to_vec(&ok).expect("encode ok");
    assert_eq!(
        String::from_utf8_lossy(&ok_bytes),
        "{\"Ok\":{\"id\":\"R\",\"header\":{\"seq_num\":1,\"total_packets\":3,\"len\":2},\
         \"data\":[222,173],\"is_parity\":false}}"
            .replace(' ', "")
    );
    let error = serde_json::from_slice::<ProtocolSlice>(&ok_bytes).expect_err("v1 must fail");
    assert!(
        error.to_string().contains("missing field `id`"),
        "unexpected error: {error}"
    );

    let err: RepairResponseV2 = Err("not held: R".to_string());
    let err_bytes = serde_json::to_vec(&err).expect("encode err");
    assert_eq!(
        String::from_utf8_lossy(&err_bytes),
        "{\"Err\":\"not held: R\"}"
    );
    let error = serde_json::from_slice::<ProtocolSlice>(&err_bytes).expect_err("v1 must fail");
    assert!(
        error.to_string().contains("missing field `id`"),
        "unexpected error: {error}"
    );
}

#[test]
fn v2_postcard_bytes_fail_v1_json_codec_and_vice_versa() {
    // Binary direction: v2's postcard bytes are not JSON text. serde_json
    // IS the v1 JSON codec's body (it does to_vec/from_slice on the stream),
    // so a serde_json failure here is exactly what the v1 codec would turn
    // into io::ErrorKind::InvalidData.
    let v2_binary = postcard::to_allocvec(&v2_slice()).expect("encode v2 binary");
    let error = serde_json::from_slice::<ProtocolSlice>(&v2_binary).expect_err("v1 json must fail");
    println!("v1 json codec on v2 postcard bytes: {error}");
    assert!(!error.to_string().contains("missing field"));

    // And the exact same io::Error the codec would produce, via PostcardCodec
    // reading v1 JSON bytes: InvalidData, no partial values.
    let mut codec: PostcardCodec<ProtocolSlice, RepairResponseV2> = PostcardCodec::default();
    let protocol = StreamProtocol::new("/fractal/repair/1.1.0/postcard");
    let v1_json = serde_json::to_vec(&v1_slice()).expect("encode v1");
    let error =
        futures::executor::block_on(codec.read_response(&protocol, &mut Cursor::new(v1_json)))
            .expect_err("postcard codec must refuse JSON text");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn v2_postcard_framing_is_byte_exact() {
    // Golden postcard frames, field by field:
    //   slice  = varint(id_len) id | varint(seq) varint(total) varint(len)
    //            | varint(data_len) data | bool
    //   Ok(_)  = 0x00 || slice          (Result variant 0, then payload)
    //   Err(_) = 0x01 || varint(str_len) str   (Result variant 1)
    let slice = postcard::to_allocvec(&v2_slice()).expect("encode slice");
    assert_eq!(
        slice,
        vec![0x01, 0x52, 0x01, 0x03, 0x02, 0x02, 0xDE, 0xAD, 0x00]
    );

    let ok = postcard::to_allocvec(&Ok::<_, String>(v2_slice())).expect("encode ok");
    assert_eq!(
        ok,
        vec![0x00, 0x01, 0x52, 0x01, 0x03, 0x02, 0x02, 0xDE, 0xAD, 0x00]
    );

    let err = postcard::to_allocvec(&Err::<ProtocolSliceV2, _>("not held: R".to_string()))
        .expect("encode err");
    assert_eq!(
        err,
        vec![0x01, 0x0B, b'n', b'o', b't', b' ', b'h', b'e', b'l', b'd', b':', b' ', b'R']
    );

    // Same frames decode back through the actual wire codec.
    let mut codec: PostcardCodec<ProtocolSliceV2, RepairResponseV2> = PostcardCodec::default();
    let protocol = StreamProtocol::new("/fractal/repair/1.1.0/postcard");
    let decoded =
        futures::executor::block_on(codec.read_response(&protocol, &mut Cursor::new(err.clone())))
            .expect("decode err response");
    assert_eq!(decoded, Err("not held: R".to_string()));
}
