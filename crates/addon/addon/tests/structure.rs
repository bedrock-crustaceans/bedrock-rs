//! The structure family (`structures/**/*.mcstructure`, little-endian NBT): every row of
//! The fixtures, the exact round trip, the lookups by identifier, the stack
//! and the deviations.

#![cfg(feature = "structure")]

#[path = "common/responder.rs"]
mod responder;

#[path = "../../families/structure/tests/structure/common.rs"]
mod common;

#[path = "structure/loader.rs"]
mod loader;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "structure/stack.rs"]
mod stack;

#[cfg(feature = "block")]
#[path = "structure/link_rules.rs"]
mod link_rules;

#[path = "structure/edges.rs"]
mod edges;
