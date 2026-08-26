import Foundation
import Git
import LibP2P
import LibP2PAutoNAT
import LibP2PDCUtR
import LibP2PRelay
import LibP2PNoise
import LibP2PYAMUX
import LibP2PMDNS

extension String {
    static func key(_ key: String) -> String { return NSLocalizedString(key, comment: "") }
}

enum State {
    case loading
    case ready
    case packed
    case create
    case first
}

public protocol GitPeerServiceDelegate: AnyObject {
    func gitPeerService(_ service: GitPeerService, didDiscover peer: PeerID)
    func gitPeerService(_ service: GitPeerService, didLose peer: PeerID)
    func gitPeerService(_ service: GitPeerService, didReceive envelope: GitPeerEnvelope, from peer: PeerID)
}

public extension GitPeerServiceDelegate {
    func gitPeerService(_ service: GitPeerService, didDiscover peer: PeerID) {}
    func gitPeerService(_ service: GitPeerService, didLose peer: PeerID) {}
    func gitPeerService(_ service: GitPeerService, didReceive envelope: GitPeerEnvelope, from peer: PeerID) {}
}

public final class GitPeerService {
    public static let shared = GitPeerService()

    public struct Status {
        public let isRunning: Bool
        public let peerCount: Int
        public let hasAdvertisement: Bool
        public let peerID: String?
    }

    private enum LifecycleState {
        case stopped
        case starting
        case running
        case stopping
    }

    public enum ServiceError: Error {
        case notRunning
        case peerIdentityUnavailable
        case unableToEncodeEnvelope
    }

    private static let peerStorageKey = "GitPeerService.peerID"
    private static let protocolName = "/gnostr/git/1.0.0"

    public weak var delegate: GitPeerServiceDelegate?
    private var app: Application?
    private var state: LifecycleState = .stopped
    private var peers: [String: PeerID] = [:]
    private var advertisement: GitPeerRepositoryAdvertisement?
    private var repositoryURL: URL?
    private var peerStatuses: [String: GitPeerRepositoryStatus] = [:]
    private var peerRefs: [String: GitPeerRepositoryRefs] = [:]

    public var isRunning: Bool { self.state == .running }
    public var peerID: PeerID? { self.app?.peerID }
    public var status: Status {
        Status(
            isRunning: self.isRunning,
            peerCount: self.peers.count,
            hasAdvertisement: self.advertisement != nil,
            peerID: self.peerID?.b58String
        )
    }
    public var connectedPeers: [PeerID] { Array(self.peers.values) }
    public var knownPeerStatuses: [GitPeerRepositoryStatus] { Array(self.peerStatuses.values) }
    public var knownPeerRefs: [GitPeerRepositoryRefs] { Array(self.peerRefs.values) }

    private init() {}

    public func start() async throws {
        guard self.state == .stopped else { return }
        self.state = .starting
        do {
            let peerID = try self.loadPeerID()
            print("[GitPeerService] starting as \(peerID.shortDescription)")
            let app = try await Application.make(.detect(), peerID: peerID)
            app.logger.logLevel = .debug
            app.security.use(.noise)
            app.muxers.use(.yamux)
            app.relay.use(.relay)
            app.autonat.use(.autonat)
            app.dcutr.use(.dcutr)
            app.discovery.use(.mdns)
            app.servers.use(.tcp(host: "0.0.0.0", port: 0))
            app.logger.notice("Enabled relay/AutoNAT/DCUtR for hole punching")
            self.installRoutes(on: app)
            self.installDiscovery(on: app)
            try await app.startup()
            self.app = app
            self.state = .running
            app.logger.notice("Git peer service started as \(peerID.shortDescription)")
            app.logger.notice("Git peer service listening on \(Self.protocolName)")
        } catch {
            self.state = .stopped
            throw error
        }
    }

    public func stop() async {
        guard self.state == .running, let app = self.app else { return }
        self.state = .stopping
        app.logger.notice("Stopping git peer service with \(self.peers.count) peer(s)")
        try? await app.asyncShutdown()
        self.app = nil
        self.state = .stopped
        print("[GitPeerService] stopped")
    }

    public func announce(repository url: URL) throws {
        guard let app = self.app, self.state == .running else { throw ServiceError.notRunning }
        let peerID = app.peerID

        self.repositoryURL = url
        let advertisement = GitPeerRepositoryAdvertisement(url: url, peer: peerID.shortDescription)
        self.advertisement = advertisement

        guard let payload = try? JSONEncoder().encode(advertisement) else { throw ServiceError.unableToEncodeEnvelope }
        let envelope = GitPeerEnvelope(
            kind: .hello,
            repository: advertisement.name,
            payload: String(decoding: payload, as: UTF8.self),
            peer: advertisement.peer
        )

        app.logger.notice("Advertising repository \(advertisement.name) at \(url.path) to \(self.peers.count) peer(s)")
        app.logger.info("Git peer advert: branch=\(Hub.branch(url) ?? "unknown") remote=\(Hub.remote(url)) commit=\(Hub.id(url) ?? "")")
        for peer in self.peers.values {
            app.logger.info("Sending hello/status/refs to \(peer.shortDescription)")
            try? self.send(envelope, to: peer)
            try? self.sendStatus(to: peer)
            try? self.requestRefs(from: peer)
        }
    }

    public func send(_ envelope: GitPeerEnvelope, to peer: PeerID) throws {
        guard let app = self.app, self.state == .running else { throw ServiceError.notRunning }
        guard let data = try? JSONEncoder().encode(envelope) else { throw ServiceError.unableToEncodeEnvelope }
        app.logger.debug("Sending \(envelope.kind.rawValue) envelope to \(peer.shortDescription)")
        app.newRequest(
            to: peer,
            forProtocol: Self.protocolName,
            withRequest: data,
            style: .noResponseExpected,
            withHandlers: .inherit
        ).whenComplete { result in
            switch result {
            case .failure(let error):
                app.logger.error("Failed to send git peer envelope to \(peer): \(error)")
            case .success:
                app.logger.trace("Sent git peer envelope to \(peer)")
            }
        }
    }

    public func sendStatus(to peer: PeerID) throws {
        guard let envelope = self.statusEnvelope else { return }
        self.app?.logger.debug("Sending status to \(peer.shortDescription)")
        try self.send(envelope, to: peer)
    }

    public func requestRefs(from peer: PeerID) throws {
        guard let app = self.app, self.state == .running else { throw ServiceError.notRunning }
        let request = GitPeerEnvelope(kind: .refs, repository: self.advertisement?.name, peer: self.peerID?.shortDescription)
        guard let data = try? JSONEncoder().encode(request) else { throw ServiceError.unableToEncodeEnvelope }
        app.logger.debug("Requesting refs from \(peer.shortDescription)")
        app.newRequest(
            to: peer,
            forProtocol: Self.protocolName,
            withRequest: data,
            style: .responseExpected,
            withHandlers: .inherit
        ).whenComplete { result in
            switch result {
            case .failure(let error):
                app.logger.error("Failed to request git peer refs from \(peer): \(error)")
            case .success(let response):
                app.logger.debug("Received refs response from \(peer.shortDescription)")
                guard let str = String(data: response, encoding: .utf8),
                      let data = str.data(using: .utf8),
                      let envelope = try? JSONDecoder().decode(GitPeerEnvelope.self, from: data),
                      let refsData = envelope.payload?.data(using: .utf8),
                      let refs = try? JSONDecoder().decode(GitPeerRepositoryRefs.self, from: refsData) else {
                    app.logger.error("Failed to decode git peer refs from \(peer)")
                    return
                }
                self.peerRefs[peer.b58String] = refs
                app.logger.notice("Peer refs from \(peer.shortDescription): \(refs.commit) on \(refs.branch)")
                guard let repositoryURL = self.repositoryURL else { return }
                let localCommit = Hub.id(repositoryURL) ?? ""
                guard refs.commit != localCommit else {
                    app.logger.info("Peer \(peer.shortDescription) is already at local commit \(localCommit)")
                    return
                }
                app.logger.info("Requesting pack from \(peer.shortDescription): want=\(refs.commit) have=\(localCommit)")
                try? self.requestPack(from: peer, want: refs.commit, have: localCommit)
            }
        }
    }

    public func requestPack(from peer: PeerID, want: String, have: String) throws {
        guard let app = self.app, self.state == .running else { throw ServiceError.notRunning }
        guard let repositoryURL = self.repositoryURL else { throw ServiceError.notRunning }
        let request = GitPeerFetchRequest(
            repository: repositoryURL.lastPathComponent,
            want: want,
            have: have,
            peer: self.peerID?.shortDescription ?? ""
        )
        guard let data = try? JSONEncoder().encode(request) else { throw ServiceError.unableToEncodeEnvelope }
        let envelope = GitPeerEnvelope(
            kind: .fetch,
            repository: request.repository,
            payload: String(decoding: data, as: UTF8.self),
            peer: request.peer
        )
        guard let requestData = try? JSONEncoder().encode(envelope) else { throw ServiceError.unableToEncodeEnvelope }
        app.logger.debug("Requesting pack from \(peer.shortDescription): want=\(want) have=\(have)")
        app.newRequest(
            to: peer,
            forProtocol: Self.protocolName,
            withRequest: requestData,
            style: .responseExpected,
            withHandlers: .inherit
        ).whenComplete { result in
            switch result {
            case .failure(let error):
                app.logger.error("Failed to fetch git pack from \(peer): \(error)")
            case .success(let response):
                app.logger.debug("Received pack response from \(peer.shortDescription)")
                guard let str = String(data: response, encoding: .utf8),
                      let data = str.data(using: .utf8),
                      let envelope = try? JSONDecoder().decode(GitPeerEnvelope.self, from: data),
                      let payload = envelope.payload?.data(using: .utf8),
                      let pack = try? JSONDecoder().decode(GitPeerPackResponse.self, from: payload),
                      let packData = Data(base64Encoded: pack.pack) else {
                    app.logger.error("Failed to decode git pack response from \(peer)")
                    return
                }
                do {
                    try Hub.unpack(packData, url: repositoryURL)
                    if !want.isEmpty {
                        try Hub.update(repositoryURL, id: want)
                        try Hub.origin(repositoryURL, id: want)
                    }
                    app.logger.notice("Applied git pack from \(peer.shortDescription): want=\(want)")
                } catch {
                    app.logger.error("Failed to apply git pack from \(peer): \(error)")
                }
            }
        }
    }

    public func sync(repository url: URL) {
        guard self.state == .running, self.repositoryURL == url, let localCommit = Hub.id(url), !localCommit.isEmpty else {
            self.app?.logger.debug("Skipping sync for \(url.lastPathComponent)")
            return
        }
        self.app?.logger.info("Syncing \(url.lastPathComponent) at \(localCommit) to \(self.peers.count) peer(s)")
        for peer in self.peers.values {
            guard let refs = self.peerRefs[peer.b58String], refs.commit != localCommit else { continue }
            self.app?.logger.debug("Pushing to \(peer.shortDescription): remote=\(refs.commit) local=\(localCommit)")
            try? self.requestPush(to: peer, old: refs.commit, new: localCommit)
        }
    }

    public func requestPush(to peer: PeerID, old: String, new: String) throws {
        guard let app = self.app, self.state == .running else { throw ServiceError.notRunning }
        guard let repositoryURL = self.repositoryURL else { throw ServiceError.notRunning }
        let pack = try Hub.pack(repositoryURL, from: new, to: old.isEmpty ? nil : old)
        let request = GitPeerPushRequest(
            repository: repositoryURL.lastPathComponent,
            old: old,
            new: new,
            pack: pack,
            peer: self.peerID?.shortDescription ?? ""
        )
        guard let data = try? JSONEncoder().encode(request) else { throw ServiceError.unableToEncodeEnvelope }
        let envelope = GitPeerEnvelope(
            kind: .push,
            repository: request.repository,
            payload: String(decoding: data, as: UTF8.self),
            peer: request.peer
        )
        guard let requestData = try? JSONEncoder().encode(envelope) else { throw ServiceError.unableToEncodeEnvelope }
        app.logger.debug("Requesting push to \(peer.shortDescription): old=\(old) new=\(new)")
        app.newRequest(
            to: peer,
            forProtocol: Self.protocolName,
            withRequest: requestData,
            style: .responseExpected,
            withHandlers: .inherit
        ).whenComplete { result in
            switch result {
            case .failure(let error):
                app.logger.error("Failed to push git pack to \(peer): \(error)")
            case .success(let response):
                app.logger.debug("Received push ack from \(peer.shortDescription)")
                guard let str = String(data: response, encoding: .utf8),
                      let data = str.data(using: .utf8),
                      let envelope = try? JSONDecoder().decode(GitPeerEnvelope.self, from: data),
                      let payload = envelope.payload?.data(using: .utf8),
                      let ack = try? JSONDecoder().decode(GitPeerPushResponse.self, from: payload),
                      ack.accepted else {
                    app.logger.error("Failed to decode git push response from \(peer)")
                    return
                }
                self.peerRefs[peer.b58String] = GitPeerRepositoryRefs(
                    repository: ack.repository,
                    branch: Hub.branch(repositoryURL) ?? "unknown",
                    reference: ack.new,
                    commit: ack.new,
                    remote: Hub.remote(repositoryURL),
                    peer: peer.shortDescription
                )
                app.logger.notice("Applied git push to \(peer.shortDescription): new=\(ack.new)")
            }
        }
    }

    private var statusEnvelope: GitPeerEnvelope? {
        guard let peerID = self.peerID, let repositoryURL = self.repositoryURL else { return nil }
        let branch = Hub.branch(repositoryURL) ?? "unknown"
        let remote = Hub.remote(repositoryURL)
        let payload = GitPeerRepositoryStatus(
            repository: repositoryURL.lastPathComponent,
            branch: branch,
            remote: remote,
            peer: peerID.shortDescription,
            peerCount: self.peers.count
        )
        guard let data = try? JSONEncoder().encode(payload) else { return nil }
        return GitPeerEnvelope(
            kind: .status,
            repository: payload.repository,
            payload: String(decoding: data, as: UTF8.self),
            peer: payload.peer
        )
    }

    private var refsEnvelope: GitPeerEnvelope? {
        guard let peerID = self.peerID, let repositoryURL = self.repositoryURL else { return nil }
        let branch = Hub.branch(repositoryURL) ?? "unknown"
        let reference = Hub.reference(repositoryURL) ?? branch
        let commit = Hub.id(repositoryURL) ?? ""
        let remote = Hub.remote(repositoryURL)
        let payload = GitPeerRepositoryRefs(
            repository: repositoryURL.lastPathComponent,
            branch: branch,
            reference: reference,
            commit: commit,
            remote: remote,
            peer: peerID.shortDescription
        )
        guard let data = try? JSONEncoder().encode(payload) else { return nil }
        return GitPeerEnvelope(
            kind: .refs,
            repository: payload.repository,
            payload: String(decoding: data, as: UTF8.self),
            peer: payload.peer
        )
    }
    private func loadPeerID() throws -> PeerID {
        if let data = UserDefaults.standard.data(forKey: Self.peerStorageKey) {
            return try PeerID(marshaledPrivateKey: data)
        }
        let peerID = try PeerID(.Ed25519)
        UserDefaults.standard.set(Data(try peerID.marshalPrivateKey()), forKey: Self.peerStorageKey)
        return peerID
    }

    private func installRoutes(on app: Application) {
        app.group("git") { routes in
            routes.on("1.0.0", handlers: [.newLineDelimited]) { [weak self] req -> Response<String> in
                guard let peer = req.remotePeer else {
                    req.logger.warning("Unidentified git peer request")
                    return .close
                }
                switch req.event {
                case .ready:
                    req.logger.debug("Git peer stream ready from \(peer.shortDescription)")
                    return .stayOpen
                case .data(let payload):
                    if let str = String(data: Data(payload.readableBytesView), encoding: .utf8),
                       let data = str.data(using: .utf8),
                       let envelope = try? JSONDecoder().decode(GitPeerEnvelope.self, from: data) {
                        req.logger.info("Received \(envelope.kind.rawValue) from \(peer.shortDescription)")
                        if envelope.kind == .hello {
                            req.logger.debug("Hello from \(peer.shortDescription); sending refs and status")
                            try? self?.sendStatus(to: peer)
                            try? self?.requestRefs(from: peer)
                        } else if envelope.kind == .refs {
                            req.logger.debug("Received refs request from \(peer.shortDescription)")
                            if let refs = self?.refsEnvelope,
                               let data = try? JSONEncoder().encode(refs),
                               let response = String(data: data, encoding: .utf8) {
                                req.logger.notice("Returning refs for \(self?.repositoryURL?.lastPathComponent ?? "unknown") to \(peer.shortDescription)")
                                return .respondThenClose(response)
                            }
                        } else if envelope.kind == .fetch,
                            let payload = envelope.payload?.data(using: .utf8),
                            let fetch = try? JSONDecoder().decode(GitPeerFetchRequest.self, from: payload),
                            let repositoryURL = self?.repositoryURL {
                            do {
                                req.logger.info("Generating pack for \(peer.shortDescription): want=\(fetch.want) have=\(fetch.have)")
                                let pack = try Hub.pack(repositoryURL, from: fetch.want, to: fetch.have.isEmpty ? nil : fetch.have)
                                let response = GitPeerPackResponse(
                                    repository: fetch.repository,
                                    want: fetch.want,
                                    have: fetch.have,
                                    pack: pack,
                                    peer: self?.peerID?.shortDescription ?? ""
                                )
                                let data = try JSONEncoder().encode(GitPeerEnvelope(
                                    kind: .pack,
                                    repository: response.repository,
                                    payload: String(decoding: try JSONEncoder().encode(response), as: UTF8.self),
                                    peer: response.peer
                                ))
                                req.logger.notice("Returning pack to \(peer.shortDescription)")
                                return .respondThenClose(String(data: data, encoding: .utf8) ?? "")
                            } catch {
                                req.logger.error("Failed to generate peer pack: \(error)")
                                return .close
                            }
                        } else if envelope.kind == .push,
                            let payload = envelope.payload?.data(using: .utf8),
                            let push = try? JSONDecoder().decode(GitPeerPushRequest.self, from: payload),
                            let repositoryURL = self?.repositoryURL,
                            let packData = Data(base64Encoded: push.pack) {
                            do {
                                req.logger.info("Applying push from \(peer.shortDescription): new=\(push.new)")
                                try Hub.unpack(packData, url: repositoryURL)
                                try Hub.update(repositoryURL, id: push.new)
                                try Hub.origin(repositoryURL, id: push.new)
                                let response = GitPeerPushResponse(
                                    repository: push.repository,
                                    accepted: true,
                                    new: push.new,
                                    peer: self?.peerID?.shortDescription ?? ""
                                )
                                let ack = GitPeerEnvelope(
                                    kind: .push,
                                    repository: response.repository,
                                    payload: String(decoding: try JSONEncoder().encode(response), as: UTF8.self),
                                    peer: response.peer
                                )
                                let data = try JSONEncoder().encode(ack)
                                req.logger.notice("Acknowledging push to \(peer.shortDescription)")
                                return .respondThenClose(String(data: data, encoding: .utf8) ?? "")
                            } catch {
                                req.logger.error("Failed to apply peer push: \(error)")
                                return .close
                            }
                        }
                        if envelope.kind == .refs,
                           let refs = envelope.payload?.data(using: .utf8).flatMap({ try? JSONDecoder().decode(GitPeerRepositoryRefs.self, from: $0) }) {
                            self?.peerRefs[peer.b58String] = refs
                            req.logger.debug("Stored refs for \(peer.shortDescription): \(refs.commit)")
                        } else if envelope.kind == .status,
                            let status = envelope.payload?.data(using: .utf8).flatMap({ try? JSONDecoder().decode(GitPeerRepositoryStatus.self, from: $0) }) {
                            self?.peerStatuses[peer.b58String] = status
                            req.logger.debug("Stored status for \(peer.shortDescription): peers=\(status.peerCount)")
                        }
                        self?.delegate?.gitPeerService(self ?? GitPeerService.shared, didReceive: envelope, from: peer)
                    } else if let str = String(data: Data(payload.readableBytesView), encoding: .utf8) {
                        let envelope = GitPeerEnvelope(kind: .error, payload: str, peer: peer.shortDescription)
                        req.logger.warning("Received undecodable payload from \(peer.shortDescription): \(str)")
                        self?.delegate?.gitPeerService(self ?? GitPeerService.shared, didReceive: envelope, from: peer)
                    }
                    return .stayOpen
                case .closed:
                    req.logger.debug("Git peer stream closed from \(peer.shortDescription)")
                    return .close
                case .error(let error):
                    req.logger.error("Git route error: \(error)")
                    return .close
                }
            }
        }
    }

    private func installDiscovery(on app: Application) {
        app.discovery.onPeerDiscovered(app) { peer in
            app.connections.getConnectionsToPeer(peer: peer.peer, on: nil).whenSuccess { conns in
                guard conns.isEmpty else {
                    app.logger.debug("Already connected to \(peer.peer.shortDescription)")
                    return
                }
                self.peers[peer.peer.b58String] = peer.peer
                app.logger.notice("Discovered peer \(peer.peer.shortDescription) at \(peer.addresses.count) address(es)")
                app.logger.debug("Peer \(peer.peer.shortDescription) addresses: \(peer.addresses.map(\.description).joined(separator: ", "))")
                if let advertisement = self.advertisement, let payload = try? JSONEncoder().encode(advertisement) {
                    let envelope = GitPeerEnvelope(
                        kind: .hello,
                        repository: advertisement.name,
                        payload: String(decoding: payload, as: UTF8.self),
                        peer: advertisement.peer
                    )
                    app.logger.debug("Sending hello to newly discovered peer \(peer.peer.shortDescription)")
                    try? self.send(envelope, to: peer.peer)
                    try? self.sendStatus(to: peer.peer)
                }
                guard let address = self.preferredDialAddress(for: peer.addresses) else {
                    app.logger.warning("No dialable address for peer \(peer.peer)")
                    return
                }
                do {
                    app.logger.notice("Dialing peer \(peer.peer.shortDescription) via \(address)")
                    try app.newStream(to: address, forProtocol: Self.protocolName)
                } catch {
                    app.logger.error("Failed to dial peer \(peer.peer): \(error)")
                }
            }
        }

        app.topology.register(
            TopologyRegistration(
                protocol: Self.protocolName,
                handler: TopologyHandler(
                    onConnect: { [weak self] peer, _ in
                        self?.peers[peer.b58String] = peer
                        self?.app?.logger.notice("Connected to peer \(peer.shortDescription)")
                        self?.delegate?.gitPeerService(self ?? GitPeerService.shared, didDiscover: peer)
                        if let advertisement = self?.advertisement, let payload = try? JSONEncoder().encode(advertisement) {
                            let envelope = GitPeerEnvelope(
                                kind: .hello,
                                repository: advertisement.name,
                                payload: String(decoding: payload, as: UTF8.self),
                                peer: advertisement.peer
                            )
                            self?.app?.logger.debug("Sending hello/status/refs after connect to \(peer.shortDescription)")
                            try? self?.send(envelope, to: peer)
                            try? self?.sendStatus(to: peer)
                            try? self?.requestRefs(from: peer)
                        }
                    },
                    onDisconnect: { [weak self] peer in
                        self?.peers.removeValue(forKey: peer.b58String)
                        self?.peerStatuses.removeValue(forKey: peer.b58String)
                        self?.app?.logger.notice("Disconnected from peer \(peer.shortDescription)")
                        self?.delegate?.gitPeerService(self ?? GitPeerService.shared, didLose: peer)
                    }
                )
            )
        )
    }

    private func preferredDialAddress(for addresses: [Multiaddr]) -> Multiaddr? {
        addresses.first(where: { $0.description.contains("/p2p-circuit/") })
            ?? addresses.first(where: { $0.description.contains("/tcp/") })
            ?? addresses.first
    }
}
