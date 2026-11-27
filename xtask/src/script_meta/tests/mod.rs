//! The `cargo xtask script-meta` step: its checks over a tiny hand-written tag, its output
//! against the committed tables (when the metadata checkout is present), and its token list against the committed
//! command tables.

/// The step, under the name its tests call it by.
use super as step;

mod commands;
mod generator;
mod regenerate;
