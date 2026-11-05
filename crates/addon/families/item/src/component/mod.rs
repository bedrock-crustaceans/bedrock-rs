//! The components of an item (`minecraft:item` → `components`), as the newest format version
//! (1.26.30) binds them: 44 typed components plus custom components keyed by their own name.
//!
//! Component types drop the `minecraft:` prefix (`minecraft:max_stack_size` →
//! [`MaxStackSize`](crate::component::MaxStackSize)). Components that hold one value also read the bare value
//! (`"minecraft:glint": true` is `{"value": true}`); they are written in the object form.

#![doc(alias = "item_components")]

mod combat;
mod equipment;
mod look;
mod placement;
mod storage;
mod use_;
mod value;

pub use combat::{
    Cooldown, CooldownType, Damage, KineticCondition, KineticWeapon, PiercingWeapon, Projectile,
    Shooter, ShooterAmmunition, SwingDuration, SwingSounds, Throwable,
};
pub use equipment::{
    DamageAbsorption, Durability, DurabilitySensor, DurabilityThreshold, Dyeable, Enchantable,
    RepairItem, Repairable, Wearable, WearableSlot,
};
pub use look::{DisplayName, Glint, HoverTextColor, Icon, InteractButton, Rarity, Record, Tags};
pub use placement::{BlockPlacer, DestroySpeed, Digger, EntityPlacer};
pub use storage::{BundleInteraction, StorageItem, StorageWeightLimit, StorageWeightModifier};
pub use use_::{Compostable, Food, Fuel, StartUsing, UseAnimation, UseAnimationKind, UseModifiers};
pub use value::{
    AllowOffHand, CanDestroyInCreative, FireResistant, HandEquipped, LiquidClipped, MaxStackSize,
    ShouldDespawn, StackedByData,
};

pub(crate) use equipment::ENCHANT_SLOTS;
pub(crate) use value::{ValueComponent, is_object as value_is_object, parse_value, read_bare};

use bedrock_addon_core::document::CustomComponent;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::{LoadContext, LoadRequirement};
use bedrock_addon_core::molang::{MolangFieldSpec, MolangVersionSource};
use bedrock_addon_core::versioning::{ConvertMode, parse_component};
use bedrock_addon_derive::Components;

use crate::history::ItemFamily;
use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics};

/// `LegacyCreatorFeatures`: satisfied only by the ten vanilla pack UUIDs.
const LCF: LoadRequirement = LoadRequirement::LegacyCreatorFeatures;

/// One component of an item: a typed component, a custom component (`ns:name`, from format
/// 1.21.90), or a key the format does not bind, kept for write-back.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = ItemFamily, custom = Custom, unknown = Unknown, write)]
#[non_exhaustive]
pub enum ItemComponent {
    /// `minecraft:allow_off_hand`.
    #[component(name = "minecraft:allow_off_hand", shorthand = scalar, released = "1.20.20", parse = typed::<AllowOffHand>)]
    AllowOffHand(AllowOffHand),
    /// `minecraft:block_placer`.
    #[component(name = "minecraft:block_placer", released = "1.20.10", requires = [LCF], fields = [{ name = "aligned_placement", since_engine = "1.26.10" }], molang, parse = typed::<BlockPlacer>)]
    BlockPlacer(BlockPlacer),
    /// `minecraft:bundle_interaction`.
    #[component(name = "minecraft:bundle_interaction", released = "1.21.30", parse = typed::<BundleInteraction>)]
    BundleInteraction(BundleInteraction),
    /// `minecraft:can_destroy_in_creative`.
    #[component(name = "minecraft:can_destroy_in_creative", shorthand = scalar, released = "1.20.10", parse = typed::<CanDestroyInCreative>)]
    CanDestroyInCreative(CanDestroyInCreative),
    /// `minecraft:compostable`.
    #[component(name = "minecraft:compostable", released = "1.21.50", parse = typed::<Compostable>)]
    Compostable(Compostable),
    /// `minecraft:cooldown`.
    #[component(name = "minecraft:cooldown", released = "1.20.10", requires = [LCF], fields = [{ name = "type", since_engine = "1.21.130" }], parse = typed::<Cooldown>)]
    Cooldown(Cooldown),
    /// `minecraft:damage`.
    #[component(name = "minecraft:damage", shorthand = scalar, released = "1.20.20", parse = typed::<Damage>)]
    Damage(Damage),
    /// `minecraft:damage_absorption`.
    #[component(name = "minecraft:damage_absorption", released = "1.21.10", parse = typed::<DamageAbsorption>)]
    DamageAbsorption(DamageAbsorption),
    /// `minecraft:digger`.
    #[component(name = "minecraft:digger", released = "1.20.20", requires = [LCF], molang, parse = typed::<Digger>)]
    Digger(Digger),
    /// `minecraft:display_name`.
    #[component(name = "minecraft:display_name", released = "1.20.0", requires = [LCF], parse = typed::<DisplayName>)]
    DisplayName(DisplayName),
    /// `minecraft:durability`.
    #[component(name = "minecraft:durability", released = "1.20.0", requires = [LCF], parse = typed::<Durability>)]
    Durability(Durability),
    /// `minecraft:durability_sensor`.
    #[component(name = "minecraft:durability_sensor", released = "1.21.10", parse = typed::<DurabilitySensor>)]
    DurabilitySensor(DurabilitySensor),
    /// `minecraft:dyeable`.
    #[component(name = "minecraft:dyeable", released = "1.21.30", requires = [LCF], parse = typed::<Dyeable>)]
    Dyeable(Dyeable),
    /// `minecraft:enchantable`.
    #[component(name = "minecraft:enchantable", released = "1.20.30", parse = typed::<Enchantable>)]
    Enchantable(Enchantable),
    /// `minecraft:entity_placer`.
    #[component(name = "minecraft:entity_placer", released = "1.20.0", requires = [LCF], molang, parse = typed::<EntityPlacer>)]
    EntityPlacer(EntityPlacer),
    /// `minecraft:fire_resistant`.
    #[component(name = "minecraft:fire_resistant", released = "1.21.90", parse = typed::<FireResistant>)]
    FireResistant(FireResistant),
    /// `minecraft:food`.
    #[component(name = "minecraft:food", released = "1.20.30", requires = [LCF], molang, parse = typed::<Food>)]
    Food(Food),
    /// `minecraft:fuel`.
    #[component(name = "minecraft:fuel", shorthand = scalar, released = "1.20.0", requires = [LCF], parse = typed::<Fuel>)]
    Fuel(Fuel),
    /// `minecraft:glint`, which supersedes the legacy `minecraft:foil`.
    #[component(name = "minecraft:glint", shorthand = scalar, released = "1.20.20", supersedes = "minecraft:foil", parse = typed::<Glint>)]
    Glint(Glint),
    /// `minecraft:hand_equipped`.
    #[component(name = "minecraft:hand_equipped", shorthand = scalar, released = "1.20.20", parse = typed::<HandEquipped>)]
    HandEquipped(HandEquipped),
    /// `minecraft:hover_text_color`.
    #[component(name = "minecraft:hover_text_color", shorthand = scalar, released = "1.20.10", parse = typed::<HoverTextColor>)]
    HoverTextColor(HoverTextColor),
    /// `minecraft:icon`.
    #[component(name = "minecraft:icon", shorthand = scalar, released = "1.20.0", requires = [LCF], parse = parse_value::<Icon>)]
    Icon(Icon),
    /// `minecraft:interact_button`.
    #[component(name = "minecraft:interact_button", released = "1.20.30", parse = look::parse_interact_button)]
    InteractButton(InteractButton),
    /// `minecraft:kinetic_weapon`.
    #[component(name = "minecraft:kinetic_weapon", released = "1.21.90", fields = [{ name = "", since_engine = "1.21.130" }], parse = typed::<KineticWeapon>)]
    KineticWeapon(KineticWeapon),
    /// `minecraft:liquid_clipped`.
    #[component(name = "minecraft:liquid_clipped", shorthand = scalar, released = "1.20.20", parse = typed::<LiquidClipped>)]
    LiquidClipped(LiquidClipped),
    /// `minecraft:max_stack_size`.
    #[component(name = "minecraft:max_stack_size", shorthand = scalar, released = "1.20.10", parse = typed::<MaxStackSize>)]
    MaxStackSize(MaxStackSize),
    /// `minecraft:piercing_weapon`.
    #[component(name = "minecraft:piercing_weapon", released = "1.21.90", fields = [{ name = "", since_engine = "1.21.130" }], parse = typed::<PiercingWeapon>)]
    PiercingWeapon(PiercingWeapon),
    /// `minecraft:projectile`.
    #[component(name = "minecraft:projectile", released = "1.20.10", requires = [LCF], parse = typed::<Projectile>)]
    Projectile(Projectile),
    /// `minecraft:rarity`.
    #[component(name = "minecraft:rarity", shorthand = scalar, released = "1.21.30", parse = typed::<Rarity>)]
    Rarity(Rarity),
    /// `minecraft:record`.
    #[component(name = "minecraft:record", released = "1.20.10", requires = [LCF], parse = typed::<Record>)]
    Record(Record),
    /// `minecraft:repairable`.
    #[component(name = "minecraft:repairable", released = "1.20.10", requires = [LCF], molang, parse = typed::<Repairable>)]
    Repairable(Repairable),
    /// `minecraft:shooter`.
    #[component(name = "minecraft:shooter", released = "1.20.10", requires = [LCF], molang, parse = typed::<Shooter>)]
    Shooter(Shooter),
    /// `minecraft:should_despawn`.
    #[component(name = "minecraft:should_despawn", shorthand = scalar, released = "1.20.20", parse = typed::<ShouldDespawn>)]
    ShouldDespawn(ShouldDespawn),
    /// `minecraft:stacked_by_data`.
    #[component(name = "minecraft:stacked_by_data", shorthand = scalar, released = "1.20.20", parse = typed::<StackedByData>)]
    StackedByData(StackedByData),
    /// `minecraft:storage_item`.
    #[component(name = "minecraft:storage_item", released = "1.21.30", molang, parse = typed::<StorageItem>)]
    StorageItem(StorageItem),
    /// `minecraft:storage_weight_limit`.
    #[component(name = "minecraft:storage_weight_limit", released = "1.21.60", parse = typed::<StorageWeightLimit>)]
    StorageWeightLimit(StorageWeightLimit),
    /// `minecraft:storage_weight_modifier`.
    #[component(name = "minecraft:storage_weight_modifier", released = "1.21.60", parse = typed::<StorageWeightModifier>)]
    StorageWeightModifier(StorageWeightModifier),
    /// `minecraft:swing_duration`.
    #[component(name = "minecraft:swing_duration", shorthand = scalar, released = "1.21.90", fields = [{ name = "", since_engine = "1.21.120" }], parse = typed::<SwingDuration>)]
    SwingDuration(SwingDuration),
    /// `minecraft:swing_sounds`.
    #[component(name = "minecraft:swing_sounds", released = "1.21.90", fields = [{ name = "", since_engine = "1.21.130" }], parse = typed::<SwingSounds>)]
    SwingSounds(SwingSounds),
    /// `minecraft:tags`.
    #[component(name = "minecraft:tags", released = "1.20.50", parse = typed::<Tags>)]
    Tags(Tags),
    /// `minecraft:throwable`.
    #[component(name = "minecraft:throwable", released = "1.20.10", requires = [LCF], parse = typed::<Throwable>)]
    Throwable(Throwable),
    /// `minecraft:use_animation`.
    #[component(name = "minecraft:use_animation", shorthand = scalar, released = "1.20.20", parse = typed::<UseAnimation>)]
    UseAnimation(UseAnimation),
    /// `minecraft:use_modifiers`.
    #[component(name = "minecraft:use_modifiers", released = "1.20.50", parse = typed::<UseModifiers>)]
    UseModifiers(UseModifiers),
    /// `minecraft:wearable`.
    #[component(name = "minecraft:wearable", released = "1.20.20", requires = [LCF], parse = typed::<Wearable>)]
    Wearable(Wearable),
    /// A custom component (`"ns:name": {…}`), its parameters kept as written.
    Custom(CustomComponent),
    /// A key the format does not bind, kept for write-back.
    Unknown(Value),
}

/// The parser of a typed component's table row: the component from its slice, its unknown
/// members kept and reported.
pub(crate) fn typed<T>(
    slice: JsonSlice<'_>,
    mode: ConvertMode,
    cx: &LoadContext,
    d: &mut Diagnostics,
) -> Option<ItemComponent>
where
    T: for<'a> facet::Facet<'a> + Into<ItemComponent>,
{
    let component = parse_component::<T, ItemComponent>(slice, mode, cx, d);
    if component.is_some() {
        // A member the schema does not know is an error under the strict slices, which
        // refuses the item; the modern slices are the only readers of this table (the
        // write-back probe of `write.rs` reads only the `Unrecognized*` codes).
        tell_range_members::<T>(slice, d, bedrock_addon_core::diagnostic::Severity::Error);
    }
    component
}

/// Tells of the members of a range given as an object that the range does not read
/// (`damage_chance: {"min": 1, "max": 5, "zz": 3}`): a range has no place for them, so they
/// are not kept as a component's own unknown members are; each is a `MemberNotKept` at the
/// member at `severity`: the game binds `min` and `max` alone for an integer and a float
/// range, so any other member is "found in the input, but is not present in the Schema": an
/// error under the strict slices and a level-2 line (a warning) under the unstrict ones.
pub(crate) fn tell_range_members<T>(
    slice: JsonSlice<'_>,
    d: &mut Diagnostics,
    severity: bedrock_addon_core::diagnostic::Severity,
) where
    T: for<'a> facet::Facet<'a>,
{
    use bedrock_addon_core::json::{
        from_str_jsonc, keep_members, keep_members_and_range_members, parse_at,
    };
    let (Ok(json), Ok(plain), Ok(ranged)) = (
        from_str_jsonc::<Value>(slice.text),
        parse_at::<T>(slice),
        parse_at::<T>(slice),
    ) else {
        return;
    };
    // Told only: the component's own parse keeps its members.
    let mut none = bedrock_addon_core::document::Unknown::new();
    let known: Vec<bedrock_addon_core::diagnostic::JsonPath> =
        keep_members(&plain, &json, &mut none)
            .0
            .into_iter()
            .map(|(path, _)| path)
            .collect();
    for (path, _) in keep_members_and_range_members(&ranged, &json, &mut none).0 {
        if !known.contains(&path) {
            d.push(
                Diagnostic::error(
                    bedrock_addon_core::diagnostic::DiagnosticCode::MemberNotKept { text: None },
                )
                .severity(severity)
                .at(path),
            );
        }
    }
}

/// `repair_amount`: the version of the item's `format_version`
/// (the Molang version mapped from `format_version`), the default query set. The item
/// descriptor `tags` use [`ITEM_TAGS_MOLANG`], block descriptor `tags` [`BLOCK_TAGS_MOLANG`],
/// and the legacy event / trigger / `shoot` members the specs of `item::expressions`.
///
/// The game logs the Molang parser's refusal at the content log's error level and the item
/// loads on with the expression unparsed (nothing checks the result of parsing a descriptor's
/// `tags`, and `repair_amount` is evaluated without checking the parse): kept errors, for all
/// three.
pub(crate) const FORMAT_VERSION_MOLANG: MolangFieldSpec =
    MolangFieldSpec::KEPT.version(MolangVersionSource::FormatVersion);

/// An item descriptor's `{"tags": …}`: the item's version, the `tags` query set.
pub(crate) const ITEM_TAGS_MOLANG: MolangFieldSpec = MolangFieldSpec::KEPT
    .version(MolangVersionSource::FormatVersion)
    .tags();

/// A block descriptor's `{"tags": …}` in an item component: a constant 13, the `tags` query
/// set.
pub(crate) const BLOCK_TAGS_MOLANG: MolangFieldSpec = MolangFieldSpec::KEPT
    .version(MolangVersionSource::Constant(
        bedrock_addon_core::molang::MolangVersion::V13,
    ))
    .tags();

/// Visits the `tags` expression of an item descriptor.
pub(crate) fn visit_item_descriptor(
    descriptor: &mut bedrock_addon_core::shared::ItemDescriptor,
    visitor: &mut bedrock_addon_core::molang::MolangVisitor<'_>,
) {
    if let bedrock_addon_core::shared::ItemDescriptor::Tags { tags, .. } = descriptor {
        visitor.field("tags", &ITEM_TAGS_MOLANG, tags);
    }
}

/// Visits the `tags` expression of every item descriptor of a list.
pub(crate) fn visit_item_descriptors(
    list: &mut [bedrock_addon_core::shared::ItemDescriptor],
    visitor: &mut bedrock_addon_core::molang::MolangVisitor<'_>,
) {
    for (index, descriptor) in list.iter_mut().enumerate() {
        visitor.enter(index.into());
        visit_item_descriptor(descriptor, visitor);
        visitor.leave();
    }
}

/// Visits the `tags` expression of an item descriptor, when one is written.
pub(crate) fn visit_optional_item_descriptor(
    descriptor: &mut Option<bedrock_addon_core::shared::ItemDescriptor>,
    visitor: &mut bedrock_addon_core::molang::MolangVisitor<'_>,
) {
    if let Some(descriptor) = descriptor {
        visit_item_descriptor(descriptor, visitor);
    }
}

/// Visits the `tags` expression of every block descriptor of a list.
pub(crate) fn visit_block_descriptors(
    list: &mut [bedrock_addon_core::shared::BlockDescriptor],
    visitor: &mut bedrock_addon_core::molang::MolangVisitor<'_>,
) {
    for (index, descriptor) in list.iter_mut().enumerate() {
        visitor.enter(index.into());
        visit_block_descriptor(descriptor, visitor);
        visitor.leave();
    }
}

/// Visits the `tags` expression of a block descriptor.
pub(crate) fn visit_block_descriptor(
    descriptor: &mut bedrock_addon_core::shared::BlockDescriptor,
    visitor: &mut bedrock_addon_core::molang::MolangVisitor<'_>,
) {
    use bedrock_addon_core::shared::BlockDescriptor;
    if let BlockDescriptor::Tags { tags, .. } | BlockDescriptor::Mixed { tags, .. } = descriptor {
        visitor.field("tags", &BLOCK_TAGS_MOLANG, tags);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::JsonPath;
    use bedrock_addon_core::molang::{MolangFields, visit_fields};
    use bedrock_addon_core::schema::{ShorthandRows, misshapen_shorthand_rows};

    /// The paths and specs the Molang walk of the payload `text` visits.
    fn walked<T: for<'a> facet::Facet<'a> + MolangFields>(
        text: &str,
    ) -> Vec<(String, MolangFieldSpec)> {
        let mut value: T = bedrock_addon_core::json::from_str(text).expect("the payload reads");
        let mut seen = Vec::new();
        visit_fields(&mut value, &JsonPath::new(), |field| {
            seen.push((field.path.to_string(), *field.spec));
        });
        seen
    }

    fn at(path: &str, spec: MolangFieldSpec) -> (String, MolangFieldSpec) {
        (path.to_owned(), spec)
    }

    const ITEM_TAGS: &str = r#"{"tags": "q.any_tag('a')"}"#;
    const BLOCK_TAGS: &str = r#"{"tags": "q.any_tag('b')"}"#;

    #[test]
    fn the_descriptor_tags_and_repair_amounts_walk_under_the_items_specs() {
        let (i, b) = (ITEM_TAGS, BLOCK_TAGS);
        assert_eq!(
            walked::<BlockPlacer>(&format!(r#"{{"block": "a:b", "use_on": ["a:c", {b}]}}"#)),
            [at("use_on/1/tags", BLOCK_TAGS_MOLANG)]
        );
        assert_eq!(
            walked::<Digger>(&format!(
                r#"{{"destroy_speeds": [{{"block": "a:c", "speed": 1}}, {{"block": {b}, "speed": 2}}]}}"#
            )),
            [at("destroy_speeds/1/block/tags", BLOCK_TAGS_MOLANG)]
        );
        assert_eq!(
            walked::<EntityPlacer>(&format!(
                r#"{{"use_on": [{b}], "dispense_on": ["a:c", {b}]}}"#
            )),
            [
                at("use_on/0/tags", BLOCK_TAGS_MOLANG),
                at("dispense_on/1/tags", BLOCK_TAGS_MOLANG)
            ]
        );
        assert_eq!(
            walked::<Food>(&format!(r#"{{"using_converts_to": {i}}}"#)),
            [at("using_converts_to/tags", ITEM_TAGS_MOLANG)]
        );
        assert_eq!(
            walked::<Repairable>(&format!(
                r#"{{"repair_items": [{{"items": ["a:c", {i}], "repair_amount": "q.x"}}]}}"#
            )),
            [
                at("repair_items/0/items/1/tags", ITEM_TAGS_MOLANG),
                at("repair_items/0/repair_amount", FORMAT_VERSION_MOLANG)
            ]
        );
        assert_eq!(
            walked::<Shooter>(&format!(
                r#"{{"ammunition": [{{"item": "a:c"}}, {{"item": {i}}}]}}"#
            )),
            // Today's walk visits an entry's descriptor at the entry, without its `item` step.
            [at("ammunition/1/tags", ITEM_TAGS_MOLANG)]
        );
        assert_eq!(
            walked::<StorageItem>(&format!(
                r#"{{"banned_items": [{i}], "allowed_items": ["a:c", {i}]}}"#
            )),
            [
                at("banned_items/0/tags", ITEM_TAGS_MOLANG),
                at("allowed_items/1/tags", ITEM_TAGS_MOLANG)
            ]
        );
    }

    #[test]
    fn every_shorthand_row_reads_a_bare_scalar() {
        assert!(misshapen_shorthand_rows::<ItemComponent>().is_empty());
        // `icon`'s one member is its `textures` map, which its bare key is no value of: its
        // row reads the key by hand (`parse_value`), not as the member.
        assert!(
            ItemComponent::SHORTHAND_PAYLOADS
                .iter()
                .any(|(key, _)| *key == "minecraft:icon")
        );
    }
}
