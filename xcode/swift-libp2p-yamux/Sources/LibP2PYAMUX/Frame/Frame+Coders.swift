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

import LibP2P

internal class FrameEncoder: MessageToByteEncoder {
    public typealias OutboundIn = Frame

    public init() {}

    public func encode(data: Frame, out: inout ByteBuffer) throws {
        //print("Outbound Frame: \(data)")
        out.write(frame: data)
    }
}

internal final class FrameDecoder: ByteToMessageDecoder {
    public typealias InboundOut = Frame

    private var header: Header? = nil

    public init() {}

    public func decode(context: ChannelHandlerContext, buffer: inout ByteBuffer) throws -> DecodingState {

        // If we have a header, then we're waiting for the payload
        if let header {
            if header.messageType == .data {
                guard buffer.readableBytes >= header.length else {
                    return .needMoreData
                }
                let payload = buffer.readSlice(length: Int(header.length))

                let frame = Frame(header: header, payload: payload)
                logFrame(frame)

                // Send the message's bytes up the pipeline to the next handler.
                context.fireChannelRead(self.wrapInboundOut(frame))
            } else {
                let frame = Frame(header: header, payload: nil)
                logFrame(frame)
                // Send the message's bytes up the pipeline to the next handler.
                context.fireChannelRead(self.wrapInboundOut(frame))
            }

            // Reset the stored header
            self.header = nil
        } else {

            if let header = buffer.readHeader() {

                if header.messageType == .data {
                    guard buffer.readableBytes >= header.length else {
                        // Store the header
                        self.header = header
                        // Wait for more data
                        return .needMoreData
                    }
                    let payload = buffer.readSlice(length: Int(header.length))

                    let frame = Frame(header: header, payload: payload)
                    logFrame(frame)
                    // Send the message's bytes up the pipeline to the next handler.
                    context.fireChannelRead(self.wrapInboundOut(frame))
                } else {
                    let frame = Frame(header: header, payload: nil)
                    logFrame(frame)
                    // Send the message's bytes up the pipeline to the next handler.
                    context.fireChannelRead(self.wrapInboundOut(frame))
                }

            } else {
                return .needMoreData
            }

        }

        // We can keep going if you have more data.
        return .continue
    }

    private func logFrame(_ frame: Frame) {
        //        print("Inbound Frame: \(frame)")
        //        for message in frame.messages {
        //            print("\t- \(message)")
        //        }
    }
}
