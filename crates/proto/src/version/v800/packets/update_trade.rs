use super::super::enums::{ContainerID, ContainerType};
use super::super::types::ActorUniqueID;
use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 80)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct UpdateTradePacket {
    pub container_id: ContainerID,
    pub container_type: ContainerType,
    #[endianness(var)]
    pub size: i32,
    #[endianness(var)]
    pub trade_tier: i32,
    pub target_actor_id: ActorUniqueID,
    pub last_trading_player_id: ActorUniqueID,
    pub display_name: String,
    pub use_new_trade_ui: bool,
    pub using_economy_trade: bool,
    #[nbt]
    pub data_tags: nbtx::Value, // TODO: NBT Structure
}