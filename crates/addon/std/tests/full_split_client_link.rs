#![allow(clippy::pedantic, missing_docs)]
//! The full-stack smoke, the link pass over the client half of the split client pack tree. `full.rs` says how the trees are loaded and run.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;

link_tests!(split_client, SPLIT_CLIENT);
