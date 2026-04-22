//
//  NostrKind.swift
//  damus
//
//  Created by William Casarin on 2022-04-27.
//

import Foundation

enum NostrKind: Int {
    case metadata = 0
    case text = 1
    case contacts = 3
    case dm = 4
    case delete = 5
    case boost = 6
    case like = 7
    case channel_create = 40
    case channel_meta = 41
    case chat = 42
    case repository_announcement = 30617 //add GitView to view
    case repository_state_announcement = 30618 //add GitView to view
    case repository_reply = 1111
    case repository_patch = 1617 //add GitView to view
    case repository_issue_open = 1630
    case repository_issue_applied = 1631
    case repository_issue_closed = 1632
    case repository_issue_draft = 1633 //add GitView to view

    // NIP-51
    case mute_list = 10000
    case pin_list = 10001
    case relay_list = 10002
    case bookmark_list = 10003
    case community_list = 10004
    case public_chat_list = 10005
    case blocked_relay_list = 10006
    case search_relay_list = 10007
    case simple_group_list = 10009
    case relay_feed_list = 10012
    case interest_list = 10015
    case media_follow_list = 10020
    case emoji_list = 10030
    case dm_relay_list = 10050
    case good_wiki_author_list = 10101
    case good_wiki_relay_list = 10102
    
    case follow_set = 30000
    case relay_set = 30002
    case bookmark_set = 30003
    case curation_set = 30004
    case video_curation_set = 30005
    case kind_mute_set = 30007
    case interest_set = 30015
    case emoji_set = 30030
    case release_artifact_set = 30063
    case app_curation_set = 30267
    case calendar = 31924
    case starter_pack = 39089
    case multimedia_starter_pack = 39092
}
