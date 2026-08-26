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
import NIOHTTP1
import NIOWebSocket

/// The HTTP handler to be used to initiate the request.
/// This initial request will be adapted by the WebSocket upgrader to contain the upgrade header parameters.
/// Channel read will only be called if the upgrade fails.
internal final class HTTPInitialRequestHandler: ChannelInboundHandler, RemovableChannelHandler, Sendable {
    public typealias InboundIn = HTTPClientResponsePart
    public typealias OutboundOut = HTTPClientRequestPart

    public let target: Multiaddr
    private let logger: Logger

    public init(target: Multiaddr, logger: Logger) {
        var logger = logger
        logger[metadataKey: "WS"] = .string("UpgradeHandler")
        self.logger = logger
        self.target = target
    }

    public func channelActive(context: ChannelHandlerContext) {
        self.logger.trace("WS HTTP Client connected to \(context.remoteAddress!)")

        // We are connected. It's time to send the message to the server to initialize the upgrade dance.
        var headers = HTTPHeaders()
        guard let ipAddy = target.tcpAddress else {
            self.logger.error("Failed to extract Target Address for Header Host parameter")
            context.close(mode: .all, promise: nil)
            return
        }
        self.logger.info("Adding Host Header -> Host:'\(ipAddy.address):\(ipAddy.port)'")
        headers.add(name: "Host", value: "\(ipAddy.address):\(ipAddy.port)")
        headers.add(name: "Content-Type", value: "text/plain; charset=utf-8")
        headers.add(name: "Content-Length", value: "\(0)")
        //headers.add(name: "Access-Control-Allow-Origin", value: "*")
        //headers.add(name: "Connection", value: "upgrade")
        //headers.add(name: "Upgrade", value: "websocket")

        let requestHead = HTTPRequestHead(
            version: .http1_1,
            method: .GET,
            uri: "/",
            headers: headers
        )

        context.write(self.wrapOutboundOut(.head(requestHead)), promise: nil)

        let body = HTTPClientRequestPart.body(.byteBuffer(ByteBuffer()))
        context.write(self.wrapOutboundOut(body), promise: nil)

        context.writeAndFlush(self.wrapOutboundOut(.end(nil)), promise: nil)
    }

    public func channelRead(context: ChannelHandlerContext, data: NIOAny) {

        let clientResponse = self.unwrapInboundIn(data)

        self.logger.error("Upgrade failed")

        switch clientResponse {
        case .head(let responseHead):
            self.logger.error("Received status: \(responseHead.status)")
            self.logger.error("\(responseHead.description)")
        case .body(let byteBuffer):
            let string = String(buffer: byteBuffer)
            self.logger.error("Received: '\(string)' back from the server.")
        case .end:
            self.logger.error("Closing channel.")
            context.close(promise: nil)
        }
    }

    public func handlerRemoved(context: ChannelHandlerContext) {
        self.logger.trace("handler removed.")
    }

    public func errorCaught(context: ChannelHandlerContext, error: Error) {
        self.logger.error("\(error)")

        // As we are not really interested getting notified on success or failure
        // we just pass nil as promise to reduce allocations.
        context.close(promise: nil)
    }
}
