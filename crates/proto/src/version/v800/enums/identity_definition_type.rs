use super::super::types::ActorUniqueID;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum IdentityDefinitionType {
    Invalid = 0,
    Player {
        #[endianness(var)]
        player_unique_id: i64,
    } = 1,
    Entity {
        actor_id: ActorUniqueID,
    } = 2,
    FakePlayer {
        fake_player_name: String,
    } = 3,
}
