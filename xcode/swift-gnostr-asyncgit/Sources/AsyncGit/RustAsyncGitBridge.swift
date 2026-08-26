import Foundation
import GnostrTypes
#if canImport(Darwin)
import Darwin
#endif

private struct RustEnvelope<T: Decodable>: Decodable {
    let ok: Bool
    let data: T?
    let error: String?
}

private typealias RustStringFn = @convention(c) (UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
private typealias RustNoteFn = @convention(c) (UnsafePointer<CChar>, UnsafePointer<CChar>, UInt8) -> UnsafeMutablePointer<CChar>?
private typealias RustFreeFn = @convention(c) (UnsafeMutablePointer<CChar>) -> Void

final class RustAsyncGitBridge: @unchecked Sendable {
    static let shared = RustAsyncGitBridge()

    private let handle: UnsafeMutableRawPointer?
    private let repoStateFn: RustStringFn?
    private let defaultNotesRefFn: RustStringFn?
    private let gitNoteEventIdFn: RustStringFn?
    private let generateGitNoteEventFn: RustNoteFn?
    private let freeFn: RustFreeFn?

    private init() {
        self.handle = Self.openLibrary()
        if let handle {
            self.repoStateFn = Self.loadSymbol("asyncgit_repo_state_json", from: handle)
            self.defaultNotesRefFn = Self.loadSymbol("asyncgit_default_notes_ref_json", from: handle)
            self.gitNoteEventIdFn = Self.loadSymbol("asyncgit_git_note_event_id_json", from: handle)
            self.generateGitNoteEventFn = Self.loadSymbol("asyncgit_generate_git_note_event_json", from: handle)
            self.freeFn = Self.loadSymbol("asyncgit_string_free", from: handle)
        } else {
            self.repoStateFn = nil
            self.defaultNotesRefFn = nil
            self.gitNoteEventIdFn = nil
            self.generateGitNoteEventFn = nil
            self.freeFn = nil
        }
    }

    var isAvailable: Bool {
        self.handle != nil
            && self.repoStateFn != nil
            && self.defaultNotesRefFn != nil
            && self.gitNoteEventIdFn != nil
            && self.generateGitNoteEventFn != nil
            && self.freeFn != nil
    }

    func repoState(at path: String) -> RepoState? {
        guard let value: String = self.call(self.repoStateFn, input: path) else { return nil }
        return RepoState(rawValue: value)
    }

    func defaultNotesRef(at path: String) -> String? {
        self.call(self.defaultNotesRefFn, input: path)
    }

    func gitNoteEventID(commitID: String) -> String? {
        self.call(self.gitNoteEventIdFn, input: commitID)
    }

    func generateGitNoteEvent(note: GitNote, privateKeyHex: String, powTargetBits: UInt8) throws -> Event {
        try NIP34.rustGenerateGitNoteEvent(note: note, privateKeyHex: privateKeyHex, powTargetBits: powTargetBits)
    }

    private func call<T: Decodable>(_ fn: RustStringFn?, input: String) -> T? {
        guard let fn else { return nil }
        let inputCString = input.cString(using: .utf8)!
        let result = inputCString.withUnsafeBufferPointer { buffer -> T? in
            guard let base = buffer.baseAddress else { return nil }
            let rawResult = fn(base)
            guard let rawResult else { return nil }
            defer { self.freeFn?(rawResult) }
            let json = String(cString: rawResult)
            guard let data = json.data(using: .utf8) else { return nil }
            let decoder = JSONDecoder()
            if T.self == String.self {
                let envelope = try? decoder.decode(RustEnvelope<String>.self, from: data)
                guard let envelope, envelope.ok, let value = envelope.data else { return nil }
                return value as? T
            }
            return nil
        }
        return result
    }

    private func call<T: Decodable>(
        _ fn: RustNoteFn?,
        input: String,
        extraInput: String,
        powTargetBits: UInt8
    ) -> T? {
        guard let fn else { return nil }
        let noteCString = input.cString(using: .utf8)!
        let keyCString = extraInput.cString(using: .utf8)!
        return noteCString.withUnsafeBufferPointer { noteBuffer in
            keyCString.withUnsafeBufferPointer { keyBuffer in
                guard let noteBase = noteBuffer.baseAddress, let keyBase = keyBuffer.baseAddress else {
                    return nil
                }
                let rawResult = fn(noteBase, keyBase, powTargetBits)
                guard let rawResult else { return nil }
                defer { self.freeFn?(rawResult) }
                let json = String(cString: rawResult)
                guard let data = json.data(using: .utf8) else { return nil }
                let decoder = JSONDecoder()
                if T.self == String.self {
                    let envelope = try? decoder.decode(RustEnvelope<String>.self, from: data)
                    guard let envelope, envelope.ok, let value = envelope.data else { return nil }
                    return value as? T
                }
                return nil
            }
        }
    }

    private static func openLibrary() -> UnsafeMutableRawPointer? {
        let env = ProcessInfo.processInfo.environment
        let candidates: [String] = {
            if let explicit = env["ASYNCGIT_FFI_LIBRARY"], !explicit.isEmpty {
                return [explicit]
            }
            return [
                "Rust/asyncgit-ffi/target/debug/libasyncgit_ffi.dylib",
                "Rust/asyncgit-ffi/target/release/libasyncgit_ffi.dylib",
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
