//! `font/`: the font list and the emoticon list, the binary files of a stack, write-back as
//! authored and the cost of long lists.

#![cfg(feature = "font")]

#[path = "common/resource_stacks.rs"]
mod common;

#[path = "font/rows.rs"]
mod rows;

#[path = "font/link.rs"]
mod link;
