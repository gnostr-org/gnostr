//
//  ContentView.swift
//  swiftyapp
//
//  Created by Jonathan McKenzie on 7/9/24.
//

import SwiftUI
import RustyLib

struct ContentView: View {
    var body: some View {
        print("ContentView: body property accessed.")
        let helloMessage = rustHello()
        let sum = rustAdd(a: 10, b: 32)
        print("ContentView: rustHello() returned \(helloMessage)")
        print("ContentView: rustAdd(a: 10, b: 32) returned \(sum)")
        return VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text(helloMessage)
            Text(String(sum))
        }
        .padding()
    }
}

#Preview {
    ContentView()
}
