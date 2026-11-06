//! What the three biome families (`biome`, `client_biome`, `biomes_client`) still share outside
//! core and the shared crates: [`biome_json::child`], the digit-key path builder of their
//! findings. Its deletion is behaviour-bearing (a step of digits stops being an array index),
//! so it stays here, pinned by its tests, until that change lands; every other helper this
//! crate held now lives in `bedrock_addon_core` or a `bedrock_addon_shared_*` crate.
//! `bedrock_addon/tests/module_independence.rs` lets those three families, and no other,
//! depend on this crate, and only for `biome_json`.

pub mod biome_json;
