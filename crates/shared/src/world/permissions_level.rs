use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Debug, Clone)]
// TODO: Give more specialized name, find out if u32 or i32
#[enum_repr(u32)]
#[enum_endianness(var)]
pub enum PermissionLevel {
    /// Any/Normal permission
    Default = 0,
    Operator = 1,
    Admin = 2,
    Host = 3,
    Owner = 4,
}
