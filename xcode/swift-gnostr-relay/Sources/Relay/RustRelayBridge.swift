import Foundation
#if canImport(Darwin)
import Darwin
#endif

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

public final class RustRelayBridge: @unchecked Sendable {
    public static let shared = RustRelayBridge()

    private let handle: UnsafeMutableRawPointer?
    private let defaultConfigurationFn: RustStringFn?
    private let listenEndpointFn: RustRelayEndpointFn?
    private let freeFn: RustFreeFn?

    private init() {
        self.handle = Self.openLibrary()
        if let handle {
            self.defaultConfigurationFn = Self.loadSymbol("relay_default_configuration_json", from: handle)
            self.listenEndpointFn = Self.loadSymbol("relay_listen_endpoint_json", from: handle)
            self.freeFn = Self.loadSymbol("relay_string_free", from: handle)
        } else {
            self.defaultConfigurationFn = nil
            self.listenEndpointFn = nil
            self.freeFn = nil
        }
    }

    public var isAvailable: Bool {
        self.handle != nil
            && self.defaultConfigurationFn != nil
            && self.listenEndpointFn != nil
            && self.freeFn != nil
    }

    public func defaultConfiguration() -> RelayConfiguration? {
        guard let json: String = self.callString(self.defaultConfigurationFn, input: "") else {
            return nil
        }
        guard let data = json.data(using: .utf8),
              let defaults = try? JSONDecoder().decode(RelayCliDefaults.self, from: data) else {
            return nil
        }
        return RelayConfiguration(logging: defaults.logging, configFilePath: defaults.configFilePath)
    }

    public func listenEndpoint(host: String, port: UInt16) -> String? {
        guard let fn = self.listenEndpointFn else { return nil }
        let hostCString = host.cString(using: .utf8)!
        return hostCString.withUnsafeBufferPointer { buffer in
            guard let base = buffer.baseAddress else { return nil }
            guard let rawResult = fn(base, port) else { return nil }
            defer { self.freeFn?(rawResult) }
            return Self.decodeEnvelopeString(rawResult)
        }
    }

    private func callString(_ fn: RustStringFn?, input: String) -> String? {
        guard let fn else { return nil }
        let inputCString = input.cString(using: .utf8)!
        return inputCString.withUnsafeBufferPointer { buffer in
            guard let base = buffer.baseAddress else { return nil }
            guard let rawResult = fn(base) else { return nil }
            defer { self.freeFn?(rawResult) }
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

    private static func openLibrary() -> UnsafeMutableRawPointer? {
        let env = ProcessInfo.processInfo.environment
        let candidates: [String] = {
            if let explicit = env["GNOSTR_RELAY_FFI_LIBRARY"], !explicit.isEmpty {
                return [explicit]
            }
            return [
                "Rust/relay-ffi/target/debug/librelay_ffi.dylib",
                "Rust/relay-ffi/target/release/librelay_ffi.dylib",
            ]
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
