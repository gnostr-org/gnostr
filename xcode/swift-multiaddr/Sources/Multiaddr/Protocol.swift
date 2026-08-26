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

import Foundation
import Multicodec

public typealias MultiaddrProtocol = Codecs

enum BitSize {
    case fixed(bits: Int)
    case variableLengthPrefixed
    case zero
}

extension MultiaddrProtocol {
    func isMultiaddrProtocol() -> Bool {
        switch self {
        case .ip4, .tcp, .dns, .dns4, .dns6, .dnsaddr, .udp, .dccp, .ip6, .ip6zone, .ipcidr, .quic, .quic_v1,
            .webtransport, .certhash, .sctp, .p2p_circuit, .udt, .utp, .unix, .p2p, .ipfs, .http, .https, .onion,
            .onion3, .garlic64, .garlic32, .p2p_webrtc_direct, .tls, .sni, .noise, .ws, .wss, .plaintextv2,
            .webrtc_direct, .webrtc:
            return true
        case .p2p_websocket_star, .p2p_webrtc_star:  //, .p2pWebrtcDirect, .p2pCircuit, .memory:
            return true
        default:
            return false
        }
    }

    /// The number of bits that an address of this protocol will consume.
    func size() -> BitSize {
        switch self {
        case .ip4:
            return .fixed(bits: 32)
        case .tcp, .udp, .dccp, .sctp:
            return .fixed(bits: 16)
        case .ip6:
            return .fixed(bits: 128)
        case .onion:
            return .fixed(bits: 96)
        case .onion3:
            return .fixed(bits: 296)
        case .ipcidr:
            return .fixed(bits: 8)
        case .ipfs, .dns, .dnsaddr, .dns4, .dns6, .unix, .p2p, .sni, .ip6zone:
            return .variableLengthPrefixed
        case .certhash:
            return .variableLengthPrefixed
        case .garlic32, .garlic64:
            return .variableLengthPrefixed
        default:
            return .zero
        }
    }

    func packedCode() -> Data {
        code.varIntData()
    }

    func isEqual(_ codec: MultiaddrProtocol) -> Bool {
        switch self {
        case .ipfs, .p2p:
            return codec == .p2p || codec == .ipfs
        default:
            return self == codec
        }
    }
}

// MARK: - Helpers

extension String {
    func isMultiaddrProtocol() -> Bool {
        (try? Codecs(self).isMultiaddrProtocol()) ?? false
        //return MultiaddrProtocol.allCases.map{$0.rawValue}.contains(self)
    }
}
