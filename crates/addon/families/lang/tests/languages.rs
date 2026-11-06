//! The `languages` family (`texts/languages.json`, `texts/language_names.json`): the shapes of
//! both documents, the deviations, the merge rules and the locale fallback.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "languages/common.rs"]
mod common;

#[path = "languages/documents.rs"]
mod documents;
