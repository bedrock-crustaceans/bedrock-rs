//! The feature family (`features/`, one schema per feature type): the fixture rows, the
//! steps, the chains, round trips, Molang and the stack.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "feature")]

#[path = "../../families/feature/tests/feature/common.rs"]
mod common;

#[path = "feature/scatter.rs"]
mod scatter;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "feature/stack.rs"]
mod stack;

#[path = "feature/deviations.rs"]
mod deviations;

#[cfg(feature = "block")]
#[path = "feature/link.rs"]
mod link;
