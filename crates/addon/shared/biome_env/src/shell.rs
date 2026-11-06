//! The payload object of a biome document, `{description, components}` (and, in the beta
//! document, `environment_attributes`): one component's read re-rooted at its key, and the
//! payload's writers.
//!
//! Each family reads the payload through shells of its own (`description` as its
//! `Description`, `components` as raw slices) and each component by its own rules (which
//! components a struct binds, its gates, its era shapes), handing the read to
//! [`read_component`]. The writers put the members in the game's order; the document's writer
//! appends its unknown members after them.
//!
//! The shells are not here, generic over the description, because facet cannot derive them:
//! on a container with a type parameter its derive dispatches each extension attribute in
//! generic-safe (`@const`) form, and the attribute grammar has no such form for a
//! string-valued key, so `schema::unknown = "keep"` (which keeps the payload's unknown members)
//! falls through to facet's own dispatcher and is refused as unknown (facet 0.50.0-rc.7,
//! `define_attr_grammar!`'s `newtype_str` arms; no upstream issue yet).

use facet::Facet;
use facet_value::VObject;

use crate::EnvironmentAttributes;
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::{ComponentMap, ComponentRef, ComponentSet};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::versioning::write_payload;

/// One component read at `path`: `read` reports relative to the component, so its findings
/// are re-rooted below `path`; when it read, the unknown members it kept go below `path` in
/// `d`'s table (a failed read keeps none).
pub fn read_component<C>(
    path: &JsonPath,
    d: &mut Diagnostics,
    read: impl FnOnce(&mut Diagnostics) -> Option<C>,
) -> Option<C> {
    let mut found = Diagnostics::default();
    let value = read(&mut found);
    let kept = found.take_unknown();
    for mut diagnostic in found {
        let mut full = path.clone();
        full.append(&diagnostic.location.json_path);
        diagnostic.location.json_path = full;
        d.push(diagnostic);
    }
    if value.is_some() {
        // The component's unknown members, below its key.
        d.unknown_mut().absorb(path, kept);
    }
    value
}

/// A component map as a JSON object, in map order: a typed component as `write` writes it,
/// custom, dynamic and unknown entries as they were read.
///
/// # Errors
///
/// The first error `write` returns.
pub fn write_components<C: ComponentSet>(
    map: &ComponentMap<C>,
    write: impl Fn(&C) -> Result<Value, EncodeError>,
) -> Result<Value, EncodeError> {
    let mut object = VObject::new();
    for entry in map.iter() {
        let (name, json) = match entry {
            ComponentRef::Typed { name, value, .. } => (name.to_owned(), write(value)?),
            ComponentRef::Custom(c) => (c.name.clone(), c.parameters.clone()),
            ComponentRef::Dynamic { name, json } | ComponentRef::Unknown { name, json } => {
                (name.to_owned(), json.clone())
            }
            _ => continue,
        };
        object.insert(name, json);
    }
    Ok(object.into())
}

/// The payload object: `description` and `components` (written by [`write_components`]); the
/// writer adds the document's unknown members after them.
///
/// # Errors
///
/// When the description has no JSON form.
pub fn payload_object<'f, D: Facet<'f>>(
    description: &D,
    components: Value,
) -> Result<VObject, EncodeError> {
    let mut object = VObject::new();
    object.insert("description", write_payload(description)?);
    object.insert("components", components);
    Ok(object)
}

/// The beta payload object: [`payload_object`], then `environment_attributes` when there are
/// any, before the unknown members.
///
/// # Errors
///
/// When the description or the attributes have no JSON form.
pub fn beta_object<'f, D: Facet<'f>>(
    description: &D,
    components: Value,
    attributes: Option<&EnvironmentAttributes>,
) -> Result<Value, EncodeError> {
    let mut object = payload_object(description, components)?;
    if let Some(attributes) = attributes {
        object.insert("environment_attributes", write_payload(attributes)?);
    }
    Ok(object.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BoolAttribute, BoolOperation};
    use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode};
    use bedrock_addon_core::json::to_json;

    /// A description of one member, as each family's is.
    #[derive(Facet)]
    struct Named {
        identifier: String,
    }

    /// A component set of one typed component.
    enum Set {
        Flag(bool),
    }

    impl ComponentSet for Set {
        fn name(&self) -> Option<&'static str> {
            Some("demo:flag")
        }
    }

    /// The beta object is `description`, `components`, `environment_attributes`, in that order;
    /// the components keep the map's order, unknown entries as read.
    #[test]
    fn the_beta_object_writes_its_members_in_the_games_order() {
        let mut map = ComponentMap::new();
        map.insert_unknown("demo:other", Value::from(3));
        map.insert_typed(Set::Flag(true), false);
        let attributes = EnvironmentAttributes {
            experimental_bool_attr: Some(BoolAttribute::new(true, BoolOperation::Nor)),
            ..Default::default()
        };
        let description = Named {
            identifier: "demo:marsh".to_owned(),
        };
        let write = |Set::Flag(flag): &Set| Ok(Value::from(*flag));
        let components = write_components(&map, write).unwrap();
        let written = beta_object(&description, components.clone(), Some(&attributes)).unwrap();
        assert_eq!(
            to_json(&written)
                .unwrap()
                .split_whitespace()
                .collect::<String>(),
            r#"{"description":{"identifier":"demo:marsh"},"components":{"demo:other":3,"demo:flag":true},"environment_attributes":{"minecraft:experimental_bool_attr":{"value":true,"operation":"nor"}}}"#
        );
        // Without attributes it is the stable payload object.
        let stable: Value = payload_object(&description, components.clone())
            .unwrap()
            .into();
        assert_eq!(beta_object(&description, components, None).unwrap(), stable);
    }

    /// A component's findings are re-rooted below its key; its unknown members go below the
    /// key only when it read.
    #[test]
    fn a_components_findings_and_unknown_members_go_below_its_key() {
        let at = JsonPath::of_keys(["minecraft:biome", "components", "demo:flag"]);
        let read = |value: Option<bool>| {
            move |found: &mut Diagnostics| {
                found.push(
                    Diagnostic::warn(DiagnosticCode::InvalidType { text: None })
                        .at(JsonPath::of_keys(["a"])),
                );
                found
                    .unknown_mut()
                    .put(&JsonPath::new(), "extra", Value::from(1));
                value
            }
        };
        let mut d = Diagnostics::default();
        assert_eq!(read_component(&at, &mut d, read(Some(true))), Some(true));
        let paths: Vec<String> = d.iter().map(|f| f.location.json_path.to_string()).collect();
        assert_eq!(paths, ["minecraft:biome/components/demo:flag/a"]);
        let kept: Vec<&str> = d.unknown().members_at(&at).map(|(key, _)| key).collect();
        assert_eq!(kept, ["extra"]);
        let mut failed = Diagnostics::default();
        assert_eq!(read_component(&at, &mut failed, read(None)), None);
        assert_eq!(failed.iter().count(), 1);
        assert_eq!(failed.unknown().members_at(&at).count(), 0);
    }
}
