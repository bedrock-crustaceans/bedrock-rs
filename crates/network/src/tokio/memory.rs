use super::Connection;
use super::error::TransportLayerError;
use super::transport::Transport;
use bedrock_protocol::V2225;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

pub(crate) struct MemoryTransport {
    tx: mpsc::UnboundedSender<Vec<u8>>,
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
    closed: Arc<AtomicBool>,
}

impl MemoryTransport {
    fn pair() -> (Self, Self) {
        let (a_tx, b_rx) = mpsc::unbounded_channel();
        let (b_tx, a_rx) = mpsc::unbounded_channel();
        let closed = Arc::new(AtomicBool::new(false));

        let a = Self {
            tx: a_tx,
            rx: a_rx,
            closed: closed.clone(),
        };
        let b = Self {
            tx: b_tx,
            rx: b_rx,
            closed,
        };

        (a, b)
    }
}

impl Transport for MemoryTransport {
    async fn send(&mut self, batch: &[u8]) -> Result<(), TransportLayerError> {
        self.tx
            .send(batch.to_vec())
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::BrokenPipe))?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<Vec<u8>, TransportLayerError> {
        Ok(self
            .rx
            .recv()
            .await
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::UnexpectedEof))?)
    }

    fn remote_addr(&self) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], 0))
    }

    fn max_message_len(&self) -> usize {
        1 << 20
    }

    async fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }

    async fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst) || self.tx.is_closed()
    }
}

pub(crate) type MemoryConnection = Connection<V2225, MemoryTransport>;

pub(crate) fn memory_connections() -> (MemoryConnection, MemoryConnection) {
    let (a, b) = MemoryTransport::pair();
    (
        Connection::from_transport_conn(a),
        Connection::from_transport_conn(b),
    )
}
