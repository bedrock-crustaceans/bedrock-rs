//! `Strategy::Sequential` against `Strategy::Rayon` over whole trees of the game's packs: each
//! install is loaded twice, every collection of both stacks, and the two loads must be the same
//! load: the same `StackReport` (`Debug`, the findings in load order, and `Display`) and the
//! same `Debug` of every collection of both stacks, compared one by one with the comparison of
//! `crates/addon/addon/tests/determinism.rs` (`common/collections.rs`, which also says why a
//! collection's `Debug` is its iteration order).
//!
//! The server install and the two client installs are loaded as a world: their packs registered
//! as the app's own, the world's lists built by `common::world::base_game_world` (the base-game
//! layer in vanilla's order, every other pack of the install above it), with the install's app
//! data. The sample packs are one behaviour and one resource pack and no install: their world is
//! those two packs.
//!
//! Each load runs on a thread of `RECOMMENDED_JOB_STACK` (the sequential one parses on it),
//! one install at a time. The wall time of each load is printed, never asserted.
//!
//! This target loads the server install and the sample packs;
//! `full_strategies_client_install_a.rs` and `full_strategies_client_install_b.rs` load one
//! client install each (`common/strategies.rs` says why they are three targets).
//!
//! Run with `BEDROCK_ADDON_CORPUS=<folder> cargo nextest run -p bedrock_addon_std --test
//! full_strategies -- --show-output`; without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[path = "common/collections.rs"]
mod collections;
#[path = "common/strategies.rs"]
mod strategies;

use common::game_files;
use strategies::{World, assert_the_strategies_agree, install};

#[test]
fn server_install_loads_the_same_sequentially_and_on_rayon() {
    let server = skip!(game_files::SERVER_INSTALL);
    assert_the_strategies_agree("server_install", &install(&server));
}

#[test]
fn samples_load_the_same_sequentially_and_on_rayon() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    assert_the_strategies_agree("samples", &World::SamplePacks(samples));
}
