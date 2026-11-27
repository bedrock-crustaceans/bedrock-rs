//! Format 1.8.0 up to 1.12.0: geometries as `"geometry.<name>"` members of the root, bones with
//! `parent` and `bind_pose_rotation`, `poly_mesh` and `texture_meshes`; and the step to 1.12,
//! which builds `minecraft:geometry[]`. The 1.8 schema's `format_version` maximum is 1.11.0,
//! and the game splits a geometry's key at its last `:` into the identifier and the parent.

use facet::Facet;
use indexmap::IndexMap;

use super::super::codes;
use super::super::walk::{LOCATORS, floats, int, read_payload, read_root, render_group_id, texels};
use super::super::{
    Bone as NewBone, BoneLegacy, Cube as NewCube, CubeUv, Description, Geometry as NewGeometry,
    GeometryLegacy, Locator, PolyMesh, TextureMesh,
};
use super::v1_12_0;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Phase, Span};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::molang::MolangVisitor;
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::version::{FormatVersion, known};
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
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = none)]
pub struct Geometry {
    /// `debug`.
    pub debug: Option<bool>,
    /// `visible_bounds_width`.
    pub visible_bounds_width: Option<f32>,
    /// `visible_bounds_height`.
    pub visible_bounds_height: Option<f32>,
    /// `visible_bounds_offset`.
    #[facet(schema::read_with = floats)]
    pub visible_bounds_offset: Option<[f32; 3]>,
    /// `texturewidth`.
    #[facet(schema::read_with = int)]
    pub texturewidth: Option<i32>,
    /// `textureheight`.
    #[facet(schema::read_with = int)]
    pub textureheight: Option<i32>,
    /// `cape`: lost by the step to 1.12.
    pub cape: Option<String>,
    /// `bones`.
    pub bones: Option<Vec<Bone>>,
    /// What the step from the older shape carried over (`preserve_model_pose`); never read
    /// from a 1.8 file.
    #[facet(skip)]
    pub legacy: GeometryLegacy,
}

/// One bone of a 1.8 file.
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = none)]
pub struct Bone {
    /// `name`.
    pub name: String,
    /// `reset`.
    pub reset: Option<bool>,
    /// `neverRender`.
    #[facet(rename = "neverRender")]
    pub never_render: Option<bool>,
    /// `parent`.
    pub parent: Option<String>,
    /// `pivot`.
    #[facet(schema::read_with = floats)]
    pub pivot: Option<[f32; 3]>,
    /// `rotation`.
    #[facet(schema::read_with = floats)]
    pub rotation: Option<[f32; 3]>,
    /// `bind_pose_rotation`.
    #[facet(schema::read_with = floats)]
    pub bind_pose_rotation: Option<[f32; 3]>,
    /// `mirror`.
    pub mirror: Option<bool>,
    /// `inflate`.
    pub inflate: Option<f32>,
    /// `debug`.
    pub debug: Option<bool>,
    /// `render_group_id`.
    #[facet(schema::read_with = render_group_id)]
    pub render_group_id: Option<u32>,
    /// `cubes`.
    pub cubes: Option<Vec<Cube>>,
    /// `locators`: a 1.8 file holds vectors only; the object form is here only when the step
    /// from the older shape carried it over.
    #[facet(schema::keys = LOCATORS)]
    pub locators: Option<IndexMap<String, Locator>>,
    /// `poly_mesh`.
    pub poly_mesh: Option<PolyMesh>,
    /// `texture_meshes`.
    pub texture_meshes: Option<Vec<TextureMesh>>,
}

/// One cube of a 1.8 file.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::molang_with = none)]
pub struct Cube {
    /// `origin`.
    #[facet(schema::read_with = floats)]
    pub origin: Option<[f32; 3]>,
    /// `size`.
    #[facet(schema::read_with = floats)]
    pub size: Option<[f32; 3]>,
    /// `uv`.
    #[facet(schema::read_with = texels)]
    pub uv: Option<[f32; 2]>,
    /// `inflate`.
    pub inflate: Option<f32>,
    /// `mirror`.
    pub mirror: Option<bool>,
}

// ---- reading -------------------------------------------------------------------------------------

/// The Molang walk of a type of this era: none, it holds no Molang.
fn none<T>(_: &mut T, _: &mut MolangVisitor<'_>) {}

impl Versioned for GeometryFile {
    const VERSION: FormatVersion = known::V1_8_0;
}

bedrock_addon_core::impl_no_molang!(GeometryFile);

impl VersionStruct for GeometryFile {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut found = Diagnostics::default();
        if cx.original_version > MAX_FORMAT_VERSION {
            // The version node's maximum: 1.11.1 up to 1.12.0 fails the schema.
            let mut finding = Diagnostic::error(codes::ABOVE_MAX_VERSION.with([
                cx.original_version.to_string(),
                MAX_FORMAT_VERSION.to_string(),
            ]))
            .at(cx.at.child("format_version"));
            finding.location.span = Some(Span { offset: 0, len: 0 });
            found.push(finding);
        }
        let read = read_payload(payload, cx, &mut found, read_root::<Geometry>);
        let failed = found.has_errors();
        d.extend(found);
        let (debug, geometries) = read.filter(|_| !failed)?;
        Some(Self {
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
                        Diagnostic::of(&codes::CAPE_DROPPED)
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
