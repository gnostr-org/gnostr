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

// MARK: Chunks of Collection (used in exporting PEM strings)
public struct ChunksOfCountCollection<Base: Collection> {
    public typealias Element = Base.SubSequence

    @usableFromInline
    internal let base: Base

    @usableFromInline
    internal let chunkCount: Int

    @usableFromInline
    internal var endOfFirstChunk: Base.Index

    ///  Creates a view instance that presents the elements of `base` in
    ///  `SubSequence` chunks of the given count.
    ///
    /// - Complexity: O(*n*), because the start index is pre-computed.
    @inlinable
    internal init(_base: Base, _chunkCount: Int) {
        self.base = _base
        self.chunkCount = _chunkCount

        // Compute the start index upfront in order to make start index a O(1)
        // lookup.
        self.endOfFirstChunk =
            _base.index(
                _base.startIndex,
                offsetBy: _chunkCount,
                limitedBy: _base.endIndex
            ) ?? _base.endIndex
    }
}

extension Collection {
    /// Returns a `ChunksOfCountCollection<Self>` view presenting the elements in
    /// chunks with count of the given count parameter.
    ///
    /// - Parameter count: The size of the chunks. If the `count` parameter is
    ///   evenly divided by the count of the base `Collection` all the chunks will
    ///   have the count equals to size. Otherwise, the last chunk will contain
    ///   the remaining elements.
    ///
    ///     let c = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    ///     print(c.chunks(ofCount: 5).map(Array.init))
    ///     // [[1, 2, 3, 4, 5], [6, 7, 8, 9, 10]]
    ///
    ///     print(c.chunks(ofCount: 3).map(Array.init))
    ///     // [[1, 2, 3], [4, 5, 6], [7, 8, 9], [10]]
    ///
    /// - Complexity: O(*n*), because the start index is pre-computed.
    @inlinable
    public func chunks(ofCount count: Int) -> ChunksOfCountCollection<Self> {
        precondition(count > 0, "Cannot chunk with count <= 0!")
        return ChunksOfCountCollection(_base: self, _chunkCount: count)
    }
}

extension ChunksOfCountCollection: Collection {
    public struct Index {
        @usableFromInline
        internal let baseRange: Range<Base.Index>

        @inlinable
        internal init(_baseRange: Range<Base.Index>) {
            self.baseRange = _baseRange
        }
    }

    /// - Complexity: O(1)
    @inlinable
    public var startIndex: Index {
        Index(_baseRange: base.startIndex..<endOfFirstChunk)
    }

    @inlinable
    public var endIndex: Index {
        Index(_baseRange: base.endIndex..<base.endIndex)
    }

    /// - Complexity: O(1)
    @inlinable
    public subscript(i: Index) -> Element {
        precondition(i != endIndex, "Index out of range")
        return base[i.baseRange]
    }

    @inlinable
    public func index(after i: Index) -> Index {
        precondition(i != endIndex, "Advancing past end index")
        let baseIdx =
            base.index(
                i.baseRange.upperBound,
                offsetBy: chunkCount,
                limitedBy: base.endIndex
            ) ?? base.endIndex
        return Index(_baseRange: i.baseRange.upperBound..<baseIdx)
    }
}

extension ChunksOfCountCollection.Index: Comparable {
    @inlinable
    public static func == (
        lhs: ChunksOfCountCollection.Index,
        rhs: ChunksOfCountCollection.Index
    ) -> Bool {
        lhs.baseRange.lowerBound == rhs.baseRange.lowerBound
    }

    @inlinable
    public static func < (
        lhs: ChunksOfCountCollection.Index,
        rhs: ChunksOfCountCollection.Index
    ) -> Bool {
        lhs.baseRange.lowerBound < rhs.baseRange.lowerBound
    }
}

extension ChunksOfCountCollection.Index: Hashable where Base.Index: Hashable {}

extension ChunksOfCountCollection: LazySequenceProtocol, LazyCollectionProtocol
where Base: LazySequenceProtocol {}
