//! The material family (`bedrock_addon::families::material`, families/materials): every row of
//! its fixtures.md and every rule of its validation.md, through the files read on their own.

#[path = "material/common.rs"]
mod common;

#[path = "material/lists.rs"]
mod lists;

#[path = "material/located.rs"]
mod located;

#[path = "material/rules.rs"]
mod rules;

#[path = "material/wrapper_version.rs"]
mod wrapper_version;

#[path = "material/write.rs"]
mod write;
