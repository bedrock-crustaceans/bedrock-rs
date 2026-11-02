#![allow(clippy::pedantic, missing_docs)]
//! The full-stack smoke, the writer over the second client install. `full.rs` says how the trees are loaded and run.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;
#[path = "full/write.rs"]
mod write;

write_tests!(client_install_b, CLIENT_INSTALL_B);
