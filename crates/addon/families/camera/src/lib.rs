//! Cameras (`Cameras`, `CameraPreset`, `CameraList`, `CameraSpline`): the three document kinds
//! below `cameras/` of a behaviour pack, presets (`cameras/presets/`,
//! `minecraft:camera_preset`), lists (`cameras/lists/`, `minecraft:camera_list`) and spline
//! files (`cameras/splines/`, `minecraft:camera_custom_splines`), loaded together as the
//! `Cameras` group, and the camera entity documents of vanilla's resource pack
//! (`CameraEntity`, read on request only).

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
pub use entity::{
    AttachToPlayer, Avoidance, AvoidanceRay, Camera, CameraComponents, CameraEntity,
    CameraEntityDescription, DirectLook, FlyMove, LookMode, Offset, Orbit, PerspectiveOption,
    PlayerViewMode, Present, RotationSpace, Shake, Stationary, Target, ThirdPersonBoom,
    UpdatePlayerFromCamera,
};
pub use list::CameraList;
pub use preset::{
    ALLOWED_MC_INHERIT, AudioListener, CameraPreset, ControlScheme, PresetAimAssist, TargetMode,
    VANILLA_CAMERAS,
};
pub use spline::{
    CameraSpline, ControlPoint, Easing, ProgressKeyFrame, RotationKeyFrame, Spline, SplineType,
};

pub mod codes;
mod entity;
mod list;
mod preset;
pub(crate) mod read;
mod read_entity;
mod read_list;
mod read_spline;
mod spline;
mod stack;
mod validate;

#[cfg(test)]
mod tests;

/// A constraint with its own line: an `InvalidValue` error at `at`.
pub(crate) fn violation(line: &'static CodeDef<0>, at: JsonPath, d: &mut Diagnostics) {
    d.push(bedrock_addon_core::versioning::shape::error(
        Code::from(line),
        at,
    ));
}

/// The closing line of a failed file.
fn closing_line(diagnostics: &mut Diagnostics, line: &'static CodeDef<0>) {
    diagnostics.push(Diagnostic::of(line));
}

use std::convert::Infallible;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, CodeDef, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath,
};
use bedrock_addon_core::document::unversioned::origin_of;
use bedrock_addon_core::document::{
    Contribution, Document, JsonDefinition, JsonDocument, JsonPlaced, NoBeta, Registry, Rejected,
    default_path,
};
use bedrock_addon_core::id::{CameraListId, CameraPresetId, kind};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::load::{self, CollectionLoader, LoadContext, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use bedrock_addon_core::versioning::chain_file as file;
use bedrock_addon_core::versioning::{
    BelowFloor, Envelope, EnvelopeCheck, Family, Identity, InputKind, Loaded, MissingVersion,
    NoComponents, Reading, StructIndex, UnknownMask, ValidateCx, Versioned, parse_document,
};
use history::{
    EntityFamily, EntityHistory, ListFamily, ListHistory, PresetFamily, PresetHistory,
    SplineFamily, SplineHistory,
};

/// The camera documents of the behaviour packs (`bp.cameras()`): three collections loaded and
/// used together.
///
/// Presets and lists are keyed by their `identifier`; a later definition replaces an earlier
/// one whole, the packs read from the lowest priority up, so the top pack's wins. After the
/// presets of every pack are read, one pass discards the presets whose inheritance or vanilla
/// override is invalid; the lists are checked against what is left.
#[derive(Debug)]
#[non_exhaustive]
pub struct Cameras {
    /// The camera presets (`cameras/presets/**/*.json`).
    pub presets: Registry<CameraPreset>,
    /// The camera lists (`cameras/lists/**/*.json`).
    pub lists: Registry<CameraList>,
    /// The camera spline files (`cameras/splines/**/*.json`).
    pub splines: Splines,
}

impl Cameras {
    /// Every finding of the three collections, presets first.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.presets
            .diagnostics()
            .chain(self.lists.diagnostics())
            .chain(self.splines.diagnostics())
    }
}

/// The spline files of the stack, every one kept, in load order (packs from the lowest
/// priority up, files by path). Vanilla appends each file's splines to one list without
/// looking at their names, so two splines of one name both stay.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Splines {
    files: Vec<Document<CameraSpline>>,
    rejected: Vec<Rejected>,
    diagnostics: Diagnostics,
}

/// Every spline of a [`Splines`], from [`Splines::iter`].
#[derive(Clone, Debug)]
pub struct SplinesIter<'a> {
    files: std::slice::Iter<'a, Document<CameraSpline>>,
    current: std::slice::Iter<'a, Spline>,
}

impl<'a> Iterator for SplinesIter<'a> {
    type Item = &'a Spline;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(spline) = self.current.next() {
                return Some(spline);
            }
            self.current = self.files.next()?.splines.iter();
        }
    }
}

impl<'a> IntoIterator for &'a Splines {
    type Item = &'a Spline;
    type IntoIter = SplinesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Splines {
    /// The spline files that loaded, in load order.
    #[must_use]
    pub fn files(&self) -> &[Document<CameraSpline>] {
        &self.files
    }

    /// Every spline of every file, in load order.
    pub fn iter(&self) -> SplinesIter<'_> {
        SplinesIter {
            files: self.files.iter(),
            current: [].iter(),
        }
    }

    /// The first spline named `name`, in load order.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Spline> {
        self.iter().find(|spline| spline.name == name)
    }

    /// The number of splines.
    #[must_use]
    pub fn len(&self) -> usize {
        self.iter().count()
    }

    /// Whether no spline is loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Files that failed to load, with their diagnostics.
    #[must_use]
    pub fn rejected(&self) -> &[Rejected] {
        &self.rejected
    }

    /// Every finding of the spline files: per file, rejected files and the duplicate names.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.diagnostics.iter()
    }
}

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
pub(crate) const V1_26_50: FormatVersion = known::V1_26_50;
/// The spline struct's version.
pub(crate) const V1_26_0: FormatVersion = known::V1_26_0;
/// The version vanilla's own camera entity documents carry, and the one written.
pub(crate) const ENTITY_WRITTEN: FormatVersion = known::V1_26_10;

// ---- the families --------------------------------------------------------------------------

impl Family for PresetFamily {
    const NAME: &'static str = "camera preset";
    const PAYLOAD_KEY: &'static str = "minecraft:camera_preset";
    const FLOOR: FormatVersion = V1_19_50;
    const ENVELOPE: Envelope = Envelope {
        // The game reads a preset through a document input: the version is read as that input
        // reads it, and a `null` document passes no shape, which the shape check applies.
        input: InputKind::Document,
        // The game hands the document reader an invalid minimum version, so the text stays raw and
        // both slices parse the whole document, envelope included.
        check: EnvelopeCheck::Never,
        // A missing version is rejected with the game's default text, "no valid version passed by
        // the caller or found in the input".
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };

    /// The legacy slice `[1.19.50, 1.26.50)` tolerates some errors the 1.26.50 slice does not.
    /// An unknown member drops the preset under both: the game scans the log of the parse for
    /// the unknown-member error, which the shape check reports as an error.
    ///
    /// Both slices' tables say `Strict`, but the legacy slice 0 is read by the game's
    /// non-strict cereal reader: its parse (`file::typed_unstrict`) reads each scalar of the
    /// other type as the bound one, with an information, before the shape check, so the strict
    /// scalars of its reading never see one.
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        file::strict(match index {
            StructIndex::Stable(0) => Reading::DOCUMENT.with_unknown(UnknownMask::Warn),
            _ => Reading::DOCUMENT,
        })
    }
    type Latest = CameraPreset;
    type Beta = NoBeta<kind::CameraPreset>;
    type Component = NoComponents;
    type History = PresetHistory;

    fn identity(payload: JsonSlice<'_>) -> Identity {
        file::identity(payload, "identifier")
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(preset) = v {
            validate::preset(preset, cx);
        }
    }

    fn close(diagnostics: &mut Diagnostics, _: Option<JsonSlice<'_>>, _: &PackPath, failed: bool) {
        // With an unknown member vanilla logs its own line and nothing else: the only line
        // is 'Unrecognized field on camera preset!'.
        let unknown = diagnostics
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .any(|d| matches!(d.code.name(), "UnrecognizedField"));
        if failed && !unknown {
            closing_line(diagnostics, &codes::FAILED_PRESETS);
        }
    }
}

impl Family for ListFamily {
    const NAME: &'static str = "camera list";
    const PAYLOAD_KEY: &'static str = "minecraft:camera_list";
    const FLOOR: FormatVersion = V1_19_50;
    type Latest = CameraList;
    type Beta = NoBeta<kind::CameraList>;
    type Component = NoComponents;
    type History = ListHistory;
    const ENVELOPE: Envelope = Envelope {
        // A plain strict load of the whole file, which binds `format_version` and the payload
        // itself: no envelope, and a third root member is an unknown member.
        check: EnvelopeCheck::Never,
        // The required bind of `format_version`.
        missing: MissingVersion::Reject {
            code: &bedrock_addon_core::diagnostic::codes::FORMAT_VERSION_REQUIRED,
        },
        ..Envelope::DEFAULT
    };
    // The game reads a list without a document input: the input kind stays the first run's,
    // and the shape check rejects a `null` and an unknown member itself.

    fn identity(payload: JsonSlice<'_>) -> Identity {
        file::identity(payload, "identifier")
    }

    fn close(
        diagnostics: &mut Diagnostics,
        _: Option<JsonSlice<'_>>,
        path: &PackPath,
        failed: bool,
    ) {
        let found = std::mem::take(diagnostics);
        for mut finding in found {
            // A syntax finding gets the list's sentence ("Camera List file %s is not valid
            // JSON: %s"), the reader's message from its detail or line kept as its detail.
            if finding.code.name() == "MalformedInput" {
                let message = finding.syntax_message().unwrap_or_default().to_owned();
                finding.code = codes::LIST_NOT_JSON.with([path.as_str(), &message]);
                finding.detail = Some(Box::new(DiagnosticDetail::Syntax(message)));
            }
            // By legacy name: the variant and the definitions that were it.
            if finding.code.name() == "UnsupportedVersion" {
                // The version range is a constraint on the version string, not a slice of the
                // chain: the game's text for it is not known, so neither generic text stands in.
                finding.code = Code::from(&core_codes::UNSUPPORTED_VERSION);
            }
            diagnostics.push(finding);
        }
        let not_json = diagnostics
            .iter()
            // By legacy name: the variant and the definitions that were it.
            .any(|d| matches!(d.code.name(), "MalformedInput"));
        if failed && !not_json {
            closing_line(diagnostics, &codes::LIST_INVALID);
        }
    }

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        file::strict(Self::ENVELOPE.input.profile())
    }
}

impl Family for SplineFamily {
    const NAME: &'static str = "camera spline";
    const PAYLOAD_KEY: &'static str = "minecraft:camera_custom_splines";
    const FLOOR: FormatVersion = V1_26_0;
    const ENVELOPE: Envelope = Envelope {
        // The input is a document, as for presets.
        input: InputKind::Document,
        // The game reads the file as a document of `minecraft:camera_custom_splines` from 1.26.0:
        // the envelope is checked from the slice's version.
        check: EnvelopeCheck::From(V1_26_0),
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    type Latest = CameraSpline;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = SplineHistory;

    fn identity(_: JsonSlice<'_>) -> Identity {
        Identity::default()
    }

    fn close(diagnostics: &mut Diagnostics, _: Option<JsonSlice<'_>>, _: &PackPath, failed: bool) {
        if failed {
            closing_line(diagnostics, &codes::SPLINES_FAILED);
        }
    }

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        file::strict(Self::ENVELOPE.input.profile())
    }
}

impl Family for EntityFamily {
    const NAME: &'static str = "camera entity";
    const PAYLOAD_KEY: &'static str = "minecraft:camera_entity";
    // The document's version is 0.0.0, and that is a valid version: the envelope is always
    // checked.
    const FLOOR: FormatVersion = known::V0_0_0;
    type Latest = CameraEntity;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = EntityHistory;
    const ENVELOPE: Envelope = Envelope {
        // The input is a document, as for presets.
        input: InputKind::Document,
        missing: MissingVersion::REJECT,
        ..Envelope::DEFAULT
    };

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        file::strict(Self::ENVELOPE.input.profile())
    }
}

// ---- the documents ---------------------------------------------------------------------------

impl JsonDocument for CameraPreset {
    type Family = PresetFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
}

impl JsonPlaced for CameraPreset {
    /// `cameras/presets/<namespace>/<name>.json`.
    fn place(&self) -> PackPath {
        default_path(PRESETS, &self.identifier)
    }
}

impl JsonDefinition for CameraPreset {
    type Kind = kind::CameraPreset;
    fn identifier(&self) -> &CameraPresetId {
        &self.identifier
    }
}

impl JsonDocument for CameraList {
    type Family = ListFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
}

impl JsonPlaced for CameraList {
    /// `cameras/lists/<namespace>/<name>.json`.
    fn place(&self) -> PackPath {
        default_path(LISTS, &self.identifier)
    }
}

impl JsonDefinition for CameraList {
    type Kind = kind::CameraList;
    fn identifier(&self) -> &CameraListId {
        &self.identifier
    }
}

impl JsonDocument for CameraSpline {
    type Family = SplineFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
}

impl JsonDocument for CameraEntity {
    type Family = EntityFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];

    /// The version vanilla's own camera entity documents carry.
    fn write_version(&self) -> FormatVersion {
        ENTITY_WRITTEN
    }
}

// ---- the group -----------------------------------------------------------------------------

/// What one walked file parses to.
#[doc(hidden)]
#[derive(Debug)]
pub enum CameraFile {
    /// A `minecraft:camera_preset` file.
    Preset(Box<Contribution<CameraPreset>>),
    /// A `minecraft:camera_list` file.
    List(Box<Contribution<CameraList>>),
    /// A `minecraft:camera_custom_splines` file.
    Spline(Box<Result<Document<CameraSpline>, Rejected>>),
}

/// `cameras/{presets, lists, splines}/**/*.json`: every pack's presets, then every pack's
/// lists, then the splines, packs from the lowest priority up, so a later definition (the
/// higher pack's) replaces an earlier one, as most of the game's loaders visit a stack.
const WALK: Walk = Walk::json_tree_in(&[PRESETS, LISTS, SPLINES]).directory_major();

/// Parses one walked file by its directory.
fn parse_walk_file(walked: &WalkFile, cx: &LoadContext) -> CameraFile {
    match WALK.matches(&walked.key) {
        // Presets and lists: the parse of the registry constructor.
        Some(0) => CameraFile::Preset(Box::new((load::registry(WALK).parse)(walked, cx))),
        Some(1) => CameraFile::List(Box::new((load::registry(WALK).parse)(walked, cx))),
        _ => CameraFile::Spline(Box::new(
            parse_document::<SplineFamily>(origin_of(walked), &walked.bytes, cx).stable(),
        )),
    }
}

#[doc(hidden)]
/// The loader of `bp.cameras()`: the three directories of every behaviour pack, combined into
/// the group.
pub const LOADER: CollectionLoader<Cameras, CameraFile> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine: stack::combine,
    needs: &[],
    then: None,
};

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
    const VERSION: FormatVersion = known::V0_0_0;
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn preset_reads_as_pinned() {
        assert_reading_table(
            "camera::PresetFamily",
            &reading_table::<super::PresetFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: Never
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.19.50..=1.19.50 (0..=0): json Rejected/Strict/Error/Warn/reports/fatal/Refuse, mcb Own/Strict/Error/Warn/reports/fatal/Refuse
                1.26.50..=1.26.50 (1..=1): json Rejected/Strict/Error/Error/reports/fatal/Refuse, mcb Own/Strict/Error/Error/reports/fatal/Refuse
            ",
        );
    }

    #[test]
    fn list_reads_as_pinned() {
        assert_reading_table(
            "ListFamily",
            &reading_table::<super::ListFamily>(),
            r#"
                text: Root, clamps beyond a double: false, duplicates kept in: []
                envelope: Never
                missing version: rejected: Fixed("missing required field")
                below floor: UseFormatVersion
                empty document: Error
                1.19.50..=1.19.50 (0..=0): json Own/Strict/Error/Warn/reports/lenient/Refuse, mcb Own/Strict/Error/Warn/reports/lenient/Refuse
            "#,
        );
    }

    #[test]
    fn spline_reads_as_pinned() {
        assert_reading_table(
            "SplineFamily",
            &reading_table::<super::SplineFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: From(FormatVersion { major: 1, minor: 26, patch: 0 })
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.26.0..=1.26.0 (0..=0): json Rejected/Strict/Error/Error/reports/fatal/Refuse, mcb Own/Strict/Error/Error/reports/fatal/Refuse
            ",
        );
    }

    #[test]
    fn entity_reads_as_pinned() {
        assert_reading_table(
            "EntityFamily",
            &reading_table::<super::EntityFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: Always
                missing version: rejected: NoValidVersion
                below floor: UseFormatVersion
                empty document: Error
                0.0.0..=0.0.0 (0..=0): json Rejected/Strict/Error/Error/reports/fatal/Refuse, mcb Own/Strict/Error/Error/reports/fatal/Refuse
            ",
        );
    }
}

#[cfg(test)]
mod identity {
    //! What the family's documents say about themselves before they are parsed.

    use bedrock_addon_core::json::JsonSlice;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::versioning::{Family, Identity, parse_as_written};

    /// The family reads no identity from its payload, not even a `description.identifier`
    /// written in it (the default would).
    #[test]
    fn spline_reads_no_identity_from_the_payload() {
        let text = format!(
            r#"{{"format_version": "1.21.0", "{}": {{"description": {{"identifier": "demo:x"}}}}}}"#,
            <super::SplineFamily as Family>::PAYLOAD_KEY,
        );
        let parsed = parse_as_written::<super::SplineFamily>(
            JsonSlice::new(&text),
            &LoadContext::detached(),
        );
        assert_eq!(parsed.identity, Identity::default());
    }
}
