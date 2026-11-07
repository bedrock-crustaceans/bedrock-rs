//! Texture sets (`bedrock_addon::families::texture_set`): every fixture row
//! through the stack accessor `rp.texture_sets()` and the resolver `rp.resolve_texture`, the
//! chain, the write side, and the walk that reads no image.
//!
//! Each fixture is a directory of packs, `pack_a` above `pack_b`, holding only the row's files.

#![cfg(all(
    feature = "texture_set",
    feature = "texture_file",
    feature = "texture_atlas",
    feature = "texture_list",
    feature = "flipbooks"
))]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

// The crate's one protocol for cost tests, and this binary's one lock.
use bedrock_addon_core::test_cost as cost;

#[path = "terrain_texture/common.rs"]
mod common;

#[path = "common/responder.rs"]
mod responder;

#[path = "texture_set/chain.rs"]
mod chain;

#[path = "texture_set/loader.rs"]
mod loader;

#[path = "texture_set/rows.rs"]
mod rows;

#[path = "texture_set/stack.rs"]
mod stack;

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::document::{Document, Rejected};
use bedrock_addon::families::texture_set::TextureSet;
use bedrock_addon::id::TexturePath;
use bedrock_addon::pack::TextureFile;
use bedrock_addon::pack::{PackStack, Resource};
use common::{Rendered, fixture_stack, rendered};

const FAMILY: &str = "texture_set";
/// The texture most fixtures are about, and its set.
const STONE: &str = "textures/blocks/stone";
const STONE_SET: &str = "textures/blocks/stone.texture_set.json";

fn stack(name: &str) -> PackStack<Resource> {
    fixture_stack(FAMILY, name)
}

/// What the client loads for `texture` in `stack`.
fn file<'a>(stack: &'a PackStack<Resource>, texture: &str) -> TextureFile<'a> {
    stack
        .resolve_texture(&TexturePath::parse(texture.to_owned()))
        .unwrap_or_else(|| panic!("no pack has {texture}"))
}

/// The texture set that loads for `texture`.
fn set<'a>(stack: &'a PackStack<Resource>, texture: &str) -> &'a Document<TextureSet> {
    let file = file(stack, texture);
    file.set.unwrap_or_else(|| {
        panic!(
            "{texture}: no set loaded: {:#?}",
            file.rejected.map(|r| rendered(r.diagnostics.iter()))
        )
    })
}

/// The rejection of the texture set that wins for `texture`: the client shows the missing
/// texture.
fn missing<'a>(stack: &'a PackStack<Resource>, texture: &str) -> &'a Rejected {
    let file = file(stack, texture);
    assert!(file.is_missing(), "{texture} loads");
    assert!(file.set.is_none() && file.image.is_none());
    file.rejected.unwrap()
}

/// The findings of the rejected set of `texture`.
fn rejected(stack: &PackStack<Resource>, texture: &str) -> Vec<Rendered> {
    rendered(missing(stack, texture).diagnostics.iter())
}

/// Every finding of the texture families of `stack`: atlases, flipbooks, lists, then sets.
fn group(stack: &PackStack<Resource>) -> impl Iterator<Item = &Diagnostic> + '_ {
    stack
        .texture_atlases()
        .diagnostics()
        .chain(stack.flipbooks().diagnostics())
        .chain(stack.texture_lists().diagnostics())
        .chain(stack.texture_sets().diagnostics())
}

/// The findings of the group that are not the capability hint (the fixtures declare no
/// capability; `stack::ts70` to `ts74` are about it).
fn findings(stack: &PackStack<Resource>) -> Vec<Rendered> {
    rendered(group(stack).filter(|finding| finding.code.id() != "core/capability_hint"))
}

/// An expected error.
fn error(code: &'static str, at: &str, text: &str) -> Rendered {
    common::finding(code, Severity::Error, at, text)
}

/// A content-log warning of the loader's (V2–V4).
fn warning(code: &'static str, at: &str, text: &str) -> Rendered {
    common::finding(code, Severity::Warning, at, text)
}

/// The line the client logs after a failed schema pass.
fn schema_failed() -> Rendered {
    error(
        "texture_set/set_schema_failed",
        "",
        "Failed to validate schema",
    )
}

/// The line the client logs after a failed post-validation (with its trailing space).
fn post_failed() -> Rendered {
    error(
        "texture_set/set_post_failed",
        "",
        "Failed to post-validate schema ",
    )
}
