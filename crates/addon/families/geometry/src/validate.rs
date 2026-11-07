//! What happens to an upgraded file after its schema: the identifier rules of
//! `_buildInheritanceTree`, and the checks and coercions of `Geometry::parse`
//!.

use super::codes;
use super::read::{V1_12_0, V1_19_30};
use super::walk::has_geometry_name;
use super::{Bone, CubeUv, Geometry, GeometryFile, ItemDisplayTransform, PolyMesh, Polys};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, Diagnostics, JsonPath, Phase, Severity,
};
use bedrock_addon_core::version::{FormatVersion, known};

/// From this file version the `embedded` transform is read.
const EMBEDDED_FROM: FormatVersion = known::V1_21_110;
/// The range `Geometry::parse` clamps `visible_bounds_width` and `visible_bounds_height` to.
const BOUNDS: (f32, f32) = (0.1, 50.0);

/// Where a file is, as far as the identifier rules care.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Place {
    /// The pack is a base-game pack (`isFromBaseGamePack`).
    pub(crate) base_game: bool,
    /// The file is below `models/blocks/`.
    pub(crate) blocks: bool,
}

impl Place {
    /// Whether the geometry must use the `minecraft` namespace: a base-game block geometry.
    pub(crate) fn requires_minecraft_namespace(self) -> bool {
        self.base_game && self.blocks
    }
}

/// A content-log error of the game's after which the geometry loads: a kept error, an
/// error-level finding after which the game keeps the document.
fn kept(diagnostics: &mut Diagnostics, code: Code, at: &JsonPath) {
    diagnostics.push_kept_error(Diagnostic::of(code).at(at.clone()).phase(Phase::Validate));
}

/// The identifier rule of a file of 1.19.30 or later; the error's definition when `identifier`
/// breaks it.
fn identifier_error(identifier: &str, place: Place) -> Option<(&'static CodeDef<1>, &'static str)> {
    if place.requires_minecraft_namespace() {
        // `isValidMinecraftNamespaceGeometryIdentifier`: `minecraft:` and a geometry name.
        let valid = identifier
            .strip_prefix("minecraft:")
            .is_some_and(has_geometry_name);
        return (!valid).then_some((&codes::NEEDS_MINECRAFT_NAMESPACE, "minecraft"));
    }
    if !identifier.contains(':') {
        return None;
    }
    Some(if place.base_game {
        (&codes::NO_NAMESPACE_INTERNAL, "internal")
    } else {
        (&codes::NO_NAMESPACE_CUSTOM, "custom")
    })
}

/// The identifier loop: the index of the first geometry whose identifier breaks its rule, with
/// the error reported. That geometry and every later one of the file are not registered.
pub(crate) fn first_bad_identifier(
    file: &GeometryFile,
    original: FormatVersion,
    place: Place,
    diagnostics: &mut Diagnostics,
) -> Option<usize> {
    if original < V1_19_30 {
        // Below 1.12 the key was split into identifier and parent; from 1.12 the schema's
        // validator already refused a `:`.
        return None;
    }
    file.geometries.iter().enumerate().find_map(|(index, g)| {
        let (def, _) = identifier_error(&g.description.identifier, place)?;
        let at = geometry_path(index)
            .child("description")
            .child("identifier");
        diagnostics.push(
            Diagnostic::of(def.with([g.description.identifier.clone()]))
                .at(at.clone())
                .phase(Phase::Validate),
        );
        Some(index)
    })
}

fn geometry_path(index: usize) -> JsonPath {
    JsonPath::new().child("minecraft:geometry").index(index)
}

/// `Geometry::parse` over every geometry of the file: clamps, truncation, the transform gates
/// and ranges, the poly-mesh counts and the bone table. Nothing here fails the file.
pub(crate) fn parse_time(
    file: &mut GeometryFile,
    original: FormatVersion,
    diagnostics: &mut Diagnostics,
) {
    // A file below 1.12 has its geometries as `geometry.<name>[:<parent>]` members of the
    // root, the members of the description directly in them: a finding names the member
    // where the file has it.
    let keyed = original < V1_12_0;
    for (index, geometry) in file.geometries.iter_mut().enumerate() {
        let mut at = if keyed {
            let mut at = JsonPath::new();
            at.push(&match &geometry.legacy.parent {
                Some(parent) => format!("{}:{parent}", geometry.description.identifier),
                None => geometry.description.identifier.clone(),
            });
            at
        } else {
            geometry_path(index)
        };
        clamp_bounds(geometry, &mut at, !keyed, diagnostics);
        transforms(geometry, original, &mut at, diagnostics);
        bones(geometry, &mut at, diagnostics);
    }
}

/// `visible_bounds_width` / `visible_bounds_height` are clamped to 0.1–50 (vanilla: silently).
fn clamp_bounds(
    geometry: &mut Geometry,
    at: &mut JsonPath,
    described: bool,
    diagnostics: &mut Diagnostics,
) {
    if described {
        at.push("description");
    }
    let description = &mut geometry.description;
    for (name, member, value) in [
        (
            "width",
            "visible_bounds_width",
            &mut description.visible_bounds_width,
        ),
        (
            "height",
            "visible_bounds_height",
            &mut description.visible_bounds_height,
        ),
    ] {
        let Some(written) = *value else { continue };
        // `fmaxnm` then a select against the maximum: a NaN becomes the minimum.
        let clamped = written.max(BOUNDS.0).min(BOUNDS.1);
        if clamped.to_bits() != written.to_bits() {
            *value = Some(clamped);
            at.push(member);
            diagnostics.push(
                Diagnostic::of(codes::BOUNDS_CLAMPED.with([
                    (*name).to_owned(),
                    format!("{written:.6}"),
                    format!("{clamped:.6}"),
                ]))
                .at(at.clone())
                .phase(Phase::Validate),
            );
            at.pop();
        }
    }
    if described {
        at.pop();
    }
}

/// The ranges of a transform's vectors: translation and the pivots −80..80, rotation −360..360,
/// scale 0..4.
const RANGES: [(&str, &str, f32, f32); 5] = [
    ("translation", "translation", -80.0, 80.0),
    ("rotation", "rotation", -360.0, 360.0),
    ("scale", "scale", 0.0, 4.0),
    ("rotation_pivot", "pivot", -80.0, 80.0),
    ("scale_pivot", "pivot", -80.0, 80.0),
];

fn transforms(
    geometry: &mut Geometry,
    original: FormatVersion,
    at: &mut JsonPath,
    diagnostics: &mut Diagnostics,
) {
    let identifier = geometry.description.identifier.clone();
    let Some(transforms) = &mut geometry.item_display_transforms else {
        return;
    };
    at.push("item_display_transforms");
    if transforms.embedded.is_some() && original < EMBEDDED_FROM {
        transforms.embedded = None;
        at.push("embedded");
        // The game logs an error, ignores the transform and loads the geometry: a kept error.
        kept(
            diagnostics,
            codes::EMBEDDED_TOO_OLD.with([identifier.clone()]),
            at,
        );
        at.pop();
    }
    for (name, slot) in super::ItemDisplayTransforms::NAMES
        .into_iter()
        .zip(transforms.slots_mut())
    {
        if let Some(transform) = slot {
            at.push(name);
            ranges(transform, at, diagnostics);
            at.pop();
        }
    }
    at.pop();
}

/// `ItemTransforms::parseItemDisplayTransform`: a vector with a component outside its range is
/// reported and not applied.
fn ranges(transform: &mut ItemDisplayTransform, at: &mut JsonPath, diagnostics: &mut Diagnostics) {
    let slots = [
        &mut transform.translation,
        &mut transform.rotation,
        &mut transform.scale,
        &mut transform.rotation_pivot,
        &mut transform.scale_pivot,
    ];
    for ((member, noun, low, high), slot) in RANGES.into_iter().zip(slots) {
        let Some(vector) = *slot else { continue };
        if vector.iter().all(|x| (low..=high).contains(x)) {
            continue;
        }
        *slot = None;
        let [x, y, z] = vector.map(|x| format!("{x:.6}"));
        let (low, high) = (low.to_string(), high.to_string());
        at.push(member);
        // The game logs an error, does not apply the vector and loads the geometry: a kept
        // error.
        kept(
            diagnostics,
            codes::TRANSFORM_OUT_OF_BOUNDS.with([
                noun.to_owned(),
                x,
                y,
                z,
                low.clone(),
                low.clone(),
                low,
                high.clone(),
                high.clone(),
                high,
            ]),
            at,
        );
        at.pop();
    }
}

/// The note that the member `key` below `at`, written `given`, is held and written as `lower`.
fn case_folded(
    key: &str,
    given: &str,
    lower: &str,
    at: &mut JsonPath,
    diagnostics: &mut Diagnostics,
) {
    at.push(key);
    diagnostics.push(
        Diagnostic::of(codes::BONE_LOWER_CASED.with([key, given, lower]))
            .at(at.clone())
            .phase(Phase::Validate),
    );
    at.pop();
}

/// `_parseBones`: names and parents are lower-cased and bones are keyed by name, so a second
/// bone of a name replaces the first; box UVs are whole texels; a mesh without an index list
/// needs lists of matching lengths.
fn bones(geometry: &mut Geometry, at: &mut JsonPath, diagnostics: &mut Diagnostics) {
    let identifier = geometry.description.identifier.clone();
    let Some(bones) = &mut geometry.bones else {
        return;
    };
    at.push("bones");
    let mut merged: Vec<Bone> = Vec::with_capacity(bones.len());
    let mut by_name = std::collections::HashMap::with_capacity(bones.len());
    // Where in the file the bone at each place of `merged` is.
    let mut written_at: Vec<usize> = Vec::with_capacity(bones.len());
    for (index, mut bone) in std::mem::take(bones).into_iter().enumerate() {
        at.push_index(index);
        // The game's lower-casing; a name the file has in another case is noted where it
        // is, with an information of the crate's own.
        let lower = bone.name.to_lowercase();
        if lower != bone.name {
            case_folded("name", &bone.name, &lower, at, diagnostics);
            bone.name = lower;
        }
        if let Some(parent) = &mut bone.parent {
            let lower = parent.to_lowercase();
            if lower != *parent {
                case_folded("parent", parent, &lower, at, diagnostics);
                *parent = lower;
            }
        }
        for cube in bone.cubes.iter_mut().flatten() {
            if let Some(CubeUv::Box(uv)) = &mut cube.uv {
                // The game reads each component as an integer, dropping the fraction.
                *uv = uv.map(f32::trunc);
            }
        }
        if let Some(mesh) = &bone.poly_mesh {
            at.push("poly_mesh");
            poly_mesh(mesh, at, diagnostics);
            at.pop();
        }
        if let Some(first) = by_name.get(&bone.name) {
            // The game merges a lower-cased name into the bone it already has, without a
            // message; the crate's own information tells of it.
            diagnostics.push(
                Diagnostic::of(&core_codes::DUPLICATE_BONE)
                    .at(at.clone())
                    .phase(Phase::Validate),
            );
            // The bone that is replaced is noted (an information of the crate's own; the game
            // merges without a word) where the file has it, so what it held is known to be gone.
            let replaced = std::mem::replace(&mut written_at[*first], index);
            at.pop();
            at.push_index(replaced);
            diagnostics.push(
                Diagnostic::of(&core_codes::DROPPED_FIELD)
                    .at(at.clone())
                    .phase(Phase::Validate),
            );
            at.pop();
            at.push_index(index);
            merged[*first] = bone;
        } else {
            written_at.push(index);
            by_name.insert(bone.name.clone(), merged.len());
            merged.push(bone);
        }
        at.pop();
    }
    // A geometry with a legacy parent also has that parent's bones, which only the pack stack
    // knows: its missing bone parents are judged there (`inherited_parents`).
    let inherits = geometry.legacy.parent.is_some();
    parents(&merged, &by_name, &identifier, inherits, at, diagnostics);
    *bones = merged;
    at.pop();
}

/// A parent that is no bone of the geometry (vanilla attaches the bone to the root
/// without a message), a bone that is its own parent, and a longer cycle.
fn parents(
    bones: &[Bone],
    by_name: &std::collections::HashMap<String, usize>,
    identifier: &str,
    inherits: bool,
    at: &mut JsonPath,
    diagnostics: &mut Diagnostics,
) {
    let parent_of: Vec<Option<usize>> = bones
        .iter()
        .map(|bone| bone.parent.as_ref().and_then(|p| by_name.get(p).copied()))
        .collect();
    // 0: not visited, 1: on the walk in progress, 2: settled.
    let mut state = vec![0_u8; bones.len()];
    let mut in_cycle = vec![false; bones.len()];
    for start in 0..bones.len() {
        let mut walk = Vec::new();
        let mut next = Some(start);
        while let Some(index) = next {
            match state[index] {
                0 => {
                    state[index] = 1;
                    walk.push(index);
                    next = parent_of[index];
                }
                1 => {
                    let from = walk.iter().position(|i| *i == index).unwrap_or(0);
                    for member in &walk[from..] {
                        in_cycle[*member] = true;
                    }
                    break;
                }
                _ => break,
            }
        }
        for index in walk {
            state[index] = 2;
        }
    }
    for (index, bone) in bones.iter().enumerate() {
        let Some(parent) = &bone.parent else { continue };
        at.push_index(index);
        at.push("parent");
        if parent_of[index].is_none() {
            if inherits {
                at.pop();
                at.pop();
                continue;
            }
            diagnostics.push(
                Diagnostic::of(codes::BONE_PARENT_MISSING.with([
                    bone.name.as_str(),
                    parent,
                    identifier,
                ]))
                .at(at.clone())
                .phase(Phase::Validate),
            );
        } else if in_cycle[index] {
            // The self-parent text is a developer assertion the game raises when a model
            // instance is built, never a content-log line at load; a longer cycle has no text
            // of the game's. Both are informations of the crate's own.
            let finding = if parent_of[index] == Some(index) {
                Diagnostic::of(&codes::PARENT_IS_SELF)
            } else {
                Diagnostic::of(&core_codes::REFERENCE_CYCLE).severity(Severity::Info)
            };
            diagnostics.push(finding.at(at.clone()).phase(Phase::Validate));
        }
        at.pop();
        at.pop();
    }
}

/// The list lengths a mesh without an index list needs.
fn poly_mesh(mesh: &PolyMesh, at: &JsonPath, diagnostics: &mut Diagnostics) {
    let Polys::List(kind) = &mesh.polys else {
        return;
    };
    let len = |n: Option<usize>| n.unwrap_or(0);
    let counts = [
        len(mesh.positions.as_ref().map(Vec::len)),
        len(mesh.normals.as_ref().map(Vec::len)),
        len(mesh.uvs.as_ref().map(Vec::len)),
    ];
    // The game logs these poly-mesh lines as errors and loads the geometry: kept errors.
    let mut warn = |code: Code| kept(diagnostics, code, at);
    if counts[0] != counts[1] || counts[1] != counts[2] {
        warn(codes::NO_POLYS_SECTION.with(counts.map(|n| n.to_string())));
    }
    if kind.contains("tri_list") && counts[0] % 3 != 0 {
        warn(Code::from(&codes::NOT_TRIANGLES));
    } else if !kind.contains("tri_list") && counts[0] % 4 != 0 {
        warn(Code::from(&codes::NOT_QUADS));
    }
}

/// The same check for a geometry with a legacy parent: the finding of each bone (by index) whose
/// parent is neither one of the geometry's own bones nor one it inherits, as the stack found
/// them. `index_in_file` is the geometry's place in `minecraft:geometry`.
pub(crate) fn missing_bone_parents(
    geometry: &Geometry,
    index_in_file: usize,
    bones: &[usize],
    diagnostics: &mut Diagnostics,
) {
    for index in bones {
        let Some(bone) = geometry.bones().get(*index) else {
            continue;
        };
        // The path the file stage gives the same rule.
        let at = geometry_path(index_in_file)
            .child("bones")
            .index(*index)
            .child("parent");
        diagnostics.push(
            Diagnostic::of(codes::BONE_PARENT_MISSING.with([
                bone.name.as_str(),
                bone.parent.as_deref().unwrap_or_default(),
                geometry.description.identifier.as_str(),
            ]))
            .at(at.clone())
            .phase(Phase::Validate),
        );
    }
}
