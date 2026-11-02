//! The upgrade steps between the component formats,
//! one `Upgrade` impl per step in chain order. Components a step does not name are copied;
//! `legacy` is carried unchanged except by the 1.20.50 step.

use bedrock_addon_core::document::{ComponentMap, CustomComponent};
use bedrock_addon_core::molang::{
    Molang, MolangContext, MolangFieldMut, MolangFields, visit_fields,
};
use bedrock_addon_core::shared::{CreativeCategory, EventTrigger, MenuCategory};
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx};
use facet_value::VObject;

use super::component::{Component, Components, CustomComponentEntry, CustomComponents};
use super::document::{ItemDocument, into_component_map, is_name_list, retype};
use super::{
    v1_16_100, v1_19_83, v1_20_0, v1_20_20, v1_20_30, v1_20_40, v1_20_50, v1_20_60, v1_20_80,
    v1_21_10, v1_21_30, v1_21_40, v1_21_50, v1_21_60, v1_21_80, v1_21_90, v1_21_110, v1_26_0,
    v1_26_30,
};
use crate::component::{
    DestroySpeed, Digger, Durability, DurabilitySensor, Food, Icon, ItemComponent, RepairItem,
    Repairable, StartUsing, StorageItem, StorageWeightLimit, StorageWeightModifier, SwingSounds,
    UseModifiers,
};
use crate::legacy::ItemFormat;
use crate::{Beta, Item, ItemDescription};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, Location, Phase, Severity, VanillaText,
};
use bedrock_addon_core::id::SoundEventName;
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::{FormatVersion, FormatVersionInput};

/// Where a 1.20.40 side-car trigger goes.
type SideCarSlot = fn(&mut v1_20_40::Item) -> &mut Option<EventTrigger>;

/// The same document in the next format: every member moved unchanged.
fn carry<const A: u32, const B: u32>(doc: ItemDocument<A>) -> ItemDocument<B> {
    ItemDocument {
        description: doc.description,
        components: doc.components,
        deprecated: doc.deprecated,
        events: doc.events,
        legacy: doc.legacy,
        extra: doc.extra,
    }
}

/// The information that the legacy key `key` of `components`, which the file's format reads, is
/// not carried into the later formats; not twice where the key is already told of. The 1.18
/// and 1.20.20 fixers log nothing, so the finding is the crate's own information.
fn not_carried(cx: &mut UpgradeCx<'_>, key: &str) {
    let at = member(&["components", key]);
    let said = cx
        .diagnostics
        .iter()
        .any(|told| told.location.json_path == at && told.severity >= Severity::Info);
    if !said {
        report(
            cx,
            DiagnosticCode::DroppedField { text: None },
            Severity::Info,
            &["components", key],
        );
    }
}

/// The game's content-log error after which the step goes on: a kept error, at
/// `minecraft:item/<path>`.
fn report_kept(cx: &mut UpgradeCx<'_>, code: DiagnosticCode, path: &[&str]) {
    let mut at = JsonPath::new();
    at.push("minecraft:item");
    for part in path {
        at.push(part);
    }
    cx.diagnostics.push_kept_error(Diagnostic::new(
        code,
        Severity::Error,
        Phase::Upgrade,
        Location::new(None, None, at, None),
    ));
}

/// A finding of a step at `minecraft:item/<path>`.
fn report(cx: &mut UpgradeCx<'_>, code: DiagnosticCode, severity: Severity, path: &[&str]) {
    let mut at = JsonPath::new();
    at.push("minecraft:item");
    for part in path {
        at.push(part);
    }
    cx.diagnostics.push(Diagnostic::new(
        code,
        severity,
        Phase::Upgrade,
        Location::new(None, None, at, None),
    ));
}

/// The entries of a legacy `repair_items` list (not the pre-1.18 name list) in the newest
/// type, one at a time. Vanilla's fixer does not look at the entries (it does not filter
/// them), and the 1.19.83 slice then reads them: `items` is required (a missing required
/// member) and a value of another shape is a type error, both errors under the legacy
/// chain's error mask after which the item is refused (" Error Parsing Item"). So an entry
/// that is no entry of the newest type (a number, no `items`, `items` of another shape) and
/// a list that is no array are `InvalidValue` errors. A `repair_amount` beyond a float is
/// the largest finite float of its sign, told at it as an information of the crate's own.
fn repair_items(items: &Value, cx: &mut UpgradeCx<'_>) -> Vec<RepairItem> {
    let mut at = JsonPath::new();
    for key in [
        "minecraft:item",
        "components",
        "minecraft:repairable",
        "repair_items",
    ] {
        at.push(key);
    }
    let tell = |cx: &mut UpgradeCx<'_>, at: JsonPath, severity: Severity| {
        cx.diagnostics.push(Diagnostic::new(
            DiagnosticCode::InvalidValue { text: None },
            severity,
            Phase::Upgrade,
            Location::new(None, None, at, None),
        ));
    };
    let Some(entries) = items.as_array() else {
        tell(cx, at, Severity::Error);
        return Vec::new();
    };
    let mut kept = Vec::with_capacity(entries.len());
    // The file's index of each entry kept.
    let mut places = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let mut entry_at = at.clone();
        entry_at.push_index(index);
        let mut entry = entry.clone();
        // The amount is a Molang member, which refuses a constant no float holds.
        if let Some(amount) = entry
            .as_object_mut()
            .and_then(|entry| entry.get_mut("repair_amount"))
            && let Some(number) = amount.as_number()
        {
            #[allow(clippy::cast_possible_truncation, reason = "the nearest f32 is asked")]
            let nearest = number.to_f64_lossy() as f32;
            if nearest.is_infinite() {
                *amount = Value::from(facet_value::VNumber::from_f64(f64::from(
                    f32::MAX.copysign(nearest),
                )));
                let mut amount_at = entry_at.clone();
                amount_at.push("repair_amount");
                // A number no double holds was clamped where the file was read, with its
                // warning at this path: the number is told once.
                let said = cx.diagnostics.iter().any(|finding| {
                    finding.location.json_path == amount_at
                        && matches!(finding.code, DiagnosticCode::InvalidValue { .. })
                });
                if !said {
                    tell(cx, amount_at, Severity::Info);
                }
            }
        }
        match retype::<RepairItem>(&entry) {
            Some(entry) => {
                kept.push(entry);
                places.push(index);
            }
            None => tell(cx, entry_at, Severity::Error),
        }
    }
    // What a later step, a check or the link pass tells of an entry is at the index the
    // file has it at.
    cx.diagnostics.left_out(&at, &places);
    kept
}

/// Gives the expressions of a value a step typed their versions, each from its field's
/// version source: what the reading of the file gives those of a struct it types. The
/// compile pass is given no expression without one.
fn settle_versions<T: MolangFields>(value: &mut T, cx: &UpgradeCx<'_>) {
    let written = FormatVersionInput {
        version: Some(cx.original_version),
        beta_literal: false,
        prerelease: false,
    };
    let molang = MolangContext::new(
        cx.load.min_engine_version.as_ref(),
        cx.load.molang_experiments,
        &written,
        Some(cx.load.molang_version),
    );
    visit_fields(value, &JsonPath::new(), |field| {
        let version = field.spec.version.resolve(&molang);
        match field.value {
            MolangFieldMut::Value(Molang::Expr(source)) | MolangFieldMut::Source(source)
                if source.raw_version().is_none() =>
            {
                source.set_context_version(version);
            }
            _ => {}
        }
    });
}

/// The `DeprecatedMember` warning of a move into the side-car, naming the source path.
fn moved(cx: &mut UpgradeCx<'_>, path: &[&str]) {
    report(
        cx,
        DiagnosticCode::DeprecatedMember { text: None },
        Severity::Info,
        path,
    );
}

/// The members of a component a step takes apart that the step does not read: each an
/// `UnrecognizedField` warning at the member of the file, since nothing of the newest format
/// holds it. The file's own slice did not bind them either: each is a member the schema does
/// not know, a warning-level line in the content log under the legacy chain while the item
/// loads.
fn unread(cx: &mut UpgradeCx<'_>, component: &str, extra: &bedrock_addon_core::document::Extra) {
    for (key, _) in extra.iter() {
        report(
            cx,
            DiagnosticCode::UnrecognizedField { text: None },
            Severity::Warning,
            &["components", component, key],
        );
    }
}

/// Fills the slot `key` of the side-car with `value`. What the file's own side-car holds
/// there, when it is another value, is written over and told at the slot as an information
/// (it is not known what the game's move makes of a filled slot).
fn fill<T: PartialEq>(cx: &mut UpgradeCx<'_>, slot: &mut Option<T>, key: &str, value: T) {
    if slot.as_ref().is_some_and(|had| *had != value) {
        report(
            cx,
            DiagnosticCode::DroppedField { text: None },
            Severity::Info,
            &["__deprecated_components", key],
        );
    }
    *slot = Some(value);
}

/// Tells of a legacy use duration whose ticks are beyond an int. The ticks are an int, which
/// holds no more; the duration written is the largest it holds, told at the member the file
/// has.
fn saturated_duration(legacy: &crate::legacy::LegacyItemData, cx: &mut UpgradeCx<'_>) {
    if !legacy.use_duration_saturates() {
        return;
    }
    let at = use_duration_at();
    let said = cx.diagnostics.iter().any(|finding| {
        finding.location.json_path.starts_with(&at)
            && matches!(finding.code, DiagnosticCode::InvalidValue { .. })
    });
    if !said {
        report(
            cx,
            DiagnosticCode::InvalidValue { text: None },
            Severity::Info,
            &["components", "minecraft:use_duration"],
        );
    }
}

/// The member a legacy item's use duration is written at.
fn use_duration_at() -> JsonPath {
    JsonPath::of_keys(["minecraft:item", "components", "minecraft:use_duration"])
}

/// Whether the legacy use duration is negative, told then at the member the file has. Such a
/// duration is not carried. The newest format's `use_modifiers.use_duration` has a minimum
/// of 0, so the written item could not hold it: its reader drops the component for it. Not
/// told again where the typed read of the member said so already.
fn negative_duration(legacy: &crate::legacy::LegacyItemData, cx: &mut UpgradeCx<'_>) -> bool {
    let negative = legacy.use_duration_ticks().is_some_and(|ticks| ticks < 0)
        && legacy.format == ItemFormat::DataDriven;
    if negative {
        let at = use_duration_at();
        let said = cx.diagnostics.iter().any(|finding| {
            finding.location.json_path.starts_with(&at)
                && matches!(finding.code, DiagnosticCode::InvalidValue { .. })
        });
        if !said {
            report(
                cx,
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
                &["components", "minecraft:use_duration"],
            );
        }
    }
    negative
}

/// Puts the copy of a triggered digger into the side-car's `legacy_on_dig_trigger`; what the
/// file's own side-car holds there is told.
fn copy_triggered_digger(
    deprecated: &mut crate::ItemDeprecatedData,
    digger: &Digger,
    cx: &mut UpgradeCx<'_>,
) {
    if deprecated.extra.remove("legacy_on_dig_trigger").is_some() {
        // A member of this name that read as no digger: the copy takes its place.
        deprecated.legacy_on_dig = None;
        report(
            cx,
            DiagnosticCode::DroppedField { text: None },
            Severity::Info,
            &["__deprecated_components", "legacy_on_dig_trigger"],
        );
    }
    fill(
        cx,
        &mut deprecated.legacy_on_dig,
        "legacy_on_dig_trigger",
        digger.clone(),
    );
}

/// A member of the file that a step replaces by what it makes of another, as the game's
/// move does without a word: `DroppedField`, an information.
fn replaced(cx: &mut UpgradeCx<'_>, path: &[&str]) {
    report(
        cx,
        DiagnosticCode::DroppedField { text: None },
        Severity::Info,
        path,
    );
}

/// `minecraft:item/<path>`.
fn member(path: &[&str]) -> JsonPath {
    let mut at = JsonPath::new();
    at.push("minecraft:item");
    for part in path {
        at.push(part);
    }
    at
}

/// Replaces the component at `key` in place (keeping its position).
fn replace(components: &mut Components, key: &str, f: impl FnOnce(Component) -> Option<Component>) {
    let Some(index) = components.get_index_of(key) else {
        return;
    };
    let Some((name, old)) = components.shift_remove_index(index) else {
        return;
    };
    if let Some(new) = f(old) {
        components.shift_insert(index, name, new);
    }
}

/// The saturation of a 1.18 food fixer name (lower-cased; `high` is 0.6, as vanilla).
fn saturation_of(name: &str) -> Option<f64> {
    Some(match name {
        "poor" => 0.1,
        "low" => 0.3,
        "normal" | "high" => 0.6,
        "good" => 0.8,
        "max" => 1.0,
        "supernatural" => 1.2,
        _ => return None,
    })
}

/// The 1.18 upgrade of vanilla's food, durability and repairable fixers; they apply below
/// 1.18.0, where the parse kept the old shapes.
impl Upgrade for v1_16_100::Item {
    type Next = v1_19_83::Item;
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut doc: v1_19_83::Item = carry(self);
        replace(&mut doc.components, "minecraft:food", |c| match c {
            Component::LegacyFood(mut food) => {
                if let Some(name) = food.saturation_modifier.as_ref().and_then(Value::as_string) {
                    let name = name.as_str().to_owned();
                    let value = saturation_of(&name.to_ascii_lowercase()).unwrap_or_else(|| {
                        // The game's food fixer logs an unknown saturation name at the
                        // content log's error level and goes on with 0.6: a kept error.
                        report_kept(
                            cx,
                            DiagnosticCode::InvalidValue {
                                text: Some(VanillaText::ItemSaturationModifier { name }),
                            },
                            &["components", "minecraft:food", "saturation_modifier"],
                        );
                        0.6
                    });
                    food.saturation_modifier = Some(Value::from(value));
                }
                Some(Component::LegacyFood(food))
            }
            other => Some(other),
        });
        replace(&mut doc.components, "minecraft:durability", |c| match c {
            Component::LegacyDurability(old) => {
                if old.max_damage.is_some() && old.max_durability.is_some() {
                    report(
                        cx,
                        DiagnosticCode::DroppedField { text: None },
                        Severity::Info,
                        &["components", "minecraft:durability", "max_damage"],
                    );
                }
                let Some(max) = old.max_durability.or(old.max_damage) else {
                    // A durability with neither maximum: the 1.18 fixer moves nothing and
                    // the slice requires `max_durability`, a missing required member after
                    // which the item is refused (" Error Parsing Item"): an error.
                    report(
                        cx,
                        DiagnosticCode::MissingRequiredField { text: None },
                        Severity::Error,
                        &["components", "minecraft:durability"],
                    );
                    return None;
                };
                Some(Component::Durability(Durability {
                    max_durability: max,
                    damage_chance: old.damage_chance,
                    extra: old.extra,
                }))
            }
            other => Some(other),
        });
        replace(&mut doc.components, "minecraft:repairable", |c| match c {
            Component::LegacyRepairable(old) => {
                let mut repairable = Repairable {
                    extra: old.extra,
                    ..Repairable::default()
                };
                if is_name_list(old.repair_items.as_ref()) {
                    report(
                        cx,
                        DiagnosticCode::DroppedField { text: None },
                        Severity::Info,
                        &["components", "minecraft:repairable", "repair_items"],
                    );
                } else if let Some(items) = &old.repair_items {
                    repairable.repair_items = repair_items(items, cx);
                    settle_versions(&mut repairable, cx);
                }
                Some(Component::Repairable(repairable))
            }
            other => Some(other),
        });
        doc
    }
}

/// a plain copy; the description becomes typed.
impl Upgrade for v1_19_83::Item {
    type Next = v1_20_0::Item;
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        carry(self)
    }
}

/// `description.category` → `menu_category.category`; `armor.protection` →
/// `wearable.protection` (an error without `wearable`), `armor` removed.
impl Upgrade for v1_20_0::Item {
    type Next = v1_20_20::Item;
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut doc: v1_20_20::Item = carry(self);
        // The two keys the legacy reader takes below 1.20.20 and no later format reads: the
        // item is written in the newest format, without them (the crate's own information:
        // the game reads them in the file's format and has no conversion for them).
        if doc.legacy.format == ItemFormat::DataDriven {
            if doc.legacy.creative_category.is_some() {
                not_carried(cx, "minecraft:creative_category");
            }
            if doc.legacy.is_glint.is_some() {
                not_carried(cx, "minecraft:foil");
            }
        }
        if doc.legacy.format == ItemFormat::DataDriven
            && let Some(category) = doc.description.category.take()
        {
            // A name the set does not hold gives no category (then `none`), and the value
            // dropped is told at the member.
            let category = CreativeCategory::try_from(Value::from(category.as_str())).ok();
            if category.is_none() {
                report(
                    cx,
                    DiagnosticCode::InvalidValue { text: None },
                    Severity::Info,
                    &["description", "category"],
                );
            }
            doc.description.menu_category = Some(MenuCategory {
                category,
                ..MenuCategory::default()
            });
        }
        let (mut replaced_protection, mut moved_protection) = (false, false);
        if let Some(Component::Armor(armor)) = doc.components.shift_remove("minecraft:armor") {
            unread(cx, "minecraft:armor", &armor.extra);
            // The 1.20.20 armour fixer moves the armour's points to the wearable.
            match doc.components.get_mut("minecraft:wearable") {
                Some(Component::Wearable(wearable)) => {
                    if let Some(protection) = armor.protection {
                        // The armour's points are written over the wearable's own: noted
                        // when it held others.
                        replaced_protection =
                            wearable.protection != protection && wearable.protection != 0;
                        wearable.protection = protection;
                        moved_protection = true;
                    }
                }
                _ => report(
                    cx,
                    DiagnosticCode::MissingRequiredField {
                        text: Some(VanillaText::Fixed(crate::texts::ARMOR_WITHOUT_WEARABLE)),
                    },
                    Severity::Error,
                    &["components", "minecraft:armor"],
                ),
            }
            if replaced_protection {
                replaced(cx, &["components", "minecraft:wearable", "protection"]);
            }
            if moved_protection {
                cx.diagnostics.moved(
                    member(&["components", "minecraft:armor", "protection"]),
                    member(&["components", "minecraft:wearable", "protection"]),
                );
            }
            if armor.texture_type.is_some() {
                report(
                    cx,
                    DiagnosticCode::DroppedField { text: None },
                    Severity::Info,
                    &["components", "minecraft:armor", "texture_type"],
                );
            }
        }
        doc
    }
}

/// The number of a food's `saturation_modifier` for the typed member of 1.20.30. The game
/// names the number alone; so that nothing is lost in silence, a value that is no number (a
/// bool, a list, an object, a name the 1.18 fixer did not convert) is dropped with an
/// `InvalidType` information at the member; a number beyond a float is the largest finite
/// float of its sign, with the `InvalidValue` information of such a number, told once.
fn saturation(value: &Value, cx: &mut UpgradeCx<'_>) -> Option<f32> {
    let member = ["components", "minecraft:food", "saturation_modifier"];
    let Some(number) = crate::legacy::number(value) else {
        // The game's own line for an unconverted name is the kept error above; a value of
        // another type has none, so this information is the crate's own.
        report(
            cx,
            DiagnosticCode::InvalidType { text: None },
            Severity::Info,
            &member,
        );
        return None;
    };
    #[allow(clippy::cast_possible_truncation, reason = "the nearest f32 is asked")]
    let nearest = number as f32;
    if nearest.is_infinite() {
        let said = cx.diagnostics.iter().any(|finding| {
            matches!(finding.code, DiagnosticCode::InvalidValue { .. })
                && finding.location.json_path
                    == JsonPath::of_keys([
                        "minecraft:item",
                        "components",
                        "minecraft:food",
                        "saturation_modifier",
                    ])
        });
        if !said {
            // The game's float holds an infinity in silence, so this information is the
            // crate's own.
            report(
                cx,
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
                &member,
            );
        }
        return Some(f32::MAX.copysign(nearest));
    }
    Some(nearest)
}

/// `food.on_consume` → `__deprecated_components.on_consume_trigger`.
impl Upgrade for v1_20_20::Item {
    type Next = v1_20_30::Item;
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut doc: v1_20_30::Item = carry(self);
        // `minecraft:mining_speed` is deprecated from 1.20.30: not carried (as above).
        if doc.legacy.format == ItemFormat::DataDriven && doc.legacy.mining_speed.is_some() {
            not_carried(cx, "minecraft:mining_speed");
        }
        replace(&mut doc.components, "minecraft:food", |c| match c {
            Component::LegacyFood(old) => {
                if let Some(trigger) = old.on_consume {
                    moved(cx, &["components", "minecraft:food", "on_consume"]);
                    doc.deprecated.on_consume = Some(trigger);
                }
                let saturation = old
                    .saturation_modifier
                    .as_ref()
                    .and_then(|value| saturation(value, cx));
                Some(Component::Food(Food {
                    nutrition: old.nutrition,
                    saturation_modifier: saturation,
                    can_always_eat: old.can_always_eat,
                    using_converts_to: old.using_converts_to,
                    extra: old.extra,
                }))
            }
            other => Some(other),
        });
        doc
    }
}

/// the six legacy triggers move into the side-car; `on_use`, `on_use_on`, `weapon`
/// and `render_offsets` are removed.
impl Upgrade for v1_20_30::Item {
    type Next = v1_20_40::Item;
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut doc: v1_20_40::Item = carry(self);
        if let Some(Component::OnUse(c)) = doc.components.shift_remove("minecraft:on_use") {
            unread(cx, "minecraft:on_use", &c.extra);
            if let Some(trigger) = c.on_use {
                moved(cx, &["components", "minecraft:on_use", "on_use"]);
                fill(cx, &mut doc.deprecated.on_use, "on_use", trigger);
            }
        }
        if let Some(Component::OnUseOn(c)) = doc.components.shift_remove("minecraft:on_use_on") {
            unread(cx, "minecraft:on_use_on", &c.extra);
            if let Some(trigger) = c.on_use_on {
                moved(cx, &["components", "minecraft:on_use_on"]);
                fill(
                    cx,
                    &mut doc.deprecated.on_use_on,
                    "on_use_on_trigger",
                    trigger,
                );
            }
        }
        if let Some(Component::Weapon(c)) = doc.components.shift_remove("minecraft:weapon") {
            unread(cx, "minecraft:weapon", &c.extra);
            let slots: [(Option<EventTrigger>, &str, &str, SideCarSlot); 3] = [
                (
                    c.on_hurt_entity,
                    "on_hurt_entity",
                    "on_hurt_actor_trigger",
                    |d| &mut d.deprecated.on_hurt_actor,
                ),
                (
                    c.on_not_hurt_entity,
                    "on_not_hurt_entity",
                    "on_hit_actor_trigger",
                    |d| &mut d.deprecated.on_hit_actor,
                ),
                (
                    c.on_hit_block,
                    "on_hit_block",
                    "on_hit_block_trigger",
                    |d| &mut d.deprecated.on_hit_block,
                ),
            ];
            for (trigger, member, key, field) in slots {
                if let Some(trigger) = trigger {
                    moved(cx, &["components", "minecraft:weapon", member]);
                    fill(cx, field(&mut doc), key, trigger);
                }
            }
        }
        if let Some(Component::RenderOffsets(c)) =
            doc.components.shift_remove("minecraft:render_offsets")
        {
            moved(cx, &["components", "minecraft:render_offsets"]);
            fill(
                cx,
                &mut doc.deprecated.render_offsets,
                "render_offsets",
                c.value,
            );
        }
        doc
    }
}

/// The digger: `on_dig` → `__deprecated_on_dig` at both levels.
fn digger_to_1_20_50(components: &mut Components, cx: &mut UpgradeCx<'_>) {
    // `__deprecated_on_dig` is the member this step moves `on_dig` to; below 1.20.50 no
    // struct reads the name. One the file has is not held beside the step's own (the
    // digger has one member of that name): told where it is.
    let mut own: Vec<Option<usize>> = Vec::new();
    replace(components, "minecraft:digger", |c| match c {
        Component::LegacyDigger(mut old) => {
            if old.extra.remove("__deprecated_on_dig").is_some() {
                own.push(None);
            }
            Some(Component::Digger(Digger {
                // The destroy speeds are carried over entry by entry.
                destroy_speeds: old
                    .destroy_speeds
                    .into_iter()
                    .enumerate()
                    .map(|(index, mut speed)| {
                        if speed.extra.remove("__deprecated_on_dig").is_some() {
                            own.push(Some(index));
                        }
                        DestroySpeed {
                            block: speed.block,
                            speed: speed.speed,
                            deprecated_on_dig: speed.on_dig,
                            extra: speed.extra,
                        }
                    })
                    .collect(),
                use_efficiency: old.use_efficiency,
                deprecated_on_dig: old.on_dig,
                extra: old.extra,
            }))
        }
        other => Some(other),
    });
    for speed in own {
        let mut at = member(&["components", "minecraft:digger"]);
        if let Some(index) = speed {
            at.push("destroy_speeds");
            at.push_index(index);
        }
        at.push("__deprecated_on_dig");
        cx.diagnostics.push(Diagnostic::new(
            DiagnosticCode::DroppedField { text: None },
            Severity::Info,
            Phase::Upgrade,
            Location::new(None, None, at, None),
        ));
    }
}

/// `use_duration` and `chargeable.movement_modifier` → `use_modifiers`,
/// `chargeable.on_complete` → the side-car, `chargeable` removed; the digger's `on_dig` →
/// `__deprecated_on_dig`; the `tag:` keys → `minecraft:tags`.
impl Upgrade for v1_20_40::Item {
    type Next = v1_20_50::Item;
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut doc: v1_20_50::Item = carry(self);
        let duration = match doc.components.get("minecraft:use_duration") {
            Some(Component::UseDuration(c)) => Some((c.value, c.extra.clone())),
            _ => None,
        };
        let chargeable = match doc.components.shift_remove("minecraft:chargeable") {
            Some(Component::Chargeable(c)) => Some(c),
            _ => None,
        };
        if let Some(chargeable) = &chargeable {
            unread(cx, "minecraft:chargeable", &chargeable.extra);
        }
        let mut movement = chargeable.as_ref().and_then(|c| c.movement_modifier);
        // A modifier outside 0 to 1 is not carried. The newest format's
        // `use_modifiers.movement_modifier` takes that range alone and its reader drops the
        // component for anything else, so the written item could not hold it; told at the
        // member the file has.
        if movement.is_some_and(|modifier| !(0.0..=1.0).contains(&modifier)) {
            report(
                cx,
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
                &["components", "minecraft:chargeable", "movement_modifier"],
            );
            movement = None;
        }
        if duration.is_some() || movement.is_some() {
            const MODIFIERS: &str = "minecraft:use_modifiers";
            let duration_moved = duration.is_some();
            let (seconds, extra) = duration.unwrap_or_default();
            // a use_modifiers built only by these moves: use_duration 0.0, emit_vibrations
            // true (the type's defaults; not verified against the game)
            let mut modifiers = UseModifiers::new(seconds);
            modifiers.movement_modifier = movement;
            // What the legacy component held beside its value goes with the value:
            // unknown there, unknown here, and kept.
            modifiers.extra = extra;
            let entry = Component::UseModifiers(modifiers);
            // the target takes the position of `use_duration` when there was one
            match doc.components.get_index_of("minecraft:use_duration") {
                Some(i) => {
                    doc.components.shift_remove_index(i);
                    doc.components.shift_insert(i, MODIFIERS.to_owned(), entry);
                }
                None => {
                    doc.components.insert(MODIFIERS.to_owned(), entry);
                }
            }
            // What is told of the modifiers from here on is told of the members the file
            // has.
            if duration_moved {
                let from = member(&["components", "minecraft:use_duration"]);
                cx.diagnostics
                    .moved(from.clone(), member(&["components", MODIFIERS]));
                cx.diagnostics
                    .moved(from, member(&["components", MODIFIERS, "use_duration"]));
            }
            if movement.is_some() {
                cx.diagnostics.moved(
                    member(&["components", "minecraft:chargeable", "movement_modifier"]),
                    member(&["components", MODIFIERS, "movement_modifier"]),
                );
            }
        }
        if let Some(trigger) = chargeable.and_then(|c| c.on_complete) {
            moved(cx, &["components", "minecraft:chargeable", "on_complete"]);
            fill(
                cx,
                &mut doc.deprecated.on_complete,
                "on_complete_trigger",
                trigger,
            );
        }
        digger_to_1_20_50(&mut doc.components, cx);
        let tags = std::mem::take(&mut doc.legacy.tags);
        if !tags.is_empty() {
            // No slice binds `minecraft:tags` before 1.20.50: one the
            // file has beside its `tag:` keys is no component to the game, and the tags of
            // the keys take its place. Noted where the file has it.
            if doc.components.contains_key("minecraft:tags") {
                replaced(cx, &["components", "minecraft:tags"]);
            }
            doc.components.insert(
                "minecraft:tags".to_owned(),
                Component::Tags(crate::component::Tags {
                    tags,
                    extra: bedrock_addon_core::document::Extra::default(),
                }),
            );
        }
        doc
    }
}

/// `icon.texture` → `icon.textures.default`.
impl Upgrade for v1_20_50::Item {
    type Next = v1_20_60::Item;
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut doc: v1_20_60::Item = carry(self);
        // A top-level `description.category` of a file from 1.20.20 on is no member of its
        // format: the game reads the file without a word and writes the document on without
        // it (its own 1.20.50 items, re-serialised, have none). The formats from 1.20.60 on
        // tell of such a member, so it is not carried into them: noted where the file has
        // it (an information of the crate's own; the game discards it too).
        if doc.description.extra.remove("category").is_some() {
            report(
                cx,
                DiagnosticCode::DroppedField { text: None },
                Severity::Info,
                &["description", "category"],
            );
        }
        let mut moved_icon = false;
        let mut replaced_textures = false;
        replace(&mut doc.components, "minecraft:icon", |c| match c {
            Component::LegacyIcon(mut old) => {
                // A `textures` the file has beside `texture` is no member of the 1.20.50
                // icon: the move writes over it.
                replaced_textures = old.extra.remove("textures").is_some();
                moved_icon = true;
                let mut icon = Icon::new(old.texture);
                icon.extra = old.extra;
                Some(Component::Icon(icon))
            }
            other => Some(other),
        });
        if replaced_textures {
            replaced(cx, &["components", "minecraft:icon", "textures"]);
        }
        if moved_icon {
            cx.diagnostics.moved(
                member(&["components", "minecraft:icon", "texture"]),
                member(&["components", "minecraft:icon", "textures", "default"]),
            );
        }
        doc
    }
}

/// a plain copy.
impl Upgrade for v1_20_60::Item {
    type Next = v1_20_80::Item;
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        carry(self)
    }
}

/// a plain copy.
impl Upgrade for v1_20_80::Item {
    type Next = v1_21_10::Item;
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        carry(self)
    }
}

/// a plain copy, but for a conversion of the crate's own: a `minecraft:rarity` of a file from
/// 1.20.30 on, which those formats bind no component for and the legacy reader takes, becomes the component 1.21.30
/// binds, so that the upgraded item holds it where it is written, and is told what a file of
/// 1.21.30 with that value is told.
impl Upgrade for v1_21_10::Item {
    type Next = v1_21_30::Item;
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        const RARITY: &str = "minecraft:rarity";
        let mut doc: v1_21_30::Item = carry(self);
        doc.legacy.held_in_shared_alone.retain(|key| key != RARITY);
        if doc.legacy.format == ItemFormat::DataDriven
            && !doc.components.contains_key(RARITY)
            && let Some(raw) = doc.legacy.shared.get(RARITY)
        {
            // Read as the struct of 1.21.30 reads the key, from the value with its number
            // tags and signs as the file had them; what that reading says is the upgrade's
            // to say, at the key, and a value it reads nothing of goes as a component of
            // that format goes.
            let text = bedrock_addon_core::json::to_json(raw).unwrap_or_default();
            let mut own = bedrock_addon_core::diagnostic::Diagnostics::default();
            let read = super::document::binding(FormatVersion::new(1, 21, 30), RARITY).and_then(
                |parser| {
                    parser(
                        bedrock_addon_core::json::JsonSlice::new(&text),
                        cx.load,
                        &mut own,
                    )
                },
            );
            // The game's legacy reader takes the key of such a file in silence; what the
            // 1.21.30 reader says of it is the crate's own, an information, its refusals
            // included.
            if read.is_none() && !own.iter().any(|told| told.location.json_path.is_empty()) {
                report(
                    cx,
                    DiagnosticCode::DroppedComponent { text: None },
                    Severity::Info,
                    &["components", RARITY],
                );
            }
            for mut told in own {
                let mut at = member(&["components", RARITY]);
                at.append(&told.location.json_path);
                told.location.json_path = at;
                told.location.span = None;
                told.phase = Phase::Upgrade;
                if told.severity == Severity::Error {
                    told.severity = Severity::Info;
                }
                cx.diagnostics.push(told);
            }
            if let Some(rarity) = read {
                doc.components.insert(RARITY.to_owned(), rarity);
            }
        }
        doc
    }
}

/// `block_placer` gains `replace_block_item: false`.
impl Upgrade for v1_21_30::Item {
    type Next = v1_21_40::Item;
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        let mut doc: v1_21_40::Item = carry(self);
        if let Some(Component::BlockPlacer(placer)) =
            doc.components.get_mut("minecraft:block_placer")
        {
            // The 1.21.40 upgrade of the planter: the block item is not replaced.
            placer.replace_block_item = Some(false);
        }
        doc
    }
}

/// a plain copy.
impl Upgrade for v1_21_40::Item {
    type Next = v1_21_50::Item;
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        carry(self)
    }
}

/// `menu_category` gets `category: items` and a `minecraft:`-prefixed group; the
/// custom component names become entries; the storage weights split into their own
/// components.
impl Upgrade for v1_21_50::Item {
    type Next = v1_21_60::Item;
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut doc: v1_21_60::Item = carry(self);
        if doc.legacy.format == ItemFormat::DataDriven {
            // The 1.21.60 menu category upgrade.
            let menu = doc
                .description
                .menu_category
                .get_or_insert_with(MenuCategory::default);
            if menu.category.is_none() {
                menu.category = Some(CreativeCategory::Items);
            }
            if let Some(group) = menu.group.as_mut()
                && !group.is_empty()
            {
                *group = format!("minecraft:{group}");
                report(
                    cx,
                    DiagnosticCode::DeprecatedMember { text: None },
                    Severity::Info,
                    &["description", "menu_category", "group"],
                );
            }
        }
        replace(
            &mut doc.components,
            "minecraft:custom_components",
            |c| match c {
                // The 1.21.60 custom components upgrade; that `data` becomes `{}` is not
                // verified against the game.
                Component::CustomComponentList(list) => {
                    Some(Component::CustomComponents(CustomComponents {
                        components: list
                            .names
                            .into_iter()
                            .map(|name| CustomComponentEntry {
                                name,
                                data: Value::from(VObject::new()),
                            })
                            .collect(),
                    }))
                }
                other => Some(other),
            },
        );
        let mut weights = None;
        replace(&mut doc.components, "minecraft:storage_item", |c| match c {
            Component::LegacyStorageItem(old) => {
                weights = Some((old.max_weight_limit, old.weight_in_storage_item));
                Some(Component::StorageItem(StorageItem {
                    max_slots: old.max_slots,
                    allow_nested_storage_items: old.allow_nested_storage_items,
                    banned_items: old.banned_items,
                    allowed_items: old.allowed_items,
                    extra: old.extra,
                }))
            }
            other => Some(other),
        });
        if let Some((limit, weight)) = weights {
            // No slice binds the two components before 1.21.60: one
            // the file has is no component to the game, and the split's own takes its
            // place. Noted where the file has it.
            for key in [
                "minecraft:storage_weight_limit",
                "minecraft:storage_weight_modifier",
            ] {
                if doc.components.contains_key(key) {
                    replaced(cx, &["components", key]);
                }
            }
            // created even when the values equal their defaults (the game's own bundles ship `{}`)
            doc.components.insert(
                "minecraft:storage_weight_limit".to_owned(),
                Component::StorageWeightLimit(StorageWeightLimit {
                    max_weight_limit: limit,
                    ..StorageWeightLimit::default()
                }),
            );
            doc.components.insert(
                "minecraft:storage_weight_modifier".to_owned(),
                Component::StorageWeightModifier(StorageWeightModifier {
                    weight_in_storage_item: weight,
                    ..StorageWeightModifier::default()
                }),
            );
        }
        doc
    }
}

/// a plain copy.
impl Upgrade for v1_21_60::Item {
    type Next = v1_21_80::Item;
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        carry(self)
    }
}

/// a plain copy.
impl Upgrade for v1_21_80::Item {
    type Next = v1_21_90::Item;
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        carry(self)
    }
}

/// the creative category becomes `CreativeItemCategory`; a 1.21.60–1.21.109 document
/// without one keeps `Undefined`.
impl Upgrade for v1_21_90::Item {
    type Next = v1_21_110::Item;
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut doc: v1_21_110::Item = carry(self);
        if doc.legacy.format == ItemFormat::DataDriven {
            // The 1.21.110 menu category upgrade: the smaller of the category and `Undefined`.
            let menu = doc
                .description
                .menu_category
                .get_or_insert_with(MenuCategory::default);
            let category = *menu.category.get_or_insert(CreativeCategory::Undefined);
            if category == CreativeCategory::Undefined {
                // The game keeps `Undefined`, and no format from here on spells it: the
                // item is written with `none`, which reads back as the command-only
                // category. The game says nothing, so this is an information of the
                // crate's own.
                report(
                    cx,
                    DiagnosticCode::ValueRewritten { text: None },
                    Severity::Info,
                    &["description", "menu_category"],
                );
            }
        }
        doc
    }
}

/// `damage` is copied unchanged (the crate's own choice; vanilla keeps the low byte), `block_placer`
/// gains `aligned_placement: false`.
impl Upgrade for v1_21_110::Item {
    type Next = v1_26_0::Item;
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Self::Next {
        let mut doc: v1_26_0::Item = carry(self);
        if let Some(Component::BlockPlacer(placer)) =
            doc.components.get_mut("minecraft:block_placer")
        {
            placer.aligned_placement = Some(false);
        }
        doc
    }
}

/// Moves the old `custom_components` entries into keyed custom components of `map` (the crate's own
/// conversion; vanilla keeps the list only in the side-car); a keyed entry of the same name wins.
pub(crate) fn custom_components_to_keyed(
    list: &CustomComponents,
    map: &mut ComponentMap<ItemComponent>,
    cx: &mut UpgradeCx<'_>,
) {
    for (index, entry) in list.components.iter().enumerate() {
        // A name that is no `ns:name` is no key of the keyed form: the 1.26.30 format reads
        // such a key as an unknown component and refuses it. It stays in the list the
        // side-car holds.
        if !super::document::is_custom_key(&entry.name) {
            let mut at = member(&["components", "minecraft:custom_components"]);
            at.push_index(index);
            cx.diagnostics.push(Diagnostic::new(
                DiagnosticCode::DroppedField { text: None },
                Severity::Info,
                Phase::Upgrade,
                Location::new(None, None, at, None),
            ));
            continue;
        }
        let taken = map.iter().any(|e| match e {
            bedrock_addon_core::document::ComponentRef::Custom(c) => c.name == entry.name,
            bedrock_addon_core::document::ComponentRef::Unknown { name, .. }
            | bedrock_addon_core::document::ComponentRef::Typed { name, .. } => name == entry.name,
            _ => false,
        });
        if taken {
            report(
                cx,
                DiagnosticCode::DroppedField { text: None },
                Severity::Info,
                &["components", "minecraft:custom_components"],
            );
            continue;
        }
        map.insert_custom(CustomComponent {
            name: entry.name.clone(),
            parameters: entry.data.clone(),
            from_v1_list: true,
        });
    }
}

/// The step from 1.26.0 to 1.26.30: an existing `use_modifiers` starts `always`; the custom
/// component list becomes keyed entries (the crate's own conversion) and moves to the
/// side-car; a digger with triggers is copied to the side-car; the legacy use duration is
/// rebuilt into `use_modifiers` (the crate's own conversion: `ticks / 20.0`).
impl Upgrade for v1_26_0::Item {
    type Next = v1_26_30::Item;
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> v1_26_30::Item {
        let mut components = self.components;
        let mut deprecated = self.deprecated;
        if let Some(Component::UseModifiers(modifiers)) =
            components.get_mut("minecraft:use_modifiers")
        {
            modifiers.start_using = StartUsing::Always;
        }
        let custom = match components.shift_remove("minecraft:custom_components") {
            Some(Component::CustomComponents(list)) => Some(list),
            Some(Component::CustomComponentList(list)) => Some(CustomComponents {
                components: list
                    .names
                    .into_iter()
                    .map(|name| CustomComponentEntry {
                        name,
                        data: Value::from(VObject::new()),
                    })
                    .collect(),
            }),
            _ => None,
        };
        if let Some(Component::Digger(digger)) = components.get("minecraft:digger")
            && digger.has_triggers()
        {
            // The digger stays in the components.
            copy_triggered_digger(&mut deprecated, digger, cx);
        }
        let mut map = into_component_map(components);
        if let Some(list) = custom {
            custom_components_to_keyed(&list, &mut map, cx);
            fill(
                cx,
                &mut deprecated.custom_components_v1,
                "custom_components_v1",
                list.to_value(),
            );
        }
        let negative = negative_duration(&self.legacy, cx);
        if let Some(ticks) = self.legacy.use_duration_ticks().filter(|_| !negative)
            && self.legacy.format == ItemFormat::DataDriven
        {
            saturated_duration(&self.legacy, cx);
            // Vanilla divides the ticks by 20 as integers.
            #[allow(clippy::cast_precision_loss, reason = "ticks of a use duration")]
            let seconds = ticks as f32 / 20.0;
            match map.get_mut::<UseModifiers>() {
                Some(modifiers) => {
                    // If `use_modifiers` is present, its `use_duration` is overwritten. The
                    // game discards the component's own: noted where the file has it.
                    #[allow(clippy::float_cmp, reason = "the same number, not one near it")]
                    if modifiers.use_duration != seconds {
                        replaced(
                            cx,
                            &["components", "minecraft:use_modifiers", "use_duration"],
                        );
                    }
                    modifiers.use_duration = seconds;
                }
                None => {
                    map.insert_typed(
                        ItemComponent::UseModifiers(UseModifiers::new(seconds)),
                        true,
                    );
                }
            }
        }
        let identifier = bedrock_addon_core::id::ItemId::parse(self.description.identifier);
        // What the document wrote, read before the format's default is filled in.
        let menu_category_presence =
            crate::MenuCategoryPresence::of(self.description.menu_category.as_ref());
        let menu_category = if self.legacy.format == ItemFormat::DataDriven {
            let mut menu = self.description.menu_category.unwrap_or_default();
            menu.category
                .get_or_insert(CreativeCategory::ItemCommandOnly);
            menu
        } else {
            MenuCategory::default()
        };
        let mut extra = self.description.extra;
        if let Some(category) = self.description.category {
            extra.insert("category".to_owned(), Value::from(category.as_str()));
        }
        v1_26_30::Item {
            description: ItemDescription {
                identifier,
                menu_category,
                menu_category_presence,
                extra,
            },
            components: map,
            deprecated,
            events: self.events,
            // The duration is in `use_modifiers` now: the newest writer does not write the
            // key the legacy reading held it under a second time.
            legacy: {
                let mut legacy = self.legacy;
                legacy
                    .held_in_shared_alone
                    .retain(|key| key != "minecraft:use_duration");
                legacy
            },
            extra: self.extra,
        }
    }
}

/// The step from 1.26.30 to 1.26.50. `use_modifiers.start_sound`, the three
/// `swing_sounds` and each `durability_sensor` threshold's `sound_event` change from a
/// `LevelSoundEvent` to a `SoundEventName`, written as the event's canonical name;
/// everything else is carried by the generic visitor.
///
/// The crate holds a level sound event as its name, which the 1.26.30 struct accepts in any
/// case: the step writes the canonical, lower-case name. A name already canonical is
/// moved as it is, so the step is an identity on every vanilla item.
impl Upgrade for v1_26_30::Item {
    type Next = Item;
    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Item {
        let mut components = self.components;
        if let Some(modifiers) = components.get_mut::<UseModifiers>()
            && let Some(name) = modifiers.start_sound.as_mut()
        {
            let at = member(&["components", "minecraft:use_modifiers", "start_sound"]);
            canonical_sound(name, cx, at);
        }
        if let Some(swing) = components.get_mut::<SwingSounds>() {
            for (key, name) in [
                ("attack_miss", &mut swing.attack_miss),
                ("attack_hit", &mut swing.attack_hit),
                ("attack_critical_hit", &mut swing.attack_critical_hit),
            ] {
                if let Some(name) = name.as_mut() {
                    let at = member(&["components", "minecraft:swing_sounds", key]);
                    canonical_sound(name, cx, at);
                }
            }
        }
        if let Some(sensor) = components.get_mut::<DurabilitySensor>() {
            for (index, threshold) in sensor.durability_thresholds.iter_mut().enumerate() {
                if let Some(name) = threshold.sound_event.as_mut() {
                    let mut at = member(&[
                        "components",
                        "minecraft:durability_sensor",
                        "durability_thresholds",
                    ]);
                    at.push_index(index);
                    at.push("sound_event");
                    canonical_sound(name, cx, at);
                }
            }
        }
        Item {
            description: self.description,
            components,
            deprecated: self.deprecated,
            events: self.events,
            legacy: self.legacy,
            extra: self.extra,
        }
    }
}

/// A level sound event `name` as the game writes it: the table's spelling (lower
/// case), with an information where the step changed it.
fn canonical_sound(name: &mut SoundEventName, cx: &mut UpgradeCx<'_>, at: JsonPath) {
    if let Some(canonical) = crate::sounds::level_sound_event(name.as_str())
        && canonical != name.as_str()
    {
        *name = SoundEventName::parse(canonical);
        cx.diagnostics.push(Diagnostic::new(
            DiagnosticCode::ValueRewritten { text: None },
            Severity::Info,
            Phase::Upgrade,
            Location::new(None, None, at, None),
        ));
    }
}

/// The step from 1.26.50 to beta moves every member unchanged; only `allow_off_hand` and
/// `record` are re-bound in the beta struct. Never run on a stable document: `"beta"` alone
/// selects the beta struct.
impl Upgrade for Item {
    type Next = Beta;
    fn upgrade(self, _: &mut UpgradeCx<'_>) -> Beta {
        Beta {
            description: self.description,
            components: self.components,
            deprecated: self.deprecated,
            events: self.events,
            legacy: self.legacy,
            extra: self.extra,
        }
    }
}

#[allow(dead_code, reason = "the 1.26.0 version the damage step targets")]
const V1_26_0: FormatVersion = FormatVersion::new(1, 26, 0);
