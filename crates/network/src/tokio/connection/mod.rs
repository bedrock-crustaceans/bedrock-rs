pub mod shard;

use super::error::ConnectionError;
use super::transport::TransportLayerConnection;
use crate::codec::{
    compress_packets, decode_packets, decompress_packets, decrypt_packets, encode_packets,
    encrypt_packets,
};
use crate::compression::Compression;
use crate::encryption::Encryption;
use bedrock_protocol_core::Packets;
use std::marker::PhantomData;
use std::net::SocketAddr;

pub struct Connection<V: Packets> {
    /// Represents the Connection's internal transport layer, which may vary
    transport_layer: TransportLayerConnection,
    /// Represents the Connection's Compression, the compression gets initialized in the
    /// login process
    pub compression: Option<Compression>,
    /// Represents the connections encryption, the encryption gets initialized in the
    /// login process, if encryption is enabled
    pub encryption: Option<Encryption>,
    _version_marker: PhantomData<V>,
}

impl<V: Packets> Connection<V> {
    pub fn into_ver<T: Packets>(self) -> Connection<T> {
        Connection::<T> {
            transport_layer: self.transport_layer,
            compression: self.compression,
            encryption: self.encryption,
            _version_marker: PhantomData,
        }
    }

    pub fn from_transport_conn(transport_layer: TransportLayerConnection) -> Self {
        Self {
            transport_layer,
            compression: None,
            encryption: None,
            _version_marker: PhantomData,
        }
    }

    pub fn enable_compression(&mut self, compression: Compression) {
        self.compression = Some(compression);
    }

    pub fn enable_encryption(&mut self, encryption: Encryption) {
        self.encryption = Some(encryption);
    }

    pub fn get_transport_conn(&self) -> &TransportLayerConnection {
        &self.transport_layer
    }

    pub fn get_socket_addr(&self) -> SocketAddr {
        self.transport_layer.remote_addr()
    }

    pub async fn send(&mut self, packets: &[V]) -> Result<(), ConnectionError> {
        let packets_stream =
            encode_packets::<V>(packets, self.compression.as_ref(), self.encryption.as_mut())?;

        self.transport_layer.send(&packets_stream).await?;

        Ok(())
    }

    pub async fn send_raw(&mut self, data: &[u8]) -> Result<(), ConnectionError> {
        self.transport_layer.send(data).await?;

        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<V>, ConnectionError> {
        let packet_stream = self.transport_layer.recv().await?;

        let packets = decode_packets::<V>(
            packet_stream,
            self.compression.as_ref(),
            self.encryption.as_mut(),
            self.transport_layer.max_message_len(),
        )?;

        Ok(packets)
    }

    pub async fn recv_raw(&mut self) -> Result<Vec<u8>, ConnectionError> {
        let stream = self.transport_layer.recv().await?;

        Ok(stream)
    }

    /// Receives one decrypted and decompressed batch without decoding its packets.
    pub async fn recv_batch(&mut self) -> Result<Vec<u8>, ConnectionError> {
        let stream = self.transport_layer.recv().await?;
        let stream = decrypt_packets(stream, self.encryption.as_mut())?;
        let stream = decompress_packets(
            stream,
            self.compression.as_ref(),
            self.transport_layer.max_message_len(),
        )?;

        Ok(stream)
    }

    /// Compresses, encrypts and sends an already encoded batch.
    pub async fn send_batch(&mut self, batch: Vec<u8>) -> Result<(), ConnectionError> {
        let stream = compress_packets(batch, self.compression.as_ref())?;
        let stream = encrypt_packets(stream, self.encryption.as_mut())?;

        self.transport_layer.send(&stream).await?;

        Ok(())
    }

    pub async fn close(&self) {
        self.transport_layer.close().await;
    }

    pub async fn is_closed(&self) -> bool {
        self.transport_layer.is_closed().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::login::{LoginPackets, LoginStatus};
    use crate::test_helpers::memory_connections;
    use bedrock_protocol::V2225;

    #[::tokio::test]
    async fn zlib_enabled_on_both_ends_carries_a_large_batch() {
        let (mut a, mut b) = memory_connections();
        let zlib = Compression::Zlib {
            threshold: 1,
            compression_level: 6,
        };
        a.enable_compression(zlib.clone());
        b.enable_compression(zlib);
        assert!(a.compression.is_some());
        assert!(a.encryption.is_none());

        let batch = vec![V2225::play_status(LoginStatus::LoginSuccess); 4000];
        a.send(&batch).await.unwrap();

        assert_eq!(b.recv().await.unwrap().len(), 4000);
    }
}
