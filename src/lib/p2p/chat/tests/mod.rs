
#[cfg(test)]
mod tests {
    use super::super::*; // Import items from the parent module (chat)
    use git2::{Repository, Signature, Time};
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use serde_json::json;
    use nostr_sdk_0_37_0::SecretKey;
    //use hex;

    // Helper function to create a dummy git repository for testing
    fn create_dummy_repo(path: &Path) -> Repository {
        let repo = Repository::init(path).unwrap();
        {
            let mut index = repo.index().unwrap();
            let oid = index.write_tree().unwrap();
            let tree = repo.find_tree(oid).unwrap();
            let signature = Signature::new("Test User", "test@example.com", &Time::new(123456789, 0)).unwrap();
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit",
                &tree,
                &[],
            ).unwrap();
        }
        repo
    }

    #[test]
    fn test_byte_array_to_hex_string() {
        let byte_array = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb,
            0xcc, 0xdd, 0xee, 0xff,
        ];
        let expected_hex = "0123456789abcdeffedcba987654321000112233445566778899aabbccddeeff";
        assert_eq!(byte_array_to_hex_string(&byte_array), expected_hex);

        let zero_array = [0u8; 32];
        let expected_zero_hex = "0000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(byte_array_to_hex_string(&zero_array), expected_zero_hex);
    }

    #[test]
    fn test_serialize_deserialize_commit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("test_repo");
        let repo = create_dummy_repo(&repo_path);

        let head = repo.head().unwrap();
        let obj = head.resolve().unwrap().peel(git2::ObjectType::Commit).unwrap();
        let commit = obj.peel_to_commit().unwrap();

        let serialized = serialize_commit(&commit).unwrap();
        let deserialized_commit = deserialize_commit(&repo, &serialized).unwrap();

        assert_eq!(commit.id(), deserialized_commit.id());
        assert_eq!(commit.message(), deserialized_commit.message());
        assert_eq!(commit.author().name(), deserialized_commit.author().name());
        assert_eq!(commit.committer().name(), deserialized_commit.committer().name());

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_generate_nostr_keys_from_commit_hash() {
        let commit_hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let keys = generate_nostr_keys_from_commit_hash(commit_hash).unwrap();
        let expected_private_key_hex = format!("{:0>64}", commit_hash);
        let secret_key_option: Option<SecretKey> = Some(keys.secret_key().clone());
        let secret_key = secret_key_option.expect("Secret key should exist");
        // assert_eq!(secret_key.to_string(), expected_private_key_hex);

        let short_commit_hash = "12345";
        let keys_short = generate_nostr_keys_from_commit_hash(short_commit_hash).unwrap();
        let expected_private_key_hex_short = format!("{:0>64}", short_commit_hash);
        let secret_key_option_short: Option<SecretKey> = Some(keys_short.secret_key().clone());
        let secret_key_short = secret_key_option_short.expect("Secret key should exist for short hash");
        // assert_eq!(secret_key_short.to_string(), expected_private_key_hex_short);
    }

    #[test]
    fn test_parse_json() {
        let json_string = r#"{"key": "value", "number": 123, "boolean": true}"#;
        let value = parse_json(json_string).unwrap();
        assert!(value.is_object());
        assert_eq!(value["key"].as_str().unwrap(), "value");
        assert_eq!(value["number"].as_i64().unwrap(), 123);
        assert_eq!(value["boolean"].as_bool().unwrap(), true);

        let invalid_json = r#"{invalid}"#;
        assert!(parse_json(invalid_json).is_err());
    }

    #[test]
    fn test_split_value_by_newline() {
        let value_string = json!("line1
line2
line3");
        let lines = split_value_by_newline(&value_string).unwrap();
        assert_eq!(lines, vec!["line1", "line2", "line3"]);

        let value_single_line = json!("single line");
        let lines_single = split_value_by_newline(&value_single_line).unwrap();
        assert_eq!(lines_single, vec!["single line"]);

        //let value_empty_string = json!(r"");
        //let lines_empty = split_value_by_newline(&value_empty_string).unwrap();
        //assert_eq!(lines_empty, vec![""]);

        let value_not_string = json!(123);
        assert!(split_value_by_newline(&value_not_string).is_none());
    }

    #[test]
    fn test_value_to_string() {
        assert_eq!(value_to_string(&json!(null)), "null");
        assert_eq!(value_to_string(&json!(true)), "true");
        assert_eq!(value_to_string(&json!(123)), "123");
        assert_eq!(value_to_string(&json!("hello")), "hello");
        assert_eq!(value_to_string(&json!([1, "two", true])), r#"[1, two, true]"#);
        assert_eq!(value_to_string(&json!({"a": 1, "b": "two"})), r#"{"a": 1, "b": two}"#);
    }

    #[test]
    fn test_split_json_string() {
        let value_string = json!("one,two,three");
        let parts = split_json_string(&value_string, ",");
        assert_eq!(parts, vec!["one", "two", "three"]);

        let value_no_separator = json!("singleword");
        let parts_no_sep = split_json_string(&value_no_separator, ",");
        assert_eq!(parts_no_sep, vec!["singleword"]);

        let value_empty_string = json!("");
        let parts_empty = split_json_string(&value_empty_string, ",");
        assert_eq!(parts_empty, vec![""]);

        let value_not_string = json!(123);
        let parts_not_string = split_json_string(&value_not_string, ",");
        assert_eq!(parts_not_string, vec![""]);
    }

    #[test]
    fn test_msg_set_kind() {
        let msg = Msg::default();
        assert_eq!(msg.kind, MsgKind::Chat);
        let new_msg = msg.set_kind(MsgKind::System);
        assert_eq!(new_msg.kind, MsgKind::System);
    }

    #[test]
    fn test_msg_set_content() {
        let msg = Msg::default();
        assert_eq!(msg.content[0], "");
        let new_msg = msg.set_content("Hello World".to_string(), 0);
        assert_eq!(new_msg.content[0], "Hello World");
    }

    #[test]
    fn test_msg_display_trait() {
        let msg_chat = Msg::default().set_content("test chat".to_string(), 0);
        assert_eq!(format!("{}", msg_chat), format!("{}: test chat", *USER_NAME));

        let msg_join = Msg::default().set_kind(MsgKind::Join);
        assert_eq!(format!("{}", msg_join), format!("{} joined!", *USER_NAME));

        let msg_system = Msg::default().set_content("system message".to_string(), 0).set_kind(MsgKind::System);
        assert_eq!(format!("{}", msg_system), "[System] system message");
    }
}
