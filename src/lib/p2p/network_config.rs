use std::str::FromStr;
use libp2p::{Multiaddr, PeerId, StreamProtocol};
use clap::ValueEnum;

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

pub const IPFS_BOOTNODES: [&str; 6] = [
    "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUuqanj75Nb",
    "QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
    "12D3KooWH1URV3uTNQW6SZ1UFDnHN8NXwznAA8JeETTBm8oimjh9",
    "12D3KooWFhXabKDwALpzqMbto94sB7rvmZ6M28hs9Y9xSopDKwQr",
];
pub const IPFS_PROTO_NAME: StreamProtocol = StreamProtocol::new("/ipfs/kad/1.0.0");
