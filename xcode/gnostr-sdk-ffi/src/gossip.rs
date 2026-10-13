// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2025 Rust Nostr Developers
// Distributed under the MIT software license

use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::Arc;

use nostr_gossip_memory::prelude::*;
use uniffi::Object;

use crate::error::{NostrSdkError, Result};

/// Gossip in-memory storage.
#[derive(Clone, Object)]
pub struct NostrGossip {
    inner: Arc<NostrGossipMemory>,
}

impl Deref for NostrGossip {
    type Target = Arc<NostrGossipMemory>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<NostrGossipMemory> for NostrGossip {
    fn from(inner: NostrGossipMemory) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

#[uniffi::export]
impl NostrGossip {
    /// Construct a new **unbounded** instance.
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(NostrGossipMemory::unbounded()),
        }
    }

    /// Construct a new **bounded** instance.
    #[uniffi::constructor]
    pub fn bounded(limit: u64) -> Result<Self> {
        let limit: NonZeroUsize = NonZeroUsize::new(limit as usize)
            .ok_or_else(|| NostrSdkError::Generic("limit must be greater than 0".to_string()))?;
        Ok(Self {
            inner: Arc::new(NostrGossipMemory::bounded(limit)),
        })
    }
}

impl Default for NostrGossip {
    fn default() -> Self {
        Self::new()
    }
}
