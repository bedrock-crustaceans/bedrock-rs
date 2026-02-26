use super::super::enums::BossEventUpdateType;
use super::super::types::ActorUniqueID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 74)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct BossEventPacket {
    pub target_actor_id: ActorUniqueID,
    pub event_type: BossEventUpdateType,
}