#![allow(clippy::pedantic, missing_docs)]
//! The loot table family (`loot_tables/`): every row of the fixtures, every rule
//! of the validation rules, the round trip, the stack accessor and the deviations.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "loot")]

#[path = "../../families/loot/tests/loot/common.rs"]
mod common;

#[path = "pack/common.rs"]
mod pack_common;

#[path = "loot/link.rs"]
mod link;

#[path = "loot/stack.rs"]
mod stack;
