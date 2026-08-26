//! Shared p2p-backed message types for chat integrations.
//!
//! Chat consumes Nostr and git-note types through the p2p facade, keeping the
//! protocol chain aligned as `asyncgit -> p2p -> chat`.

pub use gnostr_p2p::message::*;

#[cfg(test)]
mod tests {
    use super::*;
    use gnostr_asyncgit::types::{Id, MilliSatoshi, PrivateKey, PublicKey};
    use gnostr_p2p::message as p2p_message;

    #[test]
    fn reexports_p2p_nostr_types() {
        let _event_kind: EventKind = p2p_message::EventKind::TextNote;
        let _relay_message: RelayMessage = p2p_message::RelayMessage::Notice("ok".to_string());
        let _git_note: GitNote = p2p_message::GitNote {
            note_id: gnostr_asyncgit::git2::Oid::from_str(
                "b1d954d11c92c7386f040bba3937f24e64d8f9ec",
            )
            .unwrap(),
            annotated_id: gnostr_asyncgit::git2::Oid::from_str(
                "431b84edc0d2fa118d63faa3c2db9c73d630a5ae",
            )
            .unwrap(),
            notes_ref: Some("refs/notes/commits".to_string()),
            message: "chat git note".to_string(),
            author: "chat".to_string(),
            committer: "chat".to_string(),
            committer_time: 1777759186,
        };

        let _ = (_event_kind, _relay_message, _git_note);
    }

    #[test]
    fn exports_nostr_and_git_note_types() {
        let _event_kind = EventKind::TextNote;
        let _repo_ref: Option<RepoRef> = None;
        let _repo_state: Option<RepoState> = None;
        let _nip34_kind = Nip34Kind::from(REPO_ANNOUNCEMENT_KIND);
        let _nip34_event: Option<Nip34Event> = None;
        let _nip34_unsigned: Option<Nip34UnsignedEvent> = None;

        assert_eq!(
            Nip34Kind::from(gnostr_asyncgit::types::nip34::REPO_ANNOUNCEMENT_KIND),
            EventKind::RepositoryAnnouncement
        );
        assert_eq!(
            Nip34Kind::from(gnostr_asyncgit::types::nip34::REPO_STATE_KIND),
            EventKind::GitRepoAnnouncement
        );
        assert_eq!(
            Nip34Kind::from(gnostr_asyncgit::types::nip34::PULL_REQUEST_KIND),
            EventKind::Other(1618)
        );
        assert_eq!(
            Nip34Kind::from(gnostr_asyncgit::types::nip34::PULL_REQUEST_UPDATE_KIND),
            EventKind::Other(1619)
        );
        assert_eq!(
            Nip34Kind::from(gnostr_asyncgit::types::nip34::GIT_ISSUE_KIND),
            EventKind::GitIssue
        );
        assert_eq!(
            Nip34Kind::from(gnostr_asyncgit::types::nip34::GIT_REPLY_KIND),
            EventKind::GitReply
        );
        assert_eq!(
            Nip34Kind::from(gnostr_asyncgit::types::nip34::USER_GRASP_LIST_KIND),
            EventKind::Replaceable(10317)
        );
        assert_eq!(
            status_kinds().into_iter().map(u32::from).collect::<Vec<_>>(),
            vec![1630, 1631, 1632, 1633]
        );

        let _ = (
            _event_kind,
            _repo_ref,
            _repo_state,
            _nip34_kind,
            _nip34_event,
            _nip34_unsigned,
        );
    }

    #[test]
    fn builds_git_note_nostr_event() {
        let note = GitNote {
            note_id: gnostr_asyncgit::git2::Oid::from_str(
                "b1d954d11c92c7386f040bba3937f24e64d8f9ec",
            )
            .unwrap(),
            annotated_id: gnostr_asyncgit::git2::Oid::from_str(
                "431b84edc0d2fa118d63faa3c2db9c73d630a5ae",
            )
            .unwrap(),
            notes_ref: Some("refs/notes/commits".to_string()),
            message: "chat git note".to_string(),
            author: "chat".to_string(),
            committer: "chat".to_string(),
            committer_time: 1777759186,
        };
        let private_key = PrivateKey::generate();

        let event = generate_git_note_event(&note, &private_key).expect("git note event");

        assert_eq!(event.kind, EventKind::Patches);
        assert!(event.tags.iter().any(|tag| tag.tagname() == "commit"));
        assert!(event.tags.iter().any(|tag| tag.tagname() == "notes-ref"));
    }

    #[test]
    fn reexports_zap_data_and_serializes_through_chat() {
        let id = Id::try_from_hex_string(
            "5df64b33303d62afc799bdc36d178c07b2e1f0d824f31b7dc812219440affab6",
        )
        .expect("zap v1 id");
        let pubkey = PublicKey::try_from_hex_string(
            "ee11a5dff40c19a555f41fe42b48f00e618c91225622ae37b6c2bb67b76c4e49",
            true,
        )
        .expect("zap v1 pubkey");
        let provider_pubkey = PublicKey::try_from_hex_string(
            "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9",
            true,
        )
        .expect("zap v1 provider pubkey");
        let zap_v1 = ZapDataV1 {
            id,
            amount: MilliSatoshi(15423000),
            pubkey,
            provider_pubkey,
        };

        let zapped_event = EventReference::Id {
            id: Id::try_from_hex_string(
                "4d5a0a2f0eb8447d97a6b0f8bbd5f8c9a4cce7c835d3c7d6f2fd2a9f2f5f3a01",
            )
            .expect("zap v2 target id"),
            author: Some(PrivateKey::generate().public_key()),
            relays: Vec::new(),
            marker: Some("root".to_owned()),
        };
        let payee = PrivateKey::generate().public_key();
        let payer = PrivateKey::generate().public_key();
        let provider_pubkey = PrivateKey::generate().public_key();
        let zap_v2 = ZapDataV2 {
            zapped_event: zapped_event.clone(),
            amount: MilliSatoshi(15423000),
            payee,
            payer,
            provider_pubkey,
        };
        let chat_zap: ZapData = zap_v2.clone();

        let zap_v1_json = serde_json::to_string(&zap_v1).expect("serialize zap v1");
        let zap_v2_json = serde_json::to_string(&zap_v2).expect("serialize zap v2");
        let chat_zap_json = serde_json::to_string(&chat_zap).expect("serialize zap alias");

        println!("==================== chat zap data v1 ====================");
        println!("chat zap v1: {:?}", zap_v1);
        println!("chat zap v1 json: {zap_v1_json}");
        println!("==================== chat zap data v2 ====================");
        println!("chat zap v2: {:?}", zap_v2);
        println!("chat zap v2 json: {zap_v2_json}");
        println!("==================== chat zap data alias ====================");
        println!("chat zap alias: {:?}", chat_zap);
        println!("chat zap alias json: {chat_zap_json}");

        assert_eq!(
            serde_json::from_str::<ZapDataV1>(&zap_v1_json).expect("deserialize zap v1"),
            zap_v1
        );
        assert_eq!(
            serde_json::from_str::<ZapDataV2>(&zap_v2_json).expect("deserialize zap v2"),
            zap_v2
        );
        assert_eq!(
            serde_json::from_str::<ZapData>(&chat_zap_json).expect("deserialize zap alias"),
            chat_zap
        );
        assert_eq!(zap_v2.zapped_event, zapped_event);
    }
}
