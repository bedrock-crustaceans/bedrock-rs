//! Geometry: `Geometry`, `GeometryFile`, `Bone`, `Cube`, `Locator`.
//!
//! Model geometry of a resource pack: `models/mobs.json`, `models/entity/**/*.json` and
//! `models/blocks/**/*.json`. Vanilla reads these with a loader of its own: the file is validated once against the schema of its own
//! `format_version`, upgraded in place to the newest shape, and each geometry is registered by
//! its identifier.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

mod load;
mod molang;
mod new;
mod read;
mod schema;
mod validate;
mod write;

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

use indexmap::IndexMap;

use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::version::FormatVersion;

#[doc(hidden)]
pub use load::LOADER;
pub use load::{load_file, load_keyed_file};
pub use new::ItemDisplayTransformsIter;

/// A geometry file (`models/entity/*.json`, `models/blocks/*.json`, `models/mobs.json`):
/// `{"format_version", "minecraft:geometry": [{…}, …]}`.
///
/// One file holds any number of geometries. A file older than 1.12.0 holds them as
/// `"geometry.<name>": {…}` members of the root instead; loading upgrades it to this shape
/// ([`history`]). A file from 1.19.30 on is rejected for any member its schema does not declare;
/// an older file loses such a member with an information.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "models")]
#[doc(alias = "minecraft:geometry")]
pub struct GeometryFile {
    /// `format_version`: the version the file was written in. Vanilla keeps it with every
    /// geometry of the file: `embedded` needs 1.21.110 and `gui.fit_to_frame: false` changes
    /// meaning at 1.21.130. The writer writes the newest shape at 1.26.40 for an older file,
    /// except a file whose geometries or bones carry legacy members (`__legacy`): that one is
    /// written in the shape and at the version it was read in, and a member that shape cannot
    /// hold is an `EncodeError::NotRepresentable` at the member.
    pub format_version: FormatVersion,
    /// `debug`: a developer dump switch without effect on the content.
    pub debug: Option<bool>,
    /// `minecraft:geometry`: the geometries, in file order (at least one in a file that loads).
    pub geometries: Vec<Geometry>,
}

/// One geometry (`minecraft:geometry[i]`): a description and its bones.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    /// `description`.
    pub description: Description,
    /// `cape`: read by the schema and by nothing else.
    pub cape: Option<String>,
    /// `item_display_transforms` (from 1.21.0): how the geometry is posed when a block item is
    /// shown in the hand, the GUI, on the ground, in a frame, on a head or on a shelf.
    pub item_display_transforms: Option<ItemDisplayTransforms>,
    /// `bones`.
    pub bones: Option<Vec<Bone>>,
    /// `__legacy`: what an upgraded file older than 1.12.0 carried that the newest schema has
    /// no member for. Empty for a file written at 1.12.0 or later, and then not written.
    pub legacy: GeometryLegacy,
    /// The geometries this one inherits bones from (`legacy.parent` and its parents), filled
    /// when a pack stack is loaded. Never read from a file and never written.
    pub inherited: Option<Arc<Inherited>>,
}

/// `description` of a geometry.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Description {
    /// `identifier`: `geometry.<name>`. A namespace (`minecraft:geometry.<name>`) is accepted
    /// only in the base game's packs, from 1.19.30.
    pub identifier: String,
    /// `visible_bounds_width`: clamped to 0.1–50 when read; 1 when absent
    /// ([`visible_bounds_width`](Self::visible_bounds_width)).
    pub visible_bounds_width: Option<f32>,
    /// `visible_bounds_height`: clamped to 0.1–50 when read; 1 when absent.
    pub visible_bounds_height: Option<f32>,
    /// `visible_bounds_offset`: (0, 0, 0) when absent.
    pub visible_bounds_offset: Option<[f32; 3]>,
    /// `texture_width`: 64 when absent.
    pub texture_width: Option<i32>,
    /// `texture_height`: 64 when absent.
    pub texture_height: Option<i32>,
}

/// `__legacy` of a geometry: members of a file older than 1.12.0 that vanilla still reads after
/// its upgrade but that no later schema declares.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GeometryLegacy {
    /// The geometry this one inherits from: the part after the last `:` of a legacy key
    /// `geometry.child:geometry.parent`.
    pub parent: Option<String>,
    /// `preserve_model_pose` of a file older than 1.8.0.
    pub preserve_model_pose: Option<bool>,
    /// The geometry-level `debug` of a 1.8 file (vanilla copies it into `description`, where no
    /// later schema declares it).
    pub debug: Option<bool>,
}

impl GeometryLegacy {
    /// Nothing was carried over.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// One bone (`bones[i]`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bone {
    /// `name`, lower-cased as vanilla keys bones.
    pub name: String,
    /// `parent`, lower-cased: a bone of the same geometry.
    pub parent: Option<String>,
    /// `pivot`.
    pub pivot: Option<[f32; 3]>,
    /// `rotation`, in degrees.
    pub rotation: Option<[f32; 3]>,
    /// `mirror`: the default of the bone's cubes; `false` when absent.
    pub mirror: Option<bool>,
    /// `inflate`: the default of the bone's cubes; 0 when absent.
    pub inflate: Option<f32>,
    /// `debug`.
    pub debug: Option<bool>,
    /// `render_group_id`: at least 0.
    pub render_group_id: Option<u32>,
    /// `binding` (from 1.16.0): a Molang expression naming the bone this one follows, compiled
    /// at the pack's `min_engine_version`.
    pub binding: Option<Molang>,
    /// `cubes`.
    pub cubes: Option<Vec<Cube>>,
    /// `locators`, by name.
    pub locators: Option<IndexMap<String, Locator>>,
    /// `poly_mesh`.
    pub poly_mesh: Option<PolyMesh>,
    /// `texture_meshes`.
    pub texture_meshes: Option<Vec<TextureMesh>>,
    /// `__legacy`: members of a file older than 1.12.0; empty otherwise, and then not written.
    pub legacy: BoneLegacy,
}

/// `__legacy` of a bone.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoneLegacy {
    /// `reset`.
    pub reset: Option<bool>,
    /// `neverRender`.
    pub never_render: Option<bool>,
    /// `bind_pose_rotation` of a 1.8 file.
    pub bind_pose_rotation: Option<[f32; 3]>,
}

impl BoneLegacy {
    /// Nothing was carried over.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// One cube (`cubes[i]`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cube {
    /// `origin`: the corner with the lowest coordinates.
    pub origin: Option<[f32; 3]>,
    /// `size`.
    pub size: Option<[f32; 3]>,
    /// `rotation` in degrees, applied x, then y, then z, around [`pivot`](Self::pivot).
    pub rotation: Option<[f32; 3]>,
    /// `pivot`: the cube's centre when absent ([`pivot`](Self::pivot)).
    pub pivot: Option<[f32; 3]>,
    /// `inflate`: the bone's when absent.
    pub inflate: Option<f32>,
    /// `mirror`: the bone's when absent.
    pub mirror: Option<bool>,
    /// `uv`.
    pub uv: Option<CubeUv>,
}

/// `uv` of a cube.
#[derive(Clone, Debug, PartialEq)]
pub enum CubeUv {
    /// `[u, v]`: the box layout from that texel. Vanilla reads whole texels; a fraction is cut
    /// off when the file is read.
    Box([f32; 2]),
    /// `{"north": {…}, …}`: one rectangle per face; a face left out is not drawn.
    Faces(Box<Faces>),
}

/// The per-face form of a cube's `uv`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Faces {
    /// `north`.
    pub north: Option<FaceUv>,
    /// `south`.
    pub south: Option<FaceUv>,
    /// `east`.
    pub east: Option<FaceUv>,
    /// `west`.
    pub west: Option<FaceUv>,
    /// `up`.
    pub up: Option<FaceUv>,
    /// `down`.
    pub down: Option<FaceUv>,
}

/// One face of a cube's `uv`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FaceUv {
    /// `uv`.
    pub uv: [f32; 2],
    /// `uv_size`: the face's size on the cube when absent.
    pub uv_size: Option<[f32; 2]>,
    /// `uv_rotation` (from 1.21.0): 0, 90, 180 or 270; any other value is reported and read as
    /// absent.
    pub uv_rotation: Option<i32>,
    /// `material_instance`: the block material instance drawn on this face.
    pub material_instance: Option<String>,
}

/// One locator (`locators.<name>`).
#[derive(Clone, Debug, PartialEq)]
pub enum Locator {
    /// `[x, y, z]`.
    Position([f32; 3]),
    /// `{"offset", "rotation", "ignore_inherited_scale"}`.
    Placed {
        /// `offset`.
        offset: [f32; 3],
        /// `rotation`: required from 1.12.0, optional in a file older than 1.8.0.
        rotation: Option<[f32; 3]>,
        /// `ignore_inherited_scale`.
        ignore_inherited_scale: Option<bool>,
    },
}

/// `poly_mesh` of a bone (from 1.8.0).
#[derive(Clone, Debug, PartialEq)]
pub struct PolyMesh {
    /// `normalized_uvs`: UVs are 0–1 instead of texels.
    pub normalized_uvs: Option<bool>,
    /// `positions`.
    pub positions: Option<Vec<[f32; 3]>>,
    /// `normals`.
    pub normals: Option<Vec<[f32; 3]>>,
    /// `uvs`.
    pub uvs: Option<Vec<[f32; 2]>>,
    /// `polys`.
    pub polys: Polys,
}

/// `polys` of a poly mesh.
#[derive(Clone, Debug, PartialEq)]
pub enum Polys {
    /// Polygons of three or four vertices, each vertex an index into positions, normals and
    /// UVs.
    Indices(Vec<Vec<[u32; 3]>>),
    /// `"tri_list"` or `"quad_list"` (vanilla searches the value for either word): the lists
    /// are consumed three or four entries at a time.
    List(String),
}

/// One texture mesh (`texture_meshes[i]`, from 1.8.0).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureMesh {
    /// `texture`.
    pub texture: String,
    /// `position`: (0, 0, 0) when absent.
    pub position: Option<[f32; 3]>,
    /// `local_pivot`: (0, 0, 0) when absent.
    pub local_pivot: Option<[f32; 3]>,
    /// `rotation`: (0, 0, 0) when absent.
    pub rotation: Option<[f32; 3]>,
    /// `scale`: (1, 1, 1) when absent.
    pub scale: Option<[f32; 3]>,
    /// `use_pixel_depth`: `true` when absent.
    pub use_pixel_depth: Option<bool>,
}

/// `item_display_transforms` (from 1.21.0).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemDisplayTransforms {
    /// `gui`.
    pub gui: Option<ItemDisplayTransform>,
    /// `firstperson_righthand`.
    pub firstperson_righthand: Option<ItemDisplayTransform>,
    /// `firstperson_lefthand`.
    pub firstperson_lefthand: Option<ItemDisplayTransform>,
    /// `thirdperson_righthand`.
    pub thirdperson_righthand: Option<ItemDisplayTransform>,
    /// `thirdperson_lefthand`.
    pub thirdperson_lefthand: Option<ItemDisplayTransform>,
    /// `ground`.
    pub ground: Option<ItemDisplayTransform>,
    /// `fixed`.
    pub fixed: Option<ItemDisplayTransform>,
    /// `head`.
    pub head: Option<ItemDisplayTransform>,
    /// `embedded`: read only in a file of 1.21.110 or later.
    pub embedded: Option<ItemDisplayTransform>,
    /// `shelf` (from 1.26.40); an older file gets a copy of `fixed`.
    pub shelf: Option<ItemDisplayTransform>,
}

/// One transform of `item_display_transforms`. A vector outside its range is reported and read
/// as absent.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemDisplayTransform {
    /// `translation`: −80 to 80 on each axis.
    pub translation: Option<[f32; 3]>,
    /// `rotation`: −360 to 360.
    pub rotation: Option<[f32; 3]>,
    /// `scale`: 0 to 4.
    pub scale: Option<[f32; 3]>,
    /// `rotation_pivot`: −80 to 80.
    pub rotation_pivot: Option<[f32; 3]>,
    /// `scale_pivot`: −80 to 80.
    pub scale_pivot: Option<[f32; 3]>,
    /// `fit_to_frame`: declared for `gui` only; `true` when absent.
    pub fit_to_frame: Option<bool>,
}

/// How a `gui` transform fits the item into its slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FitToFrame {
    /// `fit_to_frame` is `true` or absent: scaled to fit.
    Fit,
    /// `fit_to_frame: false` from 1.21.130: not scaled, centred.
    Unscaled,
    /// `fit_to_frame: false` below 1.21.130: the older behaviour without a fit.
    Legacy,
}

/// The bones a geometry inherits: those of its legacy parent, and that parent's own
/// inheritance.
///
/// Only a file older than 1.12.0 can name a parent. The chain is as long as the packs make it;
/// it is walked, compared and dropped without recursion.
///
/// Two values are equal when they name the same geometries in the same order: equality compares
/// the identifiers along the chain (the same allocation is equal at once), not the bones, which
/// the geometries of a registry compare themselves. `Debug` prints one link (its identifier, how
/// many bones it has and how long its chain is), so the text of a registry does not repeat a
/// chain once per geometry below it.
#[derive(Clone)]
pub struct Inherited {
    identifier: String,
    bones: Vec<Bone>,
    parent: Option<Arc<Inherited>>,
    /// The links of the chain, this one included.
    depth: usize,
}

impl Inherited {
    /// The parent `identifier` with its own `bones` and what it inherits itself.
    pub fn new(identifier: impl Into<String>, bones: Vec<Bone>, parent: Option<Arc<Self>>) -> Self {
        Self {
            identifier: identifier.into(),
            bones,
            depth: parent.as_ref().map_or(1, |parent| parent.depth + 1),
            parent,
        }
    }

    /// The parent geometry's identifier.
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// The parent geometry's own bones.
    pub fn bones(&self) -> &[Bone] {
        &self.bones
    }

    /// What the parent inherits itself.
    pub fn parent(&self) -> Option<&Inherited> {
        self.parent.as_deref()
    }

    /// How many geometries the chain holds, this parent included.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// This parent, then its parent, and so on.
    pub fn chain(&self) -> impl Iterator<Item = &Inherited> {
        std::iter::successors(Some(self), |node| node.parent())
    }
}

impl Drop for Inherited {
    /// Frees the chain link by link: the derived drop would recurse once per ancestor.
    fn drop(&mut self) {
        let mut next = self.parent.take();
        while let Some(node) = next {
            match Arc::try_unwrap(node) {
                Ok(mut node) => next = node.parent.take(),
                Err(_) => break,
            }
        }
    }
}

#[allow(
    clippy::missing_fields_in_debug,
    reason = "the links above are counted, not printed: each geometry below would repeat them"
)]
impl fmt::Debug for Inherited {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Inherited")
            .field("identifier", &self.identifier)
            .field("bones", &self.bones.len())
            .field("chain", &self.depth)
            .finish()
    }
}

impl Eq for Inherited {}

impl PartialEq for Inherited {
    fn eq(&self, other: &Self) -> bool {
        if self.depth != other.depth {
            return false;
        }
        let (mut a, mut b) = (Some(self), Some(other));
        while let (Some(x), Some(y)) = (a, b) {
            if std::ptr::eq(x, y) {
                return true;
            }
            if x.identifier != y.identifier {
                return false;
            }
            a = x.parent();
            b = y.parent();
        }
        a.is_none() && b.is_none()
    }
}
