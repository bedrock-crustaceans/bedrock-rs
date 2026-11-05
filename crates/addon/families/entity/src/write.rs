//! Writing the newest struct: the payload in schema order, every kept unknown member after the
//! known ones at its level.

use bedrock_addon_core::document::{ComponentMap, ComponentRef};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::versioning::write_payload;
use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use super::component::EntityComponent;
use super::{AnimateEntry, Description, Entity, Property, PropertyDefault, Scripts};

/// The payload object.
pub(crate) fn payload(entity: &Entity) -> Result<Value, EncodeError> {
    let mut object = VObject::new();
    object.insert("description", description(&entity.description));
    if entity.written.components || !entity.components.is_empty() {
        object.insert("components", components(&entity.components)?);
    }
    if entity.written.component_groups || !entity.component_groups.is_empty() {
        let mut groups = VObject::new();
        for (name, group) in &entity.component_groups {
            groups.insert(name.as_str(), components(&group.components)?);
        }
        object.insert("component_groups", groups);
    }
    if entity.written.events || !entity.events.is_empty() {
        let mut events = VObject::new();
        for event in entity.events.values() {
            events.insert(event.name(), event.node.clone());
        }
        object.insert("events", events);
    }
    if let Some(brain) = &entity.brain {
        object.insert("brain", write_payload(brain)?);
    }
    if !entity.upgrades.is_empty() {
        object.insert("upgrades", map(&entity.upgrades));
    }
    if let Some(deprecated) = &entity.deprecated {
        let mut side_car = VObject::new();
        if !deprecated.aliases.is_empty() {
            side_car.insert("aliases", map(&deprecated.aliases));
        }
        object.insert("__deprecated", side_car);
    }
    Ok(object.into())
}

/// A map of JSON values as an object.
fn map(members: &IndexMap<String, Value>) -> Value {
    let mut object = VObject::new();
    for (key, value) in members {
        object.insert(key.as_str(), value.clone());
    }
    object.into()
}

/// `description`.
fn description(description: &Description) -> Value {
    let mut object = VObject::new();
    object.insert("identifier", description.identifier.as_str());
    if let Some(runtime) = &description.runtime_identifier {
        object.insert("runtime_identifier", runtime.as_str());
    }
    if let Some(spawnable) = description.is_spawnable {
        object.insert("is_spawnable", spawnable);
    }
    if let Some(summonable) = description.is_summonable {
        object.insert("is_summonable", summonable);
    }
    if !description.properties.is_empty() {
        let mut properties = VObject::new();
        for (name, property) in &description.properties {
            properties.insert(name.as_str(), self::property(property));
        }
        object.insert("properties", properties);
    }
    if !description.animations.is_empty() {
        let mut animations = VObject::new();
        for (name, animation) in &description.animations {
            animations.insert(name.as_str(), animation.as_str());
        }
        object.insert("animations", animations);
    }
    if let Some(scripts) = &description.scripts {
        object.insert("scripts", self::scripts(scripts));
    }
    if let Some(category) = description.spawn_category {
        object.insert("spawn_category", category.as_str());
    }
    if let Some(mapping) = &description.property_memory_mapping {
        object.insert("property_memory_mapping", mapping.clone());
    }
    if let Some(version) = &description.entity_version {
        let mut upgrades = VArray::new();
        for entry in &version.upgrades {
            let mut item = VObject::new();
            item.insert("upgrade_to", entry.upgrade_to);
            item.insert("name", entry.name.as_str());
            upgrades.push(item);
        }
        let mut object_version = VObject::new();
        object_version.insert("upgrades", upgrades);
        object.insert("entity_version", object_version);
    }
    object.into()
}

/// One property.
fn property(property: &Property) -> Value {
    let mut object = VObject::new();
    object.insert("type", property.kind.as_str());
    if let Some(default) = &property.default {
        let value: Value = match default {
            PropertyDefault::Bool(b) => (*b).into(),
            PropertyDefault::Int(i) => (*i).into(),
            PropertyDefault::Float(f) => (*f).into(),
            PropertyDefault::Enum(s) => s.as_str().into(),
            PropertyDefault::Expression(source) => source.as_str().into(),
        };
        object.insert("default", value);
    }
    if let Some([min, max]) = &property.range {
        let mut range = VArray::new();
        range.push(min.clone());
        range.push(max.clone());
        object.insert("range", range);
    }
    if !property.values.is_empty() {
        let mut values = VArray::new();
        for value in &property.values {
            values.push(value.as_str());
        }
        object.insert("values", values);
    }
    if let Some(sync) = property.client_sync {
        object.insert("client_sync", sync);
    }
    object.into()
}

/// `scripts`.
fn scripts(scripts: &Scripts) -> Value {
    let mut object = VObject::new();
    let mut animate = VArray::new();
    for entry in &scripts.animate {
        match entry {
            AnimateEntry::Always(name) => animate.push(name.as_str()),
            AnimateEntry::Conditional(members) => {
                let mut member = VObject::new();
                for (name, condition) in members {
                    member.insert(name.as_str(), condition.as_str());
                }
                animate.push(member);
            }
        }
    }
    if scripts.written.animate || !animate.is_empty() {
        object.insert("animate", animate);
    }
    object.into()
}

/// A component map, in its order.
pub(crate) fn components(map: &ComponentMap<EntityComponent>) -> Result<Value, EncodeError> {
    let mut object = VObject::new();
    for entry in map.iter() {
        match entry {
            ComponentRef::Typed { name, value, .. } => {
                object.insert(name, component(value)?);
            }
            ComponentRef::Custom(custom) => {
                object.insert(custom.name.as_str(), custom.parameters.clone());
            }
            ComponentRef::Dynamic { name, json } | ComponentRef::Unknown { name, json } => {
                object.insert(name, json.clone());
            }
            _ => {}
        }
    }
    Ok(object.into())
}

/// One typed component as JSON: the dispatch `#[derive(Components)]` emits from the rows.
///
/// # Errors
///
/// [`EncodeError::NotRepresentable`] when the component holds a value JSON cannot spell, such as
/// a non-finite number.
pub fn component(value: &EntityComponent) -> Result<Value, EncodeError> {
    value.write_component()
}
