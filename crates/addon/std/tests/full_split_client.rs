#![allow(clippy::pedantic, missing_docs)]
//! The full-stack smoke, the load of the client half of the split client pack tree. `full.rs` says how the trees are loaded and run.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;

load_tests!(split_client, SPLIT_CLIENT);

mod default_world {
    default_world_test!(SPLIT_CLIENT, pinned);
}
