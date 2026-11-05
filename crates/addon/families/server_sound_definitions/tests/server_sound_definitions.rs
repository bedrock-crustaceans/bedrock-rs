//! The server sound definitions family (`bedrock_addon::families::server_sound_definitions`,
//! the behaviour pack's `sounds/sound_definitions.json`): every row of the fixtures and every
//! rule of the file, at the one stable slice 1.26.60 of 1.26.60. No vanilla file exists, so
//! every input is written here from the schemas.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![allow(clippy::float_cmp, reason = "the readers give exact values")]

#[path = "server_sound_definitions/common.rs"]
mod common;

#[path = "server_sound_definitions/document.rs"]
mod document;

#[path = "server_sound_definitions/scalars.rs"]
mod scalars;
