pub mod client;
pub mod handshake;
pub mod packets;
pub mod server;

use crate::compression::Compression;
use crate::encryption::Encryption;

pub use packets::{DisconnectInfo, LoginEvent, LoginPackets, LoginStatus, NetworkSettings};

pub enum LoginAction<V, Done = ()> {
    Send(Vec<V>),
    EnableCompression(Compression),
    EnableEncryption(Box<Encryption>),
    Complete(Done),
}
