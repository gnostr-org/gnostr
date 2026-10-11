//
//  TopicRow.swift
//  LibP2PChatExample
//
//  Created by Copilot on 2026-07-27.
//

import SwiftUI

struct TopicRow: View {
    @ObservedObject var topic: Topic

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                Circle()
                    .fill(Color.blue.opacity(0.2))
                    .frame(width: 54, height: 54)
                Image(systemName: "antenna.radiowaves.left.and.right")
                    .foregroundColor(.blue)
            }

            VStack(alignment: .leading, spacing: 5) {
                HStack {
                    Text(topic.name)
                        .font(.body)
                        .bold()
                    Spacer()
                    Text("\(topic.members.count) members")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                    Spacer().frame(width: 8)
                    Text(topic.lastMessage?.date.formatted(date: .numeric, time: .shortened) ?? "")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
                .padding(.trailing)
                Text(topic.lastMessage?.contents ?? "Join the gossip topic")
                    .foregroundColor(.secondary)
                    .lineLimit(2)
                    .padding(.trailing)
                    .font(.body)
            }
        }
    }
}
