//! The client's `blocks.json` ([`BlocksJson`], [`ClientBlock`]): what the client draws and plays
//! for each block, one file at the root of a resource pack.
//!
//! The reader stays hand-walked, not on the reader kit (`json::kit`): the client applies each
//! member of an entry over the block it already has, testing the value's JSON type itself, and
//! every finding is that apply's (told in `Phase::Validate`, most with the client's text for
//! the block), where the kit's wrong-type, missing and unknown findings are a parse's.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

mod entry;
mod legacy;
mod link;
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

use bedrock_addon_core::document::UnknownMembers;

#[allow(unused_imports, reason = "the link stage reads them")]
#[doc(hidden)]
pub use link::{entry_members, entry_settings};

pub use history::v1_1_0::BlocksJson;
pub use shape::{BLOCK_SHAPES, block_shape_value, default_block_shape};
use std::collections::HashMap;
pub use version::{BLOCKS_JSON_DEFAULT_VERSION, BLOCKS_JSON_MAX_FORMAT_VERSION};

use crate::resolve::{Applied, FileRead};
use crate::shape::is_refused;
use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Origin, Placed, Registry,
    RegistryBuilder, Rejected, VersionInfo,
};
use bedrock_addon_core::id::{BlockId, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{Accept, CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;

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
    pub extra: UnknownMembers,
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

/// The one path the client reads.
const BLOCKS_JSON: &str = "blocks.json";

impl bedrock_addon_core::__private::Sealed for BlocksJson {}
impl bedrock_addon_core::__private::Sealed for ClientBlock {}

impl Content for BlocksJson {
    const NAME: &'static str = "blocks.json";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// The file as held, with the findings of reading it as a pack of `cx` would. A file the
    /// client skips (not a JSON object) is rejected with an error: nothing else is refused.
    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), PackSide::Resource, path.clone());
        let FileRead {
            file,
            effective,
            mut diagnostics,
            applied,
        } = resolve::read(bytes, cx.is_base_game_pack);
        for application in applied {
            diagnostics.extend(application.diagnostics);
        }
        diagnostics.locate(&origin);
        match file {
            Some(file) => {
                let version = version_info(&file, effective.version);
                Ok(Document::new(file, origin, version, diagnostics))
            }
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for BlocksJson {}

impl Definition for ClientBlock {
    type Kind = kind::Block;
    type File = BlocksJson;
    type Beta = NoBeta<kind::Block>;
}

impl Placed for BlocksJson {
    /// `blocks.json`, at the pack root: the only place the client reads it from.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(BLOCKS_JSON)
    }
}

/// The version record of a file read as `effective`: the version as written when it is a whole
/// one, the one struct (1.1.0) as the parser.
fn version_info(file: &BlocksJson, effective: FormatVersion) -> VersionInfo {
    let written =
        file.format_version
            .as_ref()
            .and_then(|value| match crate::version::from_json(value) {
                crate::version::Match::Full(version) => Some(version.version),
                _ => None,
            });
    VersionInfo::new(
        written,
        false,
        effective,
        BLOCKS_JSON_DEFAULT_VERSION,
        false,
    )
}

/// The root file of every pack, lowest priority first: the client visits the stack from its last
/// entry down to its first and reads at most one file per entry.
const BLOCKS_WALK: Walk = Walk::json_children_in(&[""]).accept(Accept::Exact(&[BLOCKS_JSON]));

/// One pack's file as read.
#[derive(Clone, Debug)]
#[doc(hidden)]
pub struct ParsedBlocks {
    /// The file's origin.
    pub(crate) origin: Origin,
    /// What reading it gave.
    pub(crate) read: FileRead,
}

/// Reads one walked file.
fn parse_blocks_file(file: &WalkFile, cx: &LoadContext) -> ParsedBlocks {
    ParsedBlocks {
        origin: Origin::file(file.pack.clone(), file.side, file.path.clone()),
        read: resolve::read(&file.bytes, cx.is_base_game_pack),
    }
}

/// `higher` written over `lower`, field by field: a field the higher entry sets replaces the
/// lower one whole (`textures` all six faces at once, `isotropic` the whole mask), as the client
/// applies each entry over the block it already has.
fn merge(lower: &mut ClientBlock, higher: ClientBlock) {
    let ClientBlock {
        textures,
        carried_textures,
        isotropic,
        blockshape,
        ambient_occlusion_exponent,
        sound,
        extra,
    } = higher;
    if textures.is_some() {
        lower.textures = textures;
    }
    if carried_textures.is_some() {
        lower.carried_textures = carried_textures;
    }
    if isotropic.is_some() {
        lower.isotropic = isotropic;
    }
    if blockshape.is_some() {
        lower.blockshape = blockshape;
    }
    if ambient_occlusion_exponent.is_some() {
        lower.ambient_occlusion_exponent = ambient_occlusion_exponent;
    }
    if sound.is_some() {
        lower.sound = sound;
    }
    for (name, value) in extra.iter() {
        lower.extra.insert(name.to_owned(), value.clone());
    }
}

/// The registry: the client's replay. Files lowest pack first, each file's entries in bytewise
/// key order, every application merged over what the block has so far. The one thing an
/// application needs from the ones before it is the block's current shape, which decides
/// whether a `blockshape` is refused.
pub(crate) fn combine_blocks(
    parsed: Vec<ParsedBlocks>,
    stack: &StackCx<'_>,
) -> Registry<ClientBlock> {
    let mut builder = RegistryBuilder::new(
        CombineRule::FieldMerge { merge },
        CombineRule::top_wins(),
        stack.options,
    );
    // The shape each block has so far: its registration default, then what a pack gave it.
    let mut shapes: HashMap<BlockId, i16> = HashMap::new();
    for ParsedBlocks { origin, read } in parsed {
        let FileRead {
            file,
            effective,
            mut diagnostics,
            applied,
        } = read;
        diagnostics.locate(&origin);
        let Some(file) = file else {
            builder.push(Contribution::Rejected(
                None,
                Rejected::new(origin, diagnostics),
            ));
            continue;
        };
        for finding in diagnostics {
            builder.push_diagnostic(finding);
        }
        let version = version_info(&file, effective.version);
        for application in applied {
            let Applied {
                id,
                key,
                mut block,
                requested_shape,
                mut diagnostics,
            } = application;
            if let Some((name, requested)) = requested_shape {
                let current = shapes.entry(id.clone()).or_insert_with(|| {
                    id.as_str()
                        .strip_prefix("minecraft:")
                        .map_or(0, default_block_shape)
                });
                if is_refused(*current, requested) {
                    // The conversion is refused: the block keeps its shape.
                    let at = JsonPath::new().child(&key).child("blockshape");
                    diagnostics.push(entry::bad_blockshape(id.as_str(), at));
                } else {
                    *current = requested;
                    block.blockshape = Some(name);
                }
            }
            // The merged block holds its textures and isotropy as the six faces.
            block.textures = block.textures.map(|textures| textures.to_six_faces());
            block.carried_textures = block
                .carried_textures
                .map(|textures| textures.to_six_faces());
            block.isotropic = block.isotropic.map(crate::Isotropic::to_six_faces);
            let entry_origin = Origin::new(
                origin.pack.clone(),
                origin.side,
                origin.path.clone(),
                Some(key.into()),
                Vec::new(),
            );
            diagnostics.locate(&entry_origin);
            let document = Document::new(block, entry_origin, version.clone(), diagnostics);
            builder.push(Contribution::Stable(id, document));
        }
    }
    builder.finish()
}

/// The loader of `rp.client_blocks()`: `blocks.json` of every resource pack, merged field by
/// field from the lowest pack up. The family's own: one file holds every block (the entries
/// keep a hand `Definition`, `File = BlocksJson`) and the combine merges them field by field,
/// which `load::registry` has no shape for.
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<ClientBlock>, ParsedBlocks> = CollectionLoader {
    walk: BLOCKS_WALK,
    parse: parse_blocks_file,
    combine: combine_blocks,
    needs: &[],
    then: None,
};

/// The sound type of a block no pack gives one.
pub(crate) const NORMAL_SOUND: &str = "normal";

/// The texture of a face no pack gives one: the client registers every block with it.
pub(crate) const MISSING_TILE: &str = "missing_tile";

impl ClientBlock {
    /// The exponent the client uses: the pack's, or 1.0, which the client starts every block
    /// with.
    #[must_use]
    pub fn ambient_occlusion_exponent(&self) -> f32 {
        self.ambient_occlusion_exponent.unwrap_or(1.0)
    }

    /// The sound type the client uses: the pack's, or `normal`.
    #[must_use]
    pub fn sound(&self) -> &str {
        self.sound.as_deref().unwrap_or(NORMAL_SOUND)
    }

    /// The textures the client draws the six faces with, in the order up, down, north, south,
    /// west, east: the pack's, or `missing_tile` on every face.
    #[must_use]
    pub fn face_textures(&self) -> [&str; 6] {
        self.textures
            .as_ref()
            .map_or([MISSING_TILE; 6], FaceTextures::faces)
    }

    /// The textures the block is carried with, in the order up, down, north, south, west,
    /// east: `carried_textures`, or the block's own face textures.
    #[must_use]
    pub fn carried_face_textures(&self) -> [&str; 6] {
        self.carried_textures
            .as_ref()
            .map_or_else(|| self.face_textures(), FaceTextures::faces)
    }

    /// The client's isotropic face mask: `down` 0x01, `up` 0x02, `north` 0x04, `south` 0x08,
    /// `west` 0x10, `east` 0x20 (0xff for `true`); 0 when no pack sets `isotropic`.
    #[must_use]
    pub fn isotropic_mask(&self) -> u8 {
        self.isotropic.map_or(0, Isotropic::mask)
    }

    /// The number of `blockshape` (vanilla's shape number), when the pack names a shape.
    #[must_use]
    pub fn blockshape_value(&self) -> Option<i16> {
        self.blockshape
            .as_deref()
            .and_then(crate::block_shape_value)
    }
}

impl FaceTextures {
    /// The texture of each face, in the order up, down, north, south, west, east.
    #[must_use]
    pub fn faces(&self) -> [&str; 6] {
        match self {
            Self::All(all) => [all; 6],
            Self::UpDownSide { up, down, side } => [up, down, side, side, side, side],
            Self::SixFaces {
                up,
                down,
                north,
                south,
                west,
                east,
            } => [up, down, north, south, west, east],
        }
    }

    /// The same textures in the six-face form.
    #[must_use]
    pub fn to_six_faces(&self) -> Self {
        let [up, down, north, south, west, east] = self.faces().map(str::to_owned);
        Self::SixFaces {
            up,
            down,
            north,
            south,
            west,
            east,
        }
    }
}

impl Isotropic {
    /// The client's face mask: `down` 0x01, `up` 0x02, `north` 0x04, `south` 0x08, `west`
    /// 0x10, `east` 0x20; `true` is 0xff.
    #[must_use]
    pub fn mask(self) -> u8 {
        let bit = |on: bool, bits: u8| if on { bits } else { 0 };
        match self {
            Self::All(true) => 0xff,
            Self::All(false) => 0,
            Self::UpDownSide(FaceBools3 { up, down, side }) => {
                bit(up, 0x02) | bit(down, 0x01) | bit(side, 0x3c)
            }
            Self::SixFaces(FaceBools6 {
                up,
                down,
                north,
                south,
                west,
                east,
            }) => {
                bit(down, 0x01)
                    | bit(up, 0x02)
                    | bit(north, 0x04)
                    | bit(south, 0x08)
                    | bit(west, 0x10)
                    | bit(east, 0x20)
            }
        }
    }

    /// The same faces in the six-face form (`true` keeps the client's whole mask, 0xff, as
    /// [`Isotropic::All`]).
    #[must_use]
    pub fn to_six_faces(self) -> Self {
        match self {
            Self::All(_) | Self::SixFaces(_) => self,
            Self::UpDownSide(FaceBools3 { up, down, side }) => Self::SixFaces(FaceBools6 {
                up,
                down,
                north: side,
                south: side,
                west: side,
                east: side,
            }),
        }
    }
}
