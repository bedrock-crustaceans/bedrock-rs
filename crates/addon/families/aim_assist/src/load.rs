//! Loading the aim-assist documents: the two families' chain parameters, `Content` /
//! `Definition`, the walk of `aim_assist/` and the group's combine rule.

use std::convert::Infallible;

use super::history::{Any, CategoriesAny, CategoriesFamily, PresetFamily};
use super::{AimAssist, AimAssistCategories, AimAssistCategory, AimAssistPreset, parse};
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, NoBeta, Notice, Placed,
    RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::{Id, kind};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, write_json};
use bedrock_addon_core::load::Reported;
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    ComponentTable, EnvelopeCheck, ErrorMask, Family, Identity, InputKind, Loaded, MissingVersion,
    NoComponents, StructIndex, ValidateCx, VersionStruct, Versioned,
};
use bedrock_addon_support::chain_file::{self as file, FileFamily, FileOutcome, puv_below_floor};

/// `aim_assist/categories`: `minecraft:aim_assist_categories` documents.
const CATEGORIES: &str = "aim_assist/categories";
/// `aim_assist/presets`: `minecraft:aim_assist_preset` documents.
const PRESETS: &str = "aim_assist/presets";

/// The first version of both families (`SCHEMA_PRESET_VERSION_1`, `SCHEMA_CATEGORY_VERSION_1`).
pub(crate) const V1_21_50: FormatVersion = FormatVersion::new(1, 21, 50);
/// The second preset struct (`SCHEMA_PRESET_VERSION_2`).
pub(crate) const V1_21_120: FormatVersion = FormatVersion::new(1, 21, 120);

impl Family for PresetFamily {
    const NAME: &'static str = "aim assist preset";
    // The game's loader passes the categories key here and never checks it; the struct binds the
    // real key.
    const PAYLOAD_KEY: &'static str = "minecraft:aim_assist_preset";
    const DIRECTORIES: &'static [&'static str] = &[PRESETS];
    const FLOOR: FormatVersion = V1_21_50;
    type Latest = AimAssistPreset;
    type Beta = Infallible;
    type Component = NoComponents;
    type Any = Any;
    // An invalid minimum version: the text stays raw and the strict root struct reads the whole
    // document.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Never;
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // The game reads a preset through the document-input reader of the cereal-based families.
    const INPUT: InputKind = InputKind::Document;

    /// Both slices are read with the strict mask: an unknown member or a wrong type rejects the
    /// file.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Strict
    }

    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }

    fn identity(payload: JsonSlice<'_>) -> Identity {
        file::identity(payload, "identifier")
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(preset) = v {
            parse::preset_namespaces(preset, cx);
        }
    }
}

impl FileFamily for PresetFamily {
    fn close(
        diagnostics: &mut Diagnostics,
        document: Option<JsonSlice<'_>>,
        _: &PackPath,
        _: bool,
    ) {
        puv_below_floor(diagnostics, document);
    }

    fn document(value: &AimAssistPreset) -> Result<Value, EncodeError> {
        value.write()
    }
}

impl Family for CategoriesFamily {
    const NAME: &'static str = "aim assist categories";
    const PAYLOAD_KEY: &'static str = "minecraft:aim_assist_categories";
    const DIRECTORIES: &'static [&'static str] = &[CATEGORIES];
    const FLOOR: FormatVersion = V1_21_50;
    type Latest = AimAssistCategories;
    type Beta = Infallible;
    type Component = NoComponents;
    type Any = CategoriesAny;
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Never;
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    // The game reads a categories file through the document-input reader of the cereal-based
    // families.
    const INPUT: InputKind = InputKind::Document;

    /// The one slice is read with the strict mask: an unknown member or a wrong type rejects the
    /// file.
    fn error_mask(_: StructIndex) -> ErrorMask {
        ErrorMask::Strict
    }

    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }

    fn identity(_: JsonSlice<'_>) -> Identity {
        Identity::default()
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(categories) = v {
            parse::category_namespaces(categories, cx);
        }
    }
}

impl FileFamily for CategoriesFamily {
    fn close(
        diagnostics: &mut Diagnostics,
        document: Option<JsonSlice<'_>>,
        _: &PackPath,
        _: bool,
    ) {
        puv_below_floor(diagnostics, document);
    }

    fn document(value: &AimAssistCategories) -> Result<Value, EncodeError> {
        value.write()
    }
}

/// The `Content` impl of a root type and its `parse_as_written`.
macro_rules! content {
    ($root:ty, $family:ty, $any:ty, name = $name:literal) => {
        impl bedrock_addon_core::__private::Sealed for $root {}

        impl Content for $root {
            const NAME: &'static str = $name;
            const SIDES: &'static [PackSide] = &[PackSide::Behavior];

            fn parse(
                path: &PackPath,
                bytes: &[u8],
                cx: &LoadContext,
            ) -> Result<Document<Self>, Rejected> {
                file::parse_file::<$family>(
                    PackRef::app_data(),
                    PackSide::Behavior,
                    path,
                    bytes,
                    cx,
                )
                .1
            }

            fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
                write_json(&<$family as FileFamily>::document(self)?, out, style)
            }

            fn to_json(&self) -> Result<String, EncodeError> {
                let mut out = Vec::new();
                self.write_json(&mut out, WriteStyle::default())?;
                Ok(String::from_utf8(out).unwrap_or_default())
            }
        }

        impl $root {
            /// Parses one file without upgrading it: the struct its `format_version` selects
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
                file::as_written::<$family>(PackSide::Behavior, path, bytes, cx)
            }
        }
    };
}

content!(
    AimAssistPreset,
    PresetFamily,
    Any,
    name = "aim assist preset"
);
content!(
    AimAssistCategories,
    CategoriesFamily,
    CategoriesAny,
    name = "aim assist categories"
);

impl Definition for AimAssistPreset {
    type Kind = kind::AimAssistPreset;
    type File = AimAssistPreset;
    type Beta = NoBeta<kind::AimAssistPreset>;
}

impl Placed for AimAssistPreset {
    /// `aim_assist/presets/<name>.json`: vanilla reads only the files directly in the folder.
    fn default_path(&self) -> PackPath {
        flat_path(PRESETS, default_path(PRESETS, &self.identifier))
    }
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
#[derive(Debug)]
pub enum AimAssistFile {
    /// A `minecraft:aim_assist_categories` file.
    Categories(Box<Result<Document<AimAssistCategories>, Rejected>>),
    /// A `minecraft:aim_assist_preset` file.
    Preset(Box<FileOutcome<AimAssistPreset>>),
}

/// An aim-assist file: an extension exactly `json` (a case-sensitive compare; `x.JSON` is
/// skipped without a message).
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// `aim_assist/categories/*.json`, then `aim_assist/presets/*.json`, the files directly in each
/// folder (not their subfolders), from the last stack entry to the first: a later load
/// replaces an earlier one, so the highest-priority pack wins.
const WALK: Walk = Walk {
    directories: &[CATEGORIES, PRESETS],
    depth: ListDepth::Children,
    accept: is_json,
    order: WalkOrder::BottomUp,
    directory_major: true,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// Parses one walked file by its directory.
fn parse_walk_file(walked: &WalkFile, cx: &LoadContext) -> AimAssistFile {
    let (pack, side, path, bytes) = (
        walked.pack.clone(),
        walked.side,
        &walked.path,
        &walked.bytes,
    );
    match WALK.matches(&walked.key) {
        Some(0) => AimAssistFile::Categories(Box::new(
            file::parse_file::<CategoriesFamily>(pack, side, path, bytes, cx).1,
        )),
        _ => AimAssistFile::Preset(Box::new(file::parse_file::<PresetFamily>(
            pack, side, path, bytes, cx,
        ))),
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
            AimAssistFile::Preset(outcome) => match *outcome {
                (_, Ok(document)) => {
                    presets.push(Contribution::Stable(document.identifier.clone(), document));
                }
                (identifier, Err(rejected)) => {
                    presets.push(Contribution::Rejected(identifier.map(Id::parse), rejected));
                }
            },
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

/// The loader of `bp.aim_assist()`: the two directories of every behaviour pack, combined into
/// the group.
pub const fn load_group() -> CollectionLoader<AimAssist, AimAssistFile> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}

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
