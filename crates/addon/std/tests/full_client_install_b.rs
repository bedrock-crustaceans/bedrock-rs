#![allow(clippy::pedantic, missing_docs)]
//! The full-stack smoke, the load of the second client install. `full.rs` says how the trees are loaded and run.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;

load_tests!(client_install_b, CLIENT_INSTALL_B);

mod default_world {
    default_world_test!(CLIENT_INSTALL_B, pinned);
}
