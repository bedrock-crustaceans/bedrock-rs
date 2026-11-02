//! `description.item` ([`ItemBinding`]), what it binds ([`Condition`]), and the bindings a
//! stack gathers ([`ItemBindingEntry`], [`ItemBindings`]).

use bedrock_addon_core::document::Registry;
use bedrock_addon_core::id::{AttachableId, ItemId};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{Molang, MolangSource, RawVersion};
use bedrock_addon_core::pack_identity::{PackPath, PackRef};
use indexmap::IndexMap;

use super::Attachable;

/// `description.item`: the item an attachable is drawn for.
///
/// Written as the item's identifier (`"minecraft:bow"`, always drawn) or as an object
/// `{ "minecraft:diamond_helmet": "query.owner_identifier == 'minecraft:player'" }` whose
/// value is the Molang condition (a number or a boolean also passes). Vanilla reads only one
/// member of the object: the one whose key sorts first byte by byte, not the first written
/// ([`ItemBinding::bound`]). The item is compared with an item's full identifier byte for
/// byte: it is not lower-cased and gets no `minecraft:` prefix.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(proxy = ItemBindingProxy)]
#[repr(u8)]
pub enum ItemBinding {
    /// The string form: drawn for the item, always.
    Item(String),
    /// The object form, every member as written.
    Conditional(IndexMap<String, Molang>),
}

impl ItemBinding {
    /// The item bound and the condition, as vanilla's loader reads them; `None` for an empty
    /// object, which binds nothing (and which no loaded document holds).
    #[must_use]
    pub fn bound(&self) -> Option<(ItemId, Condition)> {
        self.bound_at(None)
    }

    /// [`bound`](Self::bound), a condition that is no expression taking `version` (the
    /// pack's Molang version) when given.
    pub(crate) fn bound_at(&self, version: Option<RawVersion>) -> Option<(ItemId, Condition)> {
        // A string form binds the key as written, with an expression that is always 1.0; an
        // object form binds its first member alone, with the member's value read as a string
        // (`asString("")`) and compiled at the pack's Molang version.
        match self {
            Self::Item(item) => Some((ItemId::parse(item.as_str()), Condition::Always)),
            Self::Conditional(members) => {
                let (item, condition) = first_member(members)?;
                // `asString("")` gives a string as it is, a boolean as `true` or `false`, and
                // a number as the empty text.
                let text = |text: &'static str| match version {
                    Some(version) => MolangSource::string(text, version),
                    None => MolangSource::string_without_context(text),
                };
                let source = match condition {
                    Molang::Expr(source) => source.clone(),
                    Molang::Bool(true) => text("true"),
                    Molang::Bool(false) => text("false"),
                    _ => text(""),
                };
                Some((ItemId::parse(item.as_str()), Condition::Molang(source)))
            }
        }
    }

    /// The members of the object form vanilla does not read, in written order.
    pub fn ignored(&self) -> impl Iterator<Item = &str> {
        let members = match self {
            Self::Item(_) => None,
            Self::Conditional(members) => Some(members),
        };
        let read = members.and_then(first_member).map(|(name, _)| name);
        members
            .into_iter()
            .flat_map(IndexMap::keys)
            .filter(move |name| Some(*name) != read)
            .map(String::as_str)
    }
}

// The member vanilla reads: a jsoncpp object is a `std::map` ordered by `strcmp`, so the
// first member is the key that sorts first by bytes.
fn first_member(members: &IndexMap<String, Molang>) -> Option<(&String, &Molang)> {
    members
        .iter()
        .min_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()))
}

/// The condition an item binding is drawn under.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Condition {
    /// The string form of `item`: always (vanilla's constant 1.0).
    Always,
    /// The object form's value as vanilla's loader reads it: the text of an expression,
    /// `true` or `false` for a boolean, and the empty text for a number, which is never
    /// true ([`Condition::is_never`]).
    Molang(MolangSource),
}

impl Condition {
    /// Whether the binding is drawn whatever the item's owner is: the string form.
    #[must_use]
    pub fn is_always(&self) -> bool {
        matches!(self, Self::Always)
    }

    /// Whether the condition can never hold: the empty text vanilla reads from a number.
    #[must_use]
    pub fn is_never(&self) -> bool {
        matches!(self, Self::Molang(source) if source.is_empty())
    }
}

/// One item binding a stack holds: an accepted document's `description.item`, with where it
/// came from.
///
/// Vanilla appends one for every accepted document that has `item` and never removes one, so
/// a binding outlives the document it came from: when a later document replaces the
/// definition, the binding stays and now selects the replacement
/// ([`shadowed`](Self::shadowed)).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct ItemBindingEntry {
    /// The item, as written.
    pub item: ItemId,
    /// The attachable the binding selects: the identifier of the document it came from, in
    /// lower case.
    pub attachable: AttachableId,
    /// The condition.
    pub condition: Condition,
    /// The pack of the document it came from.
    pub pack: PackRef,
    /// That document's file.
    pub file: PackPath,
    /// Whether a later document replaced the definition this binding was written for. The
    /// binding still counts.
    pub shadowed: bool,
    /// Its place among every binding of the stack, in the order vanilla gathered them
    /// (the first is 0). A client tries the bindings of an item from the last to the first.
    pub sequence: usize,
}

/// The item bindings of a stack by item: which attachables a client tries for an item, in
/// which order.
///
/// A client draws an item with the first attachable, from the last binding gathered to the
/// first, whose condition holds; when none does, with the attachable whose identifier is the
/// item's (`rp.attachables().get(..)` of the item's identifier).
#[derive(Clone, Debug)]
pub struct ItemBindings<'a> {
    by_item: IndexMap<&'a str, Vec<&'a ItemBindingEntry>>,
}

impl<'a> ItemBindings<'a> {
    /// The bindings of every effective definition of `attachables`.
    #[must_use]
    pub fn from_registry(attachables: &'a Registry<Attachable>) -> Self {
        let mut entries: Vec<&ItemBindingEntry> = attachables
            .iter()
            .flat_map(|(_, document)| document.value().bound_items())
            .collect();
        entries.sort_by_key(|entry| entry.sequence);
        let mut by_item: IndexMap<&str, Vec<&ItemBindingEntry>> = IndexMap::new();
        for entry in entries {
            by_item.entry(entry.item.as_str()).or_default().push(entry);
        }
        Self { by_item }
    }

    /// The bindings of `item` (compared byte for byte), in the order vanilla gathered them.
    pub fn get(&self, item: &str) -> &[&'a ItemBindingEntry] {
        self.by_item.get(item).map_or(&[], Vec::as_slice)
    }

    /// The bindings of `item` in the order a client tries them: the last gathered first.
    pub fn tried(&self, item: &str) -> impl Iterator<Item = &'a ItemBindingEntry> + '_ {
        self.get(item).iter().rev().copied()
    }

    /// The items with a binding, in the order of their first binding.
    pub fn items(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.by_item.keys().copied()
    }

    /// The number of items with a binding.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_item.len()
    }

    /// No item has a binding.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_item.is_empty()
    }
}

/// Every JSON shape a deserializer can offer, so the conversion rejects the ones `item` does
/// not take (the untagged solver would otherwise coerce a number into a string).
#[derive(Debug, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(
    dead_code,
    reason = "the payloads of the rejected shapes are never read"
)]
pub(crate) enum ItemBindingProxy {
    /// `null`.
    Null,
    /// A string: the item.
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
    /// An object: item → condition.
    Object(IndexMap<String, Molang>),
}

/// Why a JSON value is no item binding.
#[derive(Debug, thiserror::Error)]
#[error("an item binding must be a string or an object of conditions")]
pub struct NotAnItemBinding;

impl TryFrom<ItemBindingProxy> for ItemBinding {
    type Error = NotAnItemBinding;

    fn try_from(proxy: ItemBindingProxy) -> Result<Self, NotAnItemBinding> {
        match proxy {
            ItemBindingProxy::String(item) => Ok(Self::Item(item)),
            ItemBindingProxy::Object(members) => Ok(Self::Conditional(members)),
            _ => Err(NotAnItemBinding),
        }
    }
}

impl From<&ItemBinding> for ItemBindingProxy {
    fn from(binding: &ItemBinding) -> Self {
        match binding {
            ItemBinding::Item(item) => Self::String(item.clone()),
            ItemBinding::Conditional(members) => Self::Object(members.clone()),
        }
    }
}
