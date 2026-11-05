//! The client's `blocks.json` ([`BlocksJson`], [`ClientBlock`]): what the client draws and plays
//! for each block, one file at the root of a resource pack.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

mod entry;
mod legacy;
mod link;
mod load;
mod new;
mod resolve;
mod shape;
mod tables;
mod texts;
mod version;
mod write;

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // the clock, to measure
mod cost;
#[cfg(test)]
mod tests;

use bedrock_addon_core::document::Extra;

#[allow(unused_imports, reason = "the link stage reads them")]
#[doc(hidden)]
pub use link::{entry_members, entry_settings};
#[doc(hidden)]
pub use load::LOADER;

pub use history::v1_1_0::BlocksJson;
pub use shape::{BLOCK_SHAPES, block_shape_value, default_block_shape};
pub use version::{BLOCKS_JSON_DEFAULT_VERSION, BLOCKS_JSON_MAX_FORMAT_VERSION};

/// What the client draws and plays for one block: one entry of a resource pack's `blocks.json`,
/// or, in a stack's registry, the entries of every pack merged field by field (a higher pack's
/// field replaces a lower pack's; a field no pack sets is `None`).
///
/// JSON: `"<block name>": {"textures", "carried_textures", "isotropic", "blockshape",
/// "ambient_occlusion_exponent", "sound"}`; every member is optional and no other member is
/// read. A field whose value the client refuses is `None` here, with a finding.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "blocks.json")]
#[doc(alias = "BlockGraphics")]
pub struct ClientBlock {
    /// The texture of each face, as keys of `terrain_texture.json` (JSON `textures`): one name
    /// for every face, `{up, down, side}`, or all six faces. `name:N` picks element `N` of the
    /// atlas entry. A block without textures is drawn with `missing_tile`.
    pub textures: Option<FaceTextures>,
    /// The textures of the block as an item (JSON `carried_textures`), same forms; a block
    /// without them is carried with its `textures`.
    pub carried_textures: Option<FaceTextures>,
    /// Which faces are drawn with a randomly rotated texture (JSON `isotropic`).
    pub isotropic: Option<Isotropic>,
    /// The shape the block is drawn with, one of [`BLOCK_SHAPES`] (JSON `blockshape`). Only a
    /// vanilla block takes one, and only from a pack that is not the base game's.
    pub blockshape: Option<String>,
    /// The exponent of the block's ambient occlusion (JSON `ambient_occlusion_exponent`);
    /// [`ambient_occlusion_exponent`](Self::ambient_occlusion_exponent) gives the client's 1.0
    /// when no pack sets it. A boolean is read as 1.0 or 0.0.
    pub ambient_occlusion_exponent: Option<f32>,
    /// The block's sound type, a key of `sounds.json` `block_sounds` (JSON `sound`);
    /// [`sound`](Self::sound) gives the client's `normal` when no pack sets it. A value that
    /// is not a string is read as `"true"`, `"false"` or `""`.
    pub sound: Option<String>,
    /// The members the client never reads (`brightness_gamma`, `sounds`, `client_entity`, …).
    pub extra: Extra,
}

/// The textures of a block's six faces, in one of the three forms `blocks.json` has.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FaceTextures {
    /// One texture for every face (JSON: a string).
    All(String),
    /// JSON: an object with exactly `up`, `down` and `side`; `side` is the four side faces.
    UpDownSide {
        /// The top face.
        up: String,
        /// The bottom face.
        down: String,
        /// The north, south, west and east faces.
        side: String,
    },
    /// JSON: an object with exactly the six faces.
    SixFaces {
        /// The top face.
        up: String,
        /// The bottom face.
        down: String,
        /// The north face.
        north: String,
        /// The south face.
        south: String,
        /// The west face.
        west: String,
        /// The east face.
        east: String,
    },
}

/// Which faces of a block have an isotropic (randomly rotated) texture.
///
/// The client reads a boolean, or an object of face names: with a `side` member that is not
/// `null` it reads `up`, `down` and `side`, otherwise the six faces. A face is isotropic only
/// when its member is `true`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Isotropic {
    /// Every face, or none (JSON: a boolean).
    All(bool),
    /// JSON: an object whose `side` is not `null`.
    UpDownSide(FaceBools3),
    /// JSON: an object without `side` (or with `"side": null`).
    SixFaces(FaceBools6),
}

/// The faces of [`Isotropic::UpDownSide`]: each is `true` only when its member is `true`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FaceBools3 {
    /// The top face (JSON `up`).
    pub up: bool,
    /// The bottom face (JSON `down`).
    pub down: bool,
    /// The four side faces (JSON `side`).
    pub side: bool,
}

/// The faces of [`Isotropic::SixFaces`]: each is `true` only when its member is `true`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "one flag per face, as the JSON object has one member per face"
)]
pub struct FaceBools6 {
    /// The top face (JSON `up`).
    pub up: bool,
    /// The bottom face (JSON `down`).
    pub down: bool,
    /// The north face (JSON `north`).
    pub north: bool,
    /// The south face (JSON `south`).
    pub south: bool,
    /// The west face (JSON `west`).
    pub west: bool,
    /// The east face (JSON `east`).
    pub east: bool,
}
