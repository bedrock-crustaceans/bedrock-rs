//! The two 1.8.0-shaped documents, standard spawn rules and spawn groups: the game reads both
//! with one hand-written parser over the raw text, which differs only in the payload key and
//! the spawn-distance cap.

use facet_value::{VArray, VObject};

use super::condition::{
    self, Condition, OneOrMany, Profile, ReadCx, members, put_extra, read_condition,
};
use super::read::{self, child, element, finding, wrong_type};
use super::texts;
use bedrock_addon_core::diagnostic::{
    DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::molang::{
    Admission, EmptyRule, MolangFieldSpec, MolangFields, MolangVersionSource, MolangVisitor,
};
use bedrock_addon_core::shared::BlockDescriptor;
use bedrock_addon_core::version::FormatVersion;

/// The payload key of spawn rules.
pub const RULES_KEY: &str = "minecraft:spawn_rules";
/// The payload key of spawn groups.
pub const GROUPS_KEY: &str = "minecraft:spawn_groups";
/// The one struct of both legacy chains.
pub const V1_8_0: FormatVersion = FormatVersion::new(1, 8, 0);

/// Which of the two parsers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flavor {
    /// The spawn-rules parser: unknown members are warned.
    Rules,
    /// The spawn-groups parser: unknown root, payload and description members are ignored
    /// silently; a missing payload is `MissingPayload`.
    Groups,
}

impl Flavor {
    /// The key the flavor's payload lies under.
    pub fn key(self) -> &'static str {
        match self {
            Self::Rules => RULES_KEY,
            Self::Groups => GROUPS_KEY,
        }
    }

    /// The condition profile the flavor's conditions are checked against.
    pub fn profile(self) -> Profile {
        match self {
            Self::Rules => condition::SPAWN_RULES,
            Self::Groups => condition::SPAWN_GROUPS,
        }
    }
}

/// What both parsers read.
#[derive(Clone, Debug, PartialEq)]
pub struct Read {
    /// The identifier the description names.
    pub identifier: String,
    /// The description's `population_control`, when present.
    pub population_control: Option<String>,
    /// The description's members that are not read by name.
    pub description_extra: UnknownMembers,
    /// The `conditions` list, when present.
    pub conditions: Option<Vec<Condition>>,
    /// The members of the payload that are not read by name.
    pub extra: UnknownMembers,
    /// The members of the root that are not read by name.
    pub root_extra: UnknownMembers,
    /// The root has a `format_version` member.
    pub has_version: bool,
}

/// The members of `object` other than `known`: warned (spawn rules) or kept silently (spawn
/// groups).
fn others(
    object: &VObject,
    known: &[&str],
    kept: &[&str],
    at: &JsonPath,
    flavor: Flavor,
    d: &mut Diagnostics,
) -> UnknownMembers {
    match flavor {
        Flavor::Rules => members(object, known, kept, at, d),
        Flavor::Groups => {
            let mut extra = UnknownMembers::default();
            for (key, value) in object {
                if kept.contains(&key.as_str()) || !known.contains(&key.as_str()) {
                    extra.insert(key.as_str().to_owned(), value.clone());
                }
            }
            extra
        }
    }
}

/// The parse over the whole document (the raw-text path: the envelope is never
/// checked, a third root member is kept).
#[allow(clippy::too_many_lines, reason = "vanilla's parser, step by step")]
pub fn read(document: JsonSlice<'_>, flavor: Flavor, d: &mut Diagnostics) -> Option<Read> {
    let root = match bedrock_addon_core::json::from_str_jsonc::<Value>(document.text) {
        Ok(root) => root,
        Err(error) => {
            finding(
                DiagnosticCode::MalformedInput {
                    message: error.to_string(),
                    text: None,
                },
                Severity::Error,
                &JsonPath::new(),
                d,
            );
            return None;
        }
    };
    let Some(root) = root.as_object() else {
        finding(
            DiagnosticCode::MalformedInput {
                message: "the root is not an object".to_owned(),
                text: Some(VanillaText::Fixed(texts::NOT_AN_OBJECT)),
            },
            Severity::Error,
            &JsonPath::new(),
            d,
        );
        return None;
    };
    let key = flavor.key();
    let mut at = JsonPath::new();
    at.push(key);
    let payload = root.get(key);
    // The identifier is read first, through the const `operator[]`: a missing payload, a
    // non-object payload or description, a `null` or non-string identifier all end here.
    let description = payload
        .and_then(Value::as_object)
        .and_then(|p| p.get("description"));
    let identifier = description
        .and_then(Value::as_object)
        .and_then(|o| o.get("identifier"))
        .and_then(Value::as_string)
        .map(|s| s.as_str().to_owned());
    let Some(identifier) = identifier else {
        let text = Some(VanillaText::Fixed(texts::MISSING_IDENTIFIER));
        if flavor == Flavor::Groups && payload.is_none() {
            // The payload key is absent, with vanilla's text.
            finding(
                DiagnosticCode::MissingPayload { text },
                Severity::Error,
                &JsonPath::new(),
                d,
            );
        } else {
            let mut path = child(&at, "description");
            path.push("identifier");
            finding(
                DiagnosticCode::MissingRequiredField { text },
                Severity::Error,
                &path,
                d,
            );
        }
        return None;
    };
    let root_extra = others(
        root,
        &["format_version", key],
        &[],
        &JsonPath::new(),
        flavor,
        d,
    );
    let payload = payload.and_then(Value::as_object)?;
    let description = description.and_then(Value::as_object)?;
    let description_at = child(&at, "description");
    let population_control = read::string(
        description.get("population_control"),
        &child(&description_at, "population_control"),
        d,
    );
    let description_extra = others(
        description,
        &["identifier", "population_control"],
        &[],
        &description_at,
        flavor,
        d,
    );
    let conditions_at = child(&at, "conditions");
    let conditions = match payload.get("conditions") {
        None => None,
        Some(value) => {
            if let Some(array) = value.as_array() {
                let mut cx = ReadCx {
                    identifier: &identifier,
                    profile: flavor.profile(),
                    d,
                };
                let mut list = Vec::with_capacity(array.len());
                let mut places = Vec::with_capacity(array.len());
                for (i, entry) in array.iter().enumerate() {
                    let read = read_condition(entry, &element(&conditions_at, i), &mut cx);
                    if let Some(condition) = read {
                        list.push(condition);
                        places.push(i);
                    }
                }
                // What is told of a condition from here on is at the index the file has.
                d.left_out(&conditions_at, &places);
                Some(list)
            } else {
                // Not an array: vanilla ends the parse with no conditions and no message. For
                // spawn rules the crate warns; a spawn group gets the `EmptyDocument` warning.
                if flavor == Flavor::Rules && !value.is_null() {
                    wrong_type(&conditions_at, None, d);
                }
                None
            }
        }
    };
    // A `conditions` that is not an array is kept as written, so write-back loses nothing.
    let kept: &[&str] = if conditions.is_none() && payload.contains_key("conditions") {
        &["conditions"]
    } else {
        &[]
    };
    let extra = others(
        payload,
        &["description", "conditions"],
        kept,
        &at,
        flavor,
        d,
    );
    Some(Read {
        identifier,
        population_control,
        description_extra,
        conditions,
        extra,
        root_extra,
        has_version: root.contains_key("format_version"),
    })
}

/// The document as JSON: `format_version`, the payload, the other root members.
pub fn write(
    format_version: Option<FormatVersion>,
    key: &str,
    description: VObject,
    conditions: Option<&[Condition]>,
    extra: &UnknownMembers,
    root_extra: &UnknownMembers,
) -> Value {
    let mut payload = VObject::new();
    payload.insert("description", Value::from(description));
    if let Some(conditions) = conditions {
        let list: VArray = conditions.iter().map(Condition::to_value).collect();
        payload.insert("conditions", Value::from(list));
    }
    put_extra(&mut payload, extra);
    let mut root = VObject::new();
    if let Some(version) = format_version {
        root.insert("format_version", Value::from(version.to_string()));
    }
    root.insert(key, Value::from(payload));
    put_extra(&mut root, root_extra);
    Value::from(root)
}

/// The description object from its identifier, its population control and the members kept as
/// written.
pub fn description_value(
    identifier: &str,
    population_control: Option<&str>,
    extra: &UnknownMembers,
) -> VObject {
    let mut description = VObject::new();
    description.insert("identifier", Value::from(identifier));
    if let Some(pool) = population_control {
        description.insert("population_control", Value::from(pool));
    }
    put_extra(&mut description, extra);
    description
}

// ---- Molang ------------------------------------------------------------------------------

/// A block descriptor's `tags` in a legacy condition: the document's `format_version` mapped to a
/// Molang version, only the tag queries, an empty text compiled (rejected from Molang 4), a
/// rejection kept: the parser's lines are level 3 and the descriptor is kept, so a rejected
/// `tags` is a kept error: the game builds the descriptor from the failed parse and carries on.
pub const TAGS: MolangFieldSpec = MolangFieldSpec {
    version: MolangVersionSource::FormatVersion,
    admission: Admission::Tags,
    empty: EmptyRule::Compile,
    error_severity: Severity::Error,
    error_kept: true,
    ..MolangFieldSpec::DEFAULT
};

fn visit_descriptor(descriptor: &mut BlockDescriptor, visitor: &mut MolangVisitor<'_>) {
    if let BlockDescriptor::Tags { tags, .. } | BlockDescriptor::Mixed { tags, .. } = descriptor {
        visitor.field("tags", &TAGS, tags);
    }
}

fn visit_list(key: &str, list: &mut OneOrMany<BlockDescriptor>, visitor: &mut MolangVisitor<'_>) {
    visitor.enter(key.into());
    match list {
        OneOrMany::One(descriptor) => visit_descriptor(descriptor, visitor),
        OneOrMany::Many(descriptors) => {
            for (i, descriptor) in descriptors.iter_mut().enumerate() {
                visitor.enter(i.into());
                visit_descriptor(descriptor, visitor);
                visitor.leave();
            }
        }
    }
    visitor.leave();
}

impl MolangFields for Condition {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Some(list) = &mut self.spawns_on_block_filter {
            visit_list(condition::KEYS[2], list, visitor);
        }
        if let Some(list) = &mut self.spawns_on_block_prevented_filter {
            visit_list(condition::KEYS[3], list, visitor);
        }
        if let Some(above) = &mut self.spawns_above_block_filter
            && let Some(list) = &mut above.blocks
        {
            visitor.enter(condition::KEYS[4].into());
            visit_list("blocks", list, visitor);
            visitor.leave();
        }
    }
}

/// Visit the Molang of the `conditions` under `key`.
pub fn visit_conditions(
    key: &str,
    conditions: &mut Option<Vec<Condition>>,
    visitor: &mut MolangVisitor<'_>,
) {
    if let Some(conditions) = conditions {
        visitor.enter(key.into());
        visitor.nested("conditions", conditions);
        visitor.leave();
    }
}
