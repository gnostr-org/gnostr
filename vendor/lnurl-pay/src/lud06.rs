// Copyright (c) 2024 Yuki Kishimoto
// Distributed under the MIT software license

use alloc::string::String;
use core::str::FromStr;

use bech32::{Bech32, Hrp};
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::Error;

const PREFIX: &str = "lnurl";
const HRP_PREFIX: Hrp = Hrp::parse_unchecked(PREFIX);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LnUrl {
    url: String,
}

impl LnUrl {
    pub fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        Self { url: url.into() }
    }

    #[inline]
    pub fn decode<S>(lnurl: S) -> Result<Self, Error>
    where
        S: AsRef<str>,
    {
        Self::from_str(lnurl.as_ref())
    }

    #[inline]
    pub fn encode(&self) -> Result<String, Error> {
        let bytes = self.url.as_bytes();
        Ok(bech32::encode::<Bech32>(HRP_PREFIX, bytes)?)
    }

    #[inline]
    pub fn endpoint(&self) -> String {
        self.url.clone()
    }
}

impl FromStr for LnUrl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        if s.to_lowercase().starts_with(PREFIX) {
            let (hrp, bytes) = bech32::decode(s).map_err(|_| Error::InvalidLnUrl)?;

            if hrp != HRP_PREFIX {
                return Err(Error::InvalidLnUrl);
            }

            let url = String::from_utf8(bytes).map_err(|_| Error::InvalidLnUrl)?;
            Ok(Self { url })
        } else {
            Err(Error::InvalidLnUrl)
        }
    }
}

impl Serialize for LnUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.encode().map_err(serde::ser::Error::custom)?)
    }
}

impl<'de> Deserialize<'de> for LnUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let lnurl = String::deserialize(deserializer)?;
        LnUrl::from_str(&lnurl).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn encode_test() {
        let url = "https://service.com/api?q=3fc3645b439ce8e7f2553a69e5267081d96dcd340693afabe04be7b0ccd178df";
        let expected =
            "LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS";

        let lnurl = LnUrl::new(url.to_string());
        assert_eq!(lnurl.encode().unwrap().to_uppercase(), expected);
    }

    #[test]
    fn decode_tests() {
        let str =
            "LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS";
        let expected = "https://service.com/api?q=3fc3645b439ce8e7f2553a69e5267081d96dcd340693afabe04be7b0ccd178df";

        let lnurl = LnUrl::decode(str.to_string()).unwrap();
        assert_eq!(lnurl.url, expected);
    }
}
