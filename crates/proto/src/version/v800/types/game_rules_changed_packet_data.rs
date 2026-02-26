use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct GameRulesChangedPacketData {
    #[vec_repr(u32)]
    #[vec_endianness(var)]
    pub rules_list: Vec<GameRuleChanged>,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u32)]
#[enum_endianness(var)]
#[repr(u32)]
pub enum Type {
    Invalid = 0,
    Bool(bool) = 1,
    Int = 2,
    Float = 3,
}

#[derive(ProtoCodec, Clone, Debug)]
pub struct GameRuleChanged {
    pub rule_name: String,
    pub can_be_modified_by_player: bool,
    pub rule_type: Type,
}

