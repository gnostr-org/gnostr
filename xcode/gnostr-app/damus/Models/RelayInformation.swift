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
    let fees: [String: [[String: Int]]]? // For pay-to-relay fees
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
}
