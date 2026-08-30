import XCTest
import CryptoKit
@testable import LibP2PAppTemplateCore

final class PerfectIPTests: XCTestCase {

    // MARK: - Packetization & Reconstruction

    func testPacketizeAndReconstructPayloadRoundtrip() throws {
        let original = Array("nostr dag p2p transfer payload".utf8)
        let slices = packetizePayload(rootId: "root-1", payload: original, threshold: 5)
        XCTAssertGreaterThan(slices.count, 1)
        XCTAssertTrue(slices.allSatisfy { $0.header.totalPackets == UInt64(slices.count) })

        let reconstructed = try reconstructPayload(slices: slices)
        XCTAssertEqual(reconstructed, original)
    }

    func testPacketizeEmptyPayloadEmitsSingleEmptySlice() throws {
        let slices = packetizePayload(rootId: "root-empty", payload: [], threshold: 8)
        XCTAssertEqual(slices.count, 1)
        XCTAssertEqual(slices[0].header.seqNum, 0)
        XCTAssertTrue(slices[0].data.isEmpty)
        XCTAssertFalse(slices[0].isParity)

        let reconstructed = try reconstructPayload(slices: slices)
        XCTAssertTrue(reconstructed.isEmpty)
    }

    func testReconstructRejectsMixedRoot() {
        let slices = [
            ProtocolSlice(id: "root-1.0", header: PacketHeader(seqNum: 0, totalPackets: 2), data: [1, 2], isParity: false),
            ProtocolSlice(id: "root-2.0", header: PacketHeader(seqNum: 1, totalPackets: 2), data: [3], isParity: false),
        ]
        XCTAssertThrowsError(try reconstructPayload(slices: slices)) { error in
            guard case TransferError.invalidPayload = error else {
                XCTFail("expected invalidPayload error")
                return
            }
        }
    }

    func testReconstructRejectsMixedTotalPackets() {
        let slices = [
            ProtocolSlice(id: "root.0", header: PacketHeader(seqNum: 0, totalPackets: 3), data: [1], isParity: false),
            ProtocolSlice(id: "root.1", header: PacketHeader(seqNum: 1, totalPackets: 2), data: [2], isParity: false),
        ]
        XCTAssertThrowsError(try reconstructPayload(slices: slices)) { error in
            guard case TransferError.invalidPayload = error else {
                XCTFail("expected invalidPayload error")
                return
            }
        }
    }

    // MARK: - Parity

    func testCalculateParityXorsEqualLengthBlocks() {
        let left: [UInt8] = [0xDE, 0xAD, 0xBE]
        let right: [UInt8] = [0x01, 0x02, 0x03]
        XCTAssertEqual(calculateParity(left, right), [0xDF, 0xAF, 0xBD])
    }

    func testCalculateParityHandlesUnequalLengths() {
        let left: [UInt8] = [0xFF, 0x00]
        let right: [UInt8] = [0x0F]
        XCTAssertEqual(calculateParity(left, right), [0xF0, 0x00])
    }

    func testRecoverMissingDataRestoresXorPartner() {
        let left: [UInt8] = [0xDE, 0xAD, 0xBE]
        let right: [UInt8] = [0x01, 0x02, 0x03]
        let parity = calculateParity(left, right)

        XCTAssertEqual(recoverMissingData(expectedLen: left.count, sibling: right, parity: parity), left)
        XCTAssertEqual(recoverMissingData(expectedLen: right.count, sibling: left, parity: parity), right)
    }

    func testRecoverMissingSliceRebuildsHeaderAndPayload() {
        let sibling = ProtocolSlice(
            id: "ROOT.1",
            header: PacketHeader(seqNum: 1, totalPackets: 3),
            data: [0x01, 0x02, 0x03],
            isParity: false
        )
        let parity = ProtocolSlice(
            id: "ROOT.P",
            header: PacketHeader(seqNum: 2, totalPackets: 3),
            data: [0xDF, 0xAF, 0xBD],
            isParity: true
        )
        var seq: UInt64 = 3

        let recovered = recoverMissingSlice(id: "ROOT.0", expectedLen: 3, sibling: sibling, parity: parity, seq: &seq)

        XCTAssertEqual(recovered.id, "ROOT.0")
        XCTAssertEqual(recovered.data, [0xDE, 0xAD, 0xBE])
        XCTAssertEqual(recovered.header.seqNum, 3)
        XCTAssertEqual(recovered.header.totalPackets, 3)
        XCTAssertFalse(recovered.isParity)
    }

    // MARK: - Manifest

    func testGenerateManifestMatchesPacketTree() {
        let manifest = generateManifest(id: "ROOT", len: 2000, threshold: maxLeafPayload)
        XCTAssertTrue(manifest.contains("ROOT.P"))
        XCTAssertTrue(manifest.contains(where: { $0.hasPrefix("ROOT.0.") }))
        XCTAssertTrue(manifest.contains(where: { $0.hasPrefix("ROOT.1.") }))
        XCTAssertEqual(manifest.count, packetizePayload(rootId: "ROOT", payload: [UInt8](repeating: 0xAB, count: 2000), threshold: maxLeafPayload).count)
    }

    // MARK: - Transfer Event Parsing

    func testParseTransferManifestAndSliceEvents() throws {
        let payload = Array("abcdefgh".utf8)
        let slices = packetizePayload(rootId: "root-2", payload: payload, threshold: 3)
        let sha256 = payload.sha256Hex
        let manifest = PacketManifest(
            root: "root-2",
            sha256: sha256,
            size: UInt64(payload.count),
            packets: UInt64(slices.count),
            depth: 2,
            mtu: 3,
            encoding: "json",
            path: ""
        )
        let manifestEvent = buildTransferManifestEvent(manifest: manifest)
        let parsedManifest = try parseTransferEvent(content: manifestEvent.content, kind: manifestEvent.kind)
        guard case .manifest(let parsed) = parsedManifest else {
            XCTFail("expected manifest")
            return
        }
        XCTAssertEqual(parsed, manifest)

        let sliceEvent = buildTransferSliceEvent(slice: slices[0], manifestID: manifestEvent.id)
        let parsedSlice = try parseTransferEvent(content: sliceEvent.content, kind: sliceEvent.kind)
        guard case .slice(let parsedS) = parsedSlice else {
            XCTFail("expected slice")
            return
        }
        XCTAssertEqual(parsedS, slices[0])
    }

    func testParseTransferEventRejectsWrongProtocol() {
        let content = "{\"protocol\":\"wrong\",\"version\":1}"
        XCTAssertThrowsError(try parseTransferEvent(content: content, kind: transferManifestKind)) { error in
            guard case TransferError.invalidPayload = error else {
                XCTFail("expected invalidPayload")
                return
            }
        }
    }

    func testParseTransferEventRejectsWrongVersion() {
        let content = "{\"protocol\":\"nostr-dag-transfer\",\"version\":99}"
        XCTAssertThrowsError(try parseTransferEvent(content: content, kind: transferManifestKind)) { error in
            guard case TransferError.invalidPayload = error else {
                XCTFail("expected invalidPayload")
                return
            }
        }
    }

    // MARK: - Bridge Envelope

    func testBridgeMessageRoundtrip() throws {
        let payload = Array("fractal swarm adaptation for nostr dag".utf8)
        let (manifestEvent, sliceEvents) = encodePayloadAsTransferEventsChained(
            rootId: "root-bridge",
            payload: payload,
            threshold: 7
        )

        let relayHints = ["ws://localhost:8080"]
        let manifestEnvelope = try encodeBridgeMessage(event: manifestEvent, direction: "outbound", relayHints: relayHints)
        let decodedManifest = try decodeBridgeMessage(message: manifestEnvelope)
        XCTAssertEqual(decodedManifest.event.kind, transferManifestKind)

        for event in sliceEvents {
            let envelope = try encodeBridgeMessage(event: event, direction: "outbound", relayHints: relayHints)
            let decoded = try decodeBridgeMessage(message: envelope)
            XCTAssertEqual(decoded.event.kind, transferSliceKind)
        }
    }

    func testDecodeBridgeMessageRejectsWrongProtocol() {
        let envelope = "{\"protocol\":\"wrong\",\"version\":\"1\",\"direction\":\"outbound\",\"event\":{\"kind\":39078,\"content\":\"{}\",\"id\":\"abc\",\"topicTags\":[]},\"relayHints\":[]}"
        XCTAssertThrowsError(try decodeBridgeMessage(message: envelope)) { error in
            guard case TransferError.invalidEnvelope = error else {
                XCTFail("expected invalidEnvelope")
                return
            }
        }
    }

    // MARK: - Event Chaining

    func testEventChainingParentReferences() {
        let payload = Array("hello nip-pip chain".utf8)
        let (manifest, slices) = encodePayloadAsTransferEventsChained(
            rootId: "chain-test",
            payload: payload,
            threshold: 4
        )

        XCTAssertNil(manifest.parentID)
        guard let firstSlice = slices.first else {
            XCTFail("expected slices")
            return
        }
        XCTAssertEqual(firstSlice.parentID, manifest.id)

        for i in 1..<slices.count {
            XCTAssertEqual(slices[i].parentID, slices[i - 1].id)
        }
    }

    func testEventChainingRTTPropagation() {
        let payload = Array("rtt test".utf8)
        let rtt: Int64 = 1_700_000_000_000
        let (manifest, slices) = encodePayloadAsTransferEventsChained(
            rootId: "rtt-test",
            payload: payload,
            threshold: 3,
            rttStartedAtMs: rtt
        )

        XCTAssertEqual(manifest.rttStartedAtMs, rtt)
        XCTAssertTrue(slices.allSatisfy { $0.rttStartedAtMs == rtt })
    }

    // MARK: - Integrity Manager (from perfect_ip.rs)

    func testIntegrityManagerMissingNodes() {
        var manager = IntegrityManager(expectedIds: ["ROOT.0", "ROOT.1", "ROOT.P"])
        manager.recordSlice(ProtocolSlice(
            id: "ROOT.0",
            header: PacketHeader(seqNum: 0, totalPackets: 3),
            data: [0xDE],
            isParity: false
        ))

        let missing = manager.getMissingNodes()
        XCTAssertEqual(missing.count, 2)
        XCTAssertTrue(missing.contains("ROOT.1"))
        XCTAssertTrue(missing.contains("ROOT.P"))
    }

    func testIntegrityManagerVerifyIntegrity() {
        var manager = IntegrityManager(expectedIds: ["ROOT.0", "ROOT.1", "ROOT.P"])
        let left = ProtocolSlice(
            id: "ROOT.0",
            header: PacketHeader(seqNum: 0, totalPackets: 3),
            data: [0xDE, 0xAD, 0xBE],
            isParity: false
        )
        let right = ProtocolSlice(
            id: "ROOT.1",
            header: PacketHeader(seqNum: 1, totalPackets: 3),
            data: [0x01, 0x02, 0x03],
            isParity: false
        )
        let parity = ProtocolSlice(
            id: "ROOT.P",
            header: PacketHeader(seqNum: 2, totalPackets: 3),
            data: calculateParity(left.data, right.data),
            isParity: true
        )

        manager.recordSlice(left)
        manager.recordSlice(right)
        manager.recordSlice(parity)
        XCTAssertTrue(manager.verifyIntegrity())

        var corrupted = manager
        var corruptedParity = corrupted.receivedSlices["ROOT.P"]!
        corruptedParity.data[0] ^= 0xFF
        corrupted.receivedSlices["ROOT.P"] = corruptedParity
        XCTAssertFalse(corrupted.verifyIntegrity())
    }

    func testIntegrityManagerPersistAndLoad() throws {
        let tempURL = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        defer { try? FileManager.default.removeItem(at: tempURL) }

        var manager = IntegrityManager(expectedIds: ["ROOT.0", "ROOT.1", "ROOT.P"])
        manager.recordSlice(ProtocolSlice(
            id: "ROOT.0",
            header: PacketHeader(seqNum: 0, totalPackets: 3),
            data: [0xDE, 0xAD, 0xBE],
            isParity: false
        ))
        manager.recordSlice(ProtocolSlice(
            id: "ROOT.P",
            header: PacketHeader(seqNum: 1, totalPackets: 3),
            data: [0xAA],
            isParity: true
        ))

        try manager.persist(to: tempURL)
        let loaded = try IntegrityManager.load(from: tempURL, manifest: ["ROOT.0", "ROOT.1", "ROOT.P"])

        XCTAssertEqual(loaded.receivedSlices.count, 2)
        XCTAssertTrue(loaded.receivedSlices.keys.contains("ROOT.0"))
        XCTAssertTrue(loaded.receivedSlices.keys.contains("ROOT.P"))
        XCTAssertTrue(loaded.getMissingNodes().contains("ROOT.1"))
    }

    // MARK: - JSON Roundtrip

    func testProtocolSliceJSONRoundTripsBytesAndFlags() throws {
        let slice = ProtocolSlice(
            id: "ROOT.0.P",
            header: PacketHeader(seqNum: 7, totalPackets: 63),
            data: [0x00, 0xFF, 0x10, 0x20],
            isParity: true
        )

        let encoder = JSONEncoder()
        let decoder = JSONDecoder()
        let encoded = try encoder.encode(slice)
        let jsonString = String(data: encoded, encoding: .utf8)!

        XCTAssertTrue(jsonString.contains("\"id\":\"ROOT.0.P\""))
        XCTAssertTrue(jsonString.contains("\"isParity\":true"))
        XCTAssertTrue(jsonString.contains("\"data\":[0,255,16,32]"))

        let decoded = try decoder.decode(ProtocolSlice.self, from: encoded)
        XCTAssertEqual(decoded, slice)
    }

    func testPacketManifestJSONRoundtrip() throws {
        let manifest = PacketManifest(
            root: "test-root",
            sha256: "abcd1234",
            size: 100,
            packets: 5,
            depth: 2,
            mtu: 8,
            encoding: "json",
            path: "/test"
        )
        let encoder = JSONEncoder()
        let decoder = JSONDecoder()
        let data = try encoder.encode(manifest)
        let decoded = try decoder.decode(PacketManifest.self, from: data)
        XCTAssertEqual(decoded, manifest)
    }

    // MARK: - Summaries

    func testSummarizePacketsShowsInventoryDetails() {
        let slices = packetizePayload(rootId: "ROOT", payload: [UInt8](repeating: 0xAB, count: 2000), threshold: maxLeafPayload)
        let lines = summarizePackets(slices)

        XCTAssertEqual(lines.count, slices.count)
        XCTAssertTrue(lines.contains(where: { $0.contains("ID: ROOT.P") }))
        XCTAssertTrue(lines.contains(where: { $0.contains("Type: DATA") }))
        XCTAssertTrue(lines.contains(where: { $0.contains("Type: PARITY") }))
    }

    // MARK: - Full Protocol Dump (Interop)

    func testFullProtocolReconstructsPayload() throws {
        let payload = [UInt8](repeating: 0xAB, count: 3000)
        let slices = packetizePayload(rootId: "ROOT", payload: payload, threshold: maxLeafPayload)
        let manifest = generateManifest(id: "ROOT", len: payload.count, threshold: maxLeafPayload)
        var manager = IntegrityManager(expectedIds: manifest)

        for slice in slices {
            manager.recordSlice(slice)
        }

        XCTAssertTrue(manager.getMissingNodes().isEmpty)
        XCTAssertTrue(manager.verifyIntegrity())

        var reconstructed = [UInt8]()
        for packet in slices.sorted(by: { $0.header.seqNum < $1.header.seqNum }) where !packet.isParity {
            reconstructed.append(contentsOf: packet.data)
        }
        XCTAssertEqual(reconstructed, payload)
    }

    func testInteropWithRustThreshold() throws {
        // The Rust nip-pip-example uses threshold=8 for "hello nip-pip rtt example"
        let payload = Array("hello nip-pip rtt example".utf8)
        let slices = packetizePayload(rootId: "nip-pip-example-native", payload: payload, threshold: 8)
        let reconstructed = try reconstructPayload(slices: slices)
        XCTAssertEqual(reconstructed, payload)
        // Verify manifest depth matches expected tree shape
        let manifest = generateManifest(id: "nip-pip-example-native", len: payload.count, threshold: 8)
        XCTAssertEqual(slices.count, manifest.count)
    }
}

// MARK: - SHA-256 Helper

private extension [UInt8] {
    var sha256Hex: String {
        Data(self).sha256Hex
    }
}

private extension Data {
    var sha256Hex: String {
        CryptoKit.SHA256.hash(data: self).compactMap { String(format: "%02x", $0) }.joined()
    }
}
