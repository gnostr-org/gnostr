//
//  NostrFilter.swift
//  damus
//
//  Created by William Casarin on 2022-04-11.
//

import Foundation

struct NostrFilter: Codable {
    var ids: [String]?
    var kinds: [Int]?
    var referenced_ids: [String]?
    var pubkeys: [String]?
    var since: Int64?
    var until: Int64?
    var limit: UInt32?
    var authors: [String]?
    var hashtag: [String]? = nil
    var tags: [String: [String]]? = nil

    private enum CodingKeys : String, CodingKey {
        case ids
        case kinds
        case referenced_ids = "#e"
        case pubkeys = "#p"
        case hashtag = "#t"
        case since
        case until
        case authors
        case limit
        case tags = "#d"
    }

    public static func copy(from: NostrFilter) -> NostrFilter {
        return NostrFilter(ids: from.ids, kinds: from.kinds, referenced_ids: from.referenced_ids, pubkeys: from.pubkeys, since: from.since, until: from.until, authors: from.authors, hashtag: from.hashtag)
    }

    public static func filter_hashtag(_ htags: [String]) -> NostrFilter {
        return NostrFilter(ids: nil, kinds: nil, referenced_ids: nil, pubkeys: nil, since: nil, until: nil, authors: nil, hashtag: htags)
    }

    public static var filter_text: NostrFilter {
        return filter_kinds([1])
    }

    public static var filter_repo: NostrFilter {
        return filter_kinds([
            1,
            NostrKind.repository_announcement.rawValue,// = 30617
            NostrKind.repository_state_announcement.rawValue,// = 30618
            //NostrKind.repository_reply.rawValue,// = 1111
            NostrKind.repository_patch.rawValue,// = 1617
            NostrKind.repository_issue_open.rawValue,// = 1630
            NostrKind.repository_issue_applied.rawValue,// = 1631
            NostrKind.repository_issue_closed.rawValue,// = 1632
            NostrKind.repository_issue_draft.rawValue,// = 1633

        ])
    }

    public static var filter_profiles: NostrFilter {
        return filter_kinds([0])
    }

    public static var filter_contacts: NostrFilter {
        return filter_kinds([3])
    }

    public static func filter_authors(_ authors: [String]) -> NostrFilter {
        return NostrFilter(ids: nil, kinds: nil, referenced_ids: nil, pubkeys: nil, since: nil, until: nil, authors: authors)
    }

    public static func filter_kinds(_ kinds: [Int]) -> NostrFilter {
        return NostrFilter(ids: nil, kinds: kinds, referenced_ids: nil, pubkeys: nil, since: nil, until: nil, authors: nil)
    }

    public static func filter_since(_ val: Int64) -> NostrFilter {
        return NostrFilter(ids: nil, kinds: nil, referenced_ids: nil, pubkeys: nil, since: val, until: nil, authors: nil)
    }
}
