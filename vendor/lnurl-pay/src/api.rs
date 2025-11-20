// Copyright (c) 2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::net::SocketAddr;

use reqwest::Client;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Proxy;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::lud06::LnUrl;
use crate::lud16::LightningAddress;

#[derive(Serialize, Deserialize)]
enum TagRequest {
    #[serde(rename = "payRequest")]
    Pay,
    #[serde(rename = "withdrawRequest")]
    Withdraw,
    #[serde(rename = "channelRequest")]
    Channel,
}

#[derive(Serialize, Deserialize)]
struct PayResponse {
    /// A second-level url which give you an invoice with a GET request
    /// and an amount
    pub callback: String,
    /// Max sendable amount for a given user on a given service
    #[serde(rename = "maxSendable")]
    pub max_sendable: u64,
    /// Min sendable amount for a given user on a given service,
    /// can not be less than 1 or more than `max_sendable`
    #[serde(rename = "minSendable")]
    pub min_sendable: u64,
    /// Tag of the request
    pub tag: TagRequest,
    /// Metadata json which must be presented as raw string here,
    /// this is required to pass signature verification at a later step
    pub metadata: String,
    /// Optional, if true, the service allows nostr zaps
    #[serde(rename = "allowsNostr")]
    pub allows_nostr: Option<bool>,
    // /// Optional, if true, the nostr pubkey that will be used to sign zap events
    // #[serde(rename = "nostrPubkey")]
    // pub nostr_pubkey: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct LnURLPayInvoice {
    /// Encoded bolt 11 invoice
    pr: Option<String>,
    status: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lud06OrLud16 {
    Lud06(LnUrl),
    Lud16(LightningAddress),
}

impl Lud06OrLud16 {
    pub fn endpoint(&self) -> String {
        match self {
            Self::Lud06(v) => v.endpoint(),
            Self::Lud16(v) => v.endpoint(),
        }
    }
}

impl From<LnUrl> for Lud06OrLud16 {
    fn from(value: LnUrl) -> Self {
        Self::Lud06(value)
    }
}

impl From<LightningAddress> for Lud06OrLud16 {
    fn from(value: LightningAddress) -> Self {
        Self::Lud16(value)
    }
}

/// Get invoice
///
/// **Proxy is ignored for WASM targets!**
pub async fn get_invoice<S>(
    lud: S,
    msats: u64,
    comment: Option<String>,
    zap_request: Option<String>,
    _proxy: Option<SocketAddr>,
) -> Result<String, Error>
where
    S: Into<Lud06OrLud16>,
{
    if msats == 0 {
        return Err(Error::AmountTooLow { msats, min: 1 });
    }

    // Compose client
    #[cfg(not(target_arch = "wasm32"))]
    let client: Client = {
        let mut builder = Client::builder();
        if let Some(proxy) = _proxy {
            let proxy = format!("socks5h://{proxy}");
            builder = builder.proxy(Proxy::all(proxy)?);
        }
        builder.build()?
    };

    #[cfg(target_arch = "wasm32")]
    let client: Client = Client::new();

    // Get Pay Response
    let lud: Lud06OrLud16 = lud.into();
    let endpoint: String = lud.endpoint();
    let resp = client.get(endpoint).send().await?;
    let pay_response: PayResponse = resp.error_for_status()?.json().await?;

    if msats < pay_response.min_sendable {
        return Err(Error::AmountTooLow {
            msats,
            min: pay_response.min_sendable,
        });
    }

    if msats > pay_response.max_sendable {
        return Err(Error::AmountTooHigh {
            msats,
            max: pay_response.max_sendable,
        });
    }

    // Get invoice
    let symbol: &str = if pay_response.callback.contains('?') {
        "&"
    } else {
        "?"
    };
    let url = match zap_request {
        Some(zap_request) => format!(
            "{}{}amount={}&nostr={}",
            pay_response.callback, symbol, msats, zap_request
        ),
        None => format!("{}{}amount={}", pay_response.callback, symbol, msats),
    };
    let url = match comment {
        Some(comment) => {
            format!("{url}&comment={comment}")
        }
        None => url,
    };
    let resp = client.get(&url).send().await?;
    let invoice: LnURLPayInvoice = resp.error_for_status()?.json().await?;

    match invoice.pr {
        Some(pr) => Ok(pr),
        None => Err(Error::CantGetInvoice(invoice.reason)),
    }
}
