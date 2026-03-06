use crate::compression::Compression;
use crate::connection::Connection;
use crate::encryption::Encryption;
use crate::error::ConnectionError;
use bedrockrs_proto_core::Packets;
use tokio::select;
use tokio::sync::watch::Ref;
use tokio::sync::{mpsc, watch};
use tokio::time::Interval;

pub async fn shard<'t, T: Packets + Send + Sync + 't>(
    mut connection: Connection,
    // TODO: Look into making flush_interval optional
    _flush_interval: Interval,
    gamepacket_buffer_size: usize,
) -> (ConnectionShardSender<T>, ConnectionShardReceiver<T>)
where
    T: Send + Sync + 'static,
{
    let (gamepacket_tx_task, gamepacket_rx_shard) = mpsc::channel(gamepacket_buffer_size);
    let (gamepacket_tx_shard, mut gamepacket_rx_task) = mpsc::channel(gamepacket_buffer_size);
    let (close_tx, mut close_rx) = watch::channel(());
    let (flush_tx, mut flush_rx) = watch::channel(());
    let (compression_tx, mut compression_rx) = watch::channel(None);
    let (encryption_tx, mut encryption_rx) = watch::channel(None);

    let shards = (
        ConnectionShardSender {
            gamepacket_sender: gamepacket_tx_shard,
            close_sender: close_tx.clone(),
            flush_sender: flush_tx,
            compression_sender: compression_tx.clone(),
            compression_receiver: compression_rx.clone(),
            encryption_sender: encryption_tx.clone(),
            encryption_receiver: encryption_rx.clone(),
        },
        ConnectionShardReceiver {
            gamepacket_receiver: gamepacket_rx_shard,
            close_sender: close_tx.clone(),
            compression_receiver: compression_rx.clone(),
            encryption_receiver: encryption_rx.clone(),
        },
    );

    tokio::spawn(async move {
        let mut gamepackets = Vec::with_capacity(gamepacket_buffer_size);
        'select: loop {
            select! {
                _ = close_rx.changed() => {
                    break 'select;
                },
                res = flush_rx.changed() => {
                    if res.is_err() {
                        break 'select;
                    }

                    connection.send::<T>(gamepackets.as_slice()).await.unwrap();
                    //println!("Sent {gamepackets:#?}");
                    gamepackets.clear();
                },
                res = connection.recv::<T>() => {
                    match res {
                        Ok(gamepackets) => for gamepacket in gamepackets {
                            //println!("Received {gamepacket:#?}");
                            gamepacket_tx_task.send(Ok(gamepacket)).await.unwrap();
                        },
                        Err(err) => gamepacket_tx_task.send(Err(err)).await.unwrap(),
                    }
                },
                res = gamepacket_rx_task.recv() => {
                    match res {
                        Some(gamepacket) => gamepackets.push(gamepacket),
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
    gamepacket_sender: mpsc::Sender<T>,

    close_sender: watch::Sender<()>,

    flush_sender: watch::Sender<()>,

    compression_sender: watch::Sender<Option<Compression>>,
    compression_receiver: watch::Receiver<Option<Compression>>,

    encryption_sender: watch::Sender<Option<Encryption>>,
    encryption_receiver: watch::Receiver<Option<Encryption>>,
}

impl<T: Packets + Send + Sync> ConnectionShardSender<T> {
    pub async fn send(&mut self, packet: T) -> Result<(), ConnectionError> {
        self.gamepacket_sender
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
    pub(crate) gamepacket_receiver: mpsc::Receiver<Result<T, ConnectionError>>,

    pub(crate) close_sender: watch::Sender<()>,

    pub(crate) compression_receiver: watch::Receiver<Option<Compression>>,
    pub(crate) encryption_receiver: watch::Receiver<Option<Encryption>>,
}

impl<T: Packets + Send + Sync> ConnectionShardReceiver<T> {
    pub async fn recv(&mut self) -> Result<T, ConnectionError> {
        self.gamepacket_receiver
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
