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
//  Created by Vapor
//  Modified by swift-libp2p
//

import FluentKit
public import LibP2P

extension Application {
    /// Automatically runs forward migrations without confirmation.
    /// This can be triggered by passing `--auto-migrate` flag.
    public func autoMigrate() async throws {
        try await self.migrator.setupIfNeeded().flatMap {
            self.migrator.prepareBatch()
        }.get()
    }

    /// Automatically runs reverse migrations without confirmation.
    /// This can be triggered by passing `--auto-revert` during boot.
    public func autoRevert() async throws {
        try await self.migrator.setupIfNeeded().flatMap {
            self.migrator.revertAllBatches()
        }.get()
    }
}
