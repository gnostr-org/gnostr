// Copyright (c) 2024 Yuki Kishimoto
// Distributed under the MIT software license

use alloc::format;
use alloc::string::{String, ToString};
use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::Error;
use crate::lud06::LnUrl;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LightningAddress {
    name: String,
    domain: String,
}

impl fmt::Display for LightningAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.domain)
    }
}

impl LightningAddress {
    pub fn parse<S>(ln_addr: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        let ln_addr: &str = ln_addr.as_ref();
        let mut splitted = ln_addr.split('@');
        let name = splitted.next().ok_or(Error::InvalidLightningAddress)?;
        let domain = splitted.next().ok_or(Error::InvalidLightningAddress)?;
        Ok(Self {
            name: name.to_string(),
            domain: domain.to_string(),
        })
    }

    #[inline]
    pub fn endpoint(&self) -> String {
        format!("https://{}/.well-known/lnurlp/{}", self.domain, self.name)
    }

    #[inline]
    pub fn lnurl(&self) -> LnUrl {
        LnUrl::new(self.endpoint())
    }
}

impl FromStr for LightningAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LightningAddress::parse(s)
    }
}

impl Serialize for LightningAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for LightningAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let addr = String::deserialize(deserializer)?;
        LightningAddress::parse(addr).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn test_parsing() {
        let address = LightningAddress::parse("ben@opreturnbot.com").unwrap();
        assert_eq!(
            address.endpoint(),
            "https://opreturnbot.com/.well-known/lnurlp/ben"
        );
        assert_eq!(address.to_string(), String::from("ben@opreturnbot.com"))
    }

    #[test]
    fn test_invalid_parsing() {
        assert!(LightningAddress::from_str("invalid").is_err());
        assert!(LightningAddress::from_str("####").is_err());
        assert!(LightningAddress::from_str("LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS").is_err());
    }

    #[test]
    fn test_lnurl() {
        let address = LightningAddress::from_str("ben@opreturnbot.com").unwrap();
        let lnurl = LnUrl::from_str("lnurl1dp68gurn8ghj7mmswfjhgatjde3x7apwvdhk6tewwajkcmpdddhx7amw9akxuatjd3cz7cn9dc94s6d4").unwrap();
        assert_eq!(address.lnurl(), lnurl);
    }
}
