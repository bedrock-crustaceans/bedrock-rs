//! Armour trims (`item::trim`): the tables, the packet, the world gate and template-item filter,
//! and the deviations. The folder has no pack format, so there is no chain, round trip of a
//! document or stack accessor to test.

#[path = "trim/common.rs"]
mod common;

#[path = "trim/table.rs"]
mod table;

#[path = "trim/packet.rs"]
mod packet;

#[path = "trim/corpus.rs"]
mod corpus;
