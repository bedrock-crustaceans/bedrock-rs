use bedrockrs_macros::{gamepacket, ProtoCodec};

#[gamepacket(id = 64)]
#[derive(ProtoCodec, Clone, Debug)]
pub struct SimpleEventPacket {
    pub simple_event_type: Subtype,
}

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(u16)]
#[enum_endianness(le)]
#[repr(u16)]
pub enum Subtype {
    UninitializedSubtype = 0,
    EnableCommands = 1,
    DisableCommands = 2,
    UnlockWorldTemplateSettings = 3,
}
