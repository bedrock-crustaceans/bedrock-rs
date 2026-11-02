//! Reading [`Block`] from the text of a 1.26.20 payload (the game's cereal
//! document), and the Molang walk of the document.

use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::component::{BlockComponent, DeprecatedComponents, DeprecatedDefinitionEvent};
use super::history::BlockFamily;
use super::{Block, Description, Permutation};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{ComponentMap, Extra};
use bedrock_addon_core::json::{
    JsonSlice, ReaderMode, Value, keep_unknown_members, parse_at, with_reader_mode,
};
use bedrock_addon_core::molang::{
    Admission, BLOCK_STATE_QUERIES, EmptyRule, Molang, MolangFailure, MolangFieldSpec,
    MolangFields, MolangVersion, MolangVersionSource, MolangVisitor, OpsRule, ReaderKind,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{ConvertMode, ParseCx, parse_component_map, parse_payload};

/// The first version vanilla reads with the cereal document.
pub(crate) const CEREAL: FormatVersion = FormatVersion::new(1, 26, 20);
/// The newest document's version.
pub(crate) const NEWEST: FormatVersion = FormatVersion::new(1, 26, 60);
/// The version from which a legacy permutation condition that fails is an error.
pub(crate) const CONDITION_STRICT: FormatVersion = FormatVersion::new(1, 19, 70);

/// What a payload is read into: components as raw slices, parsed one at a time.
#[derive(Facet)]
struct Shell<'a> {
    description: Description,
    #[facet(default)]
    components: IndexMap<String, RawJson<'a>>,
    #[facet(default)]
    permutations: Vec<PermutationShell<'a>>,
    #[facet(default)]
    deprecated: DeprecatedComponents,
    #[facet(default)]
    deprecated_event_handlers: IndexMap<String, RawJson<'a>>,
    /// Deprecated by name in the cereal document: the game skips the value unread, so any
    /// value is held (an `Option<Value>`, which the strict reader does not judge: `"events": null` is
    /// the deprecation alone).
    #[facet(default)]
    events: Option<Value>,
    #[facet(default)]
    ordered_v1_custom_components: Vec<String>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// What a permutation is read into.
#[derive(Facet)]
struct PermutationShell<'a> {
    condition: Molang,
    #[facet(default)]
    components: IndexMap<String, RawJson<'a>>,
    #[facet(default)]
    deprecated: DeprecatedComponents,
    #[facet(default)]
    ordered_v1_custom_components: Vec<String>,
    #[facet(skip, rename = "\u{0}extra")]
    extra: Extra,
}

/// The keys the 1.26.20 document rejects in base `components`: the game's document binds them
/// as keys to refuse, so a block that has one fails to load.
pub(crate) const REJECTED_IN_BASE: [&str; 11] = [
    "minecraft:breathability",
    "minecraft:unit_cube",
    "minecraft:on_fall_on",
    "minecraft:on_interact",
    "minecraft:on_placed",
    "minecraft:on_player_destroyed",
    "minecraft:on_player_placing",
    "minecraft:on_step_off",
    "minecraft:on_step_on",
    "minecraft:queued_ticking",
    "minecraft:random_ticking",
];

/// The keys it rejects in a permutation's `components` beyond the base list,
/// with `minecraft:block_entity` from 1.26.30.
pub(crate) const REJECTED_IN_PERMUTATIONS: [&str; 6] = [
    "minecraft:flower_pottable",
    "minecraft:redstone_consumer",
    "minecraft:support",
    "minecraft:embedded_visual",
    "minecraft:item_visual",
    "minecraft:block_entity",
];

/// A key of another namespace than `minecraft` is a custom component.
pub(crate) fn is_custom(key: &str) -> bool {
    key.split_once(':')
        .is_some_and(|(namespace, name)| namespace != "minecraft" && !name.is_empty())
}

/// Reads one payload of the 1.26.20 shape. `cx.original_version` is the document's version (for
/// the 1.26.20 rules) or, when the 1.26.20 step converts a legacy document, the legacy one.
pub(crate) fn parse_block(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<Block> {
    parse_document(payload, cx, d).map(|(block, _)| block)
}

/// [`parse_block`], and the sections (`None` the base, `Some(i)` permutation `i`) whose
/// `minecraft:collision_box` is one box written as an object (the 1.26.20 alternative, which
/// the 1.26.20 struct writes back as it).
pub(crate) fn parse_document(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<(Block, Vec<Option<usize>>)> {
    let start = d.len();
    let shell: Shell<'_> = parse_payload(payload, cx, d)?;
    let cereal = cx.original_version >= CEREAL;
    if cereal {
        unknown_traits(d, start, &cx.at);
    }
    // `events` is bound and deprecated by name in the cereal document: written at all (`null`
    // included), it is the deprecation alone, which the game checks before it reads the value.
    let root = if cereal {
        bedrock_addon_core::json::from_str_jsonc::<Value>(payload.text).ok()
    } else {
        None
    };
    let events_written = root
        .as_ref()
        .and_then(Value::as_object)
        .is_some_and(|root| root.contains_key("events"));
    if let Some(root) = &root {
        reject_crate_members(root, &cx.at, d);
    }
    let mut one_box = Vec::new();
    let mut components = section(
        &shell.components,
        payload,
        &child(&cx.at, &["components"]),
        cereal.then_some(&[][..]),
        cx,
        d,
    );
    if super::component::read_collision_box(
        &mut components,
        shell.components.get(COLLISION_BOX),
        &child(&cx.at, &["components", COLLISION_BOX]),
        cx.original_version,
        d,
    ) {
        one_box.push(None);
    }
    newest_members(
        &mut components,
        &child(&cx.at, &["components"]),
        cx.original_version,
        d,
    );
    let mut permutations = Vec::with_capacity(shell.permutations.len());
    for (index, p) in shell.permutations.iter().enumerate() {
        let mut at = child(&cx.at, &["permutations"]);
        at.push_index(index);
        at.push("components");
        let mut components = section(
            &p.components,
            payload,
            &at,
            cereal.then_some(&REJECTED_IN_PERMUTATIONS[..]),
            cx,
            d,
        );
        if super::component::read_collision_box(
            &mut components,
            p.components.get(COLLISION_BOX),
            &child(&at, &[COLLISION_BOX]),
            cx.original_version,
            d,
        ) {
            one_box.push(Some(index));
        }
        permutations.push(Permutation {
            condition: p.condition.clone(),
            components,
            deprecated: p.deprecated.clone(),
            ordered_v1_custom_components: p.ordered_v1_custom_components.clone(),
            extra: p.extra.clone(),
        });
    }
    let mut deprecated_event_handlers = IndexMap::new();
    for (name, raw) in &shell.deprecated_event_handlers {
        let at = child(&cx.at, &["deprecated_event_handlers", name]);
        if let Some(event) = parse_entry::<DeprecatedDefinitionEvent>(payload, raw, &at, d) {
            deprecated_event_handlers.insert(name.clone(), event);
        }
    }
    tell_events(events_written, shell.events.as_ref(), cx, d);
    let block = Block {
        description: shell.description,
        components,
        deprecated: shell.deprecated,
        permutations,
        deprecated_event_handlers,
        ordered_v1_custom_components: shell.ordered_v1_custom_components,
        source_format_version: cx.original_version,
        extra: shell.extra,
        refused_at: Vec::new(),
    };
    Some((block, one_box))
}

/// `minecraft:collision_box`.
const COLLISION_BOX: &str = "minecraft:collision_box";

/// The members only the 1.26.60 document binds, in a 1.26.20 one: `block_entity.entity_storage`
/// is an unknown member there (`UnrecognizedField`, an error in the strict slice; held with
/// the component's unknown members). `block_entity` is a base component.
fn newest_members(
    map: &mut ComponentMap<BlockComponent>,
    at: &JsonPath,
    version: FormatVersion,
    d: &mut Diagnostics,
) {
    if !(CEREAL..NEWEST).contains(&version) {
        return;
    }
    let Some(entity) = map.get_mut::<super::component::BlockEntity>() else {
        return;
    };
    if let Some(value) = entity.entity_storage.take() {
        // The marker `unknown_members_are_errors` turns into the error that refuses the block
        // (the strict slice rejects an unknown member): only reachable in
        // [1.26.20, 1.26.60), never a warning the load keeps.
        d.push(finding(
            DiagnosticCode::UnrecognizedField { text: None },
            Severity::Warning,
            child(at, &["minecraft:block_entity", "entity_storage"]),
        ));
        entity
            .extra
            .insert("entity_storage".to_owned(), Value::from(value.0));
    }
}

/// `events`: bound and deprecated by name in the cereal document, an error that
/// rejects the block; elsewhere a deprecated member, told.
fn tell_events(written: bool, events: Option<&Value>, cx: &ParseCx<'_>, d: &mut Diagnostics) {
    if written {
        d.push(Diagnostic::deprecated_member(
            "events",
            child(&cx.at, &["events"]),
        ));
    } else if events
        .and_then(Value::as_object)
        .is_some_and(|events| !events.is_empty())
    {
        // The legacy event reader takes them with no deprecation line (the deprecation is the
        // cereal document's), so the crate reports them as an information.
        d.push(finding(
            DiagnosticCode::DeprecatedMember {
                text: Some(VanillaText::Deprecated {
                    member: "events".to_owned(),
                    version: cx.original_version,
                }),
            },
            Severity::Info,
            child(&cx.at, &["events"]),
        ));
    }
}

/// The crate's own members a cereal document has: `deprecated` (of the block or of a
/// permutation) and `deprecated_event_handlers` are no members of the game's document
/// (`UnrecognizedField`, which the strict slice makes an error);
/// `ordered_v1_custom_components` (of either) is bound and deprecated by name.
/// Each rejects the block; what the shell read of them is not used.
fn reject_crate_members(root: &Value, at: &JsonPath, d: &mut Diagnostics) {
    let mut sections = vec![(root, at.clone())];
    for (index, permutation) in root
        .as_object()
        .and_then(|root| root.get("permutations"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let mut path = child(at, &["permutations"]);
        path.push_index(index);
        sections.push((permutation, path));
    }
    for (index, (object, path)) in sections.into_iter().enumerate() {
        let Some(object) = object.as_object() else {
            continue;
        };
        let unknown: &[&str] = if index == 0 {
            &["deprecated", "deprecated_event_handlers"]
        } else {
            &["deprecated"]
        };
        for key in unknown {
            if object.contains_key(key) {
                // The marker `unknown_members_are_errors` turns into the error that refuses
                // the block (cereal documents alone reach here).
                d.push(finding(
                    DiagnosticCode::UnrecognizedField { text: None },
                    Severity::Warning,
                    child(&path, &[key]),
                ));
            }
        }
        if object.contains_key("ordered_v1_custom_components") {
            d.push(Diagnostic::deprecated_member(
                "ordered_v1_custom_components",
                child(&path, &["ordered_v1_custom_components"]),
            ));
        }
    }
}

/// A trait name the game does not have is told by its reader with its own text, `Failed to
/// find BlockTrait with identifier: '{}'`, an error that rejects the block:
/// the walk's `UnrecognizedField` at `description/traits/<name>` of a cereal document is that
/// finding, told once (the rule of `validate` never runs on a block the strict slice rejects).
fn unknown_traits(d: &mut Diagnostics, start: usize, at: &JsonPath) {
    let traits = child(at, &["description", "traits"]);
    for finding in d.iter_mut().skip(start) {
        if !matches!(finding.code, DiagnosticCode::UnrecognizedField { .. }) {
            continue;
        }
        let Some(name) = finding
            .location
            .json_path
            .strip_prefix(&traits)
            .filter(|rest| rest.len() == 1)
            .and_then(|rest| rest.last_key().map(str::to_owned))
        else {
            continue;
        };
        finding.code = DiagnosticCode::InvalidEnumValue {
            text: Some(VanillaText::Formatted {
                template: "Failed to find BlockTrait with identifier: '{}'",
                args: vec![name],
            }),
        };
        finding.severity = Severity::Error;
    }
}

/// One `components` object: the rejected keys refused ("key {} is not allowed", an error that
/// fails the block), the rest parsed one at a time.
fn section(
    raw: &IndexMap<String, RawJson<'_>>,
    payload: JsonSlice<'_>,
    at: &JsonPath,
    rejected: Option<&[&str]>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> ComponentMap<BlockComponent> {
    let mut kept = IndexMap::new();
    let mut unread = Vec::new();
    for (key, value) in raw {
        let refused = rejected.is_some_and(|extra| {
            REJECTED_IN_BASE.contains(&key.as_str()) || extra.contains(&key.as_str())
        });
        if refused {
            d.push(finding(
                bedrock_addon_core::diagnostic::cereal::key_not_allowed(key),
                // The cereal document refuses these keys: an error that fails the block.
                Severity::Error,
                child(at, &[key]),
            ));
        } else if rejected.is_none() && NOT_IN_LEGACY.contains(&key.as_str()) {
            unread.push((key.clone(), value.clone()));
        } else {
            kept.insert(key.clone(), value.clone());
        }
    }
    // The legacy reader parses each component with its cereal description under a lenient
    // error mask too: a member of the wrong type or a required member missing still fails the
    // load. The game logs "Failed to parse block '%s' " and each error at the content log's
    // error level, the component load fails with it, and no block definition is generated:
    // the block is refused, as under the strict slice.
    let strict;
    let load = if cx.load.strict_components {
        cx.load
    } else {
        let mut with_strict = cx.load.clone();
        with_strict.strict_components = true;
        strict = with_strict;
        &strict
    };
    // The game's legacy component parser reads every cereal-registered description with
    // its non-strict schema reader (the strict flag of the component metadata is clear): a bool and a number read as each other, an
    // integer token by its low bits, a real by its truncation. A cereal document is the strict
    // reader's.
    let reader = if rejected.is_some() {
        ReaderMode::Strict
    } else {
        ReaderMode::NonStrict
    };
    let mut map = with_reader_mode(reader, || {
        parse_component_map::<BlockFamily>(
            &kept,
            payload,
            at,
            ConvertMode::Strict,
            is_custom,
            load,
            d,
        )
    });
    for (key, raw) in unread {
        // The schema of the legacy document has no child for the key: the game logs "child
        // '%s' not valid here." at the content log's error level, validation fails and with
        // it the component load: the block is refused.
        d.push(finding(
            DiagnosticCode::UnrecognizedComponent {
                text: Some(VanillaText::Formatted {
                    template: CHILD_NOT_VALID,
                    args: vec![key.clone()],
                }),
            },
            Severity::Error,
            child(at, &[&key]),
        ));
        let json =
            bedrock_addon_core::json::from_str_jsonc::<Value>(raw.as_str()).unwrap_or(Value::NULL);
        map.insert_unknown(&key, json);
    }
    map
}

/// The game's line for a key the legacy document's schema has no child for.
const CHILD_NOT_VALID: &str = "child '{}' not valid here.";

/// The components no legacy description exists for: the legacy reader of a document below
/// 1.26.20 leaves them to the document's schema, which has no child for them: the block is
/// refused, and the crate keeps them as the unknown components they are.
const NOT_IN_LEGACY: [&str; 4] = [
    "minecraft:block_entity",
    "minecraft:enchantment_power_transmitter",
    "minecraft:instrument_sound",
    "minecraft:neighbor_change",
];

/// One entry of a map-valued member, read from its own slice so its unknown members are kept.
/// The legacy events are hand-written in the game and the crate's side-car has no game
/// counterpart, so what is told here is the crate's own: informations. In a cereal document the member
/// itself refuses the block (`reject_crate_members`).
fn parse_entry<T: for<'f> Facet<'f>>(
    payload: JsonSlice<'_>,
    raw: &RawJson<'_>,
    at: &JsonPath,
    d: &mut Diagnostics,
) -> Option<T> {
    let slice = payload.nested(raw)?;
    match parse_at::<T>(slice) {
        Ok(mut value) => {
            if let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text) {
                for (path, _) in keep_unknown_members(&mut value, &json) {
                    let mut full = at.clone();
                    full.append(&path);
                    d.push(finding(
                        DiagnosticCode::UnrecognizedField { text: None },
                        Severity::Info,
                        full,
                    ));
                }
            }
            Some(value)
        }
        Err(error) => {
            let mut full = at.clone();
            full.append(&error.path);
            d.push(finding(
                DiagnosticCode::InvalidType { text: None },
                Severity::Info,
                full,
            ));
            // The entry goes whole for one member: told at the entry too, unless the member's finding is at the entry itself.
            if !error.path.is_empty() {
                d.push(finding(
                    DiagnosticCode::DroppedField { text: None },
                    Severity::Info,
                    at.clone(),
                ));
            }
            None
        }
    }
}

/// `base` followed by `keys`.
pub(crate) fn child(base: &JsonPath, keys: &[&str]) -> JsonPath {
    let mut path = base.clone();
    for key in keys {
        path.push(key);
    }
    path
}

/// A parse finding at `path`.
fn finding(code: DiagnosticCode, severity: Severity, path: JsonPath) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Parse,
        Location::new(None, None, path, None),
    )
}

// ---- Molang  -------------------

/// `permutations[].condition` of a 1.26.20 document: a value or `{expression, version}`, plain
/// strings at version 13, the default query set; an empty text or version −1 is no
/// expression; a rejected expression is kept and never matches. The game logs the parser's
/// lines at the content log's error level and builds the node without a failure, so the block
/// loads: kept errors.
pub(crate) const CONDITION: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::LenientVersioned,
    version: MolangVersionSource::Constant(MolangVersion::V13),
    admission: Admission::Default,
    ops: OpsRule::All,
    empty: EmptyRule::BlockNoExpression,
    on_failure: MolangFailure::KeepLogged,
    error_severity: Severity::Error,
    error_kept: true,
};

/// `permutations[].condition` of a document in `[1.19.70, 1.26.20)`: a string at the pack's
/// version, `{query.block_state}` only, no side effects; a rejected expression rejects
/// the block ("permutation condition failed to parse").
pub(crate) const CONDITION_LEGACY: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::SchemaValidated,
    version: MolangVersionSource::Pack,
    admission: Admission::Only(BLOCK_STATE_QUERIES),
    ops: OpsRule::NoAssignmentsOrRandom,
    empty: EmptyRule::NoExpression,
    on_failure: MolangFailure::RejectDocument,
    error_severity: Severity::Error,
    error_kept: false,
};

/// `permutations[].condition` of a document below 1.19.70: the `{query.block_state}`-only rule
/// of the legacy slice, and a condition that fails it is compiled again under the
/// unrestricted rule (validate). The first parse logs its lines at the content log's error
/// level and the load goes on from the retry, the block registered: kept errors.
pub(crate) const CONDITION_OLD: MolangFieldSpec = MolangFieldSpec {
    on_failure: MolangFailure::DropEntry,
    error_kept: true,
    ..CONDITION_LEGACY
};

/// The condition spec of a document of `version`.
pub(crate) fn condition_spec(version: FormatVersion) -> &'static MolangFieldSpec {
    if version >= CEREAL {
        &CONDITION
    } else if version >= CONDITION_STRICT {
        &CONDITION_LEGACY
    } else {
        &CONDITION_OLD
    }
}

impl MolangFields for Block {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.nested("components", &mut self.components);
        visitor.nested("deprecated", &mut self.deprecated);
        let spec = condition_spec(self.source_format_version);
        visitor.enter("permutations".into());
        for (index, permutation) in self.permutations.iter_mut().enumerate() {
            visitor.enter(index.into());
            visitor.field("condition", spec, &mut permutation.condition);
            visitor.nested("components", &mut permutation.components);
            visitor.nested("deprecated", &mut permutation.deprecated);
            visitor.leave();
        }
        visitor.leave();
        visitor.nested(
            "deprecated_event_handlers",
            &mut self.deprecated_event_handlers,
        );
    }
}
