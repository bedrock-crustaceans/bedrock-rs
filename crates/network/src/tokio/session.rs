use super::connection::Connection;
use super::transport::TransportLayerConnection;
use bedrock_protocol_core::Packets;

pub struct Session<V: Packets, Identity = (), T = TransportLayerConnection> {
    connection: Connection<V, T>,
    identity: Identity,
    pending: Vec<V>,
}

impl<V: Packets, Identity, T> Session<V, Identity, T> {
    pub(super) fn new(connection: Connection<V, T>, identity: Identity, pending: Vec<V>) -> Self {
        Self {
            connection,
            identity,
            pending,
        }
    }

    pub fn connection(&self) -> &Connection<V, T> {
        &self.connection
    }

    pub fn connection_mut(&mut self) -> &mut Connection<V, T> {
        &mut self.connection
    }

    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    pub fn pending(&self) -> &[V] {
        &self.pending
    }

    pub fn into_parts(self) -> (Connection<V, T>, Identity, Vec<V>) {
        (self.connection, self.identity, self.pending)
    }
}
