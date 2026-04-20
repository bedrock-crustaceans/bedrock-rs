use bedrockrs_macros::{ProtoCodec, packet};

#[packet(id = 346)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct ServerStoreInfoPacket {
    pub client_store_entry_point_configuration: Option<ClientStoreEntryPointConfiguration>,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientStoreEntryPointConfiguration {
    pub store_id: String,
    pub store_name: String
}