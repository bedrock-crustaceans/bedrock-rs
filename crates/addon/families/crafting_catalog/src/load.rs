//! Loading the crafting item catalogue: the family's chain parameters, `Content` and `Placed`
//! of [`CraftingCatalog`].

use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

use super::history::CatalogAny;
use super::parse::parse_catalog;
use super::{CraftingCatalog, PATH, V1_21_60, V1_26_50};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::{Content, Document, JsonContent, Origin, Placed, Rejected};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, MAX_JSON_DEPTH, Value, WriteStyle, check_depth, strip_bom, write_json,
};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{
    ENGINE_VERSION, FormatVersion, FormatVersionInput, PickedVersion,
};
use bedrock_addon_core::versioning::{
    ErrorMask, Family, Identity, InputKind, Loaded, MissingVersion, NoComponents, ParseCx, Parsed,
    Selection, StructIndex, VersionStruct, Versioned, finish, parse_as_written, select,
};

/// Vanilla's one text for a file without a valid `format_version`, and for one below 1.21.60.
const VERSION_TEXT: &str = "The file cannot be processed without a valid format_version";

/// Vanilla's text for a document whose payload member is not the catalogue's.
const PAYLOAD_TEXT: &str =
    "The crafting item catalog needs to have an object minecraft:crafting_items_catalog";

/// The catalogue family of the version-chain machinery (hidden: only the loader names it).
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct CatalogFamily;

impl Family for CatalogFamily {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with a warning at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "crafting item catalog";
    const PAYLOAD_KEY: &'static str = "minecraft:crafting_items_catalog";
    const DIRECTORIES: &'static [&'static str] = &["item_catalog"];
    // The game registers no slice below the first struct's version.
    const FLOOR: FormatVersion = V1_21_60;
    type Latest = CraftingCatalog;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = CatalogAny;
    // Vanilla's loader checks the version itself, before the versioned parse.
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    const MISSING_VERSION_TEXT: Option<VanillaText> = Some(VanillaText::Fixed(VERSION_TEXT));
    // The game reads the file as a whole document: the version is the one its JSON reader picks
    // from the text, the size limits apply, and a `null` where a value is read counts as the
    // wrong JSON type (the hand reader below refuses it).
    const INPUT: InputKind = InputKind::Document;

    /// An unknown member is a warning below 1.26.50, where the game reads the file leniently,
    /// and an error that skips the file from 1.26.50 on, where it reads it strictly.
    fn error_mask(index: StructIndex) -> ErrorMask {
        match index {
            StructIndex::Stable(0) => ErrorMask::Unstrict,
            _ => ErrorMask::Strict,
        }
    }
}

impl Versioned for CraftingCatalog {
    const VERSION: FormatVersion = V1_26_50;
}

impl VersionStruct for CraftingCatalog {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_catalog(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        Ok(self.payload())
    }
}

impl bedrock_addon_core::__private::Sealed for CraftingCatalog {}

impl Content for CraftingCatalog {
    const NAME: &'static str = "crafting item catalog";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        write_json(&self.document(), out, style)
    }
}

impl JsonContent for CraftingCatalog {}

impl Placed for CraftingCatalog {
    /// `item_catalog/crafting_item_catalog.json`: the one path vanilla reads.
    fn default_path(&self) -> PackPath {
        PackPath::from_static(PATH)
    }
}

/// The text of a file, after the BOM and the depth check; `None` when it is not UTF-8 or too
/// deep.
fn text(bytes: &[u8]) -> Option<&str> {
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).ok()?;
    std::str::from_utf8(bytes).ok()
}

/// The error of a file without a valid `format_version`.
fn no_valid_version() -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MissingFormatVersion {
        text: Some(VanillaText::Fixed(VERSION_TEXT)),
    })
}

/// The error of a `format_version` below 1.21.60: vanilla's same text.
fn below_floor() -> Diagnostic {
    Diagnostic::error(DiagnosticCode::UnsupportedVersion {
        text: Some(VanillaText::Fixed(VERSION_TEXT)),
    })
}

/// Gives the findings of a rejected envelope vanilla's codes, texts and order.
/// Vanilla's loader checks, before the versioned parse: a valid version (a syntax error,
/// a malformed version and, at or above 1.21.60, a root without exactly two members all
/// clear the version), then the floor, then the payload key; the engine version is the
/// parser's. The envelope's own text of a cleared version is kept as an info-level note
/// (vanilla writes it to a logger it never prints).
fn vanilla_checks(diagnostics: &mut Diagnostics, document: JsonSlice<'_>) {
    // The version the game's JSON reader picks from the text.
    let written = match FormatVersionInput::pick(document.text) {
        PickedVersion::Version { input, .. } => Some(input),
        PickedVersion::Absent | PickedVersion::Malformed => None,
    };
    let floor = written.is_some_and(|w| select::<CatalogFamily>(&w) == Selection::BelowFloor);
    let above_engine = written
        .and_then(|w| w.version)
        .is_some_and(|version| version > ENGINE_VERSION);
    let found = std::mem::take(diagnostics);
    for mut finding in found {
        match &mut finding.code {
            // Below its minimum version vanilla does not look at the shape of the root.
            DiagnosticCode::MalformedInput {
                text: Some(VanillaText::DocumentMemberCount),
                ..
            } if floor => diagnostics.push(below_floor()),
            DiagnosticCode::MalformedInput {
                text: Some(VanillaText::DocumentMemberCount | VanillaText::DocumentJson { .. }),
                ..
            }
            | DiagnosticCode::MalformedFormatVersion { .. }
            | DiagnosticCode::EmptyDocument { .. } => {
                finding.severity = Severity::Info;
                diagnostics.push(finding);
                diagnostics.push(no_valid_version());
            }
            DiagnosticCode::MissingPayload { .. } => diagnostics.push(if written.is_none() {
                no_valid_version()
            } else if floor {
                below_floor()
            } else {
                Diagnostic::error(DiagnosticCode::MissingRequiredField {
                    text: Some(VanillaText::Fixed(PAYLOAD_TEXT)),
                })
            }),
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::UseFormatVersion { .. }),
            } => diagnostics.push(below_floor()),
            // the versioned parser's "no parser available for version".
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion { version }),
            } if above_engine => {
                finding.code = DiagnosticCode::FormatVersionAboveEngine {
                    version: std::mem::take(version),
                };
                diagnostics.push(finding);
            }
            _ => diagnostics.push(finding),
        }
    }
}

impl CraftingCatalog {
    /// Parses the catalogue file without upgrading it: the struct its `format_version` selects
    ///; [`CatalogAny::upgrade`] runs the rest of the load.
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
    ) -> Result<Document<CatalogAny>, Rejected> {
        let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
        let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
        let Some(text) = text(bytes) else {
            return Err(unreadable(origin));
        };
        let document = JsonSlice::new(text);
        let mut parsed = parse_as_written::<CatalogFamily>(document, cx);
        if parsed.value.is_none() {
            vanilla_checks(&mut parsed.diagnostics, document);
        }
        let mut diagnostics = parsed.diagnostics;
        diagnostics.locate(&origin);
        match parsed.value {
            Some(value) => Ok(Document::new(value, origin, parsed.version, diagnostics)),
            None => Err(Rejected::new(origin, diagnostics)),
        }
    }
}

/// `CatalogAny::upgrade`: rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<CatalogAny>,
    cx: &LoadContext,
) -> Result<Document<CraftingCatalog>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let mut at = JsonPath::new();
    at.push(CatalogFamily::PAYLOAD_KEY);
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<CatalogFamily>::new(
        Some(value),
        version,
        written,
        Identity::default(),
        ActorType::Mob,
        false,
        at,
        diagnostics,
    );
    let outcome = finish(parsed, cx);
    match outcome.value {
        Some(Loaded::Stable(catalog)) => Ok(Document::new(
            catalog,
            origin,
            outcome.version,
            outcome.diagnostics,
        )),
        _ => Err(Rejected::new(origin, outcome.diagnostics)),
    }
}

/// The rejection of a file that is not UTF-8 JSON within the nesting limit: not JSON to
/// vanilla either, so the version is cleared.
fn unreadable(origin: Origin) -> Rejected {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::info(DiagnosticCode::MalformedInput {
        message: "not UTF-8 JSON within the nesting limit".to_owned(),
        text: None,
    }));
    diagnostics.push(no_valid_version());
    Rejected::located(origin, diagnostics)
}

/// Loads the catalogue file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<CraftingCatalog>, Rejected> {
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let origin = Origin::file(pack, side, path.clone());
    let Some(text) = text(bytes) else {
        return Err(unreadable(origin));
    };
    let document = JsonSlice::new(text);
    let mut parsed = parse_as_written::<CatalogFamily>(document, cx);
    if parsed.value.is_none() {
        vanilla_checks(&mut parsed.diagnostics, document);
    }
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(catalog)) => {
            Ok(Document::new(catalog, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}
