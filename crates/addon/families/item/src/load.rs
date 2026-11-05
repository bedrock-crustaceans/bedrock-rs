//! Loading items: the `Family` of `minecraft:item`, the format routing, the parse-time gates,
//! `Content` / `Definition` / `Placed`, and the `bp.items()` loader with its combine rule.

use bedrock_addon_core::document::{
    CombineRule, Contribution, Definition, Document, Placed, Registry, RegistryBuilder, Rejected,
    default_path,
};
use bedrock_addon_core::id::{ItemId, kind};
use bedrock_addon_core::json::NullRule;
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value, WriteStyle, check_depth, strip_bom};
use bedrock_addon_core::load::{
    Accept, CollectionLoader, DocumentSource, ListDepth, LoadContext, LoadRequirement, StackCx,
    Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide, PackType};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{
    self, BetaPermission, ComponentTable, EnvelopeCheck, ErrorMask, Family, InputKind, LoadOutcome,
    Loaded, MissingVersion, StructIndex, ValidateCx,
};
use facet_json::RawJson;
use indexmap::IndexMap;

use super::component::ItemComponent;
use super::history::Any;
use super::texts;
use super::{Beta, Item};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::Origin;

/// The item family (`minecraft:item`, `items/`).
pub struct ItemFamily;

/// 1.16.100: the first component format, and the start of vanilla's data-driven items.
const V1_16_100: FormatVersion = FormatVersion::new(1, 16, 100);

impl Family for ItemFamily {
    // What the document does not read is kept for write-back: a number no double holds is
    // the largest finite one, with a finding at it.
    const CLAMPS_BEYOND_DOUBLE: bool = true;
    const NAME: &'static str = "item";
    const PAYLOAD_KEY: &'static str = "minecraft:item";
    const DIRECTORIES: &'static [&'static str] = &["items"];
    // The game's item loader passes no minimum version: no floor.
    const FLOOR: FormatVersion = FormatVersion::new(0, 0, 0);
    type Latest = Item;
    type Beta = Beta;
    type Component = ItemComponent;
    type Any = Any;
    const COMPONENT_NOUN: &'static str = "item component";

    // The game builds an item's input from the already parsed document (or, for an MCB item,
    // without a payload check): neither runs the check that a JSON document holds exactly
    // two members.
    const ENVELOPE: EnvelopeCheck = EnvelopeCheck::Uncounted;
    // A JSON item without `format_version` gives no version, and the game drops it; an MCB
    // document always has one.
    const MISSING_VERSION: MissingVersion = MissingVersion::Reject;
    const MISSING_VERSION_TEXT: Option<VanillaText> =
        Some(VanillaText::Fixed(texts::VERSION_NOT_READABLE));
    // The modern slices read the parsed document (JSON) or the payload as is (MCB); the legacy
    // slice's chain reads through the jsoncpp reader, whose null rule `null_rule` gives per
    // slice.
    const INPUT: InputKind = InputKind::Document;

    /// The modern slices (1.26.30, 1.26.50, beta) read through a strict reader that rejects a
    /// `null` member; the legacy slice below 1.26.30 reads through the jsoncpp reader, which
    /// takes it as absent.
    fn null_rule(index: StructIndex) -> Option<NullRule> {
        Some(if is_modern(index) {
            NullRule::Rejected
        } else {
            NullRule::Absent
        })
    }

    /// The modern slices are registered with the strict error mask (an unknown member or
    /// component rejects the item); the legacy slice keeps the lenient reading.
    fn error_mask(index: StructIndex) -> ErrorMask {
        if is_modern(index) {
            ErrorMask::Strict
        } else {
            ErrorMask::Unstrict
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
        super::expressions::drop_failed_shoot_events(
            &mut legacy.expressions,
            events,
            &failed,
            &cx.at,
        );
        if let Loaded::Stable(item) = v {
            super::validate::validate(item, cx.original_version, cx.load, &cx.at, cx.diagnostics);
            // A component-based document the load leaves without a registered component has
            // no text: the game took it for component-based by a key the item no longer
            // holds (a component that was dropped, or the list `minecraft:custom_components`,
            // whose entries are keyed custom components now), and a document without such a
            // key is the legacy item format. The writer refuses the
            // item at `components`, and the load says so there (an information of the
            // crate's own; the game is silent).
            if item.legacy.format == super::ItemFormat::DataDriven
                && !super::write::is_component_based_as_written(item)
            {
                item.legacy.left_empty_by_the_load = true;
                let at = cx.at.child("components");
                let mut told = Diagnostic::info(DiagnosticCode::InvalidValue { text: None })
                    .at(at)
                    .phase(Phase::Validate);
                told.detail = Some(Box::new(
                    bedrock_addon_core::diagnostic::DiagnosticDetail::Note(
                        "no registered component is left, so the newest format reads the \
                         item as a legacy item: it is not written",
                    ),
                ));
                cx.diagnostics.push(told);
            } else if let Some(at) = super::write::newest_refusal(item) {
                // A member the item holds from an older format that the strict 1.26.50 format
                // does not bind: the newest writer refuses the item there, and the load
                // says so where it is (an information of the crate's own; the game reads the
                // older file and says nothing).
                let mut told = Diagnostic::info(DiagnosticCode::InvalidValue { text: None })
                    .at(at)
                    .phase(Phase::Validate);
                told.detail = Some(Box::new(
                    bedrock_addon_core::diagnostic::DiagnosticDetail::Note(
                        texts::NOT_IN_THE_NEWEST_FORMAT,
                    ),
                ));
                cx.diagnostics.push(told);
            }
        }
    }
}

/// 1.26.30: the first modern slice.
const FIRST_MODERN_SLICE: FormatVersion = FormatVersion::new(1, 26, 30);

/// Whether `index` names a modern slice: the 1.26.30 struct, the newest one or the beta one.
fn is_modern(index: StructIndex) -> bool {
    match index {
        StructIndex::Beta => true,
        StructIndex::Stable(i) => <Any as versioning::VersionChain<ItemFamily>>::VERSIONS
            .get(i)
            .is_some_and(|version| *version >= FIRST_MODERN_SLICE),
    }
}

/// Why a document is in the legacy item format: the version (a) or the components (b) fail
/// the game's check that a schema is component based.
pub(crate) fn is_component_based(
    payload: JsonSlice<'_>,
    v: FormatVersionInput,
) -> Result<(), &'static str> {
    let version_ok = match v.version {
        None => false,
        Some(version) => {
            (version == V1_16_100 && !v.prerelease) || (version.major == 1 && version.minor >= 17)
        }
    };
    if !version_ok {
        return Err("the format version is neither 1.16.100 nor 1.17.0 or later");
    }
    let probe = bedrock_addon_core::json::from_str_borrowed_jsonc::<Probe<'_>>(payload.text).ok();
    let registered = probe
        .and_then(|p| p.components)
        .is_some_and(|components| components.keys().any(|key| is_registered(key)));
    if registered {
        Ok(())
    } else {
        Err("`components` names no registered component and no `tag:` key")
    }
}

/// What the routing reads of a payload: its component names.
#[derive(facet::Facet)]
struct Probe<'a> {
    #[facet(default)]
    components: Option<IndexMap<String, RawJson<'a>>>,
}

/// Whether vanilla's component factory registers `key`, or it is a `tag:` key.
pub(crate) fn is_registered(key: &str) -> bool {
    super::legacy::is_tag_key(key)
        || ItemComponent::TABLE.get(key).is_some()
        || HISTORY_ROWS.iter().any(|row| row.name == key)
}

/// A gate row of a component no newer format binds (or binds under another name).
#[derive(Clone, Copy)]
pub(crate) struct GateRow {
    /// The JSON key.
    pub(crate) name: &'static str,
    /// The format version from which the component is released.
    pub(crate) released: Option<FormatVersion>,
    /// Unlocks it before `released`.
    pub(crate) requires: &'static [LoadRequirement],
    /// The version it was deprecated in.
    pub(crate) deprecated_since: Option<FormatVersion>,
    /// Its own deprecation text.
    pub(crate) deprecated_message: Option<&'static str>,
}

const LCF: LoadRequirement = LoadRequirement::LegacyCreatorFeatures;
const BETA_APIS: LoadRequirement =
    LoadRequirement::Experiment(bedrock_addon_core::load::Experiment::BetaApis);

/// The registered components the newest format does not bind.
pub(crate) const HISTORY_ROWS: &[GateRow] = &[
    GateRow {
        name: "minecraft:armor",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 20)),
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:chargeable",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 50)),
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:custom_components",
        released: Some(FormatVersion::new(1, 21, 10)),
        requires: &[BETA_APIS],
        deprecated_since: Some(FormatVersion::new(1, 21, 90)),
        deprecated_message: Some(texts::CUSTOM_COMPONENTS_DEPRECATED),
    },
    GateRow {
        name: "minecraft:on_use",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 40)),
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:on_use_on",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 40)),
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:render_offsets",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 10)),
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:use_duration",
        released: Some(FormatVersion::new(1, 20, 20)),
        requires: &[],
        deprecated_since: None,
        deprecated_message: None,
    },
    GateRow {
        name: "minecraft:weapon",
        released: None,
        requires: &[LCF],
        deprecated_since: Some(FormatVersion::new(1, 20, 40)),
        deprecated_message: None,
    },
];

/// The gate row of `name`: the newest format's table, else [`HISTORY_ROWS`].
fn row(name: &str) -> Option<GateRow> {
    if let Some(meta) = ItemComponent::TABLE.get(name) {
        return Some(GateRow {
            name: meta.name,
            released: meta.released,
            requires: meta.requires,
            deprecated_since: meta.deprecated_since,
            deprecated_message: meta.deprecated_message,
        });
    }
    HISTORY_ROWS.iter().find(|row| row.name == name).copied()
}

/// The smallest `released` of any component: below it, vanilla's item-level gate rejects every
/// component-based item of a pack without the legacy creator features (computed from the component
/// table, as the game does).
pub(crate) fn release_threshold(
    rows: impl Iterator<Item = Option<FormatVersion>>,
) -> Option<FormatVersion> {
    rows.flatten().min()
}

/// The threshold of the item tables.
fn item_threshold() -> Option<FormatVersion> {
    release_threshold(
        ItemComponent::TABLE
            .0
            .iter()
            .map(|meta| meta.released)
            .chain(HISTORY_ROWS.iter().map(|row| row.released)),
    )
}

/// The parse-time gates of a component-based document: vanilla's item-level release
/// gate, then per component as written its deprecation, then its `released` / `requires`.
/// Returns the keys to drop, or `None` when the item is not loaded (an error says why).
pub(crate) fn gate(
    components: &IndexMap<String, Value>,
    identifier: &str,
    original: FormatVersion,
    cx: &LoadContext,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<Vec<String>> {
    gate_with(
        components,
        identifier,
        original,
        cx,
        at,
        d,
        item_threshold(),
        row,
    )
}

/// [`gate`] over the given threshold and rows (tests lower the table).
#[allow(clippy::too_many_arguments, reason = "the gate's inputs")]
pub(crate) fn gate_with(
    components: &IndexMap<String, Value>,
    identifier: &str,
    original: FormatVersion,
    cx: &LoadContext,
    at: &JsonPath,
    d: &mut Diagnostics,
    threshold: Option<FormatVersion>,
    row: fn(&str) -> Option<GateRow>,
) -> Option<Vec<String>> {
    let lcf = LoadRequirement::LegacyCreatorFeatures.satisfied(cx);
    // The game refuses an item below the minimum format version released for any component
    // unless the legacy creator features are on.
    if !lcf
        && let Some(threshold) = threshold
        && original < threshold
    {
        d.push(Diagnostic::error(DiagnosticCode::MissingToggles {
            text: Some(VanillaText::UseFormatVersion {
                what: "item",
                name: identifier.to_owned(),
                version: threshold,
            }),
        }));
        return None;
    }
    let mut dropped = Vec::new();
    let mut rejected = false;
    for key in components.keys() {
        let Some(row) = row(key) else {
            continue;
        };
        // vanilla's armor fixer removes it from the raw JSON before the check
        if key == "minecraft:armor" && original < FormatVersion::new(1, 20, 20) {
            continue;
        }
        let path = at.child(key);
        if let Some(since) = row.deprecated_since
            && original >= since
        {
            let text = row.deprecated_message.map_or_else(
                || VanillaText::Deprecated {
                    member: key.clone(),
                    version: original,
                },
                VanillaText::Fixed,
            );
            // The game logs "%s has been deprecated and is not available in json format %s"
            // at the content log's error level and removes the member; the item loads on: a
            // kept error. A row with its own text (`custom_components`) is a deprecated
            // member of the schema instead: that line is an error under every error mask, so
            // the item is refused (the game's own unit tests expect this text for a current
            // format version).
            let finding =
                Diagnostic::error(DiagnosticCode::DeprecatedMember { text: Some(text) }).at(path);
            if row.deprecated_message.is_some() {
                d.push(finding);
            } else {
                d.push_kept_error(finding);
            }
            dropped.push(key.clone());
            continue;
        }
        let released = row.released.is_some_and(|r| original >= r);
        if released {
            continue;
        }
        if let Some(failed) = row.requires.iter().find(|r| !r.satisfied(cx)) {
            rejected = true;
            d.push(
                Diagnostic::error(DiagnosticCode::MissingToggles {
                    text: requirement_text(*failed, key, row.released),
                })
                .at(path),
            );
        }
    }
    (!rejected).then_some(dropped)
}

/// The text of a failed requirement.
fn requirement_text(
    requirement: LoadRequirement,
    name: &str,
    released: Option<FormatVersion>,
) -> Option<VanillaText> {
    use bedrock_addon_core::load::Experiment;
    let toggle = |experiment: Experiment| experiment.save_name().unwrap_or_default();
    let what = ItemFamily::COMPONENT_NOUN;
    let experiment = match requirement {
        LoadRequirement::LegacyCreatorFeatures => {
            // The crate's own text where the component has no release version.
            return Some(released.map_or_else(
                || VanillaText::ItemLegacyOnly {
                    component: name.to_owned(),
                },
                |version| VanillaText::UseFormatVersion {
                    what,
                    name: name.to_owned(),
                    version,
                },
            ));
        }
        LoadRequirement::BetaFeatures => Experiment::BetaApis,
        LoadRequirement::Experiment(experiment) => experiment,
        _ => return None,
    };
    Some(match released {
        Some(version) => VanillaText::TurnOnToggleBefore {
            what,
            name: name.to_owned(),
            version,
            toggle: toggle(experiment),
        },
        None => VanillaText::TurnOnToggle {
            what,
            name: name.to_owned(),
            toggle: toggle(experiment),
        },
    })
}

/// What one item file loads to.
pub(crate) struct ItemLoad {
    /// The outcome of the chain.
    pub(crate) outcome: LoadOutcome<ItemFamily>,
    /// The identifier the payload names, when it names one.
    pub(crate) identifier: Option<String>,
}

/// The text of an item file, ready for the envelope: without a BOM, UTF-8 (else
/// `MalformedInput`), nested at most `MAX_JSON_DEPTH` deep (else `MalformedInput`).
/// A root `use_beta_features` is no member of the current format: like any other root member
/// beside the version and the payload it is read by nothing (the item envelope does not
/// count members).
fn prepare(bytes: &[u8]) -> Result<&str, Diagnostics> {
    let mut early = Diagnostics::default();
    let Ok(text) = std::str::from_utf8(strip_bom(bytes)) else {
        early.push(Diagnostic::error(DiagnosticCode::MalformedInput {
            message: "not UTF-8".to_owned(),
            text: None,
        }));
        return Err(early);
    };
    if check_depth(text.as_bytes(), bedrock_addon_core::json::MAX_JSON_DEPTH).is_err() {
        early.push(Diagnostic::error(DiagnosticCode::MalformedInput {
            message: "nested too deeply".to_owned(),
            text: None,
        }));
        return Err(early);
    }
    Ok(text)
}

/// Loads one item file: the resource-pack rule, then the chain.
pub(crate) fn load_item(bytes: &[u8], cx: &LoadContext) -> ItemLoad {
    // A leading byte-order mark is blanked, not cut off, so every span is an offset into the
    // file as written.
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let text = match prepare(bytes) {
        Ok(text) => text,
        Err(early) => return rejected_load(early),
    };
    let document = JsonSlice::new(text);
    let mut parsed = versioning::parse_as_written::<ItemFamily>(document, cx);
    parsed.diagnostics.extend(numeric_version(text));
    json_path_texts(&mut parsed.diagnostics, text, cx);
    let identifier = parsed.identity.identifier.clone();
    // a resource-pack item at or above 1.16.100
    if cx.pack_type == PackType::Resource
        && parsed.version.effective >= V1_16_100
        && !parsed.version.beta
    {
        let mut diagnostics = parsed.diagnostics;
        diagnostics.push(Diagnostic::error(DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::ItemInResourcePack {
                identifier: identifier.clone().unwrap_or_default(),
            }),
        }));
        return ItemLoad {
            outcome: LoadOutcome {
                value: None,
                original_version: parsed.version.effective,
                version: parsed.version,
                diagnostics,
            },
            identifier,
        };
    }
    let outcome = versioning::finish::<ItemFamily>(parsed, cx);
    ItemLoad {
        outcome,
        identifier,
    }
}

/// The texts of the JSON item path (the game's JSON upgrade step) in place of the framework's
/// envelope texts, each an error that refuses the item: a JSON syntax error gets the "invalid
/// json object" line alone; a root that is no object gets the "root value needs to be a json
/// object" line and then the invalid object line; a version not found or not readable gets
/// its own line and then the invalid object line; a version above the game's current one
/// gets the "out of handled range" line; no `minecraft:item` object gets the "should be a
/// json object" line. An MCB document skips the JSON upgrade step and keeps the framework's
/// validation texts.
fn json_path_texts(d: &mut Diagnostics, text: &str, cx: &LoadContext) {
    if cx.source == DocumentSource::Mcb {
        return;
    }
    let fixed = |text: &'static str| Some(VanillaText::Fixed(text));
    let root = bedrock_addon_core::json::from_str_jsonc::<Value>(text).ok();
    let payload_not_object = root
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|root| root.get(ItemFamily::PAYLOAD_KEY))
        .is_some_and(|payload| !payload.is_object());
    let payload_at = JsonPath::new().child(ItemFamily::PAYLOAD_KEY);
    let mut invalid_object = false;
    for finding in d.iter_mut() {
        if finding.severity < Severity::Error {
            continue;
        }
        match &mut finding.code {
            DiagnosticCode::MalformedInput {
                text: text @ Some(VanillaText::DocumentJson { .. }),
                ..
            } => {
                if root.is_some() {
                    *text = fixed(texts::ROOT_NOT_OBJECT);
                    invalid_object = true;
                } else {
                    *text = fixed(texts::INVALID_JSON_OBJECT);
                }
            }
            DiagnosticCode::MalformedFormatVersion { text } => {
                *text = fixed(texts::VERSION_NOT_READABLE);
                invalid_object = true;
            }
            DiagnosticCode::MissingFormatVersion { .. } => invalid_object = true,
            DiagnosticCode::MissingPayload { text } => *text = fixed(texts::PAYLOAD_NOT_OBJECT),
            DiagnosticCode::UnsupportedVersion {
                text: text @ Some(VanillaText::NoParserForVersion { .. }),
            } => {
                let beta = matches!(text, Some(VanillaText::NoParserForVersion { version }) if version == "beta");
                if !beta {
                    *text = fixed(texts::VERSION_OUT_OF_RANGE);
                }
            }
            _ if payload_not_object && finding.location.json_path == payload_at => {
                finding.code = DiagnosticCode::MalformedInput {
                    message: "the payload is no object".to_owned(),
                    text: fixed(texts::PAYLOAD_NOT_OBJECT),
                };
            }
            _ => {}
        }
    }
    if invalid_object {
        d.push(Diagnostic::error(DiagnosticCode::MalformedInput {
            message: "the item is not loaded".to_owned(),
            text: fixed(texts::INVALID_JSON_OBJECT),
        }));
    }
}

/// A root `format_version` written as a number: the item's version is read from its text, as
/// the game's document parse re-reads a version that is no string, and the writer writes a
/// string. Told where it is, an information.
fn numeric_version(text: &str) -> Option<Diagnostic> {
    let root = bedrock_addon_core::json::from_str_jsonc::<Value>(text).ok()?;
    let version = root.as_object()?.get("format_version")?;
    version.as_number()?;
    let at = JsonPath::new().child("format_version");
    let mut told = Diagnostic::info(DiagnosticCode::ValueRewritten { text: None }).at(at);
    told.detail = Some(Box::new(
        bedrock_addon_core::diagnostic::DiagnosticDetail::Note(texts::NUMERIC_VERSION),
    ));
    Some(told)
}

/// The identifier of the item file `bytes` of the pack of `cx`, when the file loads there:
/// what the link pass reads of a resource pack's `items/` files, which no collection holds.
pub fn loaded_identifier(bytes: &[u8], cx: &LoadContext) -> Option<String> {
    let load = load_item(bytes, cx);
    load.outcome.value.as_ref()?;
    load.identifier
}

/// A load rejected before the chain ran.
fn rejected_load(diagnostics: Diagnostics) -> ItemLoad {
    ItemLoad {
        outcome: LoadOutcome {
            value: None,
            original_version: FormatVersion::new(0, 0, 0),
            version: bedrock_addon_core::document::VersionInfo::new(
                None,
                false,
                FormatVersion::new(0, 0, 0),
                FormatVersion::new(0, 0, 0),
                false,
            ),
            diagnostics,
        },
        identifier: None,
    }
}

/// One file's contribution to the item registry.
pub(crate) fn contribution(
    pack: &PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Contribution<Item> {
    let ItemLoad {
        outcome,
        identifier,
    } = load_item(bytes, cx);
    let origin = Origin::file(pack.clone(), side, path.clone());
    let diagnostics = outcome.diagnostics.located(&origin);
    let id = identifier.map(ItemId::parse);
    match outcome.value {
        Some(Loaded::Stable(item)) => {
            let id = item.description.identifier.clone();
            Contribution::Stable(
                id,
                Document::new(item, origin, outcome.version, diagnostics),
            )
        }
        Some(Loaded::Beta(beta)) => {
            let id = beta.description.identifier.clone();
            Contribution::Beta(
                id,
                Document::new(beta, origin, outcome.version, diagnostics),
            )
        }
        None => Contribution::Rejected(id, Rejected::new(origin, diagnostics)),
    }
}

/// The parse step of the `bp.items()` loader.
fn parse_file(file: &WalkFile, cx: &LoadContext) -> ParsedItem {
    ParsedItem {
        contribution: contribution(&file.pack, file.side, &file.path, &file.bytes, cx),
        base_game: cx.is_base_game_pack,
    }
}

/// One parsed item file and whether its pack is a base-game pack.
pub struct ParsedItem {
    contribution: Contribution<Item>,
    base_game: bool,
}

impl ParsedItem {
    /// The identifier and pack rank of a loaded definition.
    fn loaded(&self) -> Option<(&ItemId, usize)> {
        loaded_of(&self.contribution).map(|(id, rank, _)| (id, rank))
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
fn combine(parsed: Vec<ParsedItem>, stack: &StackCx<'_>) -> Registry<Item> {
    let mut builder = RegistryBuilder::new(
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        stack.options,
    );
    // The rank of the highest pack that defines each item (0 is the top of the stack).
    let mut highest: IndexMap<ItemId, usize> = IndexMap::new();
    for parsed in &parsed {
        if let Some((id, rank)) = parsed.loaded() {
            let best = highest.entry(id.clone()).or_insert(rank);
            *best = (*best).min(rank);
        }
    }
    // the vanilla item set: what base-game packs define, and whether at 1.16.100 or later
    let mut vanilla: IndexMap<ItemId, bool> = IndexMap::new();
    for ParsedItem {
        contribution,
        base_game,
    } in parsed
    {
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

/// The `bp.items()` loader: every `.json` below `items/`, bottom-up.
pub const fn load_registry() -> CollectionLoader<Registry<Item>, ParsedItem> {
    CollectionLoader {
        walk: Walk {
            directories: ItemFamily::DIRECTORIES,
            depth: ListDepth::Recursive,
            accept: Accept::Fn(is_json),
            order: WalkOrder::BottomUp,
            directory_major: false,
            app_data: &[],
            app_data_place: bedrock_addon_core::load::AppDataPlace::After,
            in_pack: None,
            shadowed: None,
            read_error: None,
        },
        parse: parse_file,
        combine,
        needs: &[],
        then: None,
    }
}

impl bedrock_addon_core::__private::Sealed for Item {}
impl bedrock_addon_core::__private::Sealed for Beta {}

impl bedrock_addon_core::document::Content for Item {
    const NAME: &'static str = "item";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        match contribution(&PackRef::app_data(), PackSide::Behavior, path, bytes, cx) {
            Contribution::Stable(_, document) => Ok(document),
            Contribution::Beta(_, document) => {
                let origin = document.origin().clone();
                let mut diagnostics = document.diagnostics().clone();
                diagnostics.push(Diagnostic::error(DiagnosticCode::UnsupportedVersion {
                    text: None,
                }));
                Err(Rejected::new(origin, diagnostics))
            }
            Contribution::Rejected(_, rejected) => Err(rejected),
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        super::write::write_item(self, out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for Item {}

impl bedrock_addon_core::document::Content for Beta {
    const NAME: &'static str = "item (beta)";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        match contribution(&PackRef::app_data(), PackSide::Behavior, path, bytes, cx) {
            Contribution::Beta(_, document) => Ok(document),
            Contribution::Stable(_, document) => {
                let origin = document.origin().clone();
                let mut diagnostics = document.diagnostics().clone();
                diagnostics.push(Diagnostic::error(DiagnosticCode::UnsupportedVersion {
                    text: None,
                }));
                Err(Rejected::new(origin, diagnostics))
            }
            Contribution::Rejected(_, rejected) => Err(rejected),
        }
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        super::write::write_beta(self, out, style)
    }
}

impl bedrock_addon_core::document::JsonContent for Beta {}

impl Definition for Item {
    type Kind = kind::Item;
    type File = Item;
    type Beta = Beta;
}

impl Definition for Beta {
    type Kind = kind::Item;
    type File = Beta;
    type Beta = bedrock_addon_core::document::NoBeta<kind::Item>;
}

impl Placed for Item {
    /// `items/<namespace>/<name>.json`.
    fn default_path(&self) -> PackPath {
        default_path("items", &self.description.identifier)
    }
}

impl Item {
    /// Parses an item file without upgrading it: the struct its `format_version` selects (the
    /// legacy item format for a document that is not component based).
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the file is malformed, is not JSON, or its findings include an
    /// error.
    pub fn parse_as_written(
        path: &PackPath,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> Result<Document<Any>, Rejected> {
        // A leading byte-order mark is blanked, so spans are offsets into the file as written.
        let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
        let origin = Origin::file(PackRef::app_data(), PackSide::Behavior, path.clone());
        let text = match prepare(bytes) {
            Ok(text) => text,
            Err(early) => {
                return Err(Rejected::located(origin.clone(), early));
            }
        };
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
}

impl Any {
    /// Runs the remaining upgrade steps on a document [`Item::parse_as_written`] gave: the
    /// same result `Item::parse` gives (a beta document is rejected here; it has no stable
    /// form).
    ///
    /// # Errors
    ///
    /// Returns [`Rejected`] when the document is a beta one, or a remaining upgrade step leaves an
    /// error finding.
    pub fn upgrade(document: Document<Any>, cx: &LoadContext) -> Result<Document<Item>, Rejected> {
        let origin = document.origin().clone();
        let version = document.version().clone();
        let diagnostics = document.diagnostics().clone();
        let any = document.into_value();
        let identity = versioning::Identity {
            identifier: Some(any.identifier().to_owned()),
            runtime_identifier: None,
        };
        let written = FormatVersionInput {
            version: version.written.or(Some(version.effective)),
            beta_literal: version.beta,
            prerelease: false,
        };
        let at = JsonPath::new().child(ItemFamily::PAYLOAD_KEY);
        let parsed = versioning::Parsed::<ItemFamily>::new(
            Some(any),
            version,
            written,
            identity,
            bedrock_addon_core::vanilla::ActorType::Mob,
            false,
            at,
            diagnostics,
        );
        let outcome = versioning::finish::<ItemFamily>(parsed, cx);
        let diagnostics = outcome.diagnostics.located(&origin);
        match outcome.value {
            Some(Loaded::Stable(item)) => {
                Ok(Document::new(item, origin, outcome.version, diagnostics))
            }
            _ => Err(Rejected::new(origin, diagnostics)),
        }
    }

    /// The identifier the document names.
    pub fn identifier(&self) -> &str {
        match self {
            Any::V1_10_0(v) => &v.description.identifier,
            Any::V1_16_100(v) => &v.description.identifier,
            Any::V1_19_83(v) => &v.description.identifier,
            Any::V1_20_0(v) => &v.description.identifier,
            Any::V1_20_20(v) => &v.description.identifier,
            Any::V1_20_30(v) => &v.description.identifier,
            Any::V1_20_40(v) => &v.description.identifier,
            Any::V1_20_50(v) => &v.description.identifier,
            Any::V1_20_60(v) => &v.description.identifier,
            Any::V1_20_80(v) => &v.description.identifier,
            Any::V1_21_10(v) => &v.description.identifier,
            Any::V1_21_30(v) => &v.description.identifier,
            Any::V1_21_40(v) => &v.description.identifier,
            Any::V1_21_50(v) => &v.description.identifier,
            Any::V1_21_60(v) => &v.description.identifier,
            Any::V1_21_80(v) => &v.description.identifier,
            Any::V1_21_90(v) => &v.description.identifier,
            Any::V1_21_110(v) => &v.description.identifier,
            Any::V1_26_0(v) => &v.description.identifier,
            Any::V1_26_30(v) => v.description.identifier.as_str(),
            Any::V1_26_50(v) => v.description.identifier.as_str(),
            Any::Beta(v) => v.description.identifier.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Characterization: the walk takes `.json` without regard to case, unlike
    /// `PackPath::is_json`. Pinned, not decided: no vanilla sample says which the game does.
    #[test]
    fn the_walk_takes_json_in_any_case() {
        let walk = load_registry().walk;
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
