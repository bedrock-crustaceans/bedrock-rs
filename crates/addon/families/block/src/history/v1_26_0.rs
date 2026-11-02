//! Format 1.26.0: `liquid_detection.detection_rules[].use_liquid_clipping`. Its step is the
//! conversion to the 1.26.20 document.

use indexmap::IndexMap;

use super::legacy::{Legacy, component_path, fixed, string, text, to_object, upgrade_finding};
use super::ops::{dropped_field, geometry_identifier, is_opaque};
use super::v1_20_10::FULL_BLOCK;
use crate::component::BlockComponent;
use crate::parse::{child, parse_document};
use bedrock_addon_core::diagnostic::{
    DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::ComponentRef;
use bedrock_addon_core::id::BlockId;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadRequirement;
use bedrock_addon_core::molang::{
    Admission, Molang, MolangFieldMut, MolangVersion, RawVersion, visit_fields,
};
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};
use bedrock_addon_core::versioning::{ParseCx, Upgrade, UpgradeCx};

/// A block document of `[1.26.0, 1.26.20)`.
pub type Block = Legacy<1, 26, 0>;

/// The seven legacy trigger components.
const TRIGGERS: [&str; 7] = [
    "on_fall_on",
    "on_interact",
    "on_placed",
    "on_player_destroyed",
    "on_player_placing",
    "on_step_off",
    "on_step_on",
];

/// The legacy components the conversion moves into the side-car (allow-listed packs only).
const LEGACY: [&str; 4] = [
    "unit_cube",
    "queued_ticking",
    "random_ticking",
    "breathability",
];

/// Below this original version a chest obstruction is synthesised.
const CHEST_OBSTRUCTION_BEFORE: FormatVersion = FormatVersion::new(1, 26, 1);
/// From this original version `unit_cube` is deprecated.
const UNIT_CUBE_DEPRECATED: FormatVersion = FormatVersion::new(1, 20, 60);
/// From this original version the v1 `custom_components` list is deprecated.
const CUSTOM_COMPONENTS_DEPRECATED: FormatVersion = FormatVersion::new(1, 21, 90);
/// The `states` / `permutations` gates.
const STATES_FROM: FormatVersion = FormatVersion::new(1, 19, 60);
const STATES_STRICT_FROM: FormatVersion = FormatVersion::new(1, 19, 70);

/// The conversion of a legacy document (the game's upgrader plus its legacy-compatibility
/// pass), with the gates the legacy loader applies on the way: the result is parsed as the 1.26.20 document.
impl Upgrade for Block {
    type Next = super::v1_26_20::Block;

    fn upgrade(mut self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let lcf = LoadRequirement::LegacyCreatorFeatures.satisfied(cx.load);
        let original = cx.original_version;
        drop_crate_members(&mut self, cx.diagnostics);
        if !lcf {
            gates(&mut self, original, cx.diagnostics);
        }
        for (section, map) in self.sections_mut() {
            convert_tags(map, section, cx.diagnostics);
        }
        // The chest obstruction reads `unit_cube` before its deprecation removes it (fixture
        // s13c: a 1.26.0 opaque unit cube is `always`).
        if original < CHEST_OBSTRUCTION_BEFORE {
            synthesize_chest_obstruction(&mut self);
        }
        deprecations(&mut self, original, cx.diagnostics);
        let mut skipped = Vec::new();
        let mut sidecars: Vec<IndexMap<String, Value>> = Vec::new();
        for (section, map) in self.sections_mut() {
            if let Some(sidecar) = side_car(map, section, lcf, cx.diagnostics) {
                sidecars.push(sidecar);
            } else {
                skipped.push(section);
                sidecars.push(IndexMap::new());
            }
        }
        drop_skipped(&mut self, &mut sidecars, &skipped, cx.diagnostics);
        conditions(&self, cx.diagnostics);
        lower_category(&mut self);
        // `events` that is no object holds no event; the crate tells it at the member.
        if self
            .events
            .as_ref()
            .is_some_and(|events| !events.is_object())
        {
            super::legacy::misread(
                &child(&JsonPath::new(), &["minecraft:block", "events"]),
                cx.diagnostics,
            );
        }
        let mut handlers = IndexMap::new();
        if lcf && let Some(events) = self.events.take().as_ref().and_then(super::legacy::object) {
            for (name, node) in events {
                if !strings_where_expected(&node) {
                    // The game's event reader rejects a node that holds a non-string value.
                    cx.diagnostics.push(upgrade_finding(
                        DiagnosticCode::InvalidType {
                            text: fixed("Invalid, nonstring Json "),
                        },
                        Severity::Error,
                        &child(&JsonPath::new(), &["minecraft:block", "events", &name]),
                    ));
                }
                // What the side-car's reader tells of the handler is told of the event the
                // file has.
                let from = child(&JsonPath::new(), &["minecraft:block", "events", &name]);
                let to = child(
                    &JsonPath::new(),
                    &["minecraft:block", "deprecated_event_handlers", &name],
                );
                cx.diagnostics.moved(from.clone(), to.clone());
                let handler = event_node(&node, &from, &to, cx.diagnostics);
                handlers.insert(name, handler);
            }
        }
        let payload = payload(&self, &sidecars, &handlers);
        convert(&payload, &self, cx)
    }
}

/// `deprecated` and `deprecated_event_handlers` are members this library writes for the
/// legacy components and events of an allow-listed block (the crate's own; no format of the game has
/// them). A member of that name in a legacy FILE is not what the conversion makes: it is
/// taken out, so that it is not read in the place of the file's own components and events,
/// and told as the unknown member it is.
fn drop_crate_members(block: &mut Block, d: &mut Diagnostics) {
    let payload = child(&JsonPath::new(), &["minecraft:block"]);
    for key in ["deprecated", "deprecated_event_handlers"] {
        if block.extra.remove(key).is_some() {
            d.push(upgrade_finding(
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Info,
                &child(&payload, &[key]),
            ));
        }
    }
    for (index, permutation) in block.permutations.iter_mut().enumerate() {
        if permutation.extra.remove("deprecated").is_some() {
            let mut at = child(&payload, &["permutations"]);
            at.push_index(index);
            at.push("deprecated");
            d.push(upgrade_finding(
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Info,
                &at,
            ));
        }
    }
}

/// Removes the permutations a failed legacy component skipped, with their side-cars, and
/// renumbers the synthesised entries. The permutations left are known by the index the
/// file has them at: what is told of one from here on (the reading of the converted block,
/// the checks, the link pass) is told there.
fn drop_skipped(
    block: &mut Block,
    sidecars: &mut Vec<IndexMap<String, Value>>,
    skipped: &[Option<usize>],
    d: &mut Diagnostics,
) {
    let skipped: Vec<usize> = skipped.iter().filter_map(|s| *s).collect();
    if skipped.is_empty() {
        return;
    }
    let kept: Vec<usize> = (0..block.permutations.len())
        .filter(|index| !skipped.contains(index))
        .collect();
    d.left_out(
        &child(&JsonPath::new(), &["minecraft:block", "permutations"]),
        &kept,
    );
    let mut index = 0;
    block.permutations.retain(|_| {
        index += 1;
        !skipped.contains(&(index - 1))
    });
    let mut index = 0;
    sidecars.retain(|_| {
        index += 1;
        index == 1 || !skipped.contains(&(index - 2))
    });
    block.synthesized.retain_mut(|(section, _)| match section {
        None => true,
        Some(i) if skipped.contains(i) => false,
        Some(i) => {
            *i -= skipped.iter().filter(|s| **s < *i).count();
            true
        }
    });
}

/// The gates without the legacy creator features: states before 1.19.60 dropped, states in
/// `[1.19.60, 1.19.70)` reject the block, permutations at 1.19.60 skipped, legacy events
/// cleared, legacy triggers unrecognized.
fn gates(block: &mut Block, original: FormatVersion, d: &mut Diagnostics) {
    let description = |key: &str| child(&JsonPath::new(), &["minecraft:block", "description", key]);
    let has_states = block
        .description
        .get("states")
        .is_some_and(|s| s.as_object().is_none_or(|o| !o.is_empty()));
    if original < STATES_FROM {
        if block.description.shift_remove("states").is_some() {
            dropped_field(&description("states"), None, d);
        }
    } else if original < STATES_STRICT_FROM && has_states {
        d.push(upgrade_finding(
            DiagnosticCode::MissingToggles {
                text: fixed("To use custom block properties, use format version 1.19.70 or higher"),
            },
            Severity::Error,
            &description("states"),
        ));
    }
    if original == STATES_FROM && !block.permutations.is_empty() {
        block.permutations.clear();
        // The game logs this at the content log's error level and goes on with the
        // permutations skipped: a kept error.
        d.push_kept_error(upgrade_finding(
            DiagnosticCode::MissingToggles {
                text: fixed(
                    "To use custom block permutations, use format verison 1.19.70 or higher.",
                ),
            },
            Severity::Error,
            &child(&JsonPath::new(), &["minecraft:block", "permutations"]),
        ));
    }
    if block.events.take().is_some() {
        dropped_field(
            &child(&JsonPath::new(), &["minecraft:block", "events"]),
            None,
            d,
        );
    }
    let mut unrecognized = Vec::new();
    for (section, map) in block.sections_mut() {
        for trigger in TRIGGERS {
            let name = format!("minecraft:{trigger}");
            if map.shift_remove(&name).is_some() {
                unrecognized.push((section, name));
            }
        }
    }
    for (section, name) in unrecognized {
        d.push(upgrade_finding(
            DiagnosticCode::UnrecognizedComponent { text: None },
            Severity::Info,
            &component_path(section, &name),
        ));
    }
}

/// `unit_cube` from 1.20.60 and the v1 `custom_components` list from 1.21.90: "{} component is
/// not valid from {} onward.", removed (no side-car).
fn deprecations(block: &mut Block, original: FormatVersion, d: &mut Diagnostics) {
    let rules = [
        ("minecraft:unit_cube", UNIT_CUBE_DEPRECATED),
        ("minecraft:custom_components", CUSTOM_COMPONENTS_DEPRECATED),
    ];
    let mut removed = Vec::new();
    for (section, map) in block.sections_mut() {
        for (name, since) in rules {
            if original >= since && map.shift_remove(name).is_some() {
                removed.push((section, name, since));
            }
        }
    }
    // The game removes the component and logs this at the content log's error level; the
    // block is kept: a kept error.
    for (section, name, since) in removed {
        d.push_kept_error(upgrade_finding(
            DiagnosticCode::DeprecatedMember {
                text: text(
                    "{} component is not valid from {} onward.",
                    &[name, &since.to_string()],
                ),
            },
            Severity::Error,
            &component_path(section, name),
        ));
    }
}

/// `chest_obstruction` for each section without one: `always` for an opaque full unit cube,
/// `never` otherwise; a permutation falls back to the base (TOML `callback
/// synthesize_chest_obstruction`).
pub(crate) fn synthesize_chest_obstruction(block: &mut Block) {
    const NAME: &str = "minecraft:chest_obstruction";
    let base = block.components.clone();
    let rule = |map: &IndexMap<String, Value>| {
        let get = |key: &str| map.get(key).or_else(|| base.get(key));
        let cube = get("minecraft:unit_cube").is_some();
        let full = get("minecraft:geometry").and_then(geometry_identifier) == Some(FULL_BLOCK);
        let opaque = is_opaque(get("minecraft:material_instances"));
        let mut object = facet_value::VObject::new();
        object.insert(
            "obstruction_rule",
            if cube && opaque && full {
                "always"
            } else {
                "never"
            },
        );
        Value::from(object)
    };
    let mut inserted = Vec::new();
    for (section, map) in block.sections_mut() {
        if !map.contains_key(NAME) {
            let value = rule(map);
            map.insert(NAME.to_owned(), value);
            inserted.push((section, NAME.to_owned()));
        }
    }
    block.synthesized.extend(inserted);
}

/// Moves the section's legacy components and triggers into its side-car. Without the legacy
/// creator features a legacy component rejects the block in the base and skips a permutation
/// ("Legacy component '%s' is only available to allow-listed packs", the crate's own text); `None` then.
fn side_car(
    map: &mut IndexMap<String, Value>,
    section: Option<usize>,
    lcf: bool,
    d: &mut Diagnostics,
) -> Option<IndexMap<String, Value>> {
    let mut sidecar = IndexMap::new();
    for name in LEGACY.iter().chain(TRIGGERS.iter()) {
        let key = format!("minecraft:{name}");
        let Some(value) = map.shift_remove(&key) else {
            continue;
        };
        let at = component_path(section, &key);
        if !lcf {
            d.push(upgrade_finding(
                DiagnosticCode::MissingToggles { text: None },
                if section.is_none() {
                    Severity::Error
                } else {
                    Severity::Info
                },
                &at,
            ));
            return None;
        }
        d.push(upgrade_finding(
            DiagnosticCode::DeprecatedMember {
                text: Some(VanillaText::Deprecated {
                    member: key.clone(),
                    version: FormatVersion::new(1, 26, 20),
                }),
            },
            Severity::Info,
            &at,
        ));
        let converted = match *name {
            "unit_cube" => Value::TRUE,
            "queued_ticking" => queued_ticking(&value, &at, d),
            _ => value,
        };
        sidecar.insert((*name).to_owned(), converted);
        // What is told of the side-car's member is told of the component the file has.
        let mut to = JsonPath::new();
        to.push("minecraft:block");
        if let Some(index) = section {
            to.push("permutations");
            to.push_index(index);
        }
        to.push("deprecated");
        to.push(name);
        d.moved(at.clone(), to.clone());
        if *name == "queued_ticking" {
            // The two tick counts are the two ends of the file's `interval_range`.
            for (index, member) in ["min_ticks", "max_ticks"].into_iter().enumerate() {
                let mut end = child(&at, &["interval_range"]);
                end.push_index(index);
                d.moved(end, child(&to, &[member]));
            }
        }
    }
    Some(sidecar)
}

/// The legacy `queued_ticking` as `{min_ticks, max_ticks, looping, on_tick}`.
/// An `interval_range` that is no array has no ends: neither tick count is written, and the
/// member, at `at` in the component, is told. The field mapping is the crate's own reading of the legacy shape.
fn queued_ticking(value: &Value, at: &JsonPath, d: &mut Diagnostics) -> Value {
    let Some(object) = value.as_object() else {
        return value.clone();
    };
    let mut out = facet_value::VObject::new();
    let range = object.get("interval_range");
    if range.is_some_and(|range| !range.is_array()) {
        super::legacy::misread(&child(at, &["interval_range"]), d);
    }
    if let Some(range) = range.and_then(Value::as_array) {
        if let Some(min) = range.get(0) {
            out.insert("min_ticks", min.clone());
        }
        if let Some(max) = range.get(1) {
            out.insert("max_ticks", max.clone());
        }
        // The two ends are read; an element behind them is not, and told where it is.
        for index in 2..range.len() {
            let mut element = child(at, &["interval_range"]);
            element.push_index(index);
            d.push(upgrade_finding(
                DiagnosticCode::UnrecognizedField { text: None },
                Severity::Info,
                &element,
            ));
        }
    }
    // `looping` and `on_tick`, and with them every member the conversion does not name:
    // carried into the side-car, whose reader tells of what it does not know, as it does
    // for the components that are moved as they are.
    for (key, value) in object {
        if key.as_str() != "interval_range" {
            out.insert(key.as_str(), value.clone());
        }
    }
    out.into()
}

/// Legacy `tag:<name>` components become `minecraft:tags: [<name>, …]`, after the names a
/// `minecraft:tags` list of the section already has. A `minecraft:tags` that is no array has
/// no names to keep: it is replaced by the list of the tag components, and told (the game
/// gives the tag components alone).
fn convert_tags(map: &mut IndexMap<String, Value>, section: Option<usize>, d: &mut Diagnostics) {
    let tags: Vec<String> = map
        .keys()
        .filter_map(|k| k.strip_prefix("tag:").map(str::to_owned))
        .collect();
    if tags.is_empty() {
        return;
    }
    let first = map
        .keys()
        .position(|k| k.starts_with("tag:"))
        .unwrap_or(map.len());
    // The tag is the key; the game does not read the value. A value that is
    // no empty object is noted, so that nothing an author wrote goes without a word.
    for (key, value) in map.iter().filter(|(key, _)| key.starts_with("tag:")) {
        if !value
            .as_object()
            .is_some_and(facet_value::VObject::is_empty)
        {
            d.push(upgrade_finding(
                DiagnosticCode::InvalidType { text: None },
                Severity::Info,
                &component_path(section, key),
            ));
        }
    }
    map.retain(|k, _| !k.starts_with("tag:"));
    if map
        .get("minecraft:tags")
        .is_some_and(|tags| !tags.is_array())
    {
        super::legacy::misread(&component_path(section, "minecraft:tags"), d);
    }
    // The names the list has; an element that is no string is no name, left out alone and
    // told where it is, so that the tags of the keys are not lost with it.
    let mut list = facet_value::VArray::new();
    if let Some(had) = map.get("minecraft:tags").and_then(Value::as_array) {
        let at = component_path(section, "minecraft:tags");
        for name in super::legacy::strings_told(had, &at, d) {
            list.push(name);
        }
    }
    for tag in tags {
        list.push(tag);
    }
    if map.contains_key("minecraft:tags") {
        map.insert("minecraft:tags".to_owned(), list.into());
    } else {
        map.shift_insert(
            first.min(map.len()),
            "minecraft:tags".to_owned(),
            list.into(),
        );
    }
}

/// The legacy parse reads `menu_category.category` case-insensitively (the
/// 1.19.40 fixer keeps the author's case, `ConvertMode::Lenient`).
fn lower_category(block: &mut Block) {
    let Some(menu) = block.description.get_mut("menu_category") else {
        return;
    };
    super::ops::edit_object(menu, |m| {
        if let Some(category) = m.get("category").and_then(string) {
            let lower = category.to_ascii_lowercase();
            if lower != category {
                m.insert("category".to_owned(), Value::from(lower));
            }
        }
    });
}

/// A legacy permutation condition must be a string: "permutation condition must be a string
/// value", and the block is not registered.
fn conditions(block: &Block, d: &mut Diagnostics) {
    for (index, permutation) in block.permutations.iter().enumerate() {
        if permutation
            .condition
            .as_ref()
            .is_some_and(|c| !c.is_string())
        {
            let mut at = child(&JsonPath::new(), &["minecraft:block", "permutations"]);
            at.push_index(index);
            at.push("condition");
            d.push(upgrade_finding(
                DiagnosticCode::InvalidType {
                    text: fixed("permutation condition must be a string value"),
                },
                Severity::Error,
                &at,
            ));
        }
    }
}

/// An event node whose `condition`s are strings and whose nodes are objects, at every level.
fn strings_where_expected(node: &Value) -> bool {
    let Some(object) = node.as_object() else {
        return false;
    };
    object.iter().all(|(key, value)| match key.as_str() {
        "condition" => value.is_string(),
        "sequence" | "randomize" => value
            .as_array()
            .is_some_and(|a| a.iter().all(strings_where_expected)),
        _ => true,
    })
}

/// One legacy event node (at `from` in the file) as the `DeprecatedDefinitionEvent` at `to`:
/// `condition`, `trigger`, `sequence` and `randomize` (`weight` as `probability`) are its
/// members; every other member is one response (the crate's own rule: the game converts them by reflection). The
/// move of every member that changes its place or name is recorded.
fn event_node(node: &Value, from: &JsonPath, to: &JsonPath, d: &mut Diagnostics) -> Value {
    let mut out = facet_value::VObject::new();
    let Some(object) = node.as_object() else {
        return out.into();
    };
    let mut responses = facet_value::VArray::new();
    for (key, value) in object {
        let key = key.as_str();
        match key {
            "condition" => {
                out.insert("condition", value.clone());
            }
            // A trigger is an event name or `{event, target, condition}`.
            "trigger" => match string(value) {
                Some(event) => {
                    let mut trigger = facet_value::VObject::new();
                    trigger.insert("event", event);
                    out.insert("trigger", trigger);
                    d.moved(child(from, &[key]), child(to, &[key, "event"]));
                }
                None => {
                    out.insert("trigger", value.clone());
                }
            },
            "weight" => {
                out.insert("probability", value.clone());
                d.moved(child(from, &[key]), child(to, &["probability"]));
            }
            "sequence" | "randomize" => {
                let mut children = facet_value::VArray::new();
                for (index, node) in value.as_array().into_iter().flatten().enumerate() {
                    let (mut from, mut to) = (child(from, &[key]), child(to, &[key]));
                    from.push_index(index);
                    to.push_index(index);
                    children.push(event_node(node, &from, &to, d));
                }
                out.insert(key, children);
            }
            _ => {
                let mut place = child(to, &["responses"]);
                place.push_index(responses.len());
                place.push(key);
                d.moved(child(from, &[key]), place);
                let mut response = facet_value::VObject::new();
                response.insert(key, value.clone());
                responses.push(response);
            }
        }
    }
    if !responses.is_empty() {
        out.insert("responses", responses);
    }
    out.into()
}

/// The converted document in the 1.26.20 shape.
fn payload(
    block: &Block,
    sidecars: &[IndexMap<String, Value>],
    handlers: &IndexMap<String, Value>,
) -> Value {
    let mut object = facet_value::VObject::new();
    object.insert("description", to_object(&block.description));
    object.insert("components", to_object(&block.components));
    let mut permutations = facet_value::VArray::new();
    for (index, permutation) in block.permutations.iter().enumerate() {
        let mut p = facet_value::VObject::new();
        if let Some(condition) = &permutation.condition {
            p.insert("condition", condition.clone());
        }
        p.insert("components", to_object(&permutation.components));
        if let Some(sidecar) = sidecars.get(index + 1).filter(|s| !s.is_empty()) {
            p.insert("deprecated", to_object(sidecar));
        }
        if !permutation.ordered_v1_custom_components.is_empty() {
            p.insert(
                "ordered_v1_custom_components",
                super::legacy::strings_value(&permutation.ordered_v1_custom_components),
            );
        }
        for (key, value) in permutation.extra.iter() {
            p.insert(key, value.clone());
        }
        permutations.push(p);
    }
    if !permutations.is_empty() {
        object.insert("permutations", permutations);
    }
    if let Some(sidecar) = sidecars.first().filter(|s| !s.is_empty()) {
        object.insert("deprecated", to_object(sidecar));
    }
    if !handlers.is_empty() {
        object.insert("deprecated_event_handlers", to_object(handlers));
    }
    if !block.ordered_v1_custom_components.is_empty() {
        object.insert(
            "ordered_v1_custom_components",
            super::legacy::strings_value(&block.ordered_v1_custom_components),
        );
    }
    for (key, value) in block.extra.iter() {
        object.insert(key, value.clone());
    }
    object.into()
}

/// Parses the converted payload as the 1.26.20 document (its findings without spans: the text
/// is the crate's own), gives its Molang strings the legacy context version and marks the synthesised
/// components.
fn convert(payload: &Value, legacy: &Block, cx: &mut UpgradeCx<'_>) -> super::v1_26_20::Block {
    let identifier = legacy
        .description
        .get("identifier")
        .and_then(string)
        .unwrap_or_default()
        .to_owned();
    let text = match bedrock_addon_core::json::to_json(payload) {
        Ok(text) => text,
        // The writer refuses the payload (a number that is not finite, a value with no JSON
        // form). No file is known to reach this: the text is read with every number no double
        // holds clamped, in every copy of a component written twice (the legacy reader keeps
        // the first copy, and its number is the one warned of), and the
        // steps make no number beyond a double that they do not clamp. Such a payload is not
        // read as an empty block in silence: the refusal is an error where the writer
        // stopped, and the file is rejected with it.
        Err(error) => {
            let mut place = child(&JsonPath::new(), &["minecraft:block"]);
            // Any other refusal the writer may come to have is said at the payload.
            if let bedrock_addon_core::json::EncodeError::NotRepresentable { at, .. } = &error {
                place.append(at);
            }
            cx.diagnostics.push(upgrade_finding(
                DiagnosticCode::InvalidValue { text: None },
                Severity::Error,
                &place,
            ));
            return empty(identifier);
        }
    };
    let parse = ParseCx {
        load: cx.load,
        written: FormatVersionInput {
            version: Some(cx.original_version),
            beta_literal: false,
            prerelease: false,
        },
        original_version: cx.original_version,
        at: child(&JsonPath::new(), &["minecraft:block"]),
    };
    let mut found = Diagnostics::default();
    let parsed = parse_document(JsonSlice::new(&text), &parse, &mut found);
    for mut finding in found {
        finding.location.span = None;
        cx.diagnostics.push(finding);
    }
    let Some((mut block, one_box)) = parsed else {
        return empty(identifier);
    };
    settle_versions(&mut block, cx.load.molang_version);
    for (section, name) in &legacy.synthesized {
        mark_synthesized(&mut block, *section, name);
    }
    // What the rules of 1.26.20 refuse of a `menu_category` that an older file wrote as a
    // `minecraft:creative_category` component is told here, at the member the
    // file has; `validate` then leaves that finding to this one.
    // Where the file has the members the rules of 1.26.20 refuse, for the write's refusal:
    // the moves of the chain are all recorded here, at its last step.
    let payload_at = child(&JsonPath::new(), &["minecraft:block"]);
    for finding in crate::validate::refused_by_newest(&block, &payload_at) {
        let of_block = finding.location.json_path;
        let of_file = cx.diagnostics.source_of(&of_block);
        if of_file != of_block {
            block.refused_at.push((of_block, of_file));
        }
    }
    if let Some(section) = legacy.category_from {
        let payload = child(&JsonPath::new(), &["minecraft:block"]);
        let menu = child(&payload, &["description", "menu_category"]);
        for mut finding in crate::validate::refused_by_newest(&block, &payload) {
            // The members of a menu category are keys (`category`, `group`).
            let Some(member) = finding
                .location
                .json_path
                .strip_prefix(&menu)
                .filter(|member| !member.is_empty())
            else {
                continue;
            };
            let mut place = component_path(section, "minecraft:creative_category");
            place.append(&member);
            finding.location.json_path = place;
            finding.severity = Severity::Info;
            finding.phase = bedrock_addon_core::diagnostic::Phase::Upgrade;
            cx.diagnostics.push(finding);
        }
    }
    super::v1_26_20::Block { block, one_box }
}

/// The empty block a failed conversion gives (its error is told).
fn empty(identifier: String) -> super::v1_26_20::Block {
    super::v1_26_20::Block {
        block: crate::Block::new(BlockId::parse(identifier)),
        one_box: Vec::new(),
    }
}

/// The legacy path's Molang versions: strings take the
/// pack's version, descriptor `tags` 13; an object form keeps its own.
fn settle_versions(block: &mut crate::Block, pack: RawVersion) {
    visit_fields(block, &JsonPath::new(), |field| {
        let version = if field.spec.admission == Admission::Tags {
            RawVersion::from(MolangVersion::V13)
        } else {
            pack
        };
        match field.value {
            MolangFieldMut::Value(Molang::Expr(source)) | MolangFieldMut::Source(source)
                if source.raw_version().is_none() =>
            {
                source.set_context_version(version);
            }
            _ => {}
        }
    });
}

/// Marks the component `name` of `section` as synthesised (it bypasses the release gates).
fn mark_synthesized(block: &mut crate::Block, section: Option<usize>, name: &str) {
    let map = match section {
        None => &mut block.components,
        Some(index) => match block.permutations.get_mut(index) {
            Some(p) => &mut p.components,
            None => return,
        },
    };
    let value = map.iter().find_map(|entry| match entry {
        ComponentRef::Typed { name: n, value, .. } if n == name => Some(value.clone()),
        _ => None,
    });
    if let Some(value) = value {
        let _: Option<BlockComponent> = map.insert_typed(value, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::Phase;
    use bedrock_addon_core::load::LoadContext;
    use bedrock_addon_core::vanilla::ActorType;

    /// What `convert` makes of `payload` for the legacy block `demo:x`: the block and the
    /// step's findings.
    fn converted(payload: &Value) -> (crate::Block, Diagnostics) {
        let mut legacy = Block::default();
        legacy
            .description
            .insert("identifier".to_owned(), Value::from("demo:x"));
        let load = LoadContext::detached();
        let mut diagnostics = Diagnostics::default();
        let mut cx = UpgradeCx {
            load: &load,
            original_version: FormatVersion::new(1, 21, 60),
            identifier: "demo:x",
            actor_type: ActorType::Mob,
            skip_section_a: false,
            diagnostics: &mut diagnostics,
        };
        let block = convert(payload, &legacy, &mut cx).block;
        (block, diagnostics)
    }

    fn payload_with(friction: Value) -> Value {
        let mut components = facet_value::VObject::new();
        components.insert("minecraft:friction", friction);
        let mut description = facet_value::VObject::new();
        description.insert("identifier", Value::from("demo:x"));
        let mut payload = facet_value::VObject::new();
        payload.insert("description", description);
        payload.insert("components", components);
        payload.into()
    }

    /// The step writes the converted payload as text to read it as the 1.26.20
    /// document. A payload the writer refuses, whatever the reason (here a number a step
    /// could have computed beyond a double), must not become an empty block in silence: the
    /// step says so with an error at the place the writer names.
    #[test]
    fn a_payload_the_writer_refuses_is_an_error_not_a_silent_empty_block() {
        for unwritable in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            let (block, found) = converted(&payload_with(Value::from(unwritable)));
            assert!(
                block.components.is_empty(),
                "nothing of the payload is read"
            );
            let errors: Vec<String> = found
                .iter()
                .filter(|finding| finding.severity == Severity::Error)
                .map(|finding| {
                    format!(
                        "{} {:?} at `{}`",
                        finding.code.name(),
                        finding.phase,
                        finding.location.json_path
                    )
                })
                .collect();
            assert_eq!(
                errors,
                [format!(
                    "InvalidValue {:?} at `minecraft:block/components/minecraft:friction`",
                    Phase::Upgrade
                )],
                "{unwritable}: the refusal is said where the writer stopped"
            );
        }
    }

    /// The same payload with a number the writer writes converts without an error.
    #[test]
    fn a_payload_the_writer_writes_converts_without_an_error() {
        let (block, found) = converted(&payload_with(Value::from(0.4)));
        assert!(!block.components.is_empty());
        assert!(!found.has_errors(), "{found:?}");
    }
}
