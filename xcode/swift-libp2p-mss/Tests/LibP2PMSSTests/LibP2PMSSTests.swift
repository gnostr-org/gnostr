//===----------------------------------------------------------------------===//
//
// This source file is part of the swift-libp2p open source project
//
// Copyright (c) 2022-2025 swift-libp2p project authors
// Licensed under MIT
//
// See LICENSE for license information
// See CONTRIBUTORS for the list of swift-libp2p project authors
//
// SPDX-License-Identifier: MIT
//
//===----------------------------------------------------------------------===//

import LibP2P
import Logging
import Testing

@testable import LibP2PMSS

@Suite("Libp2p MultiStreamSelect Tests")
struct LibP2PMSSTests {
    private func logger() -> Logger {
        Logger(label: "LibP2PMSSTests")
    }

    @Test func testListenerRespondsToMSSCodec() throws {
        let negotiator = Negotiator(
            mode: .listener,
            handledProtocols: ["/mplex/6.7.0"],
            logger: logger()
        )

        let initMessage = negotiator.initialize(compact: false)
        #expect(initMessage == nil)

        // Simulate initiator sending /multistream/1.0.0
        let codec = Array("/multistream/1.0.0".utf8)
        let response = negotiator.consumeMessage(encodeList([codec]))

        guard case .stillNegotiating(let bytes) = response else {
            Issue.record("Expected stillNegotiating response, got \(response)")
            return
        }

        let decoded = decodeList(bytes ?? [])
        #expect(decoded.count == 1)
        #expect(String(bytes: decoded[0], encoding: .utf8) == "/multistream/1.0.0")
        #expect(negotiator.currentState == .speaksMSS)
    }

    @Test func testListenerNegotiatesProtocol() throws {
        let negotiator = Negotiator(
            mode: .listener,
            handledProtocols: ["/mplex/6.7.0", "/yamux/1.0.0"],
            logger: logger()
        )

        _ = negotiator.initialize(compact: false)
        _ = negotiator.consumeMessage(encodeList([Array("/multistream/1.0.0".utf8)]))

        let response = negotiator.consumeMessage(encodeList([Array("/yamux/1.0.0".utf8)]))

        guard case .negotiated(let proto, let bytes, _) = response else {
            Issue.record("Expected negotiated response, got \(response)")
            return
        }

        #expect(proto == "/yamux/1.0.0")
        let decoded = decodeList(bytes ?? [])
        #expect(decoded.count == 1)
        #expect(String(bytes: decoded[0], encoding: .utf8) == "/yamux/1.0.0")
    }

    @Test func testListenerRespondsNAForUnsupportedProtocol() throws {
        let negotiator = Negotiator(
            mode: .listener,
            handledProtocols: ["/mplex/6.7.0"],
            logger: logger()
        )

        _ = negotiator.initialize(compact: false)
        _ = negotiator.consumeMessage(encodeList([Array("/multistream/1.0.0".utf8)]))

        let response = negotiator.consumeMessage(encodeList([Array("/unsupported/1.0.0".utf8)]))

        guard case .stillNegotiating(let bytes) = response else {
            Issue.record("Expected stillNegotiating response, got \(response)")
            return
        }

        let decoded = decodeList(bytes ?? [])
        #expect(decoded.count == 1)
        #expect(String(bytes: decoded[0], encoding: .utf8) == "na")
    }

    @Test func testInitiatorNegotiatesProtocol() throws {
        let negotiator = Negotiator(
            mode: .initiator,
            handledProtocols: ["/yamux/1.0.0"],
            logger: logger()
        )

        let initMessage = negotiator.initialize(compact: false)
        #expect(initMessage != nil)
        #expect(negotiator.currentState == .initialized)

        // Receive MSS codec response from listener
        let response1 = negotiator.consumeMessage(encodeList([Array("/multistream/1.0.0".utf8)]))
        guard case .stillNegotiating(let bytes1) = response1 else {
            Issue.record("Expected stillNegotiating response, got \(response1)")
            return
        }

        let decoded1 = decodeList(bytes1 ?? [])
        #expect(decoded1.count == 1)
        #expect(String(bytes: decoded1[0], encoding: .utf8) == "/yamux/1.0.0")
        #expect(negotiator.currentState == .speaksMSS)

        // Receive protocol confirmation from listener
        let response2 = negotiator.consumeMessage(encodeList([Array("/yamux/1.0.0".utf8)]))
        guard case .negotiated(let proto, _, _) = response2 else {
            Issue.record("Expected negotiated response, got \(response2)")
            return
        }

        #expect(proto == "/yamux/1.0.0")
    }

    @Test func testInitiatorCompactMode() throws {
        let negotiator = Negotiator(
            mode: .initiator,
            handledProtocols: ["/mplex/6.7.0"],
            logger: logger()
        )

        let initMessage = negotiator.initialize(compact: true)
        #expect(initMessage != nil)

        let decoded = decodeList(initMessage!)
        #expect(decoded.count == 2)
        #expect(String(bytes: decoded[0], encoding: .utf8) == "/multistream/1.0.0")
        #expect(String(bytes: decoded[1], encoding: .utf8) == "/mplex/6.7.0")
    }

    @Test func testListenerHandlesLSRequest() throws {
        let negotiator = Negotiator(
            mode: .listener,
            handledProtocols: ["/mplex/6.7.0", "/yamux/1.0.0"],
            logger: logger()
        )

        _ = negotiator.initialize(compact: false)
        _ = negotiator.consumeMessage(encodeList([Array("/multistream/1.0.0".utf8)]))

        let response = negotiator.consumeMessage(encodeList([Array("ls".utf8)]))

        guard case .stillNegotiating(let bytes) = response else {
            Issue.record("Expected stillNegotiating response, got \(response)")
            return
        }

        // The response is a single varint-prefixed message whose payload is an encodeList of protocols.
        let outer = decodeList(bytes ?? [])
        #expect(outer.count == 1)
        let inner = decodeList(outer.first ?? [])
        #expect(inner.count == 2)
        let strings = inner.compactMap { String(bytes: $0, encoding: .utf8) }
        #expect(strings.contains("/mplex/6.7.0"))
        #expect(strings.contains("/yamux/1.0.0"))
    }

    // MARK: - Helpers

    private func encode(_ bytes: [UInt8]) -> [UInt8] {
        let lengthPrefix = encodeUVarInt(UInt64(bytes.count + 1))
        return lengthPrefix + bytes + [0x0a]
    }

    private func encodeList(_ protos: [[UInt8]]) -> [UInt8] {
        protos.flatMap(encode)
    }

    private func decodeList(_ data: [UInt8]) -> [[UInt8]] {
        var messages: [[UInt8]] = []
        var remaining = data

        while !remaining.isEmpty {
            let r = decodeUVarInt(remaining)
            guard r.bytesRead > 0, r.value > 1 else { break }
            let endIndex = (r.bytesRead + Int(r.value)) - 1
            guard endIndex < remaining.count, remaining[endIndex] == 0x0a else { break }
            let message = Array(remaining[r.bytesRead..<endIndex])
            messages.append(message)
            remaining = Array(remaining[(endIndex + 1)...])
        }

        return messages
    }

    private func encodeUVarInt(_ value: UInt64) -> [UInt8] {
        var value = value
        var bytes: [UInt8] = []
        while value >= 0x80 {
            bytes.append(UInt8(value & 0x7f) | 0x80)
            value >>= 7
        }
        bytes.append(UInt8(value))
        return bytes
    }

    private func decodeUVarInt(_ bytes: [UInt8]) -> (value: UInt64, bytesRead: Int) {
        var value: UInt64 = 0
        var shift: UInt64 = 0
        var index = 0
        while index < bytes.count {
            let byte = bytes[index]
            value |= UInt64(byte & 0x7f) << shift
            index += 1
            if byte & 0x80 == 0 {
                return (value, index)
            }
            shift += 7
            if shift > 63 { break }
        }
        return (0, 0)
    }
}
