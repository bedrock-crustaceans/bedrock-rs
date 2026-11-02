#![allow(clippy::pedantic, missing_docs)]
//! The link pass, `StackSet::link`: the frame (`link/infra`) and one module per rule group,
//! each filled by its rule package. The helpers every group shares are in `link/common`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "link/common/mod.rs"]
mod common;
#[path = "pack/common.rs"]
mod pack_common;
#[allow(unused_imports, reason = "the link tests use the folder loader alone")]
#[path = "common/responder.rs"]
mod responder;

#[path = "link/infra/mod.rs"]
mod infra;

#[path = "link/content/mod.rs"]
mod content;
#[path = "link/entity/mod.rs"]
mod entity;
#[path = "link/resource/mod.rs"]
mod resource;
#[path = "link/script/mod.rs"]
mod script;
#[path = "link/visual/mod.rs"]
mod visual;
#[path = "link/world/mod.rs"]
mod world;

#[path = "link/integration.rs"]
mod integration;

// The names of the game's lists.
#[cfg(all(
    feature = "entity",
    feature = "recipe",
    feature = "feature",
    feature = "item",
    feature = "block"
))]
#[path = "link/game.rs"]
mod game;

// The game's linked-asset validation suites.
// The suites hold the game's validator suite of 76 texts, which spans these families. Of
// them `block`, `item`, `entity` and `camera` are what its packs and `linked_packs` are
// written over (the last reads accessors that exist only with `camera` and `item`); the
// others each fail a suite when dropped alone (found one at a time). Recipe, dialogue, ui,
// text, geometry, animation and skin are not needed.
#[cfg(all(
    feature = "block",
    feature = "item",
    feature = "entity",
    feature = "aim_assist",
    feature = "camera",
    feature = "biome",
    feature = "client_biome",
    feature = "sound_definitions",
    feature = "sounds_json",
    feature = "music",
    feature = "spawn_groups",
    feature = "spawn_rules",
    feature = "dimension",
    feature = "feature",
    feature = "feature_rules",
    feature = "jigsaw",
    feature = "trade",
    feature = "loot",
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "texture_set",
    feature = "render_controller",
    feature = "fog",
    feature = "material",
    feature = "deferred_settings",
    feature = "particle",
    feature = "structure"
))]
#[path = "link/linked_asset.rs"]
mod linked_asset;

// The view a pass has of a kind.
#[path = "link/view.rs"]
mod view;
