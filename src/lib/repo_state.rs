use std::collections::HashMap;

use anyhow::{Context, Result};
use git2::Oid;
use lazy_static::lazy_static;

pub struct RepoState {
    pub identifier: String,
    pub state: HashMap<String, String>,
    pub event: nostr_0_34_1::Event,
}

impl RepoState {
    pub fn try_from(mut state_events: Vec<nostr_0_34_1::Event>) -> Result<Self> {
        state_events.sort_by_key(|e| e.created_at);
        let event = state_events.first().context("no state events")?;
        let mut state = HashMap::new();
        for tag in &event.tags {
            if let Some(name) = tag.as_vec().first() {
                if ["refs/heads/", "refs/tags", "HEAD"]
                    .iter()
                    .any(|s| name.starts_with(*s))
                {
                    if let Some(value) = tag.as_vec().get(1) {
                        if Oid::from_str(value).is_ok() || value.contains("ref: refs/") {
                            state.insert(name.to_owned(), value.to_owned());
                        }
                    }
                }
            }
        }
        Ok(RepoState {
            identifier: event
                .identifier()
                .context("existing event must have an identifier")?
                .to_string(),
            state,
            event: event.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_0_34_1::{EventBuilder, Keys, Tag, Timestamp};
    use std::str::FromStr;

    const TEST_KEY_1_HEX: &str = "7a4c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c";
    lazy_static! {
        static ref TEST_KEY_1_KEYS: Keys = Keys::from_str(TEST_KEY_1_HEX).unwrap();
    }

    fn create_test_event(
        identifier: &str,
        mut tags: Vec<Tag>,
        created_at: u64,
    ) -> nostr_0_34_1::Event {
        let mut initial_tags = vec![Tag::identifier(identifier.to_string())];
        initial_tags.append(&mut tags);

        EventBuilder::new(
            nostr_0_34_1::Kind::Custom(30303),
            "",
            initial_tags,
        )
        .custom_created_at(Timestamp::from(created_at))
        .to_event(&TEST_KEY_1_KEYS)
        .unwrap()
    }

    #[test]
    fn test_try_from_valid_event() {
        let tags = vec![
            Tag::parse(&["refs/heads/main", "0123456789012345678901234567890123456789"]).unwrap(),
            Tag::parse(&["refs/tags/v1.0", "fedcba9876543210fedcba9876543210fedcba98"]).unwrap(),
            Tag::parse(&["HEAD", "ref: refs/heads/main"]).unwrap(),
            Tag::parse(&["other_tag", "some_value"]).unwrap(),
        ];
        let event = create_test_event("test_repo", tags, 1);
        let repo_state = RepoState::try_from(vec![event]).unwrap();

        assert_eq!(repo_state.identifier, "test_repo");
        assert_eq!(repo_state.state.len(), 3);
        assert_eq!(
            repo_state.state["refs/heads/main"],
            "0123456789012345678901234567890123456789"
        );
        assert_eq!(
            repo_state.state["refs/tags/v1.0"],
            "fedcba9876543210fedcba9876543210fedcba98"
        );
        assert_eq!(repo_state.state["HEAD"], "ref: refs/heads/main");
    }

    #[test]
    fn test_try_from_no_state_tags() {
        let tags = vec![Tag::parse(&["other_tag", "some_value"]).unwrap()];
        let event = create_test_event("test_repo", tags, 1);
        let repo_state = RepoState::try_from(vec![event]).unwrap();

        assert_eq!(repo_state.identifier, "test_repo");
        assert!(repo_state.state.is_empty());
    }

    #[test]
    fn test_try_from_invalid_oid_value() {
        let tags = vec![
            Tag::parse(&["refs/heads/main", "invalid_oid"]).unwrap(),
            Tag::parse(&["HEAD", "ref: refs/heads/main"]).unwrap(),
        ];
        let event = create_test_event("test_repo", tags, 1);
        let repo_state = RepoState::try_from(vec![event]).unwrap();

        assert_eq!(repo_state.identifier, "test_repo");
        assert_eq!(repo_state.state.len(), 1);
        assert_eq!(repo_state.state["HEAD"], "ref: refs/heads/main");
        assert!(!repo_state.state.contains_key("refs/heads/main"));
    }

    #[test]
    #[ignore]
    fn test_try_from_multiple_events_latest_is_used() {
        let tags1 = vec![Tag::parse(&["refs/heads/main", "0123456789012345678901234567890123456789"]).unwrap()];
        let event1 = create_test_event("test_repo", tags1, 1);

        let tags2 = vec![Tag::parse(&["refs/heads/main", "fedcba9876543210fedcba9876543210fedcba98"]).unwrap()];
        let event2 = create_test_event("test_repo", tags2, 2);

        let repo_state = RepoState::try_from(vec![event1, event2]).unwrap();

        assert_eq!(repo_state.identifier, "test_repo");
        assert_eq!(repo_state.state.len(), 1);
        assert_eq!(repo_state.state["refs/heads/main"], "fedcba9876543210fedcba9876543210fedcba98");
        assert_eq!(repo_state.event.created_at.as_u64(), 2);
    }

    #[test]
    fn test_try_from_no_identifier() {
        let event = EventBuilder::new(nostr_0_34_1::Kind::Custom(30303), "", vec![])
            .to_event(&TEST_KEY_1_KEYS)
            .unwrap();
        let result = RepoState::try_from(vec![event]);
        assert!(result.is_err());
    }

    #[test]
    fn test_try_from_empty_events_vec() {
        let result = RepoState::try_from(vec![]);
        assert!(result.is_err());
    }
}
