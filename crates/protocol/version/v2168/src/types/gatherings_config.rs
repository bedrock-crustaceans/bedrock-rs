use bedrock_macros::ProtoCodec;
use uuid::Uuid;

#[derive(ProtoCodec, Clone, Debug)]
pub struct GatheringsConfig {
    pub experience_id: Uuid,
    pub experience_name: String,
    pub world_id: Option<Uuid>,
    pub world_name: Option<String>,
    pub creator_id: String,
    pub target_id: Option<Uuid>,
    pub scenario_id: Option<String>,
    pub server_id: Option<String>,
}
