//! The crafting item catalogue (`CraftingCatalog`): `item_catalog/crafting_item_catalog.json` of a
//! behaviour pack (`minecraft:crafting_items_catalog`), which places items into the groups of the
//! four creative categories. Two structs of the same members: `v1_21_60` `[1.21.60, 1.26.50)`,
//! read under the unstrict mask, and `v1_26_50` from 1.26.50 to the engine, under the strict
//! one, joined by a plain copy; the files of a stack merge bottom-up from the vanilla
//! catalogue.

#![cfg_attr(docsrs, feature(doc_cfg))]

/// The catalogue as written at each format: the two structs of the chain and the document a
/// file parses to before its step ([`CatalogHistory`](history::CatalogHistory)).
pub mod history;
mod link;
mod merge;
mod read;
mod stack;
mod write;

#[cfg(test)]
mod tests;

use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_shared_shapes::CreativeCategory;

// The checkers the link pass calls (`pack::link::content`).
#[allow(unused_imports, reason = "the link pass reads them")]
#[doc(hidden)]
pub use link::{Definitions, link_group_names, link_placements};
#[doc(hidden)]
pub use stack::LOADER;

/// The first struct version (`v1_21_60::CraftingCatalogDocument::VERSION`), which is also the
/// oldest `format_version` vanilla reads.
pub(crate) const V1_21_60: FormatVersion = FormatVersion::new(1, 21, 60);

// The newest struct version (`v1_26_50::CraftingCatalogDocument::VERSION`), the
// `format_version` of vanilla's own 1.26.50 catalogue.
use bedrock_addon_core::json::Prepared;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
pub(crate) use bedrock_addon_core::version::known::V1_26_50;

use crate::history::CatalogHistory;
use crate::read::parse_catalog;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::{Document, JsonDocument, JsonPlaced, Origin, Rejected};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::DocumentSource;
use bedrock_addon_core::version::{ENGINE_VERSION, FormatVersionInput};
use bedrock_addon_core::versioning::{
    DocumentRead, Envelope, Family, InputKind, Loaded, MissingVersion, NoComponents, ParseCx,
    Parsed, Reading, Selection, StructIndex, UnknownMask, VersionStruct, Versioned, finish,
    parse_as_written, select,
};
use std::fmt;

/// The path of the one file a behaviour pack's catalogue is read from.
pub(crate) const PATH: &str = "item_catalog/crafting_item_catalog.json";

/// A crafting item catalogue (`minecraft:crafting_items_catalog`): the groups a pack adds to
/// the four creative categories, and the items in each group.
///
/// JSON: `{"format_version": "1.21.60", "minecraft:crafting_items_catalog": {"categories":
/// [{"category_name": "construction", "groups": [{"group_identifier": {"name": "ns:group",
/// "icon": "ns:item"}, "items": ["ns:item", {"name": "ns:other:1"}]}]}]}}`. Every
/// `format_version` from 1.21.60 to the engine version reads the same members.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "crafting_item_catalog")]
#[doc(alias = "minecraft:crafting_items_catalog")]
pub struct CraftingCatalog {
    /// The categories groups are added to (JSON `categories`, required). A file may list a
    /// category several times; every entry adds to the same category.
    pub categories: Vec<Category>,
}

/// The groups one entry of `categories` adds to a creative category.
#[derive(Clone, Debug, PartialEq)]
pub struct Category {
    /// The category the groups go to (JSON `category_name`, required).
    pub category_name: CategoryName,
    /// The groups added to it, in order (JSON `groups`, required).
    pub groups: Vec<Group>,
}

/// One of the four creative categories a catalogue can add to; a pack cannot add, rename or
/// reorder them. The JSON names are lower-case and compared with case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum CategoryName {
    /// `"construction"`.
    Construction = 0,
    /// `"nature"`.
    Nature = 1,
    /// `"equipment"`.
    Equipment = 2,
    /// `"items"`.
    Items = 3,
}

impl CategoryName {
    /// The four categories, in vanilla's order.
    pub const ALL: [CategoryName; 4] = [
        Self::Construction,
        Self::Nature,
        Self::Equipment,
        Self::Items,
    ];

    /// The JSON name (`"construction"`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Construction => "construction",
            Self::Nature => "nature",
            Self::Equipment => "equipment",
            Self::Items => "items",
        }
    }

    /// The category a JSON name selects; `None` for any other string (`"Construction"`
    /// included).
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.name() == name)
    }

    /// The creative-menu category of an item definition's `menu_category` this category is.
    ///
    /// The game numbers the categories from one (Construction 1 up to Items 4), so its value is
    /// the discriminant here plus one.
    pub fn creative_category(self) -> CreativeCategory {
        match self {
            Self::Construction => CreativeCategory::Construction,
            Self::Nature => CreativeCategory::Nature,
            Self::Equipment => CreativeCategory::Equipment,
            Self::Items => CreativeCategory::Items,
        }
    }
}

/// A group of items in a category: collapsible under a name and an icon when it has a
/// `group_identifier`, else a run of loose items.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group {
    /// The group's name and icon (JSON `group_identifier`); without it the items are added
    /// as loose items.
    pub group_identifier: Option<GroupIdentifier>,
    /// The items of the group, in order (JSON `items`, required).
    pub items: Vec<CatalogItem>,
}

/// The name and icon of a collapsible group.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupIdentifier {
    /// The group's name (JSON `name`, required): `namespace:name`, which is also the
    /// localisation key the creative menu shows it by. A group of the same name in the same
    /// category, in this or a lower pack, is the same group; names compare byte for byte.
    pub name: String,
    /// The item or block that represents the group (JSON `icon`). A group keeps the icon of
    /// its first definition.
    pub icon: Option<CatalogItem>,
}

/// One item or block of the catalogue.
///
/// JSON: `"ns:item"`, `"ns:item:1"` (with an aux value) or `{"name": "ns:item"}`. Both forms
/// read to the same value; it is written as the string.
/// Two items are equal when their names are written the same, the aux value included.
#[derive(Clone, Debug)]
pub struct CatalogItem {
    /// The item (JSON `name`, or the string itself): `namespace:name`, optionally followed
    /// by `:aux`. The namespace is required.
    pub name: ItemId,
}

/// By the name as written: `ItemId` alone compares without the aux value.
impl PartialEq for CatalogItem {
    fn eq(&self, other: &Self) -> bool {
        self.name.as_str() == other.name.as_str()
    }
}

/// What an entry of the merged catalogue is keyed by, for
/// [`Merged::origin`](bedrock_addon_core::document::Merged::origin): the pack that first defined a named
/// group, or the pack whose file placed an item.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CatalogKey {
    /// A named group of a category.
    Group {
        /// The category.
        category: CategoryName,
        /// The group's name.
        name: String,
    },
    /// A placed item: its identifier and aux value, which is what makes a second placement
    /// a duplicate.
    Item {
        /// The identifier; an aux value in its text does not count.
        id: ItemId,
        /// The aux value (`0` when the item names none).
        aux: i16,
    },
}

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
    const NAME: &'static str = "crafting item catalog";
    const PAYLOAD_KEY: &'static str = "minecraft:crafting_items_catalog";
    // The game registers no slice below the first struct's version.
    const FLOOR: FormatVersion = V1_21_60;
    type Latest = CraftingCatalog;
    type Beta = std::convert::Infallible;
    type Component = NoComponents;
    type History = CatalogHistory;
    const ENVELOPE: Envelope = Envelope {
        // The game reads the file as a whole document: the version is the one its JSON reader picks
        // from the text, the size limits apply, and a `null` where a value is read counts as the
        // wrong JSON type (the hand reader below refuses it).
        input: InputKind::Document,
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with a warning at it.
        clamps_beyond_double: true,
        // Vanilla's loader checks the version itself, before the versioned parse.
        missing: MissingVersion::Reject {
            text: Some(VanillaText::Fixed(VERSION_TEXT)),
        },
        ..Envelope::DEFAULT
    };

    /// An unknown member is a warning below 1.26.50, where the game reads the file leniently,
    /// and an error that skips the file from 1.26.50 on, where it reads it strictly.
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        match index {
            StructIndex::Stable(0) => Reading::DOCUMENT.with_unknown(UnknownMask::Warn),
            _ => Reading::DOCUMENT,
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

impl JsonDocument for CraftingCatalog {
    type Family = CatalogFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// Loads the catalogue file: the chain, with vanilla's envelope checks
    /// (`vanilla_checks`) on a document the parse refused.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<CatalogFamily> {
        let Ok(prepared) = Prepared::new(bytes) else {
            return DocumentRead::Rejected(None, unreadable(origin));
        };
        let text = prepared.as_str();
        let document = JsonSlice::new(text);
        let mut parsed = parse_as_written::<CatalogFamily>(document, cx);
        if parsed.value.is_none() {
            vanilla_checks(&mut parsed.diagnostics, document);
        }
        let identifier = parsed.identity.identifier.clone();
        let outcome = finish(parsed, cx);
        let mut diagnostics = outcome.diagnostics;
        diagnostics.locate(&origin);
        match outcome.value {
            Some(Loaded::Stable(catalog)) => {
                DocumentRead::Stable(Document::new(catalog, origin, outcome.version, diagnostics))
            }
            _ => DocumentRead::Rejected(identifier, Rejected::new(origin, diagnostics)),
        }
    }

    /// Parses the catalogue file without upgrading it: the struct its `format_version` selects,
    /// with vanilla's envelope checks on a document it refused; [`CatalogHistory::upgrade`] runs the
    /// rest of the load. Nothing is closed (the family has no close).
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`](bedrock_addon_core::document::Rejected) with the findings the game's content
    /// log would print when the bytes cannot be read as this file at all: not JSON, a
    /// `format_version` with no struct here, or a document that does not fit the struct it selects.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<CatalogHistory>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
        let Ok(prepared) = Prepared::new(bytes) else {
            return Err(unreadable(origin));
        };
        let text = prepared.as_str();
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

impl JsonPlaced for CraftingCatalog {
    /// `item_catalog/crafting_item_catalog.json`: the one path vanilla reads.
    fn place(&self) -> PackPath {
        PackPath::from_static(PATH)
    }
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
    let written = FormatVersionInput::pick(document.text).version().copied();
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

/// `CatalogHistory::upgrade`: rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<CatalogHistory>,
    cx: &LoadContext,
) -> Result<Document<CraftingCatalog>, Rejected> {
    let (value, origin, version, diagnostics) = document.into_parsed_parts();
    let mut at = JsonPath::new();
    at.push(CatalogFamily::PAYLOAD_KEY);
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed =
        Parsed::<CatalogFamily>::from_value(Some(value), version, written, at, diagnostics);
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

impl CraftingCatalog {
    /// A catalogue without categories.
    pub fn new() -> Self {
        Self::default()
    }

    /// Every group this catalogue adds to `category`, in order, over all of its entries for
    /// that category.
    pub fn groups(&self, category: CategoryName) -> impl Iterator<Item = &Group> + '_ {
        self.categories
            .iter()
            .filter(move |entry| entry.category_name == category)
            .flat_map(|entry| &entry.groups)
    }

    /// The first group named `name` this catalogue adds to `category`; names compare byte for
    /// byte, as the merge compares them.
    pub fn group(&self, category: CategoryName, name: &str) -> Option<&Group> {
        self.groups(category)
            .find(|group| group.name() == Some(name))
    }
}

impl Category {
    /// An entry adding no group yet to `category_name`.
    pub fn new(category_name: CategoryName) -> Self {
        Self {
            category_name,
            groups: Vec::new(),
        }
    }
}

impl Group {
    /// A collapsible group named `name` (`namespace:name`), without an icon or items.
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            group_identifier: Some(GroupIdentifier::new(name)),
            ..Self::default()
        }
    }

    /// A group of loose items, without items yet (also [`Group::default`]).
    pub fn loose() -> Self {
        Self::default()
    }

    /// The group's name; `None` for loose items.
    pub fn name(&self) -> Option<&str> {
        self.group_identifier
            .as_ref()
            .map(|identifier| identifier.name.as_str())
    }
}

impl GroupIdentifier {
    /// The identifier of a group named `name` (`namespace:name`), without an icon.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            icon: None,
        }
    }
}

impl CatalogItem {
    /// The item `name` (`namespace:name`, or `namespace:name:aux`).
    pub fn new(name: ItemId) -> Self {
        Self { name }
    }
}

impl From<ItemId> for CatalogItem {
    fn from(name: ItemId) -> Self {
        Self::new(name)
    }
}

/// The JSON name.
impl fmt::Display for CategoryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn crafting_catalog_reads_as_pinned() {
        assert_reading_table(
            "CatalogFamily",
            &reading_table::<super::CatalogFamily>(),
            r#"
                text: Document, clamps beyond a double: true, duplicates kept in: []
                envelope: Always
                missing version: rejected: Fixed("The file cannot be processed without a valid format_version")
                below floor: UseFormatVersion
                empty document: Error
                1.21.60..=1.21.60 (0..=0): json Rejected/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/fatal/Refuse
                1.26.50..=1.26.50 (1..=1): json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
            "#,
        );
    }
}
