//! The geometry a block names and the bones it shows or hides.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::id::GeometryId;

use super::super::{At, Findings, LinkCx, Target, kinds, unresolved};
use super::block::{geometries, placed};
use crate::families::block::component::Geometry;

/// The geometries the game has without a pack. A block below format 1.26.0 that names
/// `minecraft:geometry.full_block` gets `minecraft:geometry.full_block_v1`.
// The game refuses bone visibility on `minecraft:geometry.cross` and
// `minecraft:geometry.full_block` ("Cannot use bone visibility feature with
// minecraft:geometry.cross or minecraft:geometry.full_block"); the version-based
// initialisation maps the old full-block name to the `_v1` one.
const BUILT_IN_GEOMETRIES: [&str; 3] = [
    "minecraft:geometry.cross",
    "minecraft:geometry.full_block",
    "minecraft:geometry.full_block_v1",
];

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings) {
    for (_, document) in cx.behavior().blocks() {
        for (path, geometry) in geometries(document) {
            let name = geometry.identifier();
            if BUILT_IN_GEOMETRIES.contains(&name) {
                continue;
            }
            let id = GeometryId::parse(name);
            match cx.geometry(&id) {
                // Vanilla does not look the geometry up when the block loads and keeps the
                // name; a lint.
                Target::Missing(hint) if cx.lints() => {
                    let member = match geometry {
                        Geometry::Identifier(_) => placed(document, &path),
                        Geometry::Params(_) => placed(document, &at![&path; "identifier"]),
                    };
                    let place = At::new(document.origin(), member, Severity::Info);
                    out.push(unresolved(place, kinds::GEOMETRY, name, hint));
                }
                // Only of a geometry a pack defines are the bones known.
                Target::Defined(_) => {
                    let Geometry::Params(params) = geometry else {
                        continue;
                    };
                    for bone in params.bone_visibility.iter().flat_map(|bones| bones.keys()) {
                        // Bone names compare lower-cased, as the geometry stores them.
                        if cx.geometry_bone(&id, bone).is_none() {
                            let member = placed(document, &at![&path; "bone_visibility", bone]);
                            // The client logs "Geometry Component bone_visibilities bone %s
                            // is invalid, in block %s" at the error level and goes on with
                            // the expression: a kept error.
                            let place = At::new(document.origin(), member, Severity::Error);
                            out.push(unresolved(place, kinds::BONE, bone, None).into_kept());
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
