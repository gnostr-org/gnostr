import Foundation

public struct GitPeerEnvelope: Codable {
    public enum Kind: String, Codable {
        case hello
        case status
        case refs
        case fetch
        case pack
        case push
        case error
    }

    public var kind: Kind
    public var repository: String?
    public var payload: String?
    public var peer: String?
    public var timestamp: Date

    public init(
        kind: Kind,
        repository: String? = nil,
        payload: String? = nil,
        peer: String? = nil,
        timestamp: Date = .init()
    ) {
        self.kind = kind
        self.repository = repository
        self.payload = payload
        self.peer = peer
        self.timestamp = timestamp
    }
}

public struct GitPeerRepositoryAdvertisement: Codable {
    public let name: String
    public let path: String
    public let peer: String
    public let timestamp: Date

    public init(url: URL, peer: String, timestamp: Date = .init()) {
        self.name = url.lastPathComponent
        self.path = url.path
        self.peer = peer
        self.timestamp = timestamp
    }
}

public struct GitPeerRepositoryStatus: Codable {
    public let repository: String
    public let branch: String
    public let remote: String
    public let peer: String
    public let peerCount: Int
    public let timestamp: Date

    public init(
        repository: String,
        branch: String,
        remote: String,
        peer: String,
        peerCount: Int,
        timestamp: Date = .init()
    ) {
        self.repository = repository
        self.branch = branch
        self.remote = remote
        self.peer = peer
        self.peerCount = peerCount
        self.timestamp = timestamp
    }
}

public struct GitPeerRepositoryRefs: Codable {
    public let repository: String
    public let branch: String
    public let reference: String
    public let commit: String
    public let remote: String
    public let peer: String
    public let timestamp: Date

    public init(
        repository: String,
        branch: String,
        reference: String,
        commit: String,
        remote: String,
        peer: String,
        timestamp: Date = .init()
    ) {
        self.repository = repository
        self.branch = branch
        self.reference = reference
        self.commit = commit
        self.remote = remote
        self.peer = peer
        self.timestamp = timestamp
    }
}

public struct GitPeerFetchRequest: Codable {
    public let repository: String
    public let want: String
    public let have: String
    public let peer: String
    public let timestamp: Date

    public init(
        repository: String,
        want: String,
        have: String,
        peer: String,
        timestamp: Date = .init()
    ) {
        self.repository = repository
        self.want = want
        self.have = have
        self.peer = peer
        self.timestamp = timestamp
    }
}

public struct GitPeerPackResponse: Codable {
    public let repository: String
    public let want: String
    public let have: String
    public let pack: String
    public let peer: String
    public let timestamp: Date

    public init(
        repository: String,
        want: String,
        have: String,
        pack: Data,
        peer: String,
        timestamp: Date = .init()
    ) {
        self.repository = repository
        self.want = want
        self.have = have
        self.pack = pack.base64EncodedString()
        self.peer = peer
        self.timestamp = timestamp
    }
}

public struct GitPeerPushRequest: Codable {
    public let repository: String
    public let old: String
    public let new: String
    public let pack: String
    public let peer: String
    public let timestamp: Date

    public init(
        repository: String,
        old: String,
        new: String,
        pack: Data,
        peer: String,
        timestamp: Date = .init()
    ) {
        self.repository = repository
        self.old = old
        self.new = new
        self.pack = pack.base64EncodedString()
        self.peer = peer
        self.timestamp = timestamp
    }
}

public struct GitPeerPushResponse: Codable {
    public let repository: String
    public let accepted: Bool
    public let new: String
    public let peer: String
    public let timestamp: Date

    public init(
        repository: String,
        accepted: Bool,
        new: String,
        peer: String,
        timestamp: Date = .init()
    ) {
        self.repository = repository
        self.accepted = accepted
        self.new = new
        self.peer = peer
        self.timestamp = timestamp
    }
}
