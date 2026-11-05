//! The member types the AI goals share: the goal base (`priority`, `control_flags`), entity
//! type lists, vectors, sound events and the named-value enums the game's schema defines as constants.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
use bedrock_addon_core::document::{Extra, UnknownMembers};
use bedrock_addon_core::json::{ScalarShape, StrictBool, StrictFloat, StrictInt, Value, ValueExt};
use bedrock_addon_core::molang::{JsonType, Molang};
use bedrock_addon_core::shared::Filter;
use facet::Facet;
use facet_value::{VArray, VObject};

use super::check::Check;
use super::sound::SOUND_EVENTS;

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
    /// A number with no `f32` value, or a value with no JSON form.
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

    /// The name is one of the level sound events the crate knows.
    pub fn is_known(&self) -> bool {
        SOUND_EVENTS.binary_search(&self.0.as_str()).is_ok()
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

/// The JSON form a [`Vec2`] or [`Vec3`] was read from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VecForm {
    /// `[x, y, z]`.
    #[default]
    Array,
    /// `{"x": …, "y": …, "z": …}`; a missing coordinate is 0.
    Object,
}

/// Reads the coordinates of a vector value: an array of exactly `N` numbers, or an object with
/// the members `names` (each defaulting to 0; other members are returned).
fn read_vector<const N: usize>(
    value: &Value,
    names: [&'static str; N],
) -> Result<([f32; N], VecForm, UnknownMembers), ValueError> {
    let number = |value: &Value| {
        let x = value.as_number().ok_or(ValueError::WrongType {
            expected: "a number",
            found: value.json_type(),
        })?;
        #[allow(clippy::cast_possible_truncation)] // the nearest f32 is the reading
        let x = x.to_f64_lossy() as f32;
        if x.is_finite() {
            Ok(x)
        } else {
            Err(ValueError::OutOfRange)
        }
    };
    let mut out = [0.0; N];
    if let Some(array) = value.as_array() {
        if array.len() != N {
            return Err(ValueError::ArrayLength {
                expected: N,
                len: array.len(),
            });
        }
        for (slot, element) in out.iter_mut().zip(array.iter()) {
            *slot = number(element)?;
        }
        Ok((out, VecForm::Array, UnknownMembers::default()))
    } else if let Some(object) = value.as_object() {
        let mut extra = UnknownMembers::default();
        for (key, element) in object {
            match names.iter().position(|n| *n == key.as_str()) {
                Some(i) => out[i] = number(element)?,
                None => {
                    extra.insert(key.as_str().to_owned(), element.clone());
                }
            }
        }
        Ok((out, VecForm::Object, extra))
    } else {
        Err(ValueError::WrongType {
            expected: "an array or an object",
            found: value.json_type(),
        })
    }
}

/// An `f32` as a real JSON number in its shortest decimal form.
fn float_value(x: f32) -> Result<Value, ValueError> {
    if !x.is_finite() {
        return Err(ValueError::OutOfRange);
    }
    let shortest: f64 = format!("{x:?}")
        .parse()
        .map_err(|_| ValueError::OutOfRange)?;
    Ok(Value::from(shortest))
}

/// Writes a vector in `form`, the object form followed by its unknown members.
fn write_vector<const N: usize>(
    coordinates: [f32; N],
    names: [&'static str; N],
    form: VecForm,
    extra: &UnknownMembers,
) -> Result<Value, ValueError> {
    match form {
        VecForm::Array => {
            let mut array = VArray::new();
            for x in coordinates {
                array.push(float_value(x)?);
            }
            Ok(Value::from(array))
        }
        VecForm::Object => {
            let mut object = VObject::new();
            for (name, x) in names.into_iter().zip(coordinates) {
                object.insert(name, float_value(x)?);
            }
            for (key, value) in extra.iter() {
                object.insert(key, value.clone());
            }
            Ok(Value::from(object))
        }
    }
}

/// A 3D vector member (`target_range`, `search_area`, …): `[x, y, z]` or `{x, y, z}`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
pub struct Vec3 {
    /// The x coordinate.
    pub x: f32,
    /// The y coordinate.
    pub y: f32,
    /// The z coordinate.
    pub z: f32,
    /// The form read; writing uses it.
    pub form: VecForm,
    /// Members of the object form the vector does not define, kept for write-back.
    pub extra: UnknownMembers,
}

impl Vec3 {
    /// The vector `[x, y, z]`, written as an array.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            form: VecForm::Array,
            extra: UnknownMembers::default(),
        }
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self::new(x, y, z)
    }
}

impl TryFrom<Value> for Vec3 {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, ValueError> {
        let ([x, y, z], form, extra) = read_vector(&value, ["x", "y", "z"])?;
        Ok(Self {
            x,
            y,
            z,
            form,
            extra,
        })
    }
}

impl TryFrom<&Vec3> for Value {
    type Error = ValueError;

    fn try_from(v: &Vec3) -> Result<Self, ValueError> {
        write_vector([v.x, v.y, v.z], ["x", "y", "z"], v.form, &v.extra)
    }
}

/// A 2D vector member (`transport_items.search_distance`): `[x, y]` or `{x, y}`.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
#[facet(opaque, proxy = Value)]
pub struct Vec2 {
    /// The x coordinate.
    pub x: f32,
    /// The y coordinate.
    pub y: f32,
    /// The form read; writing uses it.
    pub form: VecForm,
    /// Members of the object form the vector does not define, kept for write-back.
    pub extra: UnknownMembers,
}

impl Vec2 {
    /// The vector `[x, y]`, written as an array.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            form: VecForm::Array,
            extra: UnknownMembers::default(),
        }
    }
}

impl From<[f32; 2]> for Vec2 {
    fn from([x, y]: [f32; 2]) -> Self {
        Self::new(x, y)
    }
}

impl TryFrom<Value> for Vec2 {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, ValueError> {
        let ([x, y], form, extra) = read_vector(&value, ["x", "y"])?;
        Ok(Self { x, y, form, extra })
    }
}

impl TryFrom<&Vec2> for Value {
    type Error = ValueError;

    fn try_from(v: &Vec2) -> Result<Self, ValueError> {
        write_vector([v.x, v.y], ["x", "y"], v.form, &v.extra)
    }
}

/// `entity_types`: the mobs a goal reacts to, one descriptor or a list of them (vanilla
/// `EntityTypes`, read as a list and also as a single descriptor).
#[derive(Clone, Debug, PartialEq, Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(
    clippy::large_enum_variant,
    reason = "unknown members behind a `Box` are not kept for write-back (they are reported, nothing is written back)"
)]
pub enum EntityTypes {
    /// `[descriptor, …]`.
    List(Vec<MobDescriptor>),
    /// A single descriptor object.
    One(MobDescriptor),
}

impl Default for EntityTypes {
    /// The empty list.
    fn default() -> Self {
        Self::List(Vec::new())
    }
}

impl EntityTypes {
    /// The descriptors, in document order.
    pub fn as_slice(&self) -> &[MobDescriptor] {
        match self {
            Self::List(list) => list,
            Self::One(one) => std::slice::from_ref(one),
        }
    }

    /// The descriptors, in document order.
    pub fn iter(&self) -> std::slice::Iter<'_, MobDescriptor> {
        self.as_slice().iter()
    }
}

impl<'a> IntoIterator for &'a EntityTypes {
    type Item = &'a MobDescriptor;
    type IntoIter = std::slice::Iter<'a, MobDescriptor>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// One entry of `entity_types`: which mobs match (`filters`) and how the goal treats them.
/// This is the game's mob descriptor.
#[derive(Clone, Debug, Default, PartialEq, Facet)]
pub struct MobDescriptor {
    /// `check_if_outnumbered`: only flee when outnumbered; default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub check_if_outnumbered: Option<StrictBool>,
    /// `cooldown`: ticks before the goal may react again; default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub cooldown: Option<StrictInt<i32>>,
    /// `filters`: the mobs this entry matches.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub filters: Option<Filter>,
    /// `max_dist`: the largest distance at which a mob matches; default 16.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_dist: Option<StrictFloat<f32>>,
    /// `max_flee`: how far to flee from it; default 10.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_flee: Option<StrictFloat<f32>>,
    /// `max_height`: the largest height difference; −1 (the default) for any.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub max_height: Option<StrictFloat<f32>>,
    /// `must_see`: the mob must be visible.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub must_see: Option<StrictBool>,
    /// `must_see_forget_duration`: seconds a mob stays matched out of sight; default 3.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub must_see_forget_duration: Option<StrictFloat<f32>>,
    /// `priority`: the entry's priority among the list; default 0.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub priority: Option<StrictInt<i32>>,
    /// `reevaluate_description`: re-test the filters while the goal runs; default `false`.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub reevaluate_description: Option<StrictBool>,
    /// `sprint_speed_multiplier`: speed multiplier when sprinting; default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub sprint_speed_multiplier: Option<StrictFloat<f32>>,
    /// `walk_speed_multiplier`: speed multiplier when walking; default 1.
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub walk_speed_multiplier: Option<StrictFloat<f32>>,
    /// Members the descriptor does not define, kept for write-back.
    #[facet(skip, rename = "\u{0}extra")]
    pub extra: Extra,
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
pub(crate) fn check_entity_types(c: &mut Check<'_>, key: &str, types: Option<&EntityTypes>) {
    let Some(types) = types else { return };
    let mut at = c.path(key);
    let list = matches!(types, EntityTypes::List(_));
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
        let two = EntityTypes::List(vec![MobDescriptor::default(), MobDescriptor::default()]);
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
