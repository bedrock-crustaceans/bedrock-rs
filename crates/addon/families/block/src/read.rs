//! Reading [`Block`] from the text of a 1.26.20 payload (the game's cereal
//! document), and the Molang walk of the document.

use crate::codes;
use facet::Facet;
use facet_json::RawJson;
use indexmap::IndexMap;

use super::component::{BlockComponent, DeprecatedComponents, DeprecatedDefinitionEvent};
use super::history::{BlockFamily, BlockHistory};
use super::{Block, Description, Permutation};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Class, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{ComponentMap, Document, Origin, Rejected};
use bedrock_addon_core::id::syntax::split_namespace;
use bedrock_addon_core::json::Prepared;
use bedrock_addon_core::json::{JsonSlice, ScalarRules, Value, keep_unknown_members, parse_at};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{
    BLOCK_STATE_QUERIES, EmptyRule, Molang, MolangFailure, MolangFieldSpec, MolangFields,
    MolangVersion, MolangVersionSource, MolangVisitor, OpsRule, ReaderKind,
};
use bedrock_addon_core::schema;
use bedrock_addon_core::version::FormatVersionInput;
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{ConvertMode, ParseCx, parse_component_map, parse_payload};
use bedrock_addon_core::versioning::{
    DocumentRead, Family, Identity, Loaded, Parsed, finish, parse_as_written,
};

/// The first version vanilla reads with the cereal document.
pub(crate) const CEREAL: FormatVersion = known::V1_26_20;
/// The newest document's version.
pub(crate) const NEWEST: FormatVersion = known::V1_26_60;
/// The version from which a legacy permutation condition that fails is an error.
pub(crate) const CONDITION_STRICT: FormatVersion = FormatVersion::new(1, 19, 70);

/// What a payload is read into: components as raw slices, parsed one at a time.
#[derive(Facet)]
#[facet(schema::unknown = "keep")]
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
    events: Option<Value>,
    #[facet(default)]
    ordered_v1_custom_components: Vec<String>,
}

/// What a permutation is read into.
#[derive(Facet)]
#[facet(schema::unknown = "keep")]
struct PermutationShell<'a> {
    condition: Molang,
    #[facet(default)]
    components: IndexMap<String, RawJson<'a>>,
    #[facet(default)]
    deprecated: DeprecatedComponents,
    #[facet(default)]
    ordered_v1_custom_components: Vec<String>,
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
    split_namespace(key)
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

/// The payload's shell. Its own members (the description's, the custom component order) are
/// plain types that facet converts and the misread walk tells: read as they always were,
/// whatever the slice's scalar rules, until they are its rule's too. The components are read
/// by the slice's (`section`).
fn parse_shell<'a>(
    payload: JsonSlice<'a>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<Shell<'a>> {
    if cx.load.scalar_rules == ScalarRules::PLAIN {
        return parse_payload(payload, cx, d);
    }
    let mut load = cx.load.clone();
    load.scalar_rules = ScalarRules::PLAIN;
    let plain = ParseCx {
        load: &load,
        written: cx.written,
        original_version: cx.original_version,
        at: cx.at.clone(),
    };
    parse_payload(payload, &plain, d)
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
    let shell = parse_shell(payload, cx, d)?;
    let cereal = cx.original_version >= CEREAL;
    if cereal {
        unknown_traits(d, start, &cx.at);
    }
    let unknown = cx.load.grade(Class::Unknown);
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
        reject_crate_members(root, &cx.at, unknown, d);
    }
    let mut one_box = Vec::new();
    let mut components = section(
        &shell.components,
        payload,
        &cx.at.with(&["components"]),
        cereal.then_some(&[][..]),
        cx,
        d,
    );
    if super::component::read_collision_box(
        &mut components,
        shell.components.get(COLLISION_BOX),
        &cx.at.with(&["components", COLLISION_BOX]),
        cx.original_version,
        unknown,
        d,
    ) {
        one_box.push(None);
    }
    newest_members(
        &mut components,
        &cx.at.with(&["components"]),
        cx.original_version,
        unknown,
        d,
    );
    let mut permutations = Vec::with_capacity(shell.permutations.len());
    for (index, p) in shell.permutations.iter().enumerate() {
        let mut at = cx.at.with(&["permutations"]);
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
            &at.with(&[COLLISION_BOX]),
            cx.original_version,
            unknown,
            d,
        ) {
            one_box.push(Some(index));
        }
        permutations.push(Permutation {
            condition: p.condition.clone(),
            components,
            deprecated: p.deprecated.clone(),
            ordered_v1_custom_components: p.ordered_v1_custom_components.clone(),
        });
    }
    let mut deprecated_event_handlers = IndexMap::new();
    for (name, raw) in &shell.deprecated_event_handlers {
        let at = cx.at.with(&["deprecated_event_handlers", name]);
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
        refused_at: Vec::new(),
    };
    Some((block, one_box))
}

/// `minecraft:collision_box`.
const COLLISION_BOX: &str = "minecraft:collision_box";

/// The members only the 1.26.60 document binds, in a 1.26.20 one: `block_entity.entity_storage`
/// is an unknown member there (`UnrecognizedField` at `unknown`, an error in the strict slice;
/// held with the component's unknown members). `block_entity` is a base component.
fn newest_members(
    map: &mut ComponentMap<BlockComponent>,
    at: &JsonPath,
    version: FormatVersion,
    unknown: Severity,
    d: &mut Diagnostics,
) {
    if !(CEREAL..NEWEST).contains(&version) {
        return;
    }
    let Some(entity) = map.get_mut::<super::component::BlockEntity>() else {
        return;
    };
    if let Some(value) = entity.entity_storage.take() {
        // The strict slice's error that refuses the block: only reachable in
        // [1.26.20, 1.26.60), never a warning the load keeps.
        d.push(
            Diagnostic::of(&core_codes::UNRECOGNIZED_FIELD)
                .severity(unknown)
                .at(at.with(&["minecraft:block_entity", "entity_storage"])),
        );
        d.unknown_mut().put(
            &at.child("minecraft:block_entity"),
            "entity_storage",
            Value::from(value),
        );
    }
}

/// `events`: bound and deprecated by name in the cereal document, an error that
/// rejects the block; elsewhere a deprecated member, told.
fn tell_events(written: bool, events: Option<&Value>, cx: &ParseCx<'_>, d: &mut Diagnostics) {
    if written {
        d.push(Diagnostic::deprecated_member(
            "events",
            cx.at.with(&["events"]),
        ));
    } else if events
        .and_then(Value::as_object)
        .is_some_and(|events| !events.is_empty())
    {
        // The legacy event reader takes them with no deprecation line (the deprecation is the
        // cereal document's), so the crate reports them as an information.
        d.push(
            Diagnostic::of(
                core_codes::DEPRECATED_IN_FORMAT
                    .with(["events".to_string(), cx.original_version.to_string()]),
            )
            .severity(Severity::Info)
            .at(cx.at.with(&["events"])),
        );
    }
}

/// The crate's own members a cereal document has: `deprecated` (of the block or of a
/// permutation) and `deprecated_event_handlers` are no members of the game's document
/// (`UnrecognizedField` at `unknown`, the strict slice's error);
/// `ordered_v1_custom_components` (of either) is bound and deprecated by name.
/// Each rejects the block; what the shell read of them is not used.
fn reject_crate_members(root: &Value, at: &JsonPath, unknown: Severity, d: &mut Diagnostics) {
    let mut sections = vec![(root, at.clone())];
    for (index, permutation) in root
        .as_object()
        .and_then(|root| root.get("permutations"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let mut path = at.with(&["permutations"]);
        path.push_index(index);
        sections.push((permutation, path));
    }
    for (index, (object, path)) in sections.into_iter().enumerate() {
        let Some(object) = object.as_object() else {
            continue;
        };
        let ours: &[&str] = if index == 0 {
            &["deprecated", "deprecated_event_handlers"]
        } else {
            &["deprecated"]
        };
        for key in ours {
            if object.contains_key(key) {
                // The strict slice's error that refuses the block (cereal documents alone
                // reach here).
                d.push(
                    Diagnostic::of(&core_codes::UNRECOGNIZED_FIELD)
                        .severity(unknown)
                        .at(path.with(&[key])),
                );
            }
        }
        if object.contains_key("ordered_v1_custom_components") {
            d.push(Diagnostic::deprecated_member(
                "ordered_v1_custom_components",
                path.with(&["ordered_v1_custom_components"]),
            ));
        }
    }
}

/// A trait name the game does not have is told by its reader with its own text, `Failed to
/// find BlockTrait with identifier: '{}'`, an error that rejects the block:
/// the walk's `UnrecognizedField` at `description/traits/<name>` of a cereal document is that
/// finding, told once (the rule of `validate` never runs on a block the strict slice rejects).
fn unknown_traits(d: &mut Diagnostics, start: usize, at: &JsonPath) {
    let traits = at.with(&["description", "traits"]);
    for finding in d.iter_mut().skip(start) {
        // The cereal document's unknown members are core's unrecognized-field definitions.
        if !matches!(
            finding.code.id(),
            "core/child_not_valid"
                | "core/member_not_in_schema"
                | "core/unrecognized_field"
                | "core/unrecognized_field_worded"
        ) {
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
        finding.code = codes::UNKNOWN_TRAIT.with([name]);
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
            d.push(
                // The cereal document refuses these keys: an error that fails the block.
                Diagnostic::error(bedrock_addon_core::diagnostic::cereal::key_not_allowed(key))
                    .at(at.with(&[key])),
            );
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
    // The game's legacy component parser reads every cereal-registered description with
    // its non-strict schema reader (the strict flag of the component metadata is clear): a bool and a number read as each other, an
    // integer token by its low bits, a real by its truncation. A cereal document is the strict
    // reader's. Both refuse a number beyond its float. The components' members are read by the
    // context's scalar rules: the `Reading` of the document's kind of slice (`reading_of`), set
    // here too for a read outside a versioned load and for a legacy document converted by the
    // 1.26.20 step.
    let scalar_rules = crate::reading_of(rejected.is_some()).scalar_rules();
    let strict;
    let load = if cx.load.strict_components && cx.load.scalar_rules == scalar_rules {
        cx.load
    } else {
        let mut with_strict = cx.load.clone();
        with_strict.strict_components = true;
        with_strict.scalar_rules = scalar_rules;
        strict = with_strict;
        &strict
    };
    let mut map = parse_component_map::<BlockFamily>(
        &kept,
        payload,
        at,
        ConvertMode::Strict,
        is_custom,
        load,
        d,
    );
    for (key, raw) in unread {
        // The schema of the legacy document has no child for the key: the game logs "child
        // '%s' not valid here." at the content log's error level, validation fails and with
        // it the component load: the block is refused.
        d.push(Diagnostic::of(codes::CHILD_NOT_VALID.with([key.clone()])).at(at.with(&[&key])));
        let json =
            bedrock_addon_core::json::from_str_jsonc::<Value>(raw.as_str()).unwrap_or(Value::NULL);
        map.insert_unknown(&key, json);
    }
    map
}

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
        Ok(value) => {
            if let Ok(json) = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text) {
                let mut kept = bedrock_addon_core::document::Unknown::new();
                let found = keep_unknown_members(&value, &json, &mut kept);
                d.unknown_mut().absorb(at, kept);
                for (path, _) in found {
                    let mut full = at.clone();
                    full.append(&path);
                    d.push(
                        Diagnostic::of(&core_codes::UNRECOGNIZED_FIELD)
                            .severity(Severity::Info)
                            .at(full),
                    );
                }
            }
            Some(value)
        }
        Err(error) => {
            let mut full = at.clone();
            full.append(&error.path);
            d.push(
                Diagnostic::of(&core_codes::INVALID_TYPE)
                    .severity(Severity::Info)
                    .at(full),
            );
            // The entry goes whole for one member: told at the entry too, unless the member's finding is at the entry itself.
            if !error.path.is_empty() {
                d.push(Diagnostic::of(&core_codes::DROPPED_FIELD).at(at.clone()));
            }
            None
        }
    }
}

// ---- Molang  -------------------

/// `permutations[].condition` of a 1.26.20 document: a value or `{expression, version}`, plain
/// strings at version 13, the default query set; an empty text or version −1 is no
/// expression; a rejected expression is kept and never matches. The game logs the parser's
/// lines at the content log's error level and builds the node without a failure, so the block
/// loads: kept errors.
pub(crate) const CONDITION: MolangFieldSpec = MolangFieldSpec::KEPT
    .reader(ReaderKind::LenientVersioned)
    .version(MolangVersionSource::Constant(MolangVersion::V13))
    .empty(EmptyRule::BlockNoExpression);

/// `permutations[].condition` of a document in `[1.19.70, 1.26.20)`: a string at the pack's
/// version, `{query.block_state}` only, no side effects; a rejected expression rejects
/// the block ("permutation condition failed to parse").
pub(crate) const CONDITION_LEGACY: MolangFieldSpec = MolangFieldSpec::REJECTING
    .reader(ReaderKind::SchemaValidated)
    .only(BLOCK_STATE_QUERIES)
    .ops(OpsRule::NoAssignmentsOrRandom);

/// `permutations[].condition` of a document below 1.19.70: the `{query.block_state}`-only rule
/// of the legacy slice, and a condition that fails it is compiled again under the
/// unrestricted rule (validate). The first parse logs its lines at the content log's error
/// level and the load goes on from the retry, the block registered: kept errors.
pub(crate) const CONDITION_OLD: MolangFieldSpec =
    CONDITION_LEGACY.on_failure(MolangFailure::DropEntry).kept();

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

// ---- one file -------------------------------------------------------------------------

/// `history::upgrade`: rebuilds the parse stage from the document and finishes it.
pub(crate) fn upgrade_document(
    document: Document<BlockHistory>,
    cx: &LoadContext,
) -> Result<Document<Block>, Rejected> {
    let (value, origin, version, diagnostics) = document.into_parsed_parts();
    let identity = Identity {
        identifier: Some(identifier_of(&value)),
        runtime_identifier: None,
    };
    let at = JsonPath::new().child(BlockFamily::PAYLOAD_KEY);
    let written = FormatVersionInput {
        version: Some(version.effective),
        beta_literal: false,
        prerelease: false,
    };
    let parsed = Parsed::<BlockFamily>::from_value(Some(value), version, written, at, diagnostics)
        .with_identity(identity);
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(block)) => {
            Ok(Document::new(block, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The identifier of a parsed document, as written.
fn identifier_of(value: &BlockHistory) -> String {
    match value {
        BlockHistory::V1_26_60(block) => block.description.identifier.as_str().to_owned(),
        BlockHistory::V1_26_20(b) => b.block.description.identifier.as_str().to_owned(),
        BlockHistory::V1_16_0(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_16_100(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_18_0(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_19_10(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_19_20(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_19_30(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_19_40(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_19_80(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_20_10(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_20_60(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_21_30(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_21_60(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_21_90(b) => legacy_identifier(&b.0.description),
        BlockHistory::V1_26_0(b) => legacy_identifier(&b.0.description),
    }
}

/// `identifier` of a legacy description.
fn legacy_identifier(
    description: &indexmap::IndexMap<String, bedrock_addon_core::json::Value>,
) -> String {
    description
        .get("identifier")
        .and_then(|v| v.as_string())
        .map(|s| s.as_str().to_owned())
        .unwrap_or_default()
}

/// The finding of an unreadable file.
fn malformed() -> Diagnostic {
    Diagnostic::of(core_codes::SYNTAX.with(["not UTF-8 JSON within the nesting limit"]))
}

/// Reads one block file at `origin`: the document, or its rejection with the identifier its
/// payload names (when it names one). `parse_document`'s steps, but for the path of a legacy
/// file's findings. It never calls `Family::close`: block has none, the default does nothing.
pub(crate) fn read_file(
    origin: Origin,
    bytes: &[u8],
    cx: &LoadContext,
) -> DocumentRead<BlockFamily> {
    // The BOM is blanked: spans stay offsets into the file as written.
    let Ok(prepared) = Prepared::new(bytes) else {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(malformed());
        return DocumentRead::Rejected(None, Rejected::located(origin, diagnostics));
    };
    let mut parsed = parse_as_written::<BlockFamily>(prepared.slice(), cx);
    // A file below 1.26.20 is read whole (its envelope is not checked), so the parse stage
    // gives the root as its path; the block it becomes is the payload, under its key, where
    // the legacy reader's findings are: the checks on the upgraded block report there too,
    // core's component gates and Molang pass 2 as well as the family's rules.
    if parsed.at.is_empty() {
        parsed.at.push(BlockFamily::PAYLOAD_KEY);
    }
    let identifier = parsed.identity.identifier.clone();
    let outcome = finish(parsed, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(block)) => {
            DocumentRead::Stable(Document::new(block, origin, outcome.version, diagnostics))
        }
        _ => DocumentRead::Rejected(identifier, Rejected::new(origin, diagnostics)),
    }
}

#[cfg(test)]
mod spec_tests {
    use crate::component::{BONE_VISIBILITY, DESCRIPTOR_TAGS, ITEM_TAGS, TRIGGER_CONDITION};
    use crate::read::{CONDITION, CONDITION_LEGACY, CONDITION_OLD};
    use bedrock_addon_core::diagnostic::Severity;
    use bedrock_addon_core::molang::{
        Admission, BLOCK_STATE_QUERIES, EmptyRule, MolangFailure, MolangFieldSpec, MolangVersion,
        MolangVersionSource, OpsRule, ReaderKind,
    };

    /// A spec at the error severity.
    fn spec(
        reader: ReaderKind,
        version: MolangVersionSource,
        admission: Admission,
        ops: OpsRule,
        empty: EmptyRule,
        on_failure: MolangFailure,
        error_kept: bool,
    ) -> MolangFieldSpec {
        MolangFieldSpec {
            reader,
            version,
            admission,
            ops,
            empty,
            on_failure,
            error_severity: Severity::Error,
            error_kept,
        }
    }

    /// The block files' Molang specs, field by field, as the game reads each member.
    #[test]
    fn the_block_molang_specs_keep_their_fields() {
        use EmptyRule::{BlockNoExpression, NoExpression};
        use MolangFailure::{DropEntry, KeepLogged, RejectDocument};
        use OpsRule::{All, NoAssignmentsOrRandom};
        use ReaderKind::{LenientVersioned, SchemaValidated, StrictVersioned};
        let v13 = MolangVersionSource::Constant(MolangVersion::V13);
        let states = Admission::Only(BLOCK_STATE_QUERIES);
        let lenient = spec(
            LenientVersioned,
            v13,
            Admission::Default,
            All,
            BlockNoExpression,
            KeepLogged,
            true,
        );
        assert_eq!(CONDITION, lenient);
        assert_eq!(TRIGGER_CONDITION, lenient);
        let legacy = spec(
            SchemaValidated,
            MolangVersionSource::Pack,
            states,
            NoAssignmentsOrRandom,
            NoExpression,
            RejectDocument,
            false,
        );
        assert_eq!(CONDITION_LEGACY, legacy);
        assert_eq!(
            CONDITION_OLD,
            MolangFieldSpec {
                on_failure: DropEntry,
                error_kept: true,
                ..legacy
            }
        );
        let bones = spec(
            LenientVersioned,
            v13,
            states,
            NoAssignmentsOrRandom,
            BlockNoExpression,
            KeepLogged,
            true,
        );
        assert_eq!(BONE_VISIBILITY, bones);
        let tags = spec(
            StrictVersioned,
            v13,
            Admission::Tags,
            All,
            BlockNoExpression,
            KeepLogged,
            true,
        );
        assert_eq!(DESCRIPTOR_TAGS, tags);
        assert_eq!(ITEM_TAGS, tags);
    }
}
