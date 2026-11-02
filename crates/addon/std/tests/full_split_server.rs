//! The full-stack smoke, the load of the server half of the split client pack tree. `full.rs` says how the trees are loaded and run.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;

load_tests!(split_server, SPLIT_SERVER);

mod default_world {
    default_world_test!(SPLIT_SERVER, pinned);
}
