use proto_derive::ProtoCodec;

#[derive(Debug, Clone, ProtoCodec)]
pub struct ResourcePacksStackPack {
    pub id: String,
    pub version: String,
    pub sub_pack_name: String,
}
