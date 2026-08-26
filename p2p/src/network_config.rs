use std::env;
use std::str::FromStr;

use clap::ValueEnum;
use libp2p::{Multiaddr, PeerId, StreamProtocol};

pub const DEFAULT_DISCOVERY_TOPIC: &str = "gnostr/p2p/presence";
pub const DEFAULT_CHAT_TOPIC: &str = "gnostr-dev";

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Network {
    Kusama,
    Polkadot,
    Ipfs,
    Ursa,
}

impl Network {
    #[rustfmt::skip]
    pub fn bootnodes(&self) -> Vec<(Multiaddr, PeerId)> {
        match self {
            Network::Kusama => {
                vec![
                    ("/dns/p2p.cc3-0.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWDgtynm4S9M3m6ZZhXYu2RrWKdvkCSScc25xKDVSg1Sjd").unwrap()),
                    ("/dns/p2p.cc3-1.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWNpGriWPmf621Lza9UWU9eLLBdCFaErf6d4HSK7Bcqnv4").unwrap()),
                    ("/dns/p2p.cc3-2.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWLmLiB4AenmN2g2mHbhNXbUcNiGi99sAkSk1kAQedp8uE").unwrap()),
                    ("/dns/p2p.cc3-3.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWEGHw84b4hfvXEfyq4XWEmWCbRGuHMHQMpby4BAtZ4xJf").unwrap()),
                    ("/dns/p2p.cc3-4.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWF9KDPRMN8WpeyXhEeURZGP8Dmo7go1tDqi7hTYpxV9uW").unwrap()),
                    ("/dns/p2p.cc3-5.kusama.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWDiwMeqzvgWNreS9sV1HW3pZv1PA7QGA7HUCo7FzN5gcA").unwrap()),
                    ("/dns/kusama-bootnode-0.paritytech.net/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWSueCPH3puP2PcvqPJdNaDNF3jMZjtJtDiSy35pWrbt5h").unwrap()),
                    ("/dns/kusama-bootnode-1.paritytech.net/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWQKqane1SqWJNWMQkbia9qiMWXkcHtAdfW5eVF8hbwEDw").unwrap())
                ]
            }
            Network::Polkadot => {
                vec![
                    // ("/dns/p2p.cc1-0.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWEdsXX9657ppNqqrRuaCHFvuNemasgU5msLDwSJ6WqsKc").unwrap()),
                    ("/dns/p2p.cc1-1.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWAtx477KzC8LwqLjWWUG6WF4Gqp2eNXmeqAG98ehAMWYH").unwrap()),
                    ("/dns/p2p.cc1-2.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWAGCCPZbr9UWGXPtBosTZo91Hb5M3hU8v6xbKgnC5LVao").unwrap()),
                    ("/dns/p2p.cc1-3.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWJ4eyPowiVcPU46pXuE2cDsiAmuBKXnFcFPapm4xKFdMJ").unwrap()),
                    ("/dns/p2p.cc1-4.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWNMUcqwSj38oEq1zHeGnWKmMvrCFnpMftw7JzjAtRj2rU").unwrap()),
                    ("/dns/p2p.cc1-5.polkadot.network/tcp/30100".parse().unwrap(), FromStr::from_str("12D3KooWDs6LnpmWDWgZyGtcLVr3E75CoBxzg1YZUPL5Bb1zz6fM").unwrap()),
                    ("/dns/cc1-0.parity.tech/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWSz8r2WyCdsfWHgPyvD8GKQdJ1UAiRmrcrs8sQB3fe2KU").unwrap()),
                    ("/dns/cc1-1.parity.tech/tcp/30333".parse().unwrap(), FromStr::from_str("12D3KooWFN2mhgpkJsDBuNuE5427AcDrsib8EoqGMZmkxWwx3Md4").unwrap()),
                ]
            }
            Network::Ipfs => {
                vec![
                    ("/ip4/104.131.131.82/tcp/4001".parse().unwrap(), FromStr::from_str("QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ").unwrap()),
                    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN").unwrap()),
                    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa").unwrap()),
                    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb").unwrap()),
                    ("/dnsaddr/bootstrap.libp2p.io".parse().unwrap(), FromStr::from_str("QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt").unwrap()),
                ]
            }
            Network::Ursa => {
                vec![
                    ("/dns/bootstrap-node-0.ursa.earth/tcp/6009".parse().unwrap(), FromStr::from_str("12D3KooWDji7xMLia6GAsyr4oiEFD2dd3zSryqNhfxU3Grzs1r9p").unwrap()),
                ]
            }
        }
    }

    pub fn protocol(&self) -> Option<String> {
        match self {
            Network::Kusama => Some("/ksmcc3/kad".to_string()),
            Network::Polkadot => Some("/dot/kad".to_string()),
            Network::Ipfs => None,
            Network::Ursa => Some("/ursa/kad/0.0.1".to_string()),
        }
    }
}

pub fn resolve_protocol_name(
    network: Option<Network>,
    protocol: Option<String>,
    protocol_version: Option<String>,
) -> String {
    let protocol_name = protocol
        .or_else(|| network.and_then(|n| n.protocol()))
        .unwrap_or_else(|| IPFS_PROTO_NAME.to_string());

    match protocol_version {
        Some(version) => match protocol_name.rsplit_once('/') {
            Some((base, last)) if !base.is_empty() && looks_like_version(last) => {
                format!("{base}/{version}")
            }
            _ => format!("{protocol_name}/{version}"),
        },
        None => protocol_name,
    }
}

pub fn active_protocol_name() -> String {
    resolve_protocol_name(
        None,
        env::var("GNOSTR_P2P_PROTOCOL").ok(),
        env::var("GNOSTR_P2P_PROTOCOL_VERSION").ok(),
    )
}

pub fn active_discovery_topic() -> String {
    normalized_env_value("GNOSTR_P2P_DISCOVERY_TOPIC").unwrap_or_else(|| DEFAULT_DISCOVERY_TOPIC.to_string())
}

pub fn active_chat_topic() -> String {
    normalized_env_value("GNOSTR_P2P_CHAT_TOPIC").unwrap_or_else(|| DEFAULT_CHAT_TOPIC.to_string())
}

pub const IPFS_BOOTNODES: [&str; 6] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUuqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
    "12D3KooWH1URV3uTNQW6SZ1UFDnHN8NXwznAA8JeETTBm8oimjh9",
    "12D3KooWFhXabKDwALpzqMbto94sB7rvmZ6M28hs9Y9xSopDKwQr",
];
pub const IPFS_PROTO_NAME: StreamProtocol = StreamProtocol::new("/ipfs/kad/1.0.0");

fn normalized_env_value(key: &str) -> Option<String> {
    let value = env::var(key).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn looks_like_version(segment: &str) -> bool {
    !segment.is_empty() && segment.chars().all(|c| c.is_ascii_digit() || c == '.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_protocols_match_expected_values() {
        assert_eq!(Network::Kusama.protocol().as_deref(), Some("/ksmcc3/kad"));
        assert_eq!(Network::Polkadot.protocol().as_deref(), Some("/dot/kad"));
        assert_eq!(Network::Ipfs.protocol(), None);
        assert_eq!(Network::Ursa.protocol().as_deref(), Some("/ursa/kad/0.0.1"));
    }

    #[test]
    fn bootnodes_have_expected_counts() {
        assert_eq!(Network::Kusama.bootnodes().len(), 8);
        assert_eq!(Network::Polkadot.bootnodes().len(), 7);
        assert_eq!(Network::Ipfs.bootnodes().len(), 5);
        assert_eq!(Network::Ursa.bootnodes().len(), 1);
    }

    #[test]
    fn ipfs_bootnodes_include_bootstrap_nodes() {
        let bootnodes = Network::Ipfs.bootnodes();
        assert!(bootnodes.iter().any(|(_, peer)| peer.to_string() == "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN"));
    }

    #[test]
    fn resolve_protocol_name_replaces_last_segment() {
        assert_eq!(
            resolve_protocol_name(
                Some(Network::Ipfs),
                Some("/ipfs/kad".to_string()),
                Some("0.0.1".to_string())
            ),
            "/ipfs/kad/0.0.1"
        );
        assert_eq!(
            resolve_protocol_name(
                None,
                Some("/custom/protocol/1.2.3".to_string()),
                Some("9.9.9".to_string())
            ),
            "/custom/protocol/9.9.9"
        );
    }
}
