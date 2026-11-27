//! The full-stack smoke, the load of the sample packs. `full.rs` says how the trees are loaded and run.

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;

load_tests!(samples, SAMPLES);
