use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum IdentityDefinitionType<V: ProtoVersion> {
    Invalid = 0,
    Player {
        #[endianness(var)]
        player_unique_id: i64,
    } = 1,
    Entity {
        actor_id: V::ActorUniqueID,
    } = 2,
    FakePlayer {
        fake_player_name: String,
    } = 3,
}
