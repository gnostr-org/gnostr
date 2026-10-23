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

/// Extract `ipfs://<cid>` references from a text.
pub fn extract_ipfs_cids(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for token in text.split_whitespace() {
        if let Some(rest) = token.strip_prefix("ipfs://") {
            let cid = rest.trim_end_matches(|c: char| !c.is_alphanumeric());
            if !cid.is_empty() {
                out.push(cid.to_string());
            }
        }
    }
    out
}

#[cfg(test)]
mod cid_tests {
    use super::*;
    #[test]
    fn extracts_cid() {
        let t = "see ipfs://QmTest123 and ipfs://bafyABC";
        let c = extract_ipfs_cids(t);
        assert_eq!(c, vec!["QmTest123", "bafyABC"]);
    }
}
