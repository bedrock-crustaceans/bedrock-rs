//! Items: `Item`, `component::*`, `history::*`, `Beta`, and the armour trims (`trim`).
//!
//! `minecraft:item` of a behaviour pack, in the newest format version; older documents are
//! upgraded when they load.

#![cfg_attr(docsrs, feature(doc_cfg))]

/// Armour trims (`TrimData`, `TrimPattern`, `TrimMaterial`): vanilla data compiled into the game.
pub mod trim;

/// The components of an item (`minecraft:item` → `components`).
pub mod component;
/// The older item formats, one struct per format version, and the chain between them.
pub mod history;

mod deprecated;
mod expressions;
mod legacy;
#[cfg(feature = "nbt")]
mod network;
mod read;
mod sounds;
mod texts;
mod types;
mod validate;
pub(crate) mod write;

#[cfg(test)]
mod tests;

pub use deprecated::ItemDeprecatedData;
pub use legacy::{
    ItemFormat, LegacyCamera, LegacyEffect, LegacyFood, LegacyFormatItem, LegacyItemData,
    LegacySeed,
};

#[allow(unused_imports, reason = "the link pass reads it")]
#[doc(hidden)]
pub use read::loaded_identifier;
pub use types::{Beta, Item, ItemDescription, MenuCategoryPresence};

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Phase, VanillaText};
use bedrock_addon_core::document::{
    CombineRule, Content, Contribution, Definition, Document, JsonDefinition, JsonDocument,
    JsonPlaced, Origin, Registry, RegistryBuilder, Rejected, default_path,
};
use bedrock_addon_core::id::{ItemId, kind};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle};
use bedrock_addon_core::load::{
    self, Accept, CollectionLoader, DocumentSource, LoadContext, StackCx, Walk,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, known};
use bedrock_addon_core::versioning::{
    self, BetaPermission, ComponentTable, DocumentRead, Envelope, EnvelopeCheck, Family, InputKind,
    Loaded, MissingVersion, Reading, StructIndex, ValidateCx,
};
use indexmap::IndexMap;

use component::ItemComponent;
use history::{Any, ItemFamily};
use read::{V1_16_100, is_component_based, json_path_texts, numeric_version, prepare, read_item};
/// The directory item files live in.
const DIRECTORY: &str = "items";

impl Family for ItemFamily {
    const NAME: &'static str = "item";
    const PAYLOAD_KEY: &'static str = "minecraft:item";
    // The game's item loader passes no minimum version: no floor.
    const FLOOR: FormatVersion = known::V0_0_0;
    type Latest = Item;
    type Beta = Beta;
    type Component = ItemComponent;
    type Any = Any;
    const COMPONENT_NOUN: &'static str = "item component";

    const ENVELOPE: Envelope = Envelope {
        // The modern slices read the parsed document (JSON) or the payload as is (MCB); the legacy
        // slice's chain reads through the jsoncpp reader, whose null rule `reading` gives per
        // slice.
        input: InputKind::Document,
        // What the document does not read is kept for write-back: a number no double holds is
        // the largest finite one, with a finding at it.
        clamps_beyond_double: true,
        // The game builds an item's input from the already parsed document (or, for an MCB item,
        // without a payload check): neither runs the check that a JSON document holds exactly
        // two members.
        check: EnvelopeCheck::Uncounted,
        // A JSON item without `format_version` gives no version, and the game drops it; an MCB
        // document always has one.
        missing: MissingVersion::Reject {
            text: Some(VanillaText::Fixed(texts::VERSION_NOT_READABLE)),
        },
        ..Envelope::DEFAULT
    };

    /// The modern slices (1.26.30, 1.26.50, beta) read through a strict reader registered with
    /// the strict error mask: a `null` member is rejected, and an unknown member or component
    /// rejects the item. The legacy slice below 1.26.30 reads through the jsoncpp reader, which
    /// takes a `null` as absent, and keeps the lenient reading.
    fn reading(index: StructIndex, _: DocumentSource) -> Reading {
        if is_modern(index) {
            Reading::DOCUMENT
        } else {
            Reading::NULL_ABSENT
        }
    }

    fn components() -> &'static ComponentTable<Self> {
        ItemComponent::TABLE
    }

    /// A JSON item passes the game's version check with the Beta APIs toggle alone; an MCB
    /// item skips that check and meets the validation of the beta document's parse instead,
    /// which also admits the base game.
    fn beta_permission(_: JsonSlice<'_>, cx: &LoadContext) -> BetaPermission {
        match cx.source {
            DocumentSource::Mcb => BetaPermission::BetaApisOrBaseGame,
            _ => BetaPermission::BetaApis,
        }
    }

    /// The legacy item format at any version a component-based document is not.
    fn route(payload: JsonSlice<'_>, v: FormatVersionInput, _: &LoadContext) -> Option<usize> {
        is_component_based(payload, v).is_err().then_some(0)
    }

    fn validate(v: &mut Loaded<Self>, cx: &mut ValidateCx<'_>) {
        // A rejected `shoot` launch_power / angle_offset makes its leaf event inert (a
        // refused expression, or a member of a wrong type).
        #[cfg(feature = "molang")]
        let failed: Vec<JsonPath> = cx
            .molang_failures
            .iter()
            .filter(|f| {
                matches!(
                    f.outcome,
                    bedrock_addon_core::molang::MolangOutcome::Failed(_)
                )
            })
            .map(|f| f.path.clone())
            .collect();
        #[cfg(not(feature = "molang"))]
        let failed: Vec<JsonPath> = Vec::new();
        let (legacy, events) = match v {
            Loaded::Stable(item) => (&mut item.legacy, &mut item.events),
            Loaded::Beta(item) => (&mut item.legacy, &mut item.events),
        };
        crate::expressions::drop_failed_shoot_events(
            &mut legacy.expressions,
            events,
            &failed,
            &cx.at,
        );
        if let Loaded::Stable(item) = v {
            crate::validate::validate(item, cx.original_version, cx.load, &cx.at, cx.diagnostics);
            // A component-based document the load leaves without a registered component has
            // no text: the game took it for component-based by a key the item no longer
            // holds (a component that was dropped, or the list `minecraft:custom_components`,
            // whose entries are keyed custom components now), and a document without such a
            // key is the legacy item format. The writer refuses the
            // item at `components`, and the load says so there (an information of the
            // crate's own; the game is silent).
            if item.legacy.format == crate::ItemFormat::DataDriven
                && !crate::write::is_component_based_as_written(item)
            {
                item.legacy.left_empty_by_the_load = true;
                let at = cx.at.child("components");
                cx.diagnostics.push(
                    Diagnostic::info(DiagnosticCode::InvalidValue { text: None })
                        .at(at)
                        .phase(Phase::Validate)
                        .detail(bedrock_addon_core::diagnostic::DiagnosticDetail::Note(
                            "no registered component is left, so the newest format reads the \
                             item as a legacy item: it is not written",
                        )),
                );
            } else if let Some(at) = crate::write::newest_refusal(item, cx.diagnostics.unknown()) {
                // A member the item holds from an older format that the strict 1.26.50 format
                // does not bind: the newest writer refuses the item there, and the load
                // says so where it is (an information of the crate's own; the game reads the
                // older file and says nothing).
                cx.diagnostics.push(
                    Diagnostic::info(DiagnosticCode::InvalidValue { text: None })
                        .at(at)
                        .phase(Phase::Validate)
                        .detail(bedrock_addon_core::diagnostic::DiagnosticDetail::Note(
                            texts::NOT_IN_THE_NEWEST_FORMAT,
                        )),
                );
            }
        }
    }
}

/// 1.26.30: the first modern slice.
const FIRST_MODERN_SLICE: FormatVersion = known::V1_26_30;

/// Whether `index` names a modern slice: the 1.26.30 struct, the newest one or the beta one.
fn is_modern(index: StructIndex) -> bool {
    match index {
        StructIndex::Beta => true,
        StructIndex::Stable(i) => <Any as versioning::VersionChain<ItemFamily>>::VERSIONS
            .get(i)
            .is_some_and(|version| *version >= FIRST_MODERN_SLICE),
    }
}

/// Whether the pack of a definition's `origin` is a base-game pack: its stack entry's context
/// (the one its files were read with); the app-data layer is the game's own data, read as a
/// base-game pack.
fn is_base_game(origin: &Origin, stack: &StackCx<'_>) -> bool {
    match origin.pack.index() {
        None => true,
        Some(index) => stack
            .packs
            .get(index)
            .is_some_and(|pack| pack.context.is_base_game_pack),
    }
}

/// The identifier, pack rank (0 is the top; app data below every pack) and origin of a loaded
/// definition; `None` for a rejected file.
fn loaded_of(contribution: &Contribution<Item>) -> Option<(&ItemId, usize, &Origin)> {
    let (id, origin) = match contribution {
        Contribution::Stable(id, document) => (id, document.origin()),
        Contribution::Beta(id, document) => (id, document.origin()),
        Contribution::Rejected(..) => return None,
    };
    Some((id, origin.pack.index().unwrap_or(usize::MAX), origin))
}

/// Combines the parsed files in walk order (bottom-up): top pack wins, and vanilla's
/// override gates for `minecraft:` items of other packs.
fn combine(parsed: Vec<Contribution<Item>>, stack: &StackCx<'_>) -> Registry<Item> {
    let mut builder = RegistryBuilder::new(
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        stack.options,
    );
    // The rank of the highest pack that defines each item (0 is the top of the stack).
    let mut highest: IndexMap<ItemId, usize> = IndexMap::new();
    for contribution in &parsed {
        if let Some((id, rank, _)) = loaded_of(contribution) {
            let best = highest.entry(id.clone()).or_insert(rank);
            *best = (*best).min(rank);
        }
    }
    // the vanilla item set: what base-game packs define, and whether at 1.16.100 or later
    let mut vanilla: IndexMap<ItemId, bool> = IndexMap::new();
    for contribution in parsed {
        let base_game =
            loaded_of(&contribution).is_some_and(|(_, _, origin)| is_base_game(origin, stack));
        // A non-base-game pack's item that a higher pack already defined is skipped by the
        // game with a warning-level line (a base-game pack is let through, as inferred from
        // the pack's base-game flag); the combine keeps the higher one.
        if !base_game
            && let Some((id, rank, origin)) = loaded_of(&contribution)
            && highest.get(id).is_some_and(|top| *top < rank)
        {
            let path = JsonPath::new().with(&["minecraft:item", "description", "identifier"]);
            builder.push_diagnostic(
                Diagnostic::warn(DiagnosticCode::ShadowedDefinition {
                    text: Some(VanillaText::Formatted {
                        template: texts::OVERRIDDEN_BY_A_HIGHER_PACK,
                        args: vec![id.as_str().to_owned()],
                    }),
                })
                .at(path)
                .phase(Phase::Validate)
                .in_file(origin),
            );
        }
        let contribution = match contribution {
            Contribution::Stable(id, document) => {
                if base_game {
                    vanilla.insert(id.clone(), data_driven_version(&document));
                    Contribution::Stable(id, document)
                } else {
                    override_gate(id, document, &vanilla)
                }
            }
            other => other,
        };
        builder.push(contribution);
    }
    builder.finish()
}

/// Vanilla's override gates for an item of a non-base-game pack in the `minecraft` namespace.
fn override_gate(
    id: ItemId,
    document: Document<Item>,
    vanilla: &IndexMap<ItemId, bool>,
) -> Contribution<Item> {
    if id.namespace() != Some("minecraft") {
        return Contribution::Stable(id, document);
    }
    let modern = data_driven_version(&document);
    let written = document
        .version()
        .written
        .map_or_else(String::new, |v| v.to_string());
    let identifier = id.as_str().to_owned();
    // A data-driven item without a vanilla entry, or whose entry is legacy, is refused with
    // the matching text; the game writes no text for a legacy item over a data-driven
    // vanilla entry, so none is given for it.
    let text = match (vanilla.get(&id).copied(), modern) {
        (Some(true), true) | (Some(false), false) => {
            return Contribution::Stable(id, document);
        }
        (Some(true), false) => VanillaText::ItemVanillaIsComponentBased {
            identifier,
            version: written,
        },
        (Some(false) | None, true) => VanillaText::ItemVanillaIsLegacy {
            identifier,
            version: written,
        },
        (None, false) => VanillaText::ItemNonVanillaNamespace { identifier },
    };
    let origin = document.origin().clone();
    let mut diagnostics = document.diagnostics().clone();
    let path = JsonPath::new().with(&["minecraft:item", "description", "identifier"]);
    diagnostics.push(
        Diagnostic::error(DiagnosticCode::NamespaceRule { text: Some(text) })
            .at(path)
            .phase(Phase::Validate)
            .in_file(&origin),
    );
    Contribution::Rejected(Some(id), Rejected::new(origin, diagnostics))
}

/// Whether a document's version is 1.16.100 or later: the override gates compare the version,
/// not the format the routing chose.
fn data_driven_version(document: &Document<Item>) -> bool {
    document.version().effective >= V1_16_100
}

/// Accepts `.json` files.
fn is_json(path: &PackPath) -> bool {
    path.as_str().to_ascii_lowercase().ends_with(".json")
}

/// The `bp.items()` loader: every `.json` below `items/`, bottom-up, read by the registry's
/// parse; the combine is the family's own (the override gates and the higher-pack line, findings
/// `RegistryBuilder::combine` does not push).
#[doc(hidden)]
pub const LOADER: CollectionLoader<Registry<Item>, Contribution<Item>> = CollectionLoader {
    combine,
    ..load::registry::<Item>(Walk::json_tree_in(&[DIRECTORY]).accept(Accept::Fn(is_json)))
};

impl JsonDocument for Item {
    type Family = ItemFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// `read::load_item`: the resource-pack rule, the chain and the JSON item path's texts.
    fn read(origin: Origin, bytes: &[u8], cx: &LoadContext) -> DocumentRead<ItemFamily> {
        read_item(origin, bytes, cx)
    }

    /// The struct the file's `format_version` selects (the legacy item format for a document
    /// that is not component based), with the JSON item path's texts; a file whose findings
    /// include an error is rejected.
    fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        // A leading byte-order mark is blanked, so spans are offsets into the file as written.
        let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
        let prepared = match prepare(bytes) {
            Ok(prepared) => prepared,
            Err(early) => {
                return Err(Rejected::located(origin.clone(), early));
            }
        };
        let text = prepared.as_str();
        let mut parsed = versioning::parse_as_written::<ItemFamily>(JsonSlice::new(text), cx);
        parsed.diagnostics.extend(numeric_version(text));
        json_path_texts(&mut parsed.diagnostics, text, cx);
        let diagnostics = parsed.diagnostics.located(&origin);
        match parsed.value {
            Some(any) if !diagnostics.has_errors() => {
                Ok(Document::new(any, origin, parsed.version, diagnostics))
            }
            _ => Err(Rejected::new(origin, diagnostics)),
        }
    }

    /// `write::item_root`: the newest format, the legacy item format at
    /// 1.10.0, or a 1.26.0 document, by what the item holds.
    fn write_root(&self) -> Result<Value, EncodeError> {
        crate::write::item_root(self)
    }

    fn write_root_with(
        &self,
        unknown: &bedrock_addon_core::document::Unknown,
    ) -> Result<(Value, Vec<bedrock_addon_core::diagnostic::Diagnostic>), EncodeError> {
        crate::write::item_root_with(self, unknown)
    }
}

impl JsonPlaced for Item {
    /// `items/<namespace>/<name>.json`.
    fn place(&self) -> PackPath {
        default_path("items", &self.description.identifier)
    }
}

impl JsonDefinition for Item {
    type Kind = kind::Item;

    fn identifier(&self) -> &ItemId {
        &self.description.identifier
    }
}

// The beta struct keeps its hand impls: its family's `Latest` is `Item`.
impl bedrock_addon_core::__private::Sealed for Beta {}

impl Content for Beta {
    const NAME: &'static str = "item (beta)";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
        match read_item(origin, bytes, cx) {
            DocumentRead::Beta(_, beta) => Ok(beta.keep()),
            DocumentRead::Stable(document) => {
                let origin = document.origin().clone();
                let mut diagnostics = document.diagnostics().clone();
                diagnostics.push(Diagnostic::error(DiagnosticCode::UnsupportedVersion {
                    text: None,
                }));
                Err(Rejected::new(origin, diagnostics))
            }
            DocumentRead::Rejected(_, rejected) => Err(rejected),
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        crate::write::write_beta(self, out, style)
    }

    fn write_json_with(
        &self,
        unknown: &bedrock_addon_core::document::Unknown,
        out: &mut Vec<u8>,
        style: WriteStyle,
    ) -> Result<Vec<bedrock_addon_core::diagnostic::Diagnostic>, EncodeError> {
        crate::write::write_beta_with(self, unknown, out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for Beta {}

impl Definition for Beta {
    type Kind = kind::Item;
    type File = Beta;
    type Beta = bedrock_addon_core::document::NoBeta<kind::Item>;
}

#[cfg(test)]
mod walk {
    use super::*;

    /// Characterization: the walk takes `.json` without regard to case, unlike
    /// `PackPath::is_json`. Pinned, not decided: no vanilla sample says which the game does.
    #[test]
    fn the_walk_takes_json_in_any_case() {
        let walk = LOADER.walk;
        for path in ["items/sword.json", "items/Sword.JSON", "items/a/b.Json"] {
            let path = PackPath::new(path).unwrap();
            assert!(is_json(&path), "{path}");
            assert_eq!(walk.matches(&path), Some(0), "{path}");
        }
        assert!(!PackPath::new("items/Sword.JSON").unwrap().is_json());
        assert_eq!(
            walk.matches(&PackPath::new("items/sword.jsonc").unwrap()),
            None
        );
    }
}

#[cfg(test)]
mod reading_table {
    //! The family's reading table: what the pipeline reads each of its files and structs by.

    use bedrock_addon_core::versioning::testing::{assert_reading_table, reading_table};

    #[test]
    fn item_reads_as_pinned() {
        assert_reading_table(
            "ItemFamily",
            &reading_table::<super::ItemFamily>(),
            r#"
                text: Document, clamps beyond a double: true, duplicates kept in: []
                envelope: Uncounted
                missing version: rejected: Fixed("document's version not found or not readable")
                below floor: UseFormatVersion
                empty document: Error
                1.10.0..=1.26.0 (0..=18): json Absent/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Warn/reports/lenient/Refuse
                1.26.30..=1.26.50 (19..=20): json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
                beta: json Rejected/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse, mcb Own/ConvertAndTell/ClampAndTell/Error/reports/fatal/Refuse
            "#,
        );
    }
}
