use crate::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct FullContainerName<V: ProtoVersion> {
    pub container: V::ContainerEnumName,
    #[endianness(le)]
    pub dynamic_id: i32,
}
