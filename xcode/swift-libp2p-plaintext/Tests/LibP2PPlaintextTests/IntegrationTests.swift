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

import LibP2P
import LibP2PYAMUX
import Testing

@testable import LibP2PPlaintext

@Suite("Active Tests", .serialized, .timeLimit(.minutes(5)))
struct InternalIntegrationTests {
    fileprivate enum DeterministicPeerIDs {
        static let host = try! PeerID(marshaledPrivateKey: PeerFixtures.goPrivKey, base: .base64Pad)
        static let client = try! PeerID(marshaledPrivateKey: PeerFixtures.samplePrivKey, base: .base64Pad)
    }

    fileprivate enum PeerFixtures {
        static let goPrivKey =
            "CAASpwkwggSjAgEAAoIBAQDWBEbO8kc6a5kEks09CKPQargY3p0DCmCczoCT52/RYFqxvH9dI+s+u4ZAvF9aLWOBvFomL7jHZODPxKDrbiNCmyEbViNgZYK+PNbwh0V3ZGbB27X3q8yZtLvYA8dhcNkz/2SHBarSoC4QLA5MXUuSWtVaYMY3MzMnzBF57Jc9Ase7NvHOIUI90M7aN5izP7hxPXpZ+shiN+TyjM8mFxYONG7ZSsY3IxUhtrU5MRzFX+tp1o/gb/aa51mHf7AL3N02j5ABiYbCK97Rbwr03hsBcwgMxoDPJmP3WZ+D5yyPcOIIF1Vd7+4/f7FQJnIw3xr9/jvaFbPyDCVbBOhr9oyxAgMBAAECggEALlrgx2Q8v0+c5hux7p1XdgYXd/OHyKfPw0cLHH4NfylCm6q7X34vLvhJHO5wLMUV/3y/ffPqLu4Pr5DkVfoWExAsvJIMuY1jIzdkStbR2glaJHUlVc7VUxmNcj1nSxi5QwT3TjORC2v8bi5Mroeqnbmk6p15cW1akC0oP+NZ4rG48+WFHRqsBaBusdSOVfA+IiZUqSd1ILysJ1w7aVN3EC7jLjDG43i+P/2BcEHy8TVClGOknJL341bHe3UPdEpmeu6k6aHGlDI4blUMXahCIUh0IdZuj+Vi/TxQME9+3bKIOjQb8RCNm3U3j/uz5gs9SyTjBuYIib9Scj/jDbLh0QKBgQDfLr3go3Q/AR0jb12QjGALJz1lc9ZRX2RQJkqqmYkZwOlHHyl+YJgqOZiO80fUkN0sJ29CmKecXU4gXuHir913Fdceei1ScBSsvZpWtBLhEZXKrRJYq8U0atKUFQADDMGutyB/uGCNeNwR6VcJezHPICvHxQfmWlWHA5VIOEtRPQKBgQD1fID76SkIpF/EaJMnN2alXWWnzKhUBUPGpQtbpwgSfaCBiZ4vr3NQwKBntOOB5QwHmifNZMoqaFQLzC4B/uyTNUcQMQQ6arYav7WQXqXTmW6poTsjUSuSOPx1swsHlYX09SmUwWDfd94XF9UOU0KUfA2/c85ixzNlV5ejkFA4hQKBgEvP3uQN4hD82d8Nl2TgqkdfnvV1cdnWY4buWvK0kOPUqelk5n1tZoMBaZc1gLLuOpMjGiIvJNByyXUpheWxA7POEXLi4b5dIEjFZ0YIiVk21gEw5UiFoMl7d+ihcY2Xqbslrb507SdhZLAY6V3pITRQo06K2XIgQWlJiE4uATepAoGBALZ2vEiBnYZW5vfN4tKbUyhGq3B1pggNgbr8odyV4mIcDlk6OOGov0WeZ5ut0AyUesSLyFnaOIoc0ZuTP/8rxBwG1bMrO8FP39sx83pDX25P9PkQZixyALjGsp+pXOFeOhtAvo9azO5M4j638Bydtjc3neBX62dwOLtyx7tDYN0hAoGAVLmr3w7XMVHTfEuCSzKHyRrOaN2PAuSX31QAji1PwlwVKMylVrb8rRvBOpTicA/wXPX9Q5O/yjegqhqLT/LXAm9ziFzy5b9/9SzXPukKebXXbvc0FOmcsrcxtijlPyUzf9fKM1ShiwqqsgM9eNyZ9GWUJw2GFATCWW7pl7rtnWk="
        static let samplePrivKey =
            "CAASpgkwggSiAgEAAoIBAQC2SKo/HMFZeBml1AF3XijzrxrfQXdJzjePBZAbdxqKR1Mc6juRHXij6HXYPjlAk01BhF1S3Ll4Lwi0cAHhggf457sMg55UWyeGKeUv0ucgvCpBwlR5cQ020i0MgzjPWOLWq1rtvSbNcAi2ZEVn6+Q2EcHo3wUvWRtLeKz+DZSZfw2PEDC+DGPJPl7f8g7zl56YymmmzH9liZLNrzg/qidokUv5u1pdGrcpLuPNeTODk0cqKB+OUbuKj9GShYECCEjaybJDl9276oalL9ghBtSeEv20kugatTvYy590wFlJkkvyl+nPxIH0EEYMKK9XRWlu9XYnoSfboiwcv8M3SlsjAgMBAAECggEAZtju/bcKvKFPz0mkHiaJcpycy9STKphorpCT83srBVQi59CdFU6Mj+aL/xt0kCPMVigJw8P3/YCEJ9J+rS8BsoWE+xWUEsJvtXoT7vzPHaAtM3ci1HZd302Mz1+GgS8Epdx+7F5p80XAFLDUnELzOzKftvWGZmWfSeDnslwVONkL/1VAzwKy7Ce6hk4SxRE7l2NE2OklSHOzCGU1f78ZzVYKSnS5Ag9YrGjOAmTOXDbKNKN/qIorAQ1bovzGoCwx3iGIatQKFOxyVCyO1PsJYT7JO+kZbhBWRRE+L7l+ppPER9bdLFxs1t5CrKc078h+wuUr05S1P1JjXk68pk3+kQKBgQDeK8AR11373Mzib6uzpjGzgNRMzdYNuExWjxyxAzz53NAR7zrPHvXvfIqjDScLJ4NcRO2TddhXAfZoOPVH5k4PJHKLBPKuXZpWlookCAyENY7+Pd55S8r+a+MusrMagYNljb5WbVTgN8cgdpim9lbbIFlpN6SZaVjLQL3J8TWH6wKBgQDSChzItkqWX11CNstJ9zJyUE20I7LrpyBJNgG1gtvz3ZMUQCn3PxxHtQzN9n1P0mSSYs+jBKPuoSyYLt1wwe10/lpgL4rkKWU3/m1Myt0tveJ9WcqHh6tzcAbb/fXpUFT/o4SWDimWkPkuCb+8j//2yiXk0a/T2f36zKMuZvujqQKBgC6B7BAQDG2H2B/ijofp12ejJU36nL98gAZyqOfpLJ+FeMz4TlBDQ+phIMhnHXA5UkdDapQ+zA3SrFk+6yGk9Vw4Hf46B+82SvOrSbmnMa+PYqKYIvUzR4gg34rL/7AhwnbEyD5hXq4dHwMNsIDq+l2elPjwm/U9V0gdAl2+r50HAoGALtsKqMvhv8HucAMBPrLikhXP/8um8mMKFMrzfqZ+otxfHzlhI0L08Bo3jQrb0Z7ByNY6M8epOmbCKADsbWcVre/AAY0ZkuSZK/CaOXNX/AhMKmKJh8qAOPRY02LIJRBCpfS4czEdnfUhYV/TYiFNnKRj57PPYZdTzUsxa/yVTmECgYBr7slQEjb5Onn5mZnGDh+72BxLNdgwBkhO0OCdpdISqk0F0Pxby22DFOKXZEpiyI9XYP1C8wPiJsShGm2yEwBPWXnrrZNWczaVuCbXHrZkWQogBDG3HGXNdU4MAWCyiYlyinIBpPpoAJZSzpGLmWbMWh28+RJS6AQX6KHrK1o2uw=="
    }

    /// ***************************************
    /// Testing Internal Swift Interoperability
    /// ***************************************
    @Test() func testInternalInterop() async throws {
        let host = try makeLocalEchoHost(port: 10000)
        let client = try makeLocalClient(port: 10001)

        try await host.startup()
        try await client.startup()

        do {
            /// Fire off an echo request
            let response = try await client.newRequest(
                to: host.listenAddresses.first!.encapsulate(proto: .p2p, address: host.peerID.b58String),
                forProtocol: "/echo/1.0.0",
                withRequest: "Hello Swift LibP2P".data(using: .utf8)!,
                withHandlers: .handlers([.newLineDelimited])
            ).get()

            #expect(response == "Hello Swift LibP2P".data(using: .utf8)!)

            try await Task.sleep(for: .milliseconds(10))
        } catch {
            Issue.record(error)
        }

        try await host.asyncShutdown()
        try await client.asyncShutdown()
    }

    @Test(.timeLimit(.minutes(2)), arguments: [3, 5, 10])
    func testInternalInteropMultipleRequests_Sequentially(_ numberOfRequests: Int) async throws {
        let host = try makeLocalEchoHost(port: 10000)
        let client = try makeLocalClient(port: 10001)

        try await host.startup()
        try await client.startup()

        do {
            for _ in 0..<numberOfRequests {
                /// Fire off an echo request
                let response = try await client.newRequest(
                    to: host.listenAddresses.first!.encapsulate(proto: .p2p, address: host.peerID.b58String),
                    forProtocol: "/echo/1.0.0",
                    withRequest: "Hello Swift LibP2P".data(using: .utf8)!,
                    withHandlers: .handlers([.newLineDelimited])
                ).get()

                #expect(response == "Hello Swift LibP2P".data(using: .utf8)!)
            }

            try await Task.sleep(for: .milliseconds(10))

            let connections = try await host.connectionManager.getTotalConnectionCount().get()
            let streams = try await host.connectionManager.getTotalStreamCount().get()

            print("Connections: \(connections)")
            print("Streams: \(streams)")
            #expect(connections == 1)
            #expect(streams == numberOfRequests + 2)
        } catch {
            Issue.record(error)
        }

        try await host.asyncShutdown()
        try await client.asyncShutdown()
    }

    // Executing a bunch of concurrent requests should cascade into a single connection when appropriate (using the same transport)
    // Expected flow
    // -> First request for /ma/peer/echo/1.0.0
    //  -> No existing connections (immediately cache a pending connection for the ma)
    // -> While connection is being established
    // -> Next request comes in for the same ma
    //  -> Check for existing connections
    //      -> No available connection, but a pending one!
    //      -> Return a CapableConnection promise
    // -> Next request comes in for the same ma
    //  -> Check for existing connections
    //      -> Connection is ready / available, reuse it!
    @Test(.disabled()) func testInternalInteropMultipleRequests_Concurrently() async throws {
        let host = try makeLocalEchoHost(port: 10000)
        let client = try makeLocalClient(port: 10001)

        try await host.startup()
        try await client.startup()

        let numberOfRequests = 3

        try await withThrowingTaskGroup(of: Void.self) { group in
            for index in 0..<numberOfRequests {
                group.addTask {
                    /// Fire off an echo request
                    let response = try await client.newRequest(
                        to: host.listenAddresses.first!.encapsulate(proto: .p2p, address: host.peerID.b58String),
                        forProtocol: "/echo/1.0.0",
                        withRequest: "Hello Swift LibP2P[\(index)]".data(using: .utf8)!,
                        withHandlers: .handlers([.newLineDelimited])
                    ).get()

                    #expect(response == "Hello Swift LibP2P[\(index)]".data(using: .utf8)!)
                }
            }

            try await group.waitForAll()
        }

        try await Task.sleep(for: .milliseconds(10))

        try await host.asyncShutdown()
        try await client.asyncShutdown()
    }
}

@Suite("External Integration Tests", .externalIntegrationTestsEnabled, .serialized)
struct ExternalIntegrationTests {

    /// **************************************************
    /// Testing Internal Swift Interoperability with External Host on same LAN
    /// **************************************************
    @Test func testExternalInterop() async throws {
        let client = try makeLocalClient(port: 10000)

        // Change this to point to your host application
        let hostToDial = try Multiaddr("/ip4/192.168.1.1/tcp/10000")

        try await client.startup()

        /// Fire off an echo request
        let response = try await client.newRequest(
            to: hostToDial,
            forProtocol: "/echo/1.0.0",
            withRequest: "Hello Swift LibP2P".data(using: .utf8)!,
            withHandlers: .handlers([.newLineDelimited])
        ).get()

        #expect(response == "Hello Swift LibP2P".data(using: .utf8)!)
        print(String(data: response, encoding: .utf8) ?? "NIL")

        try await Task.sleep(for: .seconds(1))

        try await client.asyncShutdown()
    }

    /// **************************************
    ///     Testing Go Interoperability
    /// **************************************
    /// In order to run this example, use the Go-LibP2P Examples/Echo example in listening mode on port 10000
    /// - Note: Using a shell / terminal window execute the following command to get it echoed back to you
    /// ```
    /// //if you dont have the go-libp2p repo yet
    /// git clone https://github.com/libp2p/go-libp2p.git
    /// // the default example only supports Secio and PlaintextV2
    /// cd go-libp2p/examples/echo
    /// go build
    /// ./echo -l 10000 -insecure // -> I am /ip4/127.0.0.1/tcp/10000/p2p/QmQpiLteAfLv9VQHBJ4qaGNA9bVAFPBEtZDpmv4XeRtGh2
    /// ```
    /// Now run this test...
    /// - Note: Works with RSA and Ed25519 (Secp256k1 failes)
    @Test func testGoHostInterop() async throws {
        let client = try makeLocalClient(port: 10001, peerID: PeerID(.Ed25519))

        try await client.startup()

        /// Fire off an echo request to the go echo server on port 10000
        let response = try await client.newRequest(
            to: Multiaddr("/ip4/127.0.0.1/tcp/10000"),
            forProtocol: "/echo/1.0.0",
            withRequest: "Hello Swift LibP2P".data(using: .utf8)!,
            withHandlers: .handlers([.newLineDelimited])
        ).get()

        print(String(data: response, encoding: .utf8) ?? "NIL")
        #expect(response == "Hello Swift LibP2P".data(using: .utf8)!)

        try await Task.sleep(for: .seconds(1))

        client.peers.dumpAll()

        try await client.asyncShutdown()
    }

    /// **************************************
    ///     Testing JS Interoperability
    /// **************************************
    /// In order to run this example, use the JS-LibP2P Examples/Echo example in listening mode on port 10333
    /// - Note: Using a shell / terminal window execute the following command to get it echoed back to you
    /// ```
    /// //if you dont have the js-libp2p repo yet
    /// git clone https://github.com/libp2p/js-libp2p.git
    /// // the default example only supports Noise so add Plaintext
    /// cd js-libp2p/examples/echo/src
    /// nano libp2p.js
    /// // require Plaintext
    /// const Plaintext = require('libp2p/src/insecure/plaintext')
    /// // add it to the connEncryption
    /// connEncryption: [NOISE, Plaintext],
    ///
    /// node listener.js // -> I am /ip4/127.0.0.1/tcp/10000/p2p/QmQpiLteAfLv9VQHBJ4qaGNA9bVAFPBEtZDpmv4XeRtGh2
    /// ```
    /// Now run this test...
    /// - Note: I think there is a compatibility issue between JS and GO plaintext at the moment. Our Plaintext implementation works with GO (not JS)
    /// - Note: The difference has to do with what format the two implemetations expect the public key in. (Go expects a Marshaled PubKey, while JS expects an Exchange Protobuf)
    @Test func testJSInterop() async throws {
        let str = """
            {
              "id": "Qma3GsJmB47xYuyahPZPSadh1avvxfyYQwk8R3UnFrQ6aP",
              "privKey": "CAASpwkwggSjAgEAAoIBAQCaNSDOjPz6T8HZsf7LDpxiQRiN2OjeyIHUS05p8QWOr3EFUCFsC31R4moihE5HN+FxNalUyyFZU//yjf1pdnlMJqrVByJSMa+y2y4x2FucpoCAO97Tx+iWzwlZ2UXEUXM1Y81mhPbeWXy+wP2xElTgIER0Tsn/thoA0SD2u9wJuVvM7dB7cBcHYmqV6JH+KWCedRTum6O1BssqP/4Lbm2+rkrbZ4+oVRoU2DRLoFhKqwqLtylrbuj4XOI3XykMXV5+uQXz1JzubNOB9lsc6K+eRC+w8hhhDuFMgzkZ4qomCnx3uhO67KaICd8yqqBa6PJ/+fBM5Xk4hjyR40bwcf41AgMBAAECggEAZnrCJ6IYiLyyRdr9SbKXCNDb4YByGYPEi/HT1aHgIJfFE1PSMjxcdytxfyjP4JJpVtPjiT9JFVU2ddoYu5qJN6tGwjVwgJEWg1UXmPaAw1T/drjS94kVsAs82qICtFmwp52Apg3dBZ0Qwq/8qE1XbG7lLyohIbfCBiL0tiPYMfkcsN9gnFT/kFCX0LVs2pa9fHCRMY9rqCc4/rWJa1w8sMuQ23y4lDaxKF9OZVvOHFQkbBDrkquWHE4r55fchCz/rJklkPJUNENuncBRu0/2X+p4IKFD1DnttXNwb8j4LPiSlLro1T0hiUr5gO2QmdYwXFF63Q3mjQy0+5I4eNbjjQKBgQDZvZy3gUKS/nQNkYfq9za80uLbIj/cWbO+ZZjXCsj0fNIcQFJcKMBoA7DjJvu2S/lf86/41YHkPdmrLAEQAkJ+5BBNOycjYK9minTEjIMMmZDTXXugZ62wnU6F46uLkgEChTqEP57Y6xwwV+JaEDFEsW5N1eE9lEVX9nGIr4phMwKBgQC1TazLuEt1WBx/iUT83ita7obXqoKNzwsS/MWfY2innzYZKDOqeSYZzLtt9uTtp4X4uLyPbYs0qFYhXLsUYMoGHNN8+NdjoyxCjQRJRBkMtaNR0lc5lVDWl3bTuJovjFCgAr9uqJrmI5OHcCIk/cDpdWb3nWaMihVlePmiTcTy9wKBgQCU0u7c1jKkudqks4XM6a+2HAYGdUBk4cLjLhnrUWnNAcuyl5wzdX8dGPi8KZb+IKuQE8WBNJ2VXVj7kBYh1QmSJVunDflQSvNYCOaKuOeRoxzD+y9Wkca74qkbBmPn/6FFEb7PSZTO+tPHjyodGNgz9XpJJRjQuBk1aDJtlF3m1QKBgE5SAr5ym65SZOU3UGUIOKRsfDW4Q/OsqDUImvpywCgBICaX9lHDShFFHwau7FA52ScL7vDquoMB4UtCOtLfyQYA9995w9oYCCurrVlVIJkb8jSLcADBHw3EmqF1kq3NqJqm9TmBfoDCh52vdCCUufxgKh33kfBOSlXuf7B8dgMbAoGAZ3r0/mBQX6S+s5+xCETMTSNv7TQzxgtURIpVs+ZVr2cMhWhiv+n0Omab9X9Z50se8cWl5lkvx8vn3D/XHHIPrMF6qk7RAXtvReb+PeitNvm0odqjFv0J2qki6fDs0HKwq4kojAXI1Md8Th0eobNjsy21fEEJT7uKMJdovI/SErI=",
              "pubKey": "CAASpgIwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQCaNSDOjPz6T8HZsf7LDpxiQRiN2OjeyIHUS05p8QWOr3EFUCFsC31R4moihE5HN+FxNalUyyFZU//yjf1pdnlMJqrVByJSMa+y2y4x2FucpoCAO97Tx+iWzwlZ2UXEUXM1Y81mhPbeWXy+wP2xElTgIER0Tsn/thoA0SD2u9wJuVvM7dB7cBcHYmqV6JH+KWCedRTum6O1BssqP/4Lbm2+rkrbZ4+oVRoU2DRLoFhKqwqLtylrbuj4XOI3XykMXV5+uQXz1JzubNOB9lsc6K+eRC+w8hhhDuFMgzkZ4qomCnx3uhO67KaICd8yqqBa6PJ/+fBM5Xk4hjyR40bwcf41AgMBAAE="
            }
            """
        let peerID = try PeerID(fromJSON: str.data(using: .utf8)!)

        let client = try makeLocalClient(port: 10001, peerID: peerID)

        try await client.startup()

        /// Fire off an echo request to the go echo server on port 10000
        let response = try await client.newRequest(
            to: Multiaddr("/ip4/127.0.0.1/tcp/10333"),
            forProtocol: "/echo/1.0.0",
            withRequest: "Hello Swift LibP2P".data(using: .utf8)!,
            withHandlers: .handlers([.newLineDelimited])
        ).get()

        print(String(data: response, encoding: .utf8) ?? "NIL")
        #expect(response == "Hello Swift LibP2P".data(using: .utf8)!)

        try await Task.sleep(for: .seconds(1))

        client.peers.dumpAll()

        try await client.asyncShutdown()
    }
}

private func makeLocalEchoHost(port: Int) throws -> Application {
    let lib = try Application(.testing, peerID: InternalIntegrationTests.DeterministicPeerIDs.host)
    lib.environment.arguments = ["libp2p"]
    lib.security.use(.plaintextV2)
    lib.muxers.use(.yamux)
    lib.servers.use(.tcp(host: "127.0.0.1", port: port))

    lib.logger.logLevel = .notice

    lib.routes.group("echo", handlers: [.newLineDelimited]) { echo in
        echo.on("1.0.0") { req -> Response<ByteBuffer> in
            switch req.event {
            case .ready: return .stayOpen
            case .data(let data): return .respondThenClose(data)
            case .closed: return .close
            case .error(let error):
                req.logger.error("\(error)")
                return .close
            }
        }
    }

    return lib
}

private func makeLocalClient(port: Int, peerID: PeerID? = nil) throws -> Application {
    let lib = try Application(.testing, peerID: peerID ?? InternalIntegrationTests.DeterministicPeerIDs.client)
    lib.environment.arguments = ["libp2p"]
    lib.security.use(.plaintextV2)
    lib.muxers.use(.yamux)
    lib.servers.use(.tcp(host: "127.0.0.1", port: port))

    lib.logger.logLevel = .notice

    return lib
}

struct TestHelper {
    static var integrationTestsEnabled: Bool {
        if let b = ProcessInfo.processInfo.environment["PerformIntegrationTests"], b == "true" {
            return true
        }
        return false
    }
}

extension Trait where Self == ConditionTrait {
    /// This test is only available when the `PerformIntegrationTests` environment variable is set to `true`
    public static var externalIntegrationTestsEnabled: Self {
        enabled(
            if: TestHelper.integrationTestsEnabled,
            "This test is only available when the `PerformIntegrationTests` environment variable is set to `true`"
        )
    }
}
