import Foundation
import Darwin

private struct RustEnvelope<T: Decodable>: Decodable {
    let ok: Bool
    let data: T?
    let error: String?
}

private struct RelayCliDefaults: Codable, Sendable {
    let logging: String
    let configFilePath: String

    private enum CodingKeys: String, CodingKey {
        case logging
        case configFilePath = "config_file_path"
    }
}

private typealias RustStringFn = @convention(c) (UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
private typealias RustRelayEndpointFn = @convention(c) (UnsafePointer<CChar>, UInt16) -> UnsafeMutablePointer<CChar>?
private typealias RustFreeFn = @convention(c) (UnsafeMutablePointer<CChar>) -> Void

public struct RelayProcessState: Codable, Hashable, Sendable {
    public let running: Bool
    public let pid: UInt32?
    public let message: String
    public let diskUsageBytes: UInt64?

    public init(
        running: Bool,
        pid: UInt32?,
        message: String,
        diskUsageBytes: UInt64?
    ) {
        self.running = running
        self.pid = pid
        self.message = message
        self.diskUsageBytes = diskUsageBytes
    }

    private enum CodingKeys: String, CodingKey {
        case running
        case pid
        case message
        case diskUsageBytes = "disk_usage_bytes"
    }
}

public struct RelayConfiguration: Codable, Hashable, Sendable {
    public var logging: String
    public var configFilePath: String

    public init(logging: String = "info", configFilePath: String = ".gnostr/relay.toml") {
        self.logging = logging
        self.configFilePath = configFilePath
    }

    private enum CodingKeys: String, CodingKey {
        case logging
        case configFilePath = "config_file_path"
    }

    public static func rustDefault() -> RelayConfiguration? {
        RustRelayBridge.shared.defaultConfiguration()
    }
}

public extension RelayConfiguration {
    static func resolvedConfigFilePath(
        _ configFilePath: String,
        relativeTo baseDirectoryPath: String = FileManager.default.currentDirectoryPath
    ) -> String {
        let expandedPath = (configFilePath as NSString).expandingTildeInPath
        if expandedPath.hasPrefix("/") {
            return (expandedPath as NSString).standardizingPath
        }
        return ((baseDirectoryPath as NSString).appendingPathComponent(expandedPath) as NSString).standardizingPath
    }

    var resolvedConfigFilePath: String {
        Self.resolvedConfigFilePath(configFilePath)
    }
}

public final class RelayClient: @unchecked Sendable {
    private let bridge: RustRelayBridge

    public init(bridge: RustRelayBridge = .shared) {
        self.bridge = bridge
    }

    public func defaultConfiguration() -> RelayConfiguration? {
        RelayConfiguration.rustDefault()
    }

    public func listenEndpoint(host: String, port: UInt16) -> String {
        RelayEndpoints.listenEndpoint(host: host, port: port)
    }

    public func relayStatus() throws -> RelayProcessState {
        try bridge.relayStatus().unwrap("relay status unavailable")
    }

    public func relayStart() throws -> RelayProcessState {
        try bridge.relayStart().unwrap("relay start unavailable")
    }

    public func relayStop() throws -> RelayProcessState {
        try bridge.relayStop().unwrap("relay stop unavailable")
    }

    public func relayRestart() throws -> RelayProcessState {
        try bridge.relayRestart().unwrap("relay restart unavailable")
    }
}

public enum RelayEndpoints {
    public static func listenEndpoint(host: String, port: UInt16) -> String {
        if let bridged = RustRelayBridge.shared.listenEndpoint(host: host, port: port) {
            return bridged
        }
        return "ws://\(host):\(port)"
    }
}

public final class RustRelayBridge: @unchecked Sendable {
    public static let shared = RustRelayBridge()

    private let handle: UnsafeMutableRawPointer?
    private let defaultConfigurationFn: RustStringFn?
    private let listenEndpointFn: RustRelayEndpointFn?
    private let relayStatusFn: RustStringFn?
    private let relayStartFn: RustStringFn?
    private let relayStopFn: RustStringFn?
    private let relayRestartFn: RustStringFn?
    private let freeFn: RustFreeFn?

    private init() {
        self.handle = Self.openLibrary()
        if let handle {
            self.defaultConfigurationFn = Self.loadSymbol("relay_default_configuration_json", from: handle)
            self.listenEndpointFn = Self.loadSymbol("relay_listen_endpoint_json", from: handle)
            self.relayStatusFn = Self.loadSymbol("relay_status_json", from: handle)
            self.relayStartFn = Self.loadSymbol("relay_start_json", from: handle)
            self.relayStopFn = Self.loadSymbol("relay_stop_json", from: handle)
            self.relayRestartFn = Self.loadSymbol("relay_restart_json", from: handle)
            self.freeFn = Self.loadSymbol("relay_string_free", from: handle)
        } else {
            self.defaultConfigurationFn = nil
            self.listenEndpointFn = nil
            self.relayStatusFn = nil
            self.relayStartFn = nil
            self.relayStopFn = nil
            self.relayRestartFn = nil
            self.freeFn = nil
        }
    }

    public var isAvailable: Bool {
        handle != nil
            && defaultConfigurationFn != nil
            && listenEndpointFn != nil
            && relayStatusFn != nil
            && relayStartFn != nil
            && relayStopFn != nil
            && relayRestartFn != nil
            && freeFn != nil
    }

    public func defaultConfiguration() -> RelayConfiguration? {
        guard let json: String = callString(defaultConfigurationFn, input: "") else { return nil }
        guard let data = json.data(using: .utf8),
              let defaults = try? JSONDecoder().decode(RelayCliDefaults.self, from: data) else {
            return nil
        }
        return RelayConfiguration(logging: defaults.logging, configFilePath: defaults.configFilePath)
    }

    public func listenEndpoint(host: String, port: UInt16) -> String? {
        guard let fn = listenEndpointFn else { return nil }
        let hostCString = host.cString(using: .utf8)!
        return hostCString.withUnsafeBufferPointer { buffer in
            guard let base = buffer.baseAddress else { return nil }
            guard let rawResult = fn(base, port) else { return nil }
            defer { freeFn?(rawResult) }
            return Self.decodeEnvelopeString(rawResult)
        }
    }

    public func relayStatus() -> RelayProcessState? {
        callRelayProcessState(relayStatusFn)
    }

    public func relayStart() -> RelayProcessState? {
        callRelayProcessState(relayStartFn)
    }

    public func relayStop() -> RelayProcessState? {
        callRelayProcessState(relayStopFn)
    }

    public func relayRestart() -> RelayProcessState? {
        callRelayProcessState(relayRestartFn)
    }

    private func callString(_ fn: RustStringFn?, input: String) -> String? {
        guard let fn else { return nil }
        let inputCString = input.cString(using: .utf8)!
        return inputCString.withUnsafeBufferPointer { buffer in
            guard let base = buffer.baseAddress else { return nil }
            guard let rawResult = fn(base) else { return nil }
            defer { freeFn?(rawResult) }
            return Self.decodeEnvelopeString(rawResult)
        }
    }

    private static func decodeEnvelopeString(_ rawResult: UnsafeMutablePointer<CChar>) -> String? {
        let json = String(cString: rawResult)
        guard let data = json.data(using: .utf8) else { return nil }
        let envelope = try? JSONDecoder().decode(RustEnvelope<String>.self, from: data)
        guard let envelope, envelope.ok, let value = envelope.data else { return nil }
        return value
    }

    private func callRelayProcessState(_ fn: RustStringFn?) -> RelayProcessState? {
        guard let json: String = callString(fn, input: "") else { return nil }
        guard let data = json.data(using: .utf8),
              let envelope = try? JSONDecoder().decode(RustEnvelope<RelayProcessState>.self, from: data),
              envelope.ok,
              let value = envelope.data else {
            return nil
        }
        return value
    }

    private static func openLibrary() -> UnsafeMutableRawPointer? {
        let env = ProcessInfo.processInfo.environment
        let candidates: [String] = {
            var paths: [String] = []
            if let explicit = env["GNOSTR_RELAY_FFI_LIBRARY"], !explicit.isEmpty {
                paths.append(explicit)
            }

            if let frameworksURL = Bundle.main.privateFrameworksURL {
                paths.append(frameworksURL.appendingPathComponent("librelay_ffi.dylib").path)
            }

            paths.append(Bundle.main.bundleURL.appendingPathComponent("Frameworks/librelay_ffi.dylib").path)
            paths.append(Bundle.main.bundleURL.appendingPathComponent("Contents/Frameworks/librelay_ffi.dylib").path)
            let cargoCache = URL(fileURLWithPath: NSHomeDirectory(), isDirectory: true)
                .appendingPathComponent(".cache/cargo", isDirectory: true)
            paths.append(cargoCache.appendingPathComponent("debug/librelay_ffi.dylib").path)
            paths.append(cargoCache.appendingPathComponent("debug/deps/librelay_ffi.dylib").path)
            paths.append("Rust/relay-ffi/target/debug/librelay_ffi.dylib")
            paths.append("Rust/relay-ffi/target/release/librelay_ffi.dylib")
            return paths
        }()

        for candidate in candidates {
            if let handle = dlopen(candidate, RTLD_NOW | RTLD_LOCAL) {
                return handle
            }
        }

        return nil
    }

    private static func loadSymbol<T>(_ name: String, from handle: UnsafeMutableRawPointer) -> T? {
        guard let symbol = dlsym(handle, name) else { return nil }
        return unsafeBitCast(symbol, to: T.self)
    }
}

private extension Optional where Wrapped == RelayProcessState {
    func unwrap(_ message: String) throws -> RelayProcessState {
        guard let value = self else {
            throw NSError(domain: "FFRelay", code: -1, userInfo: [
                NSLocalizedDescriptionKey: message
            ])
        }
        return value
    }
}
