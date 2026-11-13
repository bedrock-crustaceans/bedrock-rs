//! `loading_messages.json`: reading the file, the highest pack's copy, write-back as authored,
//! the cost of a long list and the tips no language translates.

#![cfg(feature = "loading_messages")]

#[path = "common/resource_stacks.rs"]
mod common;

#[path = "loading_messages/rows.rs"]
mod rows;

#[cfg(all(feature = "lang", feature = "languages"))]
#[path = "loading_messages/link.rs"]
mod link;
