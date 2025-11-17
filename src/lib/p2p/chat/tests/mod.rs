#[cfg(test)]
mod tests {

    use super::super::*; // Import items from the parent module (chat)
    use super::super::msg::USER_NAME;
    use git2::{Commit, Signature, Time};
    
    use std::collections::HashMap;
    use std::path::Path;
    use serde_json::json;
use nostr_sdk_0_37_0::prelude::*;
use nostr_0_37_0::prelude::*;
use nostr_sdk_0_37_0::EventBuilder; // Import EventBuilder
    use nostr_sdk_0_37_0::Event; // Import Event
    use nostr_sdk_0_37_0::Kind; // Import Kind
    use chrono::{TimeZone, Utc}; // Import TimeZone and Utc for timestamp
    use crate::utils::{byte_array_to_hex_string, split_value_by_newline, value_to_string};
    use crate::legit::command::create_event_with_custom_tags;
    use crate::legit::command::create_event;


    // Helper function to create a dummy git repository for testing
    fn create_dummy_repo(path: &Path) -> Repository {
        let repo = Repository::init(path).unwrap();
        {
            let mut index = repo.index().unwrap();
            let oid = index.write_tree().unwrap();
            let signature = Signature::new("Test User", "test@example.com", &Time::new(123456789, 0)).unwrap();
            let tree_builder = repo.treebuilder(None).unwrap();
            let tree_oid = tree_builder.write().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
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
        let secret_key_str = keys.secret_key().unwrap().display_secret().to_string();
        assert_eq!(secret_key_str, expected_private_key_hex.to_string());

        let short_commit_hash = "12345";
        let keys_short = generate_nostr_keys_from_commit_hash(short_commit_hash).unwrap();
        let expected_private_key_hex_short = format!("{:0>64}", short_commit_hash);
        let secret_key_str_short = keys_short.secret_key().unwrap().display_secret().to_string();
        assert_eq!(secret_key_str_short, expected_private_key_hex_short.to_string());
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

        let value_not_string = json!(123);
        assert!(split_value_by_newline(&value_not_string).is_none());
    }

    #[test]
    fn test_value_to_string() {
        assert_eq!(value_to_string(&json!(null)), "null");
        assert_eq!(value_to_string(&json!(true)), "true");
        assert_eq!(value_to_string(&json!(123)), "123");
        assert_eq!(value_to_string(&json!("hello")), r#""hello""#);
        assert_eq!(value_to_string(&json!([1, "two", true])), r#"[1, "two", true]"#);
        assert_eq!(value_to_string(&json!({"a": 1, "b": "two"})), r#"{"a": 1, "b": "two"}"#);
    }

    #[test]
    fn test_split_json_string() {
        let value_string = json!("one,two,three");
        let parts = split_json_string(&value_string, ",");
        assert_eq!(parts, vec!["one", "two", "three"]);

        let value_no_separator = json!("singleword");
        let parts_no_sep = split_json_string(&value_no_separator, ",");
        assert_eq!(parts_no_sep, vec!["singleword"]);

        let value_empty_string = json!("") ;
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
    fn test_msg_display_trait_basic_kinds() {
        // Test Chat messages
        let msg_chat_self = Msg::default().set_content("hello".to_string(), 0);
        assert_eq!(format!("{}", msg_chat_self), format!("{}: hello", *USER_NAME));

        // To truly test right-aligned, we'd need to mock USER_NAME to be different from the sender.
        // For now, we assume it's tested implicitly by the logic.

        // Test Join
        let msg_join = Msg::default().set_kind(MsgKind::Join);
        assert_eq!(format!("{}", msg_join), format!("{} joined!", *USER_NAME));

        // Test Leave
        let msg_leave = Msg::default().set_kind(MsgKind::Leave);
        assert_eq!(format!("{}", msg_leave), format!("{} left!", *USER_NAME));

        // Test System
        let msg_system = Msg::default().set_content("system info".to_string(), 0).set_kind(MsgKind::System);
        assert_eq!(format!("{}", msg_system), "[System] system info");
        
        // Test Raw
        let msg_raw = Msg::default().set_content("raw data".to_string(), 0).set_kind(MsgKind::Raw);
        assert_eq!(format!("{}", msg_raw), "raw data");

        // Test Command
        let msg_command = Msg::default().set_content("command payload".to_string(), 0).set_kind(MsgKind::Command);
        assert_eq!(format!("{}", msg_command), format!("[Command] {}:command payload", *USER_NAME));
    }
    
    #[test]
    fn test_msg_display_trait_git_kinds() {
        let mock_sender = "test_user";
        
        // GitCommitId
        let msg_commit_id = Msg {
            from: mock_sender.to_string(),
            content: vec!["commit123".to_string(), "some value".to_string()],
            kind: MsgKind::GitCommitId,
            ..Msg::default()
        };
        // The `gen_color_by_hash` will produce a color, but we focus on the string format.
        // Format is `"{{"commit": "{}"}}"` + content[1]
        assert_eq!(format!("{}", msg_commit_id), format!("{{\"commit\": \"{}\"}} some value", msg_commit_id.content[0]));

        // GitCommitTree
        let msg_commit_tree = Msg {
            from: mock_sender.to_string(),
            content: vec!["tree456".to_string(), "some value".to_string()],
            kind: MsgKind::GitCommitTree,
            ..Msg::default()
        };
        assert_eq!(format!("{}", msg_commit_tree), format!("{{\"tree\": \"{}\"}} some value", msg_commit_tree.content[0]));

        // GitCommitParent
        let msg_commit_parent = Msg {
            from: mock_sender.to_string(),
            content: vec!["parent789".to_string(), "some value".to_string()],
            kind: MsgKind::GitCommitParent,
            ..Msg::default()
        };
        assert_eq!(format!("{}", msg_commit_parent), format!("{{\"parent\": \"{}\"}} some value", msg_commit_parent.content[0]));

        // GitCommitAuthor
        let msg_commit_author = Msg {
            from: mock_sender.to_string(),
            content: vec!["Author Name Example".to_string(), "some value".to_string()],
            kind: MsgKind::GitCommitAuthor,
            ..Msg::default()
        };
        assert_eq!(format!("{}", msg_commit_author), format!("{{\"Author\": \"{}\"}} some value", msg_commit_author.content[0]));
        
        // GitCommitName
        let msg_commit_name = Msg {
            from: mock_sender.to_string(),
            content: vec!["Committer Name Example".to_string(), "some value".to_string()],
            kind: MsgKind::GitCommitName,
            ..Msg::default()
        };
        assert_eq!(format!("{}", msg_commit_name), format!("{{\"name\": \"{}\"}} some value", msg_commit_name.content[0]));

        // GitCommitEmail
        let msg_commit_email = Msg {
            from: mock_sender.to_string(),
            content: vec!["committer@example.com".to_string(), "some value".to_string()],
            kind: MsgKind::GitCommitEmail,
            ..Msg::default()
        };
        assert_eq!(format!("{}", msg_commit_email), format!("{{\"email\": \"{}\"}} some value", msg_commit_email.content[0]));

        // GitCommitTime
        let msg_commit_time = Msg {
            from: mock_sender.to_string(),
            content: vec!["1678886400".to_string(), "some value".to_string()], // Example Unix timestamp
            kind: MsgKind::GitCommitTime,
            ..Msg::default()
        };
        assert_eq!(format!("{}", msg_commit_time), format!("{{\"time\": \"{}\"}} some value", msg_commit_time.content[0]));

        // GitCommitHeader
        let msg_commit_header = Msg {
            from: mock_sender.to_string(),
            content: vec!["Subject: Example commit
".to_string(), "some value".to_string()],
            kind: MsgKind::GitCommitHeader,
            ..Msg::default()
        };
        assert_eq!(format!("{}", msg_commit_header), format!("{{\"header\": \"{}\"}} some value", msg_commit_header.content[0]));

        // GitCommitBody
        let msg_commit_body = Msg {
            from: mock_sender.to_string(),
            content: vec!["This is the body of the commit message.

More details here.".to_string(), "some value".to_string()],
            kind: MsgKind::GitCommitBody,
            ..Msg::default()
        };
        assert_eq!(format!("{}", msg_commit_body), format!("{{\"body\": \"{}\"}} some value", msg_commit_body.content[0]));
        
        // GitCommitMessagePart (used for parts of the message)
        let msg_commit_message_part = Msg {
            from: mock_sender.to_string(),
            content: vec!["message part".to_string(), "some value".to_string()],
            kind: MsgKind::GitCommitMessagePart,
            ..Msg::default()
        };
        assert_eq!(format!("{}", msg_commit_message_part), format!("{{\"msg\": \"{}\"}} some value", msg_commit_message_part.content[0]));
    }
    
    #[test]
    fn test_create_event_with_custom_tags() {
        // Use a well-known private key for deterministic testing
        let sk_hex = "1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b"; // Example private key
        let keys = Keys::parse(sk_hex).unwrap();
        let pubkey = keys.public_key();

        let content = "Test event content";
        let mut custom_tags = HashMap::new();
        custom_tags.insert("tag1".to_string(), vec!["value1".to_string()]);
        custom_tags.insert("tag2".to_string(), vec!["value2a".to_string(), "value2b".to_string()]); // Note: Nostr tags are usually single values per tag name in this context

        // Create event asynchronously
        let event_result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            create_event_with_custom_tags(&keys, content, custom_tags).await
        });

        assert!(event_result.is_ok());
        let (event, _unsigned_event) = event_result.unwrap();

        assert_eq!(event.content, content);
        assert_eq!(event.pubkey, pubkey);
        assert_eq!(event.kind, Kind::TextNote); // Default kind used by EventBuilder::new
        
        // Check tags. Note: EventBuilder might format tags differently or only take the first value.
        // We expect tags to be present and have the correct names.
        // Let's check for the presence of tag names and their values.
        // The `create_event_with_custom_tags` implementation uses `Tag::parse` and `Tag::custom`.
        // `Tag::parse` expects a `&[&str]` where first element is tag name, second is value.
        // `Tag::custom` is similar.
        // The provided `custom_tags` HashMap has `Vec<String>` for values. The implementation pops the first value.
        
        let mut found_tags = HashMap::new();
        for tag in event.tags.iter() {
                if let Some(name) = tag.clone().to_vec().get(0).map(|s| s.clone()) {
                 // Collect all values associated with a tag name
                for value in tag.clone().to_vec().iter().skip(1) {
                    found_tags.entry(name.to_string()).or_insert_with(Vec::new).push(value.to_string());
                }
            }
        }

        // Verify tags as per the implementation's handling of HashMap<String, Vec<String>>
        // The current implementation `Tag::parse([&tag_name, &tag_values[0]]).unwrap()` suggests only the first value is used.
        assert_eq!(found_tags.get("tag1").map(|v| v[0].clone()), Some("value1".to_string()));
        assert_eq!(found_tags.get("tag2").map(|v| v[0].clone()), Some("value2a".to_string()));
        assert_eq!(found_tags.len(), 2); // Ensure no extra tags were added unintentionally
    }

    #[tokio::test]
    async fn test_create_event_defaults() {
        // Test create_event without custom tags, using default values
        let sk_hex = "2a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b"; // Another example key
        let keys = Keys::parse(sk_hex).unwrap();
        let pubkey = keys.public_key();
        let content = "Default event test";
        let custom_tags = HashMap::new(); // Empty tags

        let event_result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            create_event(keys, custom_tags, content).await
        });

        assert!(event_result.is_ok());
        assert_eq!(event.content, format!("gnostr:legit {}", pubkey));
        assert_eq!(event.pubkey, pubkey);
        assert_eq!(event.kind, Kind::TextNote); // Default kind used by EventBuilder::new
        
        let expected_tags: Vec<Tag> = vec![
            Tag::public_key(pubkey),
            Tag::custom(TagKind::Custom(Cow::from("gnostr")), "1 2 3 4 11 22 33 44".chars()),
            Tag::custom(TagKind::Custom(Cow::from("gnostr")), "1 2 3 4 11 22 33".chars()),
            Tag::custom(TagKind::Custom(Cow::from("gnostr")), "1 2 3 4 11 22".chars()),
            Tag::custom(TagKind::Custom(Cow::from("gnostr")), "1 2 3 4 11".chars()),
            Tag::custom(TagKind::Custom(Cow::from("gnostr")), "1 2 3 4".chars()),
            Tag::custom(TagKind::Custom(Cow::from("gnostr")), "1 2 3".chars()),
            Tag::custom(TagKind::Custom(Cow::from("gnostr")), "1 2".chars()),
            Tag::custom(TagKind::Custom(Cow::from("gnostr")), "1".chars()),
            Tag::custom(TagKind::Custom(Cow::from("gnostr")), "".chars()),
        ];

        // Convert to HashSet for order-independent comparison
        let event_tags_set: HashSet<Tag> = event.tags.into_iter().collect();
        let expected_tags_set: HashSet<Tag> = expected_tags.into_iter().collect();

        assert_eq!(event_tags_set, expected_tags_set);
    }

    // Add more tests for different `MsgKind` scenarios if needed
}
