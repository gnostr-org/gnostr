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
import NIOWebSocket

/// The web socket handler to be used once the upgrade has occurred.
///
/// It handles converting WebSocketFrames into ByteBuffers to be passed along along the pipeline
/// It also is responsible for handling various websocket frame opcodes (ex: .ping/.pong, .connectionClose, etc)
/// It also masks data when in client / .initiator mode and handles unmasking data in host / .listener mode
internal final class WebSocketDuplexHandler: ChannelDuplexHandler {
    typealias InboundIn = WebSocketFrame
    typealias InboundOut = ByteBuffer
    typealias OutboundIn = ByteBuffer
    typealias OutboundOut = WebSocketFrame

    var didFireChannelActive: Bool = false
    weak var _context: ChannelHandlerContext? = nil

    let mode: LibP2P.Mode
    private var logger: Logger

    internal init(mode: LibP2P.Mode, logger: Logger) {
        self.logger = logger  //Logger(label: "Transport:WS[\(logger)]:DuplexHandler")
        self.mode = mode
        self.logger[metadataKey: "WS"] = .string("DuplexHandler")
    }

    // This is being hit, channel active won't be called as it is already added.
    public func handlerAdded(context: ChannelHandlerContext) {
        self.logger.trace("WebSocket handler added.")
        _context = context
        //self.pingTestFrameData(context: context)
    }

    public func handlerRemoved(context: ChannelHandlerContext) {
        self.logger.trace("WebSocket handler removed.")
    }

    internal func fireChannelActiveIfNecessary() {
        guard didFireChannelActive == false else { return }
        //_context?.fireChannelActive()
        _context = nil
        didFireChannelActive = true
    }

    public func channelRead(context: ChannelHandlerContext, data: NIOAny) {
        let frame = self.unwrapInboundIn(data)

        //        print("WebSocketHandler:channelRead \(frame)")
        //        print("Fin: \(frame.fin)")
        //        print("Opcode: \(frame.opcode)")
        //        print("Maks Key: \(frame.maskKey)")

        if didFireChannelActive == false {
            //context.fireChannelActive()
            didFireChannelActive = true
            _context = nil
        }

        switch frame.opcode {
        case .text, .binary:
            //Pass the received data along the pipeline
            let data = frame.unmaskedData
            //print("Websocket: Received \(text)")
            context.fireChannelRead(self.wrapInboundOut(data))

        case .connectionClose:
            self.receivedClose(context: context, frame: frame)

        //        case .ping:
        //            let frame = WebSocketFrame(fin: true, opcode: .ping, maskKey: mode == .initiator ? WebSocketMaskingKey(randomKeyLength: 4) : nil, data: context.channel.allocator.buffer(bytes: []))
        //            context.write(self.wrapOutboundOut(frame), promise: nil)

        case .continuation, .ping, .pong:
            // We ignore these frames.
            self.logger.warning("Frame Opcode: \(frame.opcode)")
            break

        default:
            // Unknown frames are errors.
            //self.closeOnError(context: context)
            self.logger.warning("Unknown Frame Opcode Receieved: \(frame.opcode):\(frame)")
        }
    }

    private func receivedClose(context: ChannelHandlerContext, frame: WebSocketFrame) {
        // Handle a received close frame. We're just going to close.
        self.logger.trace("Received Close instruction from server")
        context.close(promise: nil)
    }

    public func write(context: ChannelHandlerContext, data: NIOAny, promise: EventLoopPromise<Void>?) {
        self.logger.trace("Wrapping Outbound write in WebSocketFrame")
        self.logger.trace("Context: IsActive: \(context.channel.isActive), IsWritable: \(context.channel.isWritable)")
        let data = self.unwrapOutboundIn(data)
        //data.write(webSocketErrorCode: .protocolError)
        let frame = WebSocketFrame(
            fin: true,
            opcode: .binary,
            maskKey: mode == .initiator ? WebSocketMaskingKey.random4ByteKey : nil,
            data: data
        )
        //let frame = WebSocketFrame(fin: true, opcode: .binary, maskKey: nil, data: data)
        context.write(self.wrapOutboundOut(frame), promise: nil)
    }

    public func writeAndFlush(context: ChannelHandlerContext, data: NIOAny, promise: EventLoopPromise<Void>?) {
        self.logger.trace("Wrapping Outbound writeAndFlush in WebSocketFrame")

        let data = self.unwrapOutboundIn(data)
        //data.write(webSocketErrorCode: .protocolError)
        let frame = WebSocketFrame(
            fin: true,
            opcode: .binary,
            maskKey: mode == .initiator ? WebSocketMaskingKey.random4ByteKey : nil,
            data: data
        )
        //let frame = WebSocketFrame(fin: true, opcode: .binary, maskKey: nil, data: data)
        context.writeAndFlush(self.wrapOutboundOut(frame), promise: nil)
    }

    //    public func flush(context: ChannelHandlerContext) {
    //        context.flush()
    //    }

    // Flush it out. This can make use of gathering writes if multiple buffers are pending
    //    public func channelWriteComplete(context: ChannelHandlerContext) {
    //        print("MSS:Write Complete")
    //        context.flush()
    //    }
}
