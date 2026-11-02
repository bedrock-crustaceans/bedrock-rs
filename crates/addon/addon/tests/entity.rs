#![allow(clippy::pedantic, missing_docs)]
//! The entity family (`bedrock_addon::families::entity`): document, version chain, steps and
//! validation.
#![cfg(feature = "entity")]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "entity/eat_block_link.rs"]
#[cfg(feature = "block")]
mod eat_block_link;

#[path = "entity/descriptors.rs"]
mod descriptors;

#[path = "common/responder.rs"]
mod responder;

#[path = "entity/stack.rs"]
mod stack;
