import Foundation

struct RelayInformation: Codable, Identifiable {
    var id: String?
    let name: String?
    let description: String?
    let pubkey: String?
    let contact: String?
    let supportedNips: [Int]?
    let software: String?
    let version: String?
    
    struct Limitation: Codable {
        let maxMessageLength: Int?
        let maxSubscriptions: Int?
        let maxFilters: Int?
        let maxLimit: Int?
        let maxSubidLength: Int?
        let minPowDifficulty: Int?
        let authRequired: Bool?
        let paymentRequired: Bool?
    }
    
    let limitation: Limitation?
    struct Fee: Codable {
        let amount: Int64
        let unit: String
        let period: Int?
    }

    let fees: [String: [Fee]]?
}

extension RelayInformation {
    /// Checks if the relay supports a specific NIP (Nostr Improvement Proposal).
    func supports(nip: Int) -> Bool {
        return supportedNips?.contains(nip) ?? false
    }
    
    /// Helper for common NIPs to improve code readability
    var supportsSearch: Bool { supports(nip: 50) }
    var supportsAuth: Bool { supports(nip: 42) }
    var supportsDelegation: Bool { supports(nip: 26) }
    var supportsNIP34: Bool { supports(nip: 34) }
}

func relay_information_request(_ relayURL: URL) throws -> URLRequest {
    guard let url = normalized_relay_url(relayURL.absoluteString),
          var components = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
        throw URLError(.badURL)
    }
    components.scheme = components.scheme == "wss" ? "https" : "http"
    guard let informationURL = components.url else {
        throw URLError(.badURL)
    }
    var request = URLRequest(url: informationURL)
    request.timeoutInterval = 15
    request.setValue("application/nostr+json", forHTTPHeaderField: "Accept")
    return request
}

func fetch_relay_information(_ relayURL: URL) async throws -> RelayInformation {
    let request = try relay_information_request(relayURL)
    let (data, response) = try await URLSession.shared.data(for: request)
    guard let response = response as? HTTPURLResponse,
          (200..<300).contains(response.statusCode) else {
        throw URLError(.badServerResponse)
    }
    let decoder = JSONDecoder()
    decoder.keyDecodingStrategy = .convertFromSnakeCase
    return try decoder.decode(RelayInformation.self, from: data)
}
