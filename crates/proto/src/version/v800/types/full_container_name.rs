use bedrockrs_macros::ProtoCodec;
use super::super::enums::ContainerEnumName;

#[derive(ProtoCodec, Clone, Debug)]
pub struct FullContainerName {
    container: ContainerEnumName,
    #[endianness(le)]
    dynamic_id: Option<i32>,
}