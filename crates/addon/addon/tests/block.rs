//! The block family (`bedrock_addon::families::block`): the fixture rows, the upgrade steps, the chain,
//! round trips, Molang, network NBT and the thin slice.

#![cfg(feature = "block")]

#[path = "../../families/block/tests/block/common.rs"]
mod common;

#[path = "block/stacks.rs"]
mod stacks;

#[path = "block/thin_slice.rs"]
mod thin_slice;
