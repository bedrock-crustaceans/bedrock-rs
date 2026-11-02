#![allow(clippy::pedantic, missing_docs)]
//! The particle family (`bedrock_addon::families::particle`): its rows, the validation rules at
//! their boundaries, the chain, round trips, cost, and the stack's registry.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "particle")]

#[path = "particle/chain.rs"]
mod chain;
#[path = "particle/common.rs"]
mod common;
#[path = "particle/cost.rs"]
mod cost;
#[path = "particle/molang.rs"]
mod molang;
#[path = "particle/rows.rs"]
mod rows;
#[path = "particle/rules.rs"]
mod rules;
#[path = "particle/spellings.rs"]
mod spellings;
#[path = "particle/stack.rs"]
mod stack;
#[path = "particle/texts.rs"]
mod texts;
#[path = "particle/write.rs"]
mod write;
