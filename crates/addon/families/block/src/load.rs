//! Loading blocks: the family's version chain parameters, `Content` / `Definition`, the walk
//! of `blocks/` and the combine rule.

use super::component::BlockComponent;
use super::history::{Any, BlockFamily};
use super::parse::CEREAL;
use super::{Block, validate};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Origin, Placed,
    Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::{BlockId, kind};
use bedrock_addon_core::json::NullRule;
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, WriteStyle, check_depth, strip_bom,
};
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    ComponentSection, ComponentTable, EnvelopeCheck, ErrorMask, Family, Identity, InputKind,
    Loaded, Parsed, StructIndex, ValidateCx, VersionChain, finish, parse_as_written,
};

/// The floor: below 1.16.0 the game's upgrades discard most components, so the crate rejects the
/// file.
const FLOOR: FormatVersion = FormatVersion::new(1, 16, 0);

impl Family for BlockFamily {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with an information at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "block";
    const PAYLOAD_KEY: &'static str = "minecraft:block";
    const DIRECTORIES: &'static [&'static str] = &["blocks"];
    const FLOOR: FormatVersion = FLOOR;
    type Latest = Block;
    type Beta = std::convert::Infallible;
    type Component = BlockComponent;
    type Any = Any;
    // The game reads a block file as a versioned document: the raw text below 1.26.20, exactly
    // two members from it.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::From(CEREAL);
    // Rule 11 reads a component written twice ("Duplicate component … found"); every other
    // member written twice keeps its last value.
    const DUPLICATES_KEPT_IN: &'static [&'static str] = &["components"];
    // Every block file is read as a document whose version path and size limits are the
    // file's own.
    const INPUT: InputKind = InputKind::Document;

    // The v1_26_20 and v1_26_60 slices are strict: an unknown member or component of a cereal
    // document rejects it. The legacy slice is the hand-written parser's, which does not
    // reject unknown members: the crate's legacy reader keeps its findings as they are.
    fn error_mask(index: StructIndex) -> ErrorMask {
        if is_cereal(index) {
            ErrorMask::Strict
        } else {
            ErrorMask::Unstrict
        }
    }

    // Rule N2 for the cereal slices; the legacy slice's `null` is its hand-written parser's
    // (the crate's legacy reader), with no framework rule.
    fn null_rule(index: StructIndex) -> Option<NullRule> {
        is_cereal(index).then_some(NullRule::Rejected)
    }

    fn components() -> &'static ComponentTable<Self> {
        BlockComponent::TABLE
    }

    fn component_sections(v: &mut Block) -> Vec<ComponentSection<'_, BlockComponent>> {
        let mut sections = Vec::with_capacity(1 + v.permutations.len());
        let at = JsonPath::new().child("components");
        sections.push(ComponentSection::new(at, false, &mut v.components));
        for (index, permutation) in v.permutations.iter_mut().enumerate() {
            let at = JsonPath::new()
                .child("permutations")
                .index(index)
                .child("components");
            sections.push(ComponentSection::new(at, true, &mut permutation.components));
        }
        sections
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(block) = v {
            validate::validate(block, cx);
        }
    }

    fn actor_type(_: &Identity) -> ActorType {
        ActorType::Mob
    }
}

/// Whether `index` names a cereal struct (1.26.20 and later).
fn is_cereal(index: StructIndex) -> bool {
    match index {
        StructIndex::Stable(i) => <Any as VersionChain<BlockFamily>>::VERSIONS
            .get(i)
            .is_some_and(|version| *version >= CEREAL),
        StructIndex::Beta => false,
    }
}

impl bedrock_addon_core::__private::Sealed for Block {}

impl JsonContent for Block {}

impl Content for Block {
    const NAME: &'static str = "block";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx).1
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl Definition for Block {
    type Kind = kind::Block;
    type File = Block;
    type Beta = NoBeta<kind::Block>;
}

impl Placed for Block {
    /// `blocks/<namespace>/<name>.json`.
    fn default_path(&self) -> PackPath {
        default_path("blocks", &self.description.identifier)
    }
}

impl Block {
    /// Parses one block file without upgrading it: the struct its `format_version` selects.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file is not text, is not JSON, or its `format_version`
    /// selects a struct the file does not fit (the findings are in the [`Rejected`]).
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let bytes = &*bedrock_addon_core::json::blank_bom(bytes); // spans stay offsets into the file as written
        let origin = origin(PackRef::app_data(), PackSide::Behavior, path);
        let Some(text) = text(bytes, &origin) else {
            return Err(reject(origin, malformed()));
        };
        let parsed = parse_as_written::<BlockFamily>(JsonSlice::new(text), cx);
        let mut diagnostics = parsed.diagnostics;
        diagnostics.locate(&origin);
        match parsed.value {
            Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

/// `history::upgrade`: rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<Any>,
    cx: &LoadContext,
) -> Result<Document<Block>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let identity = Identity {
        identifier: Some(identifier_of(&value)),
        runtime_identifier: None,
    };
    let at = JsonPath::new().child(BlockFamily::PAYLOAD_KEY);
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<BlockFamily>::new(
        Some(value),
        version,
        written,
        identity,
        ActorType::Mob,
        false,
        at,
        diagnostics,
    );
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(block)) => {
            Ok(Document::new(block, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The identifier of a parsed document, as written.
fn identifier_of(value: &Any) -> String {
    match value {
        Any::V1_26_60(block) => block.description.identifier.as_str().to_owned(),
        Any::V1_26_20(b) => b.block.description.identifier.as_str().to_owned(),
        Any::V1_16_0(b) => legacy_identifier(&b.description),
        Any::V1_16_100(b) => legacy_identifier(&b.description),
        Any::V1_18_0(b) => legacy_identifier(&b.description),
        Any::V1_19_10(b) => legacy_identifier(&b.description),
        Any::V1_19_20(b) => legacy_identifier(&b.description),
        Any::V1_19_30(b) => legacy_identifier(&b.description),
        Any::V1_19_40(b) => legacy_identifier(&b.description),
        Any::V1_19_80(b) => legacy_identifier(&b.description),
        Any::V1_20_10(b) => legacy_identifier(&b.description),
        Any::V1_20_60(b) => legacy_identifier(&b.description),
        Any::V1_21_30(b) => legacy_identifier(&b.description),
        Any::V1_21_60(b) => legacy_identifier(&b.description),
        Any::V1_21_90(b) => legacy_identifier(&b.description),
        Any::V1_26_0(b) => legacy_identifier(&b.description),
    }
}

/// `identifier` of a legacy description.
fn legacy_identifier(
    description: &indexmap::IndexMap<String, bedrock_addon_core::json::Value>,
) -> String {
    description
        .get("identifier")
        .and_then(|v| v.as_string())
        .map(|s| s.as_str().to_owned())
        .unwrap_or_default()
}

/// The text of a file, after the BOM and the depth check; `None` when it is not UTF-8 or too
/// deep.
fn text<'b>(bytes: &'b [u8], _: &Origin) -> Option<&'b str> {
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).ok()?;
    std::str::from_utf8(bytes).ok()
}

/// The finding of an unreadable file.
fn malformed() -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MalformedInput {
        message: "not UTF-8 JSON within the nesting limit".to_owned(),
        text: None,
    })
}

/// A rejection with one finding.
fn reject(origin: Origin, finding: Diagnostic) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(finding);
    Rejected::located(origin, diagnostics)
}

/// The origin of a file of `pack`.
fn origin(pack: PackRef, side: PackSide, path: &PackPath) -> Origin {
    Origin::file(pack, side, path.clone())
}

/// Loads one block file: the identifier it names (when the payload has one) and the document or
/// its rejection.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> (Option<BlockId>, Result<Document<Block>, Rejected>) {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes); // spans stay offsets into the file as written
    let origin = origin(pack, side, path);
    let Some(text) = text(bytes, &origin) else {
        return (None, Err(reject(origin, malformed())));
    };
    let mut parsed = parse_as_written::<BlockFamily>(JsonSlice::new(text), cx);
    // A file below 1.26.20 is read whole (its envelope is not checked), so the parse stage
    // gives the root as its path; the block it becomes is the payload, under its key, where
    // the legacy reader's findings are: the checks on the upgraded block report there too,
    // core's component gates and Molang pass 2 as well as the family's rules.
    if parsed.at.is_empty() {
        parsed.at.push(BlockFamily::PAYLOAD_KEY);
    }
    let identifier = parsed
        .identity
        .identifier
        .as_deref()
        .map(|id| BlockId::parse(id.to_owned()));
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    let result = match outcome.value {
        Some(Loaded::Stable(block)) => {
            Ok(Document::new(block, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    };
    (identifier, result)
}

// ---- the collection ----------------------------------------------------------------------

/// A block file: a `.json` file below `blocks/`.
fn is_json(path: &PackPath) -> bool {
    // Vanilla's loader takes `.json` only, as written.
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// `blocks/**/*.json`, packs bottom-up (the higher pack's definition replaces the lower's).
const WALK: Walk = Walk {
    directories: &["blocks"],
    depth: ListDepth::Recursive,
    accept: Accept::Fn(is_json),
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,    // no file is left out by its pack
    shadowed: None,   // a shadowed subpack file is not read
    read_error: None, // an unreadable file has no family-specific text
};

/// One file's contribution.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<Block> {
    match parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        (_, Ok(document)) => {
            Contribution::Stable(document.description.identifier.clone(), document)
        }
        (id, Err(rejected)) => Contribution::Rejected(id, rejected),
    }
}

/// The registry, top pack wins.
fn combine(parsed: Vec<Contribution<Block>>, stack: &StackCx<'_>) -> Registry<Block> {
    let mut builder = RegistryBuilder::new(
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        stack.options,
    );
    for contribution in parsed {
        builder.push(contribution);
    }
    builder.finish()
}

/// The loader of `bp.blocks()`: `blocks/**/*.json` of every behaviour pack, combined top pack
/// wins.
pub const fn load_registry() -> CollectionLoader<Registry<Block>, Contribution<Block>> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[], // reads no other collection
        then: None, // a single walk stage
    }
}
