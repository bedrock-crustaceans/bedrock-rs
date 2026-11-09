//! The sounds.json family (`bedrock_addon::families::sounds_json`, the resource pack's
//! `sounds.json`): every row of the fixtures and the rules of the file, the merge of the
//! block `base` chains, and round trips.

#![allow(clippy::float_cmp, reason = "the readers give exact values")]
#![cfg(feature = "sounds_json")]

// The crate's one protocol for cost tests, and this binary's one lock.
use bedrock_addon_core::test_cost as cost;

#[path = "sounds_json/bindings.rs"]
mod bindings;
#[path = "common/sound_packs.rs"]
mod common;
#[path = "sounds_json/scale.rs"]
mod scale;
#[cfg(feature = "sound_definitions")]
#[path = "sounds_json/sound_definitions.rs"]
mod sound_definitions;
#[path = "sounds_json/stack.rs"]
mod stack;
#[path = "sounds_json/write_back.rs"]
mod write_back;
#[path = "sounds_json/written.rs"]
mod written;
#[path = "common/written_pins.rs"]
mod written_pins;
