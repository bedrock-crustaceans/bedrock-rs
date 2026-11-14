use bedrock_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct GatheringsConfig {
    pub experience_id: Uuid,
    pub experience_name: String,
    pub world_id: Uuid,
    pub world_name: String,
    pub creator_id: String,
    pub target_id: Uuid,
    pub scenario_id: String,
    pub server_id: String,
}
