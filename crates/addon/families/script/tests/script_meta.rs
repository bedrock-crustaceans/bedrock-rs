//! Script-module metadata (`script::meta`): the strict models of the published
//! script metadata (`metadata/`), privileges, type-descriptor lowering, the module catalog and the generated core
//! tables. The `cargo xtask script-meta` generator, its vendoring checks and the comparison of
//! its output with the committed tables are tested where the generator is, in `xtask`
//! (`xtask/src/script_meta/tests/`).
//!
//! The metadata files are Mojang's and never committed: the model tests use
//! small hand-written documents of the same shapes, and the tests over the real tag read
//! `$BEDROCK_ADDON_CORPUS/bedrock-samples` and skip with a message when it is unset.

#[path = "script_meta/privilege.rs"]
mod privilege;

#[path = "script_meta/descriptor.rs"]
mod descriptor;

#[path = "script_meta/module.rs"]
mod module;

#[path = "script_meta/families.rs"]
mod families;

#[path = "script_meta/catalog.rs"]
mod catalog;
