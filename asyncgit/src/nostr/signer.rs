/// Read-only nostr signer.
///
/// Allows building a `Client` from just a public key (npub), without the
/// ability to sign events.  Ported from nostr/src/infrastructure/nostr.rs.
use std::borrow::Cow;

use nostr_sdk::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKeySigner {
	pubkey: PublicKey,
}

impl PublicKeySigner {
	pub fn new(pubkey: PublicKey) -> Self {
		Self { pubkey }
	}
}

impl NostrSigner for PublicKeySigner {
	fn backend(&self) -> SignerBackend<'_> {
		SignerBackend::Custom(Cow::Borrowed("PublicKeySigner"))
	}

	fn get_public_key(
		&self,
	) -> BoxedFuture<'_, std::result::Result<PublicKey, SignerError>> {
		Box::pin(async { Ok(self.pubkey) })
	}

	fn sign_event(
		&self,
		_unsigned: UnsignedEvent,
	) -> BoxedFuture<'_, std::result::Result<Event, SignerError>> {
		Box::pin(async {
			Err(SignerError::backend("read-only signer cannot sign events"))
		})
	}

	fn nip04_encrypt<'a>(
		&'a self,
		_public_key: &'a PublicKey,
		_content: &'a str,
	) -> BoxedFuture<'a, std::result::Result<String, SignerError>> {
		Box::pin(async {
			Err(SignerError::backend("read-only"))
		})
	}

	fn nip04_decrypt<'a>(
		&'a self,
		_public_key: &'a PublicKey,
		_encrypted_content: &'a str,
	) -> BoxedFuture<'a, std::result::Result<String, SignerError>> {
		Box::pin(async {
			Err(SignerError::backend("read-only"))
		})
	}

	fn nip44_encrypt<'a>(
		&'a self,
		_public_key: &'a PublicKey,
		_content: &'a str,
	) -> BoxedFuture<'a, std::result::Result<String, SignerError>> {
		Box::pin(async {
			Err(SignerError::backend("read-only"))
		})
	}

	fn nip44_decrypt<'a>(
		&'a self,
		_public_key: &'a PublicKey,
		_payload: &'a str,
	) -> BoxedFuture<'a, std::result::Result<String, SignerError>> {
		Box::pin(async {
			Err(SignerError::backend("read-only"))
		})
	}
}
