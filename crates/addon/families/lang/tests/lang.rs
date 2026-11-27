//! The `lang` family (`texts/*.lang`): every row of the fixtures, the lint findings of a file,
//! the lookup formatting, the keyword helpers, the translations table and the Lines round trip.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "lang/common.rs"]
mod common;

#[path = "lang/read.rs"]
mod read;

#[path = "lang/lints.rs"]
mod lints;

#[path = "lang/keywords.rs"]
mod keywords;

#[path = "lang/table.rs"]
mod table;

#[path = "lang/round_trip.rs"]
mod round_trip;

#[path = "lang/format.rs"]
mod format;
