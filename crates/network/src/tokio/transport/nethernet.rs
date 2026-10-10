use crate::motd::BedrockMOTD;
use crate::tokio::error::TransportLayerError;
use bytes::Bytes;
use nethernet_tokio::{
    AcceptedSession, Addr, ConnectionConfig, HttpSignaling, NetherClient, NetherError,
    ServerIdentity, Session, SessionReceiver,
};
use raknet_tokio::prelude::RakSessionConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;

pub struct NetherNetConnection {
    session: Session,
    reliable: SessionReceiver,
    _unreliable: SessionReceiver,
    remote: Addr,
}

impl NetherNetConnection {
    pub async fn accepted(accepted: AcceptedSession) -> Self {
        let remote = accepted.session.remote_addr().await;
        Self {
            session: accepted.session,
            reliable: accepted.reliable,
            _unreliable: accepted.unreliable,
            remote,
        }
    }

    pub async fn connect_http(server: SocketAddr) -> Result<Self, TransportLayerError> {
        let signaling = HttpSignaling::new(rand::random::<u64>().to_string())?;
        let identity =
            ServerIdentity::generate("", SystemTime::now()).map_err(NetherError::from)?;
        let config = ConnectionConfig {
            identity: Some(Arc::new(identity)),
            ..Default::default()
        };
        let client =
            NetherClient::connect_with(signaling, format!("http://{server}"), config).await?;
        let mut connection = Self::accepted(client.into_accepted()).await;
        connection.remote.socket_addr.get_or_insert(server);
        Ok(connection)
    }

    pub async fn query_http(server: SocketAddr) -> Result<BedrockMOTD, TransportLayerError> {
        let signaling = HttpSignaling::new(rand::random::<u64>().to_string())?;
        let data = signaling.server_data(&format!("http://{server}")).await?;
        Ok(BedrockMOTD::from(&data))
    }

    pub async fn send(&mut self, batch: &[u8]) -> Result<(), TransportLayerError> {
        self.session.send(Bytes::copy_from_slice(batch)).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>, TransportLayerError> {
        match self.reliable.recv().await? {
            Some(batch) => Ok(batch.to_vec()),
            None => Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into()),
        }
    }

    pub fn remote_addr(&self) -> SocketAddr {
        self.remote
            .socket_addr
            .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 0)))
    }

    pub fn max_message_len(&self) -> usize {
        RakSessionConfig::default().max_queued_bytes as usize
    }

    pub async fn close(&self) {
        let _ = self.session.close().await;
    }

    pub async fn is_closed(&self) -> bool {
        self.session.is_closed().await
    }
}
