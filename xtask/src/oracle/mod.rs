//! Differential-testing harness (Phase 7 of the level crate's roadmap):
//! dump one opened world through this crate to a normalized, sorted text
//! form, so it can be diffed against the same world dumped by an
//! independently-run oracle implementation.
//!
//! This module only ever *reads* a world through the public `bedrock_level`
//! API -- it does not reach into the crate's internals, so it exercises
//! exactly what an external caller sees.

pub mod diff;
pub mod dump;
pub mod run;
