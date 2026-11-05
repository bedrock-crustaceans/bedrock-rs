//! Trade tables: `TradeTable`, `Tier`, `Trade`.
//!
//! Trade tables: `trading/**/*.json` of a behaviour pack, referenced from an entity's
//! `minecraft:trade_table` / `minecraft:economy_trade_table` (`table`). A document is
//! `{"format_version"?, "tiers": [...]}`; `TradeTable` is the newest form (format 1.21.30), and
//! older files (no `format_version`, or one below 1.21.30) are read by vanilla's legacy parser
//! and upgraded when they load (`history`).

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;

mod read;
mod texts;
mod types;
mod validate;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, JsonPath};
use bedrock_addon_core::document::{
    Document, Extra, Files, JsonDocument, Origin, Rejected, VersionInfo,
};
use bedrock_addon_core::json::Prepared;
use bedrock_addon_core::json::{EncodeError, EnvelopeVersion, JsonSlice, Value, probe_envelope};
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::load::{self, CollectionLoader, FileCopy, LoadContext, Walk};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    DocumentRead, Envelope, EnvelopeCheck, Family, Identity, InputKind, Loaded, MissingVersion,
    NoComponents, ParseCx, Parsed, Reading, StructIndex, ValidateCx, VersionChain, VersionStruct,
    finish, parse_as_written, write_envelope, write_payload,
};
use history::{Any, TradeFamily, v0};
use read::CEREAL;

pub use types::{Group, ItemList, Quantity, Tier, Trade, TradeItem, TradeTable};

/// The pack directory of trade tables.
const DIRECTORY: &str = "trading";

impl Family for TradeFamily {
    const NAME: &'static str = "trade table";
    // The document is `{"format_version"?, "tiers": [...]}`; the game binds the table to the
    // `tiers` member alone, so the payload is the array.
    const PAYLOAD_KEY: &'static str = "tiers";
    // The legacy slice starts at 0.0.0.
    const FLOOR: FormatVersion = v0::VERSION;
    type Latest = TradeTable;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type Any = Any;
    const ENVELOPE: Envelope = Envelope {
        // The game reads a table as a whole document: the version is the one its JSON reader picks
        // from the text, the size limits apply, and rule N2 holds at the cereal slice.
        input: InputKind::Document,
        // The legacy parser reads the whole document and ignores other root members; the strict
        // cereal slice (1.21.30 on) checks the envelope.
        check: EnvelopeCheck::From(CEREAL),
        // A missing `format_version` is 0.0.0 with no log entry.
        missing: MissingVersion::Floor {
            text: None,
            severity: None,
        },
        ..Envelope::DEFAULT
    };

    /// The legacy slice `[0.0.0, 1.21.30)` is a hand-written parser with no cereal mask: its
    /// `null` handling is the family's own. The game reads the cereal slice strictly, under rule
    /// N2: an unknown member is an error there.
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        match index {
            StructIndex::Stable(0) => Reading::OWN,
            _ => Reading::DOCUMENT,
        }
    }

    fn identity(_: JsonSlice<'_>) -> Identity {
        Identity::default()
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        if let Loaded::Stable(table) = v {
            validate::validate(table, cx);
        }
    }
}

// `JsonDocument` with its own `write_root`: a table that holds a legacy blob is written as a
// whole legacy document, a payload other than `VersionStruct::write`'s `tiers`; and its own
// `read` and `parse_as_written`: the parse stage reads two cases of the text the generic stage
// does not (`parse_stage`). No `JsonPlaced`: the writer is not given tables by value.
impl JsonDocument for TradeTable {
    type Family = TradeFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// The whole document: `format_version` (1.21.30, the only struct from it on) and `tiers`;
    /// a table that holds a legacy blob in the legacy shape instead (`v0::legacy_form`: the
    /// newest format has the blobs only under deprecated names the game rejects).
    fn write_root(&self) -> Result<Value, EncodeError> {
        if let Some(legacy) = history::v0::legacy_form(self)? {
            return write_payload(&legacy);
        }
        Ok(write_envelope::<TradeFamily>(
            CEREAL,
            VersionStruct::write(self)?,
            &Extra::default(),
        ))
    }

    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<TradeFamily> {
        let read = match parse_stage(bytes, cx) {
            Ok(parsed) => finished(parsed, origin, cx),
            Err(finding) => Err(reject(origin, *finding)),
        };
        match read {
            Ok(document) => DocumentRead::Stable(document),
            Err(rejected) => DocumentRead::Rejected(None, rejected),
        }
    }

    /// Parses one trade-table file without upgrading it: the struct its `format_version`
    /// selects.
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file cannot be read into a value at all, such as bytes that
    /// are not JSON or a document the family refuses; it carries every finding about the file.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
        let parsed = match parse_stage(bytes, cx) {
            Ok(parsed) => parsed,
            Err(finding) => return Err(reject(origin, *finding)),
        };
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
) -> Result<Document<TradeTable>, Rejected> {
    let origin = document.origin().clone();
    let version = document.version().clone();
    let diagnostics = document.diagnostics().clone();
    let value = document.into_value();
    let at = match value {
        Any::V0(_) => JsonPath::new(),
        Any::V1_21_30(_) => payload_at(),
    };
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<TradeFamily>::from_value(Some(value), version, written, at, diagnostics);
    finished(parsed, origin, cx)
}

/// The path of the payload in a checked envelope.
fn payload_at() -> JsonPath {
    JsonPath::of_keys([<TradeFamily as Family>::PAYLOAD_KEY])
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

/// The parse stage of one file: the struct its version selects, not upgraded. `Err` is a
/// finding that rejects the file before any struct is chosen.
///
/// Two cases the generic stage does not express go to the legacy parser here: a document
/// that is JSON `null` (the parser's own "must containt" text), and a `format_version` that is
/// no string (a number or an array) whose version, as the game's JSON reader picks it from the
/// text, selects the legacy slice: the legacy parser then reads its own version only from a
/// string and takes 0.0.0 without a message. A non-string that gives no version (`null`, a
/// bool, an object) is "Malformed format version" and one that selects the cereal slice is
/// read there, both by the generic stage (the document input). A pre-release of
/// 1.21.30 sorts below 1.21.30 and reaches the legacy slice through the generic stage.
fn parse_stage(bytes: &[u8], cx: &LoadContext) -> Result<Parsed<TradeFamily>, Box<Diagnostic>> {
    let prepared = Prepared::new(bytes).map_err(|_| Box::new(malformed()))?;
    let text = prepared.as_str();
    let document = JsonSlice::new(text);
    match probe_envelope(document) {
        Err(_) if is_null(text) => Ok(legacy(document, cx)),
        Ok(envelope)
            if matches!(
                envelope.format_version,
                EnvelopeVersion::Malformed | EnvelopeVersion::Array(_)
            ) && picks_legacy(text) =>
        {
            Ok(legacy(document, cx))
        }
        _ => Ok(parse_as_written::<TradeFamily>(document, cx)),
    }
}

/// Whether the version the game's JSON reader picks from `text` selects the legacy slice (below
/// 1.21.30).
fn picks_legacy(text: &str) -> bool {
    FormatVersionInput::pick(text)
        .with(|input, _| input.version.is_some_and(|v| v < CEREAL))
        .or_absent(false)
}

/// Whether the text is the JSON value `null`.
fn is_null(text: &str) -> bool {
    bedrock_addon_core::json::from_str_jsonc::<Value>(text).is_ok_and(|v| v.is_null())
}

/// The legacy parse of the whole document at 0.0.0, for a `format_version` that is no string
/// (as if absent).
fn legacy(document: JsonSlice<'_>, cx: &LoadContext) -> Parsed<TradeFamily> {
    let effective = v0::VERSION;
    let written = FormatVersionInput {
        version: Some(effective),
        beta_literal: false,
        prerelease: false,
    };
    let parse = ParseCx {
        load: cx,
        written,
        original_version: effective,
        at: JsonPath::new(),
    };
    let mut diagnostics = Diagnostics::default();
    let value = <Any as VersionChain<TradeFamily>>::parse_struct(
        document,
        StructIndex::Stable(0),
        &parse,
        &mut diagnostics,
    )
    .filter(|_| !diagnostics.has_errors());
    Parsed::from_value(
        value,
        VersionInfo::new(None, false, effective, v0::VERSION, false),
        written,
        JsonPath::new(),
        diagnostics,
    )
}

/// The rest of the load of a parsed file: the upgrade, the checks, the document.
fn finished(
    parsed: Parsed<TradeFamily>,
    origin: Origin,
    cx: &LoadContext,
) -> Result<Document<TradeTable>, Rejected> {
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(table)) => {
            Ok(Document::new(table, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

// ---- the collection ----------------------------------------------------------------------

/// `trading/**/*.json` of every pack, bottom-up.
const WALK: Walk = Walk::json_tree_in(&[DIRECTORY]);

/// The loader of `bp.trade_tables()`: `trading/**/*.json` of every behaviour pack, keyed by
/// path, the top pack's copy winning; a failing top copy leaves the path without a file
/// (vanilla looks a table up once, by path).
#[doc(hidden)]
pub const LOADER: CollectionLoader<Files<TradeTable>, FileCopy<TradeTable>> =
    load::files::<TradeTable>(WALK);

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn trade_reads_as_pinned() {
        assert_reading_table(
            "TradeFamily",
            &reading_table::<super::TradeFamily>(),
            r"
                text: Document, clamps beyond a double: false, duplicates kept in: []
                envelope: From(FormatVersion { major: 1, minor: 21, patch: 30 })
                missing version: at the floor, silently
                below floor: UseFormatVersion
                empty document: Error
                0.0.0..=0.0.0 (0..=0): json Own/Strict/Warn/lenient/Refuse, mcb Own/Strict/Warn/lenient/Refuse
                1.21.30..=1.21.30 (1..=1): json Rejected/Strict/Error/fatal/Refuse, mcb Own/Strict/Error/fatal/Refuse
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
    fn trade_reads_no_identity_from_the_payload() {
        let text = format!(
            r#"{{"format_version": "1.21.0", "{}": {{"description": {{"identifier": "demo:x"}}}}}}"#,
            <super::TradeFamily as Family>::PAYLOAD_KEY,
        );
        let parsed =
            parse_as_written::<super::TradeFamily>(JsonSlice::new(&text), &LoadContext::detached());
        assert_eq!(parsed.identity, Identity::default());
    }
}
