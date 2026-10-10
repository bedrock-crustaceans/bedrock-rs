use crate::tokio::error::{RakNetError, TransportLayerError};
use crate::tokio::transport::TransportLayerConnection;
use raknet_tokio::prelude::*;

pub enum TransportLayerListener {
    RakNet(RakServer),
    // TODO: NetherNet(...),
    // TODO: Quic(s2n_quic::server::Server),
    // TODO: Tcp(...),
}

impl TransportLayerListener {
    pub fn set_message(&mut self, message: Box<[u8]>) {
        match self {
            Self::RakNet(listener) => listener.set_message(message),
        }
    }

    pub fn set_max_connections(&mut self, n: usize) {
        match self {
            Self::RakNet(listener) => listener.set_max_connections(n),
        }
    }

    pub async fn start(&mut self) -> Result<(), TransportLayerError> {
        match self {
            Self::RakNet(listener) => listener.start().await.map_err(RakNetError::from)?,
        };

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), TransportLayerError> {
        match self {
            Self::RakNet(listener) => listener.stop().await,
        }

        Ok(())
    }

    pub async fn accept(&mut self) -> Result<TransportLayerConnection, TransportLayerError> {
        let conn = match self {
            Self::RakNet(listener) => TransportLayerConnection::RakNet(
                listener.accept().await.map_err(RakNetError::from)?,
            ),
        };

        Ok(conn)
    }
}
