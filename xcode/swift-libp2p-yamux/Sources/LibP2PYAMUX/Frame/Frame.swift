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

import NIOCore

public struct Frame: Equatable {
    private(set) var header: Header
    var payload: ByteBuffer?
}

extension Frame {
    mutating func addFlag(_ flag: Header.Flag) {
        self.header.addFlag(flag)
    }
}

extension ByteBuffer {
    mutating func write(frame: Frame) {
        self.write(header: frame.header)
        if var payload = frame.payload {
            self.writeBuffer(&payload)
        }
    }

    /// Reads in a YAMUX Frame
    /// If the frame includes a payload, it reads in and populates the payload param
    /// If theres insufficient readable bytes for the payload, we reset the reader index to BEFORE the header.
    mutating func readFrame() throws -> Frame {
        let readerIndex = self.readerIndex

        let header = try Header(buffer: &self)
        var payload: ByteBuffer? = nil

        if header.messageType == .data {
            guard let data = self.readSlice(length: Int(header.length)) else {
                self.moveReaderIndex(to: readerIndex)
                throw YAMUX.Error.invalidPacketFormat
            }
            payload = data
        }
        return Frame(header: header, payload: payload)
    }
}
