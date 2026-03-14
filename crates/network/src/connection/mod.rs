pub mod shard;

use crate::error::ConnectionError;
use crate::transport::TransportLayerConnection;
use bedrockrs_proto::codec::{decode_packets, encode_packets};
use bedrockrs_proto::compression::Compression;
use bedrockrs_proto::encryption::Encryption;
use bedrockrs_proto_core::Packets;
use std::net::SocketAddr;

pub struct Connection {
    /// Represents the Connection's internal transport layer, which may vary
    transport_layer: TransportLayerConnection,
    /// Represents the Connection's Compression, the compression gets initialized in the
    /// login process
    pub compression: Option<Compression>,
    /// Represents the connections encryption, the encryption gets initialized in the
    /// login process, if encryption is enabled
    pub encryption: Option<Encryption>,
}

impl Connection {
    pub fn from_transport_conn(transport_layer: TransportLayerConnection) -> Self {
        Self {
            transport_layer,
            compression: None,
            encryption: None,
        }
    }

    pub fn get_transport_conn(&self) -> &TransportLayerConnection {
        &self.transport_layer
    }

    pub fn get_socket_addr(&self) -> &SocketAddr {
        match &self.transport_layer {
            TransportLayerConnection::RakNet(rak) => &rak.address,
        }
    }

    pub async fn send<T: Packets>(&mut self, packets: &[T]) -> Result<(), ConnectionError> {
        let packets_stream =
            encode_packets::<T>(packets, self.compression.as_ref(), self.encryption.as_mut())?;

        self.transport_layer.send(&packets_stream).await?;

        Ok(())
    }

    pub async fn send_raw(&mut self, data: &[u8]) -> Result<(), ConnectionError> {
        self.transport_layer.send(data).await?;

        Ok(())
    }

    pub async fn recv<T: Packets>(&mut self) -> Result<Vec<T>, ConnectionError> {
        let packet_stream = self.transport_layer.recv().await?;

        let packets = decode_packets::<T>(
            packet_stream,
            self.compression.as_ref(),
            self.encryption.as_mut(),
        )?;

        Ok(packets)
    }

    pub async fn recv_raw(&mut self) -> Result<Vec<u8>, ConnectionError> {
        let stream = self.transport_layer.recv().await?;

        Ok(stream)
    }

    pub async fn close(&self) {
        self.transport_layer.close().await;
    }

    pub async fn is_closed(&self) -> bool {
        self.transport_layer.is_closed().await
    }
}
