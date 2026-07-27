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

import Dispatch
@preconcurrency import DNS
import LibP2P
import NIOConcurrencyHelpers
import dnssd

/// DNSAddr
/// Is a protocol used by libp2p to resolve `Multiaddr`s that use the `dnsaddr` protocol.
/// - [Specification](https://github.com/multiformats/multiaddr/blob/master/protocols/DNSADDR.md)
/// ```swift
/// // When configuring your libp2p instance
/// app.resolvers.use(.dnsaddr)
/// ...
/// // Later you can call resolve on any app or req object
/// app.resolve(ma).map { resolvedMultiaddr in ... }
/// req.resolve(ma, for: [.ip4, .tcp]).map { resolvedMultiaddr in ... }
/// ```
public final class DNSAddr: AddressResolver, LifecycleHandler {

    public static let key: String = "DNSADDR"

    public enum Errors: Error {
        case invalidMultiaddr
        case noMatchingHostFound
        case dnsServiceFailed(DNSServiceErrorType)
    }

    private let application: Application
    private let eventLoop: EventLoop
    private let logger: Logger
    private let uuid: UUID
    private let host: SocketAddress?

    init(application: Application, host: SocketAddress? = nil) {
        self.application = application
        self.eventLoop = application.eventLoopGroup.next()
        self.uuid = UUID()
        var logger = application.logger
        logger[metadataKey: DNSAddr.key] = .string("[\(uuid.uuidString.prefix(5))]")
        self.logger = logger
        self.host = host
    }

    private final class QueryState: @unchecked Sendable {
        let eventLoop: EventLoop
        let promise: EventLoopPromise<[Multiaddr]>
        let expectedPeerID: PeerID?
        var service: DNSServiceRef?
        var addresses: [Multiaddr] = []
        var finished = false

        init(eventLoop: EventLoop, promise: EventLoopPromise<[Multiaddr]>, expectedPeerID: PeerID?) {
            self.eventLoop = eventLoop
            self.promise = promise
            self.expectedPeerID = expectedPeerID
        }
    }

    public func willBoot(_ application: Application) throws {
        self.logger.trace("Initializing")
    }

    public func shutdown(_ application: Application) {
        self.logger.trace("Shutting Down")
    }

    /// Provided a Multiaddr that uses the `dnsaddr` codec, this method will attempt to resolve the domain into it's underyling ip address.
    /// - Note: this method is not recusrive, it will only resolve `dnsaddr`s that are at most 2 layers deep.
    public func resolve(
        multiaddr: Multiaddr,
        for codecs: Set<MultiaddrProtocol> = [.ip4, .tcp]
    ) -> EventLoopFuture<Multiaddr?> {
        self.resolve(multiaddr: multiaddr).map { addresses -> Multiaddr? in
            guard let addresses else { return nil }
            return addresses.first(where: { addy in
                Set(addy.addresses.map { $0.codec }).isSuperset(of: codecs)
            })
        }
    }

    /// Provided a Multiaddr that uses the `dnsaddr` codec, this method will attempt to resolve the domain into it's underyling ip addresses.
    /// - Note: this method is not recusrive, it will only resolve `dnsaddr`s that are at most 2 layers deep.
    public func resolve(multiaddr ma: Multiaddr) -> EventLoopFuture<[Multiaddr]?> {
        let promise = self.eventLoop.makePromise(of: [Multiaddr]?.self)

        self.eventLoop.execute {

            // Only proceed if the Mutliaddr is a dnsaddr proto and has a p2p peerID present
            guard ma.addresses.first?.codec == .dnsaddr, let domain = ma.addresses.first?.addr,
                let pid = try? ma.getPeerID()
            else { return promise.fail(Errors.invalidMultiaddr) }

            // Peform the first resolution
            let dnsAddrPrefix = "_dnsaddr."
            let _ = self.resolveAddresses(forHost: dnsAddrPrefix + domain).map { resovledAddresses in

                // This might resolve to a few different Multiaddr, but if we cant find a MA with the same peerID we bail...
                guard let host = resovledAddresses.first(where: { (try? $0.getPeerID()) == pid }) else {
                    return promise.fail(Errors.noMatchingHostFound)
                }

                // If the resolved address is another dnsaddr, attempt to resolve it...
                guard host.addresses.first?.codec == .dnsaddr, let domain2 = host.addresses.first?.addr,
                    let pid2 = try? host.getPeerID(), pid == pid2
                else {
                    return promise.succeed([host])
                }

                let _ = self.resolveAddresses(forHost: dnsAddrPrefix + domain2, enforcingPeerID: pid).map {
                    resovledAddresses2 in
                    if !resovledAddresses2.isEmpty {
                        return promise.succeed(resovledAddresses2)
                    } else {
                        return promise.fail(Errors.noMatchingHostFound)
                    }
                }
            }
        }

        return promise.futureResult
    }

    private func resolveAddresses(forHost host: String, enforcingPeerID: PeerID? = nil) -> EventLoopFuture<[Multiaddr]> {
        let promise = self.eventLoop.makePromise(of: [Multiaddr].self)
        let state = QueryState(eventLoop: self.eventLoop, promise: promise, expectedPeerID: enforcingPeerID)
        let context = Unmanaged.passRetained(state).toOpaque()
        let queue = DispatchQueue(label: "LibP2PDNSAddr.\(self.uuid.uuidString)")

        var service: DNSServiceRef?
        let callback: DNSServiceQueryRecordReply = { _, flags, _, errorCode, _, rrtype, _, rdlen, rdata, _, context in
            guard let context else { return }
            let state = Unmanaged<QueryState>.fromOpaque(context).takeUnretainedValue()
            guard !state.finished else { return }

            if errorCode != kDNSServiceErr_NoError {
                Self.completeQuery(state: state, context: context, result: .failure(Errors.dnsServiceFailed(errorCode)))
                return
            }

            guard rrtype == UInt16(kDNSServiceType_TXT), let rdata, rdlen > 0 else {
                if (flags & kDNSServiceFlagsMoreComing) == 0 {
                    Self.completeQuery(
                        state: state,
                        context: context,
                        result: state.addresses.isEmpty ? .failure(Errors.noMatchingHostFound) : .success(Array(Set(state.addresses)))
                    )
                }
                return
            }

            let records = Self.multiaddrs(
                fromTXTRecordBytes: UnsafeRawBufferPointer(start: rdata, count: Int(rdlen)),
                enforcingPeerID: state.expectedPeerID
            )
            if !records.isEmpty {
                state.addresses.append(contentsOf: records)
            }

            if (flags & kDNSServiceFlagsMoreComing) == 0 {
                Self.completeQuery(
                    state: state,
                    context: context,
                    result: state.addresses.isEmpty ? .failure(Errors.noMatchingHostFound) : .success(Array(Set(state.addresses)))
                )
            }
        }

        let error = host.withCString { cHost in
            DNSServiceQueryRecord(
                &service,
                0,
                0,
                cHost,
                UInt16(kDNSServiceType_TXT),
                UInt16(kDNSServiceClass_IN),
                callback,
                context
            )
        }

        guard error == kDNSServiceErr_NoError, let service else {
            Unmanaged<QueryState>.fromOpaque(context).release()
            promise.fail(Errors.dnsServiceFailed(error))
            return promise.futureResult
        }

        state.service = service

        let schedulingError = DNSServiceSetDispatchQueue(service, queue)
        guard schedulingError == kDNSServiceErr_NoError else {
            DNSServiceRefDeallocate(service)
            state.service = nil
            Unmanaged<QueryState>.fromOpaque(context).release()
            promise.fail(Errors.dnsServiceFailed(schedulingError))
            return promise.futureResult
        }

        return promise.futureResult
    }

    internal static func multiaddrs(fromTXTRecordBytes bytes: UnsafeRawBufferPointer, enforcingPeerID: PeerID? = nil) -> [Multiaddr] {
        guard let baseAddress = bytes.baseAddress, bytes.count > 0 else { return [] }

        var results: [Multiaddr] = []
        let count = TXTRecordGetCount(UInt16(bytes.count), baseAddress)

        for index in 0..<count {
            var keyBuffer = [CChar](repeating: 0, count: 256)
            var valueLength: UInt8 = 0
            var valuePointer: UnsafeRawPointer?

            let status = TXTRecordGetItemAtIndex(
                UInt16(bytes.count),
                baseAddress,
                index,
                UInt16(keyBuffer.count),
                &keyBuffer,
                &valueLength,
                &valuePointer
            )

            guard status == kDNSServiceErr_NoError, String(cString: keyBuffer) == "dnsaddr", let valuePointer else { continue }

            let valueData = Data(bytes: valuePointer, count: Int(valueLength))
            guard let value = String(data: valueData, encoding: .utf8), let multiaddr = try? Multiaddr(value) else { continue }

            if let enforcingPeerID {
                guard let peerID = try? multiaddr.getPeerID(), peerID == enforcingPeerID else { continue }
            }

            results.append(multiaddr)
        }

        return results
    }

    private static func completeQuery(state: QueryState, context: UnsafeMutableRawPointer, result: Result<[Multiaddr], Error>) {
        guard !state.finished else { return }
        state.finished = true

        state.eventLoop.execute {
            switch result {
            case .success(let addresses):
                state.promise.succeed(addresses)
            case .failure(let error):
                state.promise.fail(error)
            }
        }

        if let service = state.service {
            DNSServiceRefDeallocate(service)
            state.service = nil
        }

        Unmanaged<QueryState>.fromOpaque(context).release()
    }
}
