//! Reading [`Block`] from the text of a payload (the game's cereal document, and the payload a
//! legacy document converts to), and the Molang walk of the document.

use crate::codes;
use facet::Facet;
use indexmap::IndexMap;

use super::component::{
    BlockComponent, BlockEntity, DeprecatedComponents, DeprecatedDefinitionEvent,
    read_collision_box,
};
use super::history::{BlockFamily, BlockHistory};
use super::{Block, Description, Permutation};
use bedrock_addon_core::diagnostic::cereal;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::document::{ComponentMap, Document, Origin, Rejected};
use bedrock_addon_core::id::syntax::split_namespace;
use bedrock_addon_core::json::Prepared;
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{
    BLOCK_STATE_QUERIES, EmptyRule, Molang, MolangFailure, MolangFieldSpec, MolangFields,
    MolangVersion, MolangVersionSource, MolangVisitor, OpsRule, ReaderKind,
};
use bedrock_addon_core::schema::{self, FromJson, Schema, read_document};
use bedrock_addon_core::version::FormatVersionInput;
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{
    ConvertMode, DocumentRead, Family, Identity, KeyGate, Loaded, ParseCx, Parsed, Scalars, finish,
    parse_as_written, read_component_map_by, read_payload_with,
};

/// The first version vanilla reads with the cereal document.
pub(crate) const CEREAL: FormatVersion = known::V1_26_20;
/// The newest document's version.
pub(crate) const NEWEST: FormatVersion = known::V1_26_60;
/// The version from which a legacy permutation condition that fails is an error.
pub(crate) const CONDITION_STRICT: FormatVersion = FormatVersion::new(1, 19, 70);

/// What a payload is read into.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
struct Shell {
    description: Description,
    #[facet(opaque, default, schema::read_with = base_section)]
    components: Section,
    #[facet(default)]
    permutations: Vec<PermutationShell>,
    #[facet(default, schema::read_with = ours::<DeprecatedComponents>)]
    deprecated: DeprecatedComponents,
    #[facet(default, schema::read_with = handlers)]
    deprecated_event_handlers: IndexMap<String, DeprecatedDefinitionEvent>,
    #[facet(opaque, default)]
    events: Events,
    #[facet(default, schema::read_with = ordered)]
    ordered_v1_custom_components: Vec<String>,
}

/// What a permutation is read into.
#[derive(Facet, Schema)]
#[facet(schema::unknown = "keep")]
struct PermutationShell {
    condition: Molang,
    #[facet(opaque, default, schema::read_with = permutation_section)]
    components: Section,
    #[facet(default, schema::read_with = ours::<DeprecatedComponents>)]
    deprecated: DeprecatedComponents,
    #[facet(default, schema::read_with = ordered)]
    ordered_v1_custom_components: Vec<String>,
}

/// One `components` object, read: the map, and whether its `minecraft:collision_box` is one box
/// written as an object (the 1.26.20 alternative, which the 1.26.20 struct writes back as it).
#[derive(Default)]
struct Section {
    map: ComponentMap<BlockComponent>,
    one_box: bool,
}

// A section reads its own members and holds no Molang, no bound and no trigger of its own.
bedrock_addon_core::impl_no_molang!(Section);
bedrock_addon_core::impl_schema_leaf!(Section);

/// `events`, deprecated by name in the cereal document: the game checks that before it reads the
/// value and then skips it, so any value is held, `null` included, and told by what it is.
#[derive(Default, Facet)]
#[facet(opaque)]
struct Events;

impl FromJson for Events {
    const HOLDS_NULL: bool = true;

    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let version = read.known_format()?;
        let mut finding = if version >= CEREAL {
            Diagnostic::deprecated_member("events", node.path().clone())
        } else if node
            .value()
            .as_object()
            .is_some_and(|events| !events.is_empty())
        {
            // The legacy event reader takes them with no deprecation line (the deprecation is the
            // cereal document's), so the crate reports them as an information.
            Diagnostic::of(
                core_codes::DEPRECATED_IN_FORMAT.with(["events".to_string(), version.to_string()]),
            )
            .severity(Severity::Info)
            .at(node.path().clone())
        } else {
            return Some(Events);
        };
        finding.location.span = node.span();
        read.push(finding);
        Some(Events)
    }
}

bedrock_addon_core::impl_no_molang!(Events);
bedrock_addon_core::impl_schema_leaf!(Events);

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

/// The components no legacy description exists for: the legacy reader of a document below
/// 1.26.20 leaves them to the document's schema, which has no child for them: the block is
/// refused, and the crate keeps them as the unknown components they are.
const NOT_IN_LEGACY: [&str; 4] = [
    "minecraft:block_entity",
    "minecraft:enchantment_power_transmitter",
    "minecraft:instrument_sound",
    "minecraft:neighbor_change",
];

/// A key of another namespace than `minecraft` is a custom component.
pub(crate) fn is_custom(key: &str) -> bool {
    split_namespace(key)
        .is_some_and(|(namespace, name)| namespace != "minecraft" && !name.is_empty())
}

/// Whether the read is of a cereal document (the version it reads is 1.26.20 or later).
fn cereal_read(read: &Read<'_>) -> bool {
    read.known_format().is_some_and(|v| v >= CEREAL)
}

/// The base `components`.
fn base_section(node: &Node<'_>, read: &mut Read<'_>) -> Option<Section> {
    section(node, read, false)
}

/// A permutation's `components`.
fn permutation_section(node: &Node<'_>, read: &mut Read<'_>) -> Option<Section> {
    section(node, read, true)
}

/// One `components` object: the rejected keys refused ("key {} is not allowed", an error that
/// fails the block), the ones no legacy description exists for left unread (`child '%s' not valid
/// here.`, an error that refuses the block: the schema of the legacy document has no child for
/// the key), the rest read one at a time by their rows, under the reading of the slice's kind
/// (set on the read's load: a legacy description is the non-strict reader's, a cereal document
/// the strict one; the game's legacy component parser reads each cereal-registered description
/// with its non-strict schema reader, and a failed component or a missing required member still
/// fails the load). Then the collision box and the members only the 1.26.60 document binds.
fn section(node: &Node<'_>, read: &mut Read<'_>, permutation: bool) -> Option<Section> {
    let version = read.known_format()?;
    let cereal = version >= CEREAL;
    let map =
        read_component_map_by::<BlockFamily>(node, read, ConvertMode::Strict, is_custom, |key| {
            let refused = REJECTED_IN_BASE.contains(&key)
                || (permutation && REJECTED_IN_PERMUTATIONS.contains(&key));
            if cereal && refused {
                // The cereal document refuses these keys: an error that fails the block.
                KeyGate::Refuse(Diagnostic::error(cereal::key_not_allowed(key)))
            } else if !cereal && NOT_IN_LEGACY.contains(&key) {
                KeyGate::Unread(Diagnostic::of(codes::CHILD_NOT_VALID.with([key])))
            } else {
                KeyGate::Read
            }
        });
    let mut map = map?;
    let unknown = read.unknown_grade();
    let at = node.path();
    let one = node
        .value()
        .as_object()
        .and_then(|components| components.get(COLLISION_BOX))
        .is_some_and(Value::is_object);
    let found = read.findings_mut();
    let one_box = read_collision_box(
        &mut map,
        one,
        &at.child(COLLISION_BOX),
        version,
        unknown,
        found,
    );
    if !permutation {
        newest_members(&mut map, at, version, unknown, found);
    }
    Some(Section { map, one_box })
}

/// The finding of a member of ours that no cereal document takes (`deprecated`,
/// `deprecated_event_handlers`): an unknown member there, at the slice's grade, which refuses the
/// block. It is read as ever.
fn ours<T>(node: &Node<'_>, read: &mut Read<'_>) -> Option<T>
where
    T: FromJson,
{
    let value = T::from_json(node, read)?;
    if cereal_read(read) {
        let mut finding = Diagnostic::of(&core_codes::UNRECOGNIZED_FIELD)
            .severity(read.unknown_grade())
            .at(node.path().clone());
        finding.location.span = node.span();
        read.push(finding);
    }
    Some(value)
}

/// `ordered_v1_custom_components`: bound and deprecated by name in the cereal document.
fn ordered(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    let names = Vec::<String>::from_json(node, read)?;
    if cereal_read(read) {
        let mut finding =
            Diagnostic::deprecated_member("ordered_v1_custom_components", node.path().clone());
        finding.location.span = node.span();
        read.push(finding);
    }
    Some(names)
}

/// `deprecated_event_handlers`: each entry is read on its own, so a bad one goes whole while the
/// rest stay. The legacy events are hand-written in the game and the crate's side-car has no game
/// counterpart, so what is told here is the crate's own: informations (in a cereal document the
/// member itself refuses the block, `ours`).
fn handlers(
    node: &Node<'_>,
    read: &mut Read<'_>,
) -> Option<IndexMap<String, DeprecatedDefinitionEvent>> {
    let entries = ours::<Handlers>(node, read)?;
    Some(entries.0)
}

/// The handlers as the member reads them.
struct Handlers(IndexMap<String, DeprecatedDefinitionEvent>);

impl FromJson for Handlers {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        let members = read.object(node)?;
        let mut handlers = IndexMap::new();
        for (name, entry) in members.iter() {
            let mut told = Diagnostics::default();
            let event = {
                let mut attempt = Read::new(read.reading(), &mut told)
                    .profile(Profile::FIRST_FAILURE)
                    .unknown_severity(Severity::Info);
                read_document::<DeprecatedDefinitionEvent>(&entry, &mut attempt)
            };
            let kept = told.take_unknown();
            if let Some(event) = event {
                for finding in told {
                    read.push(finding);
                }
                read.findings_mut()
                    .unknown_mut()
                    .absorb(&JsonPath::new(), kept);
                handlers.insert(name.to_owned(), event);
            } else if let Some(first) = told.iter().next() {
                let mut finding = Diagnostic::of(&core_codes::INVALID_TYPE)
                    .severity(Severity::Info)
                    .at(first.location.json_path.clone());
                finding.location.span = first.location.span;
                let inside = first.location.json_path != *entry.path();
                read.push(finding);
                // The entry goes whole for one member: told at the entry too, unless the
                // member's finding is at the entry itself.
                if inside {
                    read.push(Diagnostic::of(&core_codes::DROPPED_FIELD).at(entry.path().clone()));
                }
            }
        }
        Some(Handlers(handlers))
    }
}

// The handlers are read entry by entry and hold no Molang, no bound and no trigger of their own.
bedrock_addon_core::impl_no_molang!(Handlers);
bedrock_addon_core::impl_schema_leaf!(Handlers);

/// Reads one payload of the 1.26.20 shape. `cx.original_version` is the document's version (for
/// the 1.26.20 rules) or, when the 1.26.20 step converts a legacy document, the legacy one.
pub(crate) fn parse_block(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<Block> {
    parse_document(payload, cx, d).map(|(block, _)| block)
}

/// The payload's shell. The load is the slice's kind of reading (`reading_of`: a legacy
/// description is the non-strict reader's, a cereal document the strict one) for the components;
/// the shell's own members are strict in both, as the strict wrappers they were read by.
fn read_shell(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Shell> {
    let mut load = cx.load.clone();
    load.reading = crate::reading_of(cx.original_version >= CEREAL);
    load.strict_components = true;
    let parse = ParseCx {
        load: &load,
        written: cx.written,
        original_version: cx.original_version,
        at: cx.at.clone(),
    };
    read_payload_with(payload, &parse, d, |read| read.scalars(Scalars::Strict))
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
    let shell = read_shell(payload, cx, d)?;
    if cx.original_version >= CEREAL {
        unknown_traits(d, start, &cx.at);
    }
    let mut one_box = Vec::new();
    if shell.components.one_box {
        one_box.push(None);
    }
    let mut permutations = Vec::with_capacity(shell.permutations.len());
    for (index, p) in shell.permutations.into_iter().enumerate() {
        if p.components.one_box {
            one_box.push(Some(index));
        }
        permutations.push(Permutation {
            condition: p.condition,
            components: p.components.map,
            deprecated: p.deprecated,
            ordered_v1_custom_components: p.ordered_v1_custom_components,
        });
    }
    let block = Block {
        description: shell.description,
        components: shell.components.map,
        deprecated: shell.deprecated,
        permutations,
        deprecated_event_handlers: shell.deprecated_event_handlers,
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
    let Some(entity) = map.get_mut::<BlockEntity>() else {
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
