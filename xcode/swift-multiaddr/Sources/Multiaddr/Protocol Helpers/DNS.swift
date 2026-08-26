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
import VarInt

struct DNS {
    static func data(for address: String) -> Data {
        let addressBytes = Data(address.utf8)
        let sizeBytes = UInt64(addressBytes.count).varIntData()
        let combined = [Array(sizeBytes), Array(addressBytes)].flatMap { $0 }
        return Data(bytes: combined, count: combined.count)
    }

    static func string(for data: Data) throws -> String? {
        let buffer = Array(data)
        let decodedVarint = VarInt.uVarInt(buffer)  //Varint.readUVarInt(from: buffer)
        let expectedSize = decodedVarint.value

        let addressBytes = Array(buffer[decodedVarint.bytesRead...])
        guard addressBytes.count == expectedSize else { throw MultiaddrError.parseAddressFail }

        return String(data: Data(addressBytes), encoding: .utf8)
    }
}
