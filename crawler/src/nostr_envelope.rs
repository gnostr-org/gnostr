/// Nostr-over-gossip envelope parsing for crawler interop with IPFS/P2P nodes.

/// Default gossip topic used for Nostr events in the hybrid protocol.
pub const DEFAULT_NOSTR_TOPIC: &str = "kubo-hybrid";

/// Parse a pipe-delimited gossip envelope: `topic|sender_peer_id|payload`.
pub fn parse_envelope(s: &str) -> Option<(&str, &str, &str)> {
    let mut parts = s.splitn(3, '|');
    let topic = parts.next()?;
    let peer = parts.next()?;
    let payload = parts.next()?;
    if topic.is_empty() || peer.is_empty() || payload.is_empty() {
        return None;
    }
    Some((topic, peer, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_envelope() {
        let s = "kubo-hybrid|12D3KooW|{\"kind\":1}";
        let (t, p, pl) = parse_envelope(s).unwrap();
        assert_eq!(t, "kubo-hybrid");
        assert_eq!(p, "12D3KooW");
        assert_eq!(pl, "{\"kind\":1}");
    }

    #[test]
    fn parse_invalid() {
        assert!(parse_envelope("only|two").is_none());
        assert!(parse_envelope("||").is_none());
    }
}
