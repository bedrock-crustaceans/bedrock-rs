//! The link rules of the resource data.
//!
//! Four parts, run in the order of the rows: the client block data (`blocks.json`, block
//! culling), the atlases and their textures, the client biomes with the render settings, and
//! the effects and texts (particles, sounds, loading messages, fonts).

rules! {
    // The client block data.
    mod block_culling needs ["block", "block_culling"];
    mod block_culling_geometry needs ["block", "block_culling", "geometry"];
    mod blocks_json_sound needs ["blocks_json", "sounds_json"];
    mod blocks_json needs ["blocks_json", any["block", "sounds_json"]];
    mod blocks_json_block needs ["block", "blocks_json"];
    mod blocks_json_texture needs ["block", "blocks_json", "texture_atlas"];
    mod told needs any[
        ["block", "blocks_json"],
        ["block", "block_culling"],
        ["blocks_json", "sounds_json"],
    ];

    // The atlases and textures.
    mod flipbooks_texture needs ["flipbooks", "texture_atlas", "texture_file"];
    mod item_texture needs ["item", "texture_atlas"];
    mod texture_set_texture_file needs ["texture_set", "texture_file"];

    // The client biomes and the render settings.
    mod biome_render needs ["biome", "client_biome", "biomes_client", "fog", "deferred_settings"];
    mod biome_sound needs ["biome", "client_biome", "sounds_json"];
    mod client_biome_biome needs ["biome", "client_biome"];
    mod render_block needs ["deferred_settings", "block"];
    mod render_texture needs ["deferred_settings", "texture_file", "texture_set"];

    // The effects and the texts.
    mod music_sound_definitions needs ["music", "sound_definitions"];
    mod particle needs ["particle"];
    mod particle_block needs ["particle", "block"];
    mod particle_render needs ["particle", "material"];
    mod particle_sound needs ["particle", "sounds_json"];
    mod particle_texture needs ["particle", "texture_atlas", "texture_file", "texture_set"];
    mod server_sound_definitions_sound_definitions needs [
        "server_sound_definitions",
        "sound_definitions",
    ];
    mod sound_definitions_sound_file needs ["sound_definitions", "sound_file"];
    mod sounds_json_sound_definitions needs ["sounds_json", "sound_definitions"];
    mod ui needs ["font"];
    mod ui_text needs ["loading_messages", "lang", "languages"];

    /// The rules of the client block data.
    run blocks() =
        blocks_json_block::keys,
        blocks_json_texture::run,
        blocks_json_sound::run,
        blocks_json_block::exponents,
        blocks_json_block::tessellation,
        block_culling::culling_names,
        block_culling_geometry::run,
        block_culling::culling_layers;

    /// The rules of the atlases and textures.
    run textures() =
        item_texture::run,
        flipbooks_texture::run,
        texture_set_texture_file::run;

    /// The rules of the client biomes and the render settings.
    run biomes() =
        client_biome_biome::run,
        biome_render::run,
        biome_sound::run,
        biome_render::legacy_fogs,
        client_biome_biome::water + ["biomes_client"],
        biome_render::fog_overrides,
        render_texture::run,
        render_block::run;

    /// The rules of the effects and the texts.
    run effects() =
        particle_render::run,
        particle_texture::run,
        particle::own_events,
        particle_block::run,
        particle::effects,
        particle_sound::run,
        server_sound_definitions_sound_definitions::run,
        sounds_json_sound_definitions::run,
        music_sound_definitions::run,
        sound_definitions_sound_file::run,
        ui_text::run,
        ui::run;
}

pub(super) fn run(cx: &super::LinkCx<'_>, out: &mut super::Findings) {
    blocks(cx, out);
    textures(cx, out);
    biomes(cx, out);
    effects(cx, out);
}
