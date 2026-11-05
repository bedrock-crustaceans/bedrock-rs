//! Loading `blocks.json`: `Content` / `Definition`, the walk, and the combine rule (every pack's
//! entries merged field by field, lowest pack first).

use std::collections::HashMap;

use super::resolve::{self, Applied, FileRead};
use super::shape::{default_block_shape, is_refused};
use super::version::BLOCKS_JSON_DEFAULT_VERSION;
use super::{BlocksJson, ClientBlock, entry};
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Origin, Placed, Registry,
    RegistryBuilder, Rejected, VersionInfo,
};
use bedrock_addon_core::id::{BlockId, kind};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;

/// Gives every finding the document's pack and file.
fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
        diagnostics.push(finding);
    }
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
        let origin = Origin::new(
            PackRef::app_data(),
            PackSide::Resource,
            path.clone(),
            None,
            Vec::new(),
        );
        let FileRead {
            file,
            effective,
            mut diagnostics,
            applied,
        } = resolve::read(bytes, cx.is_base_game_pack);
        for application in applied {
            diagnostics.extend(application.diagnostics);
        }
        locate(&mut diagnostics, &origin);
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
            .and_then(|value| match super::version::from_json(value) {
                super::version::Match::Full(version) => Some(version.version),
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

/// `blocks.json` at the pack root.
fn is_blocks_json(path: &PackPath) -> bool {
    path.as_str() == BLOCKS_JSON
}

/// The root file of every pack, lowest priority first: the client visits the stack from its last
/// entry down to its first and reads at most one file per entry.
const BLOCKS_WALK: Walk = Walk {
    directories: &[""],
    depth: ListDepth::Children,
    accept: Accept::Fn(is_blocks_json),
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// One pack's file as read.
#[derive(Clone, Debug)]
pub struct ParsedBlocks {
    /// The file's origin.
    pub(super) origin: Origin,
    /// What reading it gave.
    pub(super) read: FileRead,
}

/// Reads one walked file.
fn parse_blocks_file(file: &WalkFile, cx: &LoadContext) -> ParsedBlocks {
    ParsedBlocks {
        origin: Origin::new(
            file.pack.clone(),
            file.side,
            file.path.clone(),
            None,
            Vec::new(),
        ),
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
pub(super) fn combine_blocks(
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
        locate(&mut diagnostics, &origin);
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
                    let mut at = JsonPath::new();
                    at.push(&key);
                    at.push("blockshape");
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
            block.isotropic = block.isotropic.map(super::Isotropic::to_six_faces);
            let entry_origin = Origin::new(
                origin.pack.clone(),
                origin.side,
                origin.path.clone(),
                Some(key.into()),
                Vec::new(),
            );
            locate(&mut diagnostics, &entry_origin);
            let document = Document::new(block, entry_origin, version.clone(), diagnostics);
            builder.push(Contribution::Stable(id, document));
        }
    }
    builder.finish()
}

/// The loader of `rp.client_blocks()`: `blocks.json` of every resource pack, merged field by
/// field from the lowest pack up.
pub const fn load_client_blocks() -> CollectionLoader<Registry<ClientBlock>, ParsedBlocks> {
    CollectionLoader {
        walk: BLOCKS_WALK,
        parse: parse_blocks_file,
        combine: combine_blocks,
        needs: &[],
        then: None,
    }
}
