//! The link rules of the resource data.
//!
//! Four parts, run in the order of the rows: the client block data (`blocks.json`, block
//! culling), the atlases and their textures, the client biomes with the render settings, and
//! the effects and texts (particles, sounds, loading messages, fonts).

// The client block data.
#[cfg(all(feature = "block", feature = "block_culling"))]
mod block_culling;
#[cfg(all(feature = "block", feature = "block_culling", feature = "geometry"))]
mod block_culling_geometry;
#[cfg(all(feature = "blocks_json", feature = "sounds_json"))]
mod block_sound;
#[cfg(all(
    feature = "blocks_json",
    any(feature = "block", feature = "sounds_json")
))]
mod blocks_json;
#[cfg(all(feature = "block", feature = "blocks_json"))]
mod blocks_json_block;
#[cfg(all(feature = "block", feature = "blocks_json", feature = "texture_atlas"))]
mod blocks_json_texture;
#[cfg(any(
    all(
        feature = "block",
        any(feature = "blocks_json", feature = "block_culling")
    ),
    all(feature = "blocks_json", feature = "sounds_json")
))]
mod told;

// The atlases and textures.
#[cfg(all(
    feature = "flipbooks",
    feature = "texture_atlas",
    feature = "texture_file"
))]
mod flipbooks_texture;
#[cfg(all(feature = "item", feature = "texture_atlas"))]
mod item_texture;
#[cfg(all(feature = "texture_set", feature = "texture_file"))]
mod texture_set_texture_file;

// The client biomes and the render settings.
#[cfg(all(
    feature = "biome",
    feature = "client_biome",
    feature = "biomes_client",
    feature = "fog",
    feature = "deferred_settings"
))]
mod biome_render;
#[cfg(all(feature = "biome", feature = "client_biome", feature = "sounds_json"))]
mod biome_sound;
#[cfg(all(feature = "biome", feature = "client_biome"))]
mod client_biome_biome;
#[cfg(all(feature = "deferred_settings", feature = "block"))]
mod render_block;
#[cfg(all(
    feature = "deferred_settings",
    feature = "texture_file",
    feature = "texture_set"
))]
mod render_texture;

// The effects and the texts.
#[cfg(all(feature = "music", feature = "sound_definitions"))]
mod music_sound_definitions;
#[cfg(feature = "particle")]
mod particle;
#[cfg(all(feature = "particle", feature = "block"))]
mod particle_block;
#[cfg(all(feature = "particle", feature = "material"))]
mod particle_render;
#[cfg(all(feature = "particle", feature = "sounds_json"))]
mod particle_sound;
#[cfg(all(
    feature = "particle",
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "texture_set"
))]
mod particle_texture;
#[cfg(all(feature = "server_sound_definitions", feature = "sound_definitions"))]
mod server_sound_definitions_sound_definitions;
#[cfg(all(feature = "sound_definitions", feature = "sound_file"))]
mod sound_definitions_sound_file;
#[cfg(all(feature = "sounds_json", feature = "sound_definitions"))]
mod sounds_json_sound_definitions;
#[cfg(feature = "font")]
mod ui;
#[cfg(all(feature = "loading_messages", feature = "lang", feature = "languages"))]
mod ui_text;

use super::{Findings, LinkCx};

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    blocks(cx, out);
    textures(cx, out);
    biomes(cx, out);
    effects(cx, out);
}

/// The rules of the client block data.
fn blocks(cx: &LinkCx<'_>, out: &mut Findings) {
    let _ = (cx, &*out);
    #[cfg(all(feature = "block", feature = "blocks_json"))]
    blocks_json_block::keys(cx, out);
    #[cfg(all(feature = "block", feature = "blocks_json", feature = "texture_atlas"))]
    blocks_json_texture::run(cx, out);
    #[cfg(all(feature = "blocks_json", feature = "sounds_json"))]
    block_sound::run(cx, out);
    #[cfg(all(feature = "block", feature = "blocks_json"))]
    blocks_json_block::exponents(cx, out);
    #[cfg(all(feature = "block", feature = "blocks_json"))]
    blocks_json_block::tessellation(cx, out);
    #[cfg(all(feature = "block", feature = "block_culling"))]
    block_culling::culling_names(cx, out);
    #[cfg(all(feature = "block", feature = "block_culling", feature = "geometry"))]
    block_culling_geometry::run(cx, out);
    #[cfg(all(feature = "block", feature = "block_culling"))]
    block_culling::culling_layers(cx, out);
}

/// The rules of the atlases and textures.
fn textures(cx: &LinkCx<'_>, out: &mut Findings) {
    let _ = (cx, &*out);
    #[cfg(all(feature = "item", feature = "texture_atlas"))]
    item_texture::run(cx, out);
    #[cfg(all(
        feature = "flipbooks",
        feature = "texture_atlas",
        feature = "texture_file"
    ))]
    flipbooks_texture::run(cx, out);
    #[cfg(all(feature = "texture_set", feature = "texture_file"))]
    texture_set_texture_file::run(cx, out);
}

/// The rules of the client biomes and the render settings.
fn biomes(cx: &LinkCx<'_>, out: &mut Findings) {
    let _ = (cx, &*out);
    #[cfg(all(feature = "biome", feature = "client_biome"))]
    client_biome_biome::run(cx, out);
    #[cfg(all(
        feature = "biome",
        feature = "client_biome",
        feature = "biomes_client",
        feature = "fog",
        feature = "deferred_settings"
    ))]
    biome_render::run(cx, out);
    #[cfg(all(feature = "biome", feature = "client_biome", feature = "sounds_json"))]
    biome_sound::run(cx, out);
    #[cfg(all(
        feature = "biome",
        feature = "client_biome",
        feature = "biomes_client",
        feature = "fog",
        feature = "deferred_settings"
    ))]
    biome_render::legacy_fogs(cx, out);
    #[cfg(all(feature = "biome", feature = "client_biome", feature = "biomes_client"))]
    client_biome_biome::water(cx, out);
    #[cfg(all(
        feature = "biome",
        feature = "client_biome",
        feature = "biomes_client",
        feature = "fog",
        feature = "deferred_settings"
    ))]
    biome_render::fog_overrides(cx, out);
    #[cfg(all(
        feature = "deferred_settings",
        feature = "texture_file",
        feature = "texture_set"
    ))]
    render_texture::run(cx, out);
    #[cfg(all(feature = "deferred_settings", feature = "block"))]
    render_block::run(cx, out);
}

/// The rules of the effects and the texts.
fn effects(cx: &LinkCx<'_>, out: &mut Findings) {
    let _ = (cx, &*out);
    #[cfg(all(feature = "particle", feature = "material"))]
    particle_render::run(cx, out);
    #[cfg(all(
        feature = "particle",
        feature = "texture_atlas",
        feature = "texture_file",
        feature = "texture_set"
    ))]
    particle_texture::run(cx, out);
    #[cfg(feature = "particle")]
    particle::own_events(cx, out);
    #[cfg(all(feature = "particle", feature = "block"))]
    particle_block::run(cx, out);
    #[cfg(feature = "particle")]
    particle::effects(cx, out);
    #[cfg(all(feature = "particle", feature = "sounds_json"))]
    particle_sound::run(cx, out);
    #[cfg(all(feature = "server_sound_definitions", feature = "sound_definitions"))]
    server_sound_definitions_sound_definitions::run(cx, out);
    #[cfg(all(feature = "sounds_json", feature = "sound_definitions"))]
    sounds_json_sound_definitions::run(cx, out);
    #[cfg(all(feature = "music", feature = "sound_definitions"))]
    music_sound_definitions::run(cx, out);
    #[cfg(all(feature = "sound_definitions", feature = "sound_file"))]
    sound_definitions_sound_file::run(cx, out);
    #[cfg(all(feature = "loading_messages", feature = "lang", feature = "languages"))]
    ui_text::run(cx, out);
    #[cfg(feature = "font")]
    ui::run(cx, out);
}
