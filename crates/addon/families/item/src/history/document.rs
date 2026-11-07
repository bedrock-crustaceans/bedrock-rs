//! The component-format structs 1.16.100 … 1.26.0: one generic struct, [`ItemDocument`], whose
//! version parameter selects what the format binds; and the parse and write both they and the
//! newest struct use.

use bedrock_addon_core::document::Unknown as Table;
use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::json::{EncodeError, JsonSlice, NullRule, Value, parse_at};
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::schema::{self, Schema, SchemaCx};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{
    BETA_VERSION, ConvertMode, ParseCx, Reading, VersionStruct, Versioned, parse_component,
    parse_payload, write_payload,
};
use bedrock_addon_shared_shapes::{CreativeCategory, MenuCategory};
use facet::Facet;
use facet_json::RawJson;
use facet_value::VObject;
use indexmap::IndexMap;

use super::component::{
    Armor, Chargeable, Component, Components, CustomComponentEntry, CustomComponentList,
    CustomComponents, LegacyDigger, LegacyDurability, LegacyFoodComponent, LegacyIcon,
    LegacyRepairable, LegacyStorageItem, OnUse, OnUseOn, RenderOffsets, UseDuration, Weapon,
};
use super::gate;
use crate::codes;
use crate::component::{
    AllowOffHand, BlockPlacer, BundleInteraction, CanDestroyInCreative, Compostable, Cooldown,
    Damage, DamageAbsorption, Digger, DisplayName, Durability, DurabilitySensor, Dyeable,
    Enchantable, EntityPlacer, FireResistant, Food, Fuel, Glint, HandEquipped, HoverTextColor,
    Icon, InteractButton, ItemComponent, KineticWeapon, LiquidClipped, MaxStackSize,
    PiercingWeapon, Projectile, Rarity, Record, Repairable, Shooter, ShouldDespawn, StackedByData,
    StorageItem, StorageWeightLimit, StorageWeightModifier, SwingDuration, SwingSounds, Tags,
    Throwable, UseAnimation, UseModifiers, ValueComponent, Wearable,
};
use crate::deprecated::ItemDeprecatedData;
use crate::legacy::{LegacyItemData, is_legacy_read, read_legacy};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Class, Code, Diagnostic, Diagnostics, Disposition, JsonPath, Kind, Severity,
};
use bedrock_addon_core::load::LoadContext;

/// A packed `major.minor.patch` (`major × 1 000 000 + minor × 1 000 + patch`), the version
/// parameter of [`ItemDocument`].
#[doc(hidden)]
pub const fn pack(major: u32, minor: u32, patch: u32) -> u32 {
    major * 1_000_000 + minor * 1_000 + patch
}

/// The version a packed number stands for.
#[doc(hidden)]
pub const fn unpack(packed: u32) -> FormatVersion {
    #[allow(clippy::cast_possible_truncation, reason = "each part is below 1 000")]
    FormatVersion::new(
        (packed / 1_000_000) as u16,
        ((packed / 1_000) % 1_000) as u16,
        (packed % 1_000) as u16,
    )
}

/// An item document of a component format older than the newest (1.16.100 … 1.26.0): the
/// format version `V` (packed by [`pack`]) decides which components and members it binds.
/// `history::v1_20_50::Item` and its siblings name the instances.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemDocument<const V: u32> {
    /// JSON `description`.
    pub description: Description,
    /// JSON `components`, by key in file order.
    pub components: Components,
    /// JSON `__deprecated_components` (from 1.20.30).
    pub deprecated: ItemDeprecatedData,
    /// JSON `events`, as written.
    pub events: Option<Value>,
    /// What vanilla's legacy reader takes from the document; not written.
    pub legacy: LegacyItemData,
}

/// The `description` of an older item format: `identifier`, the 1.16.100–1.20.0 `category`
/// and the 1.20.20 and later `menu_category`.
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(schema::unknown = "keep")]
pub struct Description {
    /// JSON `identifier` (required).
    pub identifier: String,
    /// JSON `category` (up to 1.20.0): `construction`, `nature`, `equipment`, `items` or
    /// `none`.
    pub category: Option<String>,
    /// JSON `menu_category` (from 1.20.20).
    pub menu_category: Option<MenuCategory>,
}

const V1_17_0: FormatVersion = FormatVersion::new(1, 17, 0);
const V1_18_0: FormatVersion = FormatVersion::new(1, 18, 0);
const V1_20_20: FormatVersion = FormatVersion::new(1, 20, 20);
const V1_20_30: FormatVersion = FormatVersion::new(1, 20, 30);
const V1_20_40: FormatVersion = FormatVersion::new(1, 20, 40);
const V1_20_50: FormatVersion = FormatVersion::new(1, 20, 50);
const V1_20_60: FormatVersion = FormatVersion::new(1, 20, 60);
const V1_20_80: FormatVersion = FormatVersion::new(1, 20, 80);
const V1_21_10: FormatVersion = FormatVersion::new(1, 21, 10);
const V1_21_30: FormatVersion = known::V1_21_30;
const V1_21_40: FormatVersion = FormatVersion::new(1, 21, 40);
const V1_21_50: FormatVersion = FormatVersion::new(1, 21, 50);
const V1_21_60: FormatVersion = FormatVersion::new(1, 21, 60);
const V1_21_90: FormatVersion = known::V1_21_90;
const V1_21_110: FormatVersion = known::V1_21_110;
const V1_26_0: FormatVersion = known::V1_26_0;
const V1_26_30: FormatVersion = known::V1_26_30;
const V1_26_50: FormatVersion = known::V1_26_50;

/// From 1.20.60 vanilla reports unknown members and components (the full schema parse);
/// below it they are kept silently.
pub(crate) fn strict(original_version: FormatVersion) -> bool {
    original_version >= V1_20_60
}

/// The parser of one component in a format.
type Parser = fn(JsonSlice<'_>, &LoadContext, &mut Diagnostics) -> Option<Component>;

/// A typed component from its slice. A failed read refuses the item in this chain as in the
/// strict slices: the legacy chain's slices are read by the same schema reader as the
/// modern ones, under which a member of another type and a missing required member are
/// errors; the game then logs " Error Parsing Item" at the error level and drops the
/// document.
fn typed<T>(slice: JsonSlice<'_>, cx: &LoadContext, d: &mut Diagnostics) -> Option<Component>
where
    T: for<'a> Facet<'a> + Into<Component>,
{
    read_typed::<T, Component>(slice, cx, d)
}

/// A one-value component of the newest format, from its object form or its bare value: the
/// bare value is read by the shorthand of its row in [`ItemComponent`].
fn value<T>(slice: JsonSlice<'_>, cx: &LoadContext, d: &mut Diagnostics) -> Option<Component>
where
    T: for<'a> Facet<'a>
        + Into<ItemComponent>
        + Into<Component>
        + bedrock_addon_core::document::Component<Set = ItemComponent>,
{
    let component = read_typed::<T, ItemComponent>(slice, cx, d)?;
    T::try_from_set(component).ok().map(Into::into)
}

/// [`typed`] into the component set `C`, whose `shorthand = scalar` rows read a bare value.
fn read_typed<T, C>(slice: JsonSlice<'_>, cx: &LoadContext, d: &mut Diagnostics) -> Option<C>
where
    T: for<'a> Facet<'a> + Into<C>,
    C: bedrock_addon_core::schema::ShorthandRows,
{
    let mut strict = cx.clone();
    strict.strict_components = true;
    let component = parse_component::<T, C>(slice, ConvertMode::Strict, &strict, d);
    if component.is_some() {
        // A range's extra member is a member the schema does not know: under this chain's
        // error mask the game writes it as a warning-level line while the item loads.
        // The strict slices (the context's own `strict_components`) refuse the item.
        let severity = if cx.strict_components {
            Severity::Error
        } else {
            Severity::Warning
        };
        crate::component::tell_range_members::<T>(slice, d, severity);
    }
    component
}

/// A one-value component whose bare value is not its member as written (`icon`'s key is its
/// `textures.default`), or that no row of [`ItemComponent`] reads (`use_duration` and the
/// legacy `icon`): from its object form or its bare value.
fn bare_value<T>(slice: JsonSlice<'_>, cx: &LoadContext, d: &mut Diagnostics) -> Option<Component>
where
    T: ValueComponent + Into<Component>,
{
    if slice.opens_an_object() {
        return typed::<T>(slice, cx, d);
    }
    match crate::component::read_bare::<T::Scalar>(slice, cx, d) {
        Ok(v) => Some(T::from_scalar(v).into()),
        Err(bedrock_addon_core::json::ScalarFailure::Refused(_)) => None,
        Err(bedrock_addon_core::json::ScalarFailure::Read(error)) => {
            // A scalar of another type is a type error: the item is refused (the chain's
            // refusal is `typed`'s).
            d.push(
                Diagnostic::error(
                    bedrock_addon_core::diagnostic::code_for_deserialize_error_kind(
                        &error.error.kind,
                        false,
                        false,
                    ),
                )
                .span(error.span),
            );
            None
        }
    }
}

/// `minecraft:allow_off_hand` of the beta struct: the stable component's binding, whose
/// `value` defaults to true (it defaults to false in a stable document).
fn beta_allow_off_hand(
    slice: JsonSlice<'_>,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<Component> {
    let mut component = value::<AllowOffHand>(slice, cx, d)?;
    let written = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text)
        .ok()
        .and_then(|json| json.as_object().map(|object| object.contains_key("value")));
    if let (Component::AllowOffHand(c), Some(false)) = (&mut component, written) {
        c.value = true;
    }
    Some(component)
}

/// `minecraft:interact_button`: a bool or a string.
fn interact_button(
    slice: JsonSlice<'_>,
    _: &LoadContext,
    d: &mut Diagnostics,
) -> Option<Component> {
    let json = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).ok();
    if let Some(button) = json.as_ref().and_then(InteractButton::from_value) {
        return Some(Component::InteractButton(button));
    }
    // Neither: a type error, the item refused (the chain's refusal is `typed`'s).
    d.push(Diagnostic::of(&core_codes::INVALID_TYPE));
    None
}

/// `minecraft:render_offsets`, kept as written.
fn render_offsets(slice: JsonSlice<'_>, _: &LoadContext, _: &mut Diagnostics) -> Option<Component> {
    bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text)
        .ok()
        .map(|value| Component::RenderOffsets(RenderOffsets { value }))
}

/// `minecraft:custom_components` of 1.20.80–1.21.50: a list of names.
fn custom_list(slice: JsonSlice<'_>, _: &LoadContext, d: &mut Diagnostics) -> Option<Component> {
    match parse_at::<Vec<String>>(slice) {
        Ok(names) => {
            // An element that is no string is the name facet makes of it (`7` is `"7"`), told
            // at the element as an information of the crate's own; it is not known whether
            // the game's non-strict reader coerces a number to a string or fails.
            let json = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).ok();
            for (index, element) in json
                .iter()
                .filter_map(Value::as_array)
                .flatten()
                .enumerate()
            {
                if !element.is_string() {
                    let path = JsonPath::new().index(index);
                    d.push(
                        Diagnostic::of(&core_codes::INVALID_TYPE)
                            .severity(Severity::Info)
                            .at(path),
                    );
                }
            }
            Some(Component::CustomComponentList(CustomComponentList {
                names,
            }))
        }
        Err(error) => {
            // No list: a type error, the item refused.
            d.push(
                Diagnostic::error(
                    bedrock_addon_core::diagnostic::code_for_deserialize_error_kind(
                        &error.error.kind,
                        true,
                        false,
                    ),
                )
                .at(error.path)
                .span(error.span),
            );
            None
        }
    }
}

/// `minecraft:custom_components` of 1.21.60–1.26.0: `[name]` or `[{name, data}]`.
fn custom_entries(slice: JsonSlice<'_>, _: &LoadContext, d: &mut Diagnostics) -> Option<Component> {
    let json = bedrock_addon_core::json::from_str_jsonc::<Value>(slice.text).ok();
    let Some(array) = json.as_ref().and_then(Value::as_array) else {
        // No list: a type error, the item refused.
        d.push(Diagnostic::of(&core_codes::INVALID_TYPE));
        return None;
    };
    let mut components = Vec::new();
    // The file's index of each entry held: the check of the names counts the list held.
    let mut kept = Vec::new();
    for (i, entry) in array.iter().enumerate() {
        let parsed = if let Some(name) = entry.as_string() {
            Some(CustomComponentEntry {
                name: name.as_str().to_owned(),
                data: Value::from(VObject::new()),
            })
        } else {
            entry.as_object().and_then(|object| {
                let name = object.get("name")?.as_string()?.as_str().to_owned();
                let data = object
                    .get("data")
                    .cloned()
                    .unwrap_or_else(|| Value::from(VObject::new()));
                // An entry holds its name and its data: any other member has no place
                // in it, and is told where it is: a member the schema does not know, which
                // this chain's error mask leaves a warning-level line while the item loads.
                for (member, _) in object
                    .iter()
                    .filter(|(member, _)| !["name", "data"].contains(&member.as_str()))
                {
                    let path = JsonPath::new().index(i).child(member.as_str());
                    d.push(
                        Diagnostic::of(&core_codes::MEMBER_NOT_KEPT)
                            .severity(Severity::Warning)
                            .at(path),
                    );
                }
                Some(CustomComponentEntry { name, data })
            })
        };
        if let Some(entry) = parsed {
            components.push(entry);
            kept.push(i);
        } else {
            // Neither shape: a type error, the item refused.
            let path = JsonPath::new().index(i);
            d.push(Diagnostic::of(&core_codes::INVALID_TYPE).at(path));
        }
    }
    // `d` is the component's own list, whose paths are below the component: the list is its
    // root. Its moves stay on it while the component is checked, and are not carried into
    // the document's findings (each finding is moved below the component one by one), so a
    // check of the names asks for the file's index itself.
    d.left_out(&JsonPath::new(), &kept);
    Some(Component::CustomComponents(CustomComponents { components }))
}

/// Whether a 1.16.100–1.20.20 struct (the dynamic map of every registered component) binds
/// `key`, with its parser.
fn legacy_map(version: FormatVersion, key: &str) -> Option<Parser> {
    Some(match key {
        "minecraft:armor" if version < V1_20_20 => typed::<Armor>,
        "minecraft:chargeable" => typed::<Chargeable>,
        "minecraft:on_use" => typed::<OnUse>,
        "minecraft:on_use_on" => typed::<OnUseOn>,
        "minecraft:render_offsets" => render_offsets,
        "minecraft:weapon" => typed::<Weapon>,
        "minecraft:use_duration" => bare_value::<UseDuration>,
        "minecraft:food" => typed::<LegacyFoodComponent>,
        "minecraft:durability" if version < FormatVersion::new(1, 19, 83) => {
            typed::<LegacyDurability>
        }
        "minecraft:repairable" if version < FormatVersion::new(1, 19, 83) => {
            typed::<LegacyRepairable>
        }
        "minecraft:digger" => typed::<LegacyDigger>,
        "minecraft:icon" => bare_value::<LegacyIcon>,
        "minecraft:storage_item" => typed::<LegacyStorageItem>,
        "minecraft:use_modifiers" | "minecraft:custom_components" => return None,
        other => return common(V1_21_90, other),
    })
}

/// The binding of a component whose shape is the newest one, in a fixed set of `version`.
fn common(version: FormatVersion, key: &str) -> Option<Parser> {
    Some(match key {
        "minecraft:allow_off_hand" if version >= BETA_VERSION => beta_allow_off_hand,
        "minecraft:allow_off_hand" => value::<AllowOffHand>,
        "minecraft:block_placer" => typed::<BlockPlacer>,
        "minecraft:can_destroy_in_creative" => value::<CanDestroyInCreative>,
        "minecraft:cooldown" => typed::<Cooldown>,
        "minecraft:damage" => value::<Damage>,
        "minecraft:digger" => typed::<Digger>,
        "minecraft:display_name" => typed::<DisplayName>,
        "minecraft:durability" => typed::<Durability>,
        "minecraft:enchantable" => typed::<Enchantable>,
        "minecraft:entity_placer" => typed::<EntityPlacer>,
        "minecraft:food" => typed::<Food>,
        "minecraft:fuel" => value::<Fuel>,
        "minecraft:glint" => value::<Glint>,
        "minecraft:hand_equipped" => value::<HandEquipped>,
        "minecraft:hover_text_color" => value::<HoverTextColor>,
        "minecraft:icon" => bare_value::<Icon>,
        "minecraft:interact_button" => interact_button,
        "minecraft:liquid_clipped" => value::<LiquidClipped>,
        "minecraft:max_stack_size" => value::<MaxStackSize>,
        "minecraft:projectile" => typed::<Projectile>,
        "minecraft:record" => typed::<Record>,
        "minecraft:repairable" => typed::<Repairable>,
        "minecraft:shooter" => typed::<Shooter>,
        "minecraft:should_despawn" => value::<ShouldDespawn>,
        "minecraft:stacked_by_data" => value::<StackedByData>,
        "minecraft:tags" => typed::<Tags>,
        "minecraft:throwable" => typed::<Throwable>,
        "minecraft:use_animation" => value::<UseAnimation>,
        "minecraft:use_modifiers" => typed::<UseModifiers>,
        "minecraft:wearable" => typed::<Wearable>,
        "minecraft:damage_absorption" if version >= V1_21_10 => typed::<DamageAbsorption>,
        "minecraft:durability_sensor" if version >= V1_21_10 => typed::<DurabilitySensor>,
        "minecraft:bundle_interaction" if version >= V1_21_30 => typed::<BundleInteraction>,
        "minecraft:dyeable" if version >= V1_21_30 => typed::<Dyeable>,
        "minecraft:rarity" if version >= V1_21_30 => value::<Rarity>,
        "minecraft:storage_item" if version >= V1_21_60 => typed::<StorageItem>,
        "minecraft:storage_item" if version >= V1_21_30 => typed::<LegacyStorageItem>,
        "minecraft:compostable" if version >= V1_21_50 => typed::<Compostable>,
        "minecraft:storage_weight_limit" if version >= V1_21_60 => typed::<StorageWeightLimit>,
        "minecraft:storage_weight_modifier" if version >= V1_21_60 => {
            typed::<StorageWeightModifier>
        }
        "minecraft:kinetic_weapon" if version >= V1_21_90 => typed::<KineticWeapon>,
        "minecraft:piercing_weapon" if version >= V1_21_90 => typed::<PiercingWeapon>,
        "minecraft:fire_resistant" if version >= V1_21_90 => typed::<FireResistant>,
        "minecraft:swing_duration" if version >= V1_21_90 => value::<SwingDuration>,
        "minecraft:swing_sounds" if version >= V1_21_90 => typed::<SwingSounds>,
        "minecraft:custom_components" if version >= V1_26_30 => return None,
        "minecraft:custom_components" if version >= V1_21_60 => custom_entries,
        "minecraft:custom_components" if version >= V1_20_80 => custom_list,
        _ => return None,
    })
}

/// The component `key` the struct of `version` binds, with its parser; `None` for a key the
/// format does not bind.
pub(crate) fn binding(version: FormatVersion, key: &str) -> Option<Parser> {
    if version < V1_20_30 {
        return legacy_map(version, key);
    }
    let fixed_1_20_30 = |key: &str| -> Option<Parser> {
        Some(match key {
            "minecraft:chargeable" => typed::<Chargeable>,
            "minecraft:use_duration" => bare_value::<UseDuration>,
            "minecraft:digger" => typed::<LegacyDigger>,
            "minecraft:icon" => bare_value::<LegacyIcon>,
            "minecraft:on_use" if version < V1_20_40 => typed::<OnUse>,
            "minecraft:on_use_on" if version < V1_20_40 => typed::<OnUseOn>,
            "minecraft:render_offsets" if version < V1_20_40 => render_offsets,
            "minecraft:weapon" if version < V1_20_40 => typed::<Weapon>,
            "minecraft:allow_off_hand"
            | "minecraft:block_placer"
            | "minecraft:can_destroy_in_creative"
            | "minecraft:cooldown"
            | "minecraft:damage"
            | "minecraft:display_name"
            | "minecraft:durability"
            | "minecraft:enchantable"
            | "minecraft:entity_placer"
            | "minecraft:food"
            | "minecraft:fuel"
            | "minecraft:glint"
            | "minecraft:hand_equipped"
            | "minecraft:hover_text_color"
            | "minecraft:interact_button"
            | "minecraft:liquid_clipped"
            | "minecraft:max_stack_size"
            | "minecraft:projectile"
            | "minecraft:record"
            | "minecraft:repairable"
            | "minecraft:shooter"
            | "minecraft:should_despawn"
            | "minecraft:stacked_by_data"
            | "minecraft:throwable"
            | "minecraft:use_animation"
            | "minecraft:wearable" => return common(version, key),
            _ => return None,
        })
    };
    if version < V1_20_50 {
        return fixed_1_20_30(key);
    }
    if version < V1_20_60 && key == "minecraft:icon" {
        return Some(bare_value::<LegacyIcon>);
    }
    common(version, key)
}

/// What the parse shell reads of an item object.
#[derive(Facet)]
#[facet(schema::unknown = "keep")]
struct Shell<'a> {
    description: Description,
    components: Option<IndexMap<String, RawJson<'a>>>,
    #[facet(rename = "__deprecated_components")]
    deprecated: Option<Value>,
    deprecated_component_data: Option<Value>,
    legacy_data: Option<Value>,
    events: Option<Value>,
}

/// One component-format struct as parsed: the parts every struct holds.
pub(crate) struct Parts {
    pub(crate) description: Description,
    pub(crate) components: Components,
    pub(crate) deprecated: ItemDeprecatedData,
    pub(crate) events: Option<Value>,
    pub(crate) legacy: LegacyItemData,
}

/// Parses a component-format item object as the struct of `version` binds it: the item-level
/// and per-component gates, each bound component from its slice, the legacy read,
/// the side-car and the era's unknown-member rule. `None` when the document is rejected.
#[allow(
    clippy::too_many_lines,
    reason = "the gates, the components, the legacy read and the side-car in vanilla's order"
)]
pub(crate) fn parse_parts(
    payload: JsonSlice<'_>,
    version: FormatVersion,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<Parts> {
    let original = cx.original_version;
    let strict = strict(original);
    let unknown = cx.load.grade(Class::Unknown);
    let mut found = Diagnostics::default();
    let Some(shell) = parse_payload::<Shell<'_>>(payload, cx, &mut found) else {
        d.extend_with_unknown(found);
        return None;
    };
    let mut description = shell.description;
    let at_components = cx.at.child("components");
    if version >= V1_26_30 {
        // The modern documents bind `components` as required, with at least one entry. A
        // stable document without a registered component is routed to the legacy format
        // before this; a beta document is not routed.
        let missing = match &shell.components {
            None => Some(Code::from(&core_codes::MISSING_REQUIRED_FIELD)),
            Some(components) if components.is_empty() => Some(
                bedrock_addon_core::diagnostic::cereal::too_few_components(1),
            ),
            Some(_) => None,
        };
        if let Some(code) = missing {
            found.push(Diagnostic::error(code).at(at_components.clone()));
        }
    }
    let mut raw = shell.components.unwrap_or_default();
    // A component written as `null`: the slice's null rule. The generic null rules read
    // struct members; a component is an entry of the `components` map, so the item applies
    // the rule here. The legacy chain reads its components as members of the item data
    // through the jsoncpp reader, which takes `null` as absent, with no word. A modern
    // slice's component storage reads the value, and `null` fails its type: an error that
    // rejects the item.
    let is_null = |json: &RawJson<'_>| {
        bedrock_addon_core::json::from_str_jsonc::<Value>(json.as_str()).is_ok_and(|v| v.is_null())
    };
    match cx.load.null_rule {
        Some(NullRule::Absent) => raw.retain(|_, json| !is_null(json)),
        Some(NullRule::Rejected) => {
            for key in raw.keys().filter(|key| is_null(&raw[*key])) {
                found.push(Diagnostic::of(&core_codes::INVALID_TYPE).at(at_components.child(key)));
            }
            raw.retain(|_, json| !is_null(json));
        }
        _ => {}
    }
    // The values, for the legacy read and the gates.
    let values: IndexMap<String, Value> = raw
        .iter()
        .map(|(key, json)| {
            let value = bedrock_addon_core::json::from_str_jsonc::<Value>(json.as_str())
                .unwrap_or(Value::NULL);
            (key.clone(), value)
        })
        .collect();
    let identifier = description.identifier.clone();
    let Some(dropped) = gate(
        &values,
        &identifier,
        original,
        cx.load,
        &at_components,
        &mut found,
    ) else {
        d.extend_with_unknown(found);
        return None;
    };
    let mut components = Components::new();
    for (key, json) in &raw {
        if dropped.contains(key) {
            continue;
        }
        let path = at_components.child(key);
        let slice = payload.nested(json).unwrap_or(JsonSlice {
            text: json.as_str(),
            offset: payload.offset,
        });
        if let Some(parser) = binding(version, key) {
            // The component's own findings, below it. Iterated into `found` one by one, so
            // that what it holds beside them (a reader's moves) stays here.
            let mut own = Diagnostics::default();
            let mut component = parser(slice, cx.load, &mut own);
            if let Some(c) = component.as_mut() {
                unbind_members(key, c, &values[key], version, cx.load, &mut own);
                if check_component(key, c, version, original, cx.load, &mut own) {
                    shared_checks(c, unknown, &mut own);
                    level_sound_events(c, version, cx.load, &mut own);
                } else {
                    component = None;
                }
            }
            // An unknown member is left out of the errors: a component dropped with no other
            // error is still summarised.
            if component.is_none()
                && !own
                    .iter()
                    .any(|told| told.rejects() && !is_unknown_member(told))
                && !own.iter().any(|told| told.location.json_path.is_empty())
            {
                // Also where a member below the component is told (one the crate's own
                // checks refuse): its finding does not say that the component goes whole.
                // The game writes no line for the dropped component, so this summary is the
                // crate's own information.
                own.push(Diagnostic::of(&core_codes::DROPPED_COMPONENT));
            }
            let kept = own.take_unknown();
            for mut diagnostic in own {
                let mut full = path.clone();
                full.append(&diagnostic.location.json_path);
                diagnostic.location.json_path = full;
                found.push(diagnostic);
            }
            if let Some(component) = component {
                // The component's unknown members, below its key.
                found.unknown_mut().absorb(&path, kept);
                components.insert(key.clone(), component);
            }
        } else if is_legacy_read(key) {
            // read by the legacy reader below; no component of this format
        } else if is_custom_key(key) && original >= V1_21_90 && version >= V1_21_90 {
            // The list form of `custom_components` is replaced by keyed custom components
            // from 1.21.90; it is inferred that the game compares the file's own version.
            components.insert(
                key.clone(),
                Component::Custom(bedrock_addon_core::document::CustomComponent {
                    name: key.clone(),
                    parameters: values[key].clone(),
                    from_v1_list: false,
                }),
            );
        } else {
            components.insert(key.clone(), Component::Unknown(values[key].clone()));
            // From 1.26.30 an unknown member is an error of the strict slice (its grade of an
            // unknown member). Below it a registered name this slice does not bind (a
            // component of a later format) stays in the document and is a member the schema
            // does not know, a warning-level line in the content log; a name no component ever
            // had is removed first without a line, so the crate reports it as an information.
            let registered = crate::component::ItemComponent::TABLE.get(key).is_some();
            let severity = if version >= V1_26_30 || registered {
                unknown
            } else {
                Severity::Info
            };
            found.push(
                Diagnostic::of(&core_codes::UNRECOGNIZED_COMPONENT)
                    .severity(severity)
                    .at(path),
            );
        }
    }
    if version >= BETA_VERSION && components.is_empty() && !raw.is_empty() {
        // A beta document whose every component the load dropped holds none, which the beta
        // struct's minimum of one component refuses: the item is refused here rather than
        // written as a document that does not load (in the game the strict slice refuses the
        // failing component itself).
        found.push(
            Diagnostic::error(bedrock_addon_core::diagnostic::cereal::too_few_components(
                1,
            ))
            .at(at_components.clone()),
        );
    }
    let legacy_category = description.category.clone();
    let mut legacy = read_legacy(
        Some(&values),
        legacy_category,
        original,
        cx.load,
        &at_components,
        &mut found,
    );
    // the legacy members' Molang (rule 24c, 24d): the kept components' triggers, the side-car a
    // struct of 1.20.30–1.26.0 reads, and `events`
    let kept: Vec<(&str, &Value)> = values
        .iter()
        .filter(|(key, _)| components.contains_key(*key))
        .map(|(key, value)| (key.as_str(), value))
        .collect();
    let read_side_car = (V1_20_30..V1_26_30).contains(&version);
    legacy.expressions = crate::expressions::Expressions::read(
        shell.events.as_ref(),
        &kept,
        shell.deprecated.as_ref().filter(|_| read_side_car),
        bedrock_addon_core::molang::format_molang_version(&cx.written),
        &cx.at,
        &mut found,
    );
    // A component the legacy reader takes and this struct binds nothing for is held in the
    // legacy reading alone: marked, so that the writers carry it.
    legacy.held_in_shared_alone = legacy
        .shared
        .keys()
        .filter(|key| binding(version, key).is_none())
        .cloned()
        .collect();
    if version >= V1_26_30 && shell.events.is_some() {
        // The 1.26.30 format and the newer ones bind `description`, `components` and the
        // two deprecated side-car names alone: `events` is an unknown member there, an error
        // in the strict slice. Held, and written back.
        found.push(
            Diagnostic::of(&core_codes::UNRECOGNIZED_FIELD)
                .severity(unknown)
                .at(cx.at.child("events")),
        );
    }
    let deprecated = side_car(
        shell.deprecated,
        shell.deprecated_component_data,
        shell.legacy_data,
        version,
        &cx.at,
        unknown,
        &mut found,
    );
    description_era(
        &mut description,
        version,
        original,
        &cx.at.child("description"),
        unknown,
        &mut found,
    );
    if !strict {
        // Not told, and kept all the same.
        found.retain(|f| f.kind() != Kind::UnknownMember(Disposition::Kept));
    }
    d.extend_with_unknown(found);
    Some(Parts {
        description,
        components,
        deprecated,
        events: shell.events,
        legacy,
    })
}

/// The sound members a struct binds as a `LevelSoundEvent` take only a level sound event's
/// name, by the context's provider (compared without case: it is not known whether the game
/// folds case, so the lenient reading is taken); any other text is an enum error that
/// rejects the item. Below 1.26.50
/// these are `use_modifiers.start_sound`, the three `swing_sounds` and each
/// `durability_sensor` threshold's `sound_event`; the stable `record.sound_event` is one at
/// every stable version. The beta struct binds all five as sound names.
fn level_sound_events(
    component: &Component,
    version: FormatVersion,
    load: &LoadContext,
    d: &mut Diagnostics,
) {
    let stable = version < BETA_VERSION;
    let enum_before_1_26_50 = version < V1_26_50;
    let mut check = |name: Option<&SoundEventName>, path: JsonPath| {
        if let Some(name) = name
            && load.level_sound_event(name.as_str()).is_none()
        {
            d.push(Diagnostic::of(&core_codes::INVALID_ENUM_VALUE).at(path));
        }
    };
    let at = |members: &[&str]| {
        let mut path = JsonPath::new();
        for member in members {
            path.push(member);
        }
        path
    };
    match component {
        Component::Record(record) if stable => {
            check(record.sound_event.as_ref(), at(&["sound_event"]));
        }
        Component::UseModifiers(modifiers) if enum_before_1_26_50 => {
            check(modifiers.start_sound.as_ref(), at(&["start_sound"]));
        }
        Component::SwingSounds(swing) if enum_before_1_26_50 => {
            for (key, name) in [
                ("attack_miss", &swing.attack_miss),
                ("attack_hit", &swing.attack_hit),
                ("attack_critical_hit", &swing.attack_critical_hit),
            ] {
                check(name.as_ref(), at(&[key]));
            }
        }
        Component::DurabilitySensor(sensor) if enum_before_1_26_50 => {
            for (index, threshold) in sensor.durability_thresholds.iter().enumerate() {
                let mut path = at(&["durability_thresholds"]);
                path.push_index(index);
                path.push("sound_event");
                check(threshold.sound_event.as_ref(), path);
            }
        }
        _ => {}
    }
}

/// Whether `key` names a custom component: `ns:name` outside the `minecraft` namespace (a
/// `tag:` key is the legacy tag form).
pub(crate) fn is_custom_key(key: &str) -> bool {
    match key.split_once(':') {
        Some((namespace, name)) => {
            !namespace.is_empty()
                && !name.is_empty()
                && namespace != "minecraft"
                && namespace != "tag"
        }
        None => false,
    }
}

/// The side-car members of the item object at `version`, an unknown member at `unknown`.
fn side_car(
    written: Option<Value>,
    data: Option<Value>,
    legacy_data: Option<Value>,
    version: FormatVersion,
    at: &JsonPath,
    unknown: Severity,
    d: &mut Diagnostics,
) -> ItemDeprecatedData {
    let mut out = ItemDeprecatedData::default();
    if version >= V1_26_30 {
        // The 1.26.30 format binds the side-car only as the deprecated names
        // `deprecated_component_data` / `legacy_data`: `__deprecated_components` is an
        // unknown member.
        if let Some(value) = written {
            d.unknown_mut().put(at, "__deprecated_components", value);
            d.push(
                Diagnostic::of(&core_codes::UNRECOGNIZED_FIELD)
                    .severity(unknown)
                    .at(at.child("__deprecated_components")),
            );
        }
        for (key, value, code) in [
            (
                "deprecated_component_data",
                data,
                &codes::DEPRECATED_COMPONENT_DATA_DEPRECATED,
            ),
            ("legacy_data", legacy_data, &codes::LEGACY_DATA_DEPRECATED),
        ] {
            if value.is_some() {
                // A deprecated member is an error under both error masks and rejects the
                // item.
                d.push(Diagnostic::error(code).at(at.child(key)));
            }
        }
        return out;
    }
    for (key, value) in [
        ("deprecated_component_data", data),
        ("legacy_data", legacy_data),
    ] {
        if let Some(value) = value {
            // Bound by the 1.26.30 documents alone: here a member the schema does not
            // know, a warning-level line in the content log while the item loads.
            d.unknown_mut().put(at, key, value);
            d.push(
                Diagnostic::of(&core_codes::UNRECOGNIZED_FIELD)
                    .severity(unknown)
                    .at(at.child(key)),
            );
        }
    }
    if let Some(value) = written {
        let path = at.child("__deprecated_components");
        if version >= V1_20_30 {
            // The 1.20.30–1.26.0 roots bind the member as deprecated: the game logs
            // "member \"{}\" is deprecated" when the member is in a JSON input. That
            // deprecation line is an error under every error mask, so the item is refused
            // and the finding is an error. The 1.26.30 names above are graded the same.
            d.push(Diagnostic::of(&codes::DEPRECATED_COMPONENTS_DEPRECATED).at(path.clone()));
            out = ItemDeprecatedData::from_value(&value, &path, d);
        } else {
            // No struct below 1.20.30 binds the member: the game does not read it. The crate
            // does not hold it either, since the written item has one member of this name,
            // the one the steps fill; it is noted where it is.
            let _ = value;
            d.push(Diagnostic::of(&core_codes::DROPPED_FIELD).at(path));
        }
    }
    out
}

/// Whether `finding` tells an unknown member or component.
fn is_unknown_member(finding: &Diagnostic) -> bool {
    finding.kind() == Kind::UnknownMember(Disposition::Kept)
}

/// The description members `version` binds: `category` up to 1.20.0, `menu_category` from
/// 1.20.20, with the creative category's default of the era, a member of the other era told at
/// `severity`.
fn description_era(
    description: &mut Description,
    version: FormatVersion,
    original: FormatVersion,
    at: &JsonPath,
    severity: Severity,
    d: &mut Diagnostics,
) {
    // A description member of the other era is a member the schema does not know, at
    // `severity`: a warning-level line in the content log under the legacy chain while the
    // item loads; from 1.26.30 the strict slice's error.
    let unknown = |d: &mut Diagnostics, key: &str| {
        d.push(
            Diagnostic::of(&core_codes::UNRECOGNIZED_FIELD)
                .severity(severity)
                .at(at.child(key)),
        );
    };
    if version < V1_20_20 {
        if let Some(menu) = description.menu_category.take() {
            if let Ok(value) = bedrock_addon_core::versioning::write_payload(&menu) {
                d.unknown_mut().put(at, "menu_category", value);
            }
            unknown(d, "menu_category");
        }
        return;
    }
    if let Some(category) = description.category.take() {
        d.unknown_mut()
            .put(at, "category", Value::from(category.as_str()));
        unknown(d, "category");
    }
    if let Some(menu) = description.menu_category.as_mut() {
        if version >= V1_21_60 && version < V1_21_110 {
            // The 1.21.60 menu category's default is Undefined, and "none" reads as it.
            if matches!(
                menu.category,
                None | Some(CreativeCategory::ItemCommandOnly)
            ) {
                menu.category = Some(CreativeCategory::Undefined);
            }
        }
        // From 1.21.110 an absent `category` stays absent here: the newest struct fills in
        // `none` and records that the document did not write it.
        if let Some(group) = &menu.group {
            if group.chars().count() > 256 {
                d.push(
                    Diagnostic::of(&codes::GROUP_LENGTH)
                        .at(at.child("menu_category").child("group")),
                );
            } else if version >= V1_21_60 && !bedrock_addon_core::id::syntax::has_namespace(group) {
                // A group without a namespace fails to load from 1.21.60.
                d.push(
                    Diagnostic::of(&codes::GROUP_NAMESPACE)
                        .at(at.child("menu_category").child("group")),
                );
            }
        }
    }
    let _ = original;
}

/// Members the struct of `version` does not bind yet (`replace_block_item` before 1.21.40,
/// `aligned_placement` before 1.26.0, `hides_player_location` before 1.21.90, `start_using`
/// before 1.26.30), and members whose engine version is above the load context's: reset and
/// kept as unknown members of the component's object.
fn unbind_members(
    key: &str,
    component: &mut Component,
    raw: &Value,
    version: FormatVersion,
    cx: &LoadContext,
    d: &mut Diagnostics,
) {
    let Some(object) = raw.as_object() else {
        return;
    };
    let mut unbound: Vec<(&'static str, bool)> = Vec::new();
    match component {
        Component::BlockPlacer(c) => {
            if version < V1_21_40 && object.contains_key("replace_block_item") {
                c.replace_block_item = None;
                unbound.push(("replace_block_item", false));
            }
            if object.contains_key("aligned_placement") {
                if version < V1_26_0 {
                    c.aligned_placement = None;
                    unbound.push(("aligned_placement", false));
                } else if cx.engine_version < known::V1_26_10 {
                    c.aligned_placement = None;
                    unbound.push(("aligned_placement", true));
                }
            }
        }
        Component::Wearable(c) => {
            if version < V1_21_90 && object.contains_key("hides_player_location") {
                c.hides_player_location = None;
                unbound.push(("hides_player_location", false));
            }
        }
        Component::UseModifiers(c) => {
            if version < V1_26_30 && object.contains_key("start_using") {
                c.start_using = crate::component::StartUsing::IfFirst;
                unbound.push(("start_using", false));
            }
        }
        Component::Cooldown(c)
            if object.contains_key("type") && cx.engine_version < known::V1_21_130 =>
        {
            c.kind = crate::component::CooldownType::Use;
            unbound.push(("type", true));
        }
        _ => {}
    }
    let _ = key;
    for (member, engine) in unbound {
        if let Some(value) = object.get(member)
            && keeps_unknown(component)
        {
            // Below the component, which its parser's caller places.
            d.unknown_mut().put(&JsonPath::new(), member, value.clone());
        }
        let code = if engine {
            let since = if member == "type" {
                known::V1_21_130
            } else {
                known::V1_26_10
            };
            core_codes::FIELD_SINCE_ENGINE.with([since.to_string()])
        } else {
            Code::from(&core_codes::UNRECOGNIZED_FIELD)
        };
        let path = JsonPath::new().child(member);
        // A member the struct of this version does not bind is a member the schema does not
        // know: a warning-level line in the content log while the item loads.
        // `FieldSinceEngine` is a gate on the engine version, not an unknown member: the
        // current game binds every member predicted for an older engine, so that line is
        // never written for it and the game is silent: the crate reports an information.
        let severity = if engine {
            Severity::Info
        } else {
            Severity::Warning
        };
        d.push(Diagnostic::error(code).severity(severity).at(path));
    }
}

/// Whether a typed component keeps the members this version's struct does not bind as
/// unknown members of its object (in the document's `Unknown` table; each of these structs says
/// `schema::unknown = "keep"`).
fn keeps_unknown(component: &Component) -> bool {
    matches!(
        component,
        Component::BlockPlacer(_)
            | Component::Wearable(_)
            | Component::UseModifiers(_)
            | Component::Cooldown(_)
    )
}

/// The binding constraints and value rules of one component (and the 1.18 shapes): `false`
/// drops the component (its finding says why). A binding constraint that fails is an error
/// under both error masks, after which the game logs " Error Parsing Item" and refuses the
/// item: its finding is an error. The constraints the game binds are: fuel `duration` min,
/// damage 0..32767, durability `max_durability` min and `damage_chance` range, enchantable
/// `value` min and `slot` regex, `use_modifiers` `use_duration` min and `movement_modifier`
/// range and `max_stack_size` `value` range (`v1_20_50`), wearable `protection` min,
/// `swing_duration` `value` min, `storage_item` `max_slots` max, `storage_weight_limit` max
/// and `storage_weight_modifier` min (`v1_21_60`). A check whose game constraint is not
/// known (the legacy `use_duration`, kinetic speeds, compostable, the 1.21.30 `storage_item`,
/// `bundle_interaction`, `damage_absorption`, `durability_sensor`, `block_placer` `use_on`)
/// is a check of the crate's own: the component is dropped with an information.
#[allow(clippy::too_many_lines, reason = "one arm per component rule")]
fn check_component(
    key: &str,
    component: &mut Component,
    version: FormatVersion,
    original: FormatVersion,
    _: &LoadContext,
    d: &mut Diagnostics,
) -> bool {
    let told = |d: &mut Diagnostics, member: &str, severity: Severity| {
        let mut path = JsonPath::new();
        for part in member.split('/').filter(|p| !p.is_empty()) {
            path.push(part);
        }
        d.push(
            Diagnostic::of(&core_codes::INVALID_VALUE)
                .severity(severity)
                .at(path),
        );
        false
    };
    // A constraint the game binds: a failed constraint, the item refused.
    let invalid = |d: &mut Diagnostics, member: &str| told(d, member, Severity::Error);
    // A check of the crate's own whose game constraint is not known: dropped, an
    // information.
    let unverified = |d: &mut Diagnostics, member: &str| told(d, member, Severity::Info);
    // The bounds the component's type states, each a constraint the game binds, or each a
    // check of the crate's own.
    let bound = |d: &mut Diagnostics, member: Option<JsonPath>, severity: Severity| match member {
        Some(member) => {
            d.push(
                Diagnostic::of(&core_codes::INVALID_VALUE)
                    .severity(severity)
                    .at(member),
            );
            false
        }
        None => true,
    };
    let _ = key;
    match component {
        Component::MaxStackSize(c) => bound(d, out_of_bounds(c), Severity::Error),
        Component::Fuel(c) => bound(d, out_of_bounds(c), Severity::Error),
        Component::Damage(c) => bound(d, out_of_bounds(c), Severity::Error),
        Component::Durability(c) => {
            let chance = c
                .damage_chance
                .as_ref()
                .is_some_and(|r| r.min < 0 || r.max > 100);
            if c.max_durability < 0 {
                invalid(d, "max_durability")
            } else if chance {
                invalid(d, "damage_chance")
            } else {
                true
            }
        }
        Component::Enchantable(c) => {
            if let Some(member) = out_of_bounds(c) {
                return bound(d, Some(member), Severity::Error);
            }
            if !crate::component::ENCHANT_SLOTS.contains(&c.slot.as_str()) {
                // The slot's regex constraint fails ("expected string to match the …
                // regex"): the item is refused. The game's `none` text is reached only by a
                // name the regex takes and the slot parser does not know; the text is kept as
                // the finding's (the game's own constraint text is not known).
                let slot = std::mem::replace(&mut c.slot, "none".to_owned());
                let path = JsonPath::new().child("slot");
                d.push(Diagnostic::error(codes::ENCHANT_SLOT.with([slot])).at(path));
                return false;
            }
            true
        }
        Component::UseModifiers(c) => {
            if c.use_duration < 0.0 {
                invalid(d, "use_duration")
            } else if c
                .movement_modifier
                .is_some_and(|m| !(0.0..=1.0).contains(&m))
            {
                invalid(d, "movement_modifier")
            } else {
                true
            }
        }
        Component::UseDuration(c) => bound(d, out_of_bounds(c), Severity::Info),
        Component::Wearable(c) => bound(d, out_of_bounds(c), Severity::Error),
        Component::SwingDuration(c) => bound(d, out_of_bounds(c), Severity::Error),
        Component::KineticWeapon(c) => {
            for (name, condition) in [
                ("damage_conditions", &c.damage_conditions),
                ("knockback_conditions", &c.knockback_conditions),
                ("dismount_conditions", &c.dismount_conditions),
            ] {
                // (Told at the condition, as the crate's own check.)
                if condition.as_ref().and_then(out_of_bounds).is_some() {
                    return unverified(d, name);
                }
            }
            true
        }
        Component::Compostable(c) => bound(d, out_of_bounds(c), Severity::Info),
        Component::StorageItem(c) if c.max_slots > 64 => invalid(d, "max_slots"),
        Component::LegacyStorageItem(c) => {
            if c.max_slots > 64 {
                unverified(d, "max_slots")
            } else if c.max_weight_limit > 64 {
                unverified(d, "max_weight_limit")
            } else if c.weight_in_storage_item < 0 {
                unverified(d, "weight_in_storage_item")
            } else {
                true
            }
        }
        Component::StorageWeightLimit(c) => bound(d, out_of_bounds(c), Severity::Error),
        Component::StorageWeightModifier(c) => bound(d, out_of_bounds(c), Severity::Error),
        Component::BundleInteraction(c) => bound(d, out_of_bounds(c), Severity::Info),
        Component::DamageAbsorption(c) if c.absorbable_causes.is_empty() => {
            unverified(d, "absorbable_causes")
        }
        Component::DurabilitySensor(c) if c.durability_thresholds.is_empty() => {
            unverified(d, "durability_thresholds")
        }
        Component::BlockPlacer(c) if version >= V1_26_0 => {
            // From 1.26.0 `use_on` holds at most 256 entries, without duplicates.
            let duplicate = c
                .use_on
                .iter()
                .enumerate()
                .any(|(i, a)| c.use_on[..i].contains(a));
            if c.use_on.len() > 256 || duplicate {
                unverified(d, "use_on")
            } else {
                true
            }
        }
        Component::CustomComponentList(c) => {
            drop_minecraft_names(&mut c.names, |n| n, d);
            true
        }
        Component::CustomComponents(c) => {
            drop_minecraft_names(&mut c.components, |e| &e.name, d);
            true
        }
        Component::LegacyFood(c) if original >= V1_18_0 => {
            if c.saturation_modifier.as_ref().is_some_and(Value::is_string) {
                c.saturation_modifier = None;
                let _ = invalid(d, "saturation_modifier");
            }
            true
        }
        Component::LegacyDurability(c) if original >= V1_18_0 => {
            if c.max_damage.take().is_some() {
                // The 1.18 slices bind `max_durability` alone: `max_damage` is a member the
                // schema does not know, a warning-level line in the content log, and the
                // item loads.
                let path = JsonPath::new().child("max_damage");
                d.push(
                    Diagnostic::of(&core_codes::INVALID_VALUE)
                        .severity(Severity::Warning)
                        .at(path),
                );
            }
            if c.max_durability.is_none() {
                // Required: a missing required member, the item refused.
                let path = JsonPath::new().child("max_durability");
                d.push(Diagnostic::of(&core_codes::MISSING_REQUIRED_FIELD).at(path));
                return false;
            }
            true
        }
        Component::LegacyRepairable(c) if original >= V1_18_0 => {
            if is_name_list(c.repair_items.as_ref()) {
                // The game reads a bare name as the entry's `items`, in silence. The crate
                // drops the list and tells it as an information of its own (whether to keep
                // it instead is open).
                c.repair_items = None;
                let path = JsonPath::new().child("repair_items");
                d.push(
                    Diagnostic::of(&core_codes::INVALID_VALUE)
                        .severity(Severity::Info)
                        .at(path),
                );
            }
            true
        }
        _ => true,
    }
}

/// The path below the component of the first member of `value` outside the bounds its type
/// states (`schema::min`, `schema::max`); `None` when every member is inside them. The rule
/// that calls it says how severe the finding is.
fn out_of_bounds<T: Schema + Clone>(value: &T) -> Option<JsonPath> {
    let mut found = Diagnostics::default();
    let mut value = value.clone();
    {
        let mut cx = SchemaCx::new(Reading::DOCUMENT, JsonPath::new(), &mut found);
        value.check(&mut cx);
    }
    found
        .iter()
        .next()
        .map(|finding| finding.location.json_path.clone())
}

/// A path of `parts` below a component (`"3"` and other digits are indexes).
fn at(parts: &[&str]) -> JsonPath {
    let mut path = JsonPath::new();
    for part in parts {
        match part.parse::<usize>() {
            Ok(index) => path.push_index(index),
            Err(_) => path.push(part),
        }
    }
    path
}

/// The findings of the shared value types a component holds (each reports its own unknown
/// members, at `unknown`, and shapes), at their paths below the component.
#[allow(
    clippy::too_many_lines,
    reason = "one arm per component holding shared values"
)]
fn shared_checks(component: &Component, unknown: Severity, d: &mut Diagnostics) {
    use bedrock_addon_core::shared::EventTrigger;
    use bedrock_addon_shared_shapes::{BlockDescriptor, ItemDescriptor};
    let items = |list: &[ItemDescriptor], key: &str, d: &mut Diagnostics| {
        for (i, item) in list.iter().enumerate() {
            d.extend(item.check(&at(&[key, &i.to_string()])));
        }
    };
    let blocks = |list: &[BlockDescriptor], key: &str, d: &mut Diagnostics| {
        for (i, block) in list.iter().enumerate() {
            d.extend(block.check(&at(&[key, &i.to_string()]), unknown));
        }
    };
    let trigger = |trigger: Option<&EventTrigger>, path: &[&str], d: &mut Diagnostics| {
        if let Some(trigger) = trigger {
            d.extend(trigger.check(&at(path), unknown));
        }
    };
    match component {
        Component::Digger(c) => {
            for (i, speed) in c.destroy_speeds.iter().enumerate() {
                let i = i.to_string();
                d.extend(
                    speed
                        .block
                        .check(&at(&["destroy_speeds", &i, "block"]), unknown),
                );
                trigger(
                    speed.deprecated_on_dig.as_ref(),
                    &["destroy_speeds", &i, "__deprecated_on_dig"],
                    d,
                );
            }
            trigger(c.deprecated_on_dig.as_ref(), &["__deprecated_on_dig"], d);
        }
        Component::LegacyDigger(c) => {
            for (i, speed) in c.destroy_speeds.iter().enumerate() {
                let i = i.to_string();
                d.extend(
                    speed
                        .block
                        .check(&at(&["destroy_speeds", &i, "block"]), unknown),
                );
                trigger(speed.on_dig.as_ref(), &["destroy_speeds", &i, "on_dig"], d);
            }
            trigger(c.on_dig.as_ref(), &["on_dig"], d);
        }
        Component::BlockPlacer(c) => {
            d.extend(c.block.check(&at(&["block"]), unknown));
            blocks(&c.use_on, "use_on", d);
        }
        Component::EntityPlacer(c) => {
            blocks(&c.use_on, "use_on", d);
            blocks(&c.dispense_on, "dispense_on", d);
        }
        Component::Repairable(c) => {
            for (i, entry) in c.repair_items.iter().enumerate() {
                for (j, item) in entry.items.iter().enumerate() {
                    d.extend(item.check(&at(&[
                        "repair_items",
                        &i.to_string(),
                        "items",
                        &j.to_string(),
                    ])));
                }
            }
        }
        Component::Shooter(c) => {
            for (i, ammunition) in c.ammunition.iter().enumerate() {
                d.extend(
                    ammunition
                        .item
                        .check(&at(&["ammunition", &i.to_string(), "item"])),
                );
            }
        }
        Component::Food(c) => {
            if let Some(item) = &c.using_converts_to {
                d.extend(item.check(&at(&["using_converts_to"])));
            }
        }
        Component::LegacyFood(c) => {
            if let Some(item) = &c.using_converts_to {
                d.extend(item.check(&at(&["using_converts_to"])));
            }
            trigger(c.on_consume.as_ref(), &["on_consume"], d);
        }
        Component::StorageItem(c) => {
            items(&c.banned_items, "banned_items", d);
            items(&c.allowed_items, "allowed_items", d);
        }
        Component::LegacyStorageItem(c) => {
            items(&c.banned_items, "banned_items", d);
            items(&c.allowed_items, "allowed_items", d);
        }
        Component::Durability(c) => {
            if let Some(range) = &c.damage_chance {
                d.extend(range.check(&at(&["damage_chance"])));
            }
        }
        Component::KineticWeapon(c) => {
            for (key, range) in [("reach", &c.reach), ("creative_reach", &c.creative_reach)] {
                if let Some(range) = range {
                    d.extend(range.check(&at(&[key])));
                }
            }
        }
        Component::PiercingWeapon(c) => {
            for (key, range) in [("reach", &c.reach), ("creative_reach", &c.creative_reach)] {
                if let Some(range) = range {
                    d.extend(range.check(&at(&[key])));
                }
            }
        }
        Component::Chargeable(c) => trigger(c.on_complete.as_ref(), &["on_complete"], d),
        Component::OnUse(c) => trigger(c.on_use.as_ref(), &["on_use"], d),
        Component::OnUseOn(c) => trigger(c.on_use_on.as_ref(), &["on_use_on"], d),
        Component::Weapon(c) => {
            trigger(c.on_hurt_entity.as_ref(), &["on_hurt_entity"], d);
            trigger(c.on_not_hurt_entity.as_ref(), &["on_not_hurt_entity"], d);
            trigger(c.on_hit_block.as_ref(), &["on_hit_block"], d);
        }
        _ => {}
    }
}

/// Whether `items` is a non-empty array whose first element is a string (the pre-1.18
/// `repair_items` form).
pub(crate) fn is_name_list(items: Option<&Value>) -> bool {
    items
        .and_then(Value::as_array)
        .and_then(|a| a.iter().next())
        .is_some_and(Value::is_string)
}

/// Drops the custom component entries in the `minecraft` namespace, each with an
/// `InvalidValue` error at the index the file has the entry at. Both list formats (1.20.80
/// and 1.21.60) bind the name with a constraint that disallows the `minecraft` namespace; a
/// failed constraint logs "string has invalid namespace" and the item is refused. The reader
/// of the list may have left entries out, and has then recorded where the others are.
fn drop_minecraft_names<T>(list: &mut Vec<T>, name: fn(&T) -> &str, d: &mut Diagnostics) {
    let mut i = 0;
    let mut index = 0;
    list.retain(|entry| {
        let keep = !name(entry).starts_with("minecraft:");
        if !keep {
            let held = JsonPath::new().index(index);
            // The moves of the reader are still pending on the component's own list, and
            // go no further than it: the finding is told at the file's index here.
            let path = d.source_of(&held);
            d.push(Diagnostic::of(&codes::CUSTOM_COMPONENT_NAMESPACE).at(path));
        }
        index += 1;
        i += usize::from(keep);
        keep
    });
}

/// One component as JSON. A component of the newest format's shape is written by
/// [`ItemComponent::write_component`], the dispatch the derive emits; the match lists every
/// variant, so a new one has to say how it is written. The old shapes, which the newest enum
/// does not hold, are written here, three of them by hand: `render_offsets` as the
/// JSON kept, the 1.21.90 list as its names and the keyed 1.21.90 form as its object.
pub(crate) fn write_component(component: &Component) -> Result<Value, EncodeError> {
    Ok(match component {
        Component::Armor(c) => write_payload(c)?,
        Component::Chargeable(c) => write_payload(c)?,
        Component::OnUse(c) => write_payload(c)?,
        Component::OnUseOn(c) => write_payload(c)?,
        Component::RenderOffsets(c) => c.value.clone(),
        Component::Weapon(c) => write_payload(c)?,
        Component::UseDuration(c) => write_payload(c)?,
        Component::LegacyFood(c) => write_payload(c)?,
        Component::LegacyDurability(c) => write_payload(c)?,
        Component::LegacyRepairable(c) => write_payload(c)?,
        Component::LegacyDigger(c) => write_payload(c)?,
        Component::LegacyIcon(c) => write_payload(c)?,
        Component::LegacyStorageItem(c) => write_payload(c)?,
        Component::CustomComponentList(c) => {
            let mut array = facet_value::VArray::new();
            for name in &c.names {
                array.push(Value::from(name.as_str()));
            }
            Value::from(array)
        }
        Component::CustomComponents(c) => c.to_value(),
        Component::Custom(c) => c.parameters.clone(),
        Component::Unknown(v) => v.clone(),
        // The newest shapes, written by the dispatch the derive emits.
        Component::AllowOffHand(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::BlockPlacer(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::BundleInteraction(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::CanDestroyInCreative(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Compostable(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Cooldown(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Damage(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::DamageAbsorption(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Digger(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::DisplayName(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Durability(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::DurabilitySensor(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Dyeable(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Enchantable(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::EntityPlacer(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::FireResistant(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Food(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Fuel(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Glint(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::HandEquipped(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::HoverTextColor(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Icon(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::InteractButton(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::KineticWeapon(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::LiquidClipped(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::MaxStackSize(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::PiercingWeapon(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Projectile(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Rarity(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Record(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Repairable(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Shooter(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::ShouldDespawn(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::StackedByData(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::StorageItem(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::StorageWeightLimit(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::StorageWeightModifier(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::SwingDuration(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::SwingSounds(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Tags(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Throwable(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::UseAnimation(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::UseModifiers(c) => ItemComponent::from(c.clone()).write_component()?,
        Component::Wearable(c) => ItemComponent::from(c.clone()).write_component()?,
    })
}

/// A component as the struct of the packed version `version` writes it: without a member
/// that format does not have (`use_modifiers.start_using` below 1.26.30, which the typed
/// component always holds), unless the file had it as an unknown member.
///
/// `at` is the component's path in the document, `unknown` the document's unknown members.
fn as_read_component(
    version: u32,
    component: &Component,
    at: &JsonPath,
    unknown: &Table,
) -> Result<Value, EncodeError> {
    let mut written = write_component(component)?;
    if let Component::UseModifiers(_) = component
        && unpack(version) < V1_26_30
        && !unknown.contains(at, "start_using")
        && let Some(object) = written.as_object_mut()
    {
        object.remove("start_using");
    }
    Ok(written)
}

/// Where the side-car goes among the members of the written item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SideCarPlace {
    /// Before `events`: a format that binds `__deprecated_components` reads it back as the
    /// side-car, and writes it here again.
    BeforeEvents,
    /// After `events`: 1.26.30 binds no `__deprecated_components` and reads it back as the
    /// first unknown member, which is written behind the members it knows.
    AfterEvents,
}

/// The item object of a component-format struct as JSON.
pub(crate) fn write_parts(
    description: Result<Value, EncodeError>,
    components: impl Iterator<Item = (String, Result<Value, EncodeError>)>,
    deprecated: (&ItemDeprecatedData, SideCarPlace),
    events: Option<&Value>,
    unknown: &Table,
) -> Result<Value, EncodeError> {
    let (deprecated, place) = deprecated;
    let mut object = VObject::new();
    object.insert("description", description?);
    let mut map = VObject::new();
    for (key, value) in components {
        map.insert(key, value?);
    }
    object.insert("components", Value::from(map));
    let side_car = if deprecated.is_empty() {
        None
    } else {
        Some(deprecated.to_value()?)
    };
    // An unknown member of the side-car's name among the kept ones cannot be written beside
    // it: one object has one member of a name. (Only a caller can build it: a file's member
    // is read as the side-car or as the unknown member, never both.)
    let item = JsonPath::new().with(&["minecraft:item"]);
    if side_car.is_some() && unknown.contains(&item, "__deprecated_components") {
        let at = item.with(&["__deprecated_components"]);
        return Err(EncodeError::NotRepresentable {
            at,
            what: bedrock_addon_core::json::Unrepresentable::Unsupported,
        });
    }
    let mut side_car = side_car.map(|value| ("__deprecated_components", value));
    if place == SideCarPlace::BeforeEvents
        && let Some((key, value)) = side_car.take()
    {
        object.insert(key, value);
    }
    if let Some(events) = events {
        object.insert("events", events.clone());
    }
    if let Some((key, value)) = side_car {
        object.insert(key, value);
    }
    // The unknown members are the document's, put in by its writer.
    Ok(Value::from(object))
}

/// The description of an older struct as JSON.
fn write_description(description: &Description) -> Result<Value, EncodeError> {
    write_payload(description)
}

impl<const V: u32> Versioned for ItemDocument<V> {
    const VERSION: FormatVersion = unpack(V);
}

impl<const V: u32> VersionStruct for ItemDocument<V> {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let parts = parse_parts(payload, Self::VERSION, cx, d)?;
        Some(Self {
            description: parts.description,
            components: parts.components,
            deprecated: parts.deprecated,
            events: parts.events,
            legacy: parts.legacy,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        self.write_with_unknown(&Table::new())
    }

    fn write_with_unknown(&self, unknown: &Table) -> Result<Value, EncodeError> {
        let components = JsonPath::new().with(&["minecraft:item", "components"]);
        write_parts(
            write_description(&self.description),
            self.components
                .iter()
                .map(|(key, component)| {
                    let at = components.child(key);
                    (key.clone(), as_read_component(V, component, &at, unknown))
                })
                // What the legacy reader alone takes of a file of this format: the keys of
                // every version, the `tag:` keys below 1.20.50, and each key of `shared`
                // this struct binds no component for (`minecraft:use_duration` from
                // 1.20.50, `minecraft:rarity` from 1.20.30 to 1.21.29).
                .chain(
                    self.legacy
                        .written_keys()
                        .into_iter()
                        .map(|(key, value)| (key, Ok(value))),
                )
                .chain(
                    self.legacy
                        .written_keys_of(Self::VERSION)
                        .into_iter()
                        .map(|(key, value)| (key, Ok(value))),
                )
                .chain(
                    self.legacy
                        .tags
                        .iter()
                        .filter(|_| Self::VERSION < V1_20_50)
                        .map(|tag| (format!("tag:{tag}"), Ok(Value::from(VObject::new())))),
                )
                .chain(
                    self.legacy
                        .held_in_shared_alone
                        .iter()
                        .filter(|key| !self.components.contains_key(key.as_str()))
                        .filter_map(|key| {
                            Some((key.clone(), Ok(self.legacy.shared.get(key)?.clone())))
                        }),
                ),
            (&self.deprecated, SideCarPlace::BeforeEvents),
            self.events.as_ref(),
            unknown,
        )
    }
}

impl<const V: u32> MolangFields for ItemDocument<V> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.nested("components", &mut self.components);
        self.legacy.expressions.visit_molang(visitor);
    }
}

/// The newest struct's components from an older struct's: each typed component moves into
/// [`ItemComponent`]; an old shape the steps left (none should remain) is kept as written.
pub(crate) fn into_component_map(
    components: Components,
) -> bedrock_addon_core::document::ComponentMap<ItemComponent> {
    let mut map = bedrock_addon_core::document::ComponentMap::new();
    for (key, component) in components {
        match into_item(component) {
            Ok(typed) => {
                map.insert_typed(typed, false);
            }
            Err(Component::Custom(c)) => map.insert_custom(c),
            Err(Component::Unknown(json)) => map.insert_unknown(&key, json),
            Err(other) => {
                if let Ok(json) = write_component(&other) {
                    map.insert_unknown(&key, json);
                }
            }
        }
    }
    map
}

/// A component of the newest format's shape as the newest enum's, or the component itself
/// for an old shape, a custom component and an unknown key.
#[allow(
    clippy::result_large_err,
    reason = "the component goes back to its caller by value, as it came"
)]
fn into_item(component: Component) -> Result<ItemComponent, Component> {
    Ok(match component {
        Component::AllowOffHand(c) => c.into(),
        Component::BlockPlacer(c) => c.into(),
        Component::BundleInteraction(c) => c.into(),
        Component::CanDestroyInCreative(c) => c.into(),
        Component::Compostable(c) => c.into(),
        Component::Cooldown(c) => c.into(),
        Component::Damage(c) => c.into(),
        Component::DamageAbsorption(c) => c.into(),
        Component::Digger(c) => c.into(),
        Component::DisplayName(c) => c.into(),
        Component::Durability(c) => c.into(),
        Component::DurabilitySensor(c) => c.into(),
        Component::Dyeable(c) => c.into(),
        Component::Enchantable(c) => c.into(),
        Component::EntityPlacer(c) => c.into(),
        Component::FireResistant(c) => c.into(),
        Component::Food(c) => c.into(),
        Component::Fuel(c) => c.into(),
        Component::Glint(c) => c.into(),
        Component::HandEquipped(c) => c.into(),
        Component::HoverTextColor(c) => c.into(),
        Component::Icon(c) => c.into(),
        Component::InteractButton(c) => c.into(),
        Component::KineticWeapon(c) => c.into(),
        Component::LiquidClipped(c) => c.into(),
        Component::MaxStackSize(c) => c.into(),
        Component::PiercingWeapon(c) => c.into(),
        Component::Projectile(c) => c.into(),
        Component::Rarity(c) => c.into(),
        Component::Record(c) => c.into(),
        Component::Repairable(c) => c.into(),
        Component::Shooter(c) => c.into(),
        Component::ShouldDespawn(c) => c.into(),
        Component::StackedByData(c) => c.into(),
        Component::StorageItem(c) => c.into(),
        Component::StorageWeightLimit(c) => c.into(),
        Component::StorageWeightModifier(c) => c.into(),
        Component::SwingDuration(c) => c.into(),
        Component::SwingSounds(c) => c.into(),
        Component::Tags(c) => c.into(),
        Component::Throwable(c) => c.into(),
        Component::UseAnimation(c) => c.into(),
        Component::UseModifiers(c) => c.into(),
        Component::Wearable(c) => c.into(),
        other => return Err(other),
    })
}

/// The newest struct's components as JSON, in map order (for writing the newest struct with
/// the same writer as the older ones): a typed component by the dispatch the derive emits, a
/// custom one as its parameters, a dynamic or unknown key as written.
pub(crate) fn written_components(
    map: &bedrock_addon_core::document::ComponentMap<ItemComponent>,
) -> Vec<(String, Result<Value, EncodeError>)> {
    use bedrock_addon_core::document::ComponentRef;
    let mut out = Vec::new();
    for entry in map.iter() {
        match entry {
            ComponentRef::Typed { name, value, .. } => {
                out.push((name.to_owned(), value.write_component()));
            }
            ComponentRef::Custom(c) => out.push((c.name.clone(), Ok(c.parameters.clone()))),
            ComponentRef::Dynamic { name, json } | ComponentRef::Unknown { name, json } => {
                out.push((name.to_owned(), Ok(json.clone())));
            }
            _ => {}
        }
    }
    out
}

/// Members of a component-format struct a step writes as a fresh value: the typed text of a
/// `Value` it built.
pub(crate) fn retype<T: for<'a> Facet<'a>>(value: &Value) -> Option<T> {
    // (A real negative zero keeps its sign where `T` reads the text with it.)
    let text = bedrock_addon_core::json::signed_zero::text_for::<T>(value)?;
    bedrock_addon_core::json::from_str_jsonc::<T>(&text).ok()
}

#[allow(dead_code, reason = "names the 1.17.0 threshold the routing uses")]
const ROUTING_THRESHOLD: FormatVersion = V1_17_0;

#[cfg(test)]
#[allow(
    clippy::disallowed_methods,
    reason = "the test reads on a thread of a stack it chooses"
)]
mod tests {
    use super::retype;
    use crate::component::RepairItem;
    use bedrock_addon_core::json::{MAX_JSON_DEPTH, Value};

    /// A repair entry with an unknown member of `levels - 1` arrays: `levels` deep in all.
    fn entry(levels: usize) -> Value {
        let mut deep = Value::from(0_i64);
        for _ in 1..levels {
            deep = Value::from(facet_value::VArray::from_iter([deep]));
        }
        let mut entry = facet_value::VObject::new();
        entry.insert(
            "items",
            Value::from(facet_value::VArray::from_iter([Value::from(
                "minecraft:stick",
            )])),
        );
        entry.insert("zz", deep);
        entry.into()
    }

    /// An upgrade step types a `Value` by writing it and reading the text: the value is
    /// written to the limit and has no text past it, on a megabyte of stack (the writer is
    /// facet-json's, a call for each level: it took the megabyte at some 80).
    #[test]
    fn a_value_nested_to_the_limit_is_retyped_on_a_megabyte_of_stack() {
        let retyped = std::thread::Builder::new()
            .stack_size(1024 * 1024)
            .spawn(|| {
                [MAX_JSON_DEPTH - 1, MAX_JSON_DEPTH, MAX_JSON_DEPTH + 1, 1000].map(|levels| {
                    retype::<RepairItem>(&entry(levels)).map(|entry| entry.items.len())
                })
            })
            .unwrap()
            .join()
            .unwrap();
        assert_eq!(retyped, [Some(1), Some(1), None, None]);
    }
}
