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
pub(crate) use value::{ValueComponent, is_object as value_is_object, parse_value, tell_misread};

use bedrock_addon_core::document::CustomComponent;
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::{LoadContext, LoadRequirement};
use bedrock_addon_core::molang::{MolangFieldSpec, MolangVersionSource};
use bedrock_addon_core::versioning::{ConvertMode, parse_component};
use bedrock_addon_derive::Components;

use crate::load::ItemFamily;
use bedrock_addon_core::diagnostic::Diagnostics;

/// `LegacyCreatorFeatures`: satisfied only by the ten vanilla pack UUIDs.
const LCF: LoadRequirement = LoadRequirement::LegacyCreatorFeatures;

/// One component of an item: a typed component, a custom component (`ns:name`, from format
/// 1.21.90), or a key the format does not bind, kept for write-back.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(family = ItemFamily, custom = Custom, unknown = Unknown)]
#[non_exhaustive]
pub enum ItemComponent {
    /// `minecraft:allow_off_hand`.
    #[component(name = "minecraft:allow_off_hand", released = "1.20.20", parse = parse_value::<AllowOffHand>)]
    AllowOffHand(AllowOffHand),
    /// `minecraft:block_placer`.
    #[component(name = "minecraft:block_placer", released = "1.20.10", requires = [LCF], fields = [{ name = "aligned_placement", since_engine = "1.26.10" }], molang, parse = typed::<BlockPlacer>)]
    BlockPlacer(BlockPlacer),
    /// `minecraft:bundle_interaction`.
    #[component(name = "minecraft:bundle_interaction", released = "1.21.30", parse = typed::<BundleInteraction>)]
    BundleInteraction(BundleInteraction),
    /// `minecraft:can_destroy_in_creative`.
    #[component(name = "minecraft:can_destroy_in_creative", released = "1.20.10", parse = parse_value::<CanDestroyInCreative>)]
    CanDestroyInCreative(CanDestroyInCreative),
    /// `minecraft:compostable`.
    #[component(name = "minecraft:compostable", released = "1.21.50", parse = typed::<Compostable>)]
    Compostable(Compostable),
    /// `minecraft:cooldown`.
    #[component(name = "minecraft:cooldown", released = "1.20.10", requires = [LCF], fields = [{ name = "type", since_engine = "1.21.130" }], parse = typed::<Cooldown>)]
    Cooldown(Cooldown),
    /// `minecraft:damage`.
    #[component(name = "minecraft:damage", released = "1.20.20", parse = parse_value::<Damage>)]
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
    #[component(name = "minecraft:fuel", released = "1.20.0", requires = [LCF], parse = parse_value::<Fuel>)]
    Fuel(Fuel),
    /// `minecraft:glint`, which supersedes the legacy `minecraft:foil`.
    #[component(name = "minecraft:glint", released = "1.20.20", supersedes = "minecraft:foil", parse = parse_value::<Glint>)]
    Glint(Glint),
    /// `minecraft:hand_equipped`.
    #[component(name = "minecraft:hand_equipped", released = "1.20.20", parse = parse_value::<HandEquipped>)]
    HandEquipped(HandEquipped),
    /// `minecraft:hover_text_color`.
    #[component(name = "minecraft:hover_text_color", released = "1.20.10", parse = parse_value::<HoverTextColor>)]
    HoverTextColor(HoverTextColor),
    /// `minecraft:icon`.
    #[component(name = "minecraft:icon", released = "1.20.0", requires = [LCF], parse = parse_value::<Icon>)]
    Icon(Icon),
    /// `minecraft:interact_button`.
    #[component(name = "minecraft:interact_button", released = "1.20.30", parse = look::parse_interact_button)]
    InteractButton(InteractButton),
    /// `minecraft:kinetic_weapon`.
    #[component(name = "minecraft:kinetic_weapon", released = "1.21.90", fields = [{ name = "", since_engine = "1.21.130" }], parse = typed::<KineticWeapon>)]
    KineticWeapon(KineticWeapon),
    /// `minecraft:liquid_clipped`.
    #[component(name = "minecraft:liquid_clipped", released = "1.20.20", parse = parse_value::<LiquidClipped>)]
    LiquidClipped(LiquidClipped),
    /// `minecraft:max_stack_size`.
    #[component(name = "minecraft:max_stack_size", released = "1.20.10", parse = parse_value::<MaxStackSize>)]
    MaxStackSize(MaxStackSize),
    /// `minecraft:piercing_weapon`.
    #[component(name = "minecraft:piercing_weapon", released = "1.21.90", fields = [{ name = "", since_engine = "1.21.130" }], parse = typed::<PiercingWeapon>)]
    PiercingWeapon(PiercingWeapon),
    /// `minecraft:projectile`.
    #[component(name = "minecraft:projectile", released = "1.20.10", requires = [LCF], parse = typed::<Projectile>)]
    Projectile(Projectile),
    /// `minecraft:rarity`.
    #[component(name = "minecraft:rarity", released = "1.21.30", parse = parse_value::<Rarity>)]
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
    #[component(name = "minecraft:should_despawn", released = "1.20.20", parse = parse_value::<ShouldDespawn>)]
    ShouldDespawn(ShouldDespawn),
    /// `minecraft:stacked_by_data`.
    #[component(name = "minecraft:stacked_by_data", released = "1.20.20", parse = parse_value::<StackedByData>)]
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
    #[component(name = "minecraft:swing_duration", released = "1.21.90", fields = [{ name = "", since_engine = "1.21.120" }], parse = parse_value::<SwingDuration>)]
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
    #[component(name = "minecraft:use_animation", released = "1.20.20", parse = parse_value::<UseAnimation>)]
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
    let (Ok(json), Ok(mut plain), Ok(mut ranged)) = (
        from_str_jsonc::<Value>(slice.text),
        parse_at::<T>(slice),
        parse_at::<T>(slice),
    ) else {
        return;
    };
    let known: Vec<bedrock_addon_core::diagnostic::JsonPath> = keep_members(&mut plain, &json)
        .0
        .into_iter()
        .map(|(path, _)| path)
        .collect();
    for (path, _) in keep_members_and_range_members(&mut ranged, &json).0 {
        if !known.contains(&path) {
            d.push(bedrock_addon_core::diagnostic::Diagnostic::new(
                bedrock_addon_core::diagnostic::DiagnosticCode::MemberNotKept { text: None },
                severity,
                bedrock_addon_core::diagnostic::Phase::Parse,
                bedrock_addon_core::diagnostic::Location::new(None, None, path, None),
            ));
        }
    }
}

/// `repair_amount`: the version of the item's `format_version`
/// (the Molang version mapped from `format_version`), the default query set. The item
/// descriptor `tags` use [`ITEM_TAGS_MOLANG`], block descriptor `tags` [`BLOCK_TAGS_MOLANG`],
/// and the legacy event / trigger / `shoot` members the specs of `item::expressions`.
pub(crate) const FORMAT_VERSION_MOLANG: MolangFieldSpec = MolangFieldSpec {
    version: MolangVersionSource::FormatVersion,
    // The game logs the Molang parser's refusal at the content log's error level and the
    // item loads on with the expression unparsed (nothing checks the result of parsing a
    // descriptor's `tags`, and `repair_amount` is evaluated without checking the parse):
    // kept errors.
    error_severity: bedrock_addon_core::diagnostic::Severity::Error,
    error_kept: true,
    ..MolangFieldSpec::DEFAULT
};

/// An item descriptor's `{"tags": …}`: the item's version, the `tags` query set.
pub(crate) const ITEM_TAGS_MOLANG: MolangFieldSpec = MolangFieldSpec {
    version: MolangVersionSource::FormatVersion,
    admission: bedrock_addon_core::molang::Admission::Tags,
    // The game logs the Molang parser's refusal at the content log's error level and the
    // item loads on with the expression unparsed (nothing checks the result of parsing a
    // descriptor's `tags`, and `repair_amount` is evaluated without checking the parse):
    // kept errors.
    error_severity: bedrock_addon_core::diagnostic::Severity::Error,
    error_kept: true,
    ..MolangFieldSpec::DEFAULT
};

/// A block descriptor's `{"tags": …}` in an item component: a constant 13, the `tags` query
/// set.
pub(crate) const BLOCK_TAGS_MOLANG: MolangFieldSpec = MolangFieldSpec {
    version: MolangVersionSource::Constant(bedrock_addon_core::molang::MolangVersion::V13),
    admission: bedrock_addon_core::molang::Admission::Tags,
    // The game logs the Molang parser's refusal at the content log's error level and the
    // item loads on with the expression unparsed (nothing checks the result of parsing a
    // descriptor's `tags`, and `repair_amount` is evaluated without checking the parse):
    // kept errors.
    error_severity: bedrock_addon_core::diagnostic::Severity::Error,
    error_kept: true,
    ..MolangFieldSpec::DEFAULT
};

/// Visits the `tags` expression of an item descriptor.
pub(crate) fn visit_item_descriptor(
    descriptor: &mut bedrock_addon_core::shared::ItemDescriptor,
    visitor: &mut bedrock_addon_core::molang::MolangVisitor<'_>,
) {
    if let bedrock_addon_core::shared::ItemDescriptor::Tags { tags, .. } = descriptor {
        visitor.field("tags", &ITEM_TAGS_MOLANG, tags);
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
