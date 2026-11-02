#![allow(clippy::pedantic, missing_docs)]
//! `splashes.json`: reading one copy, the stack's pool, write-back as authored and the cost of
//! a long list.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "splashes")]

#[path = "common/resource_stacks.rs"]
mod common;

#[path = "splashes/rows.rs"]
mod rows;

#[path = "splashes/as_authored.rs"]
mod as_authored;
