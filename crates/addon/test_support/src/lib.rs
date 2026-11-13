//! The helpers the tests of the `bedrock_addon` crates share, one module each:
//!
//! - [`corpus`]: the line a test prints when the game's files it needs are absent.
//! - [`diagnostics`]: findings as lines and by code.
//! - [`document`]: the outcome of a document read, loaded or refused, and its round trip.
//! - [`fixture`]: the `tests/fixtures` folder a package's tests read, from its manifest directory.
//! - [`json`](mod@json): test JSON, read through the door, with comments or strictly.
//! - [`package`]: what a package's test files include from outside the package.
//! - [`value`]: a shared-type value read from JSON text, written back and compared.
//! - [`write_back`]: a document written back as the JSON value it was read from.
//! - [`written_pins`]: the pinned write-back of the sound documents.
//!
//! The crate is for tests alone: a crate names it in `[dev-dependencies]` and nowhere else.
//! It is never a normal dependency, so no library item of another crate can name it.
//! It depends on `bedrock_addon_core` and never on `bedrock_addon` or a family crate, so that a
//! family can use it without building the composing crate into its tests.
//! The unit tests in core's `src` cannot use it: there core is compiled a second time, and the
//! core this crate names is another crate whose `Diagnostics` is not theirs.

#![allow(
    clippy::missing_panics_doc,
    reason = "a helper panics on a broken test input: that is its job"
)]
#![allow(
    clippy::missing_errors_doc,
    reason = "a reader's error is a test's to unwrap or assert on, not a caller's to handle"
)]

pub mod corpus;
pub mod diagnostics;
pub mod document;
pub mod fixture;
pub mod json;
pub mod package;
pub mod value;
pub mod write_back;
pub mod written_pins;

pub use diagnostics::{clean, errors, line, lines, with_code};
pub use document::{loaded, rejected, round_trip};
pub use json::{json, strict};
pub use write_back::as_authored;
