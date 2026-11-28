//! The two 1.8.0-shaped documents, standard spawn rules and spawn groups: the game reads both
//! with one hand-written parser over the raw text, which differs only in the payload key, the
//! members it tells and the spawn-distance cap. Here the identifier is read first by hand (it
//! ends the file), the root's unknown members are told, and the payload is read by the derived
//! reader of its types under the game's jsoncpp accessors.

use facet::Facet;
use facet_value::{VArray, VObject};

use super::codes;
use super::condition::{self, Condition, OneOrMany, put_extra};
use super::read::{finding, finding_with};
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, DiagnosticDetail, Diagnostics, JsonPath, Severity, builders,
};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::json::kit::{self, Node, Profile, Recovery, UnknownAt};
use bedrock_addon_core::json::parse::{Options, parse_slice};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::molang::{
    Admission, EmptyRule, MolangFieldSpec, MolangVersionSource, MolangVisitor,
};
use bedrock_addon_core::schema::{
    self, ElementFailure, FromJson, Schema, UnknownRule, read_document, read_list,
};
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{MemberFailure, NullHandling, Reading, Scalars};
use bedrock_addon_shared_shapes::BlockDescriptor;

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
    pub fn profile(self) -> condition::Profile {
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

/// `description` of either payload.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Description {
    /// A string: [`read`] ends the file before the payload is read otherwise.
    identifier: String,
    population_control: Option<String>,
    extra: UnknownMembers,
}

/// The payload of spawn rules: its unknown members and its description's are told.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct RulesPayload {
    description: Description,
    #[facet(schema::reads_null, schema::or_kept = "told", schema::read_with = rules_conditions)]
    conditions: Option<Vec<Condition>>,
    extra: UnknownMembers,
}

/// The payload of spawn groups: its unknown members and its description's are kept silently.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "silent")]
struct GroupsPayload {
    #[facet(schema::unknown = "silent")]
    description: Description,
    #[facet(schema::reads_null, schema::or_kept = "told", schema::read_with = groups_conditions)]
    conditions: Option<Vec<Condition>>,
    extra: UnknownMembers,
}

/// `conditions` as an array: a condition that is no object is an `InvalidType` information and
/// skipped (this crate's choice; vanilla adds a default condition), what is told of a condition
/// from there on at the index the file has. Not an array: vanilla ends the parse with no
/// conditions and no message, and the member is kept as written.
fn conditions(node: &Node<'_>, read: &mut kit::Read<'_>) -> Option<Vec<Condition>> {
    if !node.value().is_array() {
        return None;
    }
    read_list(node, read, ElementFailure::Drop, Condition::from_json)
}

/// [`conditions`] of spawn rules: one that is no array (nor `null`) is an `InvalidType`
/// information, the crate's.
fn rules_conditions(node: &Node<'_>, read: &mut kit::Read<'_>) -> Option<Vec<Condition>> {
    if !node.value().is_array() && !node.value().is_null() {
        read.wrong_type(node);
    }
    conditions(node, read)
}

/// [`conditions`] of spawn groups: one that is no array gets the `EmptyDocument` warning of the
/// group's validate, nothing here.
fn groups_conditions(node: &Node<'_>, read: &mut kit::Read<'_>) -> Option<Vec<Condition>> {
    conditions(node, read)
}

/// The parse over the whole document (the raw-text path: the envelope is never checked, a third
/// root member is kept): the identifier first, which ends the file when it is missing, then the
/// root's unknown members, then the payload by the derived reader under the game's jsoncpp
/// accessors (`Scalars::Accessors`, each coercion an information), a `null` member absent, a
/// member that fails kept as written.
pub fn read(document: JsonSlice<'_>, flavor: Flavor, d: &mut Diagnostics) -> Option<Read> {
    let parsed = match parse_slice(document, Options::default()) {
        Ok(parsed) => parsed,
        Err(error) => {
            finding_with(
                Diagnostic::of(builders::syntax(error.kind.to_string())),
                &JsonPath::new(),
                d,
            );
            return None;
        }
    };
    let Some(root) = parsed.value.as_object() else {
        finding_with(
            Diagnostic::of(&codes::NOT_AN_OBJECT).detail(DiagnosticDetail::Syntax(
                "the root is not an object".to_owned(),
            )),
            &JsonPath::new(),
            d,
        );
        return None;
    };
    let key = flavor.key();
    let payload = root.get(key);
    // The identifier is read first, through the const `operator[]`: a missing payload, a
    // non-object payload or description, a `null` or non-string identifier all end here.
    let identifier = payload
        .and_then(Value::as_object)
        .and_then(|p| p.get("description"))
        .and_then(Value::as_object)
        .and_then(|o| o.get("identifier"))
        .and_then(Value::as_string)
        .map(|s| s.as_str().to_owned());
    let Some(identifier) = identifier else {
        if flavor == Flavor::Groups && payload.is_none() {
            // The payload key is absent, with vanilla's text.
            finding(
                Code::from(&codes::MISSING_IDENTIFIER_PAYLOAD),
                Severity::Error,
                &JsonPath::new(),
                d,
            );
        } else {
            let mut path = JsonPath::new().child(key).child("description");
            path.push("identifier");
            finding(
                Code::from(&codes::MISSING_IDENTIFIER),
                Severity::Error,
                &path,
                d,
            );
        }
        return None;
    };
    let reading = Reading::OWN
        .with_scalars(Scalars::Accessors(Severity::Info))
        .with_null(NullHandling::AbsentSilently)
        .with_on_member_failure(MemberFailure::DefaultAndTell);
    let mut read = kit::Read::new(reading, d)
        .profile(
            Profile::DECLARED_ORDER
                .with_unknown(UnknownAt::Last)
                .with_recovery(Recovery::Value),
        )
        .unknown_severity(Severity::Info)
        // What a struct kept, among its unknown members where the file has it, as the hand
        // parsers collected them.
        .keep_in_file_order()
        // The condition texts name the identifier.
        .subject(&identifier);
    let root_node = Node::parsed(&parsed, JsonPath::new());
    let members = read.object(&root_node)?;
    // The crate's own information for spawn rules (vanilla never looks); kept silently for
    // spawn groups.
    let rule = match flavor {
        Flavor::Rules => UnknownRule::Keep,
        Flavor::Groups => UnknownRule::Silent,
    };
    let root_extra = read
        .unknown(&members, &["format_version", key], rule)
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.clone()))
        .collect();
    let payload = members.get(key)?;
    let (description, conditions, extra) = match flavor {
        Flavor::Rules => {
            let payload: RulesPayload = read_document(&payload, &mut read)?;
            (payload.description, payload.conditions, payload.extra)
        }
        Flavor::Groups => {
            let payload: GroupsPayload = read_document(&payload, &mut read)?;
            (payload.description, payload.conditions, payload.extra)
        }
    };
    Some(Read {
        identifier,
        population_control: description.population_control,
        description_extra: description.extra,
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

/// The Molang of a condition: the `tags` of its block descriptors.
pub(crate) fn visit_condition(condition: &mut Condition, visitor: &mut MolangVisitor<'_>) {
    if let Some(list) = &mut condition.spawns_on_block_filter {
        visit_list(condition::KEYS[2], list, visitor);
    }
    if let Some(list) = &mut condition.spawns_on_block_prevented_filter {
        visit_list(condition::KEYS[3], list, visitor);
    }
    if let Some(above) = &mut condition.spawns_above_block_filter
        && let Some(list) = &mut above.blocks
    {
        visitor.enter(condition::KEYS[4].into());
        visit_list("blocks", list, visitor);
        visitor.leave();
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
