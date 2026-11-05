//! Format 1.8.0 up to 1.12.0: geometries as `"geometry.<name>"` members of the root, bones with
//! `parent` and `bind_pose_rotation`, `poly_mesh` and `texture_meshes`; and the step to 1.12,
//! which builds `minecraft:geometry[]`. The 1.8 schema's `format_version` maximum is 1.11.0,
//! and the game splits a geometry's key at its last `:` into the identifier and the parent.

use indexmap::IndexMap;

use super::super::read::{
    LocatorForm, read_locators, read_poly_mesh, read_render_group_id, read_texture_mesh,
};
use super::super::schema::{ABOVE_MAX_VERSION, CAPE_DROPPED, Group, Node, SchemaWalk, text};
use super::super::{
    Bone as NewBone, BoneLegacy, Cube as NewCube, CubeUv, Description, Geometry as NewGeometry,
    GeometryLegacy, Locator, PolyMesh, TextureMesh,
};
use super::v0_0_0::{SHARED_GEOMETRY_MEMBERS, read_root};
use super::{settle, v1_12_0, walk_for};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Span,
};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned};

/// The 1.8 schema's `format_version` maximum: a file above it and below 1.12.0 does not
/// validate.
pub const MAX_FORMAT_VERSION: FormatVersion = FormatVersion::new(1, 11, 0);

/// A geometry file of format 1.8.0 up to 1.12.0.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryFile {
    /// `format_version`, as written (at most 1.11.0).
    pub format_version: FormatVersion,
    /// `debug`.
    pub debug: Option<bool>,
    /// The `geometry.<name>` members, by key as written (`geometry.child:geometry.parent`
    /// declares inheritance).
    pub geometries: IndexMap<String, Geometry>,
}

/// One geometry of a 1.8 file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    /// `debug`.
    pub debug: Option<bool>,
    /// `visible_bounds_width`.
    pub visible_bounds_width: Option<f32>,
    /// `visible_bounds_height`.
    pub visible_bounds_height: Option<f32>,
    /// `visible_bounds_offset`.
    pub visible_bounds_offset: Option<[f32; 3]>,
    /// `texturewidth`.
    pub texturewidth: Option<i32>,
    /// `textureheight`.
    pub textureheight: Option<i32>,
    /// `cape`: lost by the step to 1.12.
    pub cape: Option<String>,
    /// `bones`.
    pub bones: Option<Vec<Bone>>,
    /// What the step from the older shape carried over (`preserve_model_pose`); never read
    /// from a 1.8 file.
    pub legacy: GeometryLegacy,
}

/// One bone of a 1.8 file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bone {
    /// `name`.
    pub name: String,
    /// `reset`.
    pub reset: Option<bool>,
    /// `neverRender`.
    pub never_render: Option<bool>,
    /// `parent`.
    pub parent: Option<String>,
    /// `pivot`.
    pub pivot: Option<[f32; 3]>,
    /// `rotation`.
    pub rotation: Option<[f32; 3]>,
    /// `bind_pose_rotation`.
    pub bind_pose_rotation: Option<[f32; 3]>,
    /// `mirror`.
    pub mirror: Option<bool>,
    /// `inflate`.
    pub inflate: Option<f32>,
    /// `debug`.
    pub debug: Option<bool>,
    /// `render_group_id`.
    pub render_group_id: Option<u32>,
    /// `cubes`.
    pub cubes: Option<Vec<Cube>>,
    /// `locators`: a 1.8 file holds vectors only; the object form is here only when the step
    /// from the older shape carried it over.
    pub locators: Option<IndexMap<String, Locator>>,
    /// `poly_mesh`.
    pub poly_mesh: Option<PolyMesh>,
    /// `texture_meshes`.
    pub texture_meshes: Option<Vec<TextureMesh>>,
}

/// One cube of a 1.8 file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cube {
    /// `origin`.
    pub origin: Option<[f32; 3]>,
    /// `size`.
    pub size: Option<[f32; 3]>,
    /// `uv`.
    pub uv: Option<[f32; 2]>,
    /// `inflate`.
    pub inflate: Option<f32>,
    /// `mirror`.
    pub mirror: Option<bool>,
}

// ---- reading -------------------------------------------------------------------------------------

fn read_geometry(walk: &mut SchemaWalk, node: Node<'_>) -> Option<Geometry> {
    let mut g = Geometry::default();
    let mut groups = SHARED_GEOMETRY_MEMBERS.to_vec();
    groups.push(Group::name("debug"));
    groups.push(Group::name("bones"));
    let ok = walk.object(node, &groups, |walk, group, _, v| match group {
        0 => walk
            .float(v)
            .map(|x| g.visible_bounds_width = Some(x))
            .into(),
        1 => walk
            .float(v)
            .map(|x| g.visible_bounds_height = Some(x))
            .into(),
        2 => walk
            .floats(v)
            .map(|x| g.visible_bounds_offset = Some(x))
            .into(),
        3 => walk.int(v).map(|x| g.texturewidth = Some(x)).into(),
        4 => walk.int(v).map(|x| g.textureheight = Some(x)).into(),
        5 => walk.string(v).map(|x| g.cape = Some(x)).into(),
        6 => walk.bool(v).map(|x| g.debug = Some(x)).into(),
        _ => walk.list(v, read_bone).map(|b| g.bones = Some(b)).into(),
    });
    ok.then_some(g)
}

fn read_bone(walk: &mut SchemaWalk, node: Node<'_>) -> Option<Bone> {
    let mut b = Bone::default();
    let groups = [
        Group::required("name"),
        Group::name("reset"),
        Group::name("neverRender"),
        Group::name("parent"),
        Group::name("pivot"),
        Group::name("rotation"),
        Group::name("bind_pose_rotation"),
        Group::name("mirror"),
        Group::name("inflate"),
        Group::name("debug"),
        Group::name("render_group_id"),
        Group::name("cubes"),
        Group::name("locators"),
        Group::name("poly_mesh"),
        Group::name("texture_meshes"),
    ];
    let ok = walk.object(node, &groups, |walk, group, _, v| match group {
        0 => walk.string(v).map(|s| b.name = s).into(),
        1 => walk.bool(v).map(|x| b.reset = Some(x)).into(),
        2 => walk.bool(v).map(|x| b.never_render = Some(x)).into(),
        3 => walk.string(v).map(|s| b.parent = Some(s)).into(),
        4 => walk.floats(v).map(|x| b.pivot = Some(x)).into(),
        5 => walk.floats(v).map(|x| b.rotation = Some(x)).into(),
        6 => walk
            .floats(v)
            .map(|x| b.bind_pose_rotation = Some(x))
            .into(),
        7 => walk.bool(v).map(|x| b.mirror = Some(x)).into(),
        8 => walk.float(v).map(|x| b.inflate = Some(x)).into(),
        9 => walk.bool(v).map(|x| b.debug = Some(x)).into(),
        10 => read_render_group_id(walk, v)
            .map(|x| b.render_group_id = Some(x))
            .into(),
        11 => walk.list(v, read_cube).map(|c| b.cubes = Some(c)).into(),
        12 => read_locators(walk, v, LocatorForm::VectorOnly)
            .map(|l| b.locators = Some(l))
            .into(),
        13 => read_poly_mesh(walk, v)
            .map(|m| b.poly_mesh = Some(m))
            .into(),
        _ => walk
            .list(v, read_texture_mesh)
            .map(|m| b.texture_meshes = Some(m))
            .into(),
    });
    ok.then_some(b)
}

fn read_cube(walk: &mut SchemaWalk, node: Node<'_>) -> Option<Cube> {
    let mut c = Cube::default();
    let groups = [
        Group::name("origin"),
        Group::name("size"),
        Group::name("uv"),
        Group::name("inflate"),
        Group::name("mirror"),
    ];
    let ok = walk.object(node, &groups, |walk, group, _, v| match group {
        0 => walk.floats(v).map(|x| c.origin = Some(x)).into(),
        1 => walk.floats(v).map(|x| c.size = Some(x)).into(),
        2 => walk.texels(v).map(|x| c.uv = Some(x)).into(),
        3 => walk.float(v).map(|x| c.inflate = Some(x)).into(),
        _ => walk.bool(v).map(|x| c.mirror = Some(x)).into(),
    });
    ok.then_some(c)
}

impl Versioned for GeometryFile {
    const VERSION: FormatVersion = FormatVersion::new(1, 8, 0);
}

impl MolangFields for GeometryFile {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl VersionStruct for GeometryFile {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut walk = walk_for(cx);
        if cx.original_version > MAX_FORMAT_VERSION {
            // The version node's maximum: 1.11.1 up to 1.12.0 fails the schema.
            walk.enter("format_version");
            walk.error(
                DiagnosticCode::UnsupportedVersion {
                    text: text(
                        ABOVE_MAX_VERSION,
                        &[
                            &cx.original_version.to_string(),
                            &MAX_FORMAT_VERSION.to_string(),
                        ],
                    ),
                },
                Span { offset: 0, len: 0 },
            );
            walk.leave();
        }
        let file = Node::of(payload).and_then(|root| read_root(&mut walk, root, read_geometry));
        settle(walk, file, d).map(|(debug, geometries)| Self {
            format_version: cx.original_version,
            debug,
            geometries,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.to_value()
    }
}

// ---- the step ------------------------------------------------------------------------------------

impl Upgrade for GeometryFile {
    type Next = v1_12_0::GeometryFile;

    /// The step to 1.12: one `minecraft:geometry` entry per `geometry.<name>` member,
    /// in key order. The key, split at its last `:`, gives the identifier and the parent;
    /// `texturewidth` / `textureheight` are renamed; `reset`, `neverRender` and
    /// `bind_pose_rotation` move to the bone's side-car; `cape` is lost.
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> v1_12_0::GeometryFile {
        let mut members: Vec<(String, Geometry)> = self.geometries.into_iter().collect();
        members.sort_by(|a, b| a.0.cmp(&b.0));
        let geometries = members
            .into_iter()
            .map(|(key, g)| {
                if g.cape.is_some() {
                    let at = JsonPath::new().child(&key).child("cape");
                    cx.diagnostics.push(
                        Diagnostic::info(DiagnosticCode::DroppedField {
                            text: text(CAPE_DROPPED, &[]),
                        })
                        .at(at)
                        .phase(Phase::Upgrade),
                    );
                }
                let (identifier, parent) = match key.rsplit_once(':') {
                    Some((identifier, parent)) => (identifier.to_owned(), Some(parent.to_owned())),
                    None => (key, None),
                };
                NewGeometry {
                    description: Description {
                        identifier,
                        visible_bounds_width: g.visible_bounds_width,
                        visible_bounds_height: g.visible_bounds_height,
                        visible_bounds_offset: g.visible_bounds_offset,
                        texture_width: g.texturewidth,
                        texture_height: g.textureheight,
                    },
                    cape: None,
                    item_display_transforms: None,
                    bones: g.bones.map(|bones| bones.into_iter().map(bone).collect()),
                    legacy: GeometryLegacy {
                        parent,
                        preserve_model_pose: g.legacy.preserve_model_pose,
                        debug: g.debug,
                    },
                    inherited: None,
                }
            })
            .collect();
        v1_12_0::GeometryFile {
            format_version: self.format_version,
            debug: self.debug,
            geometries,
        }
    }
}

fn bone(b: Bone) -> NewBone {
    NewBone {
        name: b.name,
        parent: b.parent,
        pivot: b.pivot,
        rotation: b.rotation,
        mirror: b.mirror,
        inflate: b.inflate,
        debug: b.debug,
        render_group_id: b.render_group_id,
        binding: None,
        cubes: b.cubes.map(|cubes| {
            cubes
                .into_iter()
                .map(|c| NewCube {
                    origin: c.origin,
                    size: c.size,
                    rotation: None,
                    pivot: None,
                    inflate: c.inflate,
                    mirror: c.mirror,
                    uv: c.uv.map(CubeUv::Box),
                })
                .collect()
        }),
        locators: b.locators,
        poly_mesh: b.poly_mesh,
        texture_meshes: b.texture_meshes,
        legacy: BoneLegacy {
            reset: b.reset,
            never_render: b.never_render,
            bind_pose_rotation: b.bind_pose_rotation,
        },
    }
}
