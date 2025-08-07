use anyhow::anyhow;
use russh::CryptoVec;

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
