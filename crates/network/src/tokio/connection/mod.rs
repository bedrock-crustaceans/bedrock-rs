pub mod shard;

use super::error::ConnectionError;
use super::transport::{Transport, TransportLayerConnection};
use crate::codec::{
    DecodedPackets, MAX_BATCH_LEN, UnreadBytes, compress_packets, decode_packets_each,
    decompress_packets, decrypt_packets, encode_packets_with_header, encrypt_packets,
};
use crate::compression::Compression;
use crate::encryption::Encryption;
use bedrock_protocol_core::{PacketHeader, Packets};
use std::marker::PhantomData;
use std::net::SocketAddr;

/// What `recv` and `recv_with_headers` do with a packet that fails to decode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadPacketPolicy {
    /// The whole batch fails with the packet's error.
    #[default]
    Fail,
    /// The packet is dropped with a warning and the rest of the batch is delivered.
    Skip,
}

pub struct Connection<V: Packets, T = TransportLayerConnection> {
    /// Represents the Connection's internal transport layer, which may vary
    transport_layer: T,
    /// Represents the Connection's Compression, the compression gets initialized in the
    /// login process
    pub compression: Option<Compression>,
    /// Represents the connections encryption, the encryption gets initialized in the
    /// login process, if encryption is enabled
    pub encryption: Option<Encryption>,
    pub unread_bytes: UnreadBytes,
    pub bad_packets: BadPacketPolicy,
    pub max_batch_len: usize,
    _version_marker: PhantomData<V>,
}

impl<V: Packets, T: Transport> Connection<V, T> {
    pub fn into_ver<W: Packets>(self) -> Connection<W, T> {
        Connection::<W, T> {
            transport_layer: self.transport_layer,
            compression: self.compression,
            encryption: self.encryption,
            unread_bytes: self.unread_bytes,
            bad_packets: self.bad_packets,
            max_batch_len: self.max_batch_len,
            _version_marker: PhantomData,
        }
    }

    pub fn from_transport_conn(transport_layer: T) -> Self {
        Self {
            transport_layer,
            compression: None,
            encryption: None,
            unread_bytes: UnreadBytes::default(),
            bad_packets: BadPacketPolicy::default(),
            max_batch_len: MAX_BATCH_LEN,
            _version_marker: PhantomData,
        }
    }

    pub fn enable_compression(&mut self, compression: Compression) {
        self.compression = Some(compression);
    }

    pub fn enable_encryption(&mut self, encryption: Encryption) {
        self.encryption = Some(encryption);
    }

    pub fn get_transport_conn(&self) -> &T {
        &self.transport_layer
    }

    pub fn get_socket_addr(&self) -> SocketAddr {
        self.transport_layer.remote_addr()
    }

    /// Sends `packets` in one batch with the default header.
    ///
    /// # Errors
    ///
    /// Returns a [`ConnectionError`] when encoding the batch or the transport fails.
    pub async fn send(&mut self, packets: &[V]) -> Result<(), ConnectionError> {
        self.send_with_header(packets, &PacketHeader::default())
            .await
    }

    /// Sends `packets` with the sender and target sub-client ids of `header` on every frame;
    /// `header.packet_id` is ignored. A server routes a received packet by its sender
    /// sub-client id and a client by its target sub-client id.
    ///
    /// # Errors
    ///
    /// Returns a [`ConnectionError`] when encoding the batch or the transport fails.
    pub async fn send_with_header(
        &mut self,
        packets: &[V],
        header: &PacketHeader,
    ) -> Result<(), ConnectionError> {
        let packets_stream = encode_packets_with_header::<V>(
            packets,
            header,
            self.compression.as_ref(),
            self.encryption.as_mut(),
        )?;

        self.transport_layer.send(&packets_stream).await?;

        Ok(())
    }

    /// Sends `data` as one transport message, without batching, compression or encryption.
    ///
    /// # Errors
    ///
    /// Returns a [`ConnectionError`] when the transport fails.
    pub async fn send_raw(&mut self, data: &[u8]) -> Result<(), ConnectionError> {
        self.transport_layer.send(data).await?;

        Ok(())
    }

    /// Receives one batch and returns its packets in order.
    ///
    /// # Errors
    ///
    /// The errors of [`Self::recv_with_headers`].
    pub async fn recv(&mut self) -> Result<Vec<V>, ConnectionError> {
        Ok(self
            .recv_with_headers()
            .await?
            .into_iter()
            .map(|(_, packet)| packet)
            .collect())
    }

    /// Receives one batch with each packet's header, exposing its sender and target
    /// sub-client ids. A packet that fails to decode is handled per `bad_packets`.
    ///
    /// # Errors
    ///
    /// Returns a [`ConnectionError`] when the transport or the batch fails, or when a packet fails
    /// to decode and `bad_packets` is [`BadPacketPolicy::Fail`].
    pub async fn recv_with_headers(&mut self) -> Result<Vec<(PacketHeader, V)>, ConnectionError> {
        let decoded = self.recv_each().await?;
        let mut packets = Vec::with_capacity(decoded.len());

        for (header, result) in decoded {
            match result {
                Ok(packet) => packets.push((header, packet)),
                Err(error) if self.bad_packets == BadPacketPolicy::Skip => {
                    tracing::warn!(
                        "dropping packet {} that failed to decode: {error}",
                        header.packet_id,
                    );
                }
                Err(error) => return Err(error.into()),
            }
        }

        Ok(packets)
    }

    /// Receives one batch and returns the outcome of every packet in order, regardless of
    /// `bad_packets`. Only a failure of the batch itself is an error.
    ///
    /// # Errors
    ///
    /// Returns a [`ConnectionError`] when the transport fails or the batch fails to decrypt,
    /// decompress or split into frames.
    pub async fn recv_each(&mut self) -> Result<DecodedPackets<V>, ConnectionError> {
        let packet_stream = self.transport_layer.recv().await?;

        Ok(decode_packets_each::<V>(
            packet_stream,
            self.compression.as_ref(),
            self.encryption.as_mut(),
            self.max_batch_len,
            self.unread_bytes,
        )?)
    }

    /// Receives one transport message as it arrived, without decrypting or decoding it.
    ///
    /// # Errors
    ///
    /// Returns a [`ConnectionError`] when the transport fails.
    pub async fn recv_raw(&mut self) -> Result<Vec<u8>, ConnectionError> {
        let stream = self.transport_layer.recv().await?;

        Ok(stream)
    }

    /// Receives one decrypted and decompressed batch without decoding its packets.
    ///
    /// # Errors
    ///
    /// Returns a [`ConnectionError`] when the transport fails or the batch fails to decrypt or
    /// decompress.
    pub async fn recv_batch(&mut self) -> Result<Vec<u8>, ConnectionError> {
        let stream = self.transport_layer.recv().await?;
        let stream = decrypt_packets(stream, self.encryption.as_mut())?;
        let stream = decompress_packets(stream, self.compression.as_ref(), self.max_batch_len)?;

        Ok(stream)
    }

    /// Compresses, encrypts and sends an already encoded batch.
    ///
    /// # Errors
    ///
    /// Returns a [`ConnectionError`] when compression, encryption or the transport fails.
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
    use crate::codec::{encode_packets, join_batch, split_batch};
    use crate::test_helpers::login_success;
    use crate::tokio::memory::memory_connections;
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

        let batch = vec![login_success(); 4000];
        a.send(&batch).await.unwrap();

        assert_eq!(b.recv().await.unwrap().len(), 4000);
    }

    #[::tokio::test]
    async fn sub_client_ids_cross_the_connection() {
        let (mut a, mut b) = memory_connections();
        let header = PacketHeader {
            packet_id: 0,
            sender_sub_client_id: 3,
            target_sub_client_id: 1,
        };

        a.send_with_header(&[login_success()], &header)
            .await
            .unwrap();

        let received = b.recv_with_headers().await.unwrap();
        assert_eq!(received[0].0.sender_sub_client_id, 3);
        assert_eq!(received[0].0.target_sub_client_id, 1);
    }

    #[::tokio::test]
    async fn decompressed_batches_are_bounded_by_the_batch_limit_not_the_transport() {
        let (mut a, mut b) = memory_connections();
        let zlib = Compression::Zlib {
            threshold: 1,
            compression_level: 6,
        };
        a.enable_compression(zlib.clone());
        b.enable_compression(zlib);

        a.send_batch(vec![0u8; 2 << 20]).await.unwrap();
        assert_eq!(b.recv_batch().await.unwrap().len(), 2 << 20);

        a.send_batch(vec![0u8; (10 << 20) + 1]).await.unwrap();
        assert!(b.recv_batch().await.is_err());
    }

    fn batch_with_a_bad_packet_at(position: usize) -> Vec<u8> {
        let good = encode_packets::<V2225>(&[login_success()], None, None).unwrap();
        let good = split_batch(&good).unwrap()[0].to_vec();
        let mut frames = vec![good.clone(), good.clone(), good];
        frames[position] = vec![2, 1];
        join_batch(&frames)
    }

    #[::tokio::test]
    async fn a_bad_packet_fails_the_batch_by_default() {
        for position in 0..3 {
            let (mut a, mut b) = memory_connections();
            a.send_batch(batch_with_a_bad_packet_at(position))
                .await
                .unwrap();

            assert!(b.recv().await.is_err(), "bad packet at {position}");
        }
    }

    #[::tokio::test]
    async fn skip_delivers_the_good_packets_around_a_bad_one() {
        for position in 0..3 {
            let (mut a, mut b) = memory_connections();
            b.bad_packets = BadPacketPolicy::Skip;
            a.send_batch(batch_with_a_bad_packet_at(position))
                .await
                .unwrap();

            assert_eq!(b.recv().await.unwrap().len(), 2, "bad packet at {position}");
        }
    }

    #[::tokio::test]
    async fn recv_each_reports_every_packet_in_order() {
        for position in 0..3 {
            let (mut a, mut b) = memory_connections();
            a.send_batch(batch_with_a_bad_packet_at(position))
                .await
                .unwrap();

            let outcomes: Vec<bool> = b
                .recv_each()
                .await
                .unwrap()
                .iter()
                .map(|(_, result)| result.is_ok())
                .collect();

            let expected: Vec<bool> = (0..3).map(|index| index != position).collect();
            assert_eq!(outcomes, expected);
        }
    }
}
