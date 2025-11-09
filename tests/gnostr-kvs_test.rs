
#[cfg(test)]
mod tests {
    use gnostr::p2p::network_config::{Network};
    use gnostr::p2p::utils::{generate_ed25519};
    use gnostr::p2p::generate_close_peer_id;
    use libp2p::{Multiaddr, PeerId};
    use std::str::FromStr;

    #[test]
    fn test_network_kusama_bootnodes() {
        let bootnodes = Network::Kusama.bootnodes();
        assert!(!bootnodes.is_empty());
        // Check a sample bootnode
        let (addr, peer_id) = &bootnodes[0];
        assert_eq!(addr, &"/dns/p2p.cc3-0.kusama.network/tcp/30100".parse::<Multiaddr>().unwrap());
        assert_eq!(peer_id, &PeerId::from_str("12D3KooWDgtynm4S9M3m6ZZhXYu2RrWKdvkCSScc25xKDVSg1Sjd").unwrap());
    }

    #[test]
    fn test_network_polkadot_bootnodes() {
        let bootnodes = Network::Polkadot.bootnodes();
        assert!(!bootnodes.is_empty());
        // Check a sample bootnode
        let (addr, peer_id) = &bootnodes[0];
        assert_eq!(addr, &"/dns/p2p.cc1-1.polkadot.network/tcp/30100".parse::<Multiaddr>().unwrap());
        assert_eq!(peer_id, &PeerId::from_str("12D3KooWAtx477KzC8LwqLjWWUG6WF4Gqp2eNXmeqAG98ehAMWYH").unwrap());
    }

    #[test]
    fn test_network_ipfs_bootnodes() {
        let bootnodes = Network::Ipfs.bootnodes();
        assert!(!bootnodes.is_empty());
        // Check a sample bootnode
        let (addr, peer_id) = &bootnodes[0];
        assert_eq!(addr, &"/ip4/104.131.131.82/tcp/4001".parse::<Multiaddr>().unwrap());
        assert_eq!(peer_id, &PeerId::from_str("QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ").unwrap());
    }

    #[test]
    fn test_network_ursa_bootnodes() {
        let bootnodes = Network::Ursa.bootnodes();
        assert!(!bootnodes.is_empty());
        // Check a sample bootnode
        let (addr, peer_id) = &bootnodes[0];
        assert_eq!(addr, &"/dns/bootstrap-node-0.ursa.earth/tcp/6009".parse::<Multiaddr>().unwrap());
        assert_eq!(peer_id, &PeerId::from_str("12D3KooWDji7xMLia6GAsyr4oiEFD2dd3zSryqNhfxU3Grzs1r9p").unwrap());
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

    #[test]
	#[ignore]
    fn test_generate_ed25519_valid_keypair() {
        let keypair = generate_ed25519(Some(0));
        // A simple check to ensure a keypair is generated.
        // More rigorous tests would involve checking public key derivation, etc.
        assert!(!keypair.public().to_peer_id().to_base58().is_empty());
    }

    #[test]
	#[ignore]
    fn test_generate_ed25519_different_seeds_different_keypairs() {
        let keypair1 = generate_ed25519(Some(0));
        let keypair2 = generate_ed25519(Some(1));
        assert_ne!(keypair1.public().to_peer_id(), keypair2.public().to_peer_id());
    }

    #[test]
    fn test_generate_close_peer_id_valid_peer_id() {
        let bytes = [0u8; 32];
        let peer_id = generate_close_peer_id(bytes, 32);
        assert!(!peer_id.to_base58().is_empty());
    }
}
