//! The family rules on the loaded entity, run once on
//! the newest struct (or the beta struct) against the document's original version.

use std::collections::HashSet;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::document::{ComponentMap, ComponentRef};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::load::{Experiment, LoadRequirement};
use bedrock_addon_core::molang::{
    Molang, MolangContext, MolangFieldSpec, MolangFields, MolangVisitor, ReaderKind, Step,
    not_checked, pass_one,
};
use bedrock_addon_core::shared::{BlockDescriptor, EventTrigger, Filter};
use bedrock_addon_core::vanilla::ActorType;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput, known};
use bedrock_addon_core::versioning::ValidateCx;
use indexmap::IndexMap;

use super::component::EntityComponent;
use super::game_schema;
use super::{Entity, MemoryType, PropertyType};

/// `ROOT_LEVEL_FILTERS_VERSION`: from 1.19.20 a root-level event `filters` is read.
const ROOT_LEVEL_FILTERS: FormatVersion = FormatVersion::new(1, 19, 20);
/// `START_FAILING_PARSE_START_WITH_NUMBER_VERSION`: from 1.20.30 a name starting with a digit
/// is rejected.
const DIGIT_NAMES_FAIL: FormatVersion = FormatVersion::new(1, 20, 30);
/// The version that released the `unleash` and `execute_event_on_home_block` responses.
const RESPONSES_1_26_30: FormatVersion = known::V1_26_30;

/// Every family rule on `entity`.
pub(crate) fn validate(entity: &mut Entity, cx: &mut ValidateCx<'_>) {
    identifier(entity, cx);
    brain(entity, cx);
    spawn_egg_interaction(entity, cx);
    super::dynamic::check(entity, cx);
    validate_function(entity, cx);
    let groups = group_names(entity.component_groups.keys());
    let events_at = cx.at.child("events");
    for (name, event) in &mut entity.events {
        let at = events_at.child(name);
        if cx.original_version < ROOT_LEVEL_FILTERS
            && let Some(object) = event.node.as_object_mut()
            && object.remove("filters").is_some()
        {
            report(
                cx,
                // The gate's text is the crate's own, so none is attached; the game drops the
                // member without a content-log line, so the crate reports an information.
                DiagnosticCode::DroppedField { text: None },
                Severity::Info,
                at.child("filters"),
            );
        }
        node(&mut event.node, &at, &groups, cx);
    }
    let upgrades_at = cx.at.child("upgrades");
    for (name, handler) in &mut entity.upgrades {
        let at = upgrades_at.child(name);
        node(handler, &at, &groups, cx);
    }
    super::references::check(entity, cx);
    event_expressions(entity, cx);
    #[cfg(feature = "molang")]
    drop_failed_properties(entity, cx);
    #[cfg(feature = "molang")]
    apply_replaced_values(entity, cx);
}

/// A lenient `behavior.eat_block.success_chance` the compile pass rejected takes the member's
/// default (`MolangFailure::ReplaceWith`), as the game's lenient conversion does.
#[cfg(feature = "molang")]
fn apply_replaced_values(entity: &mut Entity, cx: &ValidateCx<'_>) {
    use bedrock_addon_core::molang::{MolangFailure, MolangOutcome};

    use super::goal::EatBlock;

    const KEY: &str = "minecraft:behavior.eat_block";
    let replaced: Vec<JsonPath> = cx
        .molang_failures
        .iter()
        .filter(|f| {
            matches!(
                f.outcome,
                MolangOutcome::Failed(MolangFailure::ReplaceWith(_))
            )
        })
        .map(|f| f.path.clone())
        .collect();
    if replaced.is_empty() {
        return;
    }
    let field = |section: &JsonPath| section.child(KEY).child("success_chance");
    if replaced.contains(&field(&cx.at.child("components")))
        && let Some(goal) = entity.components.get_mut::<EatBlock>()
    {
        goal.success_chance = None;
    }
    let groups_at = cx.at.child("component_groups");
    for (name, group) in &mut entity.component_groups {
        if replaced.contains(&field(&groups_at.child(name)))
            && let Some(goal) = group.components.get_mut::<EatBlock>()
        {
            goal.success_chance = None;
        }
    }
}

/// A finding at `at`.
fn report(cx: &mut ValidateCx<'_>, code: DiagnosticCode, severity: Severity, at: JsonPath) {
    cx.diagnostics.push(
        Diagnostic::error(code)
            .at(at)
            .phase(Phase::Validate)
            .severity(severity),
    );
}

/// `namespace:name`; a `minecraft:` identifier only for a vanilla entity (any in
/// a base-game pack); from 1.20.30 no name starting with a digit.
fn identifier(entity: &Entity, cx: &mut ValidateCx<'_>) {
    let at = cx.at.child("description").child("identifier");
    let text = entity.description.identifier.as_str();
    let Some((namespace, name)) = bedrock_addon_core::id::syntax::split_namespace(text) else {
        report(
            cx,
            DiagnosticCode::NamespaceRule { text: None },
            Severity::Error,
            at,
        );
        return;
    };
    if namespace.is_empty() || name.is_empty() {
        report(
            cx,
            DiagnosticCode::NamespaceRule { text: None },
            Severity::Error,
            at,
        );
        return;
    }
    if namespace == "minecraft"
        && !cx.load.is_base_game_pack
        && ActorType::from_identifier(text).is_none()
    {
        report(
            cx,
            DiagnosticCode::NamespaceRule { text: None },
            Severity::Error,
            at.clone(),
        );
    }
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        let severity = if cx.original_version >= DIGIT_NAMES_FAIL {
            Severity::Error
        } else {
            // Below 1.20.30 the game writes no content-log line and the entity loads, so the
            // crate reports an information.
            Severity::Info
        };
        report(
            cx,
            DiagnosticCode::InvalidValue { text: None },
            severity,
            at,
        );
    }
}

/// `brain` needs the Brain Prototype experiment (vanilla fails the document); memory
/// names are unique (a later duplicate is dropped); `beehive_has_nectar` names a `bool` memory.
/// `description.property_memory_mapping` needs the Brain Prototype
/// experiment; without it the member is `MissingToggles` and dropped, and the entity loads
/// (the game logs an error yet keeps loading, so the finding is a warning or an information).
fn property_memory_mapping(entity: &mut Entity, cx: &mut ValidateCx<'_>) {
    if entity.description.property_memory_mapping.is_none()
        || cx.load.experiments.contains(Experiment::BrainPrototype)
    {
        return;
    }
    entity.description.property_memory_mapping = None;
    let at = cx.at.child("description").child("property_memory_mapping");
    report(
        cx,
        DiagnosticCode::MissingToggles {
            text: Some(VanillaText::TurnOnToggle {
                what: "member",
                name: "property_memory_mapping".to_owned(),
                toggle: Experiment::BrainPrototype.save_name().unwrap_or_default(),
            }),
        },
        // The game has neither the toggle text's "member" form nor a line for
        // `property_memory_mapping`, so the crate reports an information.
        Severity::Info,
        at,
    );
}

fn brain(entity: &mut Entity, cx: &mut ValidateCx<'_>) {
    property_memory_mapping(entity, cx);
    let Some(brain) = &mut entity.brain else {
        return;
    };
    let at = cx.at.child("brain");
    if !cx.load.experiments.contains(Experiment::BrainPrototype) {
        report(
            cx,
            DiagnosticCode::MissingToggles {
                text: Some(VanillaText::TurnOnToggle {
                    what: "member",
                    name: "brain".to_owned(),
                    toggle: Experiment::BrainPrototype.save_name().unwrap_or_default(),
                }),
            },
            Severity::Error,
            at,
        );
        return;
    }
    let memories_at = at.child("memories");
    let mut seen = Vec::new();
    let mut index = 0;
    let mut dropped = Vec::new();
    let mut kept = Vec::with_capacity(brain.memories.len());
    // The game keys a memory by its name and its type (so an `entity_ref` and an `int` of one
    // name are two memories) and keeps the first of two equal ones without a word.
    brain.memories.retain(|memory| {
        let key = (memory.name.clone(), memory.kind);
        let keep = !seen.contains(&key);
        if keep {
            seen.push(key);
            kept.push(index);
        } else {
            dropped.push(index);
        }
        index += 1;
        keep
    });
    for index in dropped {
        let path = memories_at.index(index);
        // The later entry is dropped and the entity loads. The game constrains `memories` only
        // by a minimum size, with no uniqueness rule, and writes no line for a duplicate name,
        // so the crate reports an information.
        report(
            cx,
            DiagnosticCode::InvalidValue { text: None },
            Severity::Info,
            path,
        );
    }
    // What is told of a memory from here on is at the index the file has.
    cx.diagnostics.left_out(&memories_at, &kept);
    let mapping = brain
        .internal_memory_access_mappings
        .as_ref()
        .and_then(|m| m.beehive_has_nectar.clone());
    if let Some(name) = mapping {
        let is_bool = brain
            .memories
            .iter()
            .any(|m| m.name == name && m.kind == MemoryType::Bool);
        if !is_bool {
            let path = at
                .child("internal_memory_access_mappings")
                .child("beehive_has_nectar");
            report(
                cx,
                DiagnosticCode::UnresolvedReference { text: None },
                // The game writes no content-log line for this: an information.
                Severity::Info,
                path,
            );
        }
    }
}

/// `spawn_egg_interaction` needs `offspring` in `components` or a group of the
/// same definition; without it the game logs its line when the definition is initialised and
/// keeps the component, and so does the crate.
fn spawn_egg_interaction(entity: &Entity, cx: &mut ValidateCx<'_>) {
    const INTERACTION: &str = "minecraft:spawn_egg_interaction";
    const OFFSPRING: &str = "minecraft:offspring";
    let has = |map: &ComponentMap<EntityComponent>, key: &str| map.iter().any(|e| name(&e) == key);
    let offspring = has(&entity.components, OFFSPRING)
        || entity
            .component_groups
            .values()
            .any(|g| has(&g.components, OFFSPRING));
    if offspring {
        return;
    }
    let text = "\"minecraft:spawn_egg_interact\" requires \"minecraft:offspring\" to be present on the same entity definition.";
    let mut sections: Vec<(JsonPath, &ComponentMap<EntityComponent>)> =
        vec![(cx.at.child("components"), &entity.components)];
    let groups_at = cx.at.child("component_groups");
    for (group, value) in &entity.component_groups {
        sections.push((groups_at.child(group), &value.components));
    }
    // The game logs the text at the error level when it initialises the definition and
    // removes nothing: the definition stays. So this is a kept error at the component, and the
    // component is kept as the game keeps it.
    for (at, map) in sections {
        if has(map, INTERACTION) {
            cx.diagnostics.push_kept_error(
                Diagnostic::error(DiagnosticCode::MissingRequiredField {
                    text: Some(VanillaText::Fixed(text)),
                })
                .at(at.child(INTERACTION))
                .phase(Phase::Validate),
            );
        }
    }
}

/// The key of a component-map entry.
fn name<'a>(entry: &ComponentRef<'a, EntityComponent>) -> &'a str {
    match *entry {
        ComponentRef::Typed { name, .. }
        | ComponentRef::Dynamic { name, .. }
        | ComponentRef::Unknown { name, .. } => name,
        ComponentRef::Custom(custom) => &custom.name,
        _ => "",
    }
}

/// One event node, recursively: the group references, the filters and triggers
/// (numbered subjects and operators are written back as names), the response gates, and
/// any member the grammar does not name.
fn node(value: &mut Value, at: &JsonPath, groups: &HashSet<&str>, cx: &mut ValidateCx<'_>) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    if object.contains_key("randomize") && object.contains_key("sequence") {
        report(
            cx,
            DiagnosticCode::IgnoredMember {
                text: Some(VanillaText::Fixed(
                    bedrock_addon_core::diagnostic::EVENT_BOTH_RANDOMIZE_AND_SEQUENCE,
                )),
            },
            // The game logs the item loader's string for an event node with both members at the
            // warning level.
            Severity::Warning,
            at.child("sequence"),
        );
    }
    // A node with a collection returns before the response validation; the others'
    // members left after `weight`, `filters`, `trigger`, `add` and `remove` are the response
    // collection's, validated strictly at every version.
    let collection = ["randomize", "sequence", "first_valid"]
        .iter()
        .any(|key| object.contains_key(key));
    let keys: Vec<String> = object.keys().map(|k| k.as_str().to_owned()).collect();
    // Whether the response collection failed.
    let mut failed = false;
    for key in &keys {
        let key = key.clone();
        let member_at = at.child(&key);
        let Some(member) = object.get_mut(&key) else {
            continue;
        };
        if !collection {
            failed |= response_schema(&key, member, &member_at, cx);
        }
        match key.as_str() {
            "add" | "remove" => group_references(member, &member_at, groups, cx),
            "sequence" | "first_valid" | "randomize" => {
                if let Some(nodes) = member.as_array_mut() {
                    for (index, child_node) in nodes.as_mut_slice().iter_mut().enumerate() {
                        let child_at = member_at.index(index);
                        node(child_node, &child_at, groups, cx);
                    }
                }
            }
            "filters" => filters(member, &member_at, cx),
            "trigger" => trigger(member, &member_at, cx),
            // A `randomize` entry's weight (`weight: number`); the parent
            // is not checked for it. Another JSON type is `InvalidType`.
            "weight" => {
                if !member.is_number() {
                    report(
                        cx,
                        DiagnosticCode::InvalidType { text: None },
                        // The game writes no content-log line for this: an information.
                        Severity::Info,
                        member_at,
                    );
                }
            }
            "set_property" | "queue_command" | "emit_vibration" | "reset_target"
            | "set_home_position" | "play_sound" | "emit_particle" | "stop_movement"
            | "drop_item" => {}
            "execute_event_on_home_block" => {
                // Experimental until 1.26.30; a base-game pack uses it
                // without the toggle.
                if cx.original_version < RESPONSES_1_26_30
                    && !cx.load.upcoming_creator_features
                    && !cx.load.is_base_game_pack
                {
                    gated(cx, &key, Experiment::UpcomingCreatorFeatures, member_at);
                }
            }
            "run_command" => {
                if !LoadRequirement::LegacyCreatorFeatures.satisfied(cx.load) {
                    report(
                        cx,
                        DiagnosticCode::MissingToggles { text: None },
                        // The game writes no content-log line for this: an information.
                        Severity::Info,
                        member_at,
                    );
                }
            }
            "set_memory" => {
                if !cx.load.experiments.contains(Experiment::BrainPrototype) {
                    gated(cx, &key, Experiment::BrainPrototype, member_at);
                }
            }
            "unleash" if cx.original_version >= RESPONSES_1_26_30 => {}
            // Beside a collection the loader passes the member over without a line (an
            // information of the crate); otherwise no child of the response collection names
            // it: the game logs a line at the error level, the event loses its responses and
            // the entity still loads (a kept error).
            _ if collection => report(
                cx,
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Info,
                member_at,
            ),
            _ => {
                failed = true;
                unknown_response(&key, member_at, cx);
            }
        }
    }
    if failed {
        drop_responses(object, &keys, at, cx);
    }
}

/// No child of the response collection names `key`: the game logs a line at the error level and
/// the entity still loads, so it is a kept error.
fn unknown_response(key: &str, at: JsonPath, cx: &mut ValidateCx<'_>) {
    cx.diagnostics.push_kept_error(
        Diagnostic::error(DiagnosticCode::UnrecognizedField {
            text: Some(VanillaText::Formatted {
                template: "child '{}' not valid here.",
                args: vec![key.to_owned()],
            }),
        })
        .at(at)
        .phase(Phase::Validate),
    );
}

/// A failed collection parses no response: the node's responses go, each with the crate's
/// information (the game writes no line for the dropped ones); what the loader read before the
/// collection stays.
fn drop_responses(
    object: &mut facet_value::VObject,
    keys: &[String],
    at: &JsonPath,
    cx: &mut ValidateCx<'_>,
) {
    for key in keys {
        if BEFORE_RESPONSES.contains(&key.as_str()) {
            continue;
        }
        if object.remove(key).is_some() {
            report(
                cx,
                DiagnosticCode::DroppedField { text: None },
                Severity::Info,
                at.child(key),
            );
        }
    }
}

/// The members of an event node the game reads before the response collection: they stay when
/// the collection fails.
const BEFORE_RESPONSES: [&str; 5] = ["weight", "filters", "trigger", "add", "remove"];

/// The members of the response `key` (an object) as its schema validates them, the game's lines
/// as kept errors. Returns whether the validation failed.
fn response_schema(key: &str, member: &Value, at: &JsonPath, cx: &mut ValidateCx<'_>) -> bool {
    if let Some(response) = game_schema::event_response(key)
        && member.is_object()
    {
        let outcome = game_schema::validate(response.members, member, true);
        // The collection's information at the node comes from `drop_responses`, not the
        // component's `ValueRewritten`: only the game's lines are told here.
        for problem_finding in outcome.findings(at, Phase::Validate) {
            if problem_finding.kept {
                cx.diagnostics.push(problem_finding);
            }
        }
        return outcome.failed;
    }
    false
}

/// A response its experiment gates, kept (the game ignores it without the toggle).
fn gated(cx: &mut ValidateCx<'_>, key: &str, experiment: Experiment, at: JsonPath) {
    report(
        cx,
        DiagnosticCode::MissingToggles {
            text: Some(VanillaText::TurnOnToggle {
                what: "event response",
                name: key.to_owned(),
                toggle: experiment.save_name().unwrap_or_default(),
            }),
        },
        // The game has no toggle text of the "event response" form (only the item's and the
        // block's), so the crate reports an information.
        Severity::Info,
        at,
    );
}

/// The names of the component groups, as the event walk asks for them.
fn group_names<'a>(names: impl Iterator<Item = &'a String>) -> HashSet<&'a str> {
    names.map(String::as_str).collect()
}

/// `add` / `remove`: each named group must be a key of `component_groups`; a miss is reported
/// and kept.
fn group_references(value: &Value, at: &JsonPath, groups: &HashSet<&str>, cx: &mut ValidateCx<'_>) {
    let Some(names) = value
        .as_object()
        .and_then(|o| o.get("component_groups"))
        .and_then(Value::as_array)
    else {
        return;
    };
    let list_at = at.child("component_groups");
    for (index, name) in names.iter().enumerate() {
        let Some(name) = name.as_string() else {
            continue;
        };
        if !groups.contains(name.as_str()) {
            let path = list_at.index(index);
            // The game stores the names and says nothing at load: the crate reports an
            // information.
            report(
                cx,
                DiagnosticCode::UnresolvedReference { text: None },
                Severity::Info,
                path,
            );
        }
    }
}

/// A `filters` member: its findings, and written back in the reader's form.
fn filters(value: &mut Value, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    match Filter::try_from(value.clone()) {
        Ok(filter) => {
            let found = filter.check(at);
            cx.diagnostics.extend(placed(found));
            if let Ok(normal) = Value::try_from(&filter) {
                *value = normal;
            }
        }
        Err(_) => report(
            cx,
            DiagnosticCode::InvalidValue { text: None },
            // The game writes no content-log line for this: an information.
            Severity::Info,
            at.clone(),
        ),
    }
}

/// A `trigger` member: its filter's findings, and written back in the reader's form.
fn trigger(value: &mut Value, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    match EventTrigger::try_from(value.clone()) {
        Ok(trigger) => {
            let found = trigger.check(at);
            cx.diagnostics.extend(placed(found));
            if let Ok(normal) = Value::try_from(&trigger) {
                *value = normal;
            }
        }
        Err(_) => report(
            cx,
            DiagnosticCode::InvalidValue { text: None },
            // The game writes no content-log line for this: an information.
            Severity::Info,
            at.clone(),
        ),
    }
}

/// Shared-type findings as validation findings.
fn placed(found: Diagnostics) -> Diagnostics {
    found
        .into_iter()
        .map(|mut d| {
            d.phase = Phase::Validate;
            d
        })
        .collect()
}

/// rule M2: a property default that does not compile drops the entity's property group.
#[cfg(feature = "molang")]
fn drop_failed_properties(entity: &mut Entity, cx: &mut ValidateCx<'_>) {
    use bedrock_addon_core::molang::{MolangFailure, MolangOutcome};
    let properties_at = cx.at.child("description").child("properties");
    let failed = cx.molang_failures.iter().any(|failure| {
        failure.outcome == MolangOutcome::Failed(MolangFailure::DropEntry)
            && failure.path.starts_with(&properties_at)
    });
    if failed {
        entity.description.properties = IndexMap::new();
        // The game logs this at the error level and the entity loads without the group: a
        // kept error.
        cx.diagnostics.push_kept_error(
            Diagnostic::error(DiagnosticCode::DroppedField {
                text: Some(VanillaText::Fixed("Error loading Actor Properties")),
            })
            .at(properties_at)
            .phase(Phase::Validate),
        );
    }
}

/// `events.*.set_property.<property>` (and the same in `upgrades` handlers): a value that is not
/// an enum entry of the property is Molang, read schema-validated at the pack's version (rule
/// M1); a rejected expression drops the `set_property` response (the extent of the drop is the
/// crate's own).
// The game logs the error and loads the entity without the response. The parser's lines
// are logged at the error level: kept errors.
const SET_PROPERTY: MolangFieldSpec = MolangFieldSpec::KEPT.reader(ReaderKind::SchemaValidated);

/// One step of a path into the payload.
#[derive(Clone, Debug)]
enum Seg {
    Key(String),
    Index(usize),
}

/// One `set_property` expression of an event node.
struct Expression {
    /// The member's path below the payload.
    path: Vec<Seg>,
    /// Which node holds it: `events` or `upgrades`, the entry's name, the steps below it.
    #[cfg_attr(not(feature = "molang"), allow(dead_code))]
    node: NodeRef,
    /// The expression.
    value: Molang,
}

/// The event expressions, walked as Molang fields.
struct Expressions(Vec<Expression>);

impl MolangFields for Expressions {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        for expression in &mut self.0 {
            let Some((last, parents)) = expression.path.split_last() else {
                continue;
            };
            for seg in parents {
                visitor.enter(match seg {
                    Seg::Key(key) => Step::Key(key),
                    Seg::Index(index) => Step::Index(*index),
                });
            }
            match last {
                Seg::Key(key) => visitor.field(key.as_str(), &SET_PROPERTY, &mut expression.value),
                Seg::Index(index) => visitor.field(*index, &SET_PROPERTY, &mut expression.value),
            }
            for _ in parents {
                visitor.leave();
            }
        }
    }
}

/// The path segments as a `JsonPath` below `base`.
fn json_path(base: &JsonPath, segs: &[Seg]) -> JsonPath {
    let mut path = base.clone();
    for seg in segs {
        match seg {
            Seg::Key(key) => path.push(key),
            Seg::Index(index) => path.push_index(*index),
        }
    }
    path
}

/// The node `steps` names below `value`.
fn node_at<'a>(value: &'a mut Value, steps: &[Seg]) -> Option<&'a mut Value> {
    let mut current = value;
    for seg in steps {
        current = match seg {
            Seg::Key(key) => current.as_object_mut()?.get_mut(key)?,
            Seg::Index(index) => current.as_array_mut()?.get_mut(*index)?,
        };
    }
    Some(current)
}

/// Collects the `set_property` values of a node and its children that are Molang: neither a
/// scalar nor an enum entry of the property.
fn collect(
    value: &Value,
    steps: &mut Vec<Seg>,
    enums: &IndexMap<String, Vec<String>>,
    out: &mut Vec<(Vec<Seg>, String, Value)>,
) {
    let Some(object) = value.as_object() else {
        return;
    };
    if let Some(properties) = object.get("set_property").and_then(Value::as_object) {
        for (name, value) in properties {
            let is_enum_value = value.as_string().is_some_and(|s| {
                enums
                    .get(name.as_str())
                    .is_some_and(|values| values.iter().any(|v| v == s.as_str()))
            });
            if !(is_enum_value || value.is_bool() || value.is_number()) {
                out.push((steps.clone(), name.as_str().to_owned(), value.clone()));
            }
        }
    }
    for key in ["sequence", "first_valid", "randomize"] {
        if let Some(nodes) = object.get(key).and_then(Value::as_array) {
            for (index, child_node) in nodes.iter().enumerate() {
                steps.push(Seg::Key(key.to_owned()));
                steps.push(Seg::Index(index));
                collect(child_node, steps, enums, out);
                steps.pop();
                steps.pop();
            }
        }
    }
}

/// A node whose `set_property` response is dropped: `events` or `upgrades`, the entry's name,
/// the steps below it.
type NodeRef = (bool, String, Vec<Seg>);

/// The `set_property` expressions of every event node and upgrade handler; a value of another
/// JSON type is `InvalidType` and its node is listed for the drop.
fn gather(entity: &Entity, cx: &mut ValidateCx<'_>, drop: &mut Vec<NodeRef>) -> Expressions {
    let enums: IndexMap<String, Vec<String>> = entity
        .description
        .properties
        .iter()
        .filter(|(_, p)| p.kind == PropertyType::Enum)
        .map(|(name, p)| (name.clone(), p.values.clone()))
        .collect();
    let roots = entity
        .events
        .iter()
        .map(|(name, event)| (true, name, &event.node))
        .chain(
            entity
                .upgrades
                .iter()
                .map(|(name, node)| (false, name, node)),
        );
    let mut expressions = Vec::new();
    for (in_events, name, node) in roots {
        let mut values = Vec::new();
        collect(node, &mut Vec::new(), &enums, &mut values);
        for (steps, property, value) in values {
            let mut path = vec![
                Seg::Key(if in_events { "events" } else { "upgrades" }.to_owned()),
                Seg::Key(name.clone()),
            ];
            path.extend(steps.iter().cloned());
            path.push(Seg::Key("set_property".to_owned()));
            path.push(Seg::Key(property));
            let molang = value.as_string().and_then(|s| {
                bedrock_addon_core::json::from_str::<Molang>(&json_string(s.as_str())).ok()
            });
            if let Some(molang) = molang {
                expressions.push(Expression {
                    path,
                    node: (in_events, name.clone(), steps),
                    value: molang,
                });
            } else {
                // An event's object or list is the response schema's type line (told by
                // `node`); the crate tells the rest (an upgrade handler's, any other value).
                if !(in_events && (value.is_object() || value.is_array())) {
                    report(
                        cx,
                        DiagnosticCode::InvalidType { text: None },
                        // The game writes no content-log line for this: an information.
                        Severity::Info,
                        json_path(&cx.at, &path),
                    );
                }
                drop.push((in_events, name.clone(), steps));
            }
        }
    }
    Expressions(expressions)
}

/// rule M1 on the event nodes: each `set_property` expression gets the pack's Molang
/// version and, with the `molang` feature and a check in the load context, is compiled; an
/// object or another non-scalar value is `InvalidType`. Either failure drops the response.
fn event_expressions(entity: &mut Entity, cx: &mut ValidateCx<'_>) {
    let mut drop: Vec<NodeRef> = Vec::new();
    let mut expressions = gather(entity, cx, &mut drop);
    if !expressions.0.is_empty() {
        let written = FormatVersionInput {
            version: Some(cx.original_version),
            beta_literal: false,
            prerelease: false,
        };
        let molang = MolangContext::new(
            cx.load.min_engine_version.as_ref(),
            cx.load.molang_experiments,
            &written,
            Some(cx.load.molang_version),
        );
        let base = Location::new(None, None, cx.at.clone(), None);
        pass_one(&mut expressions, &molang, &base, cx.diagnostics);
        compile(&mut expressions, &molang, &base, cx, &mut drop);
    }
    for (in_events, name, steps) in drop {
        let root = if in_events {
            entity.events.get_mut(&name).map(|event| &mut event.node)
        } else {
            entity.upgrades.get_mut(&name)
        };
        if let Some(object) = root
            .and_then(|root| node_at(root, &steps))
            .and_then(Value::as_object_mut)
        {
            object.remove("set_property");
        }
    }
}

/// Pass 2 of the event expressions: compiled with the load context's check, else counted as
/// not checked. A rejected expression lists its node for the drop.
/// Without the `molang` feature every expression is counted as not checked.
fn compile(
    expressions: &mut Expressions,
    molang: &MolangContext,
    base: &Location,
    cx: &mut ValidateCx<'_>,
    drop: &mut Vec<NodeRef>,
) {
    #[cfg(feature = "molang")]
    if let Some(check) = cx.load.molang.clone() {
        use bedrock_addon_core::molang::{MolangOutcome, Side, check_fields};
        let failures = check_fields(
            expressions,
            &check,
            Side::Server,
            molang,
            base,
            cx.diagnostics,
        );
        for failure in failures {
            if !matches!(failure.outcome, MolangOutcome::Failed(_)) {
                continue;
            }
            if let Some(found) = expressions
                .0
                .iter()
                .find(|e| json_path(&cx.at, &e.path) == failure.path)
            {
                drop.push(found.node.clone());
            }
        }
        return;
    }
    #[cfg(not(feature = "molang"))]
    let _ = (molang, drop);
    not_checked(expressions, base, cx.diagnostics);
}
/// `text` as a JSON string literal.
fn json_string(text: &str) -> String {
    bedrock_addon_core::json::to_json(&Value::from(text))
        .unwrap_or_default()
        .trim_end()
        .to_owned()
}

/// The game's text for a timer that has both `time` and `random_time_choices`.
const TIMER_BOTH: &str = "Both \"time\" and \"random_time_choices\" have been specified for a timer. Defaulting to \"time\".";
/// The game's text for a preferred path with a block in two costing groups, with the block's name.
const DUPLICATE_PATH_BLOCK: &str = "Duplicate block descriptors in two costing groups {}";

/// The game's component validation of the final document, `components` and then each group:
/// the preferred path's duplicate blocks (an error) and the timer's two times (a warning).
fn validate_function(entity: &Entity, cx: &mut ValidateCx<'_>) {
    let groups_at = cx.at.child("component_groups");
    let mut sections = vec![(cx.at.child("components"), &entity.components)];
    for (name, group) in &entity.component_groups {
        sections.push((groups_at.child(name), &group.components));
    }
    for (at, map) in sections {
        preferred_path(map, &at, cx);
        timer(map, &at, cx);
    }
}

/// The timer check: `time` given and `random_time_choices` not empty.
fn timer(map: &ComponentMap<EntityComponent>, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    const KEY: &str = "minecraft:timer";
    let Some(timer) = map.get::<super::component::Timer>() else {
        return;
    };
    if timer.time.is_some() && !timer.random_time_choices.is_empty() {
        report(
            cx,
            DiagnosticCode::IgnoredMember {
                text: Some(VanillaText::Fixed(TIMER_BOTH)),
            },
            // The game logs this at the warning level.
            Severity::Warning,
            at.child(KEY),
        );
    }
}

/// The preferred-path check: a block descriptor (compared by name, states and tags, whatever
/// form it is written in) in two cost groups; told once per such block, at the later group.
fn preferred_path(map: &ComponentMap<EntityComponent>, at: &JsonPath, cx: &mut ValidateCx<'_>) {
    const KEY: &str = "minecraft:preferred_path";
    let Some(path) = map.get::<super::component::PreferredPath>() else {
        return;
    };
    let sets = &path.preferred_path_blocks;
    let mut told: Vec<String> = Vec::new();
    for (j, later) in sets.iter().enumerate() {
        for block in &later.blocks {
            let earlier = sets[..j]
                .iter()
                .any(|set| set.blocks.iter().any(|other| same_block(other, block)));
            let name = block_name(block);
            if earlier && !told.contains(&name) {
                let mut place = at.child(KEY).child("preferred_path_blocks");
                place.push_index(j);
                report(
                    cx,
                    DiagnosticCode::InvalidValue {
                        text: Some(VanillaText::Formatted {
                            template: DUPLICATE_PATH_BLOCK,
                            args: vec![name.clone()],
                        }),
                    },
                    Severity::Error,
                    place,
                );
                told.push(name);
            }
        }
    }
}

/// Whether two descriptors name the same block: the name, the states and the tags, not the
/// form they are written in.
fn same_block(a: &BlockDescriptor, b: &BlockDescriptor) -> bool {
    use BlockDescriptor as B;
    match (a, b) {
        (
            B::Block {
                name: a_name,
                states: a_states,
                ..
            },
            B::Block {
                name: b_name,
                states: b_states,
                ..
            },
        ) => a_name == b_name && a_states == b_states,
        (B::Tags { tags: a, .. }, B::Tags { tags: b, .. }) => a == b,
        _ => a == b,
    }
}

/// The block's name as the text names it (its tags for a tag descriptor).
fn block_name(block: &BlockDescriptor) -> String {
    match block {
        BlockDescriptor::Block { name, .. } => name.as_str().to_owned(),
        other => bedrock_addon_core::json::to_json(other).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `n` component groups, and one event response that adds each of them and one more
    /// that no group is (a miss looks at every name).
    fn groups_and_an_event_naming_each(n: usize) -> (Vec<String>, Value) {
        let groups: Vec<String> = (0..n).map(|i| format!("demo:g_{i}")).collect();
        let named: Vec<String> = (0..=n).map(|i| format!(r#""demo:g_{i}""#)).collect();
        let text = format!(
            r#"{{"add": {{"component_groups": [{}]}}}}"#,
            named.join(",")
        );
        let event = bedrock_addon_core::json::from_str::<Value>(&text).expect("json");
        (groups, event)
    }

    /// The findings of the event walk on `event` beside `groups`.
    fn walked(groups: &[String], event: &Value) -> Diagnostics {
        use bedrock_addon_core::load::LoadContext;
        let load = LoadContext::detached();
        let mut diagnostics = Diagnostics::default();
        let mut cx = ValidateCx {
            load: &load,
            original_version: FormatVersion::new(1, 26, 50),
            at: JsonPath::new(),
            skipped: Vec::new(),
            #[cfg(feature = "molang")]
            molang_failures: Vec::new(),
            diagnostics: &mut diagnostics,
        };
        let mut at = JsonPath::new();
        at.push("events");
        at.push("demo:e");
        let groups = group_names(groups.iter());
        node(&mut event.clone(), &at, &groups, &mut cx);
        diagnostics
    }

    #[test]
    fn a_group_an_event_names_that_is_no_group_is_told_at_its_element() {
        let (groups, event) = groups_and_an_event_naming_each(3);
        let found = walked(&groups, &event);
        assert_eq!(found.len(), 1, "{found:#?}");
        let finding = found.iter().next().expect("one finding");
        assert!(matches!(
            finding.code,
            DiagnosticCode::UnresolvedReference { text: None }
        ));
        assert_eq!(finding.severity, Severity::Info);
        assert_eq!(
            finding.location.json_path.to_string(),
            "events/demo:e/add/component_groups/3"
        );
    }

    /// Each name of an `add` or `remove` list was looked for from the first group.
    #[test]
    fn n_group_references_among_n_groups_cost_n() {
        bedrock_addon_core::test_cost::assert_linear_at(
            "entity events: n group references among n groups",
            40_000,
            groups_and_an_event_naming_each,
            |(groups, event)| {
                let found = walked(groups, event);
                assert_eq!(found.len(), 1);
                found
            },
        );
    }
}
