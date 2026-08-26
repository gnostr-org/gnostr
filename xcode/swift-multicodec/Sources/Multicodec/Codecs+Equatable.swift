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

import Foundation

extension Codecs {
    public static func == (lhs: Codecs, rhs: Codecs) -> Bool {
        lhs.rawValue == rhs.rawValue
    }

    public static func == (lhs: Codecs, rhs: Int64) -> Bool {
        lhs.rawValue == rhs
    }

    public static func == (lhs: Codecs, rhs: Int) -> Bool {
        lhs.rawValue == Int64(rhs)
    }

    public static func == (lhs: Codecs, rhs: String) -> Bool {
        lhs.name == rhs
    }

    public func isEqual(object: AnyObject?) -> Bool {
        if let obj = object as? Codecs {
            return self.rawValue == obj.rawValue
        } else if let obj = object as? Int64 {
            return self.rawValue == Int64(obj)
        } else if let obj = object as? Int {
            return self.rawValue == Int64(obj)
        } else if let obj = object as? UInt64 {
            return self.rawValue == Int64(obj)
        } else if let obj = object as? String {
            return self.name == obj
        }
        return false
    }
}

//public extension String {
//    public static func ==(lhs: Codecs, rhs: String) -> Bool {
//        return lhs.name == rhs
//    }
//}
//
//public extension Int {
//    public static func ==(lhs: Codecs, rhs: Int) -> Bool {
//        return lhs.rawValue == Int64(rhs)
//    }
//}
//
//public extension Int64 {
//    public static func ==(lhs: Codecs, rhs: Int64) -> Bool {
//        return lhs.rawValue == rhs
//    }
//}
