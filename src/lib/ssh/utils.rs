use anyhow::anyhow;
use russh::CryptoVec;
use std::net::{TcpListener, SocketAddr};
use std::io::ErrorKind;

pub trait CustomContext<T> {
    fn context(self, context: &str) -> anyhow::Result<T>;
}

impl<T> CustomContext<T> for Result<T, ()> {
    fn context(self, context: &str) -> anyhow::Result<T> {
        self.map_err(|_| anyhow!(context.to_string()))
    }
}

impl<T> CustomContext<T> for Result<T, CryptoVec> {
    fn context(self, context: &str) -> anyhow::Result<T> {
        self.map_err(|e| anyhow!(context.to_string()).context(format!("{:?}", e)))
    }
}

pub async fn is_port_in_use(port: u16) -> bool {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    match TcpListener::bind(addr) {
        Ok(_) => false, // Successfully bound, so port is not in use
        Err(e) => {
            if e.kind() == ErrorKind::AddrInUse {
                true // Address already in use, so port is in use
            } else {
                // Another error occurred, might be permissions or something else
                eprintln!("Error checking port {}: {}", port, e);
                true // Treat other errors as the port being unavailable for use
            }
        }
    }
}
