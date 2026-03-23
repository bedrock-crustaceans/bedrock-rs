use std::collections::HashMap;
use crate::version::versions::ProtoVersion;
use bedrockrs_macros::{packet, ProtoCodec};

#[packet(id = 80)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateTradePacket<V: ProtoVersion> {
    pub container_id: V::ContainerID,
    pub container_type: V::ContainerType,
    #[endianness(var)]
    pub size: i32,
    #[endianness(var)]
    pub trade_tier: i32,
    pub target_actor_id: V::ActorUniqueID,
    pub last_trading_player_id: V::ActorUniqueID,
    pub display_name: String,
    pub use_new_trade_ui: bool,
    pub using_economy_trade: bool,
    #[nbt]
    pub data_tags: HashMap<String, nbtx::Value>,
}
