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

import Foundation
import LibP2P
import Multiaddr
import NIOConcurrencyHelpers
import NIOCore
import VarInt

public enum AutoNATStatus: String, Sendable {
    case unknown
    case publicReachable
    case privateBehindNAT
}

enum AutoNATWire {
    static let protocolID = "/libp2p/autonat/1.0.0"

    enum ResponseStatus: UInt64, Sendable {
        case ok = 0
        case eDialError = 100
        case eDialRefused = 101
        case eBadRequest = 200
        case eInternalError = 300
    }

    struct DialRequest: Sendable {
        let token: [UInt8]
        let addresses: [Multiaddr]
    }

    struct DialResponse: Sendable {
        let token: [UInt8]
        let status: ResponseStatus
        let dialedAddress: Multiaddr?
    }

    static func encode(_ request: DialRequest) throws -> ByteBuffer {
        let bufferAllocator = ByteBufferAllocator()
        var buffer = bufferAllocator.buffer(capacity: 256)
        buffer.writeBytes(putUVarInt(UInt64(request.token.count)))
        buffer.writeBytes(request.token)
        buffer.writeBytes(putUVarInt(UInt64(request.addresses.count)))
        for address in request.addresses {
            let bytes = try address.binaryPacked()
            buffer.writeBytes(putUVarInt(UInt64(bytes.count)))
            buffer.writeBytes(bytes)
        }
        return buffer
    }

    static func encode(_ response: DialResponse) throws -> ByteBuffer {
        let bufferAllocator = ByteBufferAllocator()
        var buffer = bufferAllocator.buffer(capacity: 256)
        buffer.writeBytes(putUVarInt(UInt64(response.token.count)))
        buffer.writeBytes(response.token)
        buffer.writeBytes(putUVarInt(response.status.rawValue))
        if let dialedAddress = response.dialedAddress {
            let bytes = try dialedAddress.binaryPacked()
            buffer.writeInteger(UInt8(1))
            buffer.writeBytes(putUVarInt(UInt64(bytes.count)))
            buffer.writeBytes(bytes)
        } else {
            buffer.writeInteger(UInt8(0))
        }
        return buffer
    }

    static func decodeRequest(_ buffer: ByteBuffer) throws -> DialRequest {
        var bytes = Array(buffer.readableBytesView)
        let (tokenLength, tokenLengthBytes) = uVarInt(bytes)
        guard tokenLengthBytes > 0 else { throw Errors.invalidMessage }
        bytes.removeFirst(tokenLengthBytes)
        guard bytes.count >= Int(tokenLength) else { throw Errors.invalidMessage }
        let token = Array(bytes.prefix(Int(tokenLength)))
        bytes.removeFirst(Int(tokenLength))

        let (addressCount, addressCountBytes) = uVarInt(bytes)
        guard addressCountBytes > 0 else { throw Errors.invalidMessage }
        bytes.removeFirst(addressCountBytes)

        var addresses: [Multiaddr] = []
        for _ in 0..<addressCount {
            let (length, lengthBytes) = uVarInt(bytes)
            guard lengthBytes > 0 else { throw Errors.invalidMessage }
            bytes.removeFirst(lengthBytes)
            guard bytes.count >= Int(length) else { throw Errors.invalidMessage }
            let addressBytes = Data(bytes.prefix(Int(length)))
            bytes.removeFirst(Int(length))
            addresses.append(try Multiaddr(addressBytes))
        }
        return DialRequest(token: token, addresses: addresses)
    }

    static func decodeResponse(_ buffer: ByteBuffer) throws -> DialResponse {
        var bytes = Array(buffer.readableBytesView)
        let (tokenLength, tokenLengthBytes) = uVarInt(bytes)
        guard tokenLengthBytes > 0 else { throw Errors.invalidMessage }
        bytes.removeFirst(tokenLengthBytes)
        guard bytes.count >= Int(tokenLength) else { throw Errors.invalidMessage }
        let token = Array(bytes.prefix(Int(tokenLength)))
        bytes.removeFirst(Int(tokenLength))

        let (statusValue, statusBytes) = uVarInt(bytes)
        guard statusBytes > 0, let status = ResponseStatus(rawValue: statusValue) else {
            throw Errors.invalidMessage
        }
        bytes.removeFirst(statusBytes)

        guard let hasAddress = bytes.first else { throw Errors.invalidMessage }
        bytes.removeFirst()

        if hasAddress == 1 {
            let (length, lengthBytes) = uVarInt(bytes)
            guard lengthBytes > 0 else { throw Errors.invalidMessage }
            bytes.removeFirst(lengthBytes)
            guard bytes.count >= Int(length) else { throw Errors.invalidMessage }
            let addressBytes = Data(bytes.prefix(Int(length)))
            return DialResponse(token: token, status: status, dialedAddress: try Multiaddr(addressBytes))
        }

        return DialResponse(token: token, status: status, dialedAddress: nil)
    }

    enum Errors: Error {
        case invalidMessage
    }
}

public final class AutoNATCoordinator: @unchecked Sendable {
    struct PendingProbe {
        let token: [UInt8]
        let peer: PeerID
        let startTime: DispatchTime
        let promise: EventLoopPromise<AutoNATStatus>
        let addresses: [Multiaddr]
    }

    private let application: Application
    private let queue = DispatchQueue(label: "LibP2PAutoNAT.probes")
    private var pending: [String: PendingProbe] = [:]
    private var _status: AutoNATStatus = .unknown

    public init(application: Application) {
        self.application = application
    }

    public var status: AutoNATStatus {
        self.queue.sync { self._status }
    }

    private func setStatus(_ status: AutoNATStatus) {
        self.queue.sync { self._status = status }
    }

    func install() {
        self.application.events.on(self, event: .identifiedPeer(self.onIdentifiedPeer(_:)))
        self.application.group("libp2p") { libp2p in
            libp2p.group("autonat", handlers: [.varIntLengthPrefixed]) { autonat in
                autonat.on("1.0.0", handlers: [.varIntLengthPrefixed]) { req in
                    try await self.handle(req)
                }
            }
        }
        self.application.logger.notice("Installed AutoNAT signal source")
    }

    public func probe(peer: PeerID) -> EventLoopFuture<AutoNATStatus> {
        let el = self.application.eventLoopGroup.any()
        let promise = el.makePromise(of: AutoNATStatus.self)
        let token = Array(UUID().uuidString.utf8)
        let addresses = self.candidateDialbackAddresses()
        var shouldOpenStream = false
        var existingFuture: EventLoopFuture<AutoNATStatus>?
        self.queue.sync {
            if let existing = self.pending[peer.b58String] {
                existingFuture = existing.promise.futureResult
                return
            }
            self.pending[peer.b58String] = PendingProbe(
                token: token,
                peer: peer,
                startTime: .now(),
                promise: promise,
                addresses: addresses
            )
            shouldOpenStream = true
        }

        guard shouldOpenStream else {
            return existingFuture ?? promise.futureResult
        }

        do {
            try self.application.newStream(to: peer, forProtocol: AutoNATWire.protocolID)
        } catch {
            self.queue.sync {
                _ = self.pending.removeValue(forKey: peer.b58String)
            }
            promise.fail(error)
        }

        return promise.futureResult
    }

    private func onIdentifiedPeer(_ identifiedPeer: IdentifiedPeer) {
        self.application.peers.getProtocols(forPeer: identifiedPeer.peer, on: self.application.eventLoopGroup.any()).whenSuccess { protocols in
            guard protocols.contains(where: { $0.stringValue == AutoNATWire.protocolID }) else { return }
            _ = self.probe(peer: identifiedPeer.peer)
        }
    }

    private func candidateDialbackAddresses() -> [Multiaddr] {
        let addresses = self.application.transports.stripInternalAddresses(self.application.peerInfo.addresses)
        return addresses.compactMap { address in
            (try? address.encapsulate(proto: .p2p, address: self.application.peerID.b58String)) ?? address
        }
    }

    private func handle(_ req: Request) async throws -> Response<ByteBuffer> {
        switch req.streamDirection {
        case .inbound:
            switch req.event {
            case .ready:
                return .stayOpen
            case .data(let payload):
                let request = try AutoNATWire.decodeRequest(payload)
                let response = try await self.answerDialRequest(request)
                return .respondThenClose(try AutoNATWire.encode(response))
            default:
                return .close
            }

        case .outbound:
            switch req.event {
            case .ready:
                guard
                    let peer = req.remotePeer,
                    let pending = self.queue.sync(execute: { self.pending[peer.b58String] })
                else {
                    return .close
                }
                let request = AutoNATWire.DialRequest(token: pending.token, addresses: pending.addresses)
                return .respond(try AutoNATWire.encode(request))

            case .data(let payload):
                let response = try AutoNATWire.decodeResponse(payload)
                let pending = self.queue.sync { self.pending.removeValue(forKey: req.remotePeer?.b58String ?? "") }
                guard let pending, pending.token == response.token else { return .close }
                let status: AutoNATStatus = response.status == .ok ? .publicReachable : .privateBehindNAT
                self.setStatus(status)
                pending.promise.succeed(status)
                return .close

            default:
                return .close
            }
        }
    }

    private func answerDialRequest(_ request: AutoNATWire.DialRequest) async throws -> AutoNATWire.DialResponse {
        for address in request.addresses {
            guard let transport = try? self.application.transports.findBest(forMultiaddr: address) else { continue }
            do {
                let connection = try await transport.dial(address: address).get()
                _ = connection.close()
                self.setStatus(.publicReachable)
                return AutoNATWire.DialResponse(token: request.token, status: .ok, dialedAddress: address)
            } catch {
                self.application.logger.debug("AutoNAT dialback failed for \(address): \(error)")
            }
        }
        self.setStatus(.privateBehindNAT)
        return AutoNATWire.DialResponse(token: request.token, status: .eDialError, dialedAddress: nil)
    }
}

extension AutoNATCoordinator {
    public static let protocolID = AutoNATWire.protocolID
}
