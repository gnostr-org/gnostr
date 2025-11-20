// Copyright (c) 2024 Yuki Kishimoto
// Distributed under the MIT software license

use lnurl_pay::{api, LightningAddress};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = LightningAddress::parse("yuki@getalby.com")?;
    let invoice =
        api::get_invoice(addr, 1000, Some(String::from("Your comment")), None, None).await?;
    println!("{invoice}");
    Ok(())
}
