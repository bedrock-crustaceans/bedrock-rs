//! How a block looks: `geometry`, `material_instances`, `transformation`, `item_visual`,
//! `embedded_visual`, `destruction_particles`, `map_color`.

use facet::Facet;
use indexmap::IndexMap;

use super::BONE_VISIBILITY;
use super::types::{Color, RenderMethod, TintMethod, Vec3};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::Unknown;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{self, Schema};

/// `minecraft:geometry`: the block's model.
///
/// JSON: a geometry identifier, or `{"identifier", "bone_visibility"?, "culling"?,
/// "culling_shape"?, "culling_layer"?, "uv_lock"?, "n_way_visual_rotation"?}`. A geometry with
/// `geometry` also needs `material_instances` (from 1.21.80).
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(
    clippy::large_enum_variant,
    reason = "a boxed object form would lose its unknown members"
)]
pub enum Geometry {
    /// The identifier alone.
    Identifier(String),
    /// The object form.
    Params(GeometryParams),
}

impl Geometry {
    /// The geometry identifier.
    pub fn identifier(&self) -> &str {
        match self {
            Geometry::Identifier(id) => id,
            Geometry::Params(params) => &params.identifier,
        }
    }
}

/// The object form of [`Geometry`].
#[derive(Clone, Debug, Default, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct GeometryParams {
    /// The geometry identifier (JSON `identifier`).
    pub identifier: String,
    /// Per-bone visibility: a bool or a Molang expression over `query.block_state` (JSON
    /// `bone_visibility`); at most 64 bones.
    #[facet(schema::molang = BONE_VISIBILITY)]
    pub bone_visibility: Option<IndexMap<String, Molang>>,
    /// The culling rules identifier (JSON `culling`).
    pub culling: Option<String>,
    /// The voxel shape used for culling (JSON `culling_shape`).
    pub culling_shape: Option<String>,
    /// The culling layer (JSON `culling_layer`).
    pub culling_layer: Option<String>,
    /// Lock the UVs of every bone, or of the named bones (JSON `uv_lock`; from 1.21.90).
    pub uv_lock: Option<UvLock>,
    /// Which states rotate the model about each axis (JSON `n_way_visual_rotation`; engine
    /// 1.26.30).
    pub n_way_visual_rotation: Option<NWayVisualRotation>,
}

impl GeometryParams {
    /// How many bones `bone_visibility` names; 0 when absent.
    pub fn bone_visibility_len(&self) -> usize {
        self.bone_visibility.as_ref().map_or(0, IndexMap::len)
    }
}

/// `geometry.uv_lock`: all bones or a list of bone names.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum UvLock {
    /// Every bone.
    All(bool),
    /// The named bones (at most 64).
    Bones(Vec<String>),
}

/// `geometry.n_way_visual_rotation`: the state that drives the rotation about each axis.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct NWayVisualRotation {
    /// JSON `x`.
    pub x: Option<String>,
    /// JSON `y`.
    pub y: Option<String>,
    /// JSON `z`.
    pub z: Option<String>,
}

impl MolangFields for Geometry {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Geometry::Params(params) = self {
            params.visit_molang(visitor);
        }
    }
}

// A geometry and the material instances are untagged and map-valued: leaves of the schema walk,
// whose bounds the family checks itself; the instances hold no Molang.
bedrock_addon_core::impl_schema_leaf!(Geometry, UvLock, MaterialInstances);
bedrock_addon_core::impl_no_molang!(UvLock, MaterialInstances);

/// `minecraft:material_instances`: the textures and render settings per face or bone.
///
/// JSON: an object of at most 64 instances keyed `*`, a face (`up`, `north`, …) or a name of
/// the geometry; a string value names another instance of the map.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(transparent)]
pub struct MaterialInstances(pub IndexMap<String, MaterialInstanceEntry>);

/// One entry of [`MaterialInstances`].
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum MaterialInstanceEntry {
    /// The name of another instance of the same map.
    Alias(String),
    /// An instance.
    Instance(MaterialInstance),
}

/// A material instance; every member optional.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct MaterialInstance {
    /// The `terrain_texture.json` short name (JSON `texture`).
    pub texture: Option<String>,
    /// JSON `render_method`; `opaque` when absent.
    pub render_method: Option<RenderMethod>,
    /// JSON `tint_method`; `none` when absent.
    pub tint_method: Option<TintMethod>,
    /// JSON `ambient_occlusion` (`0..=10`; 1 when absent).
    pub ambient_occlusion: Option<f32>,
    /// JSON `face_dimming`; `true` when absent.
    pub face_dimming: Option<bool>,
    /// Randomise the UV rotation (JSON `isotropic`; from 1.21.70); `false` when absent.
    pub isotropic: Option<bool>,
    /// Tint only where the texture's alpha is masked (JSON `alpha_masked_tint`; from 1.21.120);
    /// `false` when absent.
    pub alpha_masked_tint: Option<bool>,
    /// JSON `dithering`: base-game packs only.
    pub dithering: Option<bool>,
}

impl MaterialInstances {
    /// The JSON keys of [`MaterialInstance`].
    const KNOWN: [&'static str; 8] = [
        "texture",
        "render_method",
        "tint_method",
        "ambient_occlusion",
        "face_dimming",
        "isotropic",
        "alpha_masked_tint",
        "dithering",
    ];

    /// The instance `*` (or the one it names) renders opaque, as the game's render-layer check treats it.
    pub fn is_opaque(&self) -> bool {
        self.resolve("*")
            .is_none_or(|i| i.render_method.unwrap_or_default() == RenderMethod::Opaque)
    }

    /// The instance `name`, following aliases (at most 64 hops).
    pub fn resolve(&self, name: &str) -> Option<&MaterialInstance> {
        let mut name = name;
        for _ in 0..64 {
            match self.0.get(name)? {
                MaterialInstanceEntry::Instance(instance) => return Some(instance),
                MaterialInstanceEntry::Alias(next) => name = next,
            }
        }
        None
    }

    /// Keeps each instance's unknown members, from the JSON it was read from
    /// (`keep_unknown_members` does not reach map values), in `kept` below `at/<instance>`, and
    /// returns the unknown keys as `(instance, member)`.
    pub(crate) fn keep_unknown(
        &self,
        json: &Value,
        at: &JsonPath,
        kept: &mut Unknown,
    ) -> Vec<(String, String)> {
        let mut unknown = Vec::new();
        let Some(object) = json.as_object() else {
            return unknown;
        };
        for (name, entry) in &self.0 {
            let (MaterialInstanceEntry::Instance(_), Some(written)) =
                (entry, object.get(name.as_str()).and_then(Value::as_object))
            else {
                continue;
            };
            for (key, value) in written {
                if !Self::KNOWN.contains(&key.as_str()) {
                    kept.put(&at.child(name), key.as_str(), value.clone());
                    unknown.push((name.clone(), key.to_string()));
                }
            }
        }
        unknown
    }
}

/// `minecraft:transformation`: translation, scale and rotation of the model.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Transformation {
    /// JSON `translation`.
    pub translation: Option<Vec3>,
    /// JSON `scale`.
    pub scale: Option<Vec3>,
    /// JSON `scale_pivot`.
    pub scale_pivot: Option<Vec3>,
    /// JSON `rotation`; each axis a multiple of 90.
    pub rotation: Option<Vec3>,
    /// JSON `rotation_pivot`.
    pub rotation_pivot: Option<Vec3>,
}

/// `minecraft:item_visual`: how the block's item looks (geometry and materials, both required).
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ItemVisual {
    /// JSON `geometry`.
    pub geometry: Geometry,
    /// JSON `material_instances`.
    pub material_instances: MaterialInstances,
}

/// `minecraft:embedded_visual`: how the block looks inside another (a flower pot), geometry and
/// materials, both required.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct EmbeddedVisual {
    /// JSON `geometry`.
    pub geometry: Geometry,
    /// JSON `material_instances`.
    pub material_instances: MaterialInstances,
}

/// `minecraft:destruction_particles`: the particles shown while the block breaks.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct DestructionParticles {
    /// The texture (JSON `texture`); `""` when absent.
    pub texture: Option<String>,
    /// JSON `tint_method`.
    pub tint_method: Option<TintMethod>,
    /// How many particles (JSON `particle_count`; 100 when absent; from 1.21.90 with Upcoming
    /// Creator Features).
    pub particle_count: Option<u8>,
}

/// `minecraft:map_color`: the colour on a map.
///
/// JSON: a colour, or `{"color", "tint_method"?}`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
pub enum MapColor {
    /// A hex string.
    Hex(String),
    /// Three channels.
    Rgb([u8; 3]),
    /// The object form.
    Params(MapColorParams),
}

/// The object form of [`MapColor`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MapColorParams {
    /// JSON `color` (required).
    pub color: Color,
    /// JSON `tint_method`.
    pub tint_method: Option<TintMethod>,
}
