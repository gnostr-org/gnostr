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
//
//  Created by Matteo Sartori on 18/05/15.
//  Modified by Brandon Toms on 5/1/2022

// We use CryptoSwift due to swift-crypto not supporting the keccak variants of sha3)
import CryptoSwift
import Foundation
import Multibase
import Multicodec
import VarInt

public enum MultihashError: Error {
    case unknownCode
    case hashTooShort
    case hashTooLong
    case VarIntBufferTooShort
    case VarIntTooLarge
    case lengthNotSupported
    case hexConversionFail
    case inconsistentLength(Int)
}

// English language error strings.
extension MultihashError {
    var description: String {
        get {
            switch self {
            case .unknownCode:
                return "Unknown multihash code."
            case .hashTooShort:
                return "Multihash too short. Must be > 3 bytes"
            case .hashTooLong:
                return "Multihash too long. Must be < 129 bytes"
            case .VarIntBufferTooShort:
                return "Unsigned Variable Integer buffer too short."
            case .VarIntTooLarge:
                return "Unsigned Variable int is too big. Max is 64 bits."
            case .lengthNotSupported:
                return "Multihash does not yet support digests longer than 127 bytes"
            case .hexConversionFail:
                return "Error occurred in hex conversion."
            case .inconsistentLength(let len):
                return "Multihash length inconsistent. \(len)"
            }
        }
    }
}

public struct Multihash: Sendable, Equatable, CustomStringConvertible {
    public let value: [UInt8]
    private var decoded: DecodedMultihash? {
        try? decodeMultihashBuffer(value)
    }

    /// Initialize a Multihash from a raw byte array
    public init(_ buf: [UInt8]) throws {
        // Ensure we can decode it before initializing
        let _ = try decodeMultihashBuffer(buf)
        self.value = buf
    }

    /// Initialize a Multihash from a Hex Encoded String
    public init(hexString str: String) throws {
        self = try fromHexString(str)
    }

    /// Initialize a Multihash from a B58 Encoded String
    public init(b58String str: String) throws {
        self = try fromB58String(str)
    }

    /// A Multibase Encoded Hash
    public init(multibase: String, codec: Codecs) throws {
        let d = try BaseEncoding.decode(multibase)
        self.value = try encodeMultihashBuffer(Array(d.data), asHashType: codec)
        //self.value = try cast(Array(d.data)).value
    }

    /// Initialize a Multihash from a Multibase compliant Multihash String
    /// ```
    /// //Example
    /// // "f111488c2f11fb2ce392acb5b2986e640211c4690073e"
    /// //    f       11      14     88c2f11fb2ce392acb5b2986e640211c4690073e
    /// // <base16> <sha1> <20 bits> <sha1 digest>
    /// let mh = try Multihash(multihash: "f111488c2f11fb2ce392acb5b2986e640211c4690073e")
    /// print(mh.name) // => "sha1"
    /// print(mh.code) // =>  0x11
    /// print(mh.digest.hexString) // => "88c2f11fb2ce392acb5b2986e640211c4690073e"
    /// ```
    public init(multihash: String) throws {
        let raw = try BaseEncoding.decode(multihash)
        try self.init(multihash: raw.data)
    }
    /// Initialize a Multihash from a Multibase compliant Multihash Data Buffer
    public init(multihash: Data) throws {
        let buf = Array(multihash)
        try self.init(buf)
    }

    /// Initialize a Multihash from a raw string
    /// ```
    /// //Example
    /// let mh = try Multihash(raw: "multihash", hashedWith: .sha2_256)
    /// print(mh.asString(base: .base16)    // => "12209cbc07c3f991725836a3aa2a581ca2029198aa420b9d99bc0e131d9f3e2cbe47"
    /// print(mh.asString(base: .base32)    // => "CIQJZPAHYP4ZC4SYG2R2UKSYDSRAFEMYVJBAXHMZXQHBGHM7HYWL4RY="
    /// print(mh.asString(base: .base58btc) // => "QmYtUc4iTCbbfVSDNKvtQqrfyezPPnFvE33wFmutw9PBBk"
    /// print(mh.asString(base: .base64)    // => "EiCcvAfD+ZFyWDajqipYHKICkZiqQgudmbwOEx2fPiy+Rw=="
    /// ```
    public init(
        raw: String,
        hashedWith codec: Codecs,
        using encoding: String.Encoding = .utf8,
        customByteLength: Int? = nil
    ) throws {
        guard let rawData = raw.data(using: encoding) else { throw MultihashError.unknownCode }
        let d = Array(rawData)
        try self.init(raw: d, hashedWith: codec, customByteLength: customByteLength)
    }

    //    public convenience init(raw:String, hashedWith codec:Codecs, using encoding:String.Encoding = .utf8, customBitLength:Int? = nil) throws {
    //        var bytes:Int? = nil
    //        if let bits = customBitLength { bytes = bits / 8 }
    //        try self.init(raw: raw, hashedWith: codec, using: encoding, customByteLength: bytes)
    //    }

    public init(raw: Data, hashedWith codec: Codecs, customByteLength: Int? = nil) throws {
        try self.init(raw: Array(raw), hashedWith: codec, customByteLength: customByteLength)
    }

    /// Main Multihash Initializer
    public init(raw d: [UInt8], hashedWith codec: Codecs, customByteLength: Int? = nil) throws {
        var hashed: [UInt8]
        switch codec {
        case .identity:
            hashed = d
        case .md5:
            hashed = d.md5()
        case .sha1:
            hashed = d.sha1()
        case .sha2_256:
            hashed = d.sha256()
        case .sha2_512:
            hashed = d.sha512()
        case .sha3_224:
            hashed = d.sha3(.sha224)
        case .sha3_256:
            hashed = d.sha3(.sha256)
        case .sha3_384:
            hashed = d.sha3(.sha384)
        case .sha3_512:
            hashed = d.sha3(.sha512)
        case .keccak_224:
            hashed = d.sha3(.keccak224)
        case .keccak_256:
            hashed = d.sha3(.keccak256)
        case .keccak_384:
            hashed = d.sha3(.keccak384)
        case .keccak_512:
            hashed = d.sha3(.keccak512)
        default:
            print("\(codec) is not supported...")
            throw MultihashError.unknownCode
        }

        /// Constrain to custom byte length if one was specified
        if let bytes = customByteLength { hashed = Array(hashed.prefix(bytes)) }

        self.value = try encodeMultihashBuffer(hashed, asHashType: codec)
    }

    // MARK: Computed Properties

    ///The code of the Hash algorithm used to compute the digest
    public var code: Int? {
        decoded?.code
    }

    ///The code of the Hash algorithm used to compute the digest
    public var algorithm: Codecs? {
        if let c = self.code {
            return try? Codecs(c)
        } else {
            return nil
        }
    }

    ///The name of the Hash algorithm used to compute the digest
    public var name: String? {
        decoded?.name
    }

    ///Length of the digest in Bytes
    public var length: Int? {
        decoded?.length
    }

    ///The hashed digest (without codec and length prefix)
    public var digest: [UInt8]? {
        decoded?.digest
    }

    ///The entire multihash value (prefix's included) as a multibase compliant string in the specified base
    ///- Note: Includes the appropriate Multibase prefix
    public func asMultibase(_ base: BaseEncoding) -> String {
        self.value.asString(base: base, withMultibasePrefix: true)
    }

    ///The entire multihash value (prefix's included) as a string in the specified base
    /// - Note: does not include the Multibase compliant prefix
    public func asString(base: BaseEncoding) -> String {
        self.value.asString(base: base, withMultibasePrefix: false)
    }

    /// A debug description of the Multihash
    public var description: String {
        guard let n = name, let c = code, let l = length, let d = digest else { return "NIL" }
        return String(format: "Multihash: %@ 0x%X %d %@\n", n, c, l, d.asString(base: .base16))
    }

    /// Returns the entire Multihash value (prefixs included) as a hexadecimal string
    public var hexString: String {
        self.asString(base: .base16)
    }

    /// Returns the entire Multihash value (prefixs included) as a hexadecimal string
    public var b58String: String {
        self.asString(base: .base58btc)
    }

    /// Returns the entire Multihash value (prefixs included) as a hexadecimal string
    public var string: String {
        self.hexString
    }
}

public func == (lhs: Multihash, rhs: Multihash) -> Bool {
    lhs.value == rhs.value
}

extension Codecs {
    public static var supportedHashAlgorithms: [Codecs] {
        [
            .md5, .sha1, .sha2_256, .sha2_512, .sha3_224, .sha3_256, .sha3_384, .sha3_512, .keccak_224, .keccak_256,
            .keccak_384, .keccak_512,
        ]
    }

    public var defaultHashLength: Int? {
        switch self {
        case .md5: return 20
        case .sha1: return 20
        case .sha3_224, .keccak_224: return 28
        case .sha2_256, .sha3_256, .keccak_256: return 32
        case .sha3_384, .keccak_384: return 48
        case .sha2_512, .sha3_512, .keccak_512: return 64
        //handle blake2b //0x40
        //handle blake2s //0x41
        //handle blake3
        default:
            return nil
        }
    }
}

public struct DecodedMultihash {
    public let
        code: Int,
        name: String?,
        length: Int,
        digest: [UInt8]
}

/// Read and strip the unsigned variable int buffer size value from front of buffer
///
/// - Parameter buffer: The buffer prefixed with the size of the payload as an uvarint
/// - Returns: the size as an int64 and the buffer with the uvarint indicating size removed.
/// - Throws: MultihashError
private func uVarInt(buffer: [UInt8]) throws -> (UInt64, [UInt8]) {
    let (size, bytesRead) = VarInt.uVarInt(buffer)
    if bytesRead == 0 { throw MultihashError.VarIntBufferTooShort }
    if bytesRead < 0 { throw MultihashError.VarIntTooLarge }

    // Return the size as read from the uvarint and the buffer without the uvarint
    return (size, Array(buffer[bytesRead..<buffer.count]))
}

private func fromHexString(_ theString: String) throws -> Multihash {
    let str = theString.count % 2 == 1 ? theString : BaseEncoding.base16.charPrefix + theString
    let d = try BaseEncoding.decode(str).data

    let buf = Array(d)
    //let buf = try SwiftHex.decodeString(hexString: theString)

    return try cast(buf)
}

private func fromB58String(_ str: String) throws -> Multihash {
    let s = str.hasPrefix(BaseEncoding.base58btc.charPrefix) ? str : BaseEncoding.base58btc.charPrefix + str
    let d = try BaseEncoding.decode(s).data

    let buf = Array(d)
    return try cast(buf)
}

private func cast(_ buf: [UInt8]) throws -> Multihash {
    let dm = try decodeMultihashBuffer(buf)

    if validCode(dm.code) == false {
        throw MultihashError.unknownCode
    }

    return try Multihash(buf)
}

/// Decodes a Multihash compliant buffer into it's separate parts (digest, length, code and name)
/// ```
/// let multihashBuffer = Data(...) // 111488c2f11fb2ce392acb5b2986e640211c4690073e
/// let decoded = try decodeMultihashBuffer(multihashBuffer)
/// decoded.name   // => "sha1"
/// decoded.code   // => 0x11
/// decoded.digest // => 88c2f11fb2ce392acb5b2986e640211c4690073e (as hex)
/// decoded.length // => 20
/// ```
public func decodeMultihashBuffer(_ buf: [UInt8]) throws -> DecodedMultihash {

    if buf.count < 3 {
        throw MultihashError.hashTooShort
    }

    let (code, buffer) = try uVarInt(buffer: buf)
    let (digestLength, digest) = try uVarInt(buffer: buffer)

    if digestLength > Int32.max {
        throw MultihashError.hashTooLong
    }

    let dm = DecodedMultihash(code: Int(code), name: try Codecs(code).name, length: Int(digestLength), digest: digest)

    /// This is usually triggered when we try and instantiate a CID or PeerID as a Multihash...
    if dm.digest.count != dm.length {
        //print(dm.code)
        //print(dm.digest)
        //print(dm.length)
        //print(dm.name)
        //print(dm.digest.asString(base: .base58btc))
        //print("WARNING: Inconsistent Multihash Length: Digest(\(dm.digest.count)) != Length(\(dm.length))")
        throw MultihashError.inconsistentLength(dm.length)
    }

    return dm
}

/// Encode a hash digest along with the specified function code
/// Note: The length is derived from the length of the digest.
/// ```
/// let hash = Array("multihash".data(using: .utf8).sha1()) // 88c2f11fb2ce392acb5b2986e640211c4690073e
/// let multihashBuffer = try encodeMultihashBuffer(hash, code: 0x11) // 111488c2f11fb2ce392acb5b2986e640211c4690073e
/// ```
public func encodeMultihashBuffer(_ buf: [UInt8], code: Int?) throws -> [UInt8] {
    if validCode(code) == false {
        throw MultihashError.unknownCode
    }

    if buf.count > 129 {
        throw MultihashError.hashTooLong
    }

    var pre = [0, 0] as [UInt8]

    pre[0] = UInt8(code!)
    pre[1] = UInt8(buf.count)
    pre.append(contentsOf: buf)

    return pre
}

/// Prepends the appropriate multihash prefixes to the given buffer
/// ```
/// let hash = Array("multihash".data(using: .utf8).sha1()) // 88c2f11fb2ce392acb5b2986e640211c4690073e
/// let multihashBuffer = try encodeMultihashBuffer(hash, asHashType: .sha1) // 111488c2f11fb2ce392acb5b2986e640211c4690073e
/// ```
public func encodeMultihashBuffer(_ buf: [UInt8], asHashType: Codecs) throws -> [UInt8] {
    try encodeMultihashBuffer(buf, code: Int(asHashType.code))
}

/// Prepends the appropriate multihash prefixes to the given buffer
/// ```
/// let hash = Array("multihash".data(using: .utf8).sha1()) // 88c2f11fb2ce392acb5b2986e640211c4690073e
/// let multihashBuffer = try encodeMultihashBuffer(hash, asHashType: "sha1") // 111488c2f11fb2ce392acb5b2986e640211c4690073e
/// ```
public func encodeMultihashBuffer(_ buf: [UInt8], asHashType: String) throws -> [UInt8] {
    try encodeMultihashBuffer(buf, asHashType: try Codecs(asHashType))
}

/// ValidCode checks whether a multihash code is valid.
private func validCode(_ code: Int?) -> Bool {

    if let c = code {
        if appCode(c) == true {
            return true
        }

        if Codecs.supportedHashAlgorithms.contains(where: { $0 == c }) {
            return true
        }
    }
    return false
}

/// AppCode checks whether a multihash code is part of the App range.
private func appCode(_ code: Int) -> Bool {
    code >= 0 && code < 0x10
}
