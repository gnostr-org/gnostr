// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::net::AddrParseError;

pub type Result<T, E = NostrSdkError> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum NostrSdkError {
    Generic { err: String },
}

impl fmt::Display for NostrSdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic { err } => write!(f, "{}", err),
        }
    }
}

impl From<nostr_sdk::client::Error> for NostrSdkError {
    fn from(e: nostr_sdk::client::Error) -> NostrSdkError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<AddrParseError> for NostrSdkError {
    fn from(e: AddrParseError) -> NostrSdkError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<nostr::url::ParseError> for NostrSdkError {
    fn from(e: nostr::url::ParseError) -> NostrSdkError {
        Self::Generic { err: e.to_string() }
    }
}
