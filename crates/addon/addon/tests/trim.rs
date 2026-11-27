//! Armour trims (`item::trim`): the tables, the packet, the world gate and template-item filter,
//! and the deviations. The folder has no pack format, so there is no chain, round trip of a
//! document or stack accessor to test.

#![cfg(feature = "item")]

#[path = "../../families/item/tests/trim/common.rs"]
mod common;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "trim/gates.rs"]
mod gates;

#[cfg(feature = "loot")]
#[path = "trim/link.rs"]
mod link;
