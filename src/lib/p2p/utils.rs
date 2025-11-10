use libp2p::identity;
use std::error::Error;
use tracing::{debug, trace};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub const GNOSTR_SHA256: [u8; 32] = [
    0xca, 0x45, 0xfe, 0x80, 0x0a, 0x2c, 0x3b, 0x67, 0x8e, 0x0a, 0x87, 0x7a, 0xa7, 0x7e, 0x36, 0x76,
    0x34, 0x0a, 0x59, 0xc9, 0xa7, 0x61, 0x5e, 0x30, 0x59, 0x76, 0xfb, 0x9b, 0xa8, 0xda, 0x48, 0x06,
];

pub fn init_subscriber() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    let fmt_layer = fmt::layer().with_target(false).with_ansi(true);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    Ok(())
}

pub fn generate_ed25519(secret_key_seed: Option<u8>) -> identity::Keypair {
    let mut bytes: [u8; 32] = GNOSTR_SHA256;

    if let Some(seed) = secret_key_seed {
        bytes[31] ^= seed;
    }

    for (i, byte) in bytes.iter().enumerate() {
        debug!("Byte {:02} [{:3} / {:#04x}]: ", i, byte, byte);
        for j in (0..8).rev() {
            let mask = 1 << j;
            if byte & mask == 0 {
                trace!("0");
            } else {
                trace!("1");
            }
        }
        trace!("\n");
    }

    let keypair =
        identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length");
    keypair
}
