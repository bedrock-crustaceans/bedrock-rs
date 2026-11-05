//! The `blocks.json` family (`ClientBlock`): its rules, the version gates, round trips and the
//! stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "blocks_json")]

#[path = "blocks_json/common.rs"]
mod common;
#[path = "blocks_json/fields.rs"]
mod fields;
#[path = "blocks_json/keys.rs"]
mod keys;
// The link rules of `blocks.json` read the block family or `sounds.json`.
#[cfg(any(feature = "block", feature = "sounds_json"))]
#[path = "blocks_json/link.rs"]
mod link;
#[path = "blocks_json/located.rs"]
mod located;
#[path = "pack/common.rs"]
mod pack_common;
#[path = "blocks_json/round_trip.rs"]
mod round_trip;
#[path = "blocks_json/stack.rs"]
mod stack;
#[path = "blocks_json/vanilla_packs.rs"]
mod vanilla_packs;
#[path = "blocks_json/version.rs"]
mod version;
