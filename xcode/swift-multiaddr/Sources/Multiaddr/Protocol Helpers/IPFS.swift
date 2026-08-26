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
import Multihash
import VarInt

struct P2P {
    static func data(for address: String) throws -> Data {
        let multihash = try (try? CID(address).multihash) ?? Multihash(multihash: address)
        return Data(putUVarInt(UInt64(multihash.value.count)) + multihash.value)
    }

    static func string(for data: Data) throws -> String {
        let varInt = uVarInt(data.byteArray)
        guard varInt.bytesRead + Int(varInt.value) == data.count else { throw MultiaddrError.invalidFormat }
        return try Multihash(multihash: data.dropFirst(varInt.bytesRead)).b58String
    }
}
