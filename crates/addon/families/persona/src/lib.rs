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
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

pub use load::{PackKind, PersonaPieces, PieceSource};
pub use names::{ArmSize, BodySize, Named, PieceSide, PieceType, Written};
pub use piece::{
    EMOTE_MOLANG_VERSION, EmoteAnimation, PersonaPiece, PieceAnimation, PieceGeometry,
};
pub use text::name_key;

/// The header UUID of the client's built-in persona pack, whose pieces are identified by
/// their own `piece_id`.
pub const BUILTIN_PACK: uuid::Uuid =
    uuid::Uuid::from_u128(0x2099_DE18_429A_465A_A49B_FC47_10A1_7BB3);

/// The header UUID of the client's testing persona pack (loaded only behind a feature toggle),
/// whose pieces are identified by their own `piece_id` too.
pub const TESTING_PACK: uuid::Uuid =
    uuid::Uuid::from_u128(0xE23C_0CD8_FCCA_4E17_A825_4956_0004_047E);

/// One `<name>.meta.json`: what a persona piece is, as the file says it.
///
/// `piece_id`, `piece_name` and `piece_type` are required, every other member is optional,
/// and none is invented when the piece is written back. The 1.26.40+ reader refuses a file
/// with a member it does not know or of a JSON type it does not expect; a model built by hand
/// may still hold such members in [`extra`](Self::extra), and they are written after the
/// known ones.
#[doc(alias = "PieceMeta")]
#[doc(alias = "PersonaPieceMeta")]
#[doc(alias = "meta.json")]
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct PieceMetadata {
    /// The `format_version` of a file written as a `minecraft:persona_piece_meta` document,
    /// as written (a file loads only at 1.26.40); `None` for the flat file every vanilla piece
    /// is.
    pub envelope: Option<Value>,
    /// `piece_id`: the piece's UUID, read only in the client's built-in packs.
    pub piece_id: Option<String>,
    /// `piece_name`: the name the piece's translation key is built from.
    pub piece_name: Option<String>,
    /// `piece_type`.
    pub piece_type: Option<Written<PieceType>>,
    /// `piece_sub_type`.
    pub piece_sub_type: Option<Written<PieceType>>,
    /// `body_variant`: the height a body or skeleton piece is for.
    pub body_variant: Option<Written<BodySize>>,
    /// `clothing_map`: an image in the piece's folder.
    pub clothing_map: Option<String>,
    /// `face_clothing_map`: an image in the piece's folder.
    pub face_clothing_map: Option<String>,
    /// `zone`: the body zones the piece claims.
    pub zone: Option<Zones>,
    /// `texture_sources`.
    pub texture_sources: Option<Vec<TextureSource>>,
    /// `geometry_sources`.
    pub geometry_sources: Option<Vec<GeometrySource>>,
    /// `animation_sources`: an emote's animations.
    pub animation_sources: Option<Vec<AnimationSource>>,
    /// `tint_color`.
    pub tint_color: Option<TintColorOrString>,
    /// `tint_base_color`.
    pub tint_base_color: Option<TintColorOrString>,
    /// `allow_tint_override`; absent means `true`
    /// ([`allows_tint_override`](Self::allows_tint_override)).
    pub allow_tint_override: Option<bool>,
    /// Members the game does not know, and members of a JSON type it does not expect, as
    /// written: never in a model read from a file that loads.
    pub extra: Extra,
}

/// One entry of `texture_sources`.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct TextureSource {
    /// `texture`: an image in the piece's folder; required. The texture builder skips an
    /// entry whose texture is empty.
    pub texture: Option<String>,
    /// `tint_map`: an image in the piece's folder.
    pub tint_map: Option<String>,
    /// `animated`; absent means `false`.
    pub animated: Option<bool>,
    /// `frames`, an integer of 32 bits; absent means 1. (A model built by hand may hold
    /// another spelling in `extra`; [`PieceMetadata::textures`] gives what the 1.26.20 reader
    /// made of it.)
    pub frames: Option<i32>,
    /// `use_face_uv`; absent means `false`.
    pub use_face_uv: Option<bool>,
    /// Members the game does not know, and members of an unexpected JSON type, as written:
    /// never in a model read from a file that loads.
    pub extra: Extra,
}

/// One entry of `geometry_sources`: a model of the piece's geometry file and when it is used.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct GeometrySource {
    /// `geometry`: a root member of the piece's `<name>.geometry.json`; required.
    pub geometry: Option<String>,
    /// `arm_size`; vanilla's files write `""` for "either".
    pub arm_size: Option<Written<ArmSize>>,
    /// `body_size`.
    pub body_size: Option<Written<BodySize>>,
    /// `side`.
    pub side: Option<Written<PieceSide>>,
    /// `zone`: the body zones the entry claims.
    pub zone: Option<Zones>,
    /// `is_face`: a string from 1.26.40, as vanilla's files write it (`"true"`).
    pub is_face: Option<FaceFlag>,
    /// `texture`: an image in the piece's folder.
    pub texture: Option<String>,
    /// `tint_map`: an image in the piece's folder.
    pub tint_map: Option<String>,
    /// `animated`; absent means `false`.
    pub animated: Option<bool>,
    /// `frames`, an integer of 32 bits; absent means 1.
    pub frames: Option<i32>,
    /// Members the game does not know (`use_face_uv` among them), and members of an unexpected
    /// JSON type, as written: never in a model read from a file that loads.
    pub extra: Extra,
}

/// One entry of `animation_sources`.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct AnimationSource {
    /// `name`: a key of the file's `animations`; required.
    pub name: Option<String>,
    /// `animationFile` (camel case in every build): a file in the piece's folder; required.
    pub animation_file: Option<String>,
    /// Members the game does not know, and members of an unexpected JSON type, as written:
    /// never in a model read from a file that loads.
    pub extra: Extra,
}

/// `zone` as written: one zone or a list of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Zones {
    /// A single string.
    One(String),
    /// An array of strings.
    Many(Vec<String>),
}

impl Zones {
    /// The zones, whichever way they were written.
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        match self {
            Self::One(zone) => std::slice::from_ref(zone),
            Self::Many(zones) => zones,
        }
    }

    /// Whether the member is "empty" to the game's reader: an empty list. A string never
    /// is, whatever it holds.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::One(_) => false,
            Self::Many(zones) => zones.is_empty(),
        }
    }
}

/// `is_face` as written: the string vanilla's own files use (the type from 1.26.40), or a
/// boolean in a model built by hand (1.26.20 read one; 1.26.40+ refuses it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FaceFlag {
    /// `true` or `false`.
    Bool(bool),
    /// A string; `"true"` reads as `true`, any other text as `false`.
    Text(String),
}

impl FaceFlag {
    /// What the flag says.
    #[must_use]
    pub fn value(&self) -> bool {
        match self {
            Self::Bool(flag) => *flag,
            Self::Text(text) => text == "true",
        }
    }
}

/// `tint_color` / `tint_base_color` as written: the four colour slots of the piece's tint
/// map, or one colour for all four.
#[derive(Clone, Debug, PartialEq)]
pub enum TintColorOrString {
    /// An object with a colour per slot.
    Channels(TintColor),
    /// One colour string, used for every slot.
    String(HexColor),
}

impl TintColorOrString {
    /// The colours of the four slots (red, green, blue, alpha channel of the tint map), as
    /// the game reads them: a slot without a colour is `None`.
    #[must_use]
    pub fn slots(&self) -> [Option<u32>; 4] {
        match self {
            Self::Channels(channels) => [
                channels.r_color.as_ref().and_then(HexColor::argb),
                channels.g_color.as_ref().and_then(HexColor::argb),
                channels.b_color.as_ref().and_then(HexColor::argb),
                channels.a_color.as_ref().and_then(HexColor::argb),
            ],
            Self::String(color) => [color.argb(); 4],
        }
    }
}

/// The colour slots of a tint: the channels of the piece's tint map each select one.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct TintColor {
    /// `r_color`.
    pub r_color: Option<HexColor>,
    /// `g_color`.
    pub g_color: Option<HexColor>,
    /// `b_color`.
    pub b_color: Option<HexColor>,
    /// `a_color`.
    pub a_color: Option<HexColor>,
    /// Members the game does not know, and members of an unexpected JSON type, as written:
    /// never in a model read from a file that loads.
    pub extra: Extra,
}

impl TintColor {
    /// The four colours (`r_color`, `g_color`, `b_color`, `a_color`) as the game holds them:
    /// the empty text for a colour left out, which is what its default-constructed
    /// `TintColor` holds.
    #[must_use]
    pub fn colors(&self) -> [&str; 4] {
        [&self.r_color, &self.g_color, &self.b_color, &self.a_color]
            .map(|color| color.as_ref().map_or("", HexColor::as_str))
    }
}

/// A colour as a file writes it: `#RRGGBB` or `#AARRGGBB`.
///
/// The text is kept as written. The game checks nothing: it drops the first character and
/// reads the rest as a base-16 number `0xAARRGGBB`, so `#RRGGBB` has alpha 0.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HexColor(pub String);

impl HexColor {
    /// The text as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the text is `#` and six or eight hexadecimal digits.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.0.strip_prefix('#').is_some_and(|digits| {
            matches!(digits.len(), 6 | 8) && digits.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
    }

    /// The colour the game reads, `0xAARRGGBB`: `None` for the empty text (no colour); else
    /// the hexadecimal digits that follow the first character, up to the first other
    /// character, and 0 when there is none.
    #[must_use]
    pub fn argb(&self) -> Option<u32> {
        let mut chars = self.0.chars();
        chars.next()?;
        let mut value: u32 = 0;
        for digit in chars.map_while(|c| c.to_digit(16)) {
            value = value.saturating_mul(16).saturating_add(digit);
        }
        Some(value)
    }
}

/// One `<name>.store_info.json`: how the Character Creator lists the piece.
#[doc(alias = "store_info.json")]
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct PieceStoreInfo {
    /// `piece_store_name`: replaces `piece_name` as the name the translation key is built
    /// from.
    pub piece_store_name: Option<String>,
    /// `piece_thumbnail`: an image, from the pack root, with or without its extension.
    pub piece_thumbnail: Option<String>,
    /// Members the game does not read, and members of an unexpected JSON type, as written.
    pub extra: Extra,
}

/// One texture the game builds for a piece: an entry of `texture_sources` or
/// `geometry_sources` that names an image, with the game's defaults filled in.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PieceTexture {
    /// The image, relative to the piece's folder.
    pub texture: String,
    /// The tint map, relative to the piece's folder.
    pub tint_map: Option<String>,
    /// Whether the texture is a strip of frames.
    pub animated: bool,
    /// How many frames.
    pub frames: i32,
    /// Whether the texture uses the face's UV layout; never set for a geometry source.
    pub use_face_uv: bool,
    /// The index of the `geometry_sources` entry the texture belongs to; `None` for an entry
    /// of `texture_sources`.
    pub geometry_source: Option<usize>,
}

impl PieceMetadata {
    /// The piece's type: [`PieceType::Unknown`] for a missing member and for any text outside
    /// the 25 names the game binds (a meta file with either is rejected when read; the model
    /// can still hold one built by hand).
    #[must_use]
    pub fn piece_type(&self) -> PieceType {
        self.piece_type
            .as_ref()
            .and_then(Written::known)
            .unwrap_or(PieceType::Unknown)
    }

    /// The piece's sub-type: `None` for a missing or empty member, [`PieceType::Unknown`] for
    /// any other text outside the game's names (a meta file with an unbound name, the empty
    /// text included, is rejected when read).
    #[must_use]
    pub fn piece_sub_type(&self) -> Option<PieceType> {
        let written = self.piece_sub_type.as_ref()?;
        if written.as_str().is_empty() {
            return None;
        }
        Some(written.known().unwrap_or(PieceType::Unknown))
    }

    /// The height the piece is for: [`BodySize::Medium`] for a missing, empty or unknown
    /// `body_variant`.
    #[must_use]
    pub fn body_variant(&self) -> BodySize {
        self.body_variant
            .as_ref()
            .and_then(Written::known)
            .filter(|size| *size != BodySize::Unspecified)
            .unwrap_or(BodySize::Medium)
    }

    /// `piece_id` as a UUID, when it is one.
    #[must_use]
    pub fn piece_uuid(&self) -> Option<uuid::Uuid> {
        let id = uuid::Uuid::parse_str(self.piece_id.as_deref()?).ok()?;
        (!id.is_nil()).then_some(id)
    }

    /// Whether the player may tint the piece: `allow_tint_override`, `true` when absent.
    #[must_use]
    pub fn allows_tint_override(&self) -> bool {
        self.allow_tint_override.unwrap_or(true)
    }

    /// Whether the Character Creator lets the player change the piece's colour: tinting is
    /// allowed and the piece is skin, facial hair, a mouth, eyes or hair. (1.26.20 also let a
    /// piece with `color_swatches` be recoloured; from 1.26.40 that member invalidates the
    /// piece, so no loaded piece has a swatch.)
    #[must_use]
    pub fn is_color_change_allowed(&self) -> bool {
        self.allows_tint_override() && self.piece_type().is_recolorable()
    }

    /// Whether the piece can be worn on one side: some geometry source has both a `side` and
    /// a `zone` (an empty list of zones is none).
    #[must_use]
    pub fn can_be_side_piece(&self) -> bool {
        self.geometry_sources.iter().flatten().any(|source| {
            source.side.is_some() && source.zone.as_ref().is_some_and(|zone| !zone.is_empty())
        })
    }

    /// The textures the game builds for the piece: every entry of `texture_sources`, then of
    /// `geometry_sources`, that names an image. An entry without a `texture`, or with an empty
    /// one, is skipped as the game skips it.
    #[must_use]
    pub fn textures(&self) -> Vec<PieceTexture> {
        let mut textures = Vec::new();
        for source in self.texture_sources.iter().flatten() {
            let Some(texture) = source
                .texture
                .as_ref()
                .filter(|texture| !texture.is_empty())
            else {
                continue;
            };
            textures.push(PieceTexture {
                texture: texture.clone(),
                tint_map: source.tint_map.clone(),
                animated: source.animated.unwrap_or(false),
                frames: frame_count(source.frames, &source.extra),
                use_face_uv: source.use_face_uv.unwrap_or(false),
                geometry_source: None,
            });
        }
        for (index, source) in self.geometry_sources.iter().flatten().enumerate() {
            let Some(texture) = source
                .texture
                .as_ref()
                .filter(|texture| !texture.is_empty())
            else {
                continue;
            };
            textures.push(PieceTexture {
                texture: texture.clone(),
                tint_map: source.tint_map.clone(),
                animated: source.animated.unwrap_or(false),
                frames: frame_count(source.frames, &source.extra),
                use_face_uv: false,
                geometry_source: Some(index),
            });
        }
        textures
    }
}

/// The frame count the game reads from an entry's `frames`: the typed member, or what
/// reading an integer with default 1 makes of a member held as written.
fn frame_count(frames: Option<i32>, extra: &Extra) -> i32 {
    frames.unwrap_or_else(|| read::as_int(extra.get("frames"), 1))
}

impl GeometrySource {
    /// The arm width the entry is for: `None` (either) for a missing or empty `arm_size`,
    /// [`ArmSize::Wide`] for a text outside the game's names.
    #[must_use]
    pub fn arm_size(&self) -> Option<ArmSize> {
        let written = self.arm_size.as_ref()?;
        if written.as_str().is_empty() {
            return None;
        }
        Some(written.known().unwrap_or(ArmSize::Wide))
    }

    /// The height the entry is for: `None` (any) for a missing `body_size`,
    /// [`BodySize::Medium`] for an empty text and for one outside the game's names.
    #[must_use]
    pub fn body_size(&self) -> Option<BodySize> {
        let written = self.body_size.as_ref()?;
        Some(
            written
                .known()
                .filter(|size| *size != BodySize::Unspecified)
                .unwrap_or(BodySize::Medium),
        )
    }

    /// Whether the entry uses the face's texture: `is_face`, `false` when absent.
    #[must_use]
    pub fn is_face(&self) -> Option<bool> {
        self.is_face.as_ref().map(FaceFlag::value)
    }
}
