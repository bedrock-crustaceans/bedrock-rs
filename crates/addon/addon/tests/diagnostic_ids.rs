//! The registry of diagnostic definition ids: every table of definitions the toolkit raises
//! from, and the rule that no id appears twice across them. Every `codes.rs` has its `ALL`
//! table below, behind its family's feature (`codes_tables.rs` scans the sources for it).

use bedrock_addon::diagnostic::CodeMeta;
use std::collections::BTreeMap;

/// Every definition table, named for the crate that owns it.
#[allow(
    clippy::too_many_lines,
    reason = "one line per family's table; with every family on it is long"
)]
fn tables() -> Vec<(&'static str, &'static [&'static CodeMeta])> {
    let tables: Vec<(&'static str, &'static [&'static CodeMeta])> = vec![
        ("core", bedrock_addon::diagnostic::codes::ALL),
        ("pack_settings", bedrock_addon::pack::settings::codes::ALL),
        #[cfg(feature = "music")]
        ("music", bedrock_addon::families::music::codes::ALL),
        #[cfg(feature = "ui_texture_info")]
        (
            "ui_texture_info",
            bedrock_addon::families::ui_texture_info::codes::ALL,
        ),
        #[cfg(feature = "loading_messages")]
        (
            "loading_messages",
            bedrock_addon::families::loading_messages::codes::ALL,
        ),
        #[cfg(feature = "credits")]
        ("credits", bedrock_addon::families::credits::codes::ALL),
        #[cfg(feature = "dimension")]
        ("dimension", bedrock_addon::families::dimension::codes::ALL),
        #[cfg(feature = "server_sound_definitions")]
        (
            "server_sound_definitions",
            bedrock_addon::families::server_sound_definitions::codes::ALL,
        ),
        #[cfg(feature = "flipbooks")]
        ("flipbooks", bedrock_addon::families::flipbooks::codes::ALL),
        #[cfg(feature = "texture_list")]
        (
            "texture_list",
            bedrock_addon::families::texture_list::codes::ALL,
        ),
        #[cfg(feature = "spawn_rules")]
        (
            "spawn_rules",
            bedrock_addon::families::spawn_rules::codes::ALL,
        ),
        #[cfg(feature = "spawn_groups")]
        (
            "spawn_groups",
            bedrock_addon::families::spawn_groups::codes::ALL,
        ),
        #[cfg(any(feature = "spawn_rules", feature = "spawn_groups"))]
        ("spawn", bedrock_addon::shared::spawn::codes::ALL),
        #[cfg(feature = "feature_rules")]
        (
            "feature_rules",
            bedrock_addon::families::feature_rules::codes::ALL,
        ),
        #[cfg(feature = "aim_assist")]
        (
            "aim_assist",
            bedrock_addon::families::aim_assist::codes::ALL,
        ),
        #[cfg(feature = "function")]
        ("function", bedrock_addon::families::function::codes::ALL),
        #[cfg(feature = "voxel_shape")]
        (
            "voxel_shape",
            bedrock_addon::families::voxel_shape::codes::ALL,
        ),
        #[cfg(feature = "skin")]
        ("skin", bedrock_addon::families::skin::codes::ALL),
        #[cfg(feature = "attachable")]
        (
            "attachable",
            bedrock_addon::families::attachable::codes::ALL,
        ),
        #[cfg(feature = "atomic_client_entity")]
        (
            "atomic_client_entity",
            bedrock_addon::families::atomic_client_entity::codes::ALL,
        ),
        ("visual", bedrock_addon::visual::codes::ALL),
        ("shapes", bedrock_addon::shared::codes::ALL),
        #[cfg(feature = "jigsaw")]
        ("jigsaw", bedrock_addon::families::jigsaw::codes::ALL),
        #[cfg(feature = "structure")]
        ("structure", bedrock_addon::families::structure::codes::ALL),
        #[cfg(feature = "crafting_catalog")]
        (
            "crafting_catalog",
            bedrock_addon::families::crafting_catalog::codes::ALL,
        ),
        #[cfg(feature = "blocks_json")]
        (
            "blocks_json",
            bedrock_addon::families::blocks_json::codes::ALL,
        ),
        #[cfg(feature = "feature")]
        ("feature", bedrock_addon::families::feature::codes::ALL),
        #[cfg(feature = "biome")]
        ("biome", bedrock_addon::families::biome::codes::ALL),
        #[cfg(feature = "client_biome")]
        (
            "client_biome",
            bedrock_addon::families::client_biome::codes::ALL,
        ),
        #[cfg(feature = "block")]
        ("block", bedrock_addon::families::block::codes::ALL),
        #[cfg(feature = "fog")]
        ("fog", bedrock_addon::families::fog::codes::ALL),
        #[cfg(feature = "dialogue")]
        ("dialogue", bedrock_addon::families::dialogue::codes::ALL),
        #[cfg(feature = "block_culling")]
        (
            "block_culling",
            bedrock_addon::families::block_culling::codes::ALL,
        ),
        #[cfg(feature = "texture_atlas")]
        (
            "texture_atlas",
            bedrock_addon::families::texture_atlas::codes::ALL,
        ),
        #[cfg(feature = "texture_set")]
        (
            "texture_set",
            bedrock_addon::families::texture_set::codes::ALL,
        ),
        #[cfg(feature = "camera")]
        ("camera", bedrock_addon::families::camera::codes::ALL),
        #[cfg(feature = "font")]
        ("font", bedrock_addon::families::font::codes::ALL),
        #[cfg(feature = "sounds_json")]
        (
            "sounds_json",
            bedrock_addon::families::sounds_json::codes::ALL,
        ),
        #[cfg(feature = "animation_controller")]
        (
            "animation_controller",
            bedrock_addon::families::animation_controller::codes::ALL,
        ),
        #[cfg(feature = "render_controller")]
        (
            "render_controller",
            bedrock_addon::families::render_controller::codes::ALL,
        ),
        #[cfg(feature = "poi")]
        ("poi", bedrock_addon::families::poi::codes::ALL),
        #[cfg(feature = "behavior_tree")]
        (
            "behavior_tree",
            bedrock_addon::families::behavior_tree::codes::ALL,
        ),
        #[cfg(feature = "sound_definitions")]
        (
            "sound_definitions",
            bedrock_addon::families::sound_definitions::codes::ALL,
        ),
        #[cfg(feature = "lang")]
        ("lang", bedrock_addon::families::lang::codes::ALL),
        #[cfg(feature = "script")]
        ("script", bedrock_addon::families::script::codes::ALL),
        #[cfg(feature = "server_config")]
        (
            "server_config",
            bedrock_addon::families::server_config::codes::ALL,
        ),
        #[cfg(feature = "persona")]
        ("persona", bedrock_addon::families::persona::codes::ALL),
        #[cfg(feature = "material")]
        ("material", bedrock_addon::families::material::codes::ALL),
        #[cfg(feature = "deferred_settings")]
        (
            "deferred_settings",
            bedrock_addon::families::deferred_settings::codes::ALL,
        ),
        #[cfg(feature = "trade")]
        ("trade", bedrock_addon::families::trade::codes::ALL),
        #[cfg(feature = "loot")]
        ("loot", bedrock_addon::families::loot::codes::ALL),
        #[cfg(feature = "ddui")]
        ("ddui", bedrock_addon::families::ddui::codes::ALL),
        #[cfg(feature = "item")]
        ("item", bedrock_addon::families::item::codes::ALL),
        #[cfg(feature = "geometry")]
        ("geometry", bedrock_addon::families::geometry::codes::ALL),
        #[cfg(feature = "particle")]
        ("particle", bedrock_addon::families::particle::codes::ALL),
        #[cfg(feature = "recipe")]
        ("recipe", bedrock_addon::families::recipe::codes::ALL),
        #[cfg(feature = "json_ui")]
        ("json_ui", bedrock_addon::families::json_ui::codes::ALL),
        #[cfg(feature = "entity")]
        ("entity", bedrock_addon::families::entity::codes::ALL),
    ];
    tables
}

#[test]
fn every_table_lists_at_least_one_definition() {
    for (owner, table) in tables() {
        assert!(!table.is_empty(), "{owner} lists no definitions");
    }
}

#[test]
fn ids_are_unique_across_every_table() {
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    let mut clashes = Vec::new();
    for (owner, table) in tables() {
        for meta in table {
            if let Some(first) = seen.insert(meta.id(), owner) {
                clashes.push(format!("{} ({first}, {owner})", meta.id()));
            }
        }
    }
    assert!(clashes.is_empty(), "ids defined twice: {clashes:?}");
}

#[test]
fn every_id_names_its_owner_first() {
    let mut wrong = Vec::new();
    for (owner, table) in tables() {
        for meta in table {
            let prefix = meta.id().split('/').next().unwrap_or("");
            if prefix != owner {
                wrong.push(format!("{} in the {owner} table", meta.id()));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "ids outside their owner's prefix: {wrong:?}"
    );
}

/// `Diagnostic::syntax_message` claims a reader's message for every `Kind::Syntax` definition
/// but those of `SYNTAX_WITHOUT_MESSAGE`. Each listed id is a definition of that kind, and the
/// definitions left are the 51 that are a reader's message: a new `Kind::Syntax` definition
/// turns this red until it is counted here or listed there.
#[cfg(feature = "full")]
#[test]
fn the_syntax_kind_without_a_message_is_the_listed_ten() {
    use bedrock_addon::diagnostic::{Kind, SYNTAX_WITHOUT_MESSAGE};
    let syntax: Vec<&str> = tables()
        .into_iter()
        .flat_map(|(_, table)| table.iter())
        .filter(|meta| meta.kind() == Kind::Syntax)
        .map(|meta| meta.id())
        .collect();
    let unlisted: Vec<&&str> = SYNTAX_WITHOUT_MESSAGE
        .iter()
        .filter(|id| !syntax.contains(id))
        .collect();
    assert!(
        unlisted.is_empty(),
        "listed, but no `Kind::Syntax` definition: {unlisted:?}"
    );
    assert_eq!(SYNTAX_WITHOUT_MESSAGE.len(), 10);
    let messages: Vec<&str> = syntax
        .into_iter()
        .filter(|id| !SYNTAX_WITHOUT_MESSAGE.contains(id))
        .collect();
    assert_eq!(messages.len(), 51, "{messages:#?}");
}
