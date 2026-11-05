//! Loading voxel shapes: the family's chain parameters, `Content` / `Definition`, the walk of
//! `shapes/` and the combine rule.

use super::{FLOOR, VoxelShape, texts, validate};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Phase, Severity};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Notice, Origin,
    Placed, Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::kind;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Prepared, WriteStyle};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BelowFloor, Envelope, EnvelopeCheck, Family, InputKind, Loaded, MissingVersion, NoComponents,
    ValidateCx, load_file,
};

/// The voxel-shape family of the version-chain machinery.
#[derive(Clone, Copy, Debug)]
pub(crate) struct VoxelShapeFamily;

bedrock_addon_core::version_chain! {
    family = VoxelShapeFamily;
    /// A voxel-shape document parsed by the struct its `format_version` selects: there is one.
    #[derive(Clone, Debug, PartialEq)]
    pub(crate) enum Any {
        /// Format 1.21.110 up to the engine.
        V1_21_110(VoxelShape),
    }
    molang = none;
}

impl Family for VoxelShapeFamily {
    const NAME: &'static str = "voxel shape";
    const PAYLOAD_KEY: &'static str = "minecraft:voxel_shape";
    const FLOOR: FormatVersion = FLOOR;
    const ENVELOPE: Envelope = Envelope {
        // The game reads a shape through the document-input reader of the cereal-based families:
        // the version is read from the document, the size limits apply, and a member written
        // `null` fails its type.
        input: InputKind::Document,
        // The document input takes the raw text below 1.21.110 or without a version, and exactly
        // two members from 1.21.110 on.
        check: EnvelopeCheck::From(FLOOR),
        // Without a version the loader falls back to the caller's, which is none: "no valid version
        // passed by the caller or found in the input" and the file is skipped.
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    type Latest = VoxelShape;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Any;

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(shape) = v {
            validate::rules(shape, cx.load.is_base_game_pack, &cx.at, cx.diagnostics);
        }
    }

    /// After a file the schema loader rejected (a parse error; not a namespace or geometry
    /// failure, which log one line of their own), vanilla's "Failed to load voxel shape file" with
    /// the file's path in its pack.
    fn close(diagnostics: &mut Diagnostics, _: Option<JsonSlice<'_>>, path: &PackPath, _: bool) {
        let rejected_by_schema = diagnostics
            .iter()
            .any(|finding| finding.severity >= Severity::Error && finding.phase == Phase::Parse);
        if rejected_by_schema {
            diagnostics.push(Diagnostic::error(DiagnosticCode::DocumentRejected {
                text: Some(texts::failed_to_load(path.as_str())),
            }));
        }
    }
}

impl bedrock_addon_core::__private::Sealed for VoxelShape {}

impl Content for VoxelShape {
    const NAME: &'static str = "voxel shape";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }
}

impl JsonContent for VoxelShape {}

impl Definition for VoxelShape {
    type Kind = kind::VoxelShape;
    type File = VoxelShape;
    type Beta = NoBeta<kind::VoxelShape>;
}

impl Placed for VoxelShape {
    /// `shapes/<identifier>.json`, the identifier's `:` written `_`.
    fn default_path(&self) -> PackPath {
        default_path("shapes", &self.description.identifier)
    }
}

/// The finding of an unreadable file: not UTF-8, or nested past the limit (one text for both).
fn malformed() -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MalformedInput {
        message: "not UTF-8 JSON within the nesting limit".to_owned(),
        text: None,
    })
}

/// Loads one shape file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<VoxelShape>, Rejected> {
    let origin = Origin::file(pack, side, path.clone());
    // The BOM is blanked, not cut, so every span is an offset into the file as written.
    let Ok(prepared) = Prepared::new(bytes) else {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(malformed());
        <VoxelShapeFamily as Family>::close(&mut diagnostics, None, path, true);
        return Err(Rejected::located(origin, diagnostics));
    };
    let document = prepared.slice();
    let outcome = load_file::<VoxelShapeFamily>(document, path, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(shape)) => {
            Ok(Document::new(shape, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

// ---- the collection ----------------------------------------------------------------------

/// `shapes/**/*.json` (the extension compared case-sensitively), packs bottom-up: vanilla walks
/// the stack from its last pack to its first and binds an identifier to the shape it loads
/// last, so the highest pack's shape is the effective one. Sub-folders are read in every pack
/// (the game reads them only in packs outside the app package).
const WALK: Walk = Walk::json_tree_in(&["shapes"]);

/// One file's contribution.
fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> Contribution<VoxelShape> {
    match parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        Ok(document) => Contribution::Stable(document.description.identifier.clone(), document),
        Err(rejected) => Contribution::Rejected(None, rejected),
    }
}

/// The registry: per identifier the shape loaded last, each replaced one noted with
/// `ShadowedDefinition` at info (the crate's own note; vanilla replaces silently). A file that
/// failed binds nothing.
fn combine(parsed: Vec<Contribution<VoxelShape>>, stack: &StackCx<'_>) -> Registry<VoxelShape> {
    RegistryBuilder::combine(
        parsed,
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        },
        CombineRule::top_wins(),
        stack.options,
    )
}

/// The loader of `bp.voxel_shapes()`: `shapes/**/*.json` of every behaviour pack, the shape
/// loaded last winning per identifier.
pub const LOADER: CollectionLoader<Registry<VoxelShape>, Contribution<VoxelShape>> =
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    };

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn voxel_shape_reads_as_pinned() {
        assert_reading_table(
            "VoxelShapeFamily",
            &reading_table::<super::VoxelShapeFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: From(FormatVersion { major: 1, minor: 21, patch: 110 })
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.110..=1.21.110 (0..=0): json Rejected/Strict/Error/fatal/Refuse, mcb Own/Strict/Error/fatal/Refuse
            ",
        );
    }
}
