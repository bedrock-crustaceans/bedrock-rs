pub mod aabb_volumes;
pub mod actor;
pub mod biome;
pub mod biome_ids_table;
pub mod bits;
pub mod block_entity;
pub mod border_blocks;
pub mod checksums;
pub mod chunk;
pub mod chunk_version;
pub mod color;
pub mod db;
pub mod error;
pub mod finalized_state;
pub mod flat_records;
pub mod greedy;
pub mod hardcoded_spawn_areas;
pub mod height_map;
pub mod height_range;
pub mod iter;
pub mod key;
pub mod lazy;
pub mod legacy_terrain;
pub mod level;
pub mod level_chunk;
pub mod local_player;
pub mod map_data;
pub mod metadata_dictionary;
pub mod nbt_variants;
pub mod player;
pub mod portals;
pub mod position_track;
pub mod prng;
pub mod provider;
pub mod raw_keys;
pub mod settings;
pub mod subchunk;
pub mod ticks;
pub mod traits;
pub mod types;
pub mod upgrade;
pub mod version;

mod private {
    pub trait Sealed {}
}

pub trait UnpackingMethod: private::Sealed {
    const IS_LAZY: bool;
}

pub enum Greedy {}

impl private::Sealed for Greedy {}

impl UnpackingMethod for Greedy {
    const IS_LAZY: bool = false;
}

pub enum Lazy {}

impl private::Sealed for Lazy {}

impl UnpackingMethod for Lazy {
    const IS_LAZY: bool = true;
}

pub mod prelude {}
