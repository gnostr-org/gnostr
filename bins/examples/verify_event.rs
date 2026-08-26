use std::io::Read;

use gnostr::types::Event;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut s: String = String::new();
    std::io::stdin().read_to_string(&mut s)?;
    let event: Event = serde_json::from_str(&s)?;
    event.verify(None)?;
    println!("OK");
    Ok(())
}
