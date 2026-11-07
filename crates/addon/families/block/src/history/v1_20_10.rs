//! Format 1.20.10: `geometry.bone_visibility` takes booleans and Molang.

use crate::codes;
use indexmap::IndexMap;

use super::legacy::{BlockPayload, component_path, string};
use super::ops::{dropped_field, edit_object};
use bedrock_addon_core::diagnostic::{Diagnostic, JsonPath, Phase};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{Legacy, LegacyStep, UpgradeCx};

/// A block document of `[1.20.10, 1.20.60)`.
pub type Block = Legacy<BlockPayload, 1, 20, 10>;

/// The full-block geometry a unit cube becomes.
pub(crate) const FULL_BLOCK: &str = "minecraft:geometry.full_block";

/// Below this original version the property spellings are converted
/// (`BLOCK_PROPERTY_RENAMED_TO_STATE_1_20_20`).
pub(crate) const PROPERTY_RENAME: FormatVersion = FormatVersion::new(1, 20, 20);

/// the unit cube becomes full-block geometry, then (gated on an
/// original version below 1.20.20) the game's conversion of `properties` into states.
impl LegacyStep<1, 20, 10> for BlockPayload {
    type Next = super::v1_20_60::Block;

    fn step(this: Block, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block = this.0;
        // Op 1, per section: `unit_cube` adds `geometry: full_block`; with a geometry of its
        // own, the geometry is replaced (the game logs an error and goes on, so this is a kept error).
        let mut both = Vec::new();
        for (section, map) in block.sections_mut() {
            if !map.contains_key("minecraft:unit_cube") {
                continue;
            }
            if map.contains_key("minecraft:geometry") {
                both.push(section);
            }
            map.insert("minecraft:geometry".to_owned(), Value::from(FULL_BLOCK));
        }
        // The game logs this at the content log's error level and goes on with the
        // full-block geometry: a kept error.
        for section in both {
            cx.diagnostics.push_kept_error(
                Diagnostic::of(&codes::UNIT_CUBE_AND_GEOMETRY)
                    .at(component_path(section, "minecraft:geometry"))
                    .phase(Phase::Upgrade),
            );
        }
        // Op 2.
        if cx.original_version < PROPERTY_RENAME {
            convert_properties(&mut block, cx.diagnostics);
        } else if block.description.shift_remove("properties").is_some() {
            let at = JsonPath::new().with(&["minecraft:block", "description", "properties"]);
            dropped_field(&at, cx.diagnostics);
        }
        Legacy(block)
    }
}

/// Converts `properties` into `states`; the game's conversion logs nothing. A `properties`
/// that is no object has no member to copy: it is discarded, and told (the crate's own
/// finding).
fn convert_properties(
    block: &mut BlockPayload,
    d: &mut bedrock_addon_core::diagnostic::Diagnostics,
) {
    let properties = block.description.shift_remove("properties");
    if properties.as_ref().is_some_and(|value| !value.is_object()) {
        let at = JsonPath::new().with(&["minecraft:block", "description", "properties"]);
        super::legacy::misread(&at, d);
    }
    if let Some(properties) = properties
        && let Some(properties) = super::legacy::object(&properties)
    {
        let description =
            |member: &str| JsonPath::new().with(&["minecraft:block", "description", member]);
        let had_states = block
            .description
            .get("states")
            .is_some_and(|states| !states.is_null());
        let mut states = block
            .description
            .get("states")
            .and_then(super::legacy::object)
            .unwrap_or_default();
        if !had_states {
            // What is told of `states` is told of the member the file has.
            d.moved(description("properties"), description("states"));
        }
        for (name, values) in properties {
            let mut to = description("states");
            to.push(&name);
            // A state of the file's own `states` that a property of the same name
            // replaces by another value (the merge keeps the property): noted.
            if states
                .get(&name)
                .is_some_and(|had| !had.is_null() && *had != values)
            {
                super::legacy::replaced(&to, d);
            }
            let mut from = description("properties");
            from.push(&name);
            d.moved(from, to);
            states.insert(name, values);
        }
        block
            .description
            .insert("states".to_owned(), super::to_object(&states));
    }
    let rename = |text: &str| text.replace("block_property", "block_state");
    for permutation in &mut block.permutations {
        if let Some(condition) = &mut permutation.condition
            && let Some(text) = string(condition)
        {
            *condition = Value::from(rename(text));
        }
    }
    for (_, map) in block.sections_mut() {
        let Some(geometry) = map.get_mut("minecraft:geometry") else {
            continue;
        };
        edit_object(geometry, |g| {
            let Some(bones) = g.get_mut("bone_visibility") else {
                return;
            };
            edit_object(bones, |bones| {
                for value in bones.values_mut() {
                    if let Some(text) = string(value) {
                        *value = Value::from(rename(text));
                    }
                }
            });
        });
    }
    if let Some(events) = &mut block.events {
        edit_object(events, |events| {
            for (name, event) in events.iter_mut() {
                let at = JsonPath::new().with(&["minecraft:block", "events", name]);
                rename_set_block_property(event, &at, d);
            }
        });
    }
}

/// In an event object (at `at`) and each entry of its `sequence`, the key
/// `set_block_property` becomes `set_block_state` (the value is not rewritten; `randomize` is
/// not visited).
fn rename_set_block_property(
    event: &mut Value,
    at: &JsonPath,
    d: &mut bedrock_addon_core::diagnostic::Diagnostics,
) {
    edit_object(event, |object| {
        rename(object, at, d);
        if let Some(sequence) = object.get_mut("sequence")
            && let Some(array) = sequence.as_array_mut()
        {
            for (index, entry) in array.iter_mut().enumerate() {
                let mut at = at.with(&["sequence"]);
                at.push_index(index);
                edit_object(entry, |entry| rename(entry, &at, d));
            }
        }
    });
}

/// The key rename in the object at `at`. A `set_block_state` the file has beside the
/// `set_block_property` it is made of, with another value, is written over as the game's
/// converter does: noted where the file has it.
fn rename(
    object: &mut IndexMap<String, Value>,
    at: &JsonPath,
    d: &mut bedrock_addon_core::diagnostic::Diagnostics,
) {
    const OLD: &str = "set_block_property";
    const NEW: &str = "set_block_state";
    let Some(value) = object.get(OLD) else {
        return;
    };
    if object
        .get(NEW)
        .is_some_and(|had| !super::legacy::same(had, value))
    {
        super::legacy::replaced(&at.with(&[NEW]), d);
    }
    super::legacy::rename_key(object, OLD, NEW, &mut |v| v);
    d.moved(at.with(&[OLD]), at.with(&[NEW]));
}
