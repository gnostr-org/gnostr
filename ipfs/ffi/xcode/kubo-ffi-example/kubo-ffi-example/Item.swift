//
//  Item.swift
//  kubo-ffi-example
//
//  Created by git on 10/19/26.
//

import Foundation
import SwiftData

@Model
final class Item {
    var timestamp: Date
    
    init(timestamp: Date) {
        self.timestamp = timestamp
    }
}
