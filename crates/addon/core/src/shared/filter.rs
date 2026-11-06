//! `Filter`: filter groups, the conditions entity events, components, biome filters and trades
//! test.

use facet_value::{VArray, VObject};

use super::read::{self, Failure, ROOT, ShapeError, ShapeErrorKind, fail, wrong_type};
use super::{finding, unrecognized};
use crate::diagnostic::{Diagnostics, JsonPath, Severity, codes};
use crate::document::UnknownMembers;
use crate::json::Value;

/// A filter group: one test, a list (every element must pass), or a group of collection keys, as
/// the game's filter group.
///
/// JSON: `group := test | [group] | {key: group | [group], …}` with the keys `all_of`, `all`,
/// `AND`, `any_of`, `any`, `OR`, `none_of`, `NOT`. An object with a `test` member is a test;
/// any other object is a group. The default is the empty group, which vanilla treats as
/// unfiltered. Filters are not Molang.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
#[non_exhaustive]
#[doc(alias = "filter_group")]
pub enum Filter {
    /// One test.
    Test(FilterTest),
    /// `[group, …]`: every element must pass.
    List(Vec<Filter>),
    /// `{"all_of": …, …}`.
    Group(FilterGroup),
}

/// One filter test, as the game's filter data.
///
/// JSON: `{"test": string, "subject"?, "operator"?, "domain"?, "value"?}`. Test names are an open
/// set: an unknown name loads (with a warning from [`Filter::check`]) and evaluates false.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
pub struct FilterTest {
    /// The test's name (JSON `test`, required).
    pub test: String,
    /// Whom the test looks at (JSON `subject`); absent means `self`.
    pub subject: Option<FilterSubject>,
    /// The comparison as written (JSON `operator`): `=`, `==`, `equals`, `!=`, `<>`, `not`, `<`,
    /// `<=`, `>`, `>=`; absent means `equals`.
    pub operator: Option<String>,
    /// The test's domain (JSON `domain`): an equipment slot, a property name, ….
    pub domain: Option<FilterScalar>,
    /// The value compared against (JSON `value`).
    pub value: Option<FilterScalar>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// A group of filters by collection key, as the game's filter group map. The aliases (`all_of`,
/// `all`, `AND`; `any_of`, `any`, `OR`; `none_of`, `NOT`) are kept apart, so writing keeps the
/// key the document used. Every member absent is the empty group (unfiltered).
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
pub struct FilterGroup {
    /// Every filter must pass (JSON `all_of`).
    pub all_of: Option<Box<Filter>>,
    /// Every filter must pass (JSON `all`).
    pub all: Option<Box<Filter>>,
    /// Every filter must pass (JSON `AND`).
    pub and: Option<Box<Filter>>,
    /// At least one filter must pass (JSON `any_of`).
    pub any_of: Option<Box<Filter>>,
    /// At least one filter must pass (JSON `any`).
    pub any: Option<Box<Filter>>,
    /// At least one filter must pass (JSON `OR`).
    pub or: Option<Box<Filter>>,
    /// No filter may pass (JSON `none_of`).
    pub none_of: Option<Box<Filter>>,
    /// No filter may pass (JSON `NOT`).
    pub not: Option<Box<Filter>>,
    /// Unknown members.
    pub extra: UnknownMembers,
}

/// Whom a filter test looks at, as in the game. JSON: `self`, `other`, `player`,
/// `target`, `parent`, `baby`, `block`, `damager`, `holder`, `item`, or vanilla's number for
/// one of them (`0` for `self` … `9` for `item`), which vanilla's own re-serialised files use.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
#[non_exhaustive]
pub enum FilterSubject {
    /// The entity the filter belongs to (JSON `self`).
    #[default]
    Self_,
    /// The other entity of the interaction (JSON `other`).
    Other,
    /// The player of the interaction (JSON `player`).
    Player,
    /// The entity's target (JSON `target`).
    Target,
    /// The entity's parent (JSON `parent`).
    Parent,
    /// The entity's baby (JSON `baby`).
    Baby,
    /// The block of the interaction (JSON `block`).
    Block,
    /// The damaging entity (JSON `damager`).
    Damager,
    /// The entity holding the item (JSON `holder`).
    Holder,
    /// The item of the interaction (JSON `item`).
    Item,
}

/// A filter test's `domain` or `value`: `bool | int | float | string`, as vanilla's
/// `optional<variant<bool, int, float, string>>`; the JSON type is kept (`1` is an `Int`, `1.0`
/// a `Float`).
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
#[non_exhaustive]
pub enum FilterScalar {
    /// `true` or `false`.
    Bool(bool),
    /// An integer token.
    Int(i32),
    /// A real token.
    Float(f32),
    /// A string.
    String(String),
}

/// The group keys, in the order the fields are declared and written.
const GROUP_KEYS: [&str; 8] = [
    "all_of", "all", "AND", "any_of", "any", "OR", "none_of", "NOT",
];

/// The members of a test object.
const TEST_KEYS: [&str; 5] = ["test", "subject", "operator", "domain", "value"];

/// The JSON names of the subjects, in vanilla's numbering (`self` = 0 … `item` = 9).
const SUBJECTS: [(FilterSubject, &str); 10] = [
    (FilterSubject::Self_, "self"),
    (FilterSubject::Other, "other"),
    (FilterSubject::Player, "player"),
    (FilterSubject::Target, "target"),
    (FilterSubject::Parent, "parent"),
    (FilterSubject::Baby, "baby"),
    (FilterSubject::Block, "block"),
    (FilterSubject::Damager, "damager"),
    (FilterSubject::Holder, "holder"),
    (FilterSubject::Item, "item"),
];

/// The operators vanilla's re-serialised files write as numbers, with the spelling read for
/// each: `0` is equality and `1` inequality.
// The game's own entity files write them.
const NUMBERED_OPERATORS: [&str; 2] = ["==", "!="];

/// The filter tests vanilla registers, as far as they are recorded: the registered names, the
/// ones built from inline strings that are known (`is_biome`, `has_tag`, `is_owner`), 1.26.30's
/// `redstone_strength_at_position`, and the names vanilla's own packs use.
const VANILLA_TESTS: [&str; 107] = [
    "actor_has_item_with_enchantment_in_slot",
    "actor_health",
    "all_slots_empty",
    "any_slot_empty",
    "bool_memory",
    "bool_property",
    "clock_time",
    "distance_to_nearest_player",
    "enum_property",
    "float_property",
    "has_ability",
    "has_biome_tag",
    "has_component",
    "has_container_open",
    "has_damage",
    "has_damaged_equipment",
    "has_equipment",
    "has_equipment_tag",
    "has_memory",
    "has_mob_effect",
    "has_nametag",
    "has_property",
    "has_ranged_weapon",
    "has_same_equipment_in_slot_as",
    "has_silk_touch",
    "has_tag",
    "has_target",
    "has_trade_supply",
    "home_distance",
    "hourly_clock_time",
    "in_block",
    "in_caravan",
    "in_clouds",
    "in_contact_with_water",
    "in_lava",
    "in_nether",
    "in_overworld",
    "in_water",
    "in_water_or_rain",
    "inactivity_timer",
    "int_property",
    "is_altitude",
    "is_avoiding_mobs",
    "is_baby",
    "is_biome",
    "is_block",
    "is_bound_to_creaking_heart",
    "is_brightness",
    "is_climbing",
    "is_color",
    "is_controlling_passenger_family",
    "is_daytime",
    "is_difficulty",
    "is_family",
    "is_game_rule",
    "is_immobile",
    "is_in_same_vehicle",
    "is_in_village",
    "is_leashed",
    "is_leashed_to",
    "is_mark_variant",
    "is_missing_health",
    "is_moving",
    "is_navigating",
    "is_owner",
    "is_panicking",
    "is_persistent",
    "is_raider",
    "is_riding",
    "is_riding_self",
    "is_sitting",
    "is_skin_id",
    "is_sleeping",
    "is_sneak_held",
    "is_sneaking",
    "is_snow_covered",
    "is_sprinting",
    "is_tamed",
    "is_target",
    "is_temperature_type",
    "is_temperature_value",
    "is_underground",
    "is_underwater",
    "is_variant",
    "is_vehicle_family",
    "is_visible",
    "is_waterlogged",
    "is_weather",
    "light_level",
    "moon_intensity",
    "moon_phase",
    "on_fire",
    "on_ground",
    "on_hot_block",
    "on_ladder",
    "owner_distance",
    "random_chance",
    "redstone_strength_at_position",
    "rider_count",
    "surface_mob",
    "taking_fire_damage",
    "target_distance",
    "trusts",
    "was_last_hurt_by",
    "weather",
    "weather_at_position",
    "y_rotation",
];

impl Default for Filter {
    /// The empty group: unfiltered.
    fn default() -> Self {
        Self::Group(FilterGroup::default())
    }
}

impl Filter {
    /// The findings vanilla's reader logs for this filter, located under `at` (each at its own
    /// path below it): a test name vanilla does not register is an `UnresolvedReference` kept
    /// error with vanilla's text (the test then evaluates false); each member kept in a test's
    /// or a group's `extra` is an `UnrecognizedField` at `unknown` (cereal's `0x4`: the grade
    /// of an unknown member by the reading the caller reads under).
    pub fn check(&self, at: &JsonPath, unknown: Severity) -> Diagnostics {
        let mut out = Diagnostics::default();
        let mut path = at.clone();
        self.check_into(&mut path, unknown, &mut out);
        out
    }

    fn check_into(&self, path: &mut JsonPath, unknown: Severity, out: &mut Diagnostics) {
        match self {
            Self::Test(test) => {
                if VANILLA_TESTS.binary_search(&test.test.as_str()).is_err() {
                    // "Filter test not found -> %s":
                    // level 3, and the test evaluates false: a kept error.
                    out.push_kept_error(finding(
                        codes::FILTER_TEST_NOT_FOUND
                            .with([test.test.as_str()])
                            .into(),
                        Severity::Error,
                        path,
                    ));
                }
                unrecognized(&test.extra, path, unknown, out);
            }
            Self::List(list) => {
                for (index, filter) in list.iter().enumerate() {
                    path.push_index(index);
                    filter.check_into(path, unknown, out);
                    path.pop();
                }
            }
            Self::Group(group) => {
                for (key, member) in GROUP_KEYS.iter().zip(group.members()) {
                    if let Some(filter) = member {
                        path.push(key);
                        filter.check_into(path, unknown, out);
                        path.pop();
                    }
                }
                unrecognized(&group.extra, path, unknown, out);
            }
        }
    }

    /// Reads a filter group.
    pub(crate) fn read(value: &Value) -> Result<Self, Failure> {
        if let Some(array) = value.as_array() {
            return array
                .iter()
                .enumerate()
                .map(|(index, element)| Self::read(element).map_err(|f| f.index(index)))
                .collect::<Result<_, _>>()
                .map(Self::List);
        }
        let object = value
            .as_object()
            .ok_or_else(|| wrong_type(ROOT, "a filter object or array", value))?;
        if object.get("test").is_some() {
            return FilterTest::read(object).map(Self::Test);
        }
        let is_group = object.is_empty() || GROUP_KEYS.iter().any(|key| object.get(key).is_some());
        if !is_group && TEST_KEYS.iter().any(|key| object.get(key).is_some()) {
            // A test object without its required `test`.
            return Err(Failure::new(ShapeErrorKind::MissingMember {
                member: "test",
            }));
        }
        FilterGroup::read(object).map(Self::Group)
    }

    /// Writes a filter group.
    pub(crate) fn write(&self) -> Result<Value, Failure> {
        match self {
            Self::Test(test) => test.write(),
            Self::List(list) => {
                let mut array = VArray::new();
                for (index, filter) in list.iter().enumerate() {
                    array.push(filter.write().map_err(|f| f.index(index))?);
                }
                Ok(Value::from(array))
            }
            Self::Group(group) => group.write(),
        }
    }
}

impl FilterTest {
    /// A test with every optional member absent.
    pub fn new(test: impl Into<String>) -> Self {
        Self {
            test: test.into(),
            subject: None,
            operator: None,
            domain: None,
            value: None,
            extra: UnknownMembers::default(),
        }
    }

    /// Reads a test object; `null` members are absent.
    fn read(object: &VObject) -> Result<Self, Failure> {
        let member = |key| object.get(key).filter(|v| !v.is_null());
        let test = member("test")
            .ok_or_else(|| Failure::new(ShapeErrorKind::MissingMember { member: "test" }))?;
        let mut extra = UnknownMembers::default();
        for (key, value) in object {
            if !TEST_KEYS.contains(&key.as_str()) {
                extra.insert(key.as_str().to_owned(), value.clone());
            }
        }
        Ok(Self {
            test: read::string("test", test)?,
            subject: member("subject")
                .map(|v| FilterSubject::read(v).map_err(|f| f.key("subject")))
                .transpose()?,
            operator: member("operator").map(read_operator).transpose()?,
            domain: member("domain")
                .map(|v| FilterScalar::read("domain", v))
                .transpose()?,
            value: member("value")
                .map(|v| FilterScalar::read("value", v))
                .transpose()?,
            extra,
        })
    }

    fn write(&self) -> Result<Value, Failure> {
        let mut object = VObject::new();
        object.insert("test", Value::from(self.test.as_str()));
        if let Some(subject) = self.subject {
            object.insert("subject", Value::from(subject.name()));
        }
        if let Some(operator) = &self.operator {
            object.insert("operator", Value::from(operator.as_str()));
        }
        if let Some(domain) = &self.domain {
            object.insert("domain", domain.write().map_err(|f| f.key("domain"))?);
        }
        if let Some(value) = &self.value {
            object.insert("value", value.write().map_err(|f| f.key("value"))?);
        }
        for (key, value) in self.extra.iter() {
            object.insert(key, value.clone());
        }
        Ok(Value::from(object))
    }
}

/// Reads an operator: a string as written, or vanilla's number for equality or inequality.
fn read_operator(value: &Value) -> Result<String, Failure> {
    if let Some(text) = value.as_string() {
        return Ok(text.as_str().to_owned());
    }
    match value.as_number() {
        Some(n) if !n.is_float() => n
            .to_u64()
            .and_then(|i| usize::try_from(i).ok())
            .and_then(|i| NUMBERED_OPERATORS.get(i))
            .map(|op| (*op).to_owned())
            .ok_or_else(|| fail("operator", ShapeErrorKind::UnknownName)),
        _ => Err(wrong_type("operator", "a string", value)),
    }
}

impl FilterGroup {
    /// The members in `GROUP_KEYS` order.
    fn members(&self) -> [&Option<Box<Filter>>; 8] {
        [
            &self.all_of,
            &self.all,
            &self.and,
            &self.any_of,
            &self.any,
            &self.or,
            &self.none_of,
            &self.not,
        ]
    }

    /// Reads a group object; `null` members are absent.
    fn read(object: &VObject) -> Result<Self, Failure> {
        let member = |key: &str| -> Result<Option<Box<Filter>>, Failure> {
            match object.get(key) {
                Some(value) if !value.is_null() => Filter::read(value)
                    .map(|filter| Some(Box::new(filter)))
                    .map_err(|f| f.key(key)),
                _ => Ok(None),
            }
        };
        let mut extra = UnknownMembers::default();
        for (key, value) in object {
            if !GROUP_KEYS.contains(&key.as_str()) {
                extra.insert(key.as_str().to_owned(), value.clone());
            }
        }
        Ok(Self {
            all_of: member("all_of")?,
            all: member("all")?,
            and: member("AND")?,
            any_of: member("any_of")?,
            any: member("any")?,
            or: member("OR")?,
            none_of: member("none_of")?,
            not: member("NOT")?,
            extra,
        })
    }

    fn write(&self) -> Result<Value, Failure> {
        let mut object = VObject::new();
        for (key, member) in GROUP_KEYS.iter().zip(self.members()) {
            if let Some(filter) = member {
                object.insert(*key, filter.write().map_err(|f| f.key(key))?);
            }
        }
        for (key, value) in self.extra.iter() {
            object.insert(key, value.clone());
        }
        Ok(Value::from(object))
    }
}

impl FilterSubject {
    /// The JSON name (`"self"`, `"other"`, …).
    pub fn name(self) -> &'static str {
        SUBJECTS
            .iter()
            .find(|(subject, _)| *subject == self)
            .map_or("self", |(_, name)| name)
    }

    /// Reads a subject by name or by vanilla's number; the failure is the value's own.
    pub(crate) fn read(value: &Value) -> Result<Self, Failure> {
        let found = if let Some(name) = value.as_string() {
            SUBJECTS.iter().find(|(_, n)| *n == name.as_str())
        } else if let Some(n) = value.as_number().filter(|n| !n.is_float()) {
            n.to_u64()
                .and_then(|i| usize::try_from(i).ok())
                .and_then(|i| SUBJECTS.get(i))
        } else {
            return Err(wrong_type(ROOT, "a subject name", value));
        };
        found
            .map(|(subject, _)| *subject)
            .ok_or_else(|| Failure::new(ShapeErrorKind::UnknownName))
    }
}

impl FilterScalar {
    /// Reads a scalar member.
    fn read(member: &'static str, value: &Value) -> Result<Self, Failure> {
        if let Some(b) = value.as_bool() {
            Ok(Self::Bool(b))
        } else if let Some(s) = value.as_string() {
            Ok(Self::String(s.as_str().to_owned()))
        } else if value
            .as_number()
            .is_some_and(facet_value::VNumber::is_float)
        {
            read::float(member, value).map(Self::Float)
        } else if value.is_number() {
            read::int(member, value).map(Self::Int)
        } else {
            Err(wrong_type(member, "a boolean, a number or a string", value))
        }
    }

    fn write(&self) -> Result<Value, Failure> {
        Ok(match self {
            Self::Bool(b) => Value::from(*b),
            Self::Int(i) => Value::from(i64::from(*i)),
            Self::Float(x) => read::f32_value(*x)?,
            Self::String(s) => Value::from(s.as_str()),
        })
    }
}

impl TryFrom<Value> for Filter {
    type Error = ShapeError;

    fn try_from(value: Value) -> Result<Self, ShapeError> {
        Ok(Self::read(&value)?)
    }
}

impl TryFrom<&Filter> for Value {
    type Error = ShapeError;

    fn try_from(filter: &Filter) -> Result<Self, ShapeError> {
        Ok(filter.write()?)
    }
}

impl TryFrom<Value> for FilterTest {
    type Error = ShapeError;

    fn try_from(value: Value) -> Result<Self, ShapeError> {
        let object = value
            .as_object()
            .ok_or_else(|| wrong_type(ROOT, "an object", &value))?;
        Ok(Self::read(object)?)
    }
}

impl TryFrom<&FilterTest> for Value {
    type Error = ShapeError;

    fn try_from(test: &FilterTest) -> Result<Self, ShapeError> {
        Ok(test.write()?)
    }
}

impl TryFrom<Value> for FilterGroup {
    type Error = ShapeError;

    fn try_from(value: Value) -> Result<Self, ShapeError> {
        let object = value
            .as_object()
            .ok_or_else(|| wrong_type(ROOT, "an object", &value))?;
        Ok(Self::read(object)?)
    }
}

impl TryFrom<&FilterGroup> for Value {
    type Error = ShapeError;

    fn try_from(group: &FilterGroup) -> Result<Self, ShapeError> {
        Ok(group.write()?)
    }
}

impl TryFrom<Value> for FilterSubject {
    type Error = ShapeError;

    fn try_from(value: Value) -> Result<Self, ShapeError> {
        Ok(Self::read(&value)?)
    }
}

impl From<&FilterSubject> for Value {
    fn from(subject: &FilterSubject) -> Self {
        Value::from(subject.name())
    }
}

impl TryFrom<Value> for FilterScalar {
    type Error = ShapeError;

    fn try_from(value: Value) -> Result<Self, ShapeError> {
        Ok(Self::read(ROOT, &value)?)
    }
}

impl TryFrom<&FilterScalar> for Value {
    type Error = ShapeError;

    fn try_from(scalar: &FilterScalar) -> Result<Self, ShapeError> {
        Ok(scalar.write()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_vanilla_test_table_is_sorted_and_unique() {
        assert!(VANILLA_TESTS.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn every_subject_has_its_name() {
        for (index, (subject, name)) in SUBJECTS.iter().enumerate() {
            assert_eq!(subject.name(), *name);
            assert_eq!(*subject as usize, index);
        }
    }
}
