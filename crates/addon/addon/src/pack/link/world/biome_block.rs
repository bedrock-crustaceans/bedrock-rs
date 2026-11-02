//! The blocks a biome's components name (`steep_material_adjustment.material`, the materials
//! of `surface_material_adjustments` and of the surface builders, the noise gradient's
//! `non_replaceable_blocks` and `noise_block_specifiers[].block`).

use super::super::{
    At, Findings, LinkCx, Member, States, Target, Validator, block_specifier, kinds, unresolved,
    validator_miss,
};
use crate::diagnostic::{JsonPath, JsonPathSegment, Severity, VanillaText};
use crate::document::Origin;
use crate::families::biome::block_references;
use crate::families::biome::component::BiomeComponent;

/// vanilla's text for a block that does not resolve: the member's label, then the name. It is
/// the game's biome-component block lookup that writes it.
const FAILED_TO_RESOLVE: &str = "'{}' block with name '{}', failed to resolve.";

/// The label vanilla's text gives the member at `path` (a path below its component): the
/// member's key, the list's key for an entry of a list, and for the block of a noise
/// specifier the label the game passes.
// The label for the block of a noise specifier is "noise_block_specifier -> block"; the
// others are the member names (`top_material`, `non_replaceable_blocks`); the labels of the
// builders' own materials are inferred from the shared helper.
fn label(path: &JsonPath) -> String {
    let mut keys = path.segments().filter_map(|segment| match segment {
        JsonPathSegment::Key(key) if !key.bytes().all(|byte| byte.is_ascii_digit()) => Some(key),
        _ => None,
    });
    let mut last = "";
    let mut in_specifiers = false;
    for key in &mut keys {
        in_specifiers |= key == "noise_block_specifiers";
        last = key;
    }
    if in_specifiers && last == "block" {
        "noise_block_specifier -> block".to_owned()
    } else {
        last.to_owned()
    }
}

/// Each block of the component `component`, which is at `at` in the file of `origin`. A
/// block neither the stack nor the game has is an `UnresolvedReference` error at the member
/// with vanilla's text; of a block of the stack, a state it does not have and a value the
/// state cannot take are errors too: the game does not register the biome.
pub(super) fn component_blocks(
    cx: &LinkCx<'_>,
    out: &mut Findings,
    origin: &Origin,
    at: &JsonPath,
    component: &BiomeComponent,
) {
    for (below, block) in block_references(component) {
        let mut path = at.clone();
        path.append(&below);
        if let Target::Missing(hint) = cx.block(&block.name) {
            let text = VanillaText::Formatted {
                template: FAILED_TO_RESOLVE,
                args: vec![label(&below), block.name.as_str().to_owned()],
            };
            // nothing of its own: a second finding with its line.
            let name = block.name.as_str();
            // The linked-asset check raises this finding to an error; it is written as the error
            // that comes out.
            let line = At::new(origin, path.clone(), Severity::Error);
            if block.object {
                path.push("name");
            }
            let at = At::new(origin, path, Severity::Error).with_text(text);
            out.push(unresolved(at, kinds::BLOCK, name, hint));
            out.push(validator_miss(
                line,
                Validator::Missing,
                1,
                kinds::BLOCK,
                name,
            ));
        } else {
            let member = Member {
                origin,
                path: &path,
                severity: Severity::Error,
                validator: None,
                states: States::Error,
            };
            block_specifier(cx, out, block, member);
        }
    }
}
