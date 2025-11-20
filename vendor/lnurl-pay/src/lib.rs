// Copyright (c) 2024 Yuki Kishimoto
// Distributed under the MIT software license

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "api")]
pub mod api;
pub mod error;
pub mod lud06;
pub mod lud16;

pub use self::error::Error;
pub use self::lud06::LnUrl;
pub use self::lud16::LightningAddress;
