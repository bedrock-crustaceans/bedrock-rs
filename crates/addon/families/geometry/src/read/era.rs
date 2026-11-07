//! Reading a geometry file of 1.12.0 or later against the schema of its era, and the member
//! readers the two older eras share with it.
//!
//! The game builds the 1.12 schema and, for 1.14 and later, one shared schema whose members are
//! gated on the schema version: `binding` from 1.16, a namespaced identifier from 1.19.30 and
//! the 1.21 members from 1.21.

use indexmap::IndexMap;

use crate::codes;
use crate::walk::{Leaves, has_geometry_name};
use crate::{
    Bone, Cube, CubeUv, Description, FaceUv, Faces, Geometry, ItemDisplayTransform,
    ItemDisplayTransforms, Locator, PolyMesh, Polys, TextureMesh,
};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::json::node_tree::{Group, Hit, Kind, Node, Pattern, Walk};
use bedrock_addon_core::version::{FormatVersion, known};

/// What the schema of a file's era declares beyond the 1.12 members.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "one flag per schema gate of the shared builder"
)]
pub(crate) struct Era {
    /// Bone `binding` (from 1.16.0).
    pub(crate) binding: bool,
    /// `description.identifier` may carry a namespace (from 1.19.30).
    pub(crate) namespace: bool,
    /// `item_display_transforms` and face `uv_rotation` (from 1.21.0).
    pub(crate) transforms: bool,
    /// `item_display_transforms.shelf`: declared by an engine of 1.26.40 or later (in a file
    /// below 1.26.40 the upgrade then rejects it).
    pub(crate) shelf: bool,
}

/// 1.12.0: the first version with the `minecraft:geometry` list; below it the geometries are
/// `geometry.<name>` members of the root.
pub(crate) const V1_12_0: FormatVersion = FormatVersion::new(1, 12, 0);
/// 1.16.0.
pub(crate) const V1_16_0: FormatVersion = FormatVersion::new(1, 16, 0);
/// 1.19.30: the first strict era.
pub(crate) const V1_19_30: FormatVersion = FormatVersion::new(1, 19, 30);
/// 1.21.0.
pub(crate) const V1_21_0: FormatVersion = FormatVersion::new(1, 21, 0);
/// 1.26.40: the first engine with the `shelf` transform.
pub(crate) const V1_26_40: FormatVersion = known::V1_26_40;

impl Era {
    /// The schema of the struct `schema` (a chain version from 1.12.0 on) in a game of
    /// `engine`.
    pub(crate) fn of(schema: FormatVersion, engine: FormatVersion) -> Self {
        Self {
            binding: schema >= V1_16_0,
            namespace: schema >= V1_19_30,
            transforms: schema >= V1_21_0,
            // The member lives in `item_display_transforms`, so only a schema with `transforms`
            // asks; a file of 1.26.40 or later is only read by an engine that knows it.
            shelf: engine >= V1_26_40,
        }
    }
}

/// A file of 1.12.0 or later as its schema reads it.
pub(crate) struct Modern {
    pub(crate) debug: Option<bool>,
    pub(crate) geometries: Vec<Geometry>,
}

/// The schema walk of a whole file of 1.12.0 or later.
pub(crate) fn read_file(walk: &mut Walk, root: Node<'_>, era: Era) -> Option<Modern> {
    let mut debug = None;
    let mut geometries = None;
    let groups = [
        Group::required("format_version"),
        Group::name("debug"),
        Group::required("minecraft:geometry"),
    ];
    let ok = walk.object(root, &groups, |walk, group, _, node| match group {
        // The version was read before the schema was chosen.
        0 => Hit::Ok,
        1 => walk.bool(node).map(|b| debug = Some(b)).into(),
        _ => read_geometries(walk, node, era)
            .map(|list| geometries = Some(list))
            .into(),
    });
    let geometries = geometries?;
    ok.then_some(Modern { debug, geometries })
}

fn read_geometries(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<Vec<Geometry>> {
    let list = walk.list(node, |walk, element| read_geometry(walk, element, era))?;
    if walk.elements(node).is_some_and(|e| e.is_empty()) {
        // The array node's minimum of one element, counted on the list as written: a list
        // whose every element the lenient read left out (below 1.19.30, each with its
        // information) held its elements when it was validated and loads, with no geometry.
        walk.error(
            core_codes::ARRAY_TOO_SMALL.with(["0", "1"]).into(),
            node.span(),
        );
        return None;
    }
    Some(list)
}

fn read_geometry(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<Geometry> {
    let mut g = Geometry::default();
    let mut groups = vec![
        Group::required("description"),
        Group::name("cape"),
        Group::name("bones"),
    ];
    if era.transforms {
        groups.push(Group::name("item_display_transforms"));
    }
    let ok = walk.object(node, &groups, |walk, group, _, v| match group {
        0 => read_description(walk, v, era)
            .map(|d| g.description = d)
            .into(),
        1 => walk.string(v).map(|s| g.cape = Some(s)).into(),
        2 => walk
            .list(v, |walk, bone| read_bone(walk, bone, era))
            .map(|bones| g.bones = Some(bones))
            .into(),
        _ => read_transforms(walk, v, era)
            .map(|t| g.item_display_transforms = Some(t))
            .into(),
    });
    ok.then_some(g)
}

fn read_description(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<Description> {
    let mut d = Description::default();
    let groups = [
        Group::required("identifier"),
        Group::name("visible_bounds_width"),
        Group::name("visible_bounds_height"),
        Group::name("visible_bounds_offset"),
        Group::name("texture_width"),
        Group::name("texture_height"),
    ];
    let ok = walk.object(node, &groups, |walk, group, _, v| match group {
        0 => read_identifier(walk, v, era)
            .map(|id| d.identifier = id)
            .into(),
        1 => walk
            .float(v)
            .map(|x| d.visible_bounds_width = Some(x))
            .into(),
        2 => walk
            .float(v)
            .map(|x| d.visible_bounds_height = Some(x))
            .into(),
        3 => walk
            .floats(v)
            .map(|x| d.visible_bounds_offset = Some(x))
            .into(),
        4 => walk.int(v).map(|x| d.texture_width = Some(x)).into(),
        _ => walk.int(v).map(|x| d.texture_height = Some(x)).into(),
    });
    ok.then_some(d)
}

/// `description.identifier` through the era's validator: `isValidGeometryIdentifier` (no `:`,
/// and the geometry name somewhere in the text) or, from 1.19.30,
/// `isValidOptionalNamespaceGeometryIdentifier` (the name after an optional `namespace:`). A
/// failing validator fails the file in either mode.
fn read_identifier(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<String> {
    let identifier = walk.string(node)?;
    let name = if era.namespace {
        identifier
            .split_once(':')
            .map_or(identifier.as_str(), |(_, name)| name)
    } else if identifier.contains(':') {
        walk.error(
            codes::NO_NAMESPACE_CUSTOM.with([identifier.clone()]).into(),
            node.span(),
        );
        return None;
    } else {
        identifier.as_str()
    };
    if !has_geometry_name(name) {
        walk.error(
            DiagnosticCode::from(&core_codes::INVALID_VALUE),
            node.span(),
        );
        return None;
    }
    Some(identifier)
}

/// The bone members every era from 1.12.0 declares, then `binding` from 1.16.0.
const BONE_MEMBERS: [Group; 13] = [
    Group::required("name"),
    Group::name("parent"),
    Group::name("pivot"),
    Group::name("rotation"),
    Group::name("mirror"),
    Group::name("inflate"),
    Group::name("debug"),
    Group::name("render_group_id"),
    Group::name("cubes"),
    Group::name("locators"),
    Group::name("poly_mesh"),
    Group::name("texture_meshes"),
    Group::name("binding"),
];

fn read_bone(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<Bone> {
    let mut b = Bone::default();
    let groups = &BONE_MEMBERS[..if era.binding { 13 } else { 12 }];
    let ok = walk.object(node, groups, |walk, group, _, v| match group {
        0 => walk.string(v).map(|s| b.name = s).into(),
        1 => walk.string(v).map(|s| b.parent = Some(s)).into(),
        2 => walk.floats(v).map(|x| b.pivot = Some(x)).into(),
        3 => walk.floats(v).map(|x| b.rotation = Some(x)).into(),
        4 => walk.bool(v).map(|x| b.mirror = Some(x)).into(),
        5 => walk.float(v).map(|x| b.inflate = Some(x)).into(),
        6 => walk.bool(v).map(|x| b.debug = Some(x)).into(),
        7 => read_render_group_id(walk, v)
            .map(|x| b.render_group_id = Some(x))
            .into(),
        8 => walk
            .list(v, |walk, cube| read_cube(walk, cube, era))
            .map(|cubes| b.cubes = Some(cubes))
            .into(),
        9 => read_locators(walk, v, LocatorForm::Modern)
            .map(|l| b.locators = Some(l))
            .into(),
        10 => read_poly_mesh(walk, v)
            .map(|m| b.poly_mesh = Some(m))
            .into(),
        11 => walk
            .list(v, read_texture_mesh)
            .map(|m| b.texture_meshes = Some(m))
            .into(),
        _ => walk.molang(v).map(|m| b.binding = Some(m)).into(),
    });
    ok.then_some(b)
}

/// `render_group_id`: an int of at least 0 (the int node's inline minimum); a lower value fails
/// the file.
pub(crate) fn read_render_group_id(walk: &mut Walk, node: Node<'_>) -> Option<u32> {
    let value = walk.int(node)?;
    let Ok(id) = u32::try_from(value) else {
        walk.error(
            core_codes::OUTSIDE_RANGE
                .with([value.to_string(), "0".to_owned(), i32::MAX.to_string()])
                .into(),
            node.span(),
        );
        return None;
    };
    Some(id)
}

fn read_cube(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<Cube> {
    let mut c = Cube::default();
    let groups = [
        Group::name("origin"),
        Group::name("size"),
        Group::name("rotation"),
        Group::name("pivot"),
        Group::name("inflate"),
        Group::name("mirror"),
        Group::name("uv"),
    ];
    let ok = walk.object(node, &groups, |walk, group, _, v| match group {
        0 => walk.floats(v).map(|x| c.origin = Some(x)).into(),
        1 => walk.floats(v).map(|x| c.size = Some(x)).into(),
        2 => walk.floats(v).map(|x| c.rotation = Some(x)).into(),
        3 => walk.floats(v).map(|x| c.pivot = Some(x)).into(),
        4 => walk.float(v).map(|x| c.inflate = Some(x)).into(),
        5 => walk.bool(v).map(|x| c.mirror = Some(x)).into(),
        _ => read_cube_uv(walk, v, era).map(|uv| c.uv = Some(uv)).into(),
    });
    ok.then_some(c)
}

fn read_cube_uv(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<CubeUv> {
    match node.kind() {
        Kind::Array => walk.texels(node).map(CubeUv::Box),
        Kind::Object => {
            let mut faces = Faces::default();
            let groups = [
                Group::name("north"),
                Group::name("south"),
                Group::name("east"),
                Group::name("west"),
                Group::name("up"),
                Group::name("down"),
            ];
            let ok = walk.object(node, &groups, |walk, group, _, v| {
                let slot = match group {
                    0 => &mut faces.north,
                    1 => &mut faces.south,
                    2 => &mut faces.east,
                    3 => &mut faces.west,
                    4 => &mut faces.up,
                    _ => &mut faces.down,
                };
                read_face(walk, v, era)
                    .map(|face| *slot = Some(face))
                    .into()
            });
            ok.then_some(CubeUv::Faces(Box::new(faces)))
        }
        _ => {
            walk.bad_type("array, object", node);
            None
        }
    }
}

fn read_face(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<FaceUv> {
    let mut face = FaceUv::default();
    let groups = [
        Group::required("uv"),
        Group::name("uv_size"),
        Group::name("material_instance"),
        Group::name("uv_rotation"),
    ];
    let groups = &groups[..if era.transforms { 4 } else { 3 }];
    let ok = walk.object(node, groups, |walk, group, _, v| match group {
        0 => walk.floats(v).map(|uv| face.uv = uv).into(),
        1 => walk.floats(v).map(|x| face.uv_size = Some(x)).into(),
        2 => walk
            .string(v)
            .map(|s| face.material_instance = Some(s))
            .into(),
        _ => {
            face.uv_rotation = read_uv_rotation(walk, v);
            Hit::Ok
        }
    });
    ok.then_some(face)
}

/// `uv_rotation` (`Geometry::_parseUvRotation`): 0, 90, 180 or 270 as an integer; anything else
/// is reported with vanilla's text and the face is not rotated.
fn read_uv_rotation(walk: &mut Walk, node: Node<'_>) -> Option<i32> {
    // An integer JSON value (`isInt` / `isUInt`) is compared as the number it is.
    let quarter = (node.kind() == Kind::Number)
        .then(|| node.text().parse::<i32>().ok())
        .flatten()
        .filter(|turn| matches!(turn, 0 | 90 | 180 | 270));
    if quarter.is_none() {
        // The game logs an error, uses rotation 0 and loads the geometry: a kept error.
        walk.push_kept(
            codes::INVALID_UV_ROTATION.with([node.text()]).into(),
            node.span(),
        );
    }
    quarter
}

/// Which locator values a schema declares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LocatorForm {
    /// Below 1.8.0: a vector, or an object whose `rotation` is optional.
    Pre18,
    /// 1.8.0 up to 1.12.0: a vector only.
    VectorOnly,
    /// From 1.12.0: a vector, or an object whose `rotation` is required.
    Modern,
}

/// `locators`: names matching `[a-zA-Z0-9_.-]+`.
pub(crate) fn read_locators(
    walk: &mut Walk,
    node: Node<'_>,
    form: LocatorForm,
) -> Option<IndexMap<String, Locator>> {
    let mut map = IndexMap::new();
    let ok = walk.object(
        node,
        &[Group::pattern(Pattern::IDENTIFIER, false)],
        |walk, _, key, v| {
            read_locator(walk, v, form)
                .map(|locator| map.insert(key.to_owned(), locator))
                .into()
        },
    );
    ok.then_some(map)
}

/// A locator vector as read.
enum Vector {
    /// Three numbers.
    Read([f32; 3]),
    /// An array of numbers of another length (reported).
    WrongLength,
}

/// A locator vector. Vanilla declares these arrays without a size; the crate reports a length
/// other than three and leaves the locator out. `None`
/// is a value that is not an array of numbers.
fn read_locator_vector(walk: &mut Walk, node: Node<'_>) -> Option<Vector> {
    let values = walk.float_list(node)?;
    let vector: Result<[f32; 3], _> = values.try_into();
    if let Ok(vector) = vector {
        return Some(Vector::Read(vector));
    }
    walk.push(
        DiagnosticCode::from(&core_codes::INVALID_VALUE),
        Severity::Info,
        node.span(),
    );
    Some(Vector::WrongLength)
}

fn read_locator(walk: &mut Walk, node: Node<'_>, form: LocatorForm) -> Option<Locator> {
    match (node.kind(), form) {
        (Kind::Array, _) => match read_locator_vector(walk, node)? {
            Vector::Read(at) => Some(Locator::Position(at)),
            Vector::WrongLength => None,
        },
        (Kind::Object, LocatorForm::Pre18 | LocatorForm::Modern) => {
            let mut offset = None;
            let mut rotation = None;
            let mut ignore_inherited_scale = None;
            let mut complete = true;
            let groups = [
                Group::required("offset"),
                Group {
                    pattern: Pattern::Name("rotation"),
                    required: form == LocatorForm::Modern,
                },
                Group::name("ignore_inherited_scale"),
            ];
            let ok = walk.object(node, &groups, |walk, group, _, v| match group {
                0 | 1 => read_locator_vector(walk, v)
                    .map(|vector| match vector {
                        Vector::Read(vector) if group == 0 => offset = Some(vector),
                        Vector::Read(vector) => rotation = Some(vector),
                        Vector::WrongLength => complete = false,
                    })
                    .into(),
                _ => walk
                    .bool(v)
                    .map(|x| ignore_inherited_scale = Some(x))
                    .into(),
            });
            if ok && !complete {
                // Left out for a vector of another length (the crate's own information): the
                // vector's finding does not say that the locator's other members go with it.
                walk.push(
                    DiagnosticCode::from(&core_codes::DROPPED_FIELD),
                    Severity::Info,
                    node.span(),
                );
            }
            (ok && complete).then_some(())?;
            Some(Locator::Placed {
                offset: offset?,
                rotation,
                ignore_inherited_scale,
            })
        }
        (_, LocatorForm::VectorOnly) => {
            walk.bad_type("array", node);
            None
        }
        _ => {
            walk.bad_type("array, object", node);
            None
        }
    }
}

/// `poly_mesh` (every schema from 1.8.0).
pub(crate) fn read_poly_mesh(walk: &mut Walk, node: Node<'_>) -> Option<PolyMesh> {
    let mut normalized_uvs = None;
    let mut positions = None;
    let mut normals = None;
    let mut uvs = None;
    let mut polys = None;
    let groups = [
        Group::name("normalized_uvs"),
        Group::name("positions"),
        Group::name("normals"),
        Group::name("uvs"),
        Group::required("polys"),
    ];
    let ok = walk.object(node, &groups, |walk, group, _, v| match group {
        0 => walk.bool(v).map(|x| normalized_uvs = Some(x)).into(),
        1 => walk
            .list(v, Walk::floats)
            .map(|x| positions = Some(x))
            .into(),
        2 => walk.list(v, Walk::floats).map(|x| normals = Some(x)).into(),
        3 => walk.list(v, Walk::floats).map(|x| uvs = Some(x)).into(),
        _ => read_polys(walk, v).map(|x| polys = Some(x)).into(),
    });
    let polys = polys?;
    ok.then_some(PolyMesh {
        normalized_uvs,
        positions,
        normals,
        uvs,
        polys,
    })
}

fn read_polys(walk: &mut Walk, node: Node<'_>) -> Option<Polys> {
    match node.kind() {
        Kind::String => {
            let value = walk.string(node)?;
            // The valid values are regexes searched in the value.
            if !(value.contains("tri_list") || value.contains("quad_list")) {
                walk.error(
                    core_codes::NOT_ALLOWED_REGEX
                        .with([value.as_str(), "tri_list, quad_list"])
                        .into(),
                    node.span(),
                );
                return None;
            }
            Some(Polys::List(value))
        }
        Kind::Array => walk
            .list(node, |walk, poly| {
                let vertices = walk.list(poly, Walk::indices)?;
                let n = vertices.len();
                let (def, bound) = match n {
                    0..3 => (&core_codes::ARRAY_TOO_SMALL, "3"),
                    3..=4 => return Some(vertices),
                    _ => (&core_codes::ARRAY_TOO_LARGE, "4"),
                };
                walk.error(
                    def.with([n.to_string(), bound.to_owned()]).into(),
                    poly.span(),
                );
                None
            })
            .map(Polys::Indices),
        _ => {
            walk.bad_type("array, string", node);
            None
        }
    }
}

/// One `texture_meshes` element (every schema from 1.8.0).
pub(crate) fn read_texture_mesh(walk: &mut Walk, node: Node<'_>) -> Option<TextureMesh> {
    let mut m = TextureMesh::default();
    let groups = [
        Group::required("texture"),
        Group::name("position"),
        Group::name("local_pivot"),
        Group::name("rotation"),
        Group::name("scale"),
        Group::name("use_pixel_depth"),
    ];
    let ok = walk.object(node, &groups, |walk, group, _, v| match group {
        0 => walk.string(v).map(|s| m.texture = s).into(),
        1 => walk.floats(v).map(|x| m.position = Some(x)).into(),
        2 => walk.floats(v).map(|x| m.local_pivot = Some(x)).into(),
        3 => walk.floats(v).map(|x| m.rotation = Some(x)).into(),
        4 => walk.floats(v).map(|x| m.scale = Some(x)).into(),
        _ => walk.bool(v).map(|x| m.use_pixel_depth = Some(x)).into(),
    });
    ok.then_some(m)
}

/// `item_display_transforms`: one optional object per name of `ItemTransforms::mTypeNames`
/// (nine names, ten with `shelf`); any other key is undeclared.
fn read_transforms(walk: &mut Walk, node: Node<'_>, era: Era) -> Option<ItemDisplayTransforms> {
    let mut transforms = ItemDisplayTransforms::default();
    let groups = ItemDisplayTransforms::NAMES.map(Group::name);
    let groups = &groups[..if era.shelf { 10 } else { 9 }];
    let ok = walk.object(node, groups, |walk, group, key, v| {
        let read = read_transform(walk, v, key == "gui");
        let mut slots = transforms.slots_mut();
        read.map(|t| *slots[group] = Some(t)).into()
    });
    ok.then_some(transforms)
}

fn read_transform(walk: &mut Walk, node: Node<'_>, gui: bool) -> Option<ItemDisplayTransform> {
    let mut t = ItemDisplayTransform::default();
    let groups = [
        Group::name("translation"),
        Group::name("rotation"),
        Group::name("scale"),
        Group::name("rotation_pivot"),
        Group::name("scale_pivot"),
        Group::name("fit_to_frame"),
    ];
    // `fit_to_frame` is declared under `gui` only.
    let groups = &groups[..if gui { 6 } else { 5 }];
    let ok = walk.object(node, groups, |walk, group, _, v| match group {
        0 => walk.floats(v).map(|x| t.translation = Some(x)).into(),
        1 => walk.floats(v).map(|x| t.rotation = Some(x)).into(),
        2 => walk.floats(v).map(|x| t.scale = Some(x)).into(),
        3 => walk.floats(v).map(|x| t.rotation_pivot = Some(x)).into(),
        4 => walk.floats(v).map(|x| t.scale_pivot = Some(x)).into(),
        _ => walk.bool(v).map(|x| t.fit_to_frame = Some(x)).into(),
    });
    ok.then_some(t)
}
