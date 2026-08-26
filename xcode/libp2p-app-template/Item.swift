//
//  Item.swift
//  libp2p-app-template
//
//  Created by git on 5/23/26.
//

import Foundation
import SwiftData

@available(iOS 17, *)
@Model
final class Item {
    var timestamp: Date
    
    init(timestamp: Date) {
        self.timestamp = timestamp
    }
}
