//! `credits/credits.json` and the files a resource pack carries in vain.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "credits")]

#[path = "common/resource_stacks.rs"]
mod common;

#[path = "credits/rows.rs"]
mod rows;

#[path = "credits/as_authored.rs"]
mod as_authored;
