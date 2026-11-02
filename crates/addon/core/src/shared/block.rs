//! `BlockDescriptor`, `BlockSpecifier`, `StateValue`: blocks named by a document.

use facet_value::VObject;
use indexmap::IndexMap;
use molangx::json::MolangSource;

use super::read::{
    Failure, ROOT, ShapeError, ShapeErrorKind, compact, fail, jsoncpp_type, molang_source,
    molang_value, wrong_type,
};
use super::{finding, unrecognized};
use crate::diagnostic::{DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText};
use crate::document::Extra;
use crate::id::BlockId;
use crate::json::Value;

/// A block, a block with states, or a tag expression, as the game's block descriptor.
///
/// JSON: `"ns:name"`, `{"name": "ns:name", "states": {"<state>": bool | int | string}}` or
/// `{"tags": "<molang>"}`. A list member takes one descriptor or an array of them (the using
/// family's type). Shapes vanilla's readers log and keep are kept here too, so writing
/// reproduces them: a name or states beside `tags` ([`BlockDescriptor::Mixed`]; vanilla uses the
/// tags) and any other shape ([`BlockDescriptor::Invalid`]; vanilla keeps an empty descriptor
/// that matches nothing). [`BlockDescriptor::check`] reports both.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
#[non_exhaustive]
#[doc(alias = "block_descriptor")]
pub enum BlockDescriptor {
    /// `"ns:name"` or `{"name": …, "states"?: …}`.
    Block {
        /// The block (JSON `name`).
        name: BlockId,
        /// The required state values, in document order (JSON `states`).
        states: IndexMap<String, StateValue>,
        /// Read from the object form.
        object: bool,
        /// Unknown members of the object form.
        extra: Extra,
    },
    /// `{"tags": "<molang>"}`: evaluated against the tested block with `query.any_tag` /
    /// `query.all_tags`. Its Molang version is the using member's.
    Tags {
        /// The expression (JSON `tags`).
        tags: MolangSource,
        /// Unknown members.
        extra: Extra,
    },
    /// Both a name (or states) and `tags`: vanilla logs it and uses the tags.
    Mixed {
        /// The block (JSON `name`), when written.
        name: Option<BlockId>,
        /// The states (JSON `states`).
        states: IndexMap<String, StateValue>,
        /// The expression (JSON `tags`), the one vanilla uses.
        tags: MolangSource,
        /// Unknown members.
        extra: Extra,
    },
    /// A shape vanilla's JSON reader rejects and keeps as an empty descriptor (matches nothing),
    /// as written.
    Invalid(Value),
}

/// One block state value. JSON: a boolean, an integer or a string, kept apart as vanilla's
/// `BlockDescriptor::Compound` keeps them (`"1"` is a string, `1` an integer); a real number,
/// `null`, an array or an object is not a state value.
#[derive(Clone, Debug, PartialEq, Eq, Hash, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
#[non_exhaustive]
pub enum StateValue {
    /// A boolean state.
    Bool(bool),
    /// An integer state.
    Int(i32),
    /// A string (enum) state.
    String(String),
}

/// A block with optional states and no tag form: the game's block specifier and its
/// older schema tree's `block_reference` node.
/// JSON: `"ns:name"` or `{"name": "ns:name", "states"?: {…}}`; any other shape is a read error.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[doc(alias = "block_specifier")]
pub struct BlockSpecifier {
    /// The block (JSON `name`).
    pub name: BlockId,
    /// The required state values, in document order (JSON `states`).
    pub states: IndexMap<String, StateValue>,
    /// Read from (and written as) the object form.
    pub object: bool,
    /// Unknown members of the object form.
    pub extra: Extra,
}

/// Why a block object is not a descriptor: the three texts of vanilla's JSON reader.
enum Rejected {
    /// "`BlockDescriptor` requires a string, block object, or `any_tag` object."
    Shape,
    /// "Only strings are valid for name."
    Name(Value),
    /// "Invalid state type during block parsing", for this state.
    State(String, Value),
}

/// The parts of a block object: `name`, `states`, `tags` and the other members.
struct Parts {
    name: Option<BlockId>,
    states: Option<IndexMap<String, StateValue>>,
    tags: Option<MolangSource>,
    extra: Extra,
}

impl Parts {
    /// Splits a block object, in the order vanilla's reader checks it: `name`, then each state,
    /// then `tags`.
    fn read(object: &VObject) -> Result<Self, Rejected> {
        let name = match object.get("name") {
            None => None,
            Some(name) => Some(
                name.as_string()
                    .map(|n| BlockId::parse(n.as_str()))
                    .ok_or_else(|| Rejected::Name(name.clone()))?,
            ),
        };
        let states = match object.get("states") {
            None => None,
            Some(states) => {
                let states = states.as_object().ok_or(Rejected::Shape)?;
                let mut out = IndexMap::new();
                for (state, value) in states {
                    let value = StateValue::read(value)
                        .ok_or_else(|| Rejected::State(state.as_str().to_owned(), value.clone()))?;
                    out.insert(state.as_str().to_owned(), value);
                }
                Some(out)
            }
        };
        let tags = match object.get("tags") {
            None => None,
            Some(tags) => Some(molang_source(tags).ok_or(Rejected::Shape)?),
        };
        let mut extra = Extra::default();
        for (key, value) in object {
            if !matches!(key.as_str(), "name" | "states" | "tags") {
                extra.insert(key.as_str().to_owned(), value.clone());
            }
        }
        Ok(Self {
            name,
            states,
            tags,
            extra,
        })
    }
}

impl BlockDescriptor {
    /// One block without states, written as a string.
    pub fn block(name: BlockId) -> Self {
        Self::Block {
            name,
            states: IndexMap::new(),
            object: false,
            extra: Extra::default(),
        }
    }

    /// The findings vanilla's readers log for this descriptor, located under `at`: a
    /// [`BlockDescriptor::Mixed`] (the cereal reader's text) or a [`BlockDescriptor::Invalid`]
    /// (the JSON reader's text) is an `InvalidValue` kept error; each member kept in `extra` is
    /// an `UnrecognizedField` warning at its own path (cereal's `0x4` under the unstrict mask).
    pub fn check(&self, at: &JsonPath) -> Diagnostics {
        let mut out = Diagnostics::default();
        let text = match self {
            Self::Block { extra, .. } | Self::Tags { extra, .. } => {
                unrecognized(extra, at, &mut out);
                return out;
            }
            // The game's order for its two tests is not known; the crate tests states first.
            Self::Mixed {
                name,
                states,
                extra,
                ..
            } => {
                unrecognized(extra, at, &mut out);
                let block = name.as_ref().map_or("", BlockId::as_str).to_owned();
                if states.is_empty() {
                    VanillaText::BlockDescriptorNameAndTags { block }
                } else {
                    VanillaText::BlockDescriptorTagsAndStates { block }
                }
            }
            Self::Invalid(value) => invalid_text(value),
        };
        // Level 3 and the read goes on, a kept error: the game's descriptor proxy logs the two
        // texts and goes on with the tag expression; the block descriptor reader logs its own
        // and builds an empty descriptor.
        out.push_kept_error(finding(
            DiagnosticCode::InvalidValue { text: Some(text) },
            Severity::Error,
            at,
        ));
        out
    }

    /// Reads the object form.
    fn read_object(object: &VObject) -> Result<Self, Rejected> {
        let Parts {
            name,
            states,
            tags,
            extra,
        } = Parts::read(object)?;
        let has_states = states.is_some();
        let states = states.unwrap_or_default();
        match (name, tags) {
            (name, Some(tags)) if name.is_some() || has_states => Ok(Self::Mixed {
                name,
                states,
                tags,
                extra,
            }),
            (_, Some(tags)) => Ok(Self::Tags { tags, extra }),
            (Some(name), None) => Ok(Self::Block {
                name,
                states,
                object: true,
                extra,
            }),
            (None, None) => Err(Rejected::Shape),
        }
    }
}

/// The JSON reader's text for a value it rejects.
fn invalid_text(value: &Value) -> VanillaText {
    let rejected = match value.as_object() {
        Some(object) => BlockDescriptor::read_object(object).err(),
        None => None,
    };
    match rejected {
        Some(Rejected::Name(name)) => VanillaText::BlockDescriptorName {
            value: compact(&name),
        },
        Some(Rejected::State(state, state_value)) => VanillaText::BlockDescriptorStateType {
            block: value
                .as_object()
                .and_then(|o| o.get("name"))
                .and_then(Value::as_string)
                .map(|n| n.as_str().to_owned())
                .unwrap_or_default(),
            state,
            state_type: jsoncpp_type(&state_value),
        },
        Some(Rejected::Shape) | None => VanillaText::BlockDescriptorShape {
            value: compact(value),
        },
    }
}

impl From<Value> for BlockDescriptor {
    fn from(value: Value) -> Self {
        if let Some(name) = value.as_string() {
            return Self::block(BlockId::parse(name.as_str()));
        }
        value
            .as_object()
            .and_then(|object| Self::read_object(object).ok())
            .unwrap_or(Self::Invalid(value))
    }
}

impl From<&BlockDescriptor> for Value {
    fn from(descriptor: &BlockDescriptor) -> Self {
        let mut object = VObject::new();
        let extra = match descriptor {
            BlockDescriptor::Invalid(value) => return value.clone(),
            BlockDescriptor::Block {
                name,
                states,
                object: false,
                extra,
            } if states.is_empty() && extra.is_empty() => return Value::from(name.as_str()),
            BlockDescriptor::Block {
                name,
                states,
                extra,
                ..
            } => {
                object.insert("name", Value::from(name.as_str()));
                insert_states(&mut object, states);
                extra
            }
            BlockDescriptor::Tags { tags, extra } => {
                object.insert("tags", molang_value(tags));
                extra
            }
            BlockDescriptor::Mixed {
                name,
                states,
                tags,
                extra,
            } => {
                if let Some(name) = name {
                    object.insert("name", Value::from(name.as_str()));
                }
                insert_states(&mut object, states);
                object.insert("tags", molang_value(tags));
                extra
            }
        };
        insert_extra(&mut object, extra);
        Value::from(object)
    }
}

/// Writes `states` when there are any.
fn insert_states(object: &mut VObject, states: &IndexMap<String, StateValue>) {
    if !states.is_empty() {
        let mut map = VObject::new();
        for (state, value) in states {
            map.insert(state.as_str(), Value::from(value));
        }
        object.insert("states", Value::from(map));
    }
}

/// Writes the unknown members after the known ones.
fn insert_extra(object: &mut VObject, extra: &Extra) {
    for (key, value) in extra.iter() {
        object.insert(key, value.clone());
    }
}

impl StateValue {
    /// Reads a state value; `None` for a JSON type vanilla's reader rejects.
    fn read(value: &Value) -> Option<Self> {
        if let Some(b) = value.as_bool() {
            Some(Self::Bool(b))
        } else if let Some(s) = value.as_string() {
            Some(Self::String(s.as_str().to_owned()))
        } else {
            super::read::int(ROOT, value).ok().map(Self::Int)
        }
    }
}

impl TryFrom<Value> for StateValue {
    type Error = ShapeError;

    fn try_from(value: Value) -> Result<Self, ShapeError> {
        Self::read(&value).ok_or_else(|| {
            match value.as_number() {
                Some(n) if !n.is_float() => Failure::new(ShapeErrorKind::OutOfRange),
                _ => wrong_type(ROOT, "a boolean, an integer or a string", &value),
            }
            .into()
        })
    }
}

impl From<&StateValue> for Value {
    fn from(state: &StateValue) -> Self {
        match state {
            StateValue::Bool(b) => Value::from(*b),
            StateValue::Int(i) => Value::from(i64::from(*i)),
            StateValue::String(s) => Value::from(s.as_str()),
        }
    }
}

impl BlockSpecifier {
    /// One block without states, written as a string.
    pub fn new(name: BlockId) -> Self {
        Self {
            name,
            states: IndexMap::new(),
            object: false,
            extra: Extra::default(),
        }
    }
}

impl BlockSpecifier {
    /// The findings for this specifier, located under `at`: each member kept in `extra` is an
    /// `UnrecognizedField` warning at its own path.
    pub fn check(&self, at: &JsonPath) -> Diagnostics {
        let mut out = Diagnostics::default();
        unrecognized(&self.extra, at, &mut out);
        out
    }
}

impl TryFrom<Value> for BlockSpecifier {
    type Error = ShapeError;

    fn try_from(value: Value) -> Result<Self, ShapeError> {
        if let Some(name) = value.as_string() {
            return Ok(Self::new(BlockId::parse(name.as_str())));
        }
        let object = value
            .as_object()
            .ok_or_else(|| wrong_type(ROOT, "a string or an object", &value))?;
        let parts = Parts::read(object).map_err(|rejected| match rejected {
            Rejected::Name(name) => wrong_type("name", "a string", &name),
            Rejected::State(name, state) => {
                wrong_type(ROOT, "a boolean, an integer or a string", &state)
                    .key(&name)
                    .key("states")
            }
            Rejected::Shape => match object.get("states") {
                Some(states) if !states.is_object() => wrong_type("states", "an object", states),
                _ => fail("tags", ShapeErrorKind::UnknownName),
            },
        })?;
        if parts.tags.is_some() {
            // A specifier has no tag form; `tags` is not one of its members.
            return Err(fail("tags", ShapeErrorKind::UnknownName).into());
        }
        Ok(Self {
            name: parts
                .name
                .ok_or_else(|| Failure::new(ShapeErrorKind::MissingMember { member: "name" }))?,
            states: parts.states.unwrap_or_default(),
            object: true,
            extra: parts.extra,
        })
    }
}

impl From<&BlockSpecifier> for Value {
    fn from(specifier: &BlockSpecifier) -> Self {
        if !specifier.object && specifier.states.is_empty() && specifier.extra.is_empty() {
            return Value::from(specifier.name.as_str());
        }
        let mut object = VObject::new();
        object.insert("name", Value::from(specifier.name.as_str()));
        insert_states(&mut object, &specifier.states);
        insert_extra(&mut object, &specifier.extra);
        Value::from(object)
    }
}
