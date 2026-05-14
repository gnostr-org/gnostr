//! Shared Nostr and git-note message types for the middle layer.
//!
//! `gnostr-p2p` re-exports the canonical types from `gnostr_asyncgit`, and
//! `gnostr-chat` re-exports this module so the chain stays
//! `asyncgit -> p2p -> chat`.

pub use crate::git2::types::*;

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::Path,
    };

    use super::*;
    use gnostr_asyncgit::{
        sync::{
            add_note, append_public_attestation_log, commit, show_note, stage_add_file,
            CommitMineOptions, RepoPath,
        },
        types::{generate_git_note_event, nip3::create_attestation_with_pow, Id, PrivateKey, Unixtime},
    };
    use crate::{bitcoindev_1, bitcoindev_2, bitcoindev_3};
    use crate::time::{ClockStatus, Estimation, SyncState};
    use serde_json;
    use tempfile::{tempdir, NamedTempFile};
    use time::OffsetDateTime;

    fn real_trace_fixture() -> (GitNote, Event, SubscriptionId, Filter) {
        let temp_dir = tempdir().expect("temp repo");
        let repo = gnostr_asyncgit::git2::Repository::init(temp_dir.path()).expect("init repo");
        {
            let mut config = repo.config().expect("repo config");
            config.set_str("user.name", "gnostr-trace").expect("user name");
            config
                .set_str("user.email", "trace@gnostr.org")
                .expect("user email");
        }

        let repo_path: RepoPath = temp_dir
            .path()
            .to_str()
            .expect("repo path")
            .into();

        let trace_file = temp_dir.path().join("trace.txt");
        fs::write(&trace_file, "real p2p trace").expect("write trace file");
        stage_add_file(&repo_path, Path::new("trace.txt")).expect("stage trace file");

        let commit_id = commit(&repo_path, "real p2p commit").expect("commit");
        let notes_ref = default_notes_ref(&repo_path).expect("default notes ref");
        add_note(
            &repo_path,
            commit_id,
            "real p2p note",
            Some(&notes_ref),
            false,
        )
        .expect("add note");

        let note = show_note(&repo_path, commit_id, Some(&notes_ref))
            .expect("show note")
            .expect("note exists");
        let git_note = GitNote::from(&note);
        let private_key = PrivateKey::generate();
        let event = generate_git_note_event(&git_note, &private_key).expect("git note event");

        let mut tags = BTreeMap::new();
        for tag in &event.tags {
            if let Some(letter) = tag.tagname().chars().next() {
                tags.entry(letter)
                    .or_insert_with(Vec::new)
                    .push(tag.value().to_string());
            }
        }

        let mut filter = Filter::default();
        filter.ids = vec![event.id.into()];
        filter.authors = vec![event.pubkey.into()];
        filter.kinds = vec![event.kind];
        filter.tags = tags;
        filter.since = Some(event.created_at);
        filter.until = Some(event.created_at);
        filter.limit = Some(event.tags.len());

        let subscription_id = SubscriptionId(event.id.as_hex_string());

        (git_note, event, subscription_id, filter)
    }

    fn consensus_time(
        state: &mut SyncState,
        estimates: Vec<Estimation>,
    ) -> chrono::DateTime<chrono::Utc> {
        state.apply_bft_sync(estimates);
        let logical = state.get_logical_utc();
        println!(
            "quorum utc consensus: {} status={:?} slew_rate={:.6}",
            logical.to_rfc3339(),
            state.status,
            state.get_metrics().slew_rate
        );
        logical
    }

    #[test]
    fn real_asyncgit_note_is_reexported_and_serializes_through_p2p() {
        let (git_note, event, subscription_id, filter) = real_trace_fixture();

        let event_kind: EventKind = event.kind;
        let relay_message = RelayMessage::Event(subscription_id.clone(), Box::new(event.clone()));
        let client_message = ClientMessage::Req(subscription_id.clone(), vec![filter.clone()]);
        let typed_note: GitNote = git_note.clone();

        let relay_json = serde_json::to_string(&relay_message).expect("serialize relay message");
        let client_json = serde_json::to_string(&client_message).expect("serialize client message");
        let note_json = serde_json::to_string(&typed_note).expect("serialize git note");

        assert_eq!(event_kind, EventKind::Patches);
        assert_eq!(typed_note.message, event.content);
        assert_eq!(
            serde_json::from_str::<RelayMessage>(&relay_json).expect("deserialize relay message"),
            relay_message
        );
        assert_eq!(
            serde_json::from_str::<ClientMessage>(&client_json).expect("deserialize client message"),
            client_message
        );
        assert_eq!(
            serde_json::from_str::<GitNote>(&note_json).expect("deserialize git note"),
            typed_note
        );
        assert!(filter.ids.contains(&event.id.into()));
        assert!(filter.authors.contains(&event.pubkey.into()));
        assert!(filter.kinds.contains(&event.kind));
        assert_eq!(subscription_id.0, event.id.as_hex_string());
    }

    #[test]
    fn reexports_nip34_kinds_and_aliases_through_p2p() {
        let _nip34_kind: Nip34Kind = Nip34Kind::from(crate::git2::types::nip34::REPO_ANNOUNCEMENT_KIND);
        let _nip34_event: Option<Nip34Event> = None;
        let _nip34_unsigned: Option<Nip34UnsignedEvent> = None;

        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::REPO_ANNOUNCEMENT_KIND),
            EventKind::RepositoryAnnouncement
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::REPO_STATE_KIND),
            EventKind::GitRepoAnnouncement
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::PULL_REQUEST_KIND),
            EventKind::Other(1618)
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::PULL_REQUEST_UPDATE_KIND),
            EventKind::Other(1619)
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::GIT_ISSUE_KIND),
            EventKind::GitIssue
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::GIT_REPLY_KIND),
            EventKind::GitReply
        );
        assert_eq!(
            Nip34Kind::from(crate::git2::types::nip34::USER_GRASP_LIST_KIND),
            EventKind::Replaceable(10317)
        );

        let _ = (_nip34_kind, _nip34_event, _nip34_unsigned);
    }

    #[test]
    fn reexports_zap_data_and_serializes_through_p2p() {
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
        let p2p_zap: ZapData = zap_v2.clone();

        let zap_v1_json = serde_json::to_string(&zap_v1).expect("serialize zap v1");
        let zap_v2_json = serde_json::to_string(&zap_v2).expect("serialize zap v2");
        let p2p_zap_json = serde_json::to_string(&p2p_zap).expect("serialize zap alias");

        println!("==================== zap data v1 ====================");
        println!("zap v1: {:?}", zap_v1);
        println!("zap v1 json: {zap_v1_json}");
        println!("==================== zap data v2 ====================");
        println!("zap v2: {:?}", zap_v2);
        println!("zap v2 json: {zap_v2_json}");
        println!("==================== zap data alias ====================");
        println!("p2p zap: {:?}", p2p_zap);
        println!("p2p zap json: {p2p_zap_json}");

        assert_eq!(
            serde_json::from_str::<ZapDataV1>(&zap_v1_json).expect("deserialize zap v1"),
            zap_v1
        );
        assert_eq!(
            serde_json::from_str::<ZapDataV2>(&zap_v2_json).expect("deserialize zap v2"),
            zap_v2
        );
        assert_eq!(
            serde_json::from_str::<ZapData>(&p2p_zap_json).expect("deserialize zap alias"),
            p2p_zap
        );
        assert_eq!(zap_v2.zapped_event, zapped_event);
    }

    #[test]
    fn nip34_events_traverse_the_p2p_middle_layer() {
        let (git_note, event, subscription_id, filter) = real_trace_fixture();
        let asyncgit_event: gnostr_asyncgit::types::EventV3 = event.clone();
        let p2p_event: Nip34Event = asyncgit_event.clone();
        let unsigned_event: Nip34UnsignedEvent = Nip34UnsignedEvent {
            pubkey: event.pubkey,
            created_at: event.created_at,
            kind: event.kind,
            tags: event.tags.clone(),
            content: event.content.clone(),
        };

        let relay_message = RelayMessage::Event(subscription_id.clone(), Box::new(p2p_event.clone()));
        let client_message = ClientMessage::Req(subscription_id.clone(), vec![filter.clone()]);

        assert_eq!(unsigned_event.hash().expect("unsigned hash"), event.id);
        assert_eq!(asyncgit_event.kind, EventKind::Patches);
        assert_eq!(git_note.message, asyncgit_event.content);

        let event_json = serde_json::to_string(&p2p_event).expect("serialize p2p event");
        let unsigned_json = serde_json::to_string(&unsigned_event).expect("serialize unsigned event");
        let relay_json = serde_json::to_string(&relay_message).expect("serialize relay message");
        let client_json = serde_json::to_string(&client_message).expect("serialize client message");

        let decoded_event: Nip34Event = serde_json::from_str(&event_json).expect("deserialize p2p event");
        let decoded_unsigned: Nip34UnsignedEvent =
            serde_json::from_str(&unsigned_json).expect("deserialize unsigned event");
        let decoded_relay: RelayMessage = serde_json::from_str(&relay_json).expect("deserialize relay message");
        let decoded_client: ClientMessage = serde_json::from_str(&client_json).expect("deserialize client message");

        let asyncgit_round_trip_event: gnostr_asyncgit::types::EventV3 = decoded_event.clone();

        assert_eq!(decoded_event, p2p_event);
        assert_eq!(decoded_unsigned, unsigned_event);
        assert_eq!(asyncgit_round_trip_event, event);
        match decoded_relay {
            RelayMessage::Event(id, boxed_event) => {
                assert_eq!(id, subscription_id);
                assert_eq!(*boxed_event, p2p_event);
            }
            other => panic!("unexpected relay message: {:?}", other),
        }
        assert_eq!(decoded_client, client_message);
    }

    #[test]
    fn nip34_git_note_created_at_tracks_quorum_consensus_time() {
        let (git_note, _, _, _) = real_trace_fixture();
        let checkpoint = NamedTempFile::new().expect("temp checkpoint");
        let checkpoint_path = checkpoint.path().to_string_lossy().to_string();
        let mut state = SyncState::new(1, &checkpoint_path);
        let private_key = PrivateKey::generate();

        println!("==================== nip34 quorum created-at ====================");
        println!(
            "starting state: status={:?} slew_rate={:.6}",
            state.status,
            state.get_metrics().slew_rate
        );

        let consensus = consensus_time(
            &mut state,
            vec![
                Estimation { d: 0.005, a: 0.001 },
                Estimation { d: 0.005, a: 0.001 },
                Estimation { d: 0.007, a: 0.001 },
                Estimation { d: 0.007, a: 0.001 },
                Estimation { d: 0.250, a: 0.001 },
            ],
        );

        let mut quorum_note = git_note.clone();
        quorum_note.committer_time = consensus.timestamp();
        let event = generate_git_note_event(&quorum_note, &private_key).expect("git note event");

        println!(
            "quorum consensus event created_at={} consensus={}",
            event.created_at,
            consensus.timestamp()
        );
        println!("quorum consensus event tags:");
        for tag in &event.tags {
            println!("  - {}", tag.0.join(":"));
        }

        assert_eq!(event.kind, EventKind::Patches);
        assert_eq!(event.created_at, Unixtime(consensus.timestamp()));
        assert_eq!(event.content, quorum_note.message);
        assert!(matches!(state.status, ClockStatus::Synced | ClockStatus::Slewing));
        assert!(state.pending_alert.is_none());
    }

    #[test]
    fn nip34_git_note_created_at_tracks_quorum_rotation_time() {
        let (git_note, _, _, _) = real_trace_fixture();
        let checkpoint = NamedTempFile::new().expect("temp checkpoint");
        let checkpoint_path = checkpoint.path().to_string_lossy().to_string();
        let mut state = SyncState::new(1, &checkpoint_path);
        let private_key = PrivateKey::generate();
        let mut last_created_at = None;

        println!("==================== nip34 quorum rotation created-at ====================");

        let rounds = [
            (
                "quorum-forms",
                vec![
                    Estimation { d: 0.005, a: 0.001 },
                    Estimation { d: 0.005, a: 0.001 },
                    Estimation { d: 0.007, a: 0.001 },
                    Estimation { d: 0.007, a: 0.001 },
                    Estimation { d: 0.250, a: 0.001 },
                ],
            ),
            (
                "honest-rotation",
                vec![
                    Estimation { d: 0.005, a: 0.001 },
                    Estimation { d: 0.005, a: 0.001 },
                    Estimation { d: 0.007, a: 0.001 },
                    Estimation { d: 0.007, a: 0.001 },
                    Estimation { d: 0.250, a: 0.001 },
                ],
            ),
            (
                "replacement-complete",
                vec![
                    Estimation { d: 0.005, a: 0.001 },
                    Estimation { d: 0.005, a: 0.001 },
                    Estimation { d: 0.007, a: 0.001 },
                    Estimation { d: 0.007, a: 0.001 },
                    Estimation { d: 0.007, a: 0.001 },
                ],
            ),
        ];

        for (label, estimates) in rounds {
            println!("round {label}: {} samples", estimates.len());
            for (idx, estimate) in estimates.iter().enumerate() {
                println!(
                    "peer sample {idx}: d={:.6}s a={:.6}s",
                    estimate.d, estimate.a
                );
            }

            let consensus = consensus_time(&mut state, estimates);
            let mut quorum_note = git_note.clone();
            quorum_note.committer_time = consensus.timestamp();
            let event = generate_git_note_event(&quorum_note, &private_key).expect("git note event");

            println!(
                "round {label}: event created_at={} consensus={}",
                event.created_at,
                consensus.timestamp()
            );
            println!(
                "round {label}: status={:?} slew_rate={:.6} pending_alert={:?}",
                state.status,
                state.get_metrics().slew_rate,
                state.pending_alert
            );

            assert_eq!(event.kind, EventKind::Patches);
            assert_eq!(event.created_at, Unixtime(consensus.timestamp()));
            assert_eq!(event.content, quorum_note.message);
            assert!(matches!(state.status, ClockStatus::Synced | ClockStatus::Slewing));
            assert!(state.pending_alert.is_none());

            if let Some(previous) = last_created_at {
                assert!(event.created_at >= previous);
            }
            last_created_at = Some(event.created_at);
        }
    }

    #[test]
    fn pretty_print_attestations() -> anyhow::Result<()> {
        let (_td, repo) = tempdir().map(|td| {
            let repo = gnostr_asyncgit::git2::Repository::init(td.path()).expect("init repo");
            {
                let mut config = repo.config().expect("repo config");
                config.set_str("user.name", "gnostr-p2p").expect("user name");
                config.set_str("user.email", "p2p@gnostr.org").expect("user email");
            }
            (td, repo)
        })?;
        let root = repo.path().parent().unwrap();
        let repo_path_owned: RepoPath = root.as_os_str().to_str().unwrap().into();
        let repo_path: &RepoPath = &repo_path_owned;
        let fixtures = [bitcoindev_1, bitcoindev_2, bitcoindev_3];
        let mut previous_attestation_id: Option<String> = None;

        for (index, profile) in fixtures.iter().enumerate() {
            let file_name = format!("pretty-print-attestations-{index}.txt");
            std::fs::write(root.join(&file_name), profile.label.as_bytes())?;
            stage_add_file(repo_path, Path::new(&file_name))?;

            let commit_id = gnostr_asyncgit::sync::commit::mine_commit(
                repo_path,
                CommitMineOptions {
                    threads: 1,
                    target: "0".to_string(),
                    message: vec![format!("{} commit", profile.label)],
                    timestamp: OffsetDateTime::from_unix_timestamp(0).unwrap(),
                },
            )?;

            let attestation_target = Id::try_from_hex_string(&format!("{:0>64}", commit_id.to_string()))
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
            let secret_key = profile.private_key().0.clone();
            let (xonly_public_key, _parity) = secret_key.x_only_public_key(secp256k1::SECP256K1);
            let attestation = create_attestation_with_pow(
                attestation_target,
                profile.metadata_json(),
                &xonly_public_key,
                &secret_key,
                5,
            );
            let notes_ref = previous_attestation_id
                .as_deref()
                .map(|event_id| format!("refs/notes/public-attestations/{event_id}"))
                .unwrap_or_else(|| "refs/notes/public-attestations/root".to_string());

            let note_message = append_public_attestation_log(
                None,
                1234 + index as i64,
                &attestation.id.as_hex_string(),
                &commit_id.to_string(),
                attestation.nonce_data().map(|(_, bits)| bits).unwrap_or(0),
            );
            let note_id = add_note(repo_path, commit_id, &note_message, Some(&notes_ref), true)?;
            let note = show_note(repo_path, commit_id, Some(&notes_ref))?.expect("note exists");
            let relay_message = RelayMessage::Event(
                SubscriptionId(attestation.id.as_hex_string()),
                Box::new(attestation.clone()),
            );
            let attestation_json = serde_json::to_value(&attestation)?;
            let relay_message_json = serde_json::to_value(&relay_message)?;
            let profile_metadata = profile.metadata();
            let attestation_content = serde_json::from_str::<serde_json::Value>(&attestation.content)?;

            println!(
                "pretty_print_attestations\n{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "profile": profile.label,
                    "commit": commit_id.to_string(),
                    "note_id": note_id.to_string(),
                    "note": &note,
                    "profile_metadata": profile_metadata,
                    "profile_npub": profile.npub(),
                    "profile_nsec": profile.nsec(),
                    "attestation": attestation_json,
                    "attestation_signature": format!("{:?}", attestation.sig),
                    "attestation_content": attestation_content,
                    "relay_message": relay_message_json,
                    "notes_ref": notes_ref,
                }))?
            );

            assert_eq!(note.note_id, note_id);
            assert!(note.message.contains(&attestation.id.as_hex_string()));
            assert!(note.message.contains(&commit_id.to_string()));
            previous_attestation_id = Some(attestation.id.as_hex_string());
        }

        Ok(())
    }
}
