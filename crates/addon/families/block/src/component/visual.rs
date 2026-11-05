//! How a block looks: `geometry`, `material_instances`, `transformation`, `item_visual`,
//! `embedded_visual`, `destruction_particles`, `map_color`.

use bedrock_addon_core::json::{StrictBool, StrictFloat, StrictInt, StrictString};
use facet::Facet;
use indexmap::IndexMap;

use super::types::{Color, RenderMethod, TintMethod, Vec3};
use super::{BONE_VISIBILITY, visit_geometry};
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::{Extra, Unknown};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{Molang, MolangFields, MolangVisitor};

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
            Geometry::Params(params) => &params.identifier.0,
        }
    }
}

/// The object form of [`Geometry`].
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct GeometryParams {
    /// The geometry identifier (JSON `identifier`).
    pub identifier: StrictString,
    /// Per-bone visibility: a bool or a Molang expression over `query.block_state` (JSON
    /// `bone_visibility`); at most 64 bones.
    #[facet(default)]
    pub bone_visibility: Option<IndexMap<String, Molang>>,
    /// The culling rules identifier (JSON `culling`).
    #[facet(default)]
    pub culling: Option<StrictString>,
    /// The voxel shape used for culling (JSON `culling_shape`).
    #[facet(default)]
    pub culling_shape: Option<StrictString>,
    /// The culling layer (JSON `culling_layer`).
    #[facet(default)]
    pub culling_layer: Option<StrictString>,
    /// Lock the UVs of every bone, or of the named bones (JSON `uv_lock`; from 1.21.90).
    #[facet(default)]
    pub uv_lock: Option<UvLock>,
    /// Which states rotate the model about each axis (JSON `n_way_visual_rotation`; engine
    /// 1.26.30).
    #[facet(default)]
    pub n_way_visual_rotation: Option<NWayVisualRotation>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
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
    Bones(Vec<StrictString>),
}

/// `geometry.n_way_visual_rotation`: the state that drives the rotation about each axis.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct NWayVisualRotation {
    /// JSON `x`.
    #[facet(default)]
    pub x: Option<StrictString>,
    /// JSON `y`.
    #[facet(default)]
    pub y: Option<StrictString>,
    /// JSON `z`.
    #[facet(default)]
    pub z: Option<StrictString>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MolangFields for Geometry {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Geometry::Params(params) = self
            && let Some(bones) = &mut params.bone_visibility
        {
            visitor.field("bone_visibility", &BONE_VISIBILITY, bones);
        }
    }
}

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
pub struct MaterialInstance {
    /// The `terrain_texture.json` short name (JSON `texture`).
    #[facet(default)]
    pub texture: Option<StrictString>,
    /// JSON `render_method`; `opaque` when absent.
    #[facet(default)]
    pub render_method: Option<RenderMethod>,
    /// JSON `tint_method`; `none` when absent.
    #[facet(default)]
    pub tint_method: Option<TintMethod>,
    /// JSON `ambient_occlusion` (`0..=10`; 1 when absent).
    #[facet(default)]
    pub ambient_occlusion: Option<StrictFloat<f32>>,
    /// JSON `face_dimming`; `true` when absent.
    #[facet(default)]
    pub face_dimming: Option<StrictBool>,
    /// Randomise the UV rotation (JSON `isotropic`; from 1.21.70); `false` when absent.
    #[facet(default)]
    pub isotropic: Option<StrictBool>,
    /// Tint only where the texture's alpha is masked (JSON `alpha_masked_tint`; from 1.21.120);
    /// `false` when absent.
    #[facet(default)]
    pub alpha_masked_tint: Option<StrictBool>,
    /// JSON `dithering`: base-game packs only.
    #[facet(default)]
    pub dithering: Option<StrictBool>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
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
pub struct Transformation {
    /// JSON `translation`.
    #[facet(default)]
    pub translation: Option<Vec3>,
    /// JSON `scale`.
    #[facet(default)]
    pub scale: Option<Vec3>,
    /// JSON `scale_pivot`.
    #[facet(default)]
    pub scale_pivot: Option<Vec3>,
    /// JSON `rotation`; each axis a multiple of 90.
    #[facet(default)]
    pub rotation: Option<Vec3>,
    /// JSON `rotation_pivot`.
    #[facet(default)]
    pub rotation_pivot: Option<Vec3>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:item_visual`: how the block's item looks (geometry and materials, both required).
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct ItemVisual {
    /// JSON `geometry`.
    pub geometry: Geometry,
    /// JSON `material_instances`.
    pub material_instances: MaterialInstances,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MolangFields for ItemVisual {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_geometry(visitor, &mut self.geometry);
    }
}

/// `minecraft:embedded_visual`: how the block looks inside another (a flower pot), geometry and
/// materials, both required.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct EmbeddedVisual {
    /// JSON `geometry`.
    pub geometry: Geometry,
    /// JSON `material_instances`.
    pub material_instances: MaterialInstances,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl MolangFields for EmbeddedVisual {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit_geometry(visitor, &mut self.geometry);
    }
}

/// `minecraft:destruction_particles`: the particles shown while the block breaks.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct DestructionParticles {
    /// The texture (JSON `texture`); `""` when absent.
    #[facet(default)]
    pub texture: Option<StrictString>,
    /// JSON `tint_method`.
    #[facet(default)]
    pub tint_method: Option<TintMethod>,
    /// How many particles (JSON `particle_count`; 100 when absent; from 1.21.90 with Upcoming
    /// Creator Features).
    #[facet(default)]
    pub particle_count: Option<StrictInt<u8>>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
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
    Rgb([StrictInt<u8>; 3]),
    /// The object form.
    Params(MapColorParams),
}

/// The object form of [`MapColor`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct MapColorParams {
    /// JSON `color` (required).
    pub color: Color,
    /// JSON `tint_method`.
    #[facet(default)]
    pub tint_method: Option<TintMethod>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}
