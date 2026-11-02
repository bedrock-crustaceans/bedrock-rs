//! The pack-settings family (`bedrock_addon::pack::settings`): every row of
//! `families/pack_settings` fixtures.md except the manifest rows (`P1-MANIFEST`'s, in
//! `tests/manifest/settings.rs`), the upgrade step, the chain, round trips, the runtime values and
//! the save algorithm.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "pack_settings/common.rs"]
mod common;
#[path = "pack_settings/document.rs"]
mod document;
#[path = "pack_settings/edges.rs"]
mod edges;
#[path = "pack_settings/members.rs"]
mod members;
#[cfg(feature = "molang")]
#[path = "pack_settings/molang.rs"]
mod molang;
#[path = "pack_settings/review.rs"]
mod review;
#[path = "pack_settings/round_trip.rs"]
mod round_trip;
#[path = "pack_settings/runtime.rs"]
mod runtime;
#[path = "pack_settings/upgrades.rs"]
mod upgrades;
#[path = "pack_settings/version.rs"]
mod version;
