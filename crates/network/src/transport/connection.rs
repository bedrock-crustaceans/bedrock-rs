use crate::error::{RakNetError, TransportLayerError};
use crate::info::RAKNET_GAMEPACKET_ID;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Write};

pub enum TransportLayerConnection {
    RakNet(rak_rs::connection::Connection),
    // TODO: NetherNet(nethernet::connection::Connection),
    // TODO: Quic(s2n_quic::stream::BidirectionalStream),
    // TODO: Tcp(net::TcpStream),
}

impl TransportLayerConnection {
    pub async fn send(&mut self, stream: &[u8]) -> Result<(), TransportLayerError> {
        match self {
            Self::RakNet(conn) => {
                // 1 = RAKNET_GAMEPACKET_ID size
                let mut str = Vec::with_capacity(stream.len() + 1);

                // TODO Find out a way to avoid copying of the entire buffer
                str.write_u8(RAKNET_GAMEPACKET_ID)?;
                str.write_all(stream)?;

                // TODO Find out if immediate: true should be used
                conn.send(str.as_slice(), true)
                    .await
                    .map_err(|err| TransportLayerError::RakNetError(RakNetError::SendError(err)))?;
            }
        }

        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>, TransportLayerError> {
        let stream = match self {
            Self::RakNet(conn) => {
                let stream = conn
                    .recv()
                    .await
                    .map_err(|e| TransportLayerError::RakNetError(RakNetError::RecvError(e)))?;

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
                conn.close().await;
            }
        }
    }

    pub async fn is_closed(&self) -> bool {
        match self {
            Self::RakNet(conn) => conn.is_closed().await,
        }
    }
}
