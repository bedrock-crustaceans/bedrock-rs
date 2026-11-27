//! The sound definitions family (`bedrock_addon::families::sound_definitions`, the resource
//! pack's `sounds/sound_definitions.json`): every row of the fixtures, the rules, the chain,
//! round trips and the document's loader.

#![allow(clippy::float_cmp, reason = "the readers give exact values")]
#![cfg(feature = "sound_definitions")]

#[path = "common/sound_packs.rs"]
mod common;
#[path = "sound_definitions/definitions.rs"]
mod definitions;
#[path = "sound_definitions/history.rs"]
mod history;
#[cfg(feature = "sound_file")]
#[path = "sound_definitions/sound_files.rs"]
mod sound_files;
#[path = "sound_definitions/streaming.rs"]
mod streaming;
