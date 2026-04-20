use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct PresenceConfiguration {
    pub experience_name: String,
    pub world_name: String
}