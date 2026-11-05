//! The references of a block to voxel shapes.

use bedrock_addon_core::diagnostic::{Severity, VanillaText};
use bedrock_addon_core::document::Document;
use bedrock_addon_core::id::Id;

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use super::block::{placed, sections};
use crate::families::block::Block;
use crate::families::block::component::Geometry;

/// The voxel shapes the game has without a pack: every other `minecraft:` name is refused
/// when the block loads.
const BUILT_IN_SHAPES: [&str; 2] = ["minecraft:empty", "minecraft:unit_cube"];

/// What a block whose culling shape is missing culls with.
const DEFAULT_SHAPE: &str = "minecraft:empty";

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.behavior().blocks() {
        culling_shapes(cx, out, document);
    }
}

/// `minecraft:geometry.culling_shape` names a voxel shape of the stack or one of the
/// two the game has. A name nothing defines is vanilla's line at the content log's error
/// level, after which the block culls with the default shape: a kept error.
fn culling_shapes(cx: &LinkCx<'_>, out: &mut Findings, document: &Document<Block>) {
    for (section, components) in sections(document) {
        let Some(Geometry::Params(geometry)) = components.get::<Geometry>() else {
            continue;
        };
        let Some(shape) = &geometry.culling_shape else {
            continue;
        };
        let name = shape.as_str();
        if BUILT_IN_SHAPES.contains(&name) {
            continue;
        }
        if let Target::Missing(hint) = cx.voxel_shape(&Id::parse(name)) {
            // The game's wording, the default shape named.
            let text = VanillaText::Formatted {
                template: "Voxel shape with name \"{}\" doesn't exist in the registry. Defaulting to {} for identifier.",
                args: vec![name.to_owned(), DEFAULT_SHAPE.to_owned()],
            };
            let path = placed(document, &at![&section; "minecraft:geometry", "culling_shape"]);
            // The game logs this line at the error level and assigns the default shape: a kept
            // error.
            let place = At::new(document.origin(), path, Severity::Error).with_text(text);
            out.push(unresolved(place, kinds::VOXEL_SHAPE, name, hint).into_kept());
        }
    }
}
