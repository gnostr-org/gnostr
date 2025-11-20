#[cfg(test)]
mod tests {
    use gnostr::{
        get_blockhash, get_blockheight_sync, get_dirs, get_relays, get_relays_by_nip,
        get_relays_offline, get_relays_online, get_relays_paid, get_relays_public,
        get_weeble_sync, get_wobble_sync, Config,
    };
    use gnostr_types::{Filter, EventKind, Id, PublicKey, PublicKeyHex, Signature, Unixtime};
    use gnostr::internal;
    use secp256k1::XOnlyPublicKey;
    
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            // Initialize logger for tests
            let _ = env_logger::builder().is_test(true).try_init();
        });
    }

    #[test]
    fn test_get_dirs() {
        setup();
        let _project_dirs = get_dirs().unwrap();
    }

    // These tests rely on external API calls, which can be flaky.
    // They primarily check if the functions return Ok and a non-empty string.
    #[test]
    fn test_get_relays_by_nip() {
        setup();
        let result = get_relays_by_nip("1");
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_get_relays() {
        setup();
        let result = get_relays();
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_get_relays_online() {
        setup();
        let result = get_relays_online();
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_get_relays_public() {
        setup();
        let result = get_relays_public();
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_get_relays_paid() {
        setup();
        let result = get_relays_paid();
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_get_relays_offline() {
        setup();
        let result = get_relays_offline();
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    // These tests rely on external API calls, which can be flaky.
    #[test]
    fn test_get_weeble_sync() {
        setup();
        let result = get_weeble_sync();
        assert!(result.is_ok());
        // We can't assert the exact value, but we can check if it's a valid float string
        let value = result.unwrap();
        assert!(!value.is_empty());
        assert!(value.parse::<f64>().is_ok());
    }

    #[test]
    fn test_get_wobble_sync() {
        setup();
        let result = get_wobble_sync();
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(!value.is_empty());
        assert!(value.parse::<f64>().is_ok());
    }

    #[test]
    fn test_get_blockheight_sync() {
        setup();
        let result = get_blockheight_sync();
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(!value.is_empty());
        assert!(value.parse::<f64>().is_ok());
    }

    #[test]
    fn test_get_blockhash() {
        setup();
        let result = get_blockhash();
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_config_build_with_query() {
        setup();
        let args = vec!["program_name".to_string(), "test_query".to_string()];
        let config = Config::build(&args).unwrap();
        assert_eq!(config.query, "test_query");
    }

    #[test]
    fn test_config_build_no_args_returns_err() {
        setup();
        let args = vec!["program_name".to_string()];
        let config = Config::build(&args);
        assert!(config.is_err());
    }

    #[test]
    fn test_filters_to_wire() {
        setup();
        // Mock a filter
        use gnostr_types::{Filter, EventKind, Id, PublicKeyHex};
        let mut filter = Filter::new();
        filter.add_author(&PublicKeyHex::try_from_string("3bf0c63fcb93463407af97a5e5ee64cb5d9f0ab53a26a9645f5739c5ab8fd97b".to_string()).unwrap());
        filter.add_event_kind(EventKind::TextNote);

        let filters = vec![filter];
        let wire_message = gnostr::internal::filters_to_wire(filters);

        // This will be a partial match because of the dynamic SubscriptionId
        // We'll check for the static parts of the JSON
        assert!(wire_message.contains(r#""REQ",""#));
        assert!(wire_message.contains(r#""kinds":[1]"#));
        assert!(wire_message.contains(r#""authors":["3bf0c63fcb93463407af97a5e5ee64cb5d9f0ab53a26a9645f5739c5ab8fd97b"]"#));
    }

    #[test]
    fn test_event_to_wire() {
        setup();
        use gnostr_types::{Event, EventKind, PublicKey, Signature, Unixtime, Id};
        use secp256k1::XOnlyPublicKey;

        // Mock an Event
        let public_key = XOnlyPublicKey::from_slice(&hex::decode("3bf0c63fcb93463407af97a5e5ee64cb5d9f0ab53a26a9645f5739c5ab8fd97b").unwrap()).unwrap();
        let event = Event {
            id: Id([0; 32]), // Dummy ID
            pubkey: PublicKey::from_bytes(public_key.serialize().as_slice(), true).unwrap(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![],
            content: "Hello Nostr!".to_string(),
            sig: Signature::zeroes(), // Dummy signature
        };

        let wire_message = gnostr::internal::event_to_wire(event);

        // Check for static parts of the JSON
        assert!(wire_message.contains(r#""EVENT",""#));
        assert!(wire_message.contains(r#""content":"Hello Nostr!""#));
        assert!(wire_message.contains(r#""kind":1"#));
    }
}
