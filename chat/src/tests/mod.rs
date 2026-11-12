#[cfg(feature = "fractal")]
mod fractal_integration_tests;
mod p2p_chat_integration_tests;

#[test]
fn shared_packets_roundtrip_through_chat_repair_api() {
    println!("\n============================================================");
    println!("CHAT SHARED PACKETS: packetization and repair API roundtrip");
    println!("============================================================");
    let payload = b"chat shared packet payload\n".repeat(128);
    let payload_hash = crate::types::sha256_hex(&payload);
    println!("source payload: {} bytes", payload.len());
    println!("source sha256: {payload_hash}");
    println!("packetizer: gnostr_chat::types::perfect_ip::packetize");
    let batch = crate::types::perfect_ip::packetize("CHAT".into(), payload.clone());
    println!("packet batch: {} packets", batch.total_packets);
    for summary in crate::perfect_ip_core::summarize_packets(&batch.packets) {
        println!("{summary}");
    }
    println!("-------------------- serialized packets --------------------");
    println!(
        "{}",
        serde_json::to_string_pretty(&batch.packets).expect("pretty-print shared packets")
    );
    let encoded = serde_json::to_vec(&batch.packets).expect("serialize shared packets");
    println!("serialized payload: {} bytes", encoded.len());
    println!("-------------------- repair reconstruction -----------------");
    let packets: Vec<crate::perfect_ip_core::ProtocolSlice> =
        serde_json::from_slice(&encoded).expect("deserialize repair packets");
    println!("decoded {} canonical repair packets", packets.len());
    let reconstructed = crate::perfect_ip_core::reconstruct_payload(&packets);
    println!("reconstructed payload: {} bytes", reconstructed.len());
    println!(
        "reconstructed sha256: {}",
        crate::types::sha256_hex(&reconstructed)
    );

    assert_eq!(reconstructed, payload);
    let _: crate::types::PacketBatch = crate::perfect_ip_core::PacketBatch {
        total_packets: batch.total_packets,
        packets,
    };
    let _demo_entrypoint = crate::perfect_ip::run;
    println!("============================================================");
    println!("PASS: shared packets roundtrip through the chat repair API");
    println!("Verified canonical packet types and preserved demo entrypoint");
    println!("============================================================\n");
}

#[test]
fn fractal_slices_roundtrip_through_chat_facade() {
    let slice = crate::p2p_fractal::FractalSlice {
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
