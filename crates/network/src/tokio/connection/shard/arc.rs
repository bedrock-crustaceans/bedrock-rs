use crate::tokio::connection::Connection;
use crate::tokio::error::ConnectionError;
use crate::tokio::transport::{Transport, TransportLayerConnection};
use ::tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use bedrock_protocol_core::{PacketHeader, Packets};
use std::collections::VecDeque;
use std::sync::Arc;

pub fn shard<T: Packets, Tr: Transport>(connection: Connection<T, Tr>) -> ConnectionShared<T, Tr> {
    ConnectionShared {
        connection: Arc::new(RwLock::new(connection)),
        queue_send: Arc::new(RwLock::new(Vec::new())),
        queue_recv: Arc::new(RwLock::new(VecDeque::new())),
    }
}

type SubClientRun<T> = (PacketHeader, Vec<T>);

#[derive(Clone)]
pub struct ConnectionShared<T: Packets, Tr = TransportLayerConnection> {
    connection: Arc<RwLock<Connection<T, Tr>>>,
    queue_send: Arc<RwLock<Vec<SubClientRun<T>>>>,
    queue_recv: Arc<RwLock<VecDeque<(PacketHeader, T)>>>,
}

impl<T: Packets, Tr: Transport> ConnectionShared<T, Tr> {
    /// Queues `packet` with the default header for the next [`send`](Self::send).
    ///
    /// # Errors
    ///
    /// Never fails today; the `Result` leaves room for queueing to become fallible.
    pub async fn write(&mut self, packet: T) -> Result<(), ConnectionError> {
        self.write_with_header(&PacketHeader::default(), packet)
            .await
    }

    /// Queues `packet` under the sub-client ids of `header` for the next [`send`](Self::send).
    ///
    /// # Errors
    ///
    /// Never fails today; the `Result` leaves room for queueing to become fallible.
    pub async fn write_with_header(
        &mut self,
        header: &PacketHeader,
        packet: T,
    ) -> Result<(), ConnectionError> {
        let mut queue_send = self.queue_send.write().await;
        let header = sub_clients(*header);

        match queue_send.last_mut() {
            Some((last, run)) if *last == header => run.push(packet),
            _ => queue_send.push((header, vec![packet])),
        }

        Ok(())
    }

    pub async fn read(&mut self) -> Option<T> {
        self.read_with_header().await.map(|(_, packet)| packet)
    }

    pub async fn read_with_header(&mut self) -> Option<(PacketHeader, T)> {
        let mut queue_recv = self.queue_recv.write().await;

        queue_recv.pop_front()
    }

    pub async fn read_all(&mut self) -> Vec<T> {
        self.read_all_with_headers()
            .await
            .into_iter()
            .map(|(_, packet)| packet)
            .collect()
    }

    pub async fn read_all_with_headers(&mut self) -> Vec<(PacketHeader, T)> {
        let mut queue_recv = self.queue_recv.write().await;
        queue_recv.make_contiguous();
        queue_recv.drain(..).collect::<Vec<_>>()
    }

    /// Sends the queued packets, one batch per run of packets sharing sub-client ids, and drops
    /// the runs that were sent.
    ///
    /// # Errors
    ///
    /// Returns the [`ConnectionError`] of the first batch that fails to send; it and the runs
    /// after it stay queued.
    pub async fn send(&mut self) -> Result<(), ConnectionError> {
        let mut queued = self.queue_send.write().await;
        let mut conn = self.connection.write().await;

        let mut sent = 0;
        let mut outcome = Ok(());
        for (header, run) in queued.iter() {
            outcome = conn.send_with_header(run, header).await;
            if outcome.is_err() {
                break;
            }
            sent += 1;
        }
        queued.drain(..sent);

        outcome
    }

    /// Receives one batch and queues its packets for [`read`](Self::read).
    ///
    /// # Errors
    ///
    /// The errors of [`Connection::recv_with_headers`].
    pub async fn recv(&mut self) -> Result<(), ConnectionError> {
        let mut conn = self.connection.write().await;

        let packets = conn.recv_with_headers().await?;

        if !packets.is_empty() {
            let mut queue_recv = self.queue_recv.write().await;

            queue_recv.extend(packets);
        }

        Ok(())
    }

    pub async fn close(&self) {
        self.connection.read().await.close().await;
    }

    pub async fn is_closed(&self) -> bool {
        self.connection.read().await.is_closed().await
    }

    pub async fn get_connection(&self) -> RwLockReadGuard<'_, Connection<T, Tr>> {
        self.connection.read().await
    }

    pub async fn get_mut_connection(&self) -> RwLockWriteGuard<'_, Connection<T, Tr>> {
        self.connection.write().await
    }
}

fn sub_clients(header: PacketHeader) -> PacketHeader {
    PacketHeader {
        packet_id: 0,
        ..header
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode_packets;
    use crate::test_helpers::login_success;
    use crate::tokio::memory::memory_connections;
    use bedrock_protocol::V2225;

    fn header(sender: u8, target: u8) -> PacketHeader {
        PacketHeader {
            packet_id: 0,
            sender_sub_client_id: sender,
            target_sub_client_id: target,
        }
    }

    #[::tokio::test]
    async fn sub_client_ids_survive_a_round_trip_through_the_shared_connection() {
        let (a, b) = memory_connections();
        let (mut sender, mut receiver) = (shard(a), shard(b));

        sender
            .write_with_header(&header(3, 1), login_success())
            .await
            .unwrap();
        sender.send().await.unwrap();
        receiver.recv().await.unwrap();

        let (got, _) = receiver.read_with_header().await.unwrap();
        assert_eq!(got.sender_sub_client_id, 3);
        assert_eq!(got.target_sub_client_id, 1);
    }

    #[::tokio::test]
    async fn packets_for_different_sub_clients_keep_their_own_ids_in_order() {
        let (a, b) = memory_connections();
        let (mut sender, mut receiver) = (shard(a), shard(b));

        for id in [0, 0, 2, 1, 1] {
            sender
                .write_with_header(&header(id, 0), login_success())
                .await
                .unwrap();
        }
        sender.send().await.unwrap();
        for _ in 0..3 {
            receiver.recv().await.unwrap();
        }

        let senders: Vec<u8> = receiver
            .read_all_with_headers()
            .await
            .into_iter()
            .map(|(header, _)| header.sender_sub_client_id)
            .collect();
        assert_eq!(senders, [0, 0, 2, 1, 1]);
    }

    #[::tokio::test]
    async fn plain_write_and_read_use_the_default_header() {
        let (a, b) = memory_connections();
        let (mut sender, mut receiver) = (shard(a), shard(b));

        sender.write(login_success()).await.unwrap();
        sender.send().await.unwrap();
        receiver.recv().await.unwrap();

        let (got, _) = receiver.read_with_header().await.unwrap();
        assert_eq!((got.sender_sub_client_id, got.target_sub_client_id), (0, 0));
        assert!(receiver.read().await.is_none());
    }

    #[::tokio::test]
    async fn the_default_header_still_produces_the_old_bytes() {
        let (a, mut b) = memory_connections();
        let mut sender = shard(a);

        sender.write(login_success()).await.unwrap();
        sender.write(login_success()).await.unwrap();
        sender.send().await.unwrap();

        let expected =
            encode_packets::<V2225>(&[login_success(), login_success()], None, None).unwrap();
        assert_eq!(b.recv_raw().await.unwrap(), expected);
    }

    #[::tokio::test]
    async fn a_sent_queue_is_not_sent_twice() {
        let (a, mut b) = memory_connections();
        let mut sender = shard(a);

        sender.write(login_success()).await.unwrap();
        sender.send().await.unwrap();
        sender.send().await.unwrap();

        b.recv_raw().await.unwrap();
        drop(sender);
        assert!(b.recv_raw().await.is_err());
    }
}
