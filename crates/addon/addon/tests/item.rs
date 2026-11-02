#![allow(clippy::pedantic, missing_docs)]
//! The item family (`items/`, `minecraft:item`): the chain, its steps, round trips, Molang and
//! the network NBT.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "item")]

#[path = "../../families/item/tests/item/common.rs"]
mod common;

#[path = "item/validation.rs"]
mod validation;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "item/stack.rs"]
mod stack;
