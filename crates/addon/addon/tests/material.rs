//! The material family (`bedrock_addon::families::material`, families/materials): every row of
//! its fixtures.md and every rule of its validation.md, through the files read on their own
//! and through the stack accessor `rp.materials()`.

#![cfg(feature = "material")]

#[path = "../../families/material/tests/material/common.rs"]
mod common;

#[path = "material/forms.rs"]
mod forms;
#[path = "material/generated.rs"]
mod generated;
#[path = "material/link.rs"]
mod link;
#[path = "material/requests.rs"]
mod requests;
#[path = "material/resolve.rs"]
mod resolve;
#[path = "common/responder.rs"]
mod responder;
#[path = "material/scale.rs"]
mod scale;
#[path = "material/spans.rs"]
mod spans;
#[path = "material/stack.rs"]
mod stack;
#[path = "material/stacks.rs"]
mod stacks;
