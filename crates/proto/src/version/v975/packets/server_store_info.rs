use bedrockrs_macros::{ProtoCodec, packet};
use crate::v975::types::ClientStoreEntryPointConfiguration;

#[packet(id = 346)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerStoreInfoPacket {
    pub client_store_entry_point_configuration: ClientStoreEntryPointConfiguration,
}