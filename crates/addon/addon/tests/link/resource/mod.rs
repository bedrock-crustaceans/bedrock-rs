//! The link tests of the resource data (20 rows R27 to R57, B8, I6).

// The client block data.
#[cfg(all(feature = "block", feature = "block_culling"))]
mod block_culling;
#[cfg(all(feature = "block", feature = "blocks_json"))]
mod blocks_json;

// The atlases and textures.
#[cfg(all(
    feature = "flipbooks",
    feature = "texture_atlas",
    feature = "texture_file"
))]
mod flipbook;
#[cfg(all(feature = "item", feature = "texture_atlas"))]
mod item_texture;
#[cfg(all(feature = "item", not(feature = "texture_atlas")))]
mod item_texture_off;
#[cfg(all(
    feature = "texture_atlas",
    feature = "texture_list",
    feature = "texture_set"
))]
mod texture_audit;
#[cfg(all(feature = "texture_set", feature = "texture_file"))]
mod texture_set;

// The client biomes and the render settings.
#[cfg(all(feature = "biome", feature = "client_biome", feature = "biomes_client"))]
mod biomes_client;
#[cfg(feature = "deferred_settings")]
mod deferred_settings;

// The effects and the texts.
#[cfg(feature = "sounds_json")]
mod audit_sound;
#[cfg(feature = "font")]
mod font;
#[cfg(feature = "loading_messages")]
mod loading_messages;
#[cfg(feature = "particle")]
mod particle;
#[cfg(all(feature = "server_sound_definitions", feature = "sound_definitions"))]
mod sound;
