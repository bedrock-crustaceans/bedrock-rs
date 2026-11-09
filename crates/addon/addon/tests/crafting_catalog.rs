//! The crafting item catalogue (`item_catalog/crafting_item_catalog.json`,
//! `minecraft:crafting_items_catalog`): its rules, the chain, round trips and the stack accessor.

#![cfg(feature = "crafting_catalog")]

#[path = "common/responder.rs"]
mod responder;

#[path = "../../families/crafting_catalog/tests/crafting_catalog/common.rs"]
#[allow(
    clippy::disallowed_methods,
    reason = "calls facet-json directly: parses a fixture into a json Value to compare it; not yet moved to the door"
)]
mod common;

#[path = "crafting_catalog/link.rs"]
mod link;

#[path = "crafting_catalog/loader.rs"]
mod loader;

#[path = "crafting_catalog/merge.rs"]
mod merge;

#[path = "crafting_catalog/stack.rs"]
mod stack;

#[path = "crafting_catalog/stacks.rs"]
mod stacks;

#[path = "crafting_catalog/vanilla_1_26_50.rs"]
mod vanilla_1_26_50;

#[path = "crafting_catalog/located.rs"]
mod located;
