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

extension Data {

    var uint16: UInt16? {
        self.value(ofType: UInt16.self, at: 0, convertEndian: true)
    }

    var uint32: UInt32? {
        self.value(ofType: UInt32.self, at: 0, convertEndian: true)
    }

    fileprivate func value<T: BinaryInteger>(ofType: T.Type, at offset: Int, convertEndian: Bool = false) -> T? {
        let byteCount = MemoryLayout<T>.size
        let startIndex = self.index(self.startIndex, offsetBy: offset)
        let endIndex = self.index(startIndex, offsetBy: byteCount)
        guard self.endIndex >= endIndex else { return nil }
        let bytes = self[startIndex..<endIndex]
        if convertEndian {
            return bytes.reversed().reduce(0) { T($0) << 8 + T($1) }
        } else {
            return bytes.reduce(0) { T($0) << 8 + T($1) }
        }
    }
}
