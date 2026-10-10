use super::connection::Connection;
use bedrock_protocol_core::Packets;

pub struct Session<V: Packets, Identity = ()> {
    connection: Connection<V>,
    identity: Identity,
    pending: Vec<V>,
}

impl<V: Packets, Identity> Session<V, Identity> {
    pub(super) fn new(connection: Connection<V>, identity: Identity, pending: Vec<V>) -> Self {
        Self {
            connection,
            identity,
            pending,
        }
    }

    pub fn connection(&self) -> &Connection<V> {
        &self.connection
    }

    pub fn connection_mut(&mut self) -> &mut Connection<V> {
        &mut self.connection
    }

    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    pub fn pending(&self) -> &[V] {
        &self.pending
    }

    pub fn into_parts(self) -> (Connection<V>, Identity, Vec<V>) {
        (self.connection, self.identity, self.pending)
    }
}
