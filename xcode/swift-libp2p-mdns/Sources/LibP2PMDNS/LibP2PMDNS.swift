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

@preconcurrency import DNS
import LibP2P
import NIOConcurrencyHelpers
import dnssd

final public class MulticastPeerDiscovery: Discovery, PeerDiscovery, LifecycleHandler, @unchecked Sendable {
    public static let key: String = "MulticastDNS"
    public typealias PeerDiscoveredEvent = (@Sendable (PeerInfo) -> Void)

    public var onPeerDiscovered: PeerDiscoveredEvent? {
        get { _onPeerDiscovered.withLockedValue { $0 } }
        set { _onPeerDiscovered.withLockedValue { $0 = newValue } }
    }
    private let _onPeerDiscovered: NIOLockedValueBox<PeerDiscoveredEvent?>

    public var on: (@Sendable (PeerDiscoveryEvent) -> Void)? {
        get { _on.withLockedValue { $0 } }
        set { _on.withLockedValue { $0 = newValue } }
    }
    public let _on: NIOLockedValueBox<(@Sendable (PeerDiscoveryEvent) -> Void)?>

    public var listeningAddresses: [Multiaddr] {
        get { _listeningAddresses.withLockedValue { $0 } }
    }
    private let _listeningAddresses: NIOLockedValueBox<[Multiaddr]> = .init([])

    public var state: ServiceLifecycleState {
        get { _state.withLockedValue { $0 } }
    }
    private let _state: NIOLockedValueBox<ServiceLifecycleState>

    public let eventLoop: EventLoop

    private let application: Application
    private let logger: Logger
    //private let interfaceAddress:String
    private let interfaceAddressV4: SocketAddress
    private let interfaceAddressV6: SocketAddress

    // In Memory Discovery Cache
    private var discovered: [String: PeerInfo] {
        get { _discovered.withLockedValue { $0 } }
        set { _discovered.withLockedValue { $0 = newValue } }
    }
    private let _discovered: NIOLockedValueBox<[String: PeerInfo]>

    private let peerID: PeerID

    private var group: EventLoopGroup? {
        get { _group.withLockedValue { $0 } }
        set { _group.withLockedValue { $0 = newValue } }
    }
    private let _group: NIOLockedValueBox<EventLoopGroup?> = .init(nil)

    private var targetDevice: NIONetworkDevice? {
        get { _targetDevice.withLockedValue { $0 } }
        set { _targetDevice.withLockedValue { $0 = newValue } }
    }
    private let _targetDevice: NIOLockedValueBox<NIONetworkDevice?> = .init(nil)

    private let multicastGroup: SocketAddress

    private var datagramBootstrap: DatagramBootstrap {
        get { _datagramBootstrap.withLockedValue { $0! } }
        set { _datagramBootstrap.withLockedValue { $0 = newValue } }
    }
    private let _datagramBootstrap: NIOLockedValueBox<DatagramBootstrap?>

    private var mdnsHost: Channel? {
        get { _mdnsHost.withLockedValue { $0 } }
        set { _mdnsHost.withLockedValue { $0 = newValue } }
    }
    private let _mdnsHost: NIOLockedValueBox<Channel?>

    private var registeredServices: [String: DNSServiceRef] {
        get { _registeredServices.withLockedValue { $0 } }
        set { _registeredServices.withLockedValue { $0 = newValue } }
    }
    private let _registeredServices: NIOLockedValueBox<[String: DNSServiceRef]> = .init([:])

    private typealias OutstandingQuery = (query: Question, callback: @Sendable (DNS.Message) -> Void)
    private var outstandingQueries: [OutstandingQuery] {
        get { _outstandingQueries.withLockedValue { $0 } }
        set { _outstandingQueries.withLockedValue { $0 = newValue } }
    }
    private let _outstandingQueries: NIOLockedValueBox<[OutstandingQuery]> = .init([])

    private var serviceDiscoveryTask: RepeatedTask? {
        get { _serviceDiscoveryTask.withLockedValue { $0 } }
        set { _serviceDiscoveryTask.withLockedValue { $0 = newValue } }
    }
    private let _serviceDiscoveryTask: NIOLockedValueBox<RepeatedTask?> = .init(nil)

    init(
        app: Application,
        interfaceAddress: SocketAddress? = nil,
        multicastGroup: SocketAddress = try! SocketAddress(ipAddress: "224.0.0.251", port: 5353)
    ) {
        //init(on loop:EventLoop, peerID:PeerID, interfaceAddress:String, multicastGroup:String = "224.0.0.251:5353") {

        self.application = app
        self.peerID = app.peerID

        /// Configure our logger with some metadata
        var logger = app.logger
        logger[metadataKey: "mDNS"] = .string("\(peerID.shortDescription)")
        self.logger = logger

        if let ia = interfaceAddress {
            if ia.protocol == .inet {
                self.interfaceAddressV4 = ia
                self.interfaceAddressV6 = try! MulticastPeerDiscovery.defaultInterfaceAddress(codec: .ip6)
            } else if ia.protocol == .inet6 {
                self.interfaceAddressV6 = ia
                self.interfaceAddressV4 = try! MulticastPeerDiscovery.defaultInterfaceAddress(codec: .ip4)
            } else {
                self.interfaceAddressV4 = try! MulticastPeerDiscovery.defaultInterfaceAddress(codec: .ip4)
                self.interfaceAddressV6 = try! MulticastPeerDiscovery.defaultInterfaceAddress(codec: .ip6)
            }
        } else {
            self.interfaceAddressV4 = try! MulticastPeerDiscovery.defaultInterfaceAddress(codec: .ip4)
            self.interfaceAddressV6 = try! MulticastPeerDiscovery.defaultInterfaceAddress(codec: .ip6)
        }
        self.eventLoop = app.eventLoopGroup.next()
        self._state = .init(.stopped)
        self._discovered = .init([:])
        self._onPeerDiscovered = .init(nil)
        self._on = .init(nil)
        self._datagramBootstrap = .init(nil)
        self._mdnsHost = .init(nil)

        ///self.multicastGroup = try! SocketAddress(ipAddress: "224.1.0.26", port: 7654)
        //let parts = multicastGroup.split(separator: ":")
        //self.multicastGroup = try! SocketAddress(ipAddress: String(parts[0]), port: Int(parts[1])!)
        self.multicastGroup = multicastGroup

        self.logger.trace("Initialized")
        self.logger.notice("mDNS ipV4: \(interfaceAddressV4)")
        self.logger.notice("mDNS ipV6: \(interfaceAddressV6)")
    }

    private static func interfaceAddressDevice(_ socketAddress: SocketAddress) throws -> NIONetworkDevice? {
        var deviceMatch: NIONetworkDevice? = nil
        for device in try! System.enumerateDevices() {
            if device.address == socketAddress {
                deviceMatch = device
                break
            }
        }

        return deviceMatch
    }

    private static func defaultInterfaceAddress(
        codec: MultiaddrProtocol,
        deviceName: String? = nil
    ) throws -> SocketAddress {
        guard codec == .ip4 || codec == .ip6 else {
            return try! SocketAddress(ipAddress: "127.0.0.1", port: 0)
        }
        if let address = try System.enumerateDevices().compactMap({ device -> SocketAddress? in
            guard device.address != nil else { return nil }
            if let deviceName, device.name != deviceName { return nil }
            guard let ma = try? device.address?.toMultiaddr() else { return nil }

            guard let tcp = ma.tcpAddress else { return nil }

            switch codec {
            case .ip4:
                if tcp.ip4 { return device.address }
            case .ip6:
                if tcp.ip4 == false { return device.address }
            default:
                break
            }

            return nil

        }).first {
            return address
        }

        return try! SocketAddress(ipAddress: codec == .ip4 ? "127.0.0.1" : "::1", port: 0)
    }

    private static func interfaceAddress(
        forCodec codec: MultiaddrProtocol,
        onDevice targetDevice: NIONetworkDevice
    ) throws -> SocketAddress? {
        guard codec == .ip4 || codec == .ip6 else { return nil }
        return try System.enumerateDevices().compactMap({ device -> SocketAddress? in
            guard device == targetDevice && device.address != nil else { return nil }
            guard let ma = try? device.address?.toMultiaddr() else { return nil }

            guard let tcp = ma.tcpAddress else { return nil }

            switch codec {
            case .ip4:
                if tcp.ip4 { return device.address }
            case .ip6:
                if tcp.ip4 == false { return device.address }
            default:
                break
            }

            return nil

        }).first
    }

    deinit {
        self.logger.trace("Deinitialized")
    }

    public func start() throws {
        switch self.state {
        case .stopped:
            /// Switch our state to .starting
            self._state.withLockedValue { $0 = .starting }

            /// Update our logging state
            //self.logger.logLevel = application.logger.logLevel

            /// Create an event loop group for our UDP server to use
            self.group = MultiThreadedEventLoopGroup(numberOfThreads: 1)

            /// Instantiate our UPD Server and join the mDNS multicast group
            self.mdnsHost = try self.makeUDPMulticastHost(
                self.group!,
                interfaceAddress: self.interfaceAddressV4,
                onMulticastGroup: self.multicastGroup
            )

            /// Bootstrap the listening addresses from the application
            self._listeningAddresses.withLockedValue { $0 = self.application.listenAddresses }

            /// Register for listen events (this is when new hosts / servers get spun up on a particular port)
            /// Take this opportunity to add the address to our list of listeningAddresses (which gets included in our service registration)
            //SwiftEventBus.onBackgroundThread(self, event: .listen( { pid, newAddress in
            application.events.on(
                self,
                event: .listen({ [unowned self] pid, newAddress in
                    guard pid == self.peerID.b58String else { return }
                    let _ = self.eventLoop.submit {
                        if !self.listeningAddresses.contains(where: { $0 == newAddress }) {
                            self.logger.debug("Notified of new listening address -> \(newAddress)")
                            self.logger.debug("Updating mDNS Service Registration")
                            self._listeningAddresses.withLockedValue { $0.append(newAddress) }
                        }
                        /// Do we need to re-register our service to account for the change in addresses...
                    }
                })
            )

            /// Register for listenClosed events (this is when host / server is closed and is no longer listening for inbound traffic)
            /// Take this opportunity to remove the address from our list of listeningAddresses (which gets included in our service registration)
            //SwiftEventBus.onBackgroundThread(self, event: .listenClosed( { pid, oldAddress in
            application.events.on(
                self,
                event: .listenClosed({ [unowned self] pid, oldAddress in
                    guard pid == self.peerID.b58String else { return }
                    let _ = self.eventLoop.submit {
                        if self.listeningAddresses.contains(where: { $0 == oldAddress }) {
                            self.logger.debug("Notified of removed listening address -> \(oldAddress)")
                            self.logger.debug("Updating mDNS Service Registration")
                            self._listeningAddresses.withLockedValue { $0.removeAll(where: { $0 == oldAddress }) }
                        }
                        /// Do we need to re-register our service to account for the change in addresses...
                    }
                })
            )

            /// Register the main libp2p discovery service
            let _ = try self.registerP2PService().wait()
            //let _ = try self.registerService(name: self.peerID.b58String).wait()

            /// Scan for libp2p services every 5 seconds
            self.serviceDiscoveryTask = self.eventLoop.scheduleRepeatedAsyncTask(
                initialDelay: .milliseconds(500),
                delay: .seconds(5),
                notifying: nil
            ) { task in
                self.logger.trace("Querying for peers")
                return self.queryForPeers().map { multiaddresses in
                    guard multiaddresses.count > 0 else {
                        self.logger.trace("No Local Peers Found")
                        return
                    }

                    let pInfos = self.multiaddressesToPeerInfos(multiaddresses)
                    for remotePeer in pInfos {
                        if self.discovered[remotePeer.peer.b58String] == nil {
                            self.logger.trace("Discovered Local Peer -> \(remotePeer)")
                            self.discovered[remotePeer.peer.b58String] = remotePeer
                            /// Notify our handler / delegate of the discovered peer...
                            self.onPeerDiscovered?(remotePeer)
                            self.on?(.onPeer(remotePeer))
                        } else {
                            self.discovered[remotePeer.peer.b58String] = remotePeer
                        }
                    }

                    //                    // - TODO: We need to itterate over all addresses and create a set of unique peerIDs
                    //                    if let pidString = multiaddresses.first?.getPeerID(), let pid = try? PeerID(cid: pidString) {
                    //                        let multiaddrs = multiaddresses.fixSameHostAddresses(ip4: self.interfaceAddressV4.ipAddress!)
                    //                        let remotePeer = PeerInfo(peer: pid, addresses: multiaddrs)
                    //                        /// Add / update the PeerInfo data in our discovered peers cache
                    //                        if self.discovered[pidString] == nil {
                    //                            self.logger.trace("Discovered Local Peer -> \(remotePeer)")
                    //                            self.discovered[pidString] = remotePeer
                    //                            /// Notify our handler / delegate of the discovered peer...
                    //                            self.onPeerDiscovered?(remotePeer)
                    //                            self.on?(.onPeer(remotePeer))
                    //                        } else {
                    //                            self.discovered[pidString] = remotePeer
                    //                        }
                    //                    } else {
                    //                        self.logger.trace("Unable to infer/extract PeerID info from mDNS service record. Passing along the multiaddresses...")
                    //                        multiaddresses.forEach { self.on?(.onPotentialPeer( $0 )) }
                    //                    }
                }
            }

            /// Switch our state to started
            self._state.withLockedValue { $0 = .started }
            self.logger.trace("The mDNS service has started!")

        case .starting:
            self.logger.trace("The mDNS service is in the process of starting... give it a second... :P")

        case .stopping:
            self.logger.warning(
                "The mDNS service is attempting to stop, please wait until the service has fully stopped before attempting to start"
            )

        case .started:
            self.logger.trace("The mDNS service is already running")
        }
    }

    public func stop() throws {
        switch self.state {
        case .stopped:
            self.logger.trace("The mDNS service is already stopped!")

        case .stopping:
            self.logger.warning("The mDNS service is attempting to stop, please wait for it to finish")

        case .starting, .started:
            self._state.withLockedValue { $0 = .stopping }

            /// Cancel the service discovery task
            self.serviceDiscoveryTask?.cancel()

            /// Unregister any services we have registered
            while let (name, service) = self.registeredServices.popFirst() {
                self.logger.trace("Unregistering Service: \(name)")
                DNSServiceRefDeallocate(service)
            }

            /// Tear down the Multicast UDP Channel
            try self.mdnsHost?.close().wait()

            /// Tear down the Event Loop Group
            try self.group?.syncShutdownGracefully()

            /// Unregister ourself from our EventBus
            self.application.events.unregister(self)

            /// Release
            self.mdnsHost = nil
            self.group = nil

            self.logger.trace("The mDNS Service has been shutdown")
            self._state.withLockedValue { $0 = .stopped }
        }
    }

    /// Given a list of multiaddresses, group them into unqiue PeerInfo sets...
    private func multiaddressesToPeerInfos(_ mas: [Multiaddr]) -> [PeerInfo] {
        let uniquePeers = Set(mas.compactMap { $0.getPeerIDString() })
        var pInfos: [PeerInfo] = []
        for peer in uniquePeers {
            guard let pid = try? PeerID(cid: peer) else { continue }
            pInfos.append(
                PeerInfo(
                    peer: pid,
                    addresses: mas.filter({ $0.getPeerIDString() == peer }).fixSameHostAddresses(
                        ip4: self.interfaceAddressV4.ipAddress!
                    )
                )
            )
        }
        return pInfos
    }

    public func knownPeers() -> EventLoopFuture<[PeerInfo]> {
        self.eventLoop.submit {
            self.discovered.map { $0.value }
        }
    }

    /// A function that instantiates a new UDP Multicast Datagram Bootstrap Server/Listener that joins the mDNS multicast group at 244.0.0.251:5353
    private func makeUDPMulticastHost(
        _ group: EventLoopGroup,
        interfaceAddress: SocketAddress,
        onMulticastGroup: SocketAddress
    ) throws -> Channel {
        // We allow users to specify the interface they want to use here.
        var targetDevice: NIONetworkDevice? = nil
        //if let targetAddress = try? SocketAddress(ipAddress: interfaceAddress, port: 0) {
        for device in try! System.enumerateDevices() {
            if device.address == interfaceAddress {
                targetDevice = device
                break
            }
        }

        guard let targetDevice else {
            throw Errors.noAddressForDevice
        }

        self.logger.debug("Using device: \(targetDevice)")

        // Begin by setting up the basics of the bootstrap.
        let datagramBootstrap = DatagramBootstrap(group: group)
            .channelOption(ChannelOptions.socketOption(.so_reuseaddr), value: 1)
            .channelInitializer { channel in
                self.logger.trace("UDP Channel Initializer Called")
                return channel.pipeline.addHandler(MDNSMessageEncoder()).flatMap {
                    channel.pipeline.addHandler(MDNSMessageDecoder(messageHandler: self.onMDNSMessage))
                }
            }

        // We cast our channel to MulticastChannel to obtain the multicast operations.
        let datagramChannel =
            try datagramBootstrap
            .bind(host: "0.0.0.0", port: 0)  //7658
            .flatMap { channel -> EventLoopFuture<Channel> in
                let channel = channel as! MulticastChannel
                return channel.joinGroup(onMulticastGroup, device: targetDevice).map { channel }
            }.flatMap { channel -> EventLoopFuture<Channel> in
                let provider = channel as! SocketOptionProvider

                switch targetDevice.address {
                case .some(.v4(let addr)):
                    self.logger.trace("Setting Provider v4 \(addr.address.sin_addr)")
                    return provider.setIPMulticastIF(addr.address.sin_addr).map { channel }
                case .some(.v6):
                    self.logger.trace("Setting Provider v6 \(targetDevice.interfaceIndex)")
                    return provider.setIPv6MulticastIF(CUnsignedInt(targetDevice.interfaceIndex)).map { channel }
                case .some(.unixDomainSocket):
                    preconditionFailure("Should not be possible to create a multicast socket on a unix domain socket")
                case .none:
                    preconditionFailure(
                        "Should not be possible to create a multicast socket on an interface without an address"
                    )
                }
            }.wait()

        return datagramChannel
    }

    /// This method will query the mdns multicast group at 244.0.0.251:5353 for any machines that implement the specified service.
    public func queryForService(_ service: String) -> EventLoopFuture<[SocketAddress]> {
        guard self.state == .started else { return self.eventLoop.makeFailedFuture(Errors.mdnsServiceNotStarted) }
        /// Create the query
        let question = Question(name: "\(service).local", type: .pointer)

        /// For each reply, extract out the ip address and return them...
        return self.newQueryResponse(question).map { msgs in
            var records: [SocketAddress] = []
            for msg in msgs {
                let hostRecords = msg.answers.compactMap {
                    $0 as? DNS.HostRecord<DNS.IPv4>
                }
                for hostRecord in hostRecords {
                    records.append(try! SocketAddress(ipAddress: hostRecord.ip.presentation, port: 0))
                }
            }
            return records
        }
    }

    /// Queries the mDNS multicast group for libp2p services running on the local network, and if peers are found returns a list of Multiaddrs for dialing...
    private func queryForPeers() -> EventLoopFuture<[Multiaddr]> {
        let question = Question(name: "_p2p._udp.local", type: .all)
        return self.newQueryResponse(question).map { msgs in
            msgs.reduce(into: []) { mas, msg in
                mas.append(contentsOf: self.extractMultiaddressFromAdditionalRecords(msg.additional))
            }
        }
    }

    private func extractMultiaddressFromAdditionalRecords(_ records: [ResourceRecord]) -> [Multiaddr] {
        guard records.count >= 3 else { return [] }

        /// Lets try and recover the Multiaddr
        var peerID: String? = nil
        var ip4Address: String? = nil
        var ip6Address: String? = nil
        var protos: [String: Int] = [:]
        for record in records {
            if let serRec = record as? DNS.ServiceRecord {
                guard !serRec.name.contains(self.peerID.b58String) else { continue }
                if let pid = serRec.name.split(separator: ".").first {
                    peerID = String(pid)
                }
            }
            if let txtRec = record as? DNS.TextRecord {
                guard !txtRec.name.contains(self.peerID.b58String) else { continue }
                protos = txtRec.attributes.compactMapValues({ Int($0) })
            }
            if let ipv4Rec = record as? DNS.HostRecord<DNS.IPv4> {
                ip4Address = ipv4Rec.ip.presentation
            }
            if let ipv6Rec = record as? DNS.HostRecord<DNS.IPv6> {
                ip6Address = ipv6Rec.ip.presentation
            }
            //self.logger.trace("\(record)")
        }

        /// Filter out our own libp2p instance service and ensure we have all the necessary info to construct a Multiaddr
        guard let pid = peerID else {
            self.logger.trace("Dropping mDNS message due to no PeerID")
            return []
        }
        guard pid != self.peerID.b58String else {
            self.logger.trace("Dropping self issued mDNS message")
            return []
        }
        guard !protos.isEmpty else {
            self.logger.trace("Dropping mDNS message due to no protocols in text record")
            return []
        }
        //guard let ip = ip4Address else { self.logger.trace("Dropping mDNS message due to no ip address"); return [] }
        //guard let pid = peerID, pid != self.peerID.b58String, !protos.isEmpty else { return [] }

        var multiaddresses: [Multiaddr] = []

        if let ip4 = ip4Address {
            /// - TODO: The WebSocket exeption needs to be fixed properly... Maybe handling it in the Multiaddr initializer or something...
            multiaddresses.append(
                contentsOf: protos.compactMap { proto, port in
                    if proto == "ws" {
                        return try? Multiaddr("/ip4/\(ip4)/tcp/\(port)/ws/p2p/\(pid)")
                    } else {
                        return try? Multiaddr("/ip4/\(ip4)/\(proto)/\(port)/p2p/\(pid)")
                    }
                }
            )
        }

        if let ip6 = ip6Address {
            /// - TODO: The WebSocket exeption needs to be fixed properly... Maybe handling it in the Multiaddr initializer or something...
            multiaddresses.append(
                contentsOf: protos.compactMap { proto, port in
                    if proto == "ws" {
                        return try? Multiaddr("/ip6/\(ip6)/tcp/\(port)/ws/p2p/\(pid)")
                    } else {
                        return try? Multiaddr("/ip6/\(ip6)/\(proto)/\(port)/p2p/\(pid)")
                    }
                }
            )
        }

        /// /ip4/127.0.0.1/tcp/10000/p2p/QmQpiLteAfLv9VQHBJ4qaGNA9bVAFPBEtZDpmv4XeRtGh2
        return multiaddresses
    }

    /// This method attempts to register the specified service on the interface (en0, en1, etc).
    /// This service will be discoverable by other peers utilizing mDNS at 244.0.0.251:5353 until the service is stopped, or is unregistered using the `unregisterService` method.
    public func registerService(
        name: String,
        protocol proto: String = "udp",
        serviceName: String = "p2p"
    ) -> EventLoopFuture<Bool> {  //"instance-name"._name._protocol -> chat-room-name._p2p._udp
        guard self.state == .started else { return self.eventLoop.makeFailedFuture(Errors.mdnsServiceNotStarted) }

        return self.eventLoop.submit {
            guard proto == "udp" || proto == "tcp" else {
                self.logger.warning("Invalid protocol")
                return false
            }
            guard self.registeredServices[name] == nil else {
                self.logger.warning("Service `\(name)` already registered")
                return false
            }

            /// Attempt to register the custom service
            var dnsService: DNSServiceRef? = nil
            let flags: DNSServiceFlags = DNSServiceFlags()
            DNSServiceRegister(
                &dnsService,
                flags,
                0,
                name,
                "_\(serviceName)._\(proto)",
                "",
                "",
                UInt16(3338),
                0,
                "",
                nil,
                nil
            )

            /// if dnsService is still nil at this point, the service registration failed, so lets return false
            guard let dnsService else { return false }

            /// store the service reference in our registered services for deallocation / unregistring later
            self.registeredServices[name] = dnsService

            /// return true
            self.logger.debug("Registered Service `\(name)._\(serviceName)._\(proto)`")
            return true
        }
    }

    private func registerP2PService() -> EventLoopFuture<Bool> {
        self.eventLoop.submit {
            //guard proto == "udp" || proto == "tcp" else { self.logger.warning("Invalid protocol"); return false }
            guard self.registeredServices[self.peerID.b58String] == nil else { return false }

            /// Attempt to register the custom service
            var dnsService: DNSServiceRef? = nil
            let flags: DNSServiceFlags = DNSServiceFlags()

            /// Create a TxtRecord and append any necessary data (aka our listening addresses, including port and peerid...)
            var txtRecord = TXTRecordRef()
            var buffer = Data()
            TXTRecordCreate(&txtRecord, 1, &buffer)
            /// For each listening address, append the protocol (key) and port (value) to the txt record
            for ma in self.listeningAddresses {
                if let tcp = ma.tcpAddress {
                    self.logger.trace("Adding tcp/\(tcp.port) to text record")
                    //TXTRecordSetValue(&txtRecord, "tcp", UInt8(tcp.port.bytes.count), &tcp.port)
                    var port: String = "\(tcp.port)"
                    TXTRecordSetValue(&txtRecord, "tcp", UInt8(port.bytes.count), &port)
                }
                //else if let udp = ma.udpAddress {
                //    self.logger.trace("Adding udp/\(udp.port) to text record")
                //    //TXTRecordSetValue(&txtRecord, "udp", UInt8(udp.port.bytes.count), &udp.port)
                //    var port:String = "\(udp.port)"
                //    TXTRecordSetValue(&txtRecord, "udp", UInt8(port.bytes.count), &port)
                //}
            }
            /// Register the service...
            DNSServiceRegister(
                &dnsService,
                flags,
                0,
                self.peerID.b58String,
                "_p2p._udp",
                "",
                "",
                3338,
                TXTRecordGetLength(&txtRecord),
                TXTRecordGetBytesPtr(&txtRecord),
                nil,
                nil
            )

            /// if dnsService is still nil at this point, the service registration failed, so lets return false
            guard let dnsService else { return false }

            /// store the service reference in our registered services for deallocation / unregistring later
            self.registeredServices[self.peerID.b58String] = dnsService

            /// return true
            self.logger.debug("Registered Service `\(self.peerID.b58String)._p2p._udp`")
            return true
        }
    }

    /// Unregisters a previously registered service
    public func unregisterService(name: String) -> EventLoopFuture<Bool> {
        guard self.state == .started else { return self.eventLoop.makeFailedFuture(Errors.mdnsServiceNotStarted) }

        return self.eventLoop.submit {
            if let service = self.registeredServices[name] {
                self.logger.debug("Unregistering Service \(name)")
                DNSServiceRefDeallocate(service)
                self.registeredServices.removeValue(forKey: name)
                return true
            } else {
                self.logger.warning("Unable to find registered service with name `\(name)`")
                return false
            }
        }
    }

    /// Given a hostname, we'll attempt to resolve it into an ipv4 or and ipv6 address on your LAN
    public func resolveHostname(_ hostname: String) -> EventLoopFuture<SocketAddress?> {
        guard self.state == .started else { return self.eventLoop.makeFailedFuture(Errors.mdnsServiceNotStarted) }
        /// Create the query
        let question = Question(name: "\(hostname).local", type: .host)

        /// For each reply, extract out the ip address and return them...
        return self.newQueryResponse(question).map { msgs in
            var records: [SocketAddress] = []
            for msg in msgs {
                let hostRecords = msg.answers.compactMap {
                    $0 as? DNS.HostRecord<DNS.IPv4>
                }
                for hostRecord in hostRecords {
                    records.append(try! SocketAddress(ipAddress: hostRecord.ip.presentation, port: 0))
                }
            }
            return records.first
        }
    }

    private func onMDNSMessage(_ msg: DNS.Message) {
        //self.logger.trace("Inbound mDNS Message: \(msg)")
        let _ = self.eventLoop.submit {
            let queryCallbacks = self.outstandingQueries.filter { query in
                guard var q = msg.questions.first else { return false }
                //self.logger.trace("Comparing \(query.query.type) == \(q.type) && \(query.query.name) == \(q.name)")
                if q.name.last == "." { q.name = String(q.name.dropLast()) }
                return query.query.type == q.type && query.query.name == q.name
            }
            for queryCallback in queryCallbacks {
                //self.logger.trace("Found our handler, passing the message along")
                queryCallback.callback(msg)
            }
            return
        }
    }

    /// Or do we return a class that we can execute events on as we receive matches...
    /// Like we query for a service every 3 seconds indeffinetly
    /// Everytime we get a response we pass it along to the handler...

    /// When we issue a query to the multicast group, we might get 0, 1 or many replies. This method will take a Question, construct a promise with an appropriate timeout value, and wait/accumulate any responses we might receive from the mDNS clients
    private func newQueryResponse(_ query: Question) -> EventLoopFuture<[DNS.Message]> {
        /// Craete a promise
        let queryPromise = self.eventLoop.makePromise(of: Array<DNS.Message>.self)

        /// An array to store our answers...
        let answers: NIOLockedValueBox<[DNS.Message]> = .init([])

        /// Debouncer Task...
        let debounce: NIOLockedValueBox<Scheduled<Void>?> = .init(nil)

        /// Create the timeout...
        let timeout = self.eventLoop.scheduleTask(in: .seconds(3)) { () -> Void in
            if answers.withLockedValue({ $0 }).isEmpty {
                return queryPromise.fail(Errors.timedOut)
            } else {
                debounce.withLockedValue({ $0 })?.cancel()
                return queryPromise.succeed(answers.withLockedValue({ $0 }))
            }
        }

        /// Create the Message
        let msg = Message(
            type: .query,
            questions: [query]
        )

        /// At the moment, we only return the first repsonse, or timeout. We'll never capture other subsequent messages...
        let handler = OutstandingQuery(
            query: query,
            callback: { msg in
                //self.logger.trace("Got a response from our query, waiting another 200ms for additional responses before succeeding our promise...")

                /// Append the messages onto our answer stack
                answers.withLockedValue { $0.append(msg) }

                /// Cancel the current debounce, and add another 200ms onto our wait for additional answers...
                debounce.withLockedValue { d in
                    d?.cancel()
                    d = self.eventLoop.scheduleTask(in: .milliseconds(200)) {
                        /// Cancel our timeout
                        timeout.cancel()
                        /// Succeed with whatever messages we have so far...
                        queryPromise.succeed(answers.withLockedValue({ $0 }))
                    }
                }
            }
        )
        self.outstandingQueries.append(handler)

        /// Issue the query...
        //self.logger.trace("Issueing mDNS Query")
        self.mdnsHost?.writeAndFlush(
            AddressedEnvelope(remoteAddress: multicastGroup, data: try! msg.serialize()),
            promise: nil
        )

        /// Return the future result
        return queryPromise.futureResult
    }

    private final class MDNSMessageDecoder: ChannelInboundHandler, Sendable {
        public typealias InboundIn = AddressedEnvelope<ByteBuffer>

        private let handler: (@Sendable (DNS.Message) -> Void)

        init(messageHandler: @escaping @Sendable (DNS.Message) -> Void) {
            self.handler = messageHandler
        }

        public func channelRead(context: ChannelHandlerContext, data: NIOAny) {
            let envelope = self.unwrapInboundIn(data)

            /// Attempt to instantiate an mDNS Message with the inbound data
            if let response = try? Message(deserialize: Data(envelope.data.readableBytesView)) {
                handler(response)
            } else {
                print("Failed to decode mDNS Message Response")
                //print("Is this our ID message???")
                print(envelope)
            }
        }
    }

    private final class MDNSMessageEncoder: ChannelOutboundHandler, Sendable {
        public typealias OutboundIn = AddressedEnvelope<Data>
        public typealias OutboundOut = AddressedEnvelope<ByteBuffer>

        func write(context: ChannelHandlerContext, data: NIOAny, promise: EventLoopPromise<Void>?) {
            let message = self.unwrapOutboundIn(data)
            let buffer = context.channel.allocator.buffer(bytes: message.data)
            context.write(
                self.wrapOutboundOut(AddressedEnvelope(remoteAddress: message.remoteAddress, data: buffer)),
                promise: promise
            )
        }
    }

    enum Errors: Error {
        case timedOut
        case mdnsServiceNotStarted
        case noAddressForDevice
        case notImplemented
    }
}

extension BinaryInteger {
    // returns little endian; use .bigEndian.bytes for BE.
    var bytes: Data {
        var copy = self
        return withUnsafePointer(to: &copy) {
            Data(Data(bytes: $0, count: MemoryLayout<Self>.size).reversed())
        }
    }
}

/// Lifecycle Handler Methods
extension MulticastPeerDiscovery {
    public func willBoot(_ application: Application) throws {
        //        self.eventLoop.scheduleTask(in: .seconds(3), {
        //            try self.start()
        //        })
        DispatchQueue.global().asyncAfter(
            deadline: .now() + .seconds(3),
            execute: {
                do {
                    try self.start()
                } catch {
                    self.logger.error("Failed to start mDNS: \(error)")
                }
            }
        )
    }

    public func shutdown(_ application: Application) {
        do {
            try self.stop()
        } catch {
            self.logger.error("Failed to stop mDNS: \(error)")
        }
    }
}

extension Array where Element == Multiaddr {
    fileprivate func fixSameHostAddresses(ip4: String) -> [Multiaddr] {
        let fixed = self.compactMap { ma -> Multiaddr? in
            guard let addy = ma.addresses.first else { return nil }
            switch addy.codec {
            case .ip4:
                return ma.replace(address: ip4, with: "127.0.0.1", forCodec: .ip4)
            case .ip6:
                return ma  //.replace(address: hostIP6, with: "::1", forCodec: .ip6)
            default:
                return nil
            }
        }

        return fixed
    }
}

extension Multiaddr {
    func replace(address: String, with replacement: String, forCodec codec: MultiaddrProtocol) -> Multiaddr? {
        if self.description.contains("\(codec.name)/\(address)") {
            if let new = try? Multiaddr(
                self.description.replacingOccurrences(
                    of: "\(codec.name)/\(address)",
                    with: "\(codec.name)/\(replacement)"
                )
            ) {
                return new
            } else {
                return nil
            }
        } else {
            return self
        }
    }
}
