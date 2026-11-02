#![allow(clippy::pedantic, missing_docs)]
//! The material family (`bedrock_addon::families::material`, families/materials): every row of
//! its fixtures.md and every rule of its validation.md, through the files read on their own
//! and through the stack accessor `rp.materials()`.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "material")]

#[path = "material/common.rs"]
mod common;

#[path = "material/forms.rs"]
mod forms;
#[path = "material/generated.rs"]
mod generated;
#[path = "material/link.rs"]
mod link;
#[path = "material/lists.rs"]
mod lists;
#[path = "material/requests.rs"]
mod requests;
#[path = "material/resolve.rs"]
mod resolve;
#[path = "common/responder.rs"]
mod responder;
#[path = "material/rules.rs"]
mod rules;
#[path = "material/scale.rs"]
mod scale;
#[path = "material/spans.rs"]
mod spans;
#[path = "material/stack.rs"]
mod stack;
#[path = "material/wrapper_version.rs"]
mod wrapper_version;
#[path = "material/write.rs"]
mod write;
