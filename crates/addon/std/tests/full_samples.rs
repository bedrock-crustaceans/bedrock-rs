//! The full-stack smoke, the load of the sample packs. `full.rs` says how the trees are loaded and run.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;

load_tests!(samples, SAMPLES);
