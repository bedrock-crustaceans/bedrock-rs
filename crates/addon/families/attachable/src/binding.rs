//! `description.item` ([`ItemBinding`]), what it binds ([`Condition`]), and the bindings a
//! stack gathers ([`ItemBindingEntry`], [`ItemBindings`]).

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostic, Severity};
use bedrock_addon_core::document::Registry;
use bedrock_addon_core::id::{AttachableId, ItemId};
use bedrock_addon_core::json::kit::{Node, Pass, Read};
use bedrock_addon_core::molang::{
    Molang, MolangFields, MolangSlot, MolangSource, MolangVisitor, RawVersion,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef};
use bedrock_addon_core::schema::{self, FromJson, Schema};
use bedrock_addon_shared_shapes::named::{first_member, ignored_members};
use bedrock_addon_shared_visual::read::{is_condition_key, not_valid_here, required};
use bedrock_addon_shared_visual::rules::NAMED_SCRIPT;
use indexmap::IndexMap;

use super::Attachable;

/// The pattern of an object-form `item` key, as vanilla prints it.
const ITEM_KEY: &str = "[a-zA-Z0-9_.:-]+";

/// What an attachable's `description` has beside the shared members: `item`, from format
/// version 1.10.0.
#[derive(Clone, Debug, PartialEq, facet::Facet, Schema)]
pub struct ItemMember {
    /// `item`: the item the attachable is drawn for, always or under a condition. Absent, the
    /// attachable is drawn for the item whose identifier is its own.
    #[facet(schema::molang = NAMED_SCRIPT)]
    pub item: Option<ItemBinding>,
}

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

    /// The member of the object form vanilla reads, with its condition.
    fn first_condition(&self) -> Option<(&str, &Molang)> {
        match self {
            Self::Item(_) => None,
            Self::Conditional(members) => {
                first_member(members).map(|(item, condition)| (item.as_str(), condition))
            }
        }
    }

    /// The members of the object form vanilla does not read, in written order.
    pub fn ignored(&self) -> impl Iterator<Item = &str> {
        let members = match self {
            Self::Item(_) => None,
            Self::Conditional(members) => Some(members),
        };
        ignored_members(members)
    }
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
    pub fn is_always(&self) -> bool {
        matches!(self, Self::Always)
    }

    /// Whether the condition can never hold: the empty text vanilla reads from a number.
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
    pub fn len(&self) -> usize {
        self.by_item.len()
    }

    /// No item has a binding.
    pub fn is_empty(&self) -> bool {
        self.by_item.is_empty()
    }
}

/// A string is the item, an object its conditions; any other value is a wrong type, and the
/// rules of the schema are told after the read:
/// - the string form takes any string; one outside the object form's key pattern is noted
///   (an information of the crate's own) and kept;
/// - every key of the object form fully matches `[a-zA-Z0-9_.:-]+`, and the object holds at
///   least one member;
/// - the members after the one vanilla reads are dropped, with an information of the crate's
///   own;
/// - a number as the condition of the member that binds reads as the empty text, which is
///   never true: an information of the crate's own, the binding kept.
impl FromJson for ItemBinding {
    fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let at = node.path();
        let after = |read: &mut Read<'_>, finding: Diagnostic| read.defer(Pass::Unknown, finding);
        if let Some(text) = node.value().as_string() {
            let item = text.as_str();
            if !is_condition_key(item) {
                let told = Diagnostic::of(&core_codes::INVALID_VALUE)
                    .severity(Severity::Info)
                    .at(at.clone());
                after(read, told);
            }
            return Some(Self::Item(item.to_owned()));
        }
        if !node.value().is_object() {
            read.wrong_type(node);
            return None;
        }
        // A condition is a number, a boolean or a string; its object form is no condition here.
        let object = read.object(node)?;
        if let Some((_, odd)) = object.iter().find(|(_, member)| member.value().is_object()) {
            read.wrong_type(&odd);
            return None;
        }
        let members = IndexMap::<String, Molang>::from_json(node, read)?;
        // The pattern child is required and matched in full.
        for key in members.keys().filter(|key| !is_condition_key(key)) {
            after(read, not_valid_here(key, at.child(key)));
        }
        if members.is_empty() {
            after(read, required(ITEM_KEY, at.clone()));
        }
        let item = Self::Conditional(members);
        for key in item.ignored() {
            after(
                read,
                Diagnostic::of(&core_codes::DROPPED_FIELD).at(at.child(key)),
            );
        }
        // `asString("")` of a number is the empty text.
        if let Some((key, Molang::Const(_))) = item.first_condition() {
            let told = Diagnostic::of(&core_codes::INVALID_VALUE)
                .severity(Severity::Info)
                .at(at.child(key));
            after(read, told);
        }
        Some(item)
    }
}

bedrock_addon_core::impl_schema_leaf!(ItemBinding);

/// The conditions of an object form, each a Molang field at its key, under the walk's default
/// spec.
impl MolangFields for ItemBinding {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Self::Conditional(members) = self {
            let spec = *visitor.default_spec();
            members.visit_slot(&spec, visitor);
        }
    }
}

/// What the writer turns an item binding into: its text, or its object.
#[derive(Debug, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
pub(crate) enum ItemBindingProxy {
    /// A string: the item.
    String(String),
    /// An object: item → condition.
    Object(IndexMap<String, Molang>),
}

impl From<ItemBindingProxy> for ItemBinding {
    fn from(proxy: ItemBindingProxy) -> Self {
        match proxy {
            ItemBindingProxy::String(item) => Self::Item(item),
            ItemBindingProxy::Object(members) => Self::Conditional(members),
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
