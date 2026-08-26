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
import LibP2PCore
import NIO

protocol MessageExtractable {
    func messageBytes() -> ByteBuffer
}

protocol MessageExtractableHandler: ChannelInboundHandler where InboundOut: MessageExtractable {}

public struct YAMUX: MuxerUpgrader {

    public static let key: String = "/yamux/1.0.0"
    let application: Application

    public func upgradeConnection(_ conn: Connection, muxedPromise: EventLoopPromise<Muxer>) -> EventLoopFuture<Void> {
        conn.channel.pipeline.addHandlers(
            [
                //OutboundLoggerHandler(mode: .initiator),
                //InboundLoggerHandler(mode: .initiator),
                ByteToMessageHandler(FrameDecoder()),
                MessageToByteHandler(FrameEncoder()),
                YAMUXHandler(connection: conn, muxedPromise: muxedPromise, supportedProtocols: []),
            ],
            position: .last
        )
    }

    public func printSelf() {
        application.logger.notice("Hi I'm YAMUX v1.0.0")
    }
}

extension Application.MuxerUpgraders.Provider {
    public static var yamux: Self {
        .init { app in
            app.muxers.use {
                YAMUX(application: $0)
            }
        }
    }
}
