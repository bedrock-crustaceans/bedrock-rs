use crate::version::versions::ProtoVersion;
use bedrock_macros::ProtoCodec;

#[derive(ProtoCodec, Clone, Debug)]
pub struct FullContainerName<V: ProtoVersion> {
    container: V::ContainerEnumName,
    #[endianness(le)]
    dynamic_id: Option<i32>,
}
