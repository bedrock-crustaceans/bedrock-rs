use crate::error::{RakNetError, TransportLayerError};
use crate::transport::TransportLayerConnection;

pub enum TransportLayerListener {
    RakNet(rak_rs::Listener),
    // TODO NetherNet(...),
    // TODO Quic(s2n_quic::server::Server),
    // TODO Tcp(...),
}

impl TransportLayerListener {
    pub async fn start(&mut self) -> Result<(), TransportLayerError> {
        match self {
            Self::RakNet(listener) => listener
                .start()
                .await
                .map_err(|err| TransportLayerError::RakNetError(RakNetError::ServerError(err)))?,
        };

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), TransportLayerError> {
        match self {
            Self::RakNet(listener) => listener
                .stop()
                .await
                .map_err(|err| TransportLayerError::RakNetError(RakNetError::ServerError(err)))?,
        }

        Ok(())
    }

    pub async fn accept(&mut self) -> Result<TransportLayerConnection, TransportLayerError> {
        let conn = match self {
            Self::RakNet(listener) => {
                TransportLayerConnection::RakNet(listener.accept().await.map_err(|err| {
                    TransportLayerError::RakNetError(RakNetError::ServerError(err))
                })?)
            }
        };

        Ok(conn)
    }
}
