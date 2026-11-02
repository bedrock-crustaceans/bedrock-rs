//! `font/`: the font list and the emoticon list, the binary files of a stack, write-back as
//! authored and the cost of long lists.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "font")]

#[path = "common/resource_stacks.rs"]
mod common;

#[path = "font/rows.rs"]
mod rows;

#[path = "font/as_authored.rs"]
mod as_authored;

#[path = "font/link.rs"]
mod link;
