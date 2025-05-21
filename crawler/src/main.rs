use gnostr_crawler::processor::Processor;
use gnostr_crawler::processor::APP_SECRET_KEY;
use gnostr_crawler::processor::BOOTSTRAP_RELAY1;
use gnostr_crawler::processor::BOOTSTRAP_RELAY2;
use gnostr_crawler::processor::BOOTSTRAP_RELAY3;
use gnostr_crawler::relay_manager::RelayManager;
use gnostr_crawler::CliArgs;

use clap::Parser;
use nostr_sdk::prelude::{FromBech32, Keys, SecretKey};

#[tokio::main]
async fn main() {
    //env_logger::init();
    let args = CliArgs::parse();

    match gnostr_crawler::run(&args) {
        Ok(()) => {
            let app_secret_key = SecretKey::from_bech32(APP_SECRET_KEY);
            let app_keys = Keys::new(app_secret_key.expect("REASON"));
            let processor = Processor::new();
            let mut relay_manager = RelayManager::new(app_keys, processor);
            let _ = relay_manager
                .run(vec![BOOTSTRAP_RELAY1, BOOTSTRAP_RELAY2, BOOTSTRAP_RELAY3])
                .await;
            relay_manager.processor.dump();
        }
        Err(e) => println!("error: {}", e),
    }
}
