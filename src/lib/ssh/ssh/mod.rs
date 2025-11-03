use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use russh::server::{Msg, Session};
use russh::*;
use russh_keys::*;
use tokio::io::AsyncWriteExt;
use tokio::process::ChildStdin;

use crate::ssh::config::server::ServerUser;
use crate::ssh::State;
use log::{debug, error};
use tokio::sync::Mutex;
use toml::Table;
use std::net::TcpListener;

mod keys;
use self::keys::server_keys;

mod commands;
mod messages;

fn is_port_available(port: u16) -> anyhow::Result<()> {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => {
            drop(listener);
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Port {} is not available: {}", port, e)),
    }
}

pub async fn start_server(state: Arc<Mutex<State>>) -> anyhow::Result<()> {
    debug!("start_server");
    debug!("russh::server::Config");
    let config = russh::server::Config {
        connection_timeout: Some(std::time::Duration::from_secs(3600)),
        auth_rejection_time: std::time::Duration::from_secs(3),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keys: vec![server_keys()?],
        ..Default::default()
    };

    let config = Arc::new(config);
    debug!("russh::server::Config:{:?}", config);

    let sh = Server {
        state: state.clone(),
    };

    let port = state.lock().await.server_config.port;
    is_port_available(port)?;
    debug!(
        "russh::server::run({:?}, (\"0.0.0.0\", {}), sh)",
        config, port
    );

    russh::server::run(config, ("0.0.0.0", port), sh).await?;

    Ok(())
}

struct Server {
    state: Arc<Mutex<State>>,
}

impl server::Server for Server {
    type Handler = Handler;
    fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Handler {
        Handler {
            stdin: HashMap::default(),
            state: self.state.clone(),
            user: None,
            username: None,
            _welcome_message: None,
            _extra: None,
        }
    }
}

struct Handler {
    stdin: HashMap<ChannelId, ChildStdin>,
    state: Arc<Mutex<State>>,
    user: Option<ServerUser>,
    username: Option<String>,
    _welcome_message: Option<Table>,
    _extra: Option<Table>,
}

impl Handler {
    async fn send_stdin(&mut self, channel_id: ChannelId, data: &[u8]) -> anyhow::Result<()> {
        if let Some(stdin) = self.stdin.get_mut(&channel_id) {
            stdin.write_all(data).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl server::Handler for Handler {
    type Error = anyhow::Error;

    async fn channel_open_session(
        mut self,
        _channel: Channel<Msg>,
        session: Session,
    ) -> anyhow::Result<(Self, bool, Session)> {
        Ok((self, true, session))
    }

    async fn auth_publickey(
        mut self,
        _: &str,
        key: &key::PublicKey,
    ) -> anyhow::Result<(Self, server::Auth)> {
        let key = key.public_key_base64();
        if let Some(data) = self.state.lock().await.server_config.get_user(&key) {
            self.username = Some(data.0);
            self.user = Some(data.1);
        }
        Ok((self, server::Auth::Accept))
    }

    async fn data(
        mut self,
        channel: ChannelId,
        data: &[u8],
        session: Session,
    ) -> anyhow::Result<(Self, Session)> {
        self.send_stdin(channel, data).await?;
        Ok((self, session))
    }

    async fn channel_eof(
        mut self,
        channel: ChannelId,
        session: Session,
    ) -> Result<(Self, Session), Self::Error> {
        let stdin = self.stdin.remove(&channel);
        if let Some(mut stdin) = stdin {
            stdin.shutdown().await?;
        }

        Ok((self, session))
    }

    async fn exec_request(
        mut self,
        channel: ChannelId,
        data: &[u8],
        session: Session,
    ) -> anyhow::Result<(Self, Session)> {
        let handle = session.handle();
        if let Err(e) = self.handle_command(handle.clone(), channel, data).await {
            error!("{:#}", e);
            handle.close(channel).await.unwrap();
        }
        Ok((self, session))
    }
}
