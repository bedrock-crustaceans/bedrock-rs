use crate::connection::Connection;
use crate::error::ConnectionError;
use bedrockrs_proto_core::Packets;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn shard<T: Packets>(connection: Connection<T>) -> ConnectionShared<T> {
    ConnectionShared::<T> {
        connection: Arc::new(RwLock::new(connection)),
        queue_send: Arc::new(RwLock::new(Vec::new())),
        queue_recv: Arc::new(RwLock::new(VecDeque::new())),
    }
}

#[derive(Clone)]
pub struct ConnectionShared<T: Packets> {
    connection: Arc<RwLock<Connection<T>>>,
    queue_send: Arc<RwLock<Vec<T>>>,
    queue_recv: Arc<RwLock<VecDeque<T>>>,
}

impl<T: Packets> ConnectionShared<T> {
    pub async fn write(&mut self, packet: T) -> Result<(), ConnectionError> {
        let mut queue_send = self.queue_send.write().await;

        queue_send.push(packet);

        Ok(())
    }

    pub async fn read(&mut self) -> Option<T> {
        let mut queue_recv = self.queue_recv.write().await;

        queue_recv.pop_front()
    }

    pub async fn read_all(&mut self) -> Vec<T> {
        let mut queue_recv = self.queue_recv.write().await;
        queue_recv.make_contiguous();
        queue_recv.drain(..).collect::<Vec<_>>()
    }

    pub async fn send(&mut self) -> Result<(), ConnectionError> {
        let mut packets = self.queue_send.write().await;
        let mut conn = self.connection.write().await;

        conn.send(packets.as_slice()).await?;

        packets.clear();

        Ok(())
    }

    pub async fn recv(&mut self) -> Result<(), ConnectionError> {
        let mut conn = self.connection.write().await;

        let packets = conn.recv().await?;

        if !packets.is_empty() {
            let mut queue_recv = self.queue_recv.write().await;

            for packet in packets {
                queue_recv.push_back(packet);
            }
        }

        Ok(())
    }

    pub async fn close(&self) {
        self.connection.read().await.close().await;
    }

    pub async fn is_closed(&self) -> bool {
        self.connection.read().await.is_closed().await
    }
}
