#[cfg(feature = "nethernet-tokio")]
use super::NetherNetConnection;
use super::RakNetConnection;
use crate::tokio::error::TransportLayerError;
use std::future::Future;
use std::net::SocketAddr;

pub trait Transport {
    fn send(
        &mut self,
        batch: &[u8],
    ) -> impl Future<Output = Result<(), TransportLayerError>> + Send;

    fn recv(&mut self) -> impl Future<Output = Result<Vec<u8>, TransportLayerError>> + Send;

    fn remote_addr(&self) -> SocketAddr;

    fn max_message_len(&self) -> usize;

    fn close(&self) -> impl Future<Output = ()> + Send;

    fn is_closed(&self) -> impl Future<Output = bool> + Send;
}

pub enum TransportLayerConnection {
    RakNet(RakNetConnection),
    #[cfg(feature = "nethernet-tokio")]
    NetherNet(NetherNetConnection),
    // TODO: Quic(s2n_quic::stream::BidirectionalStream),
    // TODO: Tcp(net::TcpStream),
}

macro_rules! delegate {
    ($self:ident, $conn:ident => $call:expr) => {
        match $self {
            Self::RakNet($conn) => $call,
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet($conn) => $call,
        }
    };
}

impl Transport for TransportLayerConnection {
    async fn send(&mut self, batch: &[u8]) -> Result<(), TransportLayerError> {
        delegate!(self, conn => conn.send(batch).await)
    }

    async fn recv(&mut self) -> Result<Vec<u8>, TransportLayerError> {
        delegate!(self, conn => conn.recv().await)
    }

    fn remote_addr(&self) -> SocketAddr {
        delegate!(self, conn => conn.remote_addr())
    }

    fn max_message_len(&self) -> usize {
        delegate!(self, conn => conn.max_message_len())
    }

    async fn close(&self) {
        delegate!(self, conn => conn.close().await)
    }

    async fn is_closed(&self) -> bool {
        delegate!(self, conn => conn.is_closed().await)
    }
}
