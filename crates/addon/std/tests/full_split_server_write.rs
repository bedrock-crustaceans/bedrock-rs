//! The full-stack smoke, the writer over the server half of the split client pack tree. `full.rs` says how the trees are loaded and run.

#[macro_use]
mod common;
#[macro_use]
#[path = "full/support.rs"]
mod support;
#[path = "full/write.rs"]
mod write;

write_tests!(split_server, SPLIT_SERVER);
