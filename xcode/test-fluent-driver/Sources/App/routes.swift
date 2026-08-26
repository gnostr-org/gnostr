import LibP2P

/// Route Handlers
func routes(_ app: Application) throws {
    
    /// Define a group that will handle the `echo` protocol
    app.group("echo") { echo in
        
        /// Within the `echo` group, we can specify a version handler
        /// - In this case we define our version to be `1.0.0`
        /// - Also, we know all inbound and outbound data should be delimited with a new-line character so we can install the `.newLineDelimited` channel handler.
        /// - The route is handled by a closure that accepts a `Request` and returns any `ResponseEncodable` object, a `String` in this case.
        echo.on("1.0.0", handlers: [.newLineDelimited]) { req -> Response<String> in
            
            /// Make sure we only handle inbound streams (libp2p can pass outbound stream to this same handler if you'd like to reuse code)
            guard req.streamDirection == .inbound else { return .close }
            
            /// `Request`s have a `logger` attached to them. Here we use the logger to print a detailed description of the `Request` for more info...
            req.logger.info("\(req.detailedDescription)")
            
            /// Stream `Request`s consist of different events. This route handler will be called each time an event happens
            switch req.event {
            /// Being notified of a new/ready stream is useful in a few situations.
            /// - When we're the initiator of a stream, we can use this moment to send our data to the remote peer
            /// - Or sometimes we're expected to respond to an empty inbound request (such as with the `ipfs/id/1.0.0` protocol), this is the perfect time to pass the data along to the remote peer.
            case .ready:
                return .stayOpen
            
            /// The `.data` event gets passed along everytime there's a new batch of data ready to be handled
            case .data(let payload):
                if let str = String(data: Data(payload.readableBytesView), encoding: .utf8) {
                    req.logger.notice("Echoing: \(str)")
                    return .respondThenClose(str)
                } else {
                    req.logger.error("Invalid UTF8 String Encountered")
                    return .close
                }
                
            /// `.close` gets called when a stream closes (either write or read)
            /// `.error` gets called when an error has occured on the stream
            case .closed, .error:
                return .close
                
            }
        }
    }
}
