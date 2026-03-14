use crate::connection::Connection;
use crate::error::ConnectionError;
use bedrockrs_proto::compression::Compression;
use bedrockrs_proto::encryption::Encryption;
use bedrockrs_proto_core::Packets;
use tokio::select;
use tokio::sync::watch::Ref;
use tokio::sync::{mpsc, watch};
use tokio::time::Interval;

pub async fn shard<'t, T: Packets + Send + Sync + 't + 'static>(
    mut connection: Connection<T>,
    // TODO: Look into making flush_interval optional
    _flush_interval: Interval,
    packet_buffer_size: usize,
) -> (ConnectionShardSender<T>, ConnectionShardReceiver<T>) {
    let (packet_tx_task, packet_rx_shard) = mpsc::channel(packet_buffer_size);
    let (packet_tx_shard, mut packet_rx_task) = mpsc::channel(packet_buffer_size);
    let (close_tx, mut close_rx) = watch::channel(());
    let (flush_tx, mut flush_rx) = watch::channel(());
    let (compression_tx, mut compression_rx) = watch::channel(None);
    let (encryption_tx, mut encryption_rx) = watch::channel(None);

    let shards = (
        ConnectionShardSender {
            packet_sender: packet_tx_shard,
            close_sender: close_tx.clone(),
            flush_sender: flush_tx,
            compression_sender: compression_tx.clone(),
            compression_receiver: compression_rx.clone(),
            encryption_sender: encryption_tx.clone(),
            encryption_receiver: encryption_rx.clone(),
        },
        ConnectionShardReceiver {
            packet_receiver: packet_rx_shard,
            close_sender: close_tx.clone(),
            compression_receiver: compression_rx.clone(),
            encryption_receiver: encryption_rx.clone(),
        },
    );

    tokio::spawn(async move {
        let mut packets = Vec::with_capacity(packet_buffer_size);
        'select: loop {
            select! {
                _ = close_rx.changed() => {
                    break 'select;
                },
                res = flush_rx.changed() => {
                    if res.is_err() {
                        break 'select;
                    }

                    connection.send(packets.as_slice()).await.unwrap();
                    //println!("Sent {packets:#?}");
                    packets.clear();
                },
                res = connection.recv() => {
                    match res {
                        Ok(packets) => for packet in packets {
                            //println!("Received {packet:#?}");
                            packet_tx_task.send(Ok(packet)).await.unwrap();
                        },
                        Err(err) => packet_tx_task.send(Err(err)).await.unwrap(),
                    }
                },
                res = packet_rx_task.recv() => {
                    match res {
                        Some(packet) => packets.push(packet),
                        None => break 'select,
                    }
                },
                res = compression_rx.changed() => {
                    if res.is_err() {
                        break 'select;
                    }

                    connection.compression = compression_rx.borrow().clone();
                },
                res = encryption_rx.changed() => {
                    if res.is_err() {
                        break 'select;
                    }

                    connection.encryption = encryption_rx.borrow().clone();
                }
            }
        }
    });

    shards
}

#[derive(Debug, Clone)]
pub struct ConnectionShardSender<T: Packets + Send + Sync> {
    packet_sender: mpsc::Sender<T>,

    close_sender: watch::Sender<()>,

    flush_sender: watch::Sender<()>,

    compression_sender: watch::Sender<Option<Compression>>,
    compression_receiver: watch::Receiver<Option<Compression>>,

    encryption_sender: watch::Sender<Option<Encryption>>,
    encryption_receiver: watch::Receiver<Option<Encryption>>,
}

impl<T: Packets + Send + Sync> ConnectionShardSender<T> {
    pub async fn send(&mut self, packet: T) -> Result<(), ConnectionError> {
        self.packet_sender
            .send(packet)
            .await
            .map_err(|_| ConnectionError::ConnectionClosed)?;
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), ConnectionError> {
        self.flush_sender
            .send(())
            .map_err(|_| ConnectionError::ConnectionClosed)?;
        Ok(())
    }

    pub fn get_compression(&mut self) -> Result<Option<Compression>, ConnectionError> {
        Ok(self.compression_receiver.borrow().clone())
    }

    pub fn get_compression_ref(&mut self) -> Result<Ref<'_, Option<Compression>>, ConnectionError> {
        Ok(self.compression_receiver.borrow())
    }

    pub fn set_compression(
        &mut self,
        compression: Option<Compression>,
    ) -> Result<(), ConnectionError> {
        self.compression_sender
            .send(compression)
            .map_err(|_| ConnectionError::ConnectionClosed)?;
        Ok(())
    }

    pub fn get_encryption(&mut self) -> Result<Option<Encryption>, ConnectionError> {
        Ok(self.encryption_receiver.borrow().clone())
    }

    pub fn get_encryption_ref(&mut self) -> Result<Ref<'_, Option<Encryption>>, ConnectionError> {
        Ok(self.encryption_receiver.borrow())
    }

    pub fn set_encryption(
        &mut self,
        encryption: Option<Encryption>,
    ) -> Result<(), ConnectionError> {
        self.encryption_sender
            .send(encryption)
            .map_err(|_| ConnectionError::ConnectionClosed)?;
        Ok(())
    }

    pub async fn close(self) {
        // Already closed if Error occurs
        let _ = self.close_sender.send(());
    }
}

#[derive(Debug)]
pub struct ConnectionShardReceiver<T: Packets + Send + Sync> {
    pub packet_receiver: mpsc::Receiver<Result<T, ConnectionError>>,

    pub close_sender: watch::Sender<()>,

    pub compression_receiver: watch::Receiver<Option<Compression>>,
    pub encryption_receiver: watch::Receiver<Option<Encryption>>,
}

impl<T: Packets + Send + Sync> ConnectionShardReceiver<T> {
    pub async fn recv(&mut self) -> Result<T, ConnectionError> {
        self.packet_receiver
            .recv()
            .await
            .unwrap_or_else(|| Err(ConnectionError::ConnectionClosed))
    }

    pub fn get_compression(&mut self) -> Result<Option<Compression>, ConnectionError> {
        Ok(self.compression_receiver.borrow().clone())
    }

    pub fn get_compression_ref(&mut self) -> Result<Ref<'_, Option<Compression>>, ConnectionError> {
        Ok(self.compression_receiver.borrow())
    }

    pub fn get_encryption(&mut self) -> Result<Option<Encryption>, ConnectionError> {
        Ok(self.encryption_receiver.borrow().clone())
    }

    pub fn get_encryption_ref(&mut self) -> Result<Ref<'_, Option<Encryption>>, ConnectionError> {
        Ok(self.encryption_receiver.borrow())
    }

    pub async fn close(self) {
        // Already closed if Error occurs
        let _ = self.close_sender.send(());
    }
}
