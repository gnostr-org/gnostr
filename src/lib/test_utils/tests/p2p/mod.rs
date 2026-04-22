#[cfg(test)]
mod mod_tests {
    use crate::p2p::Network;
    use libp2p::{identity, Multiaddr, PeerId};
    use std::str::FromStr;

    #[test]
    fn test_network_kusama_bootnodes() {
        let bootnodes = Network::Kusama.bootnodes();
        assert!(!bootnodes.is_empty());
        // Check a few known bootnodes for Kusama
        let expected_addr1: Multiaddr = "/dns/p2p.cc3-0.kusama.network/tcp/30100".parse().unwrap();
        let expected_peer_id1: PeerId = FromStr::from_str("12D3KooWDgtynm4S9M3m6ZZhXYu2RrWKdvkCSScc25xKDVSg1Sjd").unwrap();
        assert!(bootnodes.contains(&(expected_addr1, expected_peer_id1)));

        let expected_addr2: Multiaddr = "/dns/kusama-bootnode-0.paritytech.net/tcp/30333".parse().unwrap();
        let expected_peer_id2: PeerId = FromStr::from_str("12D3KooWSueCPH3puP2PcvqPJdNaDNF3jMZjtJtDiSy35pWrbt5h").unwrap();
        assert!(bootnodes.contains(&(expected_addr2, expected_peer_id2)));
    }

    #[test]
    fn test_network_polkadot_bootnodes() {
        let bootnodes = Network::Polkadot.bootnodes();
        assert!(!bootnodes.is_empty());
        // Check a few known bootnodes for Polkadot
        let expected_addr1: Multiaddr = "/dns/p2p.cc1-1.polkadot.network/tcp/30100".parse().unwrap();
        let expected_peer_id1: PeerId = FromStr::from_str("12D3KooWAtx477KzC8LwqLjWWUG6WF4Gqp2eNXmeqAG98ehAMWYH").unwrap();
        assert!(bootnodes.contains(&(expected_addr1, expected_peer_id1)));

        let expected_addr2: Multiaddr = "/dns/cc1-0.parity.tech/tcp/30333".parse().unwrap();
        let expected_peer_id2: PeerId = FromStr::from_str("12D3KooWSz8r2WyCdsfWHgPyvD8GKQdJ1UAiRmrcrs8sQB3fe2KU").unwrap();
        assert!(bootnodes.contains(&(expected_addr2, expected_peer_id2)));
    }

    #[test]
    fn test_network_ipfs_bootnodes() {
        let bootnodes = Network::Ipfs.bootnodes();
        assert!(!bootnodes.is_empty());
        // Check a few known bootnodes for IPFS
        let expected_addr1: Multiaddr = "/ip4/104.131.131.82/tcp/4001".parse().unwrap();
        let expected_peer_id1: PeerId = FromStr::from_str("QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ").unwrap();
        assert!(bootnodes.contains(&(expected_addr1, expected_peer_id1)));

        let expected_addr2: Multiaddr = "/dnsaddr/bootstrap.libp2p.io".parse().unwrap();
        let expected_peer_id2: PeerId = FromStr::from_str("QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN").unwrap();
        assert!(bootnodes.contains(&(expected_addr2, expected_peer_id2)));
    }

    #[test]
    fn test_network_ursa_bootnodes() {
        let bootnodes = Network::Ursa.bootnodes();
        assert!(!bootnodes.is_empty());
        // Check the known bootnode for Ursa
        let expected_addr: Multiaddr = "/dns/bootstrap-node-0.ursa.earth/tcp/6009".parse().unwrap();
        let expected_peer_id: PeerId = FromStr::from_str("12D3KooWDji7xMLia6GAsyr4oiEFD2dd3zSryqNhfxU3Grzs1r9p").unwrap();
        assert!(bootnodes.contains(&(expected_addr, expected_peer_id)));
    }

    #[test]
    fn test_network_kusama_protocol() {
        assert_eq!(Network::Kusama.protocol(), Some("/ksmcc3/kad".to_string()));
    }

    #[test]
    fn test_network_polkadot_protocol() {
        assert_eq!(Network::Polkadot.protocol(), Some("/dot/kad".to_string()));
    }

    #[test]
    fn test_network_ipfs_protocol() {
        assert_eq!(Network::Ipfs.protocol(), None);
    }

    #[test]
    fn test_network_ursa_protocol() {
        assert_eq!(Network::Ursa.protocol(), Some("/ursa/kad/0.0.1".to_string()));
    }

    // Note: Testing async_prompt requires mocking external HTTP requests, which is complex
    // without a dedicated mocking library. For now, we'll skip direct testing of async_prompt
    // due to its reliance on external network calls and the current tool limitations.
    // If a test URL and a way to mock ureq were available, it would look something like this:
    /*
    #[tokio::test]
    async fn test_async_prompt_success() {
        // Mock ureq to return a specific response for a given URL
        // let mock_url = "http://example.com/mempool";
        // let mock_response = "{"data": "some_mempool_data"}";
        // mock_ureq::mock::get(mock_url).return_with_json(serde_json::json!({ "data": "some_mempool_data" })).mount();

        // let result = async_prompt(mock_url.to_string()).await;
        // assert_eq!(result, mock_response);
    }
    */
}