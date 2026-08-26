#![warn(clippy::pedantic)]
#![deny(unsafe_code)]

//! Tor based transport for libp2p. Connect through the Tor network to TCP listeners.
//!
//! This is vendored from the local `libp2p-tor` reference crate.

use arti_client::{TorClient, TorClientBuilder};
use futures::future::BoxFuture;
use libp2p::{
    core::{
        connection::Endpoint,
        transport::{DialOpts, ListenerId, PortUse, TransportEvent},
    },
    Multiaddr, Transport, TransportError,
};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use thiserror::Error;
use tor_rtcompat::tokio::TokioRustlsRuntime;

#[cfg(feature = "listen-onion-service")]
use std::collections::HashMap;
#[cfg(feature = "listen-onion-service")]
use std::str::FromStr;
#[cfg(feature = "listen-onion-service")]
use tor_cell::relaycell::msg::{Connected, End, EndReason};
#[cfg(feature = "listen-onion-service")]
use tor_hsservice::{
    handle_rend_requests, status::OnionServiceStatus, HsId, OnionServiceConfig,
    RunningOnionService, StreamRequest,
};
#[cfg(feature = "listen-onion-service")]
use tor_proto::stream::IncomingStreamRequest;

mod address;
mod provider;

use address::{dangerous_extract, safe_extract};
pub use provider::TokioTorStream;

pub type TorError = arti_client::Error;

type PendingUpgrade = BoxFuture<'static, Result<TokioTorStream, TorTransportError>>;
#[cfg(feature = "listen-onion-service")]
type OnionServiceStream = futures::stream::BoxStream<'static, StreamRequest>;
#[cfg(feature = "listen-onion-service")]
type OnionServiceStatusStream = futures::stream::BoxStream<'static, OnionServiceStatus>;

/// Struct representing an onion address we are listening on for libp2p connections.
#[cfg(feature = "listen-onion-service")]
struct TorListener {
    #[allow(dead_code)]
    service: Arc<RunningOnionService>,
    status_stream: OnionServiceStatusStream,
    request_stream: OnionServiceStream,
    port: u16,
    onion_address: Multiaddr,
    announced: bool,
}

/// Mode of address conversion.
#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddressConversion {
    #[default]
    DnsOnly,
    IpAndDns,
}

pub struct TorTransport {
    pub conversion_mode: AddressConversion,
    client: Arc<TorClient<TokioRustlsRuntime>>,
    #[cfg(feature = "listen-onion-service")]
    listeners: HashMap<ListenerId, TorListener>,
    #[cfg(feature = "listen-onion-service")]
    services: Vec<(Arc<RunningOnionService>, OnionServiceStream)>,
}

impl TorTransport {
    /// Creates a new `TorClientBuilder`.
    pub fn builder() -> TorClientBuilder<TokioRustlsRuntime> {
        let runtime =
            TokioRustlsRuntime::current().expect("Couldn't get the current tokio rustls runtime");
        TorClient::with_runtime(runtime)
    }

    /// Creates a bootstrapped `TorTransport`.
    pub async fn bootstrapped() -> Result<Self, TorError> {
        let builder = Self::builder();
        let ret = Self::from_builder(&builder, AddressConversion::DnsOnly)?;
        ret.bootstrap().await?;
        Ok(ret)
    }

    /// Builds a `TorTransport` from an Arti `TorClientBuilder` but does not bootstrap it.
    pub fn from_builder(
        builder: &TorClientBuilder<TokioRustlsRuntime>,
        conversion_mode: AddressConversion,
    ) -> Result<Self, TorError> {
        let client = Arc::new(builder.create_unbootstrapped()?);

        Ok(Self::from_client(client, conversion_mode))
    }

    /// Builds a `TorTransport` from an existing Arti `TorClient`.
    pub fn from_client(
        client: Arc<TorClient<TokioRustlsRuntime>>,
        conversion_mode: AddressConversion,
    ) -> Self {
        Self {
            conversion_mode,
            client,
            #[cfg(feature = "listen-onion-service")]
            listeners: HashMap::new(),
            #[cfg(feature = "listen-onion-service")]
            services: Vec::new(),
        }
    }

    /// Bootstraps the `TorTransport` into the Tor network.
    pub async fn bootstrap(&self) -> Result<(), TorError> {
        self.client.bootstrap().await
    }

    #[must_use]
    pub fn with_address_conversion(mut self, conversion_mode: AddressConversion) -> Self {
        self.conversion_mode = conversion_mode;
        self
    }

    #[cfg(feature = "listen-onion-service")]
    pub fn add_onion_service(
        &mut self,
        svc_cfg: OnionServiceConfig,
        port: u16,
    ) -> anyhow::Result<Multiaddr> {
        let (service, request_stream) = self.client.launch_onion_service(svc_cfg)?;
        let request_stream = Box::pin(handle_rend_requests(request_stream));

        let multiaddr = service
            .onion_name()
            .ok_or_else(|| anyhow::anyhow!("Onion service has no onion address"))?
            .to_multiaddr(port);

        self.services.push((service, request_stream));

        Ok(multiaddr)
    }
}

#[derive(Debug, Error)]
pub enum TorTransportError {
    #[error(transparent)]
    Client(#[from] TorError),
    #[cfg(feature = "listen-onion-service")]
    #[error(transparent)]
    Service(#[from] tor_hsservice::ClientError),
    #[cfg(feature = "listen-onion-service")]
    #[error("Stream closed before receiving data")]
    StreamClosed,
    #[cfg(feature = "listen-onion-service")]
    #[error("Stream port does not match listener port")]
    StreamPortMismatch,
    #[cfg(feature = "listen-onion-service")]
    #[error("Onion service is broken")]
    Broken,
}

#[cfg(feature = "listen-onion-service")]
trait HsIdExt {
    fn to_multiaddr(&self, port: u16) -> Multiaddr;
}

#[cfg(feature = "listen-onion-service")]
impl HsIdExt for HsId {
    fn to_multiaddr(&self, port: u16) -> Multiaddr {
        let onion_domain = self.to_string();
        let onion_without_dot_onion = onion_domain
            .split('.')
            .next()
            .expect("Display formatting of HsId to contain .onion suffix");
        let multiaddress_string = format!("/onion3/{onion_without_dot_onion}:{port}");

        Multiaddr::from_str(&multiaddress_string)
            .expect("A valid onion address to be convertible to a Multiaddr")
    }
}

impl Transport for TorTransport {
    type Output = TokioTorStream;
    type Error = TorTransportError;
    type Dial = BoxFuture<'static, Result<Self::Output, Self::Error>>;
    type ListenerUpgrade = PendingUpgrade;

    #[cfg(not(feature = "listen-onion-service"))]
    fn listen_on(
        &mut self,
        _id: ListenerId,
        onion_address: Multiaddr,
    ) -> Result<(), TransportError<Self::Error>> {
        Err(TransportError::MultiaddrNotSupported(onion_address.clone()))
    }

    #[cfg(feature = "listen-onion-service")]
    fn listen_on(
        &mut self,
        id: ListenerId,
        onion_address: Multiaddr,
    ) -> Result<(), TransportError<Self::Error>> {
        let Some(libp2p::multiaddr::Protocol::Onion3(address)) = onion_address.into_iter().nth(0)
        else {
            return Err(TransportError::MultiaddrNotSupported(onion_address.clone()));
        };

        let position = self
            .services
            .iter()
            .position(|(service, _)| {
                service.onion_name().map_or(false, |name| {
                    name.to_multiaddr(address.port()) == onion_address
                })
            })
            .ok_or_else(|| TransportError::MultiaddrNotSupported(onion_address.clone()))?;

        let (service, request_stream) = self.services.remove(position);
        let status_stream = Box::pin(service.status_events());

        self.listeners.insert(
            id,
            TorListener {
                service,
                request_stream,
                onion_address: onion_address.clone(),
                port: address.port(),
                status_stream,
                announced: false,
            },
        );

        Ok(())
    }

    #[cfg(not(feature = "listen-onion-service"))]
    fn remove_listener(&mut self, _id: ListenerId) -> bool {
        false
    }

    #[cfg(feature = "listen-onion-service")]
    fn remove_listener(&mut self, id: ListenerId) -> bool {
        if let Some(listener) = self.listeners.remove(&id) {
            self.services
                .push((listener.service, listener.request_stream));
            return true;
        }

        false
    }

    fn dial(
        &mut self,
        addr: Multiaddr,
        opts: DialOpts,
    ) -> Result<Self::Dial, TransportError<Self::Error>> {
        let maybe_tor_addr = match self.conversion_mode {
            AddressConversion::DnsOnly => safe_extract(&addr),
            AddressConversion::IpAndDns => dangerous_extract(&addr),
        };

        if !matches!(opts.role, Endpoint::Dialer) || !matches!(opts.port_use, PortUse::Reuse) {
            tracing::debug!(?opts, "Ignoring unsupported dial options for Tor transport");
        }

        let tor_address =
            maybe_tor_addr.ok_or(TransportError::MultiaddrNotSupported(addr.clone()))?;
        let onion_client = self.client.clone();

        Ok(Box::pin(async move {
            let stream = onion_client.connect(tor_address).await?;

            tracing::debug!(%addr, "Established connection to peer through Tor");

            Ok(TokioTorStream::from(stream))
        }))
    }

    #[cfg(not(feature = "listen-onion-service"))]
    fn poll(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<TransportEvent<Self::ListenerUpgrade, Self::Error>> {
        Poll::Pending
    }

    #[cfg(feature = "listen-onion-service")]
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<TransportEvent<Self::ListenerUpgrade, Self::Error>> {
        for (listener_id, listener) in &mut self.listeners {
            if let Poll::Ready(Some(status)) = listener.status_stream.as_mut().poll_next(cx) {
                tracing::debug!(
                    status = ?status.state(),
                    address = listener.onion_address.to_string(),
                    "Onion service status changed"
                );
            }

            if !listener.announced {
                listener.announced = true;
                return Poll::Ready(TransportEvent::NewAddress {
                    listener_id: *listener_id,
                    listen_addr: listener.onion_address.clone(),
                });
            }

            match listener.request_stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(request)) => {
                    let port = listener.port;
                    let upgrade: PendingUpgrade = Box::pin(async move {
                        if let IncomingStreamRequest::Begin(begin) = request.request() {
                            if begin.port() != port {
                                request
                                    .reject(End::new_with_reason(EndReason::CONNECTREFUSED))
                                    .await?;

                                return Err(TorTransportError::StreamPortMismatch);
                            }
                        }

                        let data_stream = request.accept(Connected::new_empty()).await?;
                        Ok(TokioTorStream::from(data_stream))
                    });

                    return Poll::Ready(TransportEvent::Incoming {
                        listener_id: *listener_id,
                        upgrade,
                        local_addr: listener.onion_address.clone(),
                        send_back_addr: listener.onion_address.clone(),
                    });
                }
                Poll::Ready(None) => {
                    return Poll::Ready(TransportEvent::ListenerClosed {
                        listener_id: *listener_id,
                        reason: Ok(()),
                    });
                }
                Poll::Pending => {}
            }
        }

        Poll::Pending
    }
}

/// Build a bootstrapped Tor transport wrapped for libp2p swarm use.
pub async fn build_transport(
    keypair: &libp2p::identity::Keypair,
) -> Result<
    libp2p::core::transport::Boxed<(
        libp2p::PeerId,
        libp2p::core::muxing::StreamMuxerBox,
    )>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    let transport = TorTransport::bootstrapped()
        .await?
        .with_address_conversion(AddressConversion::IpAndDns);
    let auth_upgrade = libp2p::noise::Config::new(keypair)?;
    let multiplex_upgrade = libp2p::yamux::Config::default();

    Ok(transport
        .boxed()
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(auth_upgrade)
        .multiplex(multiplex_upgrade)
        .map(|(peer, muxer), _| (peer, libp2p::core::muxing::StreamMuxerBox::new(muxer)))
        .boxed())
}
