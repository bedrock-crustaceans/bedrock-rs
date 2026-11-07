//! Camera aim assist (`AimAssist`, `AimAssistPreset`, `AimAssistCategories`): the two document
//! kinds below `aim_assist/` of a behaviour pack, presets (`aim_assist/presets/`,
//! `minecraft:aim_assist_preset`) and category files (`aim_assist/categories/`,
//! `minecraft:aim_assist_categories`), loaded together as the `AimAssist` group.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
pub use categories::{AimAssistCategories, AimAssistCategory, Priorities};
#[doc(hidden)]
pub use link::{Targets, link, link_camera_presets};
pub use preset::{AimAssistPreset, ExclusionSettings};

mod categories;
mod checks;
pub mod codes;
mod link;
mod preset;
mod read;

#[cfg(test)]
mod tests;

use std::convert::Infallible;

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::Registry;
use bedrock_addon_core::document::unversioned::origin_of;
use bedrock_addon_core::document::{
    CombineRule, Contribution, Definition, Document, JsonDefinition, JsonDocument, JsonPlaced,
    NoBeta, Notice, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::{AimAssistPresetId, kind};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::{self, CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::load::{DocumentSource, Reported};
use bedrock_addon_core::pack_identity::{PackPath, PackSide};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::chain_file as file;
use bedrock_addon_core::versioning::{
    BelowFloor, Envelope, EnvelopeCheck, Family, Identity, InputKind, Loaded, MissingVersion,
    NoComponents, Reading, StructIndex, ValidateCx, Versioned, parse_document,
};
use history::{CategoriesFamily, CategoriesHistory, PresetFamily, PresetHistory};

/// The aim-assist documents of the behaviour packs (`bp.aim_assist()`): two collections loaded
/// and used together.
///
/// Presets are keyed by their `identifier`, categories by their `name`; the highest-priority
/// pack's definition wins, whole. Vanilla reads only the files directly in `aim_assist/presets/`
/// and `aim_assist/categories/` whose extension is exactly `json`. A preset file holds one
/// preset; a categories file holds several categories and loads all of them or none.
#[derive(Debug)]
#[non_exhaustive]
pub struct AimAssist {
    /// The aim-assist presets (`aim_assist/presets/*.json`).
    pub presets: Registry<AimAssistPreset>,
    /// The aim-assist categories (`aim_assist/categories/*.json`), one entry per category.
    pub categories: Registry<AimAssistCategory>,
}

impl AimAssist {
    /// Every finding of the two collections, categories first (vanilla's load order).
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.categories
            .diagnostics()
            .chain(self.presets.diagnostics())
    }
}

/// `aim_assist/categories`: `minecraft:aim_assist_categories` documents.
const CATEGORIES: &str = "aim_assist/categories";
/// `aim_assist/presets`: `minecraft:aim_assist_preset` documents.
const PRESETS: &str = "aim_assist/presets";

/// The first version of both families (`SCHEMA_PRESET_VERSION_1`, `SCHEMA_CATEGORY_VERSION_1`).
pub(crate) const V1_21_50: FormatVersion = FormatVersion::new(1, 21, 50);
/// The second preset struct (`SCHEMA_PRESET_VERSION_2`).
pub(crate) const V1_21_120: FormatVersion = known::V1_21_120;

impl Family for PresetFamily {
    const NAME: &'static str = "aim assist preset";
    // The game's loader passes the categories key here and never checks it; the struct binds the
    // real key.
    const PAYLOAD_KEY: &'static str = "minecraft:aim_assist_preset";
    const FLOOR: FormatVersion = V1_21_50;
    const ENVELOPE: Envelope = Envelope {
        // The game reads a preset through the document-input reader of the cereal-based families.
        input: InputKind::Document,
        // An invalid minimum version: the text stays raw and the strict root struct reads the whole
        // document.
        check: EnvelopeCheck::Never,
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    type Latest = AimAssistPreset;
    type Beta = NoBeta<kind::AimAssistPreset>;
    type Component = NoComponents;
    type History = PresetHistory;

    fn identity(payload: JsonSlice<'_>) -> Identity {
        file::identity(payload, "identifier")
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(preset) = v {
            read::preset_namespaces(preset, cx);
        }
    }

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        file::strict(Self::ENVELOPE.input.profile())
    }
}

impl Family for CategoriesFamily {
    const NAME: &'static str = "aim assist categories";
    const PAYLOAD_KEY: &'static str = "minecraft:aim_assist_categories";
    const FLOOR: FormatVersion = V1_21_50;
    const ENVELOPE: Envelope = Envelope {
        // The game reads a categories file through the document-input reader of the cereal-based
        // families.
        input: InputKind::Document,
        check: EnvelopeCheck::Never,
        missing: MissingVersion::REJECT,
        // Below the first slice the versioned loader has no parser at all.
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    type Latest = AimAssistCategories;
    type Beta = Infallible;
    type Component = NoComponents;
    type History = CategoriesHistory;

    fn identity(_: JsonSlice<'_>) -> Identity {
        Identity::default()
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(categories) = v {
            read::category_namespaces(categories, cx);
        }
    }

    fn reading(_: StructIndex, _: DocumentSource) -> Reading {
        file::strict(Self::ENVELOPE.input.profile())
    }
}

impl JsonDocument for AimAssistPreset {
    type Family = PresetFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
}

impl JsonPlaced for AimAssistPreset {
    /// `aim_assist/presets/<name>.json`: vanilla reads only the files directly in the folder.
    fn place(&self) -> PackPath {
        flat_path(PRESETS, default_path(PRESETS, &self.identifier))
    }
}

impl JsonDefinition for AimAssistPreset {
    type Kind = kind::AimAssistPreset;
    fn identifier(&self) -> &AimAssistPresetId {
        &self.identifier
    }
}

/// A categories file holds several categories: the file is the document, each category a
/// definition of its own (a hand `Definition`, its file the categories file).
impl JsonDocument for AimAssistCategories {
    type Family = CategoriesFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
}

impl bedrock_addon_core::__private::Sealed for AimAssistCategory {}

impl Definition for AimAssistCategory {
    type Kind = kind::AimAssistCategory;
    type File = AimAssistCategories;
    type Beta = NoBeta<kind::AimAssistCategory>;
}

/// `nested` (a path below `directory` with a namespace folder) as a file directly in
/// `directory`: the folders become part of the file name, joined by `_`.
fn flat_path(directory: &str, nested: PackPath) -> PackPath {
    let rest = nested
        .as_str()
        .strip_prefix(directory)
        .map_or(nested.as_str(), |rest| rest.trim_start_matches('/'));
    PackPath::new(&format!("{directory}/{}", rest.replace('/', "_"))).unwrap_or(nested)
}

// ---- the group -----------------------------------------------------------------------------

/// What one walked file parses to.
#[doc(hidden)]
#[derive(Debug)]
pub enum AimAssistFile {
    /// A `minecraft:aim_assist_categories` file.
    Categories(Box<Result<Document<AimAssistCategories>, Rejected>>),
    /// A `minecraft:aim_assist_preset` file.
    Preset(Box<Contribution<AimAssistPreset>>),
}

/// `aim_assist/categories/*.json`, then `aim_assist/presets/*.json`, the files directly in each
/// folder (not their subfolders), from the last stack entry to the first: a later load
/// replaces an earlier one, so the highest-priority pack wins.
const WALK: Walk = Walk::json_children_in(&[CATEGORIES, PRESETS]).directory_major();

/// Parses one walked file by its directory.
fn parse_walk_file(walked: &WalkFile, cx: &LoadContext) -> AimAssistFile {
    match WALK.matches(&walked.key) {
        Some(0) => AimAssistFile::Categories(Box::new(
            parse_document::<CategoriesFamily>(origin_of(walked), &walked.bytes, cx).stable(),
        )),
        // The parse of the registry constructor.
        _ => AimAssistFile::Preset(Box::new((load::registry(WALK).parse)(walked, cx))),
    }
}

/// A registry in which the later load of an identifier replaces the earlier one whole
/// (the game's registries assign or emplace the same way); the replaced one is a
/// `ShadowedDefinition` info.
fn last_loaded_wins<T: Definition>(stack: &StackCx<'_>) -> RegistryBuilder<T> {
    RegistryBuilder::new(
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        },
        CombineRule::last_loaded_wins(),
        stack.options,
    )
}

/// The group, from every parsed file in walk order.
fn combine(parsed: Vec<AimAssistFile>, stack: &StackCx<'_>) -> AimAssist {
    let mut presets = last_loaded_wins::<AimAssistPreset>(stack);
    let mut categories = last_loaded_wins::<AimAssistCategory>(stack);
    for parsed_file in parsed {
        match parsed_file {
            AimAssistFile::Preset(contribution) => presets.push(*contribution),
            AimAssistFile::Categories(outcome) => match *outcome {
                Ok(document) => {
                    // One document per category; the file's findings go to the registry once.
                    for finding in document.diagnostics().iter().cloned() {
                        categories.push_diagnostic(finding);
                    }
                    let (origin, version) = (document.origin().clone(), document.version().clone());
                    for category in document.into_value().categories {
                        let id = category.name.clone();
                        let one = Document::new(
                            category,
                            origin.clone(),
                            version.clone(),
                            Diagnostics::default(),
                        );
                        categories.push(Contribution::Stable(id, one));
                    }
                }
                Err(rejected) => categories.push(Contribution::Rejected(None, rejected)),
            },
        }
    }
    AimAssist {
        presets: presets.finish(),
        categories: categories.finish(),
    }
}

#[doc(hidden)]
/// The loader of `bp.aim_assist()`: the two directories of every behaviour pack, combined into
/// the group.
pub const LOADER: CollectionLoader<AimAssist, AimAssistFile> = CollectionLoader {
    walk: WALK,
    parse: parse_walk_file,
    combine,
    needs: &[],
    then: None,
};

impl Reported for AimAssist {
    fn reported(&self) -> Box<dyn Iterator<Item = &Diagnostic> + '_> {
        Box::new(self.diagnostics())
    }
}

impl Versioned for AimAssistPreset {
    const VERSION: FormatVersion = V1_21_120;
}

impl Versioned for AimAssistCategories {
    const VERSION: FormatVersion = V1_21_50;
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn preset_reads_as_pinned() {
        assert_reading_table(
            "aim_assist::PresetFamily",
            &reading_table::<super::PresetFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: Never
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.50..=1.21.120 (0..=1): json Rejected/Strict/Error/Error/reports/fatal/Refuse, mcb Own/Strict/Error/Error/reports/fatal/Refuse
            ",
        );
    }

    #[test]
    fn categories_reads_as_pinned() {
        assert_reading_table(
            "CategoriesFamily",
            &reading_table::<super::CategoriesFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: Never
                missing version: rejected: NoValidVersion
                below floor: NoParser
                empty document: Error
                1.21.50..=1.21.50 (0..=0): json Rejected/Strict/Error/Error/reports/fatal/Refuse, mcb Own/Strict/Error/Error/reports/fatal/Refuse
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
    fn categories_reads_no_identity_from_the_payload() {
        let text = format!(
            r#"{{"format_version": "1.21.0", "{}": {{"description": {{"identifier": "demo:x"}}}}}}"#,
            <super::CategoriesFamily as Family>::PAYLOAD_KEY,
        );
        let parsed = parse_as_written::<super::CategoriesFamily>(
            JsonSlice::new(&text),
            &LoadContext::detached(),
        );
        assert_eq!(parsed.identity, Identity::default());
    }
}
