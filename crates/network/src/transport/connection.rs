use crate::error::{RakNetError, TransportLayerError};
use crate::info::RAKNET_GAMEPACKET_ID;
use byteorder::{ReadBytesExt, WriteBytesExt};
use raknet_tokio::prelude::*;
use std::io::{Cursor, Write};

pub enum TransportLayerConnection {
    RakNet(RakSession),
    // TODO: NetherNet(nethernet::connection::Connection),
    // TODO: Quic(s2n_quic::stream::BidirectionalStream),
    // TODO: Tcp(net::TcpStream),
}

impl TransportLayerConnection {
    pub async fn send(&mut self, stream: &[u8]) -> Result<(), TransportLayerError> {
        match self {
            Self::RakNet(conn) => {
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
        }

        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>, TransportLayerError> {
        let stream = match self {
            Self::RakNet(conn) => {
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
        };

        Ok(stream)
    }

    pub async fn close(&self) {
        match self {
            Self::RakNet(conn) => {
                let _ = conn.close().await;
            }
        }
    }

    pub async fn is_closed(&self) -> bool {
        match self {
            Self::RakNet(conn) => conn.is_closed().await,
        }
    }
}
