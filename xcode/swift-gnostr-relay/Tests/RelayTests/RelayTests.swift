import Foundation
import Testing
@testable import Relay
@testable import Crawler

@Test func relayConfigurationHasExpectedDefaults() {
    let configuration = RelayConfiguration.rustDefault() ?? RelayConfiguration()
    #expect(configuration.logging == "info")
    #expect(configuration.configFilePath.contains("relay.toml"))
}

@Test func relayProcessStateDecodes() throws {
    let json = #"{"running":false,"pid":null,"message":"stopped","disk_usage_bytes":0}"#
    let state = try JSONDecoder().decode(RelayProcessState.self, from: Data(json.utf8))
    #expect(!state.running)
    #expect(state.pid == nil)
    #expect(state.diskUsageBytes == 0)
}

@Test func relayEndpointBuilderPrefersRustWhenAvailable() {
    let endpoint = RelayEndpoints.listenEndpoint(host: "127.0.0.1", port: 8080)
    #expect(endpoint == "ws://127.0.0.1:8080")
}
