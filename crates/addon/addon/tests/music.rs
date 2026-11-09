//! The music family (`bedrock_addon::families::music`, the resource pack's
//! `sounds/music_definitions.json`): every row of the fixtures and the rules of the file.

#![allow(clippy::float_cmp, reason = "the readers give exact values")]
#![cfg(feature = "music")]

#[path = "common/sound_packs.rs"]
mod common;
#[path = "music/definitions.rs"]
mod definitions;
#[cfg(feature = "sound_definitions")]
#[path = "music/sound_definitions.rs"]
mod sound_definitions;
#[path = "music/write_back.rs"]
mod write_back;
#[path = "music/written.rs"]
mod written;
#[path = "common/written_pins.rs"]
mod written_pins;
