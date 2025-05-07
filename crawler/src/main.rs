use nostr_relays::processor::Processor;
use nostr_relays::processor::APP_SECRET_KEY;
use nostr_relays::processor::BOOTSTRAP_RELAY1;
use nostr_relays::processor::BOOTSTRAP_RELAY2;
use nostr_relays::processor::BOOTSTRAP_RELAY3;
use nostr_relays::relay_manager::RelayManager;
use nostr_sdk::prelude::{FromBech32, Keys, Result, SecretKey};

use env_logger::Env;
use log::log_enabled;
use log::Level;
use log::{debug, error, info, trace, warn};

#[tokio::main]
async fn main() -> Result<()> {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "none")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    trace!("some trace log");
    debug!("some debug log");
    info!("some information log");
    warn!("some warning log");
    error!("some error log");

    debug!("this is a debug {}", "message");
    error!("this is printed by default");

    if log_enabled!(Level::Info) {
        let x = 3 * 4; // expensive computation
        info!("the answer was: {}", x);
    }
    let app_secret_key = SecretKey::from_bech32(APP_SECRET_KEY)?;
    let app_keys = Keys::new(app_secret_key);
    let processor = Processor::new();
    let mut relay_manager = RelayManager::new(app_keys, processor);
    relay_manager
        .run(vec![BOOTSTRAP_RELAY1, BOOTSTRAP_RELAY2, BOOTSTRAP_RELAY3])
        .await?;
    //relay_manager.processor.dump();

    Ok(())
}
