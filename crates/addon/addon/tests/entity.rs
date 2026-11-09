//! The entity family (`bedrock_addon::families::entity`): document, version chain, steps and
//! validation.
#![cfg(feature = "entity")]

#[path = "entity/eat_block_link.rs"]
#[cfg(feature = "block")]
mod eat_block_link;

#[path = "common/responder.rs"]
mod responder;

#[path = "entity/stack.rs"]
mod stack;
