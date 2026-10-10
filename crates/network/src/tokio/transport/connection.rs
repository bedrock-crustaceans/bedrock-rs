use crate::info::RAKNET_GAMEPACKET_ID;
use crate::tokio::error::{RakNetError, TransportLayerError};
#[cfg(any(test, feature = "test-util"))]
use ::tokio::sync::mpsc;
use byteorder::{ReadBytesExt, WriteBytesExt};
use raknet_tokio::prelude::*;
use std::io::{Cursor, Write};
#[cfg(any(test, feature = "test-util"))]
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

pub enum TransportLayerConnection {
    RakNet(RakSession),
    RakNetClient {
        session: RakSession,
        client: ::tokio::sync::Mutex<RakClient>,
    },
    #[cfg(any(test, feature = "test-util"))]
    Memory(MemoryConnection),
    // TODO: NetherNet(nethernet::connection::Connection),
    // TODO: Quic(s2n_quic::stream::BidirectionalStream),
    // TODO: Tcp(net::TcpStream),
}

impl TransportLayerConnection {
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
                // 1 = RAKNET_GAMEPACKET_ID size
                let mut buf = Vec::with_capacity(stream.len() + 1);

                // TODO Find out a way to avoid copying of the entire buffer
                buf.write_u8(RAKNET_GAMEPACKET_ID)?;
                buf.write_all(stream)?;

                // TODO Find out if immediate: true should be used
                conn.send(buf, RakReliability::ReliableOrdered, RakPriority::Immediate)
                    .await
                    .map_err(RakNetError::from)?;
            }
            #[cfg(any(test, feature = "test-util"))]
            Self::Memory(conn) => {
                let mut buf = Vec::with_capacity(stream.len() + 1);
                buf.write_u8(RAKNET_GAMEPACKET_ID)?;
                buf.write_all(stream)?;

                conn.tx
                    .send(buf)
                    .map_err(|_| std::io::Error::from(std::io::ErrorKind::BrokenPipe))?;
            }
        }

        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>, TransportLayerError> {
        let stream = match self {
            Self::RakNet(conn) | Self::RakNetClient { session: conn, .. } => {
                let stream: Vec<u8> = conn.recv().await.map_err(RakNetError::from)?;

                let mut stream = Cursor::new(stream);

                // Read the RakNet Packet ID
                let raknet_packet_id = stream.read_u8()?;

                if raknet_packet_id != RAKNET_GAMEPACKET_ID {
                    return Err(TransportLayerError::RakNetError(
                        RakNetError::InvalidRakNetHeader(raknet_packet_id),
                    ));
                };

                let mut stream = stream.into_inner();
                stream.drain(..1);

                stream
            }
            #[cfg(any(test, feature = "test-util"))]
            Self::Memory(conn) => {
                let mut stream = conn
                    .rx
                    .recv()
                    .await
                    .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::UnexpectedEof))?;

                let packet_id = *stream
                    .first()
                    .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::UnexpectedEof))?;

                if packet_id != RAKNET_GAMEPACKET_ID {
                    return Err(TransportLayerError::RakNetError(
                        RakNetError::InvalidRakNetHeader(packet_id),
                    ));
                }

                stream.drain(..1);

                stream
            }
        };

        Ok(stream)
    }

    pub fn remote_addr(&self) -> std::net::SocketAddr {
        match self {
            Self::RakNet(rak) | Self::RakNetClient { session: rak, .. } => rak.get_addr(),
            #[cfg(any(test, feature = "test-util"))]
            Self::Memory(_) => SocketAddr::from(([127, 0, 0, 1], 0)),
        }
    }

    pub fn max_message_len(&self) -> usize {
        match self {
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
            #[cfg(any(test, feature = "test-util"))]
            Self::Memory(conn) => {
                conn.closed.store(true, Ordering::SeqCst);
            }
        }
    }

    pub async fn is_closed(&self) -> bool {
        match self {
            Self::RakNet(conn) | Self::RakNetClient { session: conn, .. } => conn.is_closed().await,
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
