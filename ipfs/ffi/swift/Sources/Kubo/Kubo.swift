import CKubo
import Darwin
import Foundation

public enum KuboError: Error {
    case commandFailed(String)
}

public struct Kubo {
    public let repoPath: String

    public init(repoPath: String) {
        self.repoPath = repoPath
    }

    public static func version() -> String {
        guard let p = kubo_version() else { return "" }
        let s = String(cString: p)
        kubo_ffi_free_string(p)
        return s
    }

    public static func initRepo(_ repoPath: String) throws {
        let rc = repoPath.withCString { kubo_init_repo(UnsafeMutablePointer(mutating: $0)) }
        if rc != 0 {
            let msg: String
            if let p = kubo_ffi_last_error() {
                msg = String(cString: p)
                kubo_ffi_free_string(p)
            } else {
                msg = "init failed"
            }
            throw KuboError.commandFailed(msg)
        }
    }

    public func run(_ command: String) throws -> String {
        let result: UnsafeMutablePointer<CChar>? = repoPath.withCString { rp in
            command.withCString { cmd in
                kubo_run_command_line(UnsafeMutablePointer(mutating: rp), UnsafeMutablePointer(mutating: cmd))
            }
        }
        guard let p = result else {
            let msg: String
            if let e = kubo_ffi_last_error() {
                msg = String(cString: e)
                kubo_ffi_free_string(e)
            } else {
                msg = "command failed"
            }
            throw KuboError.commandFailed(msg)
        }
        let s = String(cString: p)
        kubo_ffi_free_string(p)
        return s
    }

    // MARK: - Node lifecycle (typed)
    public class Node {
        private let handle: UInt64
        public let repoPath: String
        private var _apiUrl: String? = nil

        public init(repoPath: String, online: Bool) throws {
            self.repoPath = repoPath
            let h = repoPath.withCString { rp in
                kubo_node_start(UnsafeMutablePointer(mutating: rp), online ? 1 : 0)
            }
            if h == 0 {
                let msg = Kubo.lastError() ?? "node start failed"
                throw KuboError.commandFailed(msg)
            }
            self.handle = h
        }

        public var peerId: String? {
            guard let p = kubo_node_peer_id(handle) else { return nil }
            let s = String(cString: p)
            kubo_ffi_free_string(p)
            return s
        }

        public var listeningAddrs: String? {
            guard let p = kubo_node_listening_addrs(handle) else { return nil }
            let s = String(cString: p)
            kubo_ffi_free_string(p)
            return s
        }

        public func stop() {
            kubo_node_stop(handle)
        }

        public func run(_ command: String) throws -> String {
            let result: UnsafeMutablePointer<CChar>? = repoPath.withCString { rp in
                command.withCString { cmd in
                    let args = command.split(separator: " ").map { String($0) }
                    let cargs = args.map { strdup($0) }
                    defer { cargs.forEach { free($0) } }
                    let argc = CInt(cargs.count)
                    let argv = UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>.allocate(capacity: max(cargs.count, 1))
                    defer { argv.deallocate() }
                    for (i, p) in cargs.enumerated() { argv[i] = p }
                    return kubo_run_command_on_node(UnsafeMutablePointer(mutating: rp), handle, argc, argv)
                }
            }
            guard let p = result else {
                let msg = Kubo.lastError() ?? "command failed"
                throw KuboError.commandFailed(msg)
            }
            let s = String(cString: p)
            kubo_ffi_free_string(p)
            return s
        }

        public func addStatic(prefix: String, dir: String) {
            _ = prefix.withCString { p in
                dir.withCString { d in
                    kubo_node_add_static(handle, UnsafeMutablePointer(mutating: p), UnsafeMutablePointer(mutating: d))
                }
            }
        }

        public func startApi(_ multiaddr: String) -> String? {
            let p = multiaddr.withCString { a in
                kubo_node_start_api(handle, UnsafeMutablePointer(mutating: a))
            }
            if let p = p {
                let s = String(cString: p)
                kubo_ffi_free_string(p)
                return s
            }
            return nil
        }

        /// Base HTTP URL for the node's RPC API (e.g. http://127.0.0.1:5001).
        public func apiUrl() -> String? {
            if let cached = _apiUrl { return cached }
            guard let api = startApi("/ip4/127.0.0.1/tcp/0") else { return nil }
            let parts = api.split(separator: "/")
            var host = "127.0.0.1"; var port = "5001"
            var i = 0
            while i < parts.count {
                if parts[i] == "ip4" && i + 1 < parts.count { host = String(parts[i+1]) }
                if parts[i] == "tcp" && i + 1 < parts.count { port = String(parts[i+1]) }
                i += 1
            }
            let url = "http://\(host):\(port)"
            _apiUrl = url
            return url
        }

        /// Call any Kubo HTTP RPC endpoint (full API coverage).
        public func apiCall(_ endpoint: String, params: [String: String] = [:]) async throws -> String {
            guard let base = apiUrl() else { throw KuboError.commandFailed("no api") }
            var comps = URLComponents(string: base + "/api/v0/" + endpoint)!
            comps.queryItems = params.map { URLQueryItem(name: $0.key, value: $0.value) }
            guard let url = comps.url else { throw KuboError.commandFailed("bad url") }
            var req = URLRequest(url: url)
            req.httpMethod = "POST"
            let (data, resp) = try await URLSession.shared.data(for: req)
            guard let http = resp as? HTTPURLResponse, http.statusCode == 200 else {
                throw KuboError.commandFailed("http \( (resp as? HTTPURLResponse)?.statusCode ?? -1)")
            }
            return String(data: data, encoding: .utf8) ?? ""
        }

        public func swarmPeers() -> [String] {
            guard let p = kubo_swarm_peers(handle) else { return [] }
            let s = String(cString: p)
            kubo_ffi_free_string(p)
            return s.split(separator: "\n").map { String($0) }.filter { !$0.isEmpty }
        }

        public func connect(_ addr: String) -> Bool {
            let rc = addr.withCString { a in
                kubo_node_connect(handle, UnsafeMutablePointer(mutating: a))
            }
            return rc == 0
        }

        deinit { kubo_node_stop(handle) }
    }

    public class WatchHandle {
        private let handle: UInt64
        public init(repoPath: String, watchPath: String) throws {
            let h = repoPath.withCString { rp in
                watchPath.withCString { wp in
                    kubo_watch_path(UnsafeMutablePointer(mutating: rp), UnsafeMutablePointer(mutating: wp))
                }
            }
            if h == 0 {
                let msg = Kubo.lastError() ?? "watch start failed"
                throw KuboError.commandFailed(msg)
            }
            self.handle = h
        }
        public func stop() { kubo_watch_stop(handle) }
        deinit { kubo_watch_stop(handle) }
    }

    public static func lastError() -> String? {
        guard let p = kubo_ffi_last_error() else { return nil }
        let s = String(cString: p)
        kubo_ffi_free_string(p)
        return s
    }

    public static func getLogs() -> String {
        guard let p = kubo_get_logs() else { return "" }
        let s = String(cString: p)
        kubo_ffi_free_string(p)
        return s
    }
}
