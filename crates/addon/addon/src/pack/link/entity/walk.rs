//! The entity documents of a pass as the rules read them: every component of every entity of
//! the behaviour stack as JSON in the newest format's shape, with its place in its file, and
//! the members of a component a pattern names.
//!
//! A typed component is written back to JSON once per pass (a component that cannot be
//! written is left out: it holds a value the format cannot express, which the load has
//! told); a component the game reads at run time is the JSON the file has. A custom
//! component is a script's and a key no table knows has no member a rule could name: neither
//! is walked.

use std::borrow::Cow;
use std::collections::HashMap;

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::{ComponentMap, ComponentRef, Origin};
use bedrock_addon_core::json::Value;

use super::super::LinkCx;
use crate::families::entity::Entity;
use crate::families::entity::component::EntityComponent;

/// The payload key of an entity document.
pub(super) const PAYLOAD: &str = "minecraft:entity";

/// One entity definition of the stack: a stable document or a beta one.
#[derive(Debug)]
pub(super) struct Definition<'a> {
    /// Where the document is.
    pub(super) origin: &'a Origin,
    /// The entity.
    pub(super) entity: &'a Entity,
    /// The document's findings: they give the place in the file of a member the entity
    /// holds (`Diagnostics::place_of`).
    pub(super) places: &'a Diagnostics,
}

/// Every effective entity definition of the behaviour stack, stable then beta, in load order.
pub(super) fn definitions<'a>(cx: &LinkCx<'a>) -> Vec<Definition<'a>> {
    let registry = cx.behavior().entities();
    let mut all: Vec<Definition<'a>> = registry
        .iter()
        .map(|(_, document)| Definition {
            origin: document.origin(),
            entity: document.value(),
            places: document.diagnostics(),
        })
        .collect();
    if let Some(beta) = registry.beta() {
        all.extend(beta.iter().map(|(_, document)| Definition {
            origin: document.origin(),
            entity: &document.value().0,
            places: document.diagnostics(),
        }));
    }
    all
}

/// The sections of an entity that hold components: `components` and each component group,
/// with the JSON path of each.
pub(super) fn sections(
    entity: &Entity,
) -> impl Iterator<Item = (JsonPath, &ComponentMap<EntityComponent>)> + '_ {
    let own = std::iter::once((path(&[PAYLOAD, "components"]), &entity.components));
    let groups = entity.component_groups.iter().map(|(name, group)| {
        (
            path(&[PAYLOAD, "component_groups", name]),
            &group.components,
        )
    });
    own.chain(groups)
}

/// One component of an entity as JSON.
#[derive(Debug)]
pub(super) struct Component<'a> {
    /// The document that holds it.
    pub(super) origin: &'a Origin,
    /// The component's key (`minecraft:loot`).
    pub(super) key: &'a str,
    /// The component's path in the document as loaded.
    pub(super) at: JsonPath,
    /// Its value in the newest format's shape.
    pub(super) json: Cow<'a, Value>,
    /// The findings of the document that holds it ([`Definition::places`]).
    pub(super) places: &'a Diagnostics,
}

/// One node of an entity's events: an event's body or a node nested in its `sequence`,
/// `randomize` or `first_valid`.
#[cfg_attr(
    not(feature = "particle"),
    allow(dead_code, reason = "the event nodes are the particle rule's")
)]
#[derive(Debug)]
pub(super) struct EventNode<'a> {
    /// The document that holds it.
    pub(super) origin: &'a Origin,
    /// The node's path in its file.
    pub(super) at: JsonPath,
    /// The node.
    pub(super) json: &'a Value,
}

/// What the rules over entity documents read, built once per pass.
#[derive(Debug)]
pub(super) struct Entities<'a> {
    /// The definitions.
    #[cfg_attr(
        not(feature = "block"),
        allow(dead_code, reason = "the typed goal of the block rule reads them")
    )]
    pub(super) definitions: Vec<Definition<'a>>,
    /// Every component of every definition.
    pub(super) components: Vec<Component<'a>>,
    /// Every event node of every definition.
    #[cfg_attr(
        not(feature = "particle"),
        allow(dead_code, reason = "the event nodes are the particle rule's")
    )]
    pub(super) events: Vec<EventNode<'a>>,
}

impl<'a> Entities<'a> {
    /// The entities of the behaviour stack of `cx`.
    pub(super) fn new(cx: &LinkCx<'a>) -> Self {
        let definitions = definitions(cx);
        let mut components = Vec::new();
        let mut events = Vec::new();
        for definition in &definitions {
            for (section, map) in sections(definition.entity) {
                for entry in map.iter() {
                    let (key, json) = match entry {
                        ComponentRef::Typed { name, value, .. } => {
                            match crate::families::entity::component_json(value) {
                                Ok(json) => (name, Cow::Owned(json)),
                                Err(_) => continue,
                            }
                        }
                        ComponentRef::Dynamic { name, json } => (name, Cow::Borrowed(json)),
                        _ => continue,
                    };
                    let mut at = section.clone();
                    at.push(key);
                    components.push(Component {
                        origin: definition.origin,
                        key,
                        at,
                        json,
                        places: definition.places,
                    });
                }
            }
            for event in definition.entity.events.values() {
                let at = path(&[PAYLOAD, "events", event.name()]);
                events.extend(event_nodes(definition.origin, at, &event.node));
            }
        }
        Self {
            definitions,
            components,
            events,
        }
    }

    /// Calls `found` with every member the rows of `table` name, in document order: the
    /// component, the member's path in its file and its value. A row is a component key and a
    /// member pattern ([`members`]).
    ///
    /// The members are walked in the shape the entity holds, and each is named where the
    /// file has it: the lenient conversion of a document below a component's typed version
    /// leaves a list element out that the typed member does not take, a step moves a member,
    /// and the document keeps where each member it holds is in the file.
    pub(super) fn each(
        &self,
        table: &[(&'static str, &'static str)],
        found: &mut dyn FnMut(&Component<'a>, JsonPath, &Value),
    ) {
        let mut rows: HashMap<&str, Vec<&str>> = HashMap::new();
        for (key, pattern) in table {
            rows.entry(key).or_default().push(pattern);
        }
        for component in &self.components {
            for pattern in rows.get(component.key).into_iter().flatten() {
                members(&component.json, pattern, &component.at, &mut |at, value| {
                    found(component, component.places.place_of(&at), value);
                });
            }
        }
    }
}

/// `node` and the nodes nested in it, each with its path.
fn event_nodes<'a>(origin: &'a Origin, at: JsonPath, node: &'a Value) -> Vec<EventNode<'a>> {
    let mut out = Vec::new();
    // A stack of the nodes still to visit: an event's nesting is the file's, not the stack's.
    let mut pending = vec![(at, node)];
    while let Some((at, node)) = pending.pop() {
        let Some(object) = node.as_object() else {
            continue;
        };
        let mut nested = Vec::new();
        for key in ["sequence", "randomize", "first_valid"] {
            let Some(list) = object.get(key).and_then(Value::as_array) else {
                continue;
            };
            for (index, child) in list.iter().enumerate() {
                let mut child_at = at.clone();
                child_at.push(key);
                child_at.push_index(index);
                nested.push((child_at, child));
            }
        }
        out.push(EventNode {
            origin,
            at,
            json: node,
        });
        // Document order: the first nested node is visited next.
        pending.extend(nested.into_iter().rev());
    }
    out
}

/// The path of `keys`.
pub(super) fn path(keys: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    for key in keys {
        path.push(key);
    }
    path
}

/// Calls `found` with every value the member `pattern` names below `value` (which is at
/// `at`), with its path. A pattern is member keys joined by `.`; a key followed by `[]` names
/// each element of the member when it is an array and the member itself when it is not (the
/// game reads one or a list there); `*` names every member of an object.
pub(super) fn members<'v>(
    value: &'v Value,
    pattern: &str,
    at: &JsonPath,
    found: &mut dyn FnMut(JsonPath, &'v Value),
) {
    if pattern.is_empty() {
        found(at.clone(), value);
        return;
    }
    let (step, rest) = pattern.split_once('.').unwrap_or((pattern, ""));
    let Some(object) = value.as_object() else {
        return;
    };
    if step == "*" {
        for (key, member) in object {
            let mut member_at = at.clone();
            member_at.push(key.as_str());
            members(member, rest, &member_at, found);
        }
        return;
    }
    let (key, list) = match step.strip_suffix("[]") {
        Some(key) => (key, true),
        None => (step, false),
    };
    let Some(member) = object.get(key) else {
        return;
    };
    let mut member_at = at.clone();
    member_at.push(key);
    match member.as_array() {
        Some(elements) if list => {
            for (index, element) in elements.iter().enumerate() {
                let mut element_at = member_at.clone();
                element_at.push_index(index);
                members(element, rest, &element_at, found);
            }
        }
        _ => members(member, rest, &member_at, found),
    }
}
