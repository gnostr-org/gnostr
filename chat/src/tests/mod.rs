mod p2p_chat_integration_tests;

#[test]
fn shared_packets_roundtrip_through_chat_repair_api() {
    let payload = b"chat shared packet payload\n".repeat(128);
    let batch = crate::types::perfect_ip::packetize("CHAT".into(), payload.clone());
    let encoded = serde_json::to_vec(&batch.packets).expect("serialize shared packets");
    let packets: Vec<crate::perfect_ip_core::ProtocolSlice> =
        serde_json::from_slice(&encoded).expect("deserialize repair packets");

    assert_eq!(
        crate::fractal::fractal_ip_core::reconstruct_payload(&packets),
        payload
    );
    let _: crate::types::PacketBatch = crate::perfect_ip_core::PacketBatch {
        total_packets: batch.total_packets,
        packets,
    };
    let _demo_entrypoint = crate::perfect_ip::run;
}

#[test]
fn fractal_slices_roundtrip_through_chat_facade() {
    let slice = crate::fractal::FractalSlice {
        block_id: "chat-block".into(),
        id: "CHAT.0".into(),
        offset: 0,
        data: b"chat fractal payload".to_vec(),
    };
    let encoded = serde_json::to_vec(&slice).expect("serialize fractal slice");
    let decoded: gnostr_p2p::fractal::FractalSlice =
        serde_json::from_slice(&encoded).expect("deserialize canonical fractal slice");

    assert_eq!(slice, decoded);
}
