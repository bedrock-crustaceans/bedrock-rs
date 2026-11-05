//! Loading the camera documents: the four families' chain parameters, `Content` /
//! `Definition`, the walk of `cameras/` and the group's combine rule with the pass over every
//! preset and the list checks.

use std::convert::Infallible;

use indexmap::IndexMap;

use super::history::{
    Any, EntityAny, EntityFamily, ListAny, ListFamily, PresetFamily, SplineAny, SplineFamily,
};
use super::{
    CameraEntity, CameraList, CameraPreset, CameraSpline, Cameras, Splines, stack, texts, validate,
};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Phase, VanillaText};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonContent, NoBeta, Notice, Placed,
    RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::{CameraListId, CameraPresetId, Id, kind};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::load::{
    Accept, AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile,
    WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    EnvelopeCheck, ErrorMask, Family, Identity, InputKind, Loaded, MissingVersion, NoComponents,
    StructIndex, ValidateCx, VersionStruct, Versioned,
};
use bedrock_addon_support::chain_file::{
    self as file, FileFamily, FileOutcome, closing_line, puv_below_floor,
};

/// `cameras/presets`: `minecraft:camera_preset` documents.
const PRESETS: &str = "cameras/presets";
/// `cameras/lists`: `minecraft:camera_list` documents.
const LISTS: &str = "cameras/lists";
/// `cameras/splines`: `minecraft:camera_custom_splines` documents (the game reads this
/// directory; the singular `camera/splines/` of the 1.26.0 release notes is not read).
const SPLINES: &str = "cameras/splines";

/// The first preset and list version.
pub(crate) const V1_19_50: FormatVersion = FormatVersion::new(1, 19, 50);
/// The newest preset struct's version.
pub(crate) const V1_26_50: FormatVersion = FormatVersion::new(1, 26, 50);
/// The spline struct's version.
pub(crate) const V1_26_0: FormatVersion = FormatVersion::new(1, 26, 0);
/// The version vanilla's own camera entity documents carry, and the one written.
pub(crate) const ENTITY_WRITTEN: FormatVersion = FormatVersion::new(1, 26, 10);

// ---- the families --------------------------------------------------------------------------

impl Family for PresetFamily {
    const NAME: &'static str = "camera preset";
    const PAYLOAD_KEY: &'static str = "minecraft:camera_preset";
    const DIRECTORIES: &'static [&'static str] = &[PRESETS];
    const FLOOR: FormatVersion = V1_19_50;
    type Latest = CameraPreset;
    type Beta = Infallible;
    type Component = NoComponents;
    type Any = Any;
    // The game hands the document reader an invalid minimum version, so the text stays raw and
    // both slices parse the whole document, envelope included.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Never;
    // A missing version is rejected with the game's default text, "no valid version passed by
    // the caller or found in the input".
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // The game reads a preset through a document input: the version is read as that input
    // reads it, and a `null` document passes no shape, which the shape check applies.
    const INPUT: InputKind = InputKind::Document;

    /// The legacy slice `[1.19.50, 1.26.50)` tolerates some errors the 1.26.50 slice does not.
    /// An unknown member drops the preset under both: the game scans the log of the parse for
    /// the unknown-member error, which the shape check reports as an error.
    fn error_mask(index: StructIndex) -> ErrorMask {
        match index {
            StructIndex::Stable(0) => ErrorMask::Unstrict,
            _ => ErrorMask::Strict,
        }
    }

    fn identity(payload: JsonSlice<'_>) -> Identity {
        file::identity(payload, "identifier")
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(preset) = v {
            validate::preset(preset, cx);
        }
    }
}

impl FileFamily for PresetFamily {
    fn close(
        diagnostics: &mut Diagnostics,
        document: Option<JsonSlice<'_>>,
        _: &PackPath,
        failed: bool,
    ) {
        puv_below_floor(diagnostics, document);
        // With an unknown member vanilla logs its own line and nothing else: the only line
        // is 'Unrecognized field on camera preset!'.
        let unknown = diagnostics
            .iter()
            .any(|d| matches!(d.code, DiagnosticCode::UnrecognizedField { .. }));
        if failed && !unknown {
            closing_line(diagnostics, texts::FAILED_PRESETS);
        }
    }

    fn document(value: &CameraPreset) -> Result<Value, EncodeError> {
        value.write()
    }
}

impl Family for ListFamily {
    const NAME: &'static str = "camera list";
    const PAYLOAD_KEY: &'static str = "minecraft:camera_list";
    const DIRECTORIES: &'static [&'static str] = &[LISTS];
    const FLOOR: FormatVersion = V1_19_50;
    type Latest = CameraList;
    type Beta = Infallible;
    type Component = NoComponents;
    type Any = ListAny;
    // A plain strict load of the whole file, which binds `format_version` and the payload
    // itself: no envelope, and a third root member is an unknown member.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Never;
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // The required bind of `format_version`.
    const MISSING_VERSION_TEXT: Option<VanillaText> = Some(VanillaText::Fixed(
        bedrock_addon_support::shape::MISSING_REQUIRED,
    ));
    // The game reads a list without a document input: the input kind stays the first run's,
    // and the shape check rejects a `null` and an unknown member itself.

    fn identity(payload: JsonSlice<'_>) -> Identity {
        file::identity(payload, "identifier")
    }
}

impl FileFamily for ListFamily {
    fn close(
        diagnostics: &mut Diagnostics,
        _: Option<JsonSlice<'_>>,
        path: &PackPath,
        failed: bool,
    ) {
        let found = std::mem::take(diagnostics);
        for mut finding in found {
            match &mut finding.code {
                // The version range is a constraint on the version string, not a slice of the
                // chain: the game's text for it is not known, so neither generic text stands in.
                DiagnosticCode::UnsupportedVersion { text } => *text = None,
                // "Camera List file %s is not valid JSON: %s"
                DiagnosticCode::MalformedInput { message, text } => {
                    *text = texts::formatted(texts::LIST_NOT_JSON, &[path.as_str(), message]);
                }
                _ => {}
            }
            diagnostics.push(finding);
        }
        let not_json = diagnostics
            .iter()
            .any(|d| matches!(d.code, DiagnosticCode::MalformedInput { .. }));
        if failed && !not_json {
            closing_line(diagnostics, texts::LIST_INVALID);
        }
    }

    fn document(value: &CameraList) -> Result<Value, EncodeError> {
        value.write()
    }
}

impl Family for SplineFamily {
    const NAME: &'static str = "camera spline";
    const PAYLOAD_KEY: &'static str = "minecraft:camera_custom_splines";
    const DIRECTORIES: &'static [&'static str] = &[SPLINES];
    const FLOOR: FormatVersion = V1_26_0;
    type Latest = CameraSpline;
    type Beta = Infallible;
    type Component = NoComponents;
    type Any = SplineAny;
    // The game reads the file as a document of `minecraft:camera_custom_splines` from 1.26.0:
    // the envelope is checked from the slice's version.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::From(V1_26_0);
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // The input is a document, as for presets.
    const INPUT: InputKind = InputKind::Document;

    /// One slice, read strictly.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Strict
    }

    fn identity(_: JsonSlice<'_>) -> Identity {
        Identity::default()
    }
}

impl FileFamily for SplineFamily {
    fn close(
        diagnostics: &mut Diagnostics,
        document: Option<JsonSlice<'_>>,
        _: &PackPath,
        failed: bool,
    ) {
        puv_below_floor(diagnostics, document);
        if failed {
            closing_line(diagnostics, texts::SPLINES_FAILED);
        }
    }

    fn document(value: &CameraSpline) -> Result<Value, EncodeError> {
        enveloped::<Self>(V1_26_0, value)
    }
}

impl Family for EntityFamily {
    const NAME: &'static str = "camera entity";
    const PAYLOAD_KEY: &'static str = "minecraft:camera_entity";
    const DIRECTORIES: &'static [&'static str] = &["cameras"];
    // The document's version is 0.0.0, and that is a valid version: the envelope is always
    // checked.
    const FLOOR: FormatVersion = FormatVersion::new(0, 0, 0);
    type Latest = CameraEntity;
    type Beta = Infallible;
    type Component = NoComponents;
    type Any = EntityAny;
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // The input is a document, as for presets.
    const INPUT: InputKind = InputKind::Document;

    /// One slice, read strictly.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Strict
    }
}

impl FileFamily for EntityFamily {
    fn close(_: &mut Diagnostics, _: Option<JsonSlice<'_>>, _: &PackPath, _: bool) {}

    fn document(value: &CameraEntity) -> Result<Value, EncodeError> {
        enveloped::<Self>(ENTITY_WRITTEN, value)
    }
}

/// `{"format_version": version, <payload key>: value}` for a family whose struct is the
/// payload.
fn enveloped<F: FileFamily>(
    version: FormatVersion,
    value: &F::Latest,
) -> Result<Value, EncodeError> {
    let mut root = facet_value::VObject::new();
    root.insert("format_version", version.to_string());
    root.insert(F::PAYLOAD_KEY, value.write()?);
    Ok(root.into())
}

// ---- Content ---------------------------------------------------------------------------------

/// The `Content` impl of a root type and its `parse_as_written`.
macro_rules! content {
    ($root:ty, $family:ty, $any:ty, name = $name:literal, side = $side:ident) => {
        impl bedrock_addon_core::__private::Sealed for $root {}

        impl Content for $root {
            const NAME: &'static str = $name;
            const SIDES: &'static [PackSide] = &[PackSide::$side];

            fn parse(
                path: &PackPath,
                bytes: &[u8],
                cx: &LoadContext,
            ) -> Result<Document<Self>, Rejected> {
                file::parse_file::<$family>(PackRef::app_data(), PackSide::$side, path, bytes, cx).1
            }

            fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
                write_json(&<$family as FileFamily>::document(self)?, out, style)
            }
        }

        impl JsonContent for $root {}

        impl $root {
            /// Parses one file without upgrading it: the struct its `format_version` selects.
            ///
            /// # Errors
            ///
            /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) with the findings the game's content
            /// log would print when the bytes cannot be read as this file at all: not JSON, a
            /// `format_version` with no struct here, or a document that does not fit the struct it selects.
            pub fn parse_as_written(
                path: &PackPath,
                bytes: &[u8],
                cx: &LoadContext,
            ) -> Result<Document<$any>, Rejected> {
                file::as_written::<$family>(PackSide::$side, path, bytes, cx)
            }
        }
    };
}

content!(
    CameraPreset,
    PresetFamily,
    Any,
    name = "camera preset",
    side = Behavior
);
content!(
    CameraList,
    ListFamily,
    ListAny,
    name = "camera list",
    side = Behavior
);
content!(
    CameraSpline,
    SplineFamily,
    SplineAny,
    name = "camera spline",
    side = Behavior
);
content!(
    CameraEntity,
    EntityFamily,
    EntityAny,
    name = "camera entity",
    side = Resource
);

impl Definition for CameraPreset {
    type Kind = kind::CameraPreset;
    type File = CameraPreset;
    type Beta = NoBeta<kind::CameraPreset>;
}

impl Placed for CameraPreset {
    /// `cameras/presets/<namespace>/<name>.json`.
    fn default_path(&self) -> PackPath {
        default_path(PRESETS, &self.identifier)
    }
}

impl Definition for CameraList {
    type Kind = kind::CameraList;
    type File = CameraList;
    type Beta = NoBeta<kind::CameraList>;
}

impl Placed for CameraList {
    /// `cameras/lists/<namespace>/<name>.json`.
    fn default_path(&self) -> PackPath {
        default_path(LISTS, &self.identifier)
    }
}

// ---- the group -----------------------------------------------------------------------------

/// What one walked file parses to.
#[derive(Debug)]
pub enum CameraFile {
    /// A `minecraft:camera_preset` file.
    Preset(Box<FileOutcome<CameraPreset>>),
    /// A `minecraft:camera_list` file.
    List(Box<FileOutcome<CameraList>>),
    /// A `minecraft:camera_custom_splines` file.
    Spline(Box<Result<Document<CameraSpline>, Rejected>>),
}

/// A camera file: an extension exactly `json`.
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// `cameras/{presets, lists, splines}/**/*.json`: every pack's presets, then every pack's
/// lists, then the splines, packs from the lowest priority up, so a later definition (the
/// higher pack's) replaces an earlier one, as most of the game's loaders visit a stack.
const WALK: Walk = Walk {
    directories: &[PRESETS, LISTS, SPLINES],
    depth: ListDepth::Recursive,
    accept: Accept::Fn(is_json),
    order: WalkOrder::BottomUp,
    directory_major: true,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// Parses one walked file by its directory.
fn parse_walk_file(walked: &WalkFile, cx: &LoadContext) -> CameraFile {
    let (pack, side, path, bytes) = (
        walked.pack.clone(),
        walked.side,
        &walked.path,
        &walked.bytes,
    );
    match WALK.matches(&walked.key) {
        Some(0) => CameraFile::Preset(Box::new(file::parse_file::<PresetFamily>(
            pack, side, path, bytes, cx,
        ))),
        Some(1) => CameraFile::List(Box::new(file::parse_file::<ListFamily>(
            pack, side, path, bytes, cx,
        ))),
        _ => CameraFile::Spline(Box::new(
            file::parse_file::<SplineFamily>(pack, side, path, bytes, cx).1,
        )),
    }
}

/// A registry in which the later definition of an identifier replaces the earlier one whole,
/// as vanilla assigns it in place; the replaced one is a `ShadowedDefinition` info.
fn last_loaded_wins<T: Definition>(stack: &StackCx<'_>) -> RegistryBuilder<T> {
    RegistryBuilder::new(
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        },
        CombineRule::last_loaded_wins(),
        stack.options,
    )
}

/// The rejection of a document the stack stage refuses: its own findings, then `finding`.
fn refuse<T>(document: &Document<T>, finding: Diagnostic) -> Rejected {
    refuse_all(document, std::iter::once(finding))
}

/// [`refuse`] with every finding of `findings`.
fn refuse_all<T>(
    document: &Document<T>,
    findings: impl IntoIterator<Item = Diagnostic>,
) -> Rejected {
    let origin = document.origin().clone();
    let mut diagnostics = document.diagnostics().clone();
    for finding in findings {
        diagnostics.push(finding);
    }
    Rejected::new(origin, diagnostics)
}

/// The presets: last loaded wins per identifier, then one pass over the merged set discards
/// the invalid ones. Returns the registry and the identifiers the
/// stack defined, with whether each survived.
fn presets(
    parsed: Vec<FileOutcome<CameraPreset>>,
    stack: &StackCx<'_>,
) -> (
    bedrock_addon_core::document::Registry<CameraPreset>,
    IndexMap<CameraPresetId, bool>,
) {
    // The effective definition of each identifier: the last one that loaded.
    let mut effective: IndexMap<CameraPresetId, usize> = IndexMap::new();
    for (index, (_, outcome)) in parsed.iter().enumerate() {
        if let Ok(document) = outcome {
            effective.insert(document.identifier.clone(), index);
        }
    }
    let merged: IndexMap<&CameraPresetId, &Document<CameraPreset>> = effective
        .iter()
        .filter_map(|(id, &index)| Some((id, parsed.get(index)?.1.as_ref().ok()?)))
        .collect();
    let mut discarded = stack::discard_invalid_presets(&merged);
    let defined: IndexMap<CameraPresetId, bool> = effective
        .keys()
        .map(|id| (id.clone(), !discarded.contains_key(id)))
        .collect();

    let mut builder = last_loaded_wins::<CameraPreset>(stack);
    for (index, (identifier, outcome)) in parsed.into_iter().enumerate() {
        match outcome {
            Err(rejected) => {
                builder.push(Contribution::Rejected(identifier.map(Id::parse), rejected));
            }
            Ok(document) => {
                let id = document.identifier.clone();
                if defined.get(&id) != Some(&false) {
                    builder.push(Contribution::Stable(id, document));
                } else if effective.get(&id) == Some(&index) {
                    let finding = discarded.swap_remove(&id);
                    let rejected = match finding {
                        // An unknown parent also gets the game's own line, besides the
                        // reference finding.
                        Some(finding)
                            if matches!(
                                finding.code,
                                DiagnosticCode::UnresolvedReference { .. }
                            ) =>
                        {
                            // Only where the direct parent is no preset of the stack: a
                            // parent the pass discarded is the chain's, not the validator's.
                            let line = stack::unknown_parent_line(&document).filter(|_| {
                                document
                                    .inherit_from
                                    .as_ref()
                                    .is_some_and(|parent| !effective.contains_key(parent))
                            });
                            refuse_all(&document, std::iter::once(finding).chain(line))
                        }
                        Some(finding) => refuse(&document, finding),
                        None => Rejected::new(document.origin().clone(), Diagnostics::default()),
                    };
                    builder.push(Contribution::Rejected(Some(id), rejected));
                } else {
                    // An earlier definition of a discarded identifier: vanilla replaced it
                    // before the pass ran, so it does not come back.
                    for finding in document.diagnostics().iter().cloned() {
                        builder.push_diagnostic(finding);
                    }
                    builder.push_diagnostic(
                        Diagnostic::info(DiagnosticCode::ShadowedDefinition { text: None })
                            .phase(Phase::Validate)
                            .in_file(document.origin()),
                    );
                }
            }
        }
    }
    (builder.finish(), defined)
}

/// The lists: each checked against the presets that survived when it loads, then last loaded
/// wins per identifier.
fn lists(
    parsed: Vec<FileOutcome<CameraList>>,
    defined: &IndexMap<CameraPresetId, bool>,
    stack: &StackCx<'_>,
) -> bedrock_addon_core::document::Registry<CameraList> {
    let mut builder = last_loaded_wins::<CameraList>(stack);
    for (identifier, outcome) in parsed {
        let contribution = match outcome {
            Err(rejected) => Contribution::Rejected(identifier.map(CameraListId::parse), rejected),
            Ok(document) => {
                let id = document.identifier.clone();
                match stack::check_list(&document, defined, stack) {
                    // Each unknown entry also gets the game's own line, besides the
                    // reference finding.
                    Some(finding)
                        if matches!(finding.code, DiagnosticCode::UnresolvedReference { .. }) =>
                    {
                        let lines = stack::unknown_camera_lines(&document, defined, stack);
                        let all = std::iter::once(finding).chain(lines);
                        Contribution::Rejected(Some(id), refuse_all(&document, all))
                    }
                    Some(finding) => Contribution::Rejected(Some(id), refuse(&document, finding)),
                    None => Contribution::Stable(id, document),
                }
            }
        };
        builder.push(contribution);
    }
    builder.finish()
}

/// The spline files: every one kept, in walk order, with the duplicate names reported.
fn splines(parsed: Vec<Result<Document<CameraSpline>, Rejected>>) -> Splines {
    let mut splines = Splines::default();
    for outcome in parsed {
        match outcome {
            Ok(document) => {
                splines
                    .diagnostics
                    .extend(document.diagnostics().iter().cloned());
                splines.files.push(document);
            }
            Err(rejected) => {
                splines
                    .diagnostics
                    .extend(rejected.diagnostics.iter().cloned());
                splines.rejected.push(rejected);
            }
        }
    }
    for finding in stack::duplicate_splines(&splines.files) {
        splines.diagnostics.push(finding);
    }
    splines
}

/// The group, from every parsed file in walk order.
fn combine(parsed: Vec<CameraFile>, stack: &StackCx<'_>) -> Cameras {
    let mut preset_files = Vec::new();
    let mut list_files = Vec::new();
    let mut spline_files = Vec::new();
    for parsed_file in parsed {
        match parsed_file {
            CameraFile::Preset(outcome) => preset_files.push(*outcome),
            CameraFile::List(outcome) => list_files.push(*outcome),
            CameraFile::Spline(outcome) => spline_files.push(*outcome),
        }
    }
    let (presets, defined) = presets(preset_files, stack);
    Cameras {
        presets,
        lists: lists(list_files, &defined, stack),
        splines: splines(spline_files),
    }
}

/// The loader of `bp.cameras()`: the three directories of every behaviour pack, combined into
/// the group.
pub const fn load_group() -> CollectionLoader<Cameras, CameraFile> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

impl Reported for Cameras {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

impl Versioned for CameraPreset {
    const VERSION: FormatVersion = V1_26_50;
}

impl Versioned for CameraList {
    const VERSION: FormatVersion = V1_19_50;
}

impl Versioned for CameraSpline {
    const VERSION: FormatVersion = V1_26_0;
}

impl Versioned for CameraEntity {
    const VERSION: FormatVersion = FormatVersion::new(0, 0, 0);
}
