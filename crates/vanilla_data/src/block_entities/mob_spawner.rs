use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct MobSpawner {
    pub display_entity_height: f32,
    pub display_entity_scale: f32,
    pub display_entity_width: f32,
    pub spawn_count: i16,
    pub max_nearby_entities: i16,
    pub spawn_range: i16,
    pub max_spawn_delay: i16,
    pub entity_identifier: String,
    pub min_spawn_delay: i16,
    pub delay: i16,
    pub required_player_range: i16,
}
