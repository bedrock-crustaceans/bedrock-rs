//! The newest struct and the beta struct as `VersionStruct`s, and writing items.

use std::borrow::Cow;

use bedrock_addon_core::diagnostic::{Diagnostic, Disposition, JsonPath, Kind};
use bedrock_addon_core::document::{ComponentMap, ComponentRef, Unknown, UnknownMembers};
use bedrock_addon_core::json::{
    EncodeError, JsonSlice, Value, WriteStyle, merge_unknown, write_json,
};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{
    BETA_VERSION, ParseCx, VersionStruct, Versioned, write_envelope, write_payload,
};
use bedrock_addon_shared_shapes::{CreativeCategory, MenuCategory};
use facet_value::VObject;

use super::component::ItemComponent;
use super::history::ItemFamily;
use super::history::{
    self, CustomComponentEntry, CustomComponents, SideCarPlace, write_parts, written_components,
};
use super::legacy::{ItemFormat, LegacyFormatItem};
use super::read::parse_newest;
use super::types::{ModernItem, modern_version};
use super::{Beta, Item, ItemDeprecatedData, ItemDescription, LegacyItemData};
use bedrock_addon_core::diagnostic::Diagnostics;

/// 1.26.50: the newest item format.
const V1_26_50: FormatVersion = known::V1_26_50;

/// The path of the item object of a document.
fn item_at() -> JsonPath {
    JsonPath::of_keys(["minecraft:item"])
}

/// The `description` of the newest format as JSON; `unknown` the document's unknown members,
/// whose `menu_category` members make the placement written whole.
fn description_value(
    description: &ItemDescription,
    unknown: &Unknown,
) -> Result<Value, EncodeError> {
    let mut object = VObject::new();
    object.insert("identifier", Value::from(description.identifier.as_str()));
    // A `none` and a `menu_category` the document did not write are not written, so the text
    // reads back as the document did; everything else of the placement is.
    let menu = &description.menu_category;
    let menu_at = item_at().child("description").child("menu_category");
    let whole = menu.group.is_some()
        || menu.is_hidden_in_commands.is_some()
        || unknown.members_at(&menu_at).next().is_some();
    if description.category_is_written() {
        object.insert("menu_category", write_payload(menu)?);
    } else if whole || description.menu_category_presence != super::MenuCategoryPresence::Absent {
        let without_category = MenuCategory {
            category: None,
            ..menu.clone()
        };
        object.insert("menu_category", write_payload(&without_category)?);
    }
    Ok(Value::from(object))
}

/// The side-car the newest format writes: without `custom_components_v1` when the keyed
/// custom components the 1.26.30 upgrade made from that list (`from_v1_list`) spell it out
/// again, entry for entry and in order. 1.26.30 binds no `__deprecated_components`,
/// so a side-car written only for the list would read back as an unrecognised member.
fn side_car_to_write<'d>(
    deprecated: &'d ItemDeprecatedData,
    components: &ComponentMap<ItemComponent>,
) -> Cow<'d, ItemDeprecatedData> {
    let Some(list) = &deprecated.custom_components_v1 else {
        return Cow::Borrowed(deprecated);
    };
    let keyed = CustomComponents {
        components: components
            .iter()
            .filter_map(|entry| match entry {
                ComponentRef::Custom(c) if c.from_v1_list => Some(CustomComponentEntry {
                    name: c.name.clone(),
                    data: c.parameters.clone(),
                }),
                _ => None,
            })
            .collect(),
    };
    if keyed.to_value() == *list {
        let mut without = deprecated.clone();
        without.custom_components_v1 = None;
        Cow::Owned(without)
    } else {
        Cow::Borrowed(deprecated)
    }
}

/// The item object of a modern format as JSON.
pub(crate) fn newest_value(
    description: &ItemDescription,
    components: &ComponentMap<ItemComponent>,
    deprecated: &ItemDeprecatedData,
    events: Option<&Value>,
    legacy: &LegacyItemData,
    unknown: &Unknown,
) -> Result<Value, EncodeError> {
    let typed = written_components(components);
    // What the legacy reading alone holds of a newest-format document, after the components.
    let mut held: Vec<(String, Result<Value, EncodeError>)> = legacy
        .held_in_shared_alone
        .iter()
        // (A key the newest format binds a component for is that component or nothing:
        // an older file's `use_modifiers`, told when it was read, is not written raw.)
        .filter(|key| history::binding(V1_26_50, key).is_none())
        .filter(|key| !typed.iter().any(|(typed, _)| typed == *key))
        .filter_map(|key| Some((key.clone(), Ok(legacy.shared.get(key)?.clone()))))
        .collect();
    // And the two keys the legacy reader takes at every version.
    held.extend(
        legacy
            .written_keys()
            .into_iter()
            .map(|(key, value)| (key, Ok(value))),
    );
    write_parts(
        description_value(description, unknown),
        typed.into_iter().chain(held),
        (
            &side_car_to_write(deprecated, components),
            SideCarPlace::AfterEvents,
        ),
        events,
        unknown,
    )
}

impl<const V: u32> Versioned for ModernItem<V> {
    const VERSION: FormatVersion = modern_version(V);
}

impl<const V: u32> VersionStruct for ModernItem<V> {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_newest(payload, cx, d)
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.write_with_unknown(&Unknown::new())
    }

    fn write_with_unknown(&self, unknown: &Unknown) -> Result<Value, EncodeError> {
        newest_value(
            &self.description,
            &self.components,
            &self.deprecated,
            self.events.as_ref(),
            &self.legacy,
            unknown,
        )
    }
}

impl<const V: u32> MolangFields for ModernItem<V> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.nested("components", &mut self.components);
        self.legacy.expressions.visit_molang(visitor);
    }
}

/// `{"format_version": version, "minecraft:item": payload}` (`"beta"` for the beta version).
fn envelope(version: FormatVersion, payload: Value) -> Value {
    write_envelope::<ItemFamily>(version, payload, &UnknownMembers::default())
}

/// Whether the text the newest format writes of `item` is a component-based document to the
/// game: its `components` has a registered component (a typed one, or one the legacy reading
/// alone holds) or a `tag:` key. Custom components and keys no table knows are not registered.
pub(crate) fn is_component_based_as_written(item: &Item) -> bool {
    item.components
        .iter()
        .filter_map(|entry| match entry {
            ComponentRef::Typed { name, .. }
            | ComponentRef::Dynamic { name, .. }
            | ComponentRef::Unknown { name, .. } => Some(name),
            ComponentRef::Custom(c) => Some(c.name.as_str()),
            _ => None,
        })
        .chain(
            item.legacy
                .held_in_shared_alone
                .iter()
                .filter(|key| item.legacy.shared.contains_key(*key))
                .map(String::as_str),
        )
        .any(super::read::is_registered)
}

/// The document of an item: the newest format, or, for an item in the legacy item format, that format
/// at 1.10.0 (the newest format cannot hold it).
///
/// A document is component-based to the game by a registered component of its `components`;
/// without one it is the legacy item format at every version. So an item that holds nothing
/// (no component, no side-car member) is written in the legacy item format, and these have
/// no text: an item without a registered component that holds custom components, keys no
/// table knows or side-car members, and one the load left without a registered component
/// (`NotRepresentable` at `components`, where the load told of it); and an empty item with a
/// `menu_category` to write, which a legacy item does not have (`NotRepresentable` at
/// `description/menu_category`).
pub(crate) fn item_root(item: &Item) -> Result<Value, EncodeError> {
    item_root_with(item, &Unknown::new()).map(|(root, _)| root)
}

/// [`item_root`] with the document's unknown members in it, and the findings of those left
/// out: which format is written depends on them (the newest one refuses an unknown member).
pub(crate) fn item_root_with(
    item: &Item,
    unknown: &Unknown,
) -> Result<(Value, Vec<Diagnostic>), EncodeError> {
    let refused = |members: &[&str]| {
        let mut at = bedrock_addon_core::diagnostic::JsonPath::new();
        at.push("minecraft:item");
        for member in members {
            at.push(member);
        }
        EncodeError::NotRepresentable {
            at,
            what: bedrock_addon_core::json::Unrepresentable::Unsupported,
        }
    };
    let empty = LegacyFormatItem::default();
    let format_item = match (&item.legacy.format, &item.legacy.format_item) {
        (ItemFormat::Legacy, Some(format_item)) => Some(format_item),
        _ if is_component_based_as_written(item) => None,
        // No registered component: the text of the newest format would read back as a
        // legacy item, without what the item holds. Only an item that holds nothing has a
        // text then, the legacy document.
        _ if item.legacy.left_empty_by_the_load
            || item.components.iter().next().is_some()
            || !item.deprecated.is_empty() =>
        {
            return Err(refused(&["components"]));
        }
        _ => {
            let writes_a_category = description_value(&item.description, unknown)?
                .as_object()
                .is_some_and(|description| description.contains_key("menu_category"));
            if writes_a_category {
                return Err(refused(&["description", "menu_category"]));
            }
            Some(&empty)
        }
    };
    let root = if let Some(format_item) = format_item {
        let legacy = history::v1_10_0::Item {
            description: history::Description {
                identifier: item.description.identifier.as_str().to_owned(),
                category: item.legacy.category.clone(),
                menu_category: None,
            },
            components: format_item.clone(),
            events: item.events.clone(),
        };
        let mut root = envelope(V1_10_0, legacy.write()?);
        let told = merge_unknown(&mut root, unknown);
        (root, told)
    } else {
        let mut root = envelope(V1_26_50, item.write_with_unknown(unknown)?);
        let told = merge_unknown(&mut root, unknown);
        let payload = payload_of(&root);
        match refuse_what_the_format_refuses::<Item>(&payload) {
            Ok(()) => (root, told),
            Err(refused) => written_at_1_26_0(item, payload, unknown).ok_or(refused)?,
        }
    };
    Ok(root)
}

/// The item object of a whole document.
fn payload_of(root: &Value) -> Value {
    root.as_object()
        .and_then(|root| root.get("minecraft:item"))
        .cloned()
        .unwrap_or(Value::NULL)
}

/// 1.26.0: the newest format of the legacy chain (`ComponentItemData_v1_26_0`), whose reader
/// binds `events` and the side-car `__deprecated_components` and reads an unknown member with
/// a warning (an unstrict slice).
const V1_26_0: FormatVersion = known::V1_26_0;

/// 1.10.0: the version the legacy item format is written at.
const V1_10_0: FormatVersion = FormatVersion::new(1, 10, 0);

/// An item the 1.26.50 format refuses (it holds
/// `events`, the side-car or an unknown member, from an older file) is written as a 1.26.0
/// legacy component document, when that text reads back as the same item: loaded without an
/// error, the same value but for the legacy reading's raw copies. The newest item does not
/// carry the version of the file it came from, so the version is the newest one whose reader
/// takes those members. `None` when the text does not read back so (the caller refuses at the
/// member the 1.26.50 format refuses): nothing is dropped in silence.
///
/// `payload` has the document's unknown members in it already; so has the text, and it reads
/// back with them where the item has them (`unknown`).
fn written_at_1_26_0(
    item: &Item,
    payload: Value,
    unknown: &Unknown,
) -> Option<(Value, Vec<Diagnostic>)> {
    let mut root = envelope(V1_26_0, payload);
    // The root's own members (the payload's are in it, and are put again in place).
    let told = merge_unknown(&mut root, unknown);
    let text = bedrock_addon_core::json::to_facet_json(&root).ok()?;
    let cx = bedrock_addon_core::load::LoadContext::detached();
    let reread = super::read::load_item(text.as_bytes(), &cx);
    if reread.outcome.diagnostics.has_errors() {
        return None;
    }
    let Some(bedrock_addon_core::versioning::Loaded::Stable(mut again)) = reread.outcome.value
    else {
        return None;
    };
    let mut held = item.clone();
    again.legacy.shared.clear();
    held.legacy.shared.clear();
    let same_unknown = reread.outcome.diagnostics.unknown() == unknown;
    (again == held && same_unknown).then_some((root, told))
}

/// Where the newest writer refuses `item` for a member the strict 1.26.50 format does not
/// bind (`refuse_what_the_format_refuses`); `None` when it is written, or written in the legacy
/// item format.
pub(crate) fn newest_refusal(
    item: &Item,
    unknown: &Unknown,
) -> Option<bedrock_addon_core::diagnostic::JsonPath> {
    if item.legacy.format == ItemFormat::Legacy || !is_component_based_as_written(item) {
        return None;
    }
    let mut root = envelope(V1_26_50, item.write_with_unknown(unknown).ok()?);
    let _ = merge_unknown(&mut root, unknown);
    let payload = payload_of(&root);
    match refuse_what_the_format_refuses::<Item>(&payload) {
        Err(EncodeError::NotRepresentable { at, .. }) => Some(at),
        _ => None,
    }
}

/// The modern formats are strict slices: a member or a
/// component of the payload that `version`'s struct does not bind rejects the document. An
/// item upgraded from an older format may hold such members for write-back (an unknown member,
/// `events`, the side-car); written into a modern format they make a text the game refuses,
/// so the newest writer does not write them into it: it writes the item at 1.26.0 where that
/// reads back as the item, else refuses it at the first of them. What the
/// struct binds is what its reader reads: the payload is read back with it, and an unknown
/// member or component it tells of is the refusal.
fn refuse_what_the_format_refuses<S: VersionStruct>(payload: &Value) -> Result<(), EncodeError> {
    let version = S::VERSION;
    let Ok(text) = bedrock_addon_core::json::to_facet_json(payload) else {
        return Ok(());
    };
    let mut cx = bedrock_addon_core::load::LoadContext::detached();
    cx.is_base_game_pack = true;
    cx.null_rule = None;
    let mut at = bedrock_addon_core::diagnostic::JsonPath::new();
    at.push("minecraft:item");
    let parse = ParseCx {
        load: &cx,
        written: bedrock_addon_core::version::FormatVersionInput {
            version: Some(version),
            beta_literal: version == BETA_VERSION,
            prerelease: false,
        },
        original_version: version,
        at: at.clone(),
    };
    let mut told = Diagnostics::default();
    let _ = S::parse(JsonSlice::new(&text), &parse, &mut told);
    let refused = told.iter().find(|finding| {
        finding.kind() == Kind::UnknownMember(Disposition::Kept)
            && finding.location.json_path.starts_with(&at)
    });
    match refused {
        Some(finding) => Err(EncodeError::NotRepresentable {
            at: finding.location.json_path.clone(),
            what: bedrock_addon_core::json::Unrepresentable::Unsupported,
        }),
        None => Ok(()),
    }
}

/// Writes a beta item (`"format_version": "beta"`).
pub(crate) fn write_beta(
    beta: &Beta,
    out: &mut Vec<u8>,
    style: WriteStyle,
) -> Result<(), EncodeError> {
    write_beta_with(beta, &Unknown::new(), out, style).map(drop)
}

/// [`write_beta`] with the document's unknown members, which the beta format refuses as it
/// refuses them in the payload.
pub(crate) fn write_beta_with(
    beta: &Beta,
    unknown: &Unknown,
    out: &mut Vec<u8>,
    style: WriteStyle,
) -> Result<Vec<Diagnostic>, EncodeError> {
    let mut root = envelope(BETA_VERSION, beta.write_with_unknown(unknown)?);
    let told = merge_unknown(&mut root, unknown);
    refuse_what_the_format_refuses::<Beta>(&payload_of(&root))?;
    write_json(&root, out, style)?;
    Ok(told)
}

#[allow(dead_code, reason = "the default menu placement of a new item")]
pub(crate) fn default_menu() -> MenuCategory {
    MenuCategory {
        category: Some(CreativeCategory::ItemCommandOnly),
        ..MenuCategory::default()
    }
}
