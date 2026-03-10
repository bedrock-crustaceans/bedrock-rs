use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct GameRulesChangedPacketData {
    pub rules_list: Vec<GameRuleChanged>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum Type {
    Bool(bool) = 1,
    Int(#[endianness(le)] i32) = 2,
    Float(#[endianness(le)] f32) = 3,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct GameRuleChanged {
    pub rule_name: String,
    pub can_be_modified_by_player: bool,
    pub rule_type: Type,
}
