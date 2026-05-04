//! Shared nostr message types for p2p compatibility.
//!
//! These are re-exported from `gnostr_asyncgit` so `gnostr-p2p` can speak the
//! same nostr event, relay, client, and metadata shapes without a second model.

pub use crate::git2::types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_core_nostr_types() {
        let _event_kind = EventKind::TextNote;
        let _subscription_id = SubscriptionId("sub-1".to_string());
        let _filter = Filter::default();
        let _client_message = ClientMessage::Req(_subscription_id.clone(), vec![_filter.clone()]);
        let _relay_message = RelayMessage::Notice("ok".to_string());
        let _public_key = PublicKey::try_from_hex_string(
            "ee11a5dff40c19a555f41fe42b48f00e618c91225622ae37b6c2bb67b76c4e49",
            true,
        )
        .expect("valid public key");
        let _private_key = PrivateKey::generate();
        let _tag = Tag::new(&["e", "abc"]);
        let _id = Id::try_from_hex_string(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )
        .expect("valid id");
        let _signature = Signature::zeroes();

        let _ = (
            _event_kind,
            _subscription_id,
            _filter,
            _client_message,
            _relay_message,
            _public_key,
            _private_key,
            _tag,
            _id,
            _signature,
        );
    }
}
