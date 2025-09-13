#[cfg(test)]
mod tests {
    use gnostr::{
        get_blockhash, get_blockheight_sync, get_dirs, get_relays, get_relays_by_nip,
        get_relays_offline, get_relays_online, get_relays_paid, get_relays_public,
        get_weeble_sync, get_wobble_sync, url_to_host_and_uri, Config, Event, EventKind, Id,
        IdHex, KeySigner, PreEvent, Unixtime, run, search, print_event, Signer,
    };
    use std::io::{self, Cursor, Write};
    use std::process;
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
        let project_dirs = get_dirs().unwrap();
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
}
