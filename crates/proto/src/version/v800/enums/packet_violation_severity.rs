use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i32)]
#[enum_endianness(var)]
#[repr(i32)]
pub enum PacketViolationSeverity {
    Unknown = -1,
    Warning = 0,
    FinalWarning = 1,
    TerminatingConnection = 2,
}