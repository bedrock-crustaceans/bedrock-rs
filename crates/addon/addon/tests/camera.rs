//! The camera family (`cameras/`): the fixture rows, the validation rules, the chains, round
//! trips and the stack accessor.

#![cfg(feature = "camera")]

// The crate's one protocol for cost tests, and this binary's one lock.
use bedrock_addon_core::test_cost as cost;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "camera/common.rs"]
mod common;

#[path = "camera/presets.rs"]
mod presets;

#[path = "camera/envelope.rs"]
mod envelope;

#[path = "camera/stack.rs"]
mod stack;

#[path = "camera/splines.rs"]
mod splines;

#[path = "camera/entity.rs"]
mod entity;

#[path = "camera/chain.rs"]
mod chain;
