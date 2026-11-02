//! The trade-table family (`bedrock_addon::families::trade`): its rows and rules, the upgrade
//! step, the chain, round trips and the stack accessor.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "trade")]

#[path = "pack/common.rs"]
mod pack_common;

#[path = "trade/stack.rs"]
mod stack;

#[path = "trade/link.rs"]
mod link;
