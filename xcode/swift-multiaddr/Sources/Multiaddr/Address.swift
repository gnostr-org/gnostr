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
//  Created by Luke Reichold
//  Modified by Brandon Toms on 5/1/22.
//

import CID
import Foundation
import Multicodec
import Multihash
import VarInt

public struct Address: Equatable, Sendable {
    let addrProtocol: MultiaddrProtocol
    let address: String?

    init(addrProtocol: MultiaddrProtocol, addressData: Data) {
        self.addrProtocol = addrProtocol
        guard !addressData.isEmpty else {
            self.address = nil
            return
        }
        self.address = try? Address.unpackAddress(addressData, for: addrProtocol)
    }

    init(addrProtocol: MultiaddrProtocol, address: String? = nil) throws {
        self.addrProtocol = addrProtocol
        switch addrProtocol {
        case .p2p, .ipfs:
            //Ensure addy is a valid CID or Multihash compliant String and store it as a b58 String if so...
            guard let address = address, !address.isEmpty else { throw MultiaddrError.parseAddressFail }
            guard ((try? CID(address)) != nil) || ((try? Multihash(multihash: address)) != nil) else {
                throw MultiaddrError.parseAddressFail
            }
            self.address = address
        case .certhash:
            // Ensure Certhash is a valid Multihash
            guard let address = address, !address.isEmpty else { throw MultiaddrError.parseAddressFail }
            guard (try? Multihash(multihash: address)) != nil else { throw MultiaddrError.parseAddressFail }
            self.address = address
        default:
            if var address = address {
                if address.hasSuffix("/") { address.removeLast() }
                if address.isEmpty { self.address = nil } else { self.address = address }
            } else {
                self.address = nil
            }
        }
        let _ = try Address.binaryPackedAddress(self.address, for: self.addrProtocol)
    }

    func binaryPacked() throws -> Data {
        let bytes = [addrProtocol.packedCode(), try Address.binaryPackedAddress(self.address, for: self.addrProtocol)]
            .compactMap { $0 }.flatMap { $0 }
        return Data(bytes: bytes, count: bytes.count)
    }

    public static func == (lhs: Address, rhs: Address) -> Bool {
        switch (lhs.addrProtocol, rhs.addrProtocol) {
        case (.certhash, .certhash):
            do {
                guard let leftAddress = lhs.address, let rightAddress = rhs.address else { return false }
                return try Multihash(multihash: leftAddress).value == Multihash(multihash: rightAddress).value
            } catch {
                return false
            }
        case (.p2p, .p2p), (.ipfs, .ipfs), (.p2p, .ipfs), (.ipfs, .p2p):
            do {
                guard let leftAddress = lhs.address, let rightAddress = rhs.address else { return false }
                return try CID(leftAddress).multihash == CID(rightAddress).multihash
            } catch {
                return false
            }
        default:
            return lhs.addrProtocol == rhs.addrProtocol && lhs.address == rhs.address
        }
    }
}

extension Address {

    static private func unpackAddress(_ addressData: Data, for addrProtocol: MultiaddrProtocol) throws -> String? {
        switch addrProtocol {
        case .tcp, .udp, .dccp, .sctp:
            guard addressData.count == 2 else { throw MultiaddrError.parseAddressFail }
            guard let uint16 = addressData.uint16 else { throw MultiaddrError.parseAddressFail }
            return String(uint16.bigEndian)
        case .ip4:
            return try IPv4.string(for: addressData)
        case .ip6:
            return try IPv6.string(for: addressData)
        case .ip6zone:
            guard !addressData.isEmpty else { throw MultiaddrError.parseAddressFail }
            let varInt = VarInt.uVarInt(addressData.byteArray)
            guard Int(varInt.value) + varInt.bytesRead == addressData.count else {
                throw MultiaddrError.parseAddressFail
            }
            guard let address = String(data: Data(addressData.dropFirst(varInt.bytesRead)), encoding: .utf8) else {
                throw MultiaddrError.invalidFormat
            }
            guard address.count > 0, !address.contains("/") else { throw MultiaddrError.invalidFormat }
            return address
        case .ipcidr:
            guard addressData.count == 1 else { throw MultiaddrError.parseAddressFail }
            let ipMask = addressData.asString(base: .base10)
            return ipMask
        case .onion:
            return try Onion.string(for: addressData)
        case .onion3:
            return try Onion3.string(for: addressData)
        case .garlic32:
            guard !addressData.isEmpty else { throw MultiaddrError.parseAddressFail }
            let varInt = VarInt.uVarInt(addressData.byteArray)
            guard Int(varInt.value) + varInt.bytesRead == addressData.count else {
                throw MultiaddrError.parseAddressFail
            }
            return try Garlic32.string(for: addressData.dropFirst(varInt.bytesRead))
        case .garlic64:
            guard !addressData.isEmpty else { throw MultiaddrError.parseAddressFail }
            let varInt = VarInt.uVarInt(addressData.byteArray)
            guard Int(varInt.value) + varInt.bytesRead == addressData.count else {
                throw MultiaddrError.parseAddressFail
            }
            return try Garlic64.string(for: addressData.dropFirst(varInt.bytesRead))
        case .p2p, .ipfs:
            return try P2P.string(for: addressData)
        case .dns, .dns4, .dns6, .dnsaddr, .sni, .unix:
            return try DNS.string(for: addressData)
        case .http, .https, .utp, .udt, .ws, .wss, .quic, .p2p_circuit:
            guard addressData.isEmpty else { throw MultiaddrError.parseAddressFail }
            return nil
        //case .http, .https:
        //    if addressData.isEmpty { return nil }
        //    guard let str = String(data: addressData, encoding: .utf8) else { throw MultiaddrError.parseAddressFail }
        //    return str
        case .certhash:
            guard !addressData.isEmpty else { throw MultiaddrError.parseAddressFail }
            let varInt = VarInt.uVarInt(addressData.byteArray)
            guard Int(varInt.value) + varInt.bytesRead == addressData.count else {
                throw MultiaddrError.parseAddressFail
            }
            return try Multihash(multihash: addressData.dropFirst(varInt.bytesRead)).asMultibase(.base16)
        default:
            throw MultiaddrError.parseAddressFail
        }
    }

    static private func binaryPackedAddress(_ address: String?, for addrProtocol: MultiaddrProtocol) throws -> Data? {
        switch addrProtocol {
        case .tcp, .udp, .dccp, .sctp:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            guard let port = UInt16(address) else { throw MultiaddrError.invalidPortValue }
            var bigEndianPort = port.bigEndian
            return Data(bytes: &bigEndianPort, count: MemoryLayout<UInt16>.size)
        case .ip4:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            return try IPv4.data(for: address)
        case .ip6:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            return try IPv6.data(for: address)
        case .ip6zone:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            guard !address.contains("/") else { throw MultiaddrError.invalidFormat }
            let data = Data(address.utf8)
            return Data(putUVarInt(UInt64(data.count)) + data)
        case .ipcidr:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            let ipMask = try Data(decoding: address, as: .base10)
            guard ipMask.count == 1 else { throw MultiaddrError.parseAddressFail }
            return ipMask
        case .onion:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            return try Onion.data(for: address)
        case .onion3:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            return try Onion3.data(for: address)
        case .garlic32:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            return try Garlic32.data(for: address)
        case .garlic64:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            return try Garlic64.data(for: address)
        case .p2p, .ipfs:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            return try P2P.data(for: address)
        case .dns, .dns4, .dns6, .dnsaddr, .sni, .unix:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            return DNS.data(for: address)
        case .http, .https, .utp, .udt, .ws, .wss, .quic, .p2p_circuit:
            guard address == nil else { throw MultiaddrError.parseAddressFail }
            return nil
        //case .http, .https:
        //    if let address = address {
        //        return Data(address.utf8)
        //    } else {
        //        return nil
        //    }
        case .certhash:
            guard let address = address else { throw MultiaddrError.parseAddressFail }
            let mh = try Multihash(multihash: address)
            return Data(VarInt.putUVarInt(UInt64(mh.value.count)) + mh.value)
        default:
            if address == nil { return nil }
            throw MultiaddrError.parseAddressFail
        }
    }

    static func byteSizeForAddress(_ proto: MultiaddrProtocol, buffer: [UInt8]) -> Int {
        switch proto.size() {
        case .zero:
            return 0
        case .fixed(let bits):
            return bits / 8
        case .variableLengthPrefixed:
            let (sizeValue, bytesRead) = VarInt.uVarInt(buffer)  //Varint.readUVarInt(from: buffer)
            return Int(sizeValue) + bytesRead
        }
    }
}

extension Address: CustomStringConvertible {
    public var description: String {
        "/" + [addrProtocol.name, address].compactMap { $0 }.joined(separator: "/")
    }

    public var codec: MultiaddrProtocol {
        addrProtocol
    }

    public var addr: String? {
        address
    }
}
