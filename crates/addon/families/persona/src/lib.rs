//! Persona pieces: the items of the Character Creator (skins, bodies, hair, clothing, eyes,
//! mouths, emotes).
//!
//! A piece is a directory of a persona pack: one `<name>.meta.json`
//! ([`PieceMetadata`]) and, beside it, an optional
//! `<name>.store_info.json` ([`PieceStoreInfo`]),
//! `<name>.geometry.json` ([`PieceGeometry`]), an emote's animation
//! file ([`EmoteAnimation`]), textures and a `texts/` folder. Only the
//! client reads them, from its built-in persona pack and from `persona_piece` packs of the
//! Marketplace cache: never from a world's resource packs, where a `pieces/` folder is dead
//! content. [`PersonaPiece::load_all`] reads one pack the way
//! the client does and reports what the client skips in silence.
//!
//! A meta file is read as the 1.26.40+ client reads it: one strict slice at exactly 1.26.40, so
//! an unknown member, a name outside a bound enum, a member of the wrong JSON type (`null`
//! included), a missing required member or another `format_version` makes the piece invalid
//! and the file is rejected. A file that loads is written back as it was read: the fields of
//! the structs here hold what a file says, the methods beside them what the game takes it for.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod load;
mod names;
mod piece;
mod read;
mod text;
mod types;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::document::{Content, Document, Rejected, unversioned};
use bedrock_addon_core::json::{EncodeError, WriteStyle, write_json};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use load::{meta_not_parsed, parse_file, store_info_not_parsed};
use read::{read_meta, read_store_info};

pub use load::PieceSource;
pub use names::{ArmSize, BodySize, Named, PieceSide, PieceType, Written};
pub use piece::{
    EMOTE_MOLANG_VERSION, EmoteAnimation, PersonaPiece, PieceAnimation, PieceGeometry,
};
pub use text::name_key;
pub use types::{
    AnimationSource, FaceFlag, GeometrySource, HexColor, PackKind, PersonaPieces, PieceMetadata,
    PieceStoreInfo, PieceTexture, TextureSource, TintColor, TintColorOrString, Zones,
};

/// The header UUID of the client's built-in persona pack, whose pieces are identified by
/// their own `piece_id`.
pub const BUILTIN_PACK: uuid::Uuid =
    uuid::Uuid::from_u128(0x2099_DE18_429A_465A_A49B_FC47_10A1_7BB3);

/// The header UUID of the client's testing persona pack (loaded only behind a feature toggle),
/// whose pieces are identified by their own `piece_id` too.
pub const TESTING_PACK: uuid::Uuid =
    uuid::Uuid::from_u128(0xE23C_0CD8_FCCA_4E17_A825_4956_0004_047E);

impl bedrock_addon_core::__private::Sealed for PieceMetadata {}
impl bedrock_addon_core::__private::Sealed for PieceStoreInfo {}

impl Content for PieceMetadata {
    const NAME: &'static str = "persona piece metadata";
    // A persona pack is on neither stack.
    const SIDES: &'static [PackSide] = &[];

    /// Reads one `<name>.meta.json` on its own, as the 1.26.40+ reader does. A file that is not
    /// JSON, one whose root is no object, and one the reader finds an error in is rejected:
    /// the client skips the piece.
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(unversioned::lone(path), bytes, meta_not_parsed, read_meta)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&self.to_value(), out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for PieceMetadata {}

impl Content for PieceStoreInfo {
    const NAME: &'static str = "persona piece store info";
    // A persona pack is on neither stack.
    const SIDES: &'static [PackSide] = &[];

    /// Reads one `<name>.store_info.json` on its own. A file that is not JSON, and one whose
    /// root is no object, is rejected: the client ignores the file (the piece loads without
    /// it, which [`PersonaPiece::load_all`] reports as a warning).
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(
            unversioned::lone(path),
            bytes,
            store_info_not_parsed,
            read_store_info,
        )
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&self.to_value(), out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for PieceStoreInfo {}
