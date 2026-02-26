use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
#[enum_repr(i8)]
#[repr(i8)]
pub enum CommandOutputType {
    None = 0,
    LastOutput = 1,
    Silent = 2,
    AllOutput = 3,
    DataSet(String) = 4,
}