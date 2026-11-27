//! The references of a block to other blocks, and the walk over a block's component
//! sections that every block rule of this group uses.

use bedrock_addon_core::diagnostic::{JsonPath, Severity};
use bedrock_addon_core::document::{ComponentMap, Document, Origin};

use crate::pack::link::prelude::*;
use crate::families::block::Block;
use crate::families::block::component::{BlockComponent, Geometry, PlacementFilter};

/// Every component section of the block of `document` with the path the block holds its
/// `components` object at: the block's own, then each permutation's. A rule names a member
/// below it by [`placed`], at the path the file has (the load takes out a permutation a gate
/// or a rule skips, and an upgrade step moves members inside a component).
pub(super) fn sections(
    document: &Document<Block>,
) -> impl Iterator<Item = (JsonPath, &ComponentMap<BlockComponent>)> + '_ {
    let block = document.value();
    let own = (at!["minecraft:block", "components"], &block.components);
    let permutations = block
        .permutations
        .iter()
        .enumerate()
        .map(move |(index, permutation)| {
            let held = at!["minecraft:block", "permutations", index, "components"];
            (held, &permutation.components)
        });
    std::iter::once(own).chain(permutations)
}

/// The path the member at `held` (a path of the loaded block, below a section of
/// [`sections`]) has in the file.
pub(super) fn placed(document: &Document<Block>, held: &JsonPath) -> JsonPath {
    document.diagnostics().place_of(held)
}

/// Every geometry a block names with the path the block holds it at: of each section the `minecraft:geometry`
/// component and the geometries of `minecraft:item_visual` and `minecraft:embedded_visual`.
#[cfg_attr(
    not(feature = "geometry"),
    allow(dead_code, reason = "the geometry rules are compiled with `geometry`")
)]
pub(super) fn geometries(document: &Document<Block>) -> Vec<(JsonPath, &Geometry)> {
    use crate::families::block::component::{EmbeddedVisual, ItemVisual};
    let mut all = Vec::new();
    for (section, components) in sections(document) {
        if let Some(geometry) = components.get::<Geometry>() {
            all.push((at![&section; "minecraft:geometry"], geometry));
        }
        if let Some(visual) = components.get::<ItemVisual>() {
            let path = at![&section; "minecraft:item_visual", "geometry"];
            all.push((path, &visual.geometry));
        }
        if let Some(visual) = components.get::<EmbeddedVisual>() {
            let path = at![&section; "minecraft:embedded_visual", "geometry"];
            all.push((path, &visual.geometry));
        }
    }
    all
}

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.behavior().blocks() {
        descriptors(cx, out, document);
    }
}

/// The block descriptors of a block name blocks: the `block_filter` of each
/// `minecraft:placement_filter` condition and the `blocks_to_corner_with` of the
/// `minecraft:placement_direction` trait.
fn descriptors(cx: &LinkCx<'_>, out: &mut Findings, document: &Document<Block>) {
    let block = document.value();
    let origin = document.origin();
    for (section, components) in sections(document) {
        let Some(filter) = components.get::<PlacementFilter>() else {
            continue;
        };
        for (condition_index, condition) in filter.conditions().iter().enumerate() {
            for (index, descriptor) in condition.block_filter().iter().enumerate() {
                let held = at![&section; "minecraft:placement_filter", "conditions", condition_index, "block_filter", index];
                let path = placed(document, &held);
                block_descriptor(cx, out, descriptor, member(origin, &path));
            }
        }
    }
    if let Some(direction) = &block.description.traits.placement_direction {
        for (index, descriptor) in direction.blocks_to_corner_with().iter().enumerate() {
            let path = at![
                "minecraft:block",
                "description",
                "traits",
                "minecraft:placement_direction",
                "blocks_to_corner_with",
                index
            ];
            block_descriptor(cx, out, descriptor, member(origin, &path));
        }
    }
}

/// A block descriptor of `origin` at `path`: a name nothing defines is the linked-asset check's
/// error, a state of a block of the stack the crate's own information.
fn member<'r>(origin: &'r Origin, path: &'r JsonPath) -> Member<'r> {
    Member {
        origin,
        path,
        // The linked-asset check's grading: no path of the resolver reads another.
        severity: Severity::Error,
        validator: Some(1),
        states: States::Lint,
    }
}
