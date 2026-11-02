//! The link tests of scripting and pack settings: thirteen rules, S1 to S13.
//!
//! Ten of the rows are load rules: each has a test here that builds the reference on a stack,
//! requires the row's finding from the load (through `scripts()`, the stack's own report or the
//! reader the row names), exactly once, and requires `StackSet::link()` to add nothing. The
//! three link rows are raised outside `link()`, over inputs that are no stack content: S8 by
//! `ScriptConfig::unmatched_folders` over the stack's plugins, S10 by `ScriptConfig::parse`,
//! S11 by `pack::settings::unmatched_entries`.

#[cfg(feature = "script")]
mod scripting;
mod settings;
