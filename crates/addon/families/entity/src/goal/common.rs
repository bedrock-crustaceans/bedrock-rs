//! The member types the AI goals share: the goal base (`priority`, `control_flags`), entity
//! type lists, vectors, sound events and the named-value enums the game's schema defines as constants.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::json::{ScalarShape, Value, ValueExt};
use bedrock_addon_core::molang::{JsonType, Molang};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::Filter;
use facet::Facet;
use facet_value::VArray;

use super::check::GoalCheck;

/// Why a goal member's JSON value could not be read: the facet message of an `InvalidType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ValueError {
    /// A value of a JSON type the member does not take.
    #[error("expected {expected}, found a JSON {found}")]
    WrongType {
        /// What the member takes, in words.
        expected: &'static str,
        /// What the document has.
        found: JsonType,
    },
    /// A name that is none of the member's spellings.
    #[error("not one of the accepted names")]
    UnknownName,
    /// An array of the wrong length.
    #[error("expected an array of {expected} numbers, found {len} elements")]
    ArrayLength {
        /// The length the member takes.
        expected: usize,
        /// The array's length.
        len: usize,
    },
    /// An integer that does not fit the member's type: a [`ParticleColor`] channel outside
    /// `i32`, the one reader left that reports it.
    #[error("the number does not fit the member's type")]
    OutOfRange,
}

/// The JSON type of a scalar shape.
fn scalar_type(shape: &ScalarShape) -> JsonType {
    match shape {
        ScalarShape::Null => JsonType::Null,
        ScalarShape::String(_) => JsonType::String,
        ScalarShape::Bool(_) => JsonType::Bool,
        ScalarShape::UInt(_) | ScalarShape::Int(_) | ScalarShape::Number(_) => JsonType::Number,
    }
}

/// Declares a closed set of names read from a JSON string: an enum with a `name` method and a
/// facet proxy that rejects any other string (`InvalidType`) and any other JSON type.
macro_rules! names {
    (
        $(#[$meta:meta])*
        pub enum $name:ident ($expected:literal, $fold:ident) {
            $(
                $(#[$vmeta:meta])*
                $variant:ident = $text:literal
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Facet)]
        #[facet(proxy = ScalarShape)]
        #[repr(u8)]
        #[non_exhaustive]
        pub enum $name {
            $(
                $(#[$vmeta])*
                $variant,
            )*
        }

        impl $name {
            /// Every value, in vanilla's order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];

            /// The JSON spelling.
            pub fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $text,)*
                }
            }

            /// The value spelt `name`.
            pub fn from_name(name: &str) -> Option<Self> {
                Self::ALL.iter().copied().find(|value| names!(@eq $fold, value.name(), name))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.name())
            }
        }

        impl TryFrom<ScalarShape> for $name {
            type Error = ValueError;

            fn try_from(json: ScalarShape) -> Result<Self, ValueError> {
                match json {
                    ScalarShape::String(text) => Self::from_name(&text).ok_or(ValueError::UnknownName),
                    other => Err(ValueError::WrongType {
                        expected: $expected,
                        found: scalar_type(&other),
                    }),
                }
            }
        }

        // A proxied name: no member is walked, nothing in it is Molang.
        bedrock_addon_core::impl_schema_leaf!($name);
        bedrock_addon_core::impl_no_molang!($name);

        impl From<&$name> for ScalarShape {
            fn from(value: &$name) -> Self {
                ScalarShape::String(value.name().to_owned())
            }
        }
    };
    (@eq exact, $a:expr, $b:expr) => { $a == $b };
    (@eq fold, $a:expr, $b:expr) => { $a.eq_ignore_ascii_case($b) };
}

names! {
    /// One entry of a goal's `control_flags`: what the goal claims while it runs, so goals
    /// that claim the same flag do not run together. The game names these flags.
    ///
    /// [`component::ControlFlag`](crate::component::ControlFlag) has the same names but tells
    /// an unknown one differently in a strict read (`InvalidEnumValue` at the list, where this
    /// one tells `InvalidType` at the element), so the two stay apart;
    /// `tests/entity/descriptors.rs` pins both.
    pub enum ControlFlag ("a control flag name", exact) {
        /// `move`: the goal moves the mob.
        Move = "move",
        /// `look`: the goal turns the mob's head.
        Look = "look",
        /// `jump`: the goal makes the mob jump.
        Jump = "jump",
    }
}

names! {
    /// A point-of-interest kind (`hide.poi_type`, `move_to_poi.poi_type`): the game's
    /// point-of-interest kinds.
    pub enum PoiType ("a point-of-interest type name", exact) {
        /// `bed`.
        Bed = "bed",
        /// `meeting_area`: a bell.
        MeetingArea = "meeting_area",
        /// `jobsite`: a workstation.
        Jobsite = "jobsite",
    }
}

/// The liquid a `move_to_liquid` goal looks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LiquidMaterial {
    /// `Any`: water or lava.
    Any,
    /// `Water`.
    Water,
    /// `Lava`.
    Lava,
}

impl LiquidMaterial {
    /// Every value, in vanilla's order.
    pub const ALL: &'static [Self] = &[Self::Any, Self::Water, Self::Lava];

    /// The name vanilla's bind spells it with.
    pub fn name(self) -> &'static str {
        match self {
            Self::Any => "Any",
            Self::Water => "Water",
            Self::Lava => "Lava",
        }
    }

    /// The material spelt `name`, in any case.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|m| m.name().eq_ignore_ascii_case(name))
    }
}

impl std::fmt::Display for LiquidMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// `material_type` of `move_to_liquid`: a [`LiquidMaterial`] and the spelling read. Vanilla binds
/// the names capitalised and its own files write them in lower case, so any case is read; the
/// spelling read is written back.
// The game's schema names the constants `Water`, `Lava` and `Any`; the default is lowercase
// `"any"`, and either case is accepted.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(opaque, proxy = ScalarShape)]
pub struct MaterialType {
    /// The material.
    pub material: LiquidMaterial,
    /// The name as written.
    pub spelling: String,
}

impl MaterialType {
    /// `material`, spelt in lower case as vanilla's files write it.
    pub fn new(material: LiquidMaterial) -> Self {
        Self {
            material,
            spelling: material.name().to_ascii_lowercase(),
        }
    }
}

impl TryFrom<ScalarShape> for MaterialType {
    type Error = ValueError;

    fn try_from(json: ScalarShape) -> Result<Self, ValueError> {
        match json {
            ScalarShape::String(text) => Ok(Self {
                material: LiquidMaterial::from_name(&text).ok_or(ValueError::UnknownName)?,
                spelling: text,
            }),
            other => Err(ValueError::WrongType {
                expected: "a liquid name",
                found: scalar_type(&other),
            }),
        }
    }
}

impl From<&MaterialType> for ScalarShape {
    fn from(value: &MaterialType) -> Self {
        ScalarShape::String(value.spelling.clone())
    }
}

names! {
    /// Which game events `memory_sensor.game_event` responds to.
    // `event_type` is one of `All`, `GoatHorn` or `Instrument`; the JSON spelling follows the
    // enumerator names and is read in any case.
    pub enum GameEventType ("a game event type name", fold) {
        /// `All`: every game event.
        All = "All",
        /// `GoatHorn`: a goat horn being played.
        GoatHorn = "GoatHorn",
        /// `Instrument`: an instrument being played.
        Instrument = "Instrument",
    }
}

names! {
    /// How a goal picks among the blocks it found (`move_to_block.target_selection_method`).
    pub enum TargetSelectionMethod ("a selection method name", exact) {
        /// `nearest`.
        Nearest = "nearest",
        /// `random`.
        Random = "random",
    }
}

names! {
    /// Where `transport_items` puts an item (`place_strategy`).
    pub enum PlaceStrategy ("a place strategy name", exact) {
        /// `any`: any container with room.
        Any = "any",
        /// `with_matching`: only a container that already holds the item.
        WithMatching = "with_matching",
        /// `with_matching_or_empty`: a container that holds the item, or an empty one.
        WithMatchingOrEmpty = "with_matching_or_empty",
    }
}

names! {
    /// Which container `transport_items` visits next (`search_strategy`).
    pub enum SearchStrategy ("a search strategy name", exact) {
        /// `nearest`.
        Nearest = "nearest",
        /// `random`.
        Random = "random",
    }
}

names! {
    /// What `ranged_attack` does while its target is in range (`in_range_movement_mode`).
    pub enum InRangeMovementMode ("a movement mode name", exact) {
        /// `hold_position`: stand still and shoot.
        HoldPosition = "hold_position",
        /// `follow_target`: keep moving towards the target.
        FollowTarget = "follow_target",
    }
}

names! {
    /// How `summon_entity` lays out what it summons (`shape`).
    pub enum SummonShape ("a summon shape name", exact) {
        /// `circle`.
        Circle = "circle",
        /// `line`.
        Line = "line",
    }
}

names! {
    /// Where `summon_entity` summons (`target`).
    pub enum SummonTarget ("a summon target name", exact) {
        /// `self`: around the summoner.
        Self_ = "self",
        /// `target`: around the summoner's target.
        Target = "target",
    }
}

names! {
    /// A damage cause (`play_dead.damage_sources`): the game's damage causes.
    pub enum DamageCause ("a damage cause name", exact) {
        /// `none`.
        None = "none",
        /// `override`.
        Override = "override",
        /// `contact`.
        Contact = "contact",
        /// `entity_attack`.
        EntityAttack = "entity_attack",
        /// `projectile`.
        Projectile = "projectile",
        /// `suffocation`.
        Suffocation = "suffocation",
        /// `fall`.
        Fall = "fall",
        /// `fire`.
        Fire = "fire",
        /// `fire_tick`.
        FireTick = "fire_tick",
        /// `lava`.
        Lava = "lava",
        /// `drowning`.
        Drowning = "drowning",
        /// `block_explosion`.
        BlockExplosion = "block_explosion",
        /// `entity_explosion`.
        EntityExplosion = "entity_explosion",
        /// `void`.
        Void = "void",
        /// `self_destruct`.
        SelfDestruct = "self_destruct",
        /// `magic`.
        Magic = "magic",
        /// `wither`.
        Wither = "wither",
        /// `starve`.
        Starve = "starve",
        /// `anvil`.
        Anvil = "anvil",
        /// `thorns`.
        Thorns = "thorns",
        /// `falling_block`.
        FallingBlock = "falling_block",
        /// `piston`.
        Piston = "piston",
        /// `fly_into_wall`.
        FlyIntoWall = "fly_into_wall",
        /// `magma`.
        Magma = "magma",
        /// `fireworks`.
        Fireworks = "fireworks",
        /// `lightning`.
        Lightning = "lightning",
        /// `charging`.
        Charging = "charging",
        /// `temperature`.
        Temperature = "temperature",
        /// `freezing`.
        Freezing = "freezing",
        /// `stalactite`.
        Stalactite = "stalactite",
        /// `stalagmite`.
        Stalagmite = "stalagmite",
        /// `ram_attack`.
        RamAttack = "ram_attack",
        /// `sonic_boom`.
        SonicBoom = "sonic_boom",
        /// `campfire`.
        Campfire = "campfire",
        /// `soul_campfire`.
        SoulCampfire = "soul_campfire",
        /// `mace_smash`.
        MaceSmash = "mace_smash",
        /// `all`: every cause.
        All = "all",
    }
}

/// A level sound event name (`*_sound`, `sound_event` members). Vanilla's typed goals take one
/// of its `LevelSoundEvent` names (`undefined` for none); a name outside the list the crate
/// knows is kept and reported as an `UnknownSound` warning, since vanilla adds names over time.
/// Any JSON type other than a string is a typed parse error.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Facet)]
#[facet(proxy = ScalarShape)]
pub struct SoundEvent(String);

impl SoundEvent {
    /// The event spelt `name`.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The event's name.
    pub fn name(&self) -> &str {
        &self.0
    }

    /// The name is one of the game's level sound events, as written (the vanilla data
    /// crate's table).
    pub fn is_known(&self) -> bool {
        bedrock_addon_core::vanilla::sound_events::is_level_sound_event(&self.0)
    }
}

impl From<&str> for SoundEvent {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl From<String> for SoundEvent {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl AsRef<str> for SoundEvent {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<ScalarShape> for SoundEvent {
    type Error = ValueError;

    fn try_from(json: ScalarShape) -> Result<Self, ValueError> {
        match json {
            ScalarShape::String(text) => Ok(Self(text)),
            other => Err(ValueError::WrongType {
                expected: "a sound event name",
                found: scalar_type(&other),
            }),
        }
    }
}

impl From<&SoundEvent> for ScalarShape {
    fn from(value: &SoundEvent) -> Self {
        ScalarShape::String(value.0.clone())
    }
}

/// The goal vectors are the shared shapes: `[x, y, z]` or `{x, y, z}`, kept in the form read.
pub use bedrock_addon_shared_shapes::vec::{Vec2, Vec3, VecForm};

/// `entity_types` of a goal: one [`MobDescriptor`] or a list of them.
pub type EntityTypes = crate::EntityTypesOf<MobDescriptor>;

/// One entry of `entity_types`: which mobs match (`filters`) and how the goal treats them.
/// This is the game's mob descriptor.
///
/// [`component::MobDescriptor`](crate::component::MobDescriptor) is the same descriptor read
/// differently, kept apart until a capture settles it: here every member is an `Option` and
/// only what was read is written back, and `must_see_forget_duration` is an `f32` (2.5 reads),
/// as the game's schema types the goal member; the component's is an `i32` with its defaults
/// written back. No capture has the member inside a descriptor (`zombie_nautilus.json` has it
/// as a `nearest_attackable_target` member, `17`). `tests/entity/descriptors.rs` pins both.
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct MobDescriptor {
    /// `check_if_outnumbered`: only flee when outnumbered; default `false`.
    pub check_if_outnumbered: Option<bool>,
    /// `cooldown`: ticks before the goal may react again; default 0.
    pub cooldown: Option<i32>,
    /// `filters`: the mobs this entry matches.
    pub filters: Option<Filter>,
    /// `max_dist`: the largest distance at which a mob matches; default 16.
    pub max_dist: Option<f32>,
    /// `max_flee`: how far to flee from it; default 10.
    pub max_flee: Option<f32>,
    /// `max_height`: the largest height difference; −1 (the default) for any.
    pub max_height: Option<f32>,
    /// `must_see`: the mob must be visible.
    pub must_see: Option<bool>,
    /// `must_see_forget_duration`: seconds a mob stays matched out of sight; default 3.
    pub must_see_forget_duration: Option<f32>,
    /// `priority`: the entry's priority among the list; default 0.
    pub priority: Option<i32>,
    /// `reevaluate_description`: re-test the filters while the goal runs; default `false`.
    pub reevaluate_description: Option<bool>,
    /// `sprint_speed_multiplier`: speed multiplier when sprinting; default 1.
    pub sprint_speed_multiplier: Option<f32>,
    /// `walk_speed_multiplier`: speed multiplier when walking; default 1.
    pub walk_speed_multiplier: Option<f32>,
}

impl MobDescriptor {
    /// A descriptor matching the mobs `filters` passes, every other member at its default.
    pub fn new(filters: Filter) -> Self {
        Self {
            filters: Some(filters),
            ..Self::default()
        }
    }

    /// The findings of the descriptor's filter, located under `at`.
    pub(crate) fn check(&self, at: &JsonPath, out: &mut Diagnostics) {
        if let Some(filters) = &self.filters {
            let path = at.child("filters");
            out.extend(filters.check(&path));
        }
    }
}

/// Checks every descriptor of an `entity_types` member.
pub(crate) fn check_entity_types(c: &mut GoalCheck<'_>, key: &str, types: Option<&EntityTypes>) {
    let Some(types) = types else { return };
    let mut at = c.path(key);
    let list = matches!(types, EntityTypes::Many(_));
    let mut found = Diagnostics::default();
    for (i, descriptor) in types.as_slice().iter().enumerate() {
        if list {
            at.push_index(i);
        }
        descriptor.check(&at, &mut found);
        if list {
            at.pop();
        }
    }
    c.extend(found);
}

/// A colour written as text (`"#RRGGBB"`) or as four channels `[r, g, b, a]` (0–255):
/// `summon_entity` `particle_color`.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
pub enum ParticleColor {
    /// A colour string.
    Text(String),
    /// `[r, g, b, a]`.
    Channels([i32; 4]),
}

impl TryFrom<Value> for ParticleColor {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, ValueError> {
        if let Some(text) = value.as_string() {
            return Ok(Self::Text(text.as_str().to_owned()));
        }
        let Some(array) = value.as_array() else {
            return Err(ValueError::WrongType {
                expected: "a string or an array",
                found: value.json_type(),
            });
        };
        if array.len() != 4 {
            return Err(ValueError::ArrayLength {
                expected: 4,
                len: array.len(),
            });
        }
        let mut channels = [0; 4];
        for (slot, element) in channels.iter_mut().zip(array.iter()) {
            *slot = match element.as_number() {
                Some(number) if !number.is_float() => number
                    .to_i64()
                    .and_then(|i| i32::try_from(i).ok())
                    .ok_or(ValueError::OutOfRange)?,
                _ => {
                    return Err(ValueError::WrongType {
                        expected: "an integer",
                        found: element.json_type(),
                    });
                }
            };
        }
        Ok(Self::Channels(channels))
    }
}

impl TryFrom<&ParticleColor> for Value {
    type Error = ValueError;

    fn try_from(color: &ParticleColor) -> Result<Self, ValueError> {
        Ok(match color {
            ParticleColor::Text(text) => Value::from(text.as_str()),
            ParticleColor::Channels(channels) => {
                let mut array = VArray::new();
                for channel in channels {
                    array.push(Value::from(i64::from(*channel)));
                }
                Value::from(array)
            }
        })
    }
}

/// The legacy `condition`s of a goal's triggers as Molang values, read from each trigger's
/// `extra` the first time the goal's Molang members are walked, so the load's two Molang passes
/// see one value each. A cache: it is never written, and any two compare equal.
#[doc(hidden)]
#[derive(Clone, Debug, Default, Facet)]
#[facet(opaque)]
pub struct LegacyConditions {
    /// `(trigger path, condition)`; `None` until the first walk.
    pub(crate) entries: Option<Vec<(String, Molang)>>,
}

impl PartialEq for LegacyConditions {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every value of a closed set writes the name `name` gives.
    fn displays_its_name<T: Copy + std::fmt::Display>(all: &[T], name: fn(T) -> &'static str) {
        assert!(!all.is_empty());
        for value in all {
            assert_eq!(value.to_string(), name(*value));
        }
    }

    #[test]
    fn name_enums_display_their_json_spelling() {
        displays_its_name(ControlFlag::ALL, ControlFlag::name);
        displays_its_name(PoiType::ALL, PoiType::name);
        displays_its_name(GameEventType::ALL, GameEventType::name);
        displays_its_name(TargetSelectionMethod::ALL, TargetSelectionMethod::name);
        displays_its_name(PlaceStrategy::ALL, PlaceStrategy::name);
        displays_its_name(SearchStrategy::ALL, SearchStrategy::name);
        displays_its_name(InRangeMovementMode::ALL, InRangeMovementMode::name);
        displays_its_name(SummonShape::ALL, SummonShape::name);
        displays_its_name(SummonTarget::ALL, SummonTarget::name);
        displays_its_name(DamageCause::ALL, DamageCause::name);
        displays_its_name(LiquidMaterial::ALL, LiquidMaterial::name);
        assert_eq!(ControlFlag::Move.to_string(), "move");
        assert_eq!(LiquidMaterial::Lava.to_string(), "Lava");
    }

    #[test]
    fn sound_event_converts_from_and_to_text() {
        let from_str = SoundEvent::from("mob.cow.say");
        assert_eq!(from_str, SoundEvent::from("mob.cow.say".to_owned()));
        assert_eq!(from_str.as_ref(), "mob.cow.say");
        assert_eq!(from_str, SoundEvent::new("mob.cow.say"));
    }

    #[test]
    fn vectors_convert_from_arrays() {
        assert_eq!(Vec3::from([1.0, 2.0, 3.0]), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(Vec2::from([1.0, 2.0]), Vec2::new(1.0, 2.0));
    }

    #[test]
    fn entity_types_iterate_in_document_order() {
        let two = EntityTypes::Many(vec![MobDescriptor::default(), MobDescriptor::default()]);
        assert_eq!(two.iter().count(), 2);
        assert_eq!((&two).into_iter().count(), 2);
        let one = EntityTypes::One(MobDescriptor::default());
        let mut seen = 0;
        for descriptor in &one {
            assert_eq!(descriptor, &MobDescriptor::default());
            seen += 1;
        }
        assert_eq!(seen, 1);
    }
}

// Proxied or data-carrying values: no member is walked, nothing in them is Molang.
bedrock_addon_core::impl_schema_leaf!(
    LiquidMaterial,
    MaterialType,
    SoundEvent,
    EntityTypes,
    ParticleColor
);
bedrock_addon_core::impl_no_molang!(
    LiquidMaterial,
    MaterialType,
    SoundEvent,
    EntityTypes,
    ParticleColor
);
