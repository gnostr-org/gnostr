import Foundation
import CryptoKit

// MARK: - Constants

/// Gossipsub topic and bridge protocol identifier used by all PIP peers.
public let nostrDagTopic = "nostr-dag-bridge"
public let networkTimeProtocol = "nostr-dag-network-time"
public let networkTimeVersion: UInt64 = 1

/// PIP Nostr event kind used for transfer manifests.
public let transferManifestKind: UInt64 = 39078
/// PIP Nostr event kind used for transfer slices.
public let transferSliceKind: UInt64 = 39079

/// PIP protocol name carried in transfer manifest and slice event payloads.
public let transferProtocol = "nostr-dag-transfer"
/// PIP transfer payload version.
public let transferVersion: UInt64 = 1

/// Maximum payload size this packet protocol is allowed to emit.
public let mtuPayload: Int = 1460
/// Half of `mtuPayload`; leaf threshold from the original `perfect_ip.rs`.
public let maxLeafPayload: Int = mtuPayload / 2

// MARK: - Errors

public enum TransferError: Error, Equatable {
    case unsupportedKind(String)
    case invalidPayload(String)
    case missingField(String)
    case invalidEnvelope(String)
    case json(String)
}

// MARK: - Types

/// Packet header metadata shared by all packet types in the tree.
public struct PacketHeader: Codable, Equatable, Sendable {
    /// Monotonic sequence number assigned during packetization.
    public var seqNum: UInt64
    /// Total number of packets in the finalized batch.
    public var totalPackets: UInt64

    public init(seqNum: UInt64, totalPackets: UInt64) {
        self.seqNum = seqNum
        self.totalPackets = totalPackets
    }
}

/// A single packet produced by the recursive packetizer.
///
/// `id` carries the recursive path for the packet, such as `ROOT.0.1.P`.
/// `isParity` marks frames that store XOR parity rather than original user
/// data. `data` always contains the raw bytes that would be transmitted on the
/// wire inside the JSON-encoded repair message.
public struct ProtocolSlice: Codable, Equatable, Sendable {
    /// Stable recursive packet identifier.
    public var id: String
    /// Packet sequencing metadata.
    public var header: PacketHeader
    /// Raw payload bytes for the packet or parity frame.
    public var data: [UInt8]
    /// `true` when this slice is a parity frame.
    public var isParity: Bool

    public init(id: String, header: PacketHeader, data: [UInt8], isParity: Bool) {
        self.id = id
        self.header = header
        self.data = data
        self.isParity = isParity
    }
}

/// PIP transfer manifest describing a multi-slice payload.
///
/// `root` is an application-defined identifier shared by the manifest and all
/// its slices. `sha256` is the lowercase-hex digest of the *full reconstructed*
/// payload. `path` is an optional resource identifier (e.g. a git repo URL)
/// that browsers use to index available bundles.
public struct PacketManifest: Codable, Equatable, Sendable {
    public var root: String
    public var sha256: String
    public var size: UInt64
    public var packets: UInt64
    public var depth: UInt32
    public var mtu: UInt64
    public var encoding: String
    public var path: String

    public init(root: String, sha256: String, size: UInt64, packets: UInt64, depth: UInt32, mtu: UInt64, encoding: String, path: String) {
        self.root = root
        self.sha256 = sha256
        self.size = size
        self.packets = packets
        self.depth = depth
        self.mtu = mtu
        self.encoding = encoding
        self.path = path
    }
}

/// Parsed transfer event payload.
public enum TransferEventPayload: Equatable, Sendable {
    case manifest(PacketManifest)
    case slice(ProtocolSlice)
}

extension TransferEventPayload: Codable {
    private enum CodingKeys: String, CodingKey {
        case type, manifest, slice
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)
        switch type {
        case "manifest":
            self = .manifest(try container.decode(PacketManifest.self, forKey: .manifest))
        case "slice":
            self = .slice(try container.decode(ProtocolSlice.self, forKey: .slice))
        default:
            throw TransferError.invalidPayload("unknown payload type: \(type)")
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .manifest(let m):
            try container.encode("manifest", forKey: .type)
            try container.encode(m, forKey: .manifest)
        case .slice(let s):
            try container.encode("slice", forKey: .type)
            try container.encode(s, forKey: .slice)
        }
    }
}

/// A lightweight transfer event suitable for bridge encoding.
///
/// In the canonical Rust implementation this is a signed Nostr event. In the
/// Swift port the ID is a deterministic SHA-256 of the JSON content so that
/// parent chaining and round-trip timing still work without a Nostr signer.
public struct TransferEvent: Codable, Equatable, Sendable {
    /// Event kind (39078 manifest or 39079 slice).
    public var kind: UInt64
    /// JSON-encoded event content.
    public var content: String
    /// Deterministic event ID (SHA-256 hex of content).
    public var id: String
    /// Parent event ID for chaining (the `e` tag value).
    public var parentID: String?
    /// Optional RTT start timestamp in milliseconds.
    public var rttStartedAtMs: Int64?
    /// Topic tags for discoverability.
    public var topicTags: [String]
}

/// Bridge envelope wrapping a transfer event for gossipsub publication.
public struct BridgeEnvelope: Codable, Equatable, Sendable {
    public var `protocol`: String
    public var version: String
    public var direction: String
    public var event: TransferEvent
    public var relayHints: [String]
}

/// Tracks the expected manifest and the packets that have arrived so far.
///
/// The manager is deliberately simple: it stores the expected ids, the received
/// slices, and exposes helpers for missing-node inspection, parity validation,
/// and persistence.
public struct IntegrityManager: Equatable, Sendable {
    private var manifest: Set<String>
    public var receivedSlices: [String: ProtocolSlice]

    /// Create a manager from the set of expected packet ids.
    public init(expectedIds: [String]) {
        self.manifest = Set(expectedIds)
        self.receivedSlices = [:]
    }

    private init(manifest: Set<String>, receivedSlices: [String: ProtocolSlice]) {
        self.manifest = manifest
        self.receivedSlices = receivedSlices
    }

    /// Record a received packet by id, replacing any prior packet with the same id.
    public mutating func recordSlice(_ slice: ProtocolSlice) {
        receivedSlices[slice.id] = slice
    }

    /// Persist the received packet map to disk as JSON.
    public func persist(to url: URL) throws {
        let encoder = JSONEncoder()
        let data = try encoder.encode(receivedSlices)
        try data.write(to: url)
    }

    /// Load a persisted packet map from disk and attach it to the given manifest.
    public static func load(from url: URL, manifest: Set<String>) throws -> IntegrityManager {
        let data = try Data(contentsOf: url)
        let decoder = JSONDecoder()
        let receivedSlices = try decoder.decode([String: ProtocolSlice].self, from: data)
        return IntegrityManager(manifest: manifest, receivedSlices: receivedSlices)
    }

    /// Return the packet ids from the manifest that are still missing.
    public func getMissingNodes() -> [String] {
        manifest.filter { !receivedSlices.keys.contains($0) }.sorted()
    }

    /// Verify that every parity slice matches the XOR of its sibling data slices.
    public func verifyIntegrity() -> Bool {
        for (id, slice) in receivedSlices {
            guard slice.isParity else { continue }
            guard id.hasSuffix(".P") else { return false }
            let baseId = String(id.dropLast(2))
            guard !baseId.isEmpty else { return false }
            guard let left = receivedSlices["\(baseId).0"],
                  let right = receivedSlices["\(baseId).1"] else {
                continue
            }
            if calculateParity(left.data, right.data) != slice.data {
                return false
            }
        }
        return true
    }
}

// MARK: - Core Functions

/// XOR two payloads into a parity buffer.
///
/// Missing bytes are treated as zero so the returned buffer is as long as the
/// larger input. This is the core repair primitive used at every branch in the
/// recursive packet tree.
public func calculateParity(_ left: [UInt8], _ right: [UInt8]) -> [UInt8] {
    let maxLen = max(left.count, right.count)
    var parity = [UInt8](repeating: 0, count: maxLen)
    for i in 0..<maxLen {
        let l = i < left.count ? left[i] : 0
        let r = i < right.count ? right[i] : 0
        parity[i] = l ^ r
    }
    return parity
}

/// Recursively split a payload into MTU-safe slices and parity frames.
private func recursivePacketize(
    id: String,
    data: [UInt8],
    threshold: Int,
    seq: inout UInt64,
    maxDepth: inout UInt32
) -> [ProtocolSlice] {
    if data.count <= threshold {
        let slice = ProtocolSlice(
            id: id,
            header: PacketHeader(seqNum: seq, totalPackets: 0),
            data: data,
            isParity: false
        )
        seq += 1
        let depth = id.filter { $0 == "." }.count
        maxDepth = max(maxDepth, UInt32(depth))
        return [slice]
    }

    let half = data.count / 2
    let leftData = Array(data[..<half])
    let rightData = Array(data[half...])
    let parityData = calculateParity(leftData, rightData)

    var slices = recursivePacketize(id: "\(id).0", data: leftData, threshold: threshold, seq: &seq, maxDepth: &maxDepth)
    slices.append(contentsOf: recursivePacketize(id: "\(id).1", data: rightData, threshold: threshold, seq: &seq, maxDepth: &maxDepth))
    slices.append(ProtocolSlice(
        id: "\(id).P",
        header: PacketHeader(seqNum: seq, totalPackets: 0),
        data: parityData,
        isParity: true
    ))
    seq += 1
    return slices
}

/// Split payload bytes into a recursive PIP packet tree with parity slices.
///
/// The output follows the gnostr recursive packet-tree model:
/// - binary split until `data.count <= threshold`
/// - one parity slice (`id.P`) emitted at each internal node
/// - leaf slices are data slices with `isParity: false`
/// - empty payloads still emit a single empty slice so reconstruction remains well-defined
public func packetizePayload(rootId: String, payload: [UInt8], threshold: Int) -> [ProtocolSlice] {
    let threshold = max(threshold, 1)
    var seq: UInt64 = 0
    var depth: UInt32 = 0
    var slices = recursivePacketize(id: rootId, data: payload, threshold: threshold, seq: &seq, maxDepth: &depth)
    let total = UInt64(slices.count)
    for i in slices.indices {
        slices[i].header.totalPackets = total
    }
    return slices
}

/// Generate the expected packet ids for a recursive packet tree.
public func generateManifest(id: String, len: Int, threshold: Int) -> [String] {
    let threshold = max(threshold, 1)
    if len <= threshold {
        return [id]
    }
    let half = len / 2
    var ids = generateManifest(id: "\(id).0", len: half, threshold: threshold)
    ids.append(contentsOf: generateManifest(id: "\(id).1", len: len - half, threshold: threshold))
    ids.append("\(id).P")
    return ids
}

/// Recover the missing payload by XORing the sibling payload and parity frame.
public func recoverMissingData(expectedLen: Int, sibling: [UInt8], parity: [UInt8]) -> [UInt8] {
    let recovered = calculateParity(sibling, parity)
    return Array(recovered.prefix(expectedLen))
}

/// Rebuild a missing packet using a sibling packet and parity packet.
public func recoverMissingSlice(id: String, expectedLen: Int, sibling: ProtocolSlice, parity: ProtocolSlice, seq: inout UInt64) -> ProtocolSlice {
    let header = PacketHeader(
        seqNum: seq,
        totalPackets: max(sibling.header.totalPackets, parity.header.totalPackets)
    )
    seq += 1
    return ProtocolSlice(
        id: id,
        header: header,
        data: recoverMissingData(expectedLen: expectedLen, sibling: sibling.data, parity: parity.data),
        isParity: false
    )
}

/// Reconstruct the original payload from validated PIP transfer slices.
///
/// Reconstruction collects all data slices (`isParity: false`), orders them by
/// `seqNum`, and concatenates their payloads. Parity slices are ignored during
/// normal reconstruction; they are used only when slices are missing.
public func reconstructPayload(slices: [ProtocolSlice]) throws -> [UInt8] {
    if slices.isEmpty {
        return []
    }

    let rootId = slices[0].id.split(separator: ".").first.map(String.init) ?? ""
    let expectedTotal = slices[0].header.totalPackets

    for slice in slices {
        let sliceRoot = slice.id.split(separator: ".").first.map(String.init) ?? ""
        if sliceRoot != rootId {
            throw TransferError.invalidPayload("mixed root values in transfer slices")
        }
        if slice.header.totalPackets != expectedTotal {
            throw TransferError.invalidPayload("mixed totalPackets values in transfer slices")
        }
    }

    var dataSlices = slices.filter { !$0.isParity }
    dataSlices.sort { $0.header.seqNum < $1.header.seqNum }
    return dataSlices.flatMap(\.data)
}

// MARK: - Transfer Event Builders

/// Standard topic tags applied to every NIP-PIP event for discoverability.
public func pipTopicTags() -> [String] {
    ["nostr-dag", "nip-pip", "transfer"]
}

/// Deterministic event ID from content (mimics Nostr SHA-256 without signer).
private func deterministicEventID(content: String) -> String {
    Data(SHA256.hash(data: Data(content.utf8))).map { String(format: "%02x", $0) }.joined()
}

/// Build a PIP transfer-manifest event.
public func buildTransferManifestEvent(manifest: PacketManifest, rttStartedAtMs: Int64? = nil) -> TransferEvent {
    let contentObj: [String: Any] = [
        "protocol": transferProtocol,
        "version": transferVersion,
        "type": "manifest",
        "root": manifest.root,
        "sha256": manifest.sha256,
        "size": manifest.size,
        "packets": manifest.packets,
        "depth": manifest.depth,
        "mtu": manifest.mtu,
        "encoding": manifest.encoding,
        "path": manifest.path,
    ]
    let content = jsonString(contentObj)
    return TransferEvent(
        kind: transferManifestKind,
        content: content,
        id: deterministicEventID(content: content),
        parentID: nil,
        rttStartedAtMs: rttStartedAtMs,
        topicTags: pipTopicTags()
    )
}

/// Build a PIP transfer-slice event and link it to the manifest via parent ID.
public func buildTransferSliceEvent(slice: ProtocolSlice, manifestID: String, rttStartedAtMs: Int64? = nil) -> TransferEvent {
    let contentObj: [String: Any] = [
        "protocol": transferProtocol,
        "version": transferVersion,
        "type": "slice",
        "id": slice.id,
        "header": [
            "seq_num": slice.header.seqNum,
            "total_packets": slice.header.totalPackets,
        ],
        "data": slice.data,
        "is_parity": slice.isParity,
    ]
    let content = jsonString(contentObj)
    return TransferEvent(
        kind: transferSliceKind,
        content: content,
        id: deterministicEventID(content: content),
        parentID: manifestID,
        rttStartedAtMs: rttStartedAtMs,
        topicTags: pipTopicTags()
    )
}

/// Encode a payload into a deterministic chain of PIP manifest + slice events.
///
/// Each slice carries a parent reference (the manifest for slice 0, the
/// previous slice for slice N), producing a verifiable chain. When
/// `rttStartedAtMs` is provided, every event is stamped with RTT metadata.
public func encodePayloadAsTransferEventsChained(
    rootId: String,
    payload: [UInt8],
    threshold: Int,
    rttStartedAtMs: Int64? = nil,
    path: String = ""
) -> (TransferEvent, [TransferEvent]) {
    let sha256 = Data(SHA256.hash(data: Data(payload))).map { String(format: "%02x", $0) }.joined()
    let slices = packetizePayload(rootId: rootId, payload: payload, threshold: threshold)
    let maxDepth: UInt32 = {
        var d: UInt32 = 0
        for s in slices {
            let depth = s.id.filter { $0 == "." }.count
            d = max(d, UInt32(depth))
        }
        return d
    }()
    let manifest = PacketManifest(
        root: rootId,
        sha256: sha256,
        size: UInt64(payload.count),
        packets: UInt64(slices.count),
        depth: maxDepth,
        mtu: UInt64(threshold),
        encoding: "json",
        path: path
    )

    let manifestEvent = buildTransferManifestEvent(manifest: manifest, rttStartedAtMs: rttStartedAtMs)

    var sliceEvents: [TransferEvent] = []
    var parentID = manifestEvent.id
    for slice in slices {
        let event = buildTransferSliceEvent(slice: slice, manifestID: parentID, rttStartedAtMs: rttStartedAtMs)
        sliceEvents.append(event)
        parentID = event.id
    }

    return (manifestEvent, sliceEvents)
}

/// Parse a PIP manifest or slice transfer event payload.
public func parseTransferEvent(content: String, kind: UInt64) throws -> TransferEventPayload {
    guard let data = content.data(using: .utf8) else {
        throw TransferError.json("invalid utf-8 content")
    }
    let json = try JSONSerialization.jsonObject(with: data) as? [String: Any] ?? [:]

    guard let proto = json["protocol"] as? String, proto == transferProtocol else {
        throw TransferError.invalidPayload("protocol mismatch: expected \(transferProtocol)")
    }
    guard let version = json["version"] as? UInt64, version == transferVersion else {
        throw TransferError.invalidPayload("version mismatch: expected \(transferVersion)")
    }

    if kind == transferManifestKind {
        guard let root = json["root"] as? String else { throw TransferError.missingField("root") }
        guard let sha256 = json["sha256"] as? String else { throw TransferError.missingField("sha256") }
        guard let size = json["size"] as? UInt64 else { throw TransferError.missingField("size") }
        guard let packets = json["packets"] as? UInt64 else { throw TransferError.missingField("packets") }
        guard let depth = json["depth"] as? UInt32 else { throw TransferError.missingField("depth") }
        guard let mtu = json["mtu"] as? UInt64 else { throw TransferError.missingField("mtu") }
        guard let encoding = json["encoding"] as? String else { throw TransferError.missingField("encoding") }
        guard let path = json["path"] as? String else { throw TransferError.missingField("path") }
        return .manifest(PacketManifest(root: root, sha256: sha256, size: size, packets: packets, depth: depth, mtu: mtu, encoding: encoding, path: path))
    }

    if kind == transferSliceKind {
        guard let id = json["id"] as? String else { throw TransferError.missingField("id") }
        guard let headerObj = json["header"] as? [String: Any] else { throw TransferError.missingField("header") }
        guard let seqNum = headerObj["seq_num"] as? UInt64 else { throw TransferError.missingField("seq_num") }
        guard let totalPackets = headerObj["total_packets"] as? UInt64 else { throw TransferError.missingField("total_packets") }
        guard let dataArr = json["data"] as? [UInt64] else { throw TransferError.missingField("data") }
        let sliceData = dataArr.compactMap { UInt8(exactly: $0) }
        guard sliceData.count == dataArr.count else { throw TransferError.invalidPayload("slice data byte out of range") }
        guard let isParity = json["is_parity"] as? Bool else { throw TransferError.missingField("is_parity") }
        return .slice(ProtocolSlice(id: id, header: PacketHeader(seqNum: seqNum, totalPackets: totalPackets), data: sliceData, isParity: isParity))
    }

    throw TransferError.unsupportedKind("\(kind)")
}

// MARK: - Bridge Envelope

/// Encode a transfer event as a PIP bridge envelope.
public func encodeBridgeMessage(event: TransferEvent, direction: String, relayHints: [String]) throws -> String {
    let envelope = BridgeEnvelope(
        protocol: nostrDagTopic,
        version: "1",
        direction: direction,
        event: event,
        relayHints: relayHints
    )
    let data: Data
    do {
        data = try JSONEncoder().encode(envelope)
    } catch {
        throw TransferError.json("\(error)")
    }
    guard let string = String(data: data, encoding: .utf8) else {
        throw TransferError.json("utf-8 encoding failed")
    }
    return string
}

/// Decode and validate a PIP bridge envelope.
public func decodeBridgeMessage(message: String) throws -> BridgeEnvelope {
    guard let data = message.data(using: .utf8) else {
        throw TransferError.json("invalid utf-8")
    }
    let envelope: BridgeEnvelope
    do {
        envelope = try JSONDecoder().decode(BridgeEnvelope.self, from: data)
    } catch {
        throw TransferError.json("\(error)")
    }
    guard envelope.protocol == nostrDagTopic else {
        throw TransferError.invalidEnvelope("protocol mismatch: expected \(nostrDagTopic), got \(envelope.protocol)")
    }
    return envelope
}

// MARK: - Helpers

private func jsonString(_ obj: [String: Any]) -> String {
    guard let data = try? JSONSerialization.data(withJSONObject: obj, options: .sortedKeys),
          let string = String(data: data, encoding: .utf8) else {
        return "{}"
    }
    return string
}

/// Render packet summaries for logs or diagnostics.
public func summarizePackets(_ packets: [ProtocolSlice]) -> [String] {
    packets.map { packet in
        let typeStr = packet.isParity ? "PARITY" : "DATA"
        return String(
            format: "ID: %-8@ | Seq: %2d/%d | Type: %-6@ | Size: %dB",
            packet.id,
            Int(packet.header.seqNum),
            Int(packet.header.totalPackets),
            typeStr,
            packet.data.count
        )
    }
}
