//! The full-stack smoke, the load of the first client install. `full.rs` says how the trees are loaded and run.

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;

load_tests!(client_install_a, CLIENT_INSTALL_A);

mod default_world {
    default_world_test!(CLIENT_INSTALL_A, pinned);
}
