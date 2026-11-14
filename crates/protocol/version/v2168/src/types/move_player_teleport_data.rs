use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct MovePlayerTeleportData {
    #[endianness(le)]
    pub teleportation_cause: i32,
    #[endianness(le)]
    pub source_actor_type: i32,
}
