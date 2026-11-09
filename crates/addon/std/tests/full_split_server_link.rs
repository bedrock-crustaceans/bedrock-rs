//! The full-stack smoke, the link pass over the server half of the split client pack tree. `full.rs` says how the trees are loaded and run.

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;

link_tests!(split_server, SPLIT_SERVER);
