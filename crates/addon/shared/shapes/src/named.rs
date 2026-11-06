//! [`NamedCondition`]: a list entry or member that is a name, or a `{ name: condition }`
//! object of which the game reads one member. `render_controllers[]` ([`ControllerUse`]) and
//! `scripts.animate[]` ([`AnimateEntry`]) are two of them.

use bedrock_addon_core::id::RenderControllerId;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{Molang, MolangFields, MolangSlot, MolangVisitor};
use bedrock_addon_core::schema::{SchemaCx, SchemaFields};
use indexmap::IndexMap;

/// What the string form of a [`NamedCondition`] names: read from the text as written, and
/// written back as its text.
pub trait Name: Clone {
    /// The name read from the string form.
    fn from_name(name: String) -> Self;
    /// The text the string form writes.
    fn as_name(&self) -> &str;
}

impl Name for String {
    fn from_name(name: String) -> Self {
        name
    }

    fn as_name(&self) -> &str {
        self
    }
}

impl Name for RenderControllerId {
    fn from_name(name: String) -> Self {
        RenderControllerId::parse(name)
    }

    fn as_name(&self) -> &str {
        self.as_str()
    }
}

/// A name, used always, or a `{ name: condition }` object whose value is the Molang condition
/// (a number or a boolean also works).
///
/// Vanilla reads only one member of the object: the one whose key sorts first byte by byte,
/// not the first written; [`first`](Self::first) gives that member,
/// [`ignored`](Self::ignored) the rest. An empty object is kept as written and names
/// nothing: the game uses nothing of such an entry, and the loaders warn of it where the file
/// has it.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(proxy = NamedConditionProxy)]
#[repr(u8)]
pub enum NamedCondition<N: Name> {
    /// The string form: always used.
    Always(N),
    /// The object form, every member as written.
    Conditional(IndexMap<String, Molang>),
}

impl<N: Name> NamedCondition<N> {
    /// The text of the name this entry uses (the string form's, or the object's member that
    /// is read); `None` for an empty object.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Always(name) => Some(name.as_name()),
            Self::Conditional(members) => first_member(members).map(|(name, _)| name.as_str()),
        }
    }

    /// The condition it is used under; `None` when it is always used (or names nothing).
    pub fn condition(&self) -> Option<&Molang> {
        match self {
            Self::Always(_) => None,
            Self::Conditional(members) => first_member(members).map(|(_, condition)| condition),
        }
    }

    /// The member of the object form vanilla reads, the one whose key sorts first byte by
    /// byte; `None` for the string form and for an empty object.
    pub fn first(&self) -> Option<(&str, &Molang)> {
        match self {
            Self::Always(_) => None,
            Self::Conditional(members) => {
                first_member(members).map(|(name, condition)| (name.as_str(), condition))
            }
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

impl NamedCondition<RenderControllerId> {
    /// The render controller this entry draws; `None` for an empty object.
    pub fn controller(&self) -> Option<RenderControllerId> {
        match self {
            Self::Always(id) => Some(id.clone()),
            Self::Conditional(members) => {
                first_member(members).map(|(name, _)| RenderControllerId::parse(name.as_str()))
            }
        }
    }
}

/// One entry of `render_controllers`: a render controller, drawn always or under a condition.
///
/// Written as the controller's id (`"controller.render.pig"`, always drawn) or as an object
/// `{ "controller.render.pig": "query.is_baby" }` whose value is the Molang condition, read
/// as every [`NamedCondition`] is; [`controller`](NamedCondition::controller) and
/// [`condition`](NamedCondition::condition) give the member read.
pub type ControllerUse = NamedCondition<RenderControllerId>;

/// One entry of `scripts.animate`: an animation or animation controller (by its short name in
/// `animations`), played always or under a condition.
///
/// Written as the short name (`"walk"`) or as an object `{ "walk": "query.is_moving" }`, read
/// as every [`NamedCondition`] is: only the member whose key sorts first byte by byte counts,
/// and an empty object is kept as written, names nothing and is warned of.
pub type AnimateEntry = NamedCondition<String>;

/// The member vanilla reads of `members`: its JSON objects are ordered maps, so the first
/// member is the key that sorts first by bytes.
#[doc(hidden)]
pub fn first_member(members: &IndexMap<String, Molang>) -> Option<(&String, &Molang)> {
    members
        .iter()
        .min_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()))
}

/// The keys of `members` other than the [`first_member`], in written order.
#[doc(hidden)]
pub fn ignored_members(members: Option<&IndexMap<String, Molang>>) -> impl Iterator<Item = &str> {
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

impl<N: Name> TryFrom<NamedConditionProxy> for NamedCondition<N> {
    type Error = NotANamedCondition;

    fn try_from(proxy: NamedConditionProxy) -> Result<Self, NotANamedCondition> {
        match proxy {
            NamedConditionProxy::String(name) => Ok(Self::Always(N::from_name(name))),
            NamedConditionProxy::Object(members) => Ok(Self::Conditional(members)),
            _ => Err(NotANamedCondition),
        }
    }
}

impl<N: Name> From<&NamedCondition<N>> for NamedConditionProxy {
    fn from(entry: &NamedCondition<N>) -> Self {
        match entry {
            NamedCondition::Always(name) => Self::String(name.as_name().to_owned()),
            NamedCondition::Conditional(members) => Self::Object(members.clone()),
        }
    }
}

/// The object form: every member's condition is a Molang field at its key, under the walk's
/// default spec (a family whose conditions read otherwise walks them with
/// `schema::molang_with`).
impl<N: Name> MolangFields for NamedCondition<N> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Self::Conditional(members) = self {
            let spec = *visitor.default_spec();
            members.visit_slot(&spec, visitor);
        }
    }
}

/// A named condition reads its own syntax and holds no bound the derive checks and no trigger
/// (its members are Molang, walked by `MolangFields`).
impl<N: Name> SchemaFields for NamedCondition<N> {
    fn check_fields(&mut self, _: &mut SchemaCx<'_>) {}

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry<N: Name + facet::Facet<'static>>(text: &str) -> NamedCondition<N> {
        bedrock_addon_core::json::from_str(text).unwrap_or_else(|error| panic!("{text}: {error}"))
    }

    #[test]
    fn the_object_form_reads_the_member_whose_key_sorts_first() {
        let named: AnimateEntry = entry(r#"{"walk": "q.a", "run": "q.b", "Zed": 1}"#);
        assert_eq!(named.name(), Some("Zed"));
        assert_eq!(named.first().map(|(name, _)| name), Some("Zed"));
        assert_eq!(named.ignored().collect::<Vec<_>>(), ["walk", "run"]);
        let always: ControllerUse = entry(r#""controller.render.a""#);
        assert_eq!(always.name(), Some("controller.render.a"));
        assert_eq!(always.condition(), None);
        assert_eq!(
            always.controller().map(|id| id.as_str().to_owned()),
            Some("controller.render.a".to_owned())
        );
    }

    #[test]
    fn an_empty_object_names_nothing_and_a_number_is_no_entry() {
        let empty: AnimateEntry = entry("{}");
        assert_eq!(empty.name(), None);
        assert!(bedrock_addon_core::json::from_str::<AnimateEntry>("1").is_err());
    }
}
