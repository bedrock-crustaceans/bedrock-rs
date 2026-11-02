//! The full-stack smoke, the link pass over the first client install. `full.rs` says how the trees are loaded and run.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;

link_tests!(client_install_a, CLIENT_INSTALL_A);
