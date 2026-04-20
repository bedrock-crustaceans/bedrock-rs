use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct ClientStoreEntryPointConfiguration {
    pub store_id: String,
    pub store_name: String
}