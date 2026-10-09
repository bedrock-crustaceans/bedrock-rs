use crate::ProtoVersion;
use bedrock_macros::{ProtoCodec, packet};

#[packet(id = 357)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SetPassengerOfBlockPacket<V: ProtoVersion> {
    pub passenger_actor_id: V::ActorUniqueID,
    pub passenger_of_block_data: Option<V::PassengerOfBlockArguments>,
}
