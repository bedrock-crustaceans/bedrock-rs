//! The trade-table family (`bedrock_addon::families::trade`): its rows and rules, the upgrade
//! step, the chain, round trips and the stack accessor.

#![cfg(feature = "trade")]

#[path = "pack/common.rs"]
mod pack_common;

#[path = "trade/stack.rs"]
mod stack;

// The trade item pass runs with the item family, and an item name is looked up among the
// blocks too.
#[cfg(all(feature = "item", feature = "block"))]
#[path = "trade/link.rs"]
mod link;

#[path = "trade/located.rs"]
mod located;
