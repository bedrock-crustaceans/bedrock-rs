//! The `flipbooks` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites raised, so
//! a pin or a filter on the code's name keeps matching.
//!
//! The link rule (`bedrock_addon`'s `flipbooks_texture`) raises them; the game's debug assertions are
//! informations here.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// A flipbook's tile is not in the terrain atlas and its index is outside the `missing` tile's UV
/// sets; the argument is the tile.
pub static TILE_MISSING: CodeDef<1> = CodeDef::new(
    "flipbooks/tile_missing",
    Kind::UnresolvedReference,
    Class::Error,
    "flipbook tile {} is not in the terrain atlas",
)
.game("Texture {} is missing from atlas textures")
.was("UnresolvedReference");

/// A flipbook's index is outside its tile; the arguments are the tile, the index and the tile's
/// index count.
pub static BAD_ATLAS_INDEX: CodeDef<3> = CodeDef::new(
    "flipbooks/bad_atlas_index",
    Kind::InvalidValue,
    Class::Error,
    "flipbook tile {} has no atlas index {} (it has {})",
)
.game("Texture {} is using an invalid atlas index {} for the expected UV count {}")
.was("InvalidValue");

/// A flipbook's image is in no pack; the argument is the path.
pub static NO_METADATA: CodeDef<1> = CodeDef::new(
    "flipbooks/no_metadata",
    Kind::UnresolvedReference,
    Class::Lint,
    "flipbook image {} is in no pack",
)
.game("Texture metadata for texture: {} is invalid")
.was("TextureNotFound");

/// A flipbook image wider than tall (the trailing space is the game's).
pub static NO_FRAMES: CodeDef<0> = CodeDef::new(
    "flipbooks/no_frames",
    Kind::InvalidValue,
    Class::Lint,
    "flipbook image has no frame",
)
.game("Flipbook texture should have at least one frame. ")
.was("InvalidValue");

/// A frame index outside the image's frames.
pub static FRAME_OUT_OF_RANGE: CodeDef<0> = CodeDef::new(
    "flipbooks/frame_out_of_range",
    Kind::InvalidValue,
    Class::Lint,
    "flipbook frame index out of range",
)
.game("frame index out of range!")
.was("InvalidValue");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    TILE_MISSING.meta(),
    BAD_ATLAS_INDEX.meta(),
    NO_METADATA.meta(),
    NO_FRAMES.meta(),
    FRAME_OUT_OF_RANGE.meta(),
];
