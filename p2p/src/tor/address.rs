use arti_client::{DangerouslyIntoTorAddr, IntoTorAddr, TorAddr};
use libp2p::{core::multiaddr::Protocol, multiaddr::Onion3Addr, Multiaddr};
use std::net::SocketAddr;

pub fn dangerous_extract(multiaddr: &Multiaddr) -> Option<TorAddr> {
    if let Some(tor_addr) = safe_extract(multiaddr) {
        return Some(tor_addr);
    }

    let mut protocols = multiaddr.into_iter();

    let tor_addr = try_to_socket_addr(&protocols.next()?, &protocols.next()?)?
        .into_tor_addr_dangerously()
        .ok()?;

    Some(tor_addr)
}

pub fn safe_extract(multiaddr: &Multiaddr) -> Option<TorAddr> {
    let mut protocols = multiaddr.into_iter();

    let tor_addr = try_to_domain_and_port(&protocols.next()?, &protocols.next())?
        .into_tor_addr()
        .ok()?;

    Some(tor_addr)
}

fn libp2p_onion_address_to_domain_and_port<'a>(
    onion_address: &'a Onion3Addr<'_>,
) -> (&'a str, u16) {
    let hash = data_encoding::BASE32.encode(onion_address.hash());
    let onion_domain = format!("{hash}.onion");
    let onion_domain = Box::leak(onion_domain.into_boxed_str());

    (onion_domain, onion_address.port())
}

fn try_to_domain_and_port<'a>(
    maybe_domain: &'a Protocol,
    maybe_port: &Option<Protocol>,
) -> Option<(&'a str, u16)> {
    match (maybe_domain, maybe_port) {
        (
            Protocol::Dns(domain) | Protocol::Dns4(domain) | Protocol::Dns6(domain),
            Some(Protocol::Tcp(port)),
        ) => Some((domain.as_ref(), *port)),
        (Protocol::Onion3(domain), _) => Some(libp2p_onion_address_to_domain_and_port(domain)),
        _ => None,
    }
}

fn try_to_socket_addr(maybe_ip: &Protocol, maybe_port: &Protocol) -> Option<SocketAddr> {
    match (maybe_ip, maybe_port) {
        (Protocol::Ip4(ip), Protocol::Tcp(port)) => Some(SocketAddr::from((*ip, *port))),
        (Protocol::Ip6(ip), Protocol::Tcp(port)) => Some(SocketAddr::from((*ip, *port))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arti_client::TorAddr;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn extract_correct_address_from_dns() {
        let addresses = [
            "/dns/ip.tld/tcp/10".parse().unwrap(),
            "/dns4/dns.ip4.tld/tcp/11".parse().unwrap(),
            "/dns6/dns.ip6.tld/tcp/12".parse().unwrap(),
        ];

        let actual = addresses
            .iter()
            .filter_map(safe_extract)
            .collect::<Vec<_>>();

        assert_eq!(
            &[
                TorAddr::from(("ip.tld", 10)).unwrap(),
                TorAddr::from(("dns.ip4.tld", 11)).unwrap(),
                TorAddr::from(("dns.ip6.tld", 12)).unwrap(),
            ],
            actual.as_slice()
        );
    }

    #[test]
    fn extract_correct_address_from_ips() {
        let addresses = ["/ip4/127.0.0.1/tcp/10".parse().unwrap(), "/ip6/::1/tcp/10".parse().unwrap()];

        let actual = addresses
            .iter()
            .filter_map(dangerous_extract)
            .collect::<Vec<_>>();

        assert_eq!(
            &[
                TorAddr::dangerously_from((Ipv4Addr::LOCALHOST, 10)).unwrap(),
                TorAddr::dangerously_from((Ipv6Addr::LOCALHOST, 10)).unwrap(),
            ],
            actual.as_slice()
        );
    }
}
