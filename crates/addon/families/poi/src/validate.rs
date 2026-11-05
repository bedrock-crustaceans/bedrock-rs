//! The rules that judge one loaded document. The checks over several documents run when the
//! group combines (`resolve.rs`).

use super::{PoiTag, PoiType};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, VanillaText,
};
use bedrock_addon_core::id::{BlockId, PoiTagId};

/// `base` followed by the member `key`.
fn child(base: &JsonPath, key: &str) -> JsonPath {
    let mut path = base.clone();
    path.push(key);
    path
}

/// The rules of a POI type, `at` its payload.
pub(crate) fn poi_type(value: &mut PoiType, at: &JsonPath, d: &mut Diagnostics) {
    let blocks = child(at, "blocks");
    // A type without blocks never matches a placed block. Vanilla registers it silently, so
    // the crate's own note is an information.
    if value.blocks.is_empty() {
        d.push(
            Diagnostic::info(DiagnosticCode::PoiNoBlocks)
                .at(blocks.clone())
                .phase(Phase::Validate),
        );
    }
    // Vanilla collects the blocks into a set, so a repeat changes nothing; the crate keeps the
    // first of each and reports every repeat where it was written.
    let mut kept: Vec<BlockId> = Vec::with_capacity(value.blocks.len());
    for (i, block) in std::mem::take(&mut value.blocks).into_iter().enumerate() {
        if kept.contains(&block) {
            d.push(
                Diagnostic::info(DiagnosticCode::NoEffect { text: None })
                    .at(element(&blocks, i))
                    .phase(Phase::Validate),
            );
        } else {
            kept.push(block);
        }
    }
    value.blocks = kept;
}

/// `base` followed by the element `index`.
fn element(base: &JsonPath, index: usize) -> JsonPath {
    let mut path = base.clone();
    path.push_index(index);
    path
}

/// What a tag of a cycle is told, with the tag's name: the phrasing of the game's own unit
/// test of tag cycles (vanilla has no text for it outside a test).
pub(crate) const PART_OF_A_CYCLE: &str = "{} was a part of a cycle";

/// The `ReferenceCycle` code of `tag`.
pub(crate) fn part_of_a_cycle(tag: &PoiTagId) -> DiagnosticCode {
    DiagnosticCode::ReferenceCycle {
        text: Some(VanillaText::Formatted {
            template: PART_OF_A_CYCLE,
            args: vec![tag.as_str().to_owned()],
        }),
    }
}

/// The rules of a POI tag, `at` its payload.
pub(crate) fn poi_tag(value: &mut PoiTag, at: &JsonPath, d: &mut Diagnostics) {
    // A tag that names itself is a cycle of one; that edge is dropped, the member the author
    // wrote stays.
    let own = value.description.identifier.clone();
    for (key, list) in [
        ("children", &mut value.children),
        ("parents", &mut value.parents),
    ] {
        let Some(list) = list else {
            continue;
        };
        let at = child(at, key);
        let mut i = 0;
        list.retain(|tag| {
            let itself = *tag == own;
            if itself {
                // The game logs no line for this, so the crate's own note is an information.
                d.push(
                    Diagnostic::info(part_of_a_cycle(&own))
                        .at(element(&at, i))
                        .phase(Phase::Validate),
                );
            }
            i += 1;
            !itself
        });
    }
}
