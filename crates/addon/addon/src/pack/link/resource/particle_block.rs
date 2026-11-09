//! The blocks a particle expires in, or outside of, are blocks.
//!
//! The game reads each name of the two block lists as a block reference; a name that is no
//! block matches nothing, so the list is one entry shorter than written. Told here as an
//! error at the name, as written. A name without a namespace is a block of `minecraft`.
// `particle_expire_if_not_in_blocks` is read by the same reader as
// `particle_expire_if_in_blocks`, so both lists are held to the rule.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::document::Document;
use bedrock_addon_core::id::BlockId;

use super::particle::component;
use crate::families::particle::ParticleEffect;
use crate::families::particle::component::{ParticleExpireIfInBlocks, ParticleExpireIfNotInBlocks};
use crate::pack::link::prelude::*;

/// Each name of `minecraft:particle_expire_if_in_blocks` and of
/// `minecraft:particle_expire_if_not_in_blocks` is a block a behaviour pack defines or one of
/// the game's. One that is none is an `UnresolvedReference` error at the element, with the
/// near miss the block lookup gives. The index is the file's: the reader of an effect below
/// 1.20.80 leaves out an element that is no string, and the document keeps where each name
/// it holds is in the file.
pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.resource().particles() {
        let components = &document.value().components;
        if let Some(list) = components.get::<ParticleExpireIfInBlocks>() {
            let name = "minecraft:particle_expire_if_in_blocks";
            blocks(cx, out, document, name, &list.0);
        }
        if let Some(list) = components.get::<ParticleExpireIfNotInBlocks>() {
            let name = "minecraft:particle_expire_if_not_in_blocks";
            blocks(cx, out, document, name, &list.0);
        }
    }
}

/// The names of the block list that is the value of the component `name` of `document`.
fn blocks(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    document: &Document<ParticleEffect>,
    name: &str,
    list: &[String],
) {
    for (index, block) in list.iter().enumerate() {
        if let Target::Missing(hint) = cx.block(&BlockId::parse(block.as_str())) {
            let mut path = component(name);
            path.push_index(index);
            let path = document.diagnostics().place_of(&path);
            // The linked-asset check raises the finding to an error; it is written as the error
            // that comes out.
            let at = At::new(document.origin(), path, Severity::Error).validator(
                Validator::Missing,
                1,
                block,
            );
            out.push(unresolved(at, kinds::BLOCK, block, hint));
        }
    }
}
