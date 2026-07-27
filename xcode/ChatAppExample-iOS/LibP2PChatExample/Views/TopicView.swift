//
//  TopicView.swift
//  LibP2PChatExample
//
//  Created by Copilot on 2026-07-27.
//

import SwiftUI

struct TopicView: View {
    @EnvironmentObject var viewModel: ViewModel
    @FocusState private var isFocused
    @State private var text: String = ""
    @State private var messageIDToScrollTo: UUID?

    @ObservedObject var topic: Topic
    @State private var height: CGFloat = 53

    var body: some View {
        ZStack {
            GeometryReader { geoProxy in
                ScrollView {
                    ScrollViewReader { scrollProxy in
                        Section(footer: VStack {
                            Spacer()
                            Text("Updated at: \(topic.messages.last?.date.formatted(date: .omitted, time: .shortened) ?? "???")")
                                .foregroundColor(.secondary)
                                .font(.caption)
                        }) {
                            MessageView(viewWidth: geoProxy.size.width)
                                .padding(.horizontal)
                                .onChange(of: topic.messages) { _ in
                                    if let msgID = topic.messages.last?.id {
                                        scrollTo(messageID: msgID, shouldAnimate: true, proxy: scrollProxy)
                                    }
                                }
                                .onChange(of: isFocused) { _ in
                                    DispatchQueue.main.async {
                                        if let msgID = messageIDToScrollTo {
                                            scrollTo(messageID: msgID, shouldAnimate: true, proxy: scrollProxy)
                                        }
                                    }
                                }
                                .onAppear {
                                    if let messageID = topic.messages.last?.id {
                                        scrollTo(messageID: messageID, anchor: .bottom, shouldAnimate: false, proxy: scrollProxy)
                                        self.messageIDToScrollTo = messageID
                                    }
                                }
                        }
                        .padding(.bottom, height + 10)
                        .onChange(of: height) { _ in
                            if let id = topic.messages.last?.id {
                                scrollTo(messageID: id, shouldAnimate: true, proxy: scrollProxy)
                            }
                        }
                    }
                }
            }
            .padding(.bottom, 0)

            VStack {
                Spacer()
                toolbarView()
            }
        }
        .padding(.top, 1)
        .navigationBarTitleDisplayMode(.inline)
        .navigationTitle(topic.name)
    }

    let columns = [GridItem(.flexible(minimum: 10))]

    func MessageView(viewWidth: CGFloat) -> some View {
        LazyVGrid(columns: columns, spacing: 0) {
            ForEach(topic.messages) { message in
                let isReceived = message.type == .received
                HStack {
                    ZStack {
                        Text(message.contents)
                            .padding(.horizontal)
                            .padding(.vertical, 8)
                            .background(isReceived ? Color.primary.opacity(0.2) : Color.blue.opacity(0.7))
                            .cornerRadius(12)
                    }
                    .frame(width: viewWidth * 0.7, alignment: isReceived ? .leading : .trailing)
                    .padding(8)
                }
                .frame(maxWidth: .infinity, alignment: isReceived ? .leading : .trailing)
                .id(message.id)
            }
        }
    }

    func toolbarView() -> some View {
        VStack {
            let height2: CGFloat = 38
            HStack {
                ZStack {
                    Text(text).foregroundColor(.clear)
                        .padding(.horizontal, 14)
                        .background(GeometryReader {
                            Color.clear.preference(
                                key: ViewHeightKey.self,
                                value: $0.frame(in: .local).size.height
                            )
                        })
                    TextEditor(text: $text)
                        .padding(.horizontal, 10)
                        .frame(minHeight: height)
                        .background()
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                        .focused($isFocused)
                }
                .onPreferenceChange(ViewHeightKey.self) { newVal in
                    height = max(53.0, min(newVal, 112.0))
                }

                Button(action: sendMessage) {
                    Image(systemName: "paperplane.fill")
                        .foregroundColor(.white)
                        .frame(width: height2, height: height2)
                        .background(
                            Circle()
                                .foregroundColor(text.isEmpty ? .gray : .blue)
                        )
                }
                .disabled(text.isEmpty)
            }
            .frame(height: height)
        }
        .padding()
        .background(.thickMaterial)
    }

    func scrollTo(messageID: UUID, anchor: UnitPoint? = nil, shouldAnimate: Bool, proxy: ScrollViewProxy) {
        DispatchQueue.main.async {
            withAnimation(shouldAnimate ? Animation.easeOut : nil) {
                if let anchor = anchor {
                    proxy.scrollTo(messageID, anchor: anchor)
                } else {
                    proxy.scrollTo(messageID, anchor: UnitPoint(x: 0, y: -70.0))
                }
            }
        }
    }

    func sendMessage() {
        if let message = viewModel.send(message: text, to: topic) {
            text = ""
            messageIDToScrollTo = message.id
        }
    }

    struct ViewHeightKey: PreferenceKey {
        typealias Value = CGFloat
        static var defaultValue: CGFloat = 32.0
        static func reduce(value: inout Value, nextValue: () -> Value) {
            value += nextValue()
        }
    }
}

