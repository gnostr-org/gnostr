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
// DO NOT EDIT.
// swift-format-ignore-file
// swiftlint:disable all

import Foundation
import SwiftProtobuf

fileprivate struct _GeneratedWithProtocGenSwiftVersion: SwiftProtobuf.ProtobufAPIVersionCheck {
  struct _2: SwiftProtobuf.ProtobufAPIVersion_2 {}
  typealias Version = _2
}

struct HolePunch: Sendable {
  enum Kind: Int, SwiftProtobuf.Enum, CaseIterable {
    case connect = 100
    case sync = 300

    init() { self = .connect }
  }

  var type: HolePunch.Kind = .connect
  var obsAddrs: [Data] = []
  var unknownFields = SwiftProtobuf.UnknownStorage()
  init() {}
}

extension HolePunch: SwiftProtobuf.Message, SwiftProtobuf._MessageImplementationBase, SwiftProtobuf._ProtoNameProviding {
  static let protoMessageName: String = "HolePunch"
  static let _protobuf_nameMap = SwiftProtobuf._NameMap(bytecode: "\0\u{4}type\0\u{8}obs_addrs\0")

  mutating func decodeMessage<D: SwiftProtobuf.Decoder>(decoder: inout D) throws {
    while let fieldNumber = try decoder.nextFieldNumber() {
      switch fieldNumber {
      case 1: try { try decoder.decodeSingularEnumField(value: &self.type) }()
      case 2: try { try decoder.decodeRepeatedBytesField(value: &self.obsAddrs) }()
      default: break
      }
    }
  }

  func traverse<V: SwiftProtobuf.Visitor>(visitor: inout V) throws {
    try visitor.visitSingularEnumField(value: self.type, fieldNumber: 1)
    try visitor.visitRepeatedBytesField(value: self.obsAddrs, fieldNumber: 2)
    try unknownFields.traverse(visitor: &visitor)
  }

  static func ==(lhs: HolePunch, rhs: HolePunch) -> Bool {
    lhs.type == rhs.type && lhs.obsAddrs == rhs.obsAddrs && lhs.unknownFields == rhs.unknownFields
  }
}

extension HolePunch.Kind: SwiftProtobuf._ProtoNameProviding {
  static let _protobuf_nameMap = SwiftProtobuf._NameMap(bytecode: "\0\u{7}\0CONNECT\0\u{4}SYNC\0")
}
