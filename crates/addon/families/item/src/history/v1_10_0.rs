//! `v1_10_0`: the legacy item format of 1.10, and every document the game's
//! component-based check rejects (below 1.16.100, 1.16.101–1.16.x, and later documents
//! whose `components` name no registered component).

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::json::kit::{Members, Node, Profile, Read};
use bedrock_addon_core::json::parse::{Options, parse_slice};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{FromJson, Schema, read_document};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{
    MemberFailure, NullHandling, ParseCx, Upgrade, UpgradeCx, VersionStruct, Versioned,
    read_payload_with, write_payload,
};
use facet::Facet;
use facet_value::{VArray, VObject};
use indexmap::IndexMap;

use super::document::Description;
use crate::codes;
use crate::legacy::{
    ItemFormat, LegacyEffect, LegacyFood, LegacyFormatItem, LegacyItemData, number, seed_warning,
};
use crate::read::{Raw, is_component_based};
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Diagnostics};

/// An item in the legacy item format (no `SemVersion`
/// reaches its readers, so one shape serves 1.10 to 1.16.0).
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// JSON `description`: `identifier` and the client's `category`.
    pub description: Description,
    /// JSON `components`: the fixed legacy key set.
    pub components: LegacyFormatItem,
    /// JSON `events`, as written.
    pub events: Option<Value>,
}

impl Versioned for Item {
    const VERSION: FormatVersion = known::V1_10_0;
}

/// What the legacy item is parsed from.
#[derive(Facet, Schema)]
struct Shell {
    description: Description,
    components: Option<Raw>,
    events: Option<Value>,
}

impl VersionStruct for Item {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let mut found = Diagnostics::default();
        let shell = read_payload_with::<Shell>(payload, cx, &mut found, crate::typed_read);
        // unknown keys are ignored silently at every version (and kept)
        found.retain(|f| {
            !matches!(
                f.code.id(),
                "core/unrecognized_field"
                    | "core/unrecognized_field_worded"
                    | "core/child_not_valid"
                    | "core/member_not_in_schema"
            )
        });
        d.extend_with_unknown(found);
        let mut shell = shell?;
        if let Err(reason) = is_component_based(payload, cx.written) {
            d.push(Diagnostic::of(core_codes::LEGACY_ITEM_FORMAT.with([reason])).at(cx.at.clone()));
        }
        if let Some(menu) = shell.description.menu_category.take()
            && let Ok(value) = write_payload(&menu)
        {
            d.unknown_mut()
                .put(&cx.at.child("description"), "menu_category", value);
        }
        let at = cx.at.child("components");
        let components = match &shell.components {
            None => LegacyFormatItem::default(),
            Some(raw) => {
                if !raw.value.is_object() {
                    d.push(Diagnostic::of(&codes::COMPONENTS_NOT_OBJECT).at(at));
                    return None;
                }
                let parsed = parse_slice(raw.slice(), Options::default());
                let node = match &parsed {
                    Ok(parsed) => Node::parsed(parsed, at.clone()),
                    Err(_) => Node::new(&raw.value, at.clone()),
                };
                read_format_item(&node, &shell.description.identifier, cx.load, d)
            }
        };
        if components.seed.is_some() && cx.original_version > known::V1_16_100 {
            seed_warning(&at, d);
        }
        Some(Item {
            description: shell.description,
            components,
            events: shell.events,
        })
    }

    fn write(&self) -> Result<Value, EncodeError> {
        let mut object = VObject::new();
        object.insert("description", write_payload(&self.description)?);
        object.insert("components", format_item_value(&self.components));
        if let Some(events) = &self.events {
            object.insert("events", events.clone());
        }
        // The unknown members are the document's, written from its table.
        Ok(Value::from(object))
    }
}

// The 1.10.0 item holds no Molang a document's walk visits.
bedrock_addon_core::impl_no_molang!(Item);

/// The legacy readers take any JSON value for a member, as jsoncpp's `asInt`, `asFloat` and
/// `asBool` do, and say what they make of it: an information at the member (the game is silent,
/// so each is the crate's own).
fn told(read: &mut Read<'_>, code: Code, node: &Node<'_>) {
    let mut finding = Diagnostic::info(code).at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// A value of another JSON type than the member's: an `InvalidType` information at the member.
fn wrong_type(read: &mut Read<'_>, node: &Node<'_>) {
    told(read, Code::from(&core_codes::INVALID_TYPE), node);
}

/// A JSON number as the double it spells.
fn double(value: &Value) -> Option<f64> {
    value.as_number().map(facet_value::VNumber::to_f64_lossy)
}

/// `asInt`: a number truncated toward zero, a bool as 0 or 1. A fraction is truncated and a
/// number beyond an `i32` is not taken, each with an `InvalidValue` information; a bool and any
/// other value come with an `InvalidType` information.
#[allow(clippy::cast_possible_truncation, reason = "vanilla's asInt truncates")]
pub(crate) fn as_int(node: &Node<'_>, read: &mut Read<'_>) -> Option<i32> {
    let Some(number) = double(node.value()) else {
        wrong_type(read, node);
        return node.value().as_bool().map(i32::from);
    };
    let whole = number.trunc();
    if whole < f64::from(i32::MIN) || whole > f64::from(i32::MAX) {
        told(read, Code::from(&core_codes::INVALID_VALUE), node);
        return None;
    }
    if number.fract() != 0.0 {
        told(read, Code::from(&core_codes::INVALID_VALUE), node);
    }
    Some(whole as i32)
}

/// `asFloat`. A number beyond an `f32` is the largest finite float of its sign, with the
/// information of such a number; another value comes with an `InvalidType` information.
#[allow(clippy::cast_possible_truncation, reason = "an f32 member")]
pub(crate) fn as_float(node: &Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    let Some(number) = double(node.value()) else {
        wrong_type(read, node);
        return None;
    };
    let nearest = number as f32;
    if nearest.is_infinite() {
        let mut finding = bedrock_addon_core::molang::beyond_f32_warning(node.path().clone(), None);
        finding.location.span = node.span();
        read.push(finding);
        return Some(f32::MAX.copysign(nearest));
    }
    Some(nearest)
}

/// `asBool`: a bool, or a non-zero number (with an `InvalidType` information, as any other
/// value).
pub(crate) fn as_bool(node: &Node<'_>, read: &mut Read<'_>) -> Option<bool> {
    if let Some(flag) = node.value().as_bool() {
        return Some(flag);
    }
    wrong_type(read, node);
    number(node.value()).map(|n| n != 0.0)
}

/// A string member; another value is not taken, with an `InvalidType` information.
pub(crate) fn as_string(node: &Node<'_>, read: &mut Read<'_>) -> Option<String> {
    let text = node.value().as_string().map(|s| s.as_str().to_owned());
    if text.is_none() {
        wrong_type(read, node);
    }
    text
}

/// An object member read by its type; another value is told, and is the type's default (the
/// game asks it for members it does not have).
pub(crate) fn object<T: FromJson + Default>(node: &Node<'_>, read: &mut Read<'_>) -> Option<T> {
    if node.value().is_object() {
        return T::from_json(node, read);
    }
    wrong_type(read, node);
    Some(T::default())
}

/// The members of the object `node`; a value of another JSON type is told, and has none.
fn members_of<'v>(node: &Node<'v>, read: &mut Read<'_>) -> Option<Members<'v>> {
    if node.value().is_object() {
        return read.object(node);
    }
    wrong_type(read, node);
    None
}

/// The elements of the array `node`; a value of another JSON type is told, and has none.
fn elements_of<'v>(node: &Node<'v>, read: &mut Read<'_>) -> Vec<Node<'v>> {
    if node.value().is_array() {
        return read
            .array(node)
            .map_or_else(Vec::new, |e| e.iter().collect());
    }
    wrong_type(read, node);
    Vec::new()
}

/// The client keys of the legacy format, kept as written.
const CLIENT_KEYS: [&str; 6] = [
    "minecraft:icon",
    "minecraft:icon_variant",
    "minecraft:icon_atlas",
    "minecraft:use_animation",
    "minecraft:hover_text_color",
    "minecraft:rarity",
];

/// The rule after the read of the legacy key set: of the keys no member reads, the client keys
/// are held apart from the others, each in the file's order.
pub(crate) fn client_keys(item: &mut LegacyFormatItem, _: &Node<'_>, _: &mut Read<'_>) {
    for (key, value) in std::mem::take(&mut item.extra).iter() {
        if CLIENT_KEYS.contains(&key) {
            item.client.insert(key.to_owned(), value.clone());
        } else {
            item.extra.insert(key.to_owned(), value.clone());
        }
    }
}

/// Reads the legacy key set from the `components` object at `node`, under `load` (its reading,
/// and its provider, which answers which mob effects the game has); `identifier` is the item's.
/// Every finding is an information but an unknown effect's, and every member reads: a member
/// the game's accessor gives no value is its default.
pub(crate) fn read_format_item(
    node: &Node<'_>,
    identifier: &str,
    load: &LoadContext,
    d: &mut Diagnostics,
) -> LegacyFormatItem {
    let reading = load
        .reading
        .with_on_member_failure(MemberFailure::DefaultAndTell);
    let mut read = Read::new(reading, d)
        .profile(Profile::EVERY_FAILURE)
        .null(NullHandling::AsValue)
        .load(load);
    // The effect lines name the item.
    read.scoped(ITEM, identifier.to_owned(), |read| {
        read_document(node, read)
    })
    .unwrap_or_default()
}

/// The name the item's identifier is in scope under, for the effect lines.
const ITEM: &str = "identifier";

/// Whether `name` is one of the game's mob effects by the read's provider (bare, without ASCII
/// case); a read without a load context checks no name.
fn known_effect(read: &Read<'_>, name: &str) -> bool {
    read.load_context()
        .is_none_or(|load| load.built_ins().effect(name))
}

/// The game logs an unknown effect name at the content log's error level and the loop goes on: a
/// kept error, naming the item ([`read_format_item`] puts its identifier in scope).
fn told_unknown_effect(read: &mut Read<'_>, node: &Node<'_>, what: &str, name: &str) {
    let item = read.scope::<String>(ITEM).map_or("", String::as_str);
    let code = codes::LEGACY_EFFECT_NAME.with([item, what, name]);
    let mut finding = Diagnostic::error(code).at(node.path().clone()).into_kept();
    finding.location.span = node.span();
    read.push(finding);
}

/// `effects` of a legacy food as the game's legacy reader reads it: an element that is no object
/// is told and left out; of the others it asks `name`, checks it, then `chance`, `duration`
/// and `amplifier`, and nothing else.
#[allow(clippy::unnecessary_wraps)] // a `schema::read_with` fn: every list reads
pub(crate) fn effects(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<LegacyEffect>> {
    let mut effects = Vec::new();
    for effect in elements_of(node, read) {
        let Some(entry) = members_of(&effect, read) else {
            continue;
        };
        // A member of the effect, read by `read` when it is written.
        let name = entry
            .get("name")
            .and_then(|name| as_string(&name, read))
            .unwrap_or_default();
        if !known_effect(read, &name) {
            told_unknown_effect(read, &effect, "effect", &name);
        }
        let chance = entry.get("chance").and_then(|n| as_float(&n, read));
        let duration = entry.get("duration").and_then(|n| as_int(&n, read));
        let amplifier = entry.get("amplifier").and_then(|n| as_int(&n, read));
        effects.push(LegacyEffect {
            name,
            chance: chance.unwrap_or(1.0),
            duration: duration.unwrap_or_default(),
            amplifier: amplifier.unwrap_or_default(),
        });
    }
    Some(effects)
}

/// `remove_effects` of a legacy food: each element read as a name and checked.
#[allow(clippy::unnecessary_wraps)] // a `schema::read_with` fn: every list reads
pub(crate) fn remove_effects(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<String>> {
    let mut names = Vec::new();
    for entry in elements_of(node, read) {
        let name = as_string(&entry, read).unwrap_or_default();
        if !known_effect(read, &name) {
            told_unknown_effect(read, &entry, "remove_effect", &name);
        }
        names.push(name);
    }
    Some(names)
}

/// A real number as JSON.
fn real(n: f32) -> Value {
    Value::from(facet_value::VNumber::from_f64(f64::from(n)))
}

/// An integer as JSON.
fn int(n: i32) -> Value {
    Value::from(facet_value::VNumber::from_i64(i64::from(n)))
}

/// The legacy key set as JSON: the members that differ from their defaults, then the client
/// keys and the unknown ones.
pub(crate) fn format_item_value(item: &LegacyFormatItem) -> Value {
    let mut object = VObject::new();
    if item.max_damage != 0 {
        object.insert("minecraft:max_damage", int(item.max_damage));
    }
    if item.hand_equipped {
        object.insert("minecraft:hand_equipped", Value::from(true));
    }
    if item.stacked_by_data {
        object.insert("minecraft:stacked_by_data", Value::from(true));
    }
    if item.foil {
        object.insert("minecraft:foil", Value::from(true));
    }
    if let Some(block) = &item.block {
        object.insert("minecraft:block", Value::from(block.as_str()));
    }
    if item.use_duration != 0 {
        object.insert("minecraft:use_duration", int(item.use_duration));
    }
    if item.max_stack_size != 64 {
        object.insert("minecraft:max_stack_size", int(item.max_stack_size));
    }
    if let Some(food) = &item.food {
        object.insert("minecraft:food", food_value(food));
    }
    if let Some(seed) = &item.seed {
        let mut out = VObject::new();
        if let Some(crop) = &seed.crop_result {
            out.insert("crop_result", Value::from(crop.as_str()));
        }
        if let Some(at) = &seed.plant_at {
            out.insert("plant_at", at.clone());
        }
        if seed.plant_at_any_solid_surface {
            out.insert("plant_at_any_solid_surface", Value::from(true));
        }
        if let Some(face) = &seed.plant_at_face {
            out.insert("plant_at_face", Value::from(face.as_str()));
        }
        for (key, value) in seed.extra.iter() {
            out.insert(key, value.clone());
        }
        object.insert("minecraft:seed", Value::from(out));
    }
    if let Some(camera) = &item.camera {
        let mut out = VObject::new();
        for (key, n) in [
            ("black_bars_duration", camera.black_bars_duration),
            ("black_bars_screen_ratio", camera.black_bars_screen_ratio),
            ("shutter_duration", camera.shutter_duration),
            ("shutter_screen_ratio", camera.shutter_screen_ratio),
            ("picture_duration", camera.picture_duration),
            ("slide_away_duration", camera.slide_away_duration),
        ] {
            out.insert(key, real(n));
        }
        for (key, value) in camera.extra.iter() {
            out.insert(key, value.clone());
        }
        object.insert("minecraft:camera", Value::from(out));
    }
    for (key, value) in &item.client {
        object.insert(key, value.clone());
    }
    for (key, value) in item.extra.iter() {
        object.insert(key, value.clone());
    }
    Value::from(object)
}

/// A legacy food as JSON.
fn food_value(food: &LegacyFood) -> Value {
    let mut out = VObject::new();
    out.insert("nutrition", int(food.nutrition));
    if let Some(saturation) = &food.saturation_modifier {
        out.insert("saturation_modifier", saturation.clone());
    }
    if let Some(to) = &food.using_converts_to {
        out.insert("using_converts_to", Value::from(to.as_str()));
    }
    if let Some(action) = &food.on_use_action {
        out.insert("on_use_action", Value::from(action.as_str()));
    }
    if let Some(range) = &food.on_use_range {
        out.insert("on_use_range", range.clone());
    }
    if let Some(kind) = &food.cooldown_type {
        out.insert("cooldown_type", Value::from(kind.as_str()));
    }
    if food.cooldown_time != 0 {
        out.insert("cooldown_time", int(food.cooldown_time));
    }
    if food.can_always_eat {
        out.insert("can_always_eat", Value::from(true));
    }
    if !food.effects.is_empty() {
        let mut effects = VArray::new();
        for effect in &food.effects {
            let mut e = VObject::new();
            e.insert("name", Value::from(effect.name.as_str()));
            e.insert("chance", real(effect.chance));
            e.insert("duration", int(effect.duration));
            e.insert("amplifier", int(effect.amplifier));
            effects.push(Value::from(e));
        }
        out.insert("effects", Value::from(effects));
    }
    if !food.remove_effects.is_empty() {
        let mut names = VArray::new();
        for name in &food.remove_effects {
            names.push(Value::from(name.as_str()));
        }
        out.insert("remove_effects", Value::from(names));
    }
    for (key, value) in food.extra.iter() {
        out.insert(key, value.clone());
    }
    Value::from(out)
}

/// the legacy reading is carried unchanged, not converted: `legacy.format` is
/// `Legacy`, `legacy.format_item` holds the key set, the component map is empty, and the
/// description's `category` moves to `legacy.category` (no `menu_category` is created).
impl Upgrade for Item {
    type Next = super::v1_16_100::Item;
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        let mut description = self.description;
        let category = description.category.take();
        let legacy = LegacyItemData {
            format: ItemFormat::Legacy,
            format_item: Some(self.components),
            category,
            ..LegacyItemData::default()
        };
        super::v1_16_100::Item {
            description,
            components: IndexMap::new(),
            deprecated: crate::ItemDeprecatedData::default(),
            events: self.events,
            legacy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::{JsonPath, Severity};
    use bedrock_addon_core::load::BuiltIns;
    use bedrock_addon_core::versioning::Reading;

    /// A provider that knows the effects of its list, compared as `known_effect` compared
    /// before the provider: bare and without ASCII case.
    struct Effects(&'static [&'static str]);

    impl BuiltIns for Effects {
        fn contains(&self, _kind: &'static str, _id: &str) -> bool {
            false
        }
        fn effect(&self, name: &str) -> bool {
            self.0.iter().any(|e| {
                e.trim_start_matches("minecraft:")
                    .eq_ignore_ascii_case(name)
            })
        }
    }

    /// The legacy slice's load context with `effects` as its provider.
    fn loaded_with(effects: Effects) -> LoadContext {
        let mut load = LoadContext::detached();
        load.reading = Reading::NULL_ABSENT;
        load.set_built_ins(Some(std::sync::Arc::new(effects)));
        load
    }

    /// an unknown `effects` or `remove_effects` name is a warning with vanilla's
    /// text, the effect kept.
    #[test]
    fn rule_19d_unknown_legacy_effect_names_warn() {
        let json: Value = bedrock_addon_core::json::from_str_jsonc(
            r#"{"minecraft:food": {"nutrition": 1, "effects": [{"name": "speed"}, {"name": "zoom"}], "remove_effects": ["poison", "fizz"]}}"#,
        )
        .unwrap();
        let mut d = Diagnostics::default();
        let load = loaded_with(Effects(&["minecraft:speed", "minecraft:poison"]));
        let item = read_format_item(&Node::new(&json, JsonPath::new()), "demo:x", &load, &mut d);
        let texts: Vec<String> = d.iter().map(|f| f.code.to_string()).collect();
        assert_eq!(
            texts,
            [
                "Item[demo:x] Failed to load effect with name[zoom] from Json. bad name?",
                "Item[demo:x] Failed to load remove_effect with name[fizz] from Json. bad name?",
            ]
        );
        // The game logs both at the error level and adds the effect on: kept errors.
        assert!(
            d.iter()
                .all(|f| (f.severity, f.kept) == (Severity::Error, true))
        );
        let food = item.food.unwrap();
        assert_eq!(food.effects.len(), 2);
        assert_eq!(food.remove_effects.len(), 2);
        // a provider that knows every name tells none
        let mut d = Diagnostics::default();
        let load = loaded_with(Effects(&["speed", "zoom", "poison", "fizz"]));
        read_format_item(&Node::new(&json, JsonPath::new()), "demo:x", &load, &mut d);
        assert!(d.is_empty());
    }
}
