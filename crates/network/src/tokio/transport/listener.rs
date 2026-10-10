use crate::motd::BedrockMOTD;
use crate::tokio::error::{RakNetError, TransportLayerError};
use crate::tokio::transport::TransportLayerConnection;
use raknet_tokio::prelude::*;

#[cfg(feature = "nethernet-tokio")]
pub struct NetherNetListener {
    server: nethernet_tokio::NetherServer,
    signaling: nethernet_tokio::ServerSignaling,
}

#[cfg(feature = "nethernet-tokio")]
impl NetherNetListener {
    pub async fn bind(
        signaling: impl Into<nethernet_tokio::ServerSignaling>,
        config: nethernet_tokio::ConnectionConfig,
    ) -> Result<Self, TransportLayerError> {
        let signaling = signaling.into();
        let server = nethernet_tokio::NetherServer::bind_with(signaling.clone(), config).await?;
        Ok(Self { server, signaling })
    }

    pub(crate) fn advertise(&self, motd: &BedrockMOTD) {
        let data = nethernet_tokio::ServerData::from(motd);
        match &self.signaling {
            nethernet_tokio::ServerSignaling::Lan(lan) => lan.set_server_data(data),
            nethernet_tokio::ServerSignaling::Http(http) => http.set_server_data(data),
        }
    }
}

pub enum TransportLayerListener {
    RakNet(RakServer),
    #[cfg(feature = "nethernet-tokio")]
    NetherNet(NetherNetListener),
    // TODO: Quic(s2n_quic::server::Server),
    // TODO: Tcp(...),
}

impl TransportLayerListener {
    pub fn advertise(&mut self, motd: &BedrockMOTD) {
        match self {
            Self::RakNet(listener) => listener.set_message::<Box<[u8]>>(motd.into()),
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(listener) => listener.advertise(motd),
        }
    }

    pub fn set_max_connections(&mut self, n: usize) {
        match self {
            Self::RakNet(listener) => listener.set_max_connections(n),
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(_) => {}
        }
    }

    pub async fn start(&mut self) -> Result<(), TransportLayerError> {
        match self {
            Self::RakNet(listener) => listener.start().await.map_err(RakNetError::from)?,
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(_) => {}
        };

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), TransportLayerError> {
        match self {
            Self::RakNet(listener) => listener.stop().await,
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(listener) => listener.server.close().await?,
        }

        Ok(())
    }

    pub async fn accept(&mut self) -> Result<TransportLayerConnection, TransportLayerError> {
        let conn = match self {
            Self::RakNet(listener) => TransportLayerConnection::RakNet(
                listener.accept().await.map_err(RakNetError::from)?,
            ),
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(listener) => {
                TransportLayerConnection::nethernet(listener.server.accept().await?).await
            }
        };

        Ok(conn)
    }
}
