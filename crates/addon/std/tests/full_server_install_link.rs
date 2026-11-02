//! The full-stack smoke, the link pass over the Bedrock Dedicated Server install. `full.rs` says how the trees are loaded and run.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;

link_tests!(server_install, SERVER_INSTALL);
