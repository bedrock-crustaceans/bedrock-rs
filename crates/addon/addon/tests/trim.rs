#![allow(clippy::pedantic, missing_docs)]
//! Armour trims (`item::trim`): the tables, the packet, the world gate and template-item filter,
//! and the deviations. The folder has no pack format, so there is no chain, round trip of a
//! document or stack accessor to test.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "item")]

#[path = "trim/common.rs"]
mod common;

#[path = "trim/table.rs"]
mod table;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "trim/gates.rs"]
mod gates;

#[path = "trim/packet.rs"]
mod packet;

#[path = "trim/corpus.rs"]
mod corpus;

#[cfg(feature = "loot")]
#[path = "trim/link.rs"]
mod link;
