use crate::error::NetworkCodecError;
use crate::raknet::RakNetGamePacket;
use crate::tokio::error::{RakNetError, TransportLayerError};
#[cfg(any(test, feature = "test-util"))]
use ::tokio::sync::mpsc;
use raknet_tokio::prelude::*;
#[cfg(any(test, feature = "test-util", feature = "nethernet-tokio"))]
use std::net::SocketAddr;
#[cfg(any(test, feature = "test-util"))]
use std::sync::Arc;
#[cfg(any(test, feature = "test-util"))]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(any(test, feature = "test-util"))]
pub struct MemoryConnection {
    tx: mpsc::UnboundedSender<Vec<u8>>,
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
    max_message_len: usize,
    closed: Arc<AtomicBool>,
}

#[cfg(feature = "nethernet-tokio")]
pub struct NetherNetConnection {
    session: nethernet_tokio::Session,
    reliable: nethernet_tokio::SessionReceiver,
    _unreliable: nethernet_tokio::SessionReceiver,
    remote: nethernet_tokio::Addr,
}

pub enum TransportLayerConnection {
    RakNet(RakSession),
    RakNetClient {
        session: RakSession,
        client: ::tokio::sync::Mutex<RakClient>,
    },
    #[cfg(feature = "nethernet-tokio")]
    NetherNet(NetherNetConnection),
    #[cfg(any(test, feature = "test-util"))]
    Memory(MemoryConnection),
    // TODO: Quic(s2n_quic::stream::BidirectionalStream),
    // TODO: Tcp(net::TcpStream),
}

impl TransportLayerConnection {
    #[cfg(feature = "nethernet-tokio")]
    pub async fn nethernet(accepted: nethernet_tokio::AcceptedSession) -> Self {
        let remote = accepted.session.remote_addr().await;
        Self::NetherNet(NetherNetConnection {
            session: accepted.session,
            reliable: accepted.reliable,
            _unreliable: accepted.unreliable,
            remote,
        })
    }

    #[cfg(any(test, feature = "test-util"))]
    pub fn memory_pair(max_message_len: usize) -> (Self, Self) {
        let (a_tx, b_rx) = mpsc::unbounded_channel();
        let (b_tx, a_rx) = mpsc::unbounded_channel();
        let closed = Arc::new(AtomicBool::new(false));

        let a = MemoryConnection {
            tx: a_tx,
            rx: a_rx,
            max_message_len,
            closed: closed.clone(),
        };
        let b = MemoryConnection {
            tx: b_tx,
            rx: b_rx,
            max_message_len,
            closed,
        };

        (Self::Memory(a), Self::Memory(b))
    }

    pub async fn send(&mut self, stream: &[u8]) -> Result<(), TransportLayerError> {
        match self {
            Self::RakNet(conn) | Self::RakNetClient { session: conn, .. } => {
                conn.send(
                    RakNetGamePacket::wrap(stream),
                    RakReliability::ReliableOrdered,
                    RakPriority::Immediate,
                )
                .await
                .map_err(RakNetError::from)?;
            }
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(conn) => {
                conn.session
                    .send(bytes::Bytes::copy_from_slice(stream))
                    .await?;
            }
            #[cfg(any(test, feature = "test-util"))]
            Self::Memory(conn) => {
                conn.tx
                    .send(RakNetGamePacket::wrap(stream))
                    .map_err(|_| std::io::Error::from(std::io::ErrorKind::BrokenPipe))?;
            }
        }

        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>, TransportLayerError> {
        let datagram = match self {
            Self::RakNet(conn) | Self::RakNetClient { session: conn, .. } => {
                conn.recv::<Vec<u8>>().await.map_err(RakNetError::from)?
            }
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(conn) => {
                return match conn.reliable.recv().await? {
                    Some(batch) => Ok(batch.to_vec()),
                    None => Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into()),
                };
            }
            #[cfg(any(test, feature = "test-util"))]
            Self::Memory(conn) => conn
                .rx
                .recv()
                .await
                .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::UnexpectedEof))?,
        };

        match RakNetGamePacket::unwrap(&datagram) {
            Ok(batch) => Ok(batch.to_vec()),
            Err(NetworkCodecError::InvalidGamePacketHeader(found)) => Err(
                TransportLayerError::RakNetError(RakNetError::InvalidRakNetHeader(found)),
            ),
            Err(_) => Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into()),
        }
    }

    pub fn remote_addr(&self) -> std::net::SocketAddr {
        match self {
            Self::RakNet(rak) | Self::RakNetClient { session: rak, .. } => rak.get_addr(),
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(conn) => conn
                .remote
                .socket_addr
                .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 0))),
            #[cfg(any(test, feature = "test-util"))]
            Self::Memory(_) => SocketAddr::from(([127, 0, 0, 1], 0)),
        }
    }

    pub fn max_message_len(&self) -> usize {
        match self {
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(_) => RakSessionConfig::default().max_queued_bytes as usize,
            Self::RakNet(_) | Self::RakNetClient { .. } => {
                RakSessionConfig::default().max_queued_bytes as usize
            }
            #[cfg(any(test, feature = "test-util"))]
            Self::Memory(conn) => conn.max_message_len,
        }
    }

    pub async fn close(&self) {
        match self {
            Self::RakNet(conn) => {
                let _ = conn.close().await;
            }
            Self::RakNetClient { session, client } => {
                let _ = session.close().await;
                client.lock().await.stop().await;
            }
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(conn) => {
                let _ = conn.session.close().await;
            }
            #[cfg(any(test, feature = "test-util"))]
            Self::Memory(conn) => {
                conn.closed.store(true, Ordering::SeqCst);
            }
        }
    }

    pub async fn is_closed(&self) -> bool {
        match self {
            Self::RakNet(conn) | Self::RakNetClient { session: conn, .. } => conn.is_closed().await,
            #[cfg(feature = "nethernet-tokio")]
            Self::NetherNet(conn) => conn.session.is_closed().await,
            #[cfg(any(test, feature = "test-util"))]
            Self::Memory(conn) => conn.closed.load(Ordering::SeqCst) || conn.tx.is_closed(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{decode_packets, encode_packets};
    use crate::login::{LoginPackets, LoginStatus};
    use bedrock_protocol::V2225;

    #[::tokio::test]
    async fn memory_pair_carries_packets_both_ways() {
        let (mut a, mut b) = TransportLayerConnection::memory_pair(1 << 20);
        let to_b =
            encode_packets::<V2225>(&[V2225::play_status(LoginStatus::LoginSuccess)], None, None)
                .unwrap();
        let to_a =
            encode_packets::<V2225>(&[V2225::play_status(LoginStatus::PlayerSpawn)], None, None)
                .unwrap();

        a.send(&to_b).await.unwrap();
        b.send(&to_a).await.unwrap();

        assert_eq!(b.recv().await.unwrap(), to_b);
        assert_eq!(a.recv().await.unwrap(), to_a);
        let decoded = decode_packets::<V2225>(to_b, None, None, 1 << 20).unwrap();
        assert_eq!(decoded.len(), 1);
    }

    #[::tokio::test]
    async fn memory_pair_rejects_wrong_gamepacket_header() {
        let (a, mut b) = TransportLayerConnection::memory_pair(1024);
        let TransportLayerConnection::Memory(a) = &a else {
            unreachable!()
        };
        a.tx.send(vec![0x00, 1, 2]).unwrap();

        let result = b.recv().await;
        assert!(matches!(
            result,
            Err(TransportLayerError::RakNetError(
                RakNetError::InvalidRakNetHeader(0x00)
            ))
        ));
    }
}
