//! The two list entries that are a name or a `{ name: condition }` object: `render_controllers[]`
//! ([`ControllerUse`]) and `scripts.animate[]` ([`AnimateEntry`]).

use indexmap::IndexMap;

use crate::id::RenderControllerId;
use crate::json::Value;
use crate::molang::Molang;

/// One entry of `render_controllers`: a render controller, drawn always or under a condition.
///
/// Written as the controller's id (`"controller.render.pig"`, always drawn) or as an object
/// `{ "controller.render.pig": "query.is_baby" }` whose value is the Molang condition (a
/// number or a boolean also works). Vanilla reads only one member of the object: the one whose
/// key sorts first byte by byte, not the first written; [`controller`](Self::controller) and
/// [`condition`](Self::condition) give that member, [`ignored`](Self::ignored) the rest. An
/// empty object is kept as written and names no controller: the game uses nothing of such an
/// entry (the entry is skipped), and the loaders warn of it where the file has it.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(proxy = NamedConditionProxy)]
#[repr(u8)]
pub enum ControllerUse {
    /// The string form: the controller is always drawn.
    Always(RenderControllerId),
    /// The object form, every member as written.
    Conditional(IndexMap<String, Molang>),
}

impl ControllerUse {
    /// The render controller this entry draws; `None` for an empty object.
    #[must_use]
    pub fn controller(&self) -> Option<RenderControllerId> {
        match self {
            Self::Always(id) => Some(id.clone()),
            Self::Conditional(members) => {
                first_member(members).map(|(name, _)| RenderControllerId::parse(name.as_str()))
            }
        }
    }

    /// The condition it is drawn under; `None` when it is always drawn (or names nothing).
    #[must_use]
    pub fn condition(&self) -> Option<&Molang> {
        match self {
            Self::Always(_) => None,
            Self::Conditional(members) => first_member(members).map(|(_, condition)| condition),
        }
    }

    /// The members of the object form vanilla does not read, in written order.
    pub fn ignored(&self) -> impl Iterator<Item = &str> {
        let members = match self {
            Self::Always(_) => None,
            Self::Conditional(members) => Some(members),
        };
        ignored_members(members)
    }
}

/// One entry of `scripts.animate`: an animation or animation controller (by its short name in
/// `animations`), played always or under a condition.
///
/// Written as the short name (`"walk"`) or as an object `{ "walk": "query.is_moving" }`, read as
/// [`ControllerUse`] is: only the member whose key sorts first byte by byte counts, and an
/// empty object is kept as written, names nothing and is warned of.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(proxy = NamedConditionProxy)]
#[repr(u8)]
pub enum AnimateEntry {
    /// The string form: always played.
    Always(String),
    /// The object form, every member as written.
    Conditional(IndexMap<String, Molang>),
}

impl AnimateEntry {
    /// The short name this entry plays; `None` for an empty object.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Always(name) => Some(name),
            Self::Conditional(members) => first_member(members).map(|(name, _)| name.as_str()),
        }
    }

    /// The condition it is played under; `None` when it is always played (or names nothing).
    #[must_use]
    pub fn condition(&self) -> Option<&Molang> {
        match self {
            Self::Always(_) => None,
            Self::Conditional(members) => first_member(members).map(|(_, condition)| condition),
        }
    }

    /// The members of the object form vanilla does not read, in written order.
    pub fn ignored(&self) -> impl Iterator<Item = &str> {
        let members = match self {
            Self::Always(_) => None,
            Self::Conditional(members) => Some(members),
        };
        ignored_members(members)
    }
}

// The member vanilla reads: its JSON objects are ordered maps, so the first member is the key
// that sorts first by bytes.
fn first_member(members: &IndexMap<String, Molang>) -> Option<(&String, &Molang)> {
    members
        .iter()
        .min_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()))
}

fn ignored_members(members: Option<&IndexMap<String, Molang>>) -> impl Iterator<Item = &str> {
    let read = members.and_then(first_member).map(|(name, _)| name);
    members
        .into_iter()
        .flat_map(IndexMap::keys)
        .filter(move |name| Some(*name) != read)
        .map(String::as_str)
}

/// Every JSON shape a deserializer can offer, so the conversion rejects the ones a name-or-object
/// entry does not take (the untagged solver would otherwise coerce a number into a string).
#[derive(Debug, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(
    dead_code,
    reason = "the payloads of the rejected shapes are never read"
)]
pub enum NamedConditionProxy {
    /// `null`.
    Null,
    /// A string: the name.
    String(String),
    /// A boolean.
    Bool(bool),
    /// A non-negative integer.
    UInt(u64),
    /// A negative integer.
    Int(i64),
    /// Any other number.
    Number(f64),
    /// An array.
    Array(Vec<Value>),
    /// An object: name → condition.
    Object(IndexMap<String, Molang>),
}

/// Why a JSON value is not a name-or-object entry.
#[derive(Debug, thiserror::Error)]
#[error("a list entry must be a string or an object of conditions")]
pub struct NotANamedCondition;

impl TryFrom<NamedConditionProxy> for ControllerUse {
    type Error = NotANamedCondition;

    fn try_from(proxy: NamedConditionProxy) -> Result<Self, NotANamedCondition> {
        match proxy {
            NamedConditionProxy::String(name) => Ok(Self::Always(RenderControllerId::parse(name))),
            NamedConditionProxy::Object(members) => Ok(Self::Conditional(members)),
            _ => Err(NotANamedCondition),
        }
    }
}

impl From<&ControllerUse> for NamedConditionProxy {
    fn from(entry: &ControllerUse) -> Self {
        match entry {
            ControllerUse::Always(id) => Self::String(id.as_str().to_owned()),
            ControllerUse::Conditional(members) => Self::Object(members.clone()),
        }
    }
}

impl TryFrom<NamedConditionProxy> for AnimateEntry {
    type Error = NotANamedCondition;

    fn try_from(proxy: NamedConditionProxy) -> Result<Self, NotANamedCondition> {
        match proxy {
            NamedConditionProxy::String(name) => Ok(Self::Always(name)),
            NamedConditionProxy::Object(members) => Ok(Self::Conditional(members)),
            _ => Err(NotANamedCondition),
        }
    }
}

impl From<&AnimateEntry> for NamedConditionProxy {
    fn from(entry: &AnimateEntry) -> Self {
        match entry {
            AnimateEntry::Always(name) => Self::String(name.clone()),
            AnimateEntry::Conditional(members) => Self::Object(members.clone()),
        }
    }
}
