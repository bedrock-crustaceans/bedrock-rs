use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
#[enum_repr(i32)]
#[enum_endianness(var)]
pub enum Gamemode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    SurvivalSpectator = 3,
    CreativeSpectator = 4,
    Default = 5,
    Spectator = 6,
}
