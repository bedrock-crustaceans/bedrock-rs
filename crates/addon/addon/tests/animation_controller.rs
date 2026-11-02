#![allow(clippy::pedantic, missing_docs)]
//! The animation controller family (`bedrock_addon::families::animation_controller`) on the
//! resource stack: the rows and the chain.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(all(feature = "animation", feature = "animation_controller"))]

#[path = "animation/common.rs"]
mod common;
#[path = "common/responder.rs"]
mod responder;

#[path = "animation_controller/chain.rs"]
mod chain;
#[path = "animation_controller/rows.rs"]
mod rows;
