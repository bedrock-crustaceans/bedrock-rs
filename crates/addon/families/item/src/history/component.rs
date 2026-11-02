//! The components of the older item formats: the newest types where the shape did not change,
//! and the old shapes below.

use bedrock_addon_core::document::CustomComponent;
use bedrock_addon_core::molang::{MolangFields, MolangVisitor};
use bedrock_addon_core::shared::{BlockDescriptor, EventTrigger, IntRange, ItemDescriptor};
use bedrock_addon_derive::Components;
use facet::Facet;
use indexmap::IndexMap;

use crate::component::{
    AllowOffHand, BlockPlacer, BundleInteraction, CanDestroyInCreative, Compostable, Cooldown,
    Damage, DamageAbsorption, Digger, DisplayName, Durability, DurabilitySensor, Dyeable,
    Enchantable, EntityPlacer, FireResistant, Food, Fuel, Glint, HandEquipped, HoverTextColor,
    Icon, InteractButton, KineticWeapon, LiquidClipped, MaxStackSize, PiercingWeapon, Projectile,
    Rarity, Record, Repairable, Shooter, ShouldDespawn, StackedByData, StorageItem,
    StorageWeightLimit, StorageWeightModifier, SwingDuration, SwingSounds, Tags, Throwable,
    UseAnimation, UseModifiers, Wearable,
};
use crate::component::{ValueComponent, visit_block_descriptor, visit_item_descriptor};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

/// One component of an older item format: a component of the newest format whose shape it
/// already had, an old shape, a custom component or a key the format does not bind.
#[derive(Clone, Debug, PartialEq, Components)]
#[component(from_only, custom = Custom, unknown = Unknown)]
#[allow(
    clippy::large_enum_variant,
    reason = "a component map holds few, and boxing every variant would cost each access"
)]
#[non_exhaustive]
#[allow(
    missing_docs,
    reason = "each variant holds the component type of its name"
)]
pub enum Component {
    AllowOffHand(AllowOffHand),
    BlockPlacer(BlockPlacer),
    BundleInteraction(BundleInteraction),
    CanDestroyInCreative(CanDestroyInCreative),
    Compostable(Compostable),
    Cooldown(Cooldown),
    Damage(Damage),
    DamageAbsorption(DamageAbsorption),
    Digger(Digger),
    DisplayName(DisplayName),
    Durability(Durability),
    DurabilitySensor(DurabilitySensor),
    Dyeable(Dyeable),
    Enchantable(Enchantable),
    EntityPlacer(EntityPlacer),
    FireResistant(FireResistant),
    Food(Food),
    Fuel(Fuel),
    Glint(Glint),
    HandEquipped(HandEquipped),
    HoverTextColor(HoverTextColor),
    Icon(Icon),
    InteractButton(InteractButton),
    KineticWeapon(KineticWeapon),
    LiquidClipped(LiquidClipped),
    MaxStackSize(MaxStackSize),
    PiercingWeapon(PiercingWeapon),
    Projectile(Projectile),
    Rarity(Rarity),
    Record(Record),
    Repairable(Repairable),
    Shooter(Shooter),
    ShouldDespawn(ShouldDespawn),
    StackedByData(StackedByData),
    StorageItem(StorageItem),
    StorageWeightLimit(StorageWeightLimit),
    StorageWeightModifier(StorageWeightModifier),
    SwingDuration(SwingDuration),
    SwingSounds(SwingSounds),
    Tags(Tags),
    Throwable(Throwable),
    UseAnimation(UseAnimation),
    UseModifiers(UseModifiers),
    Wearable(Wearable),
    // ---- old shapes ----
    Armor(Armor),
    Chargeable(Chargeable),
    OnUse(OnUse),
    OnUseOn(OnUseOn),
    RenderOffsets(RenderOffsets),
    Weapon(Weapon),
    UseDuration(UseDuration),
    LegacyFood(LegacyFoodComponent),
    LegacyDurability(LegacyDurability),
    LegacyRepairable(LegacyRepairable),
    LegacyDigger(LegacyDigger),
    LegacyIcon(LegacyIcon),
    LegacyStorageItem(LegacyStorageItem),
    CustomComponentList(CustomComponentList),
    CustomComponents(CustomComponents),
    Custom(CustomComponent),
    Unknown(Value),
}

/// `minecraft:armor` (up to 1.20.0): a marker vanilla's component binds no member of; the
/// 1.20.20 upgrade reads `protection` into `minecraft:wearable`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Armor {
    /// The armour points (JSON `protection`).
    #[facet(default)]
    pub protection: Option<i32>,
    /// The armour texture (JSON `texture_type`); dropped by the upgrade.
    #[facet(default)]
    pub texture_type: Option<String>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:chargeable` (up to 1.20.40).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Chargeable {
    /// The movement speed factor while charging (JSON `movement_modifier`).
    #[facet(default)]
    pub movement_modifier: Option<f32>,
    /// The trigger when charged (JSON `on_complete`).
    #[facet(default)]
    pub on_complete: Option<EventTrigger>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:on_use` (up to 1.20.30).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct OnUse {
    /// The trigger (JSON `on_use`).
    #[facet(default)]
    pub on_use: Option<EventTrigger>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:on_use_on` (up to 1.20.30).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct OnUseOn {
    /// The trigger (JSON `on_use_on`).
    #[facet(default)]
    pub on_use_on: Option<EventTrigger>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:render_offsets` (up to 1.20.30): hand → person → `{position, rotation, scale}`,
/// or a string vanilla reads and ignores; kept as written.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderOffsets {
    /// The value as written.
    pub value: Value,
}

/// `minecraft:weapon` (up to 1.20.30).
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct Weapon {
    /// The trigger on hurting an entity (JSON `on_hurt_entity`).
    #[facet(default)]
    pub on_hurt_entity: Option<EventTrigger>,
    /// The trigger on hitting an entity without hurting it (JSON `on_not_hurt_entity`).
    #[facet(default)]
    pub on_not_hurt_entity: Option<EventTrigger>,
    /// The trigger on hitting a block (JSON `on_hit_block`).
    #[facet(default)]
    pub on_hit_block: Option<EventTrigger>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:use_duration` (up to 1.20.40): seconds, `{"value": n}` or `n`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct UseDuration {
    /// The use duration in seconds (JSON `value`).
    #[facet(default)]
    pub value: f32,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl ValueComponent for UseDuration {
    type Scalar = f32;
    fn from_scalar(value: f32) -> Self {
        Self {
            value,
            extra: Extra::default(),
        }
    }
}

/// `minecraft:food` up to 1.20.20: an `on_consume` trigger, and (below 1.18.0) a named
/// `saturation_modifier`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LegacyFoodComponent {
    /// JSON `nutrition`.
    #[facet(default)]
    pub nutrition: Option<i32>,
    /// JSON `saturation_modifier`: a number, or a name below 1.18.0.
    #[facet(default)]
    pub saturation_modifier: Option<Value>,
    /// JSON `can_always_eat`.
    #[facet(default)]
    pub can_always_eat: Option<bool>,
    /// JSON `using_converts_to`.
    #[facet(default)]
    pub using_converts_to: Option<ItemDescriptor>,
    /// JSON `on_consume`.
    #[facet(default)]
    pub on_consume: Option<EventTrigger>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:durability` of 1.16.100: `max_damage` before the 1.18 rename.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LegacyDurability {
    /// JSON `max_damage` (below 1.18.0).
    #[facet(default)]
    pub max_damage: Option<i32>,
    /// JSON `max_durability`.
    #[facet(default)]
    pub max_durability: Option<i32>,
    /// JSON `damage_chance`.
    #[facet(default)]
    pub damage_chance: Option<IntRange>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:repairable` of 1.16.100: `repair_items` may be a list of item names (removed by
/// the 1.18 fixer), as written.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LegacyRepairable {
    /// JSON `repair_items`, as written.
    #[facet(default)]
    pub repair_items: Option<Value>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:digger` up to 1.20.40: `on_dig` triggers.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LegacyDigger {
    /// JSON `destroy_speeds`.
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub destroy_speeds: Vec<LegacyDestroySpeed>,
    /// JSON `use_efficiency`.
    #[facet(default)]
    pub use_efficiency: Option<bool>,
    /// JSON `on_dig`.
    #[facet(default)]
    pub on_dig: Option<EventTrigger>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// One entry of a [`LegacyDigger`].
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct LegacyDestroySpeed {
    /// JSON `block` (required).
    pub block: BlockDescriptor,
    /// JSON `speed` (required).
    pub speed: i32,
    /// JSON `on_dig`.
    #[facet(default)]
    pub on_dig: Option<EventTrigger>,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:icon` up to 1.20.50: one texture key, `{"texture": key}` or `key`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct LegacyIcon {
    /// JSON `texture`.
    #[facet(default)]
    pub texture: String,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

impl ValueComponent for LegacyIcon {
    type Scalar = String;
    fn from_scalar(texture: String) -> Self {
        Self {
            texture,
            extra: Extra::default(),
        }
    }
}

/// `minecraft:storage_item` of 1.21.30–1.21.50: the weight members before the 1.21.60 split.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct LegacyStorageItem {
    /// JSON `max_slots`.
    #[facet(default = 64)]
    pub max_slots: i32,
    /// JSON `allow_nested_storage_items`.
    #[facet(default = true)]
    pub allow_nested_storage_items: bool,
    /// JSON `banned_items`.
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub banned_items: Vec<ItemDescriptor>,
    /// JSON `allowed_items`.
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub allowed_items: Vec<ItemDescriptor>,
    /// JSON `max_weight_limit` (default 64).
    #[facet(default = 64)]
    pub max_weight_limit: i32,
    /// JSON `weight_in_storage_item` (default 4).
    #[facet(default = 4)]
    pub weight_in_storage_item: i32,
    /// Unknown members.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
}

/// `minecraft:custom_components` of 1.20.80–1.21.50: a list of names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomComponentList {
    /// The names, in order.
    pub names: Vec<String>,
}

/// `minecraft:custom_components` of 1.21.60–1.26.0: `[name]` or `[{name, data}]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomComponents {
    /// The entries, in order.
    pub components: Vec<CustomComponentEntry>,
}

/// One entry of [`CustomComponents`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CustomComponentEntry {
    /// The name, `ns:name`.
    pub name: String,
    /// The parameters (`{}` when written as a name).
    pub data: Value,
}

impl CustomComponents {
    /// The list as JSON: `[{"name", "data"}]`.
    pub(crate) fn to_value(&self) -> Value {
        let mut array = facet_value::VArray::new();
        for entry in &self.components {
            let mut object = facet_value::VObject::new();
            object.insert("name", Value::from(entry.name.as_str()));
            object.insert("data", entry.data.clone());
            array.push(Value::from(object));
        }
        Value::from(array)
    }
}

impl MolangFields for Component {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        match self {
            Self::BlockPlacer(c) => c.visit_molang(visitor),
            Self::Digger(c) => c.visit_molang(visitor),
            Self::EntityPlacer(c) => c.visit_molang(visitor),
            Self::Food(c) => c.visit_molang(visitor),
            Self::Repairable(c) => c.visit_molang(visitor),
            Self::Shooter(c) => c.visit_molang(visitor),
            Self::StorageItem(c) => c.visit_molang(visitor),
            Self::LegacyFood(c) => {
                if let Some(item) = &mut c.using_converts_to {
                    visitor.enter("using_converts_to".into());
                    visit_item_descriptor(item, visitor);
                    visitor.leave();
                }
            }
            Self::LegacyDigger(c) => {
                visitor.enter("destroy_speeds".into());
                for (i, speed) in c.destroy_speeds.iter_mut().enumerate() {
                    visitor.enter(i.into());
                    visitor.enter("block".into());
                    visit_block_descriptor(&mut speed.block, visitor);
                    visitor.leave();
                    visitor.leave();
                }
                visitor.leave();
            }
            Self::LegacyStorageItem(c) => {
                for (key, list) in [
                    ("banned_items", &mut c.banned_items),
                    ("allowed_items", &mut c.allowed_items),
                ] {
                    visitor.enter(key.into());
                    for (i, item) in list.iter_mut().enumerate() {
                        visitor.enter(i.into());
                        visit_item_descriptor(item, visitor);
                        visitor.leave();
                    }
                    visitor.leave();
                }
            }
            _ => {}
        }
    }
}

/// The components of an older format by key, in file order.
pub type Components = IndexMap<String, Component>;
