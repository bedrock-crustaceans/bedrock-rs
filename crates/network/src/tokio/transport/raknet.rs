use crate::error::NetworkCodecError;
use crate::raknet::RakNetGamePacket;
use crate::tokio::error::{RakNetError, TransportLayerError};
use raknet_tokio::prelude::*;
use std::net::SocketAddr;

pub struct RakNetConnection {
    session: RakSession,
    dialled_from: Option<::tokio::sync::Mutex<RakClient>>,
}

impl From<RakSession> for RakNetConnection {
    fn from(session: RakSession) -> Self {
        Self {
            session,
            dialled_from: None,
        }
    }
}

impl RakNetConnection {
    pub fn dialled(session: RakSession, client: RakClient) -> Self {
        Self {
            session,
            dialled_from: Some(::tokio::sync::Mutex::new(client)),
        }
    }

    pub async fn send(&mut self, batch: &[u8]) -> Result<(), TransportLayerError> {
        self.session
            .send(
                RakNetGamePacket::wrap(batch),
                RakReliability::ReliableOrdered,
                RakPriority::Immediate,
            )
            .await
            .map_err(RakNetError::from)?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>, TransportLayerError> {
        let datagram = self
            .session
            .recv::<Vec<u8>>()
            .await
            .map_err(RakNetError::from)?;

        match RakNetGamePacket::unwrap(&datagram) {
            Ok(batch) => Ok(batch.to_vec()),
            Err(NetworkCodecError::InvalidGamePacketHeader(found)) => Err(
                TransportLayerError::RakNetError(RakNetError::InvalidRakNetHeader(found)),
            ),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into()),
        }
    }

    pub fn remote_addr(&self) -> SocketAddr {
        self.session.get_addr()
    }

    pub fn max_message_len(&self) -> usize {
        RakSessionConfig::default().max_queued_bytes as usize
    }

    pub async fn close(&self) {
        let _ = self.session.close().await;
        if let Some(client) = &self.dialled_from {
            client.lock().await.stop().await;
        }
    }

    pub async fn is_closed(&self) -> bool {
        self.session.is_closed().await
    }
}
