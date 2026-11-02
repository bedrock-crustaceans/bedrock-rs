//! `Strategy::Sequential` against `Strategy::Rayon` over the first client install: one of the two
//! client-install targets beside `full_strategies.rs`, which says what is compared and how. Each
//! client install has a target of its own (`full_strategies_client_install_a`,
//! `full_strategies_client_install_b`) because loading both in one test binary needs more than
//! the 600 s of CPU a test binary may use.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<folder> cargo nextest run -p bedrock_addon_std --test
//! full_strategies_client_install_a -- --show-output`; without the variable the test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[path = "common/collections.rs"]
mod collections;
#[path = "common/strategies.rs"]
mod strategies;

use common::game_files;
use strategies::{assert_the_strategies_agree, install};

#[test]
fn client_install_a_loads_the_same_sequentially_and_on_rayon() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    assert_the_strategies_agree("client_install_a", &install(&client));
}
