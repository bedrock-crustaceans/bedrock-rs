//! The Molang members of the components kept as JSON (`Dynamic`:
//! `ambient_sound_interval.event_names[].condition`, `heartbeat.interval` and
//! `reflect_projectiles.{azimuth_angle, elevation_angle, reflection_scale}`; the member
//! `rideable.seats[].rotate_rider_by` is the typed `rideable`'s from 1.26.60): read
//! schema-validated at the pack's Molang version
//! (rule M1); a rejected expression, or a value that is neither a number, a bool nor a string,
//! fails the component's typed node, and the component is dropped (`DroppedComponent`).
//!
//! And the `tags` of the descriptors these components hold (a block or item descriptor
//! `{"tags": …}` in a component: the pack's version in `Dynamic` components, rule M3), at every member of these components that holds a block or an item descriptor.
//! A rejected expression is told and the descriptor kept with it.

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, JsonPath, Location, Phase};
use bedrock_addon_core::document::{ComponentMap, ComponentRef};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{
    Molang, MolangContext, MolangFieldSpec, MolangFields, MolangVisitor, ReaderKind, Step,
    not_checked, pass_one,
};
use bedrock_addon_core::version::FormatVersionInput;
use bedrock_addon_core::versioning::ValidateCx;

use super::Entity;
use super::component::EntityComponent;

/// A Molang member of a `Dynamic` component: the typed node vanilla's parser reads it with.
/// The game logs a rejected expression at the error level and loads the entity, so it is a kept
/// error; the crate drops the component itself.
// The parser logs the rejection at the error level and the entity loads: a kept error. The
// game discards the parse result and keeps the component, so the crate's drop of the
// component is its own act.
const MEMBER: MolangFieldSpec = MolangFieldSpec::KEPT.reader(ReaderKind::SchemaValidated);

/// The `tags` of a block or item descriptor of a `Dynamic` component: a string, at the
/// pack's Molang version, with the `tags` query set. The game logs a rejected expression and
/// loads the entity, the descriptor kept with it: a kept error, and nothing is dropped.
// Rule M3: the descriptor is kept with the rejected expression.
// The parser logs the rejection at the error level and the entity loads: a kept error. The
// game returns the descriptor even when the expression fails to parse.
const TAGS: MolangFieldSpec = MolangFieldSpec::KEPT.tags();

/// The members of `Dynamic` components that hold a block or an item descriptor: the
/// component's key and the member's pattern (keys joined by `.`; `[]` after a key for "each
/// element, or the member itself when it is no array"). A descriptor is a name or an object;
/// the object may be an entry whose `item` is the descriptor.
///
/// They are the rows of the link pass's tables of block and item members whose component is
/// kept as JSON (`BLOCKS`, `BLOCKS_AS_ITEMS` and `ITEMS` in
/// `crates/addon/addon/src/pack/link/entity/table.rs`); a test holds the two lists to each other.
/// The game's own files write a `tags` at two of them (`navigation.walk` `blocks_to_avoid[]`,
/// `behavior.tempt` `items[]`), and the rule is the same for all.
#[doc(hidden)]
pub const DESCRIPTORS: &[(&str, &str)] = &[
    ("minecraft:ageable", "drop_items[]"),
    ("minecraft:ageable", "feedItems[]"),
    ("minecraft:ageable", "feedItems[].result_item"),
    ("minecraft:ageable", "feed_items[]"),
    ("minecraft:ageable", "feed_items[].result_item"),
    ("minecraft:ageable", "pause_growth_items[]"),
    ("minecraft:ageable", "reset_growth_items[]"),
    ("minecraft:block_sensor", "on_break[].block_list[]"),
    ("minecraft:boostable", "boost_items[]"),
    ("minecraft:boostable", "boost_items[].replaceItem"),
    ("minecraft:boostable", "boost_items[].replace_item"),
    ("minecraft:break_blocks", "breakable_blocks[]"),
    ("minecraft:breathable", "breatheBlocks[]"),
    ("minecraft:breathable", "breathe_blocks[]"),
    ("minecraft:breathable", "nonBreatheBlocks[]"),
    ("minecraft:breathable", "non_breathe_blocks[]"),
    ("minecraft:breedable", "breedItems[]"),
    ("minecraft:breedable", "breedItems[].result_item"),
    ("minecraft:breedable", "breed_items[]"),
    ("minecraft:breedable", "breed_items[].result_item"),
    ("minecraft:breedable", "environment_requirements[].blocks[]"),
    ("minecraft:bribeable", "bribe_items[]"),
    ("minecraft:equip_item", "excluded_items[]"),
    ("minecraft:equippable", "slots[].accepted_items[]"),
    ("minecraft:equippable", "slots[].item"),
    ("minecraft:giveable", "triggers[].items[]"),
    ("minecraft:inside_block_notifier", "block_list[].block"),
    ("minecraft:item_controllable", "control_items[]"),
    (
        "minecraft:memory_sensor.find_nearest_block_from",
        "sensor_entries[].blocks[]",
    ),
    ("minecraft:navigation.climb", "blocks_to_avoid[]"),
    ("minecraft:navigation.float", "blocks_to_avoid[]"),
    ("minecraft:navigation.fly", "blocks_to_avoid[]"),
    ("minecraft:navigation.generic", "blocks_to_avoid[]"),
    ("minecraft:navigation.hover", "blocks_to_avoid[]"),
    ("minecraft:navigation.swim", "blocks_to_avoid[]"),
    ("minecraft:navigation.walk", "blocks_to_avoid[]"),
    ("minecraft:spawn_entity", "entities[].spawn_item"),
    ("minecraft:tameable", "tameItems[]"),
    ("minecraft:tameable", "tameItems[].result_item"),
    ("minecraft:tameable", "tame_items[]"),
    ("minecraft:tameable", "tame_items[].result_item"),
    ("minecraft:trusting", "trust_items[]"),
];

/// Calls `found` with every value the member `pattern` names below `value`, with its path
/// below `value`.
fn each_member<'v>(
    value: &'v Value,
    pattern: &str,
    at: &mut Vec<Seg>,
    found: &mut dyn FnMut(&[Seg], &'v Value),
) {
    if pattern.is_empty() {
        found(at, value);
        return;
    }
    let (step, rest) = pattern.split_once('.').unwrap_or((pattern, ""));
    let (key, list) = match step.strip_suffix("[]") {
        Some(key) => (key, true),
        None => (step, false),
    };
    let Some(member) = value.as_object().and_then(|object| object.get(key)) else {
        return;
    };
    at.push(Seg::Key(key.to_owned()));
    match member.as_array() {
        Some(elements) if list => {
            for (index, element) in elements.iter().enumerate() {
                at.push(Seg::Index(index));
                each_member(element, rest, at, found);
                at.pop();
            }
        }
        _ => each_member(member, rest, at, found),
    }
    at.pop();
}

/// The `tags` expression of the descriptor `value` is, or of the descriptor its `item` is,
/// with the path of the `tags` below `value`: a string; a `tags` of another JSON type is no
/// expression.
fn descriptor_tags(value: &Value) -> Option<(Vec<Seg>, &Value)> {
    fn tags_of(descriptor: &Value) -> Option<&Value> {
        descriptor
            .as_object()?
            .get("tags")
            .filter(|tags| tags.is_string())
    }
    let key = |k: &str| Seg::Key(k.to_owned());
    let object = value.as_object()?;
    if object.contains_key("tags") {
        return tags_of(value).map(|tags| (vec![key("tags")], tags));
    }
    let tags = tags_of(object.get("item")?)?;
    Some((vec![key("item"), key("tags")], tags))
}

/// One step of a path below the payload.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Seg {
    Key(String),
    Index(usize),
}

/// A component in its section: `None` for `components`, else the group's name; the key.
type ComponentAt = (Option<String>, String);

/// One Molang member of a `Dynamic` component.
struct Member {
    /// The component that holds it.
    #[cfg_attr(not(feature = "molang"), allow(dead_code))]
    component: ComponentAt,
    /// The member's path below the payload.
    path: Vec<Seg>,
    /// The expression.
    value: Molang,
    /// Whether it is the `tags` of a descriptor: kept in its component when it is rejected,
    /// where a typed node fails the component.
    descriptor: bool,
}

impl Member {
    /// How the member is read and compiled.
    fn spec(&self) -> &'static MolangFieldSpec {
        if self.descriptor { &TAGS } else { &MEMBER }
    }
}

/// The members, walked as Molang fields from the payload.
struct Members(Vec<Member>);

impl MolangFields for Members {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        for member in &mut self.0 {
            let Some((last, parents)) = member.path.split_last() else {
                continue;
            };
            for seg in parents {
                visitor.enter(match seg {
                    Seg::Key(key) => Step::Key(key),
                    Seg::Index(index) => Step::Index(*index),
                });
            }
            let spec = member.spec();
            match last {
                Seg::Key(key) => visitor.field(key.as_str(), spec, &mut member.value),
                Seg::Index(index) => visitor.field(*index, spec, &mut member.value),
            }
            for _ in parents {
                visitor.leave();
            }
        }
    }
}

/// Checks the Molang members of every `Dynamic` component of `entity` and drops the components
/// whose member fails.
pub(crate) fn check(entity: &mut Entity, cx: &mut ValidateCx<'_>) {
    let mut drop: Vec<ComponentAt> = Vec::new();
    let mut members = gather(entity, cx, &mut drop);
    if !members.0.is_empty() {
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
        pass_one(&mut members, &molang, &base, cx.diagnostics);
        compile(&mut members, &molang, &base, cx, &mut drop);
    }
    drop.dedup();
    for (section, key) in drop {
        let map = match &section {
            None => Some(&mut entity.components),
            Some(group) => entity
                .component_groups
                .get_mut(group)
                .map(|group| &mut group.components),
        };
        if map.is_some_and(|map| map.remove_named(&key)) {
            let at = json_path(&cx.at, &section_path(section.as_deref(), &key));
            // The game keeps the component and writes no content-log line; the drop is the crate's own,
            // so it is reported as an information.
            cx.diagnostics.push(
                Diagnostic::info(DiagnosticCode::DroppedComponent { text: None })
                    .at(at)
                    .phase(Phase::Validate),
            );
        }
    }
}

/// The Molang members of every `Dynamic` component; a member of another JSON type is
/// `InvalidType` and its component is listed for the drop.
fn gather(entity: &Entity, cx: &mut ValidateCx<'_>, drop: &mut Vec<ComponentAt>) -> Members {
    let sections = std::iter::once((None, &entity.components)).chain(
        entity
            .component_groups
            .iter()
            .map(|(name, group)| (Some(name.as_str()), &group.components)),
    );
    let mut members = Vec::new();
    for (section, map) in sections {
        for (key, rel, value, descriptor) in dynamic_members(map) {
            let mut path = section_path(section, &key);
            path.extend(rel);
            let component = (section.map(str::to_owned), key);
            if value.is_number() || value.is_bool() {
                continue;
            }
            let molang = value.as_string().and_then(|s| {
                let text = bedrock_addon_core::json::to_json(&Value::from(s.as_str())).ok()?;
                bedrock_addon_core::json::from_str::<Molang>(text.trim_end()).ok()
            });
            if let Some(value) = molang {
                members.push(Member {
                    component,
                    path,
                    value,
                    descriptor,
                });
            } else {
                // An information: the game writes no content-log line for a member of another
                // JSON type in the legacy reader (the typed slice's type error belongs to the
                // strict read).
                cx.diagnostics.push(
                    Diagnostic::info(DiagnosticCode::InvalidType { text: None })
                        .at(json_path(&cx.at, &path))
                        .phase(Phase::Validate),
                );
                drop.push(component);
            }
        }
    }
    Members(members)
}

/// The path of the component `key` in its section, below the payload.
fn section_path(section: Option<&str>, key: &str) -> Vec<Seg> {
    let mut path = match section {
        None => vec![Seg::Key("components".to_owned())],
        Some(group) => vec![
            Seg::Key("component_groups".to_owned()),
            Seg::Key(group.to_owned()),
        ],
    };
    path.push(Seg::Key(key.to_owned()));
    path
}

/// The Molang members of the `Dynamic` entries of `map`: the component's key, the member's
/// path below the component, its value, and whether it is the `tags` of a descriptor (a
/// string; a `tags` of another JSON type is no expression and is not listed).
fn dynamic_members(map: &ComponentMap<EntityComponent>) -> Vec<(String, Vec<Seg>, Value, bool)> {
    let mut out = Vec::new();
    for entry in map.iter() {
        let ComponentRef::Dynamic { name, json } = entry else {
            continue;
        };
        let Some(object) = json.as_object() else {
            continue;
        };
        let key = |k: &str| Seg::Key(k.to_owned());
        for (_, pattern) in DESCRIPTORS
            .iter()
            .filter(|(component, _)| *component == name)
        {
            each_member(json, pattern, &mut Vec::new(), &mut |at, descriptor| {
                if let Some((below, tags)) = descriptor_tags(descriptor) {
                    let rel = at.iter().cloned().chain(below).collect();
                    out.push((name.to_owned(), rel, tags.clone(), true));
                }
            });
        }
        let mut push = |rel: Vec<Seg>, value: &Value| {
            out.push((name.to_owned(), rel, value.clone(), false));
        };
        match name {
            "minecraft:heartbeat" => {
                if let Some(value) = object.get("interval") {
                    push(vec![key("interval")], value);
                }
            }
            "minecraft:reflect_projectiles" => {
                for member in ["azimuth_angle", "elevation_angle", "reflection_scale"] {
                    if let Some(value) = object.get(member) {
                        push(vec![key(member)], value);
                    }
                }
            }
            "minecraft:ambient_sound_interval" => {
                let events = object.get("event_names").and_then(Value::as_array);
                for (i, event) in events.into_iter().flatten().enumerate() {
                    if let Some(value) = event.as_object().and_then(|e| e.get("condition")) {
                        push(
                            vec![key("event_names"), Seg::Index(i), key("condition")],
                            value,
                        );
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Pass 2: compiled with the load context's check, else counted as not checked. A rejected
/// expression lists its component for the drop.
/// Without the `molang` feature every expression is counted as not checked.
fn compile(
    members: &mut Members,
    molang: &MolangContext,
    base: &Location,
    cx: &mut ValidateCx<'_>,
    drop: &mut Vec<ComponentAt>,
) {
    #[cfg(feature = "molang")]
    if let Some(check) = cx.load.molang.clone() {
        use bedrock_addon_core::molang::{MolangOutcome, Side, check_fields};
        let failures = check_fields(members, &check, Side::Server, molang, base, cx.diagnostics);
        for failure in failures {
            if !matches!(failure.outcome, MolangOutcome::Failed(_)) {
                continue;
            }
            if let Some(found) = members
                .0
                .iter()
                .find(|m| !m.descriptor && json_path(&cx.at, &m.path) == failure.path)
            {
                drop.push(found.component.clone());
            }
        }
        return;
    }
    #[cfg(not(feature = "molang"))]
    let _ = (molang, drop);
    not_checked(members, base, cx.diagnostics);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pattern_names_each_element_of_a_list_or_the_member_itself() {
        let json = bedrock_addon_core::json::from_str::<Value>(
            r#"{"a": [{"b": [1, 2]}, {"b": 3}, 4], "c": {"d": 5}}"#,
        )
        .expect("json");
        let found = |pattern: &str| {
            let mut out = Vec::new();
            each_member(&json, pattern, &mut Vec::new(), &mut |at, value| {
                let path: Vec<String> = at
                    .iter()
                    .map(|seg| match seg {
                        Seg::Key(key) => key.clone(),
                        Seg::Index(index) => index.to_string(),
                    })
                    .collect();
                out.push(format!(
                    "{}={:?}",
                    path.join("/"),
                    value.as_number().map(facet_value::VNumber::to_f64_lossy)
                ));
            });
            out
        };
        assert_eq!(
            found("a[].b[]"),
            ["a/0/b/0=Some(1.0)", "a/0/b/1=Some(2.0)", "a/1/b=Some(3.0)"]
        );
        assert_eq!(found("c.d"), ["c/d=Some(5.0)"]);
        assert_eq!(found("c[].d"), ["c/d=Some(5.0)"]);
        assert_eq!(found("zz[]"), [] as [&str; 0]);
    }
}
