//
//  ContentView.swift
//  icons
//
//  Created by Randy McMillan on 9/27/24.
//

import SwiftUI

struct ContentView: View {
    var body: some View {

#if os(OSX)
        print("macOS")
        let image = NSImage(imageLiteralResourceName: "Icon")
        VStack {
            VStack {
                Image(nsImage: image)
                    .imageScale(.large)
                    .foregroundStyle(.tint)
                Text("large")
            }
            .padding()
            VStack {
                Image(nsImage: image)
                    .imageScale(.medium)
                    .foregroundStyle(.clear)
                Text("medium")
            }
            .padding()
            VStack {
                Image(nsImage: image)
                    .imageScale(.small)
                    .foregroundStyle(.clear)
                Text("small")
            }
            .padding()
        }
#elseif os(watchOS)
        print("watchOS")
#elseif os(tvOS)
        print("tvOS")
#elseif os(iOS)
#if targetEnvironment(macCatalyst)
        Text("macOS - Catalyst")
        let image = UIImage(imageLiteralResourceName: "Icon")
        VStack {
            VStack {
                Image(uiImage: image)
                    .imageScale(.large)
                Text("large")
            }
            .padding()
            VStack {
                Image(uiImage: image)
                    .imageScale(.medium)
                Text("medium")
            }
            .padding()
            VStack {
                Image(uiImage: image)
                    .imageScale(.small)
                Text("small")
            }
            .padding()
        }
#else
        let image = UIImage(imageLiteralResourceName: "Icon")
        VStack {
            VStack {
                Image(uiImage: image)
                    .imageScale(.large)
                    //.foregroundStyle(.tint)
                Text("large")
            }
            .padding()
            VStack {
                Image(uiImage: image)
                    .imageScale(.medium)
                    //.foregroundStyle(.clear)
                Text("medium")
            }
            .padding()
            VStack {
                Image(uiImage: image)
                    .imageScale(.small)
                    //.foregroundStyle(.clear)
                Text("small")
            }
            .padding()
        }
        #endif
        #endif
        }
        }
