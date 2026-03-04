use crate::version::proto_version::ProtoVersion;
use bedrockrs_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct FullContainerName<V: ProtoVersion> {
    container: V::ContainerEnumName,
    #[endianness(le)]
    dynamic_id: i32,
}
