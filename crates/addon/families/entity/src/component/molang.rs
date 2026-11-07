//! The components whose members hold Molang: `experience_reward`, `anger_level` and
//! `rideable` (its seats' `rotate_rider_by`, typed from 1.26.60).
//!
//! Their expressions take the pack's Molang version (from `min_engine_version`), not the
//! document's. A document at or above the component's typed version reads a number, a string
//! or the `{"expression", "version"}` object; the lenient conversion of an older document reads
//! a number, a bool or a string, and an object there is `InvalidType` (a warning, the member
//! takes its default).
//!
//! `experience_reward` is typed from 1.21.110 (redeclared in 1.26.0) and `anger_level` from
//! 1.26.50.

#[cfg(feature = "molang")]
use bedrock_addon_core::diagnostic::Location;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{Molang, MolangFieldSpec, ReaderKind};
#[cfg(feature = "molang")]
use bedrock_addon_core::molang::{MolangContext, MolangOutcome, MolangSource, Side};
use bedrock_addon_core::schema::{self, Schema};
use bedrock_addon_core::shared::Filter;
use bedrock_addon_core::versioning::ConvertMode;
use facet::Facet;
use facet_value::VObject;

use super::parse::{ComponentCheck, Typed, drop_deprecated, read_prepared};
use super::value::{one_element, tell_partial_vector};
use crate::goal::Vec3;

/// The spec of every Molang member here: the usual cereal expression node (strict versioned,
/// the pack's version, the default query set).
const EXPRESSION: MolangFieldSpec = MolangFieldSpec::DEFAULT;

/// The spec of a member in the lenient conversion: the schema-validated reader, and a rejected
/// expression is a kept error.
#[cfg(feature = "molang")]
// The game's parser logs a level-3 line for a rejected expression and the entity loads
// anyway: a kept error. The legacy expression nodes (`experience_reward`, `anger_level`)
// store the text and drop the parse result.
const LENIENT: MolangFieldSpec = MolangFieldSpec::KEPT.reader(ReaderKind::SchemaValidated);

/// Whether the lenient conversion keeps `value` at `at`. A value the schema-validated reader
/// rejects (the object form) is `InvalidType`, and with the `molang` feature an expression that
/// does not compile at the pack's version is `InvalidMolang` (a kept error), and the member takes
/// its default (`false`). A strict read keeps every value; pass 2 checks it.
fn lenient_value(value: &Molang, at: &JsonPath, cx: &mut ComponentCheck<'_>) -> bool {
    if !matches!(cx.mode(), ConvertMode::Lenient) {
        return true;
    }
    if value.check(ReaderKind::SchemaValidated).is_err() {
        cx.push(
            Code::from(&core_codes::INVALID_TYPE),
            // The lenient reader's refusal of the object form: the game writes no content-log
            // line for this, so the crate reports it as an information.
            Severity::Info,
            at,
        );
        return false;
    }
    #[cfg(feature = "molang")]
    if let Some(source) = value.source() {
        return compiles(source, at, cx);
    }
    true
}

/// Compiles `source` at the pack's Molang version as the lenient conversion reads it; on a
/// rejection reports the compiler's findings (errors as warnings) and returns `false`. Without
/// the load's compile check (`LoadContext::molang`) nothing is compiled here.
#[cfg(feature = "molang")]
fn compiles(source: &MolangSource, at: &JsonPath, cx: &mut ComponentCheck<'_>) -> bool {
    let load = cx.load();
    let Some(check) = load.molang.clone() else {
        return true;
    };
    let mut source = source.clone();
    source.set_context_version(load.molang_version);
    let molang = MolangContext {
        pack: load.molang_version,
        pack_before_controller_fix: false,
        document: load.molang_version,
        injected: None,
        experiments: load.molang_experiments,
    };
    let mut found = Diagnostics::default();
    let outcome = check.check_source(
        &source,
        &LENIENT,
        Side::Server,
        &molang,
        &Location::new(None, None, at.clone(), None),
        &mut found,
    );
    if matches!(outcome, MolangOutcome::Failed(_)) {
        // The member takes its default, so pass 2 never sees it: these are its only findings.
        cx.extend(found);
        return false;
    }
    true
}

/// `minecraft:experience_reward`: the experience the entity drops when bred and when it dies,
/// each a Molang expression or a number.
#[doc(alias = "minecraft:experience_reward")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct ExperienceReward {
    /// `on_bred`: the experience when bred; absent drops none.
    #[facet(schema::molang = EXPRESSION)]
    pub on_bred: Option<Molang>,
    /// `on_death`: the experience on death; absent drops none.
    #[facet(schema::molang = EXPRESSION)]
    pub on_death: Option<Molang>,
    /// `on_bred_array` (not in the published schema): further `on_bred` expressions.
    #[facet(schema::molang = EXPRESSION)]
    pub on_bred_array: Option<Vec<Molang>>,
    /// `on_death_array` (not in the published schema): further `on_death` expressions.
    #[facet(schema::molang = EXPRESSION)]
    pub on_death_array: Option<Vec<Molang>>,
}

impl ExperienceReward {
    /// The 1.26.0 upgrade (vanilla's typed-to-typed hook): where a scalar is empty (absent or
    /// the empty expression) and its array holds exactly one element, the element moves into the
    /// scalar and the array goes; otherwise both stay as they are.
    pub fn fold_single_element_arrays(&mut self) {
        fold(&mut self.on_bred, &mut self.on_bred_array);
        fold(&mut self.on_death, &mut self.on_death_array);
    }

    /// Reads the component as the 1.21.110 – 1.21.130 versions define it (the parse their
    /// tables use): there `on_bred_array` and `on_death_array` are deprecated names, so a
    /// document that writes one gets `DeprecatedMember` and the member is dropped; the lenient
    /// conversion of an older document keeps them.
    // The game binds these names in 1.21.110 and deprecates them at once; 1.26.0 binds them
    // again.
    #[doc(hidden)]
    pub fn read_v1_21_110(
        slice: JsonSlice<'_>,
        mode: ConvertMode,
        cx: &LoadContext,
        d: &mut Diagnostics,
    ) -> Option<Self> {
        read_prepared::<Self>(slice, mode, cx, d, |json, cx| match cx.mode() {
            ConvertMode::Strict => drop_deprecated(json, &["on_bred_array", "on_death_array"], cx),
            ConvertMode::Lenient => false,
        })
    }
}

/// [`ExperienceReward::fold_single_element_arrays`] for one scalar and its array.
fn fold(scalar: &mut Option<Molang>, array: &mut Option<Vec<Molang>>) {
    let empty = match scalar {
        None => true,
        Some(value) => value.source().is_some_and(|src| src.as_str().is_empty()),
    };
    if empty && array.as_ref().is_some_and(|list| list.len() == 1) {
        *scalar = array.take().and_then(|mut list| list.pop());
    }
}

impl Typed for ExperienceReward {
    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        for (key, scalar) in [
            ("on_bred", &mut self.on_bred),
            ("on_death", &mut self.on_death),
        ] {
            if let Some(value) = scalar
                && !lenient_value(value, &JsonPath::new().child(key), cx)
            {
                *scalar = None;
            }
        }
        for (key, array) in [
            ("on_bred_array", &mut self.on_bred_array),
            ("on_death_array", &mut self.on_death_array),
        ] {
            if let Some(list) = array {
                let at = JsonPath::new().child(key);
                let mut i = 0;
                let mut kept = Vec::with_capacity(list.len());
                list.retain(|value| {
                    let keep = lenient_value(value, &at.index(i), cx);
                    if keep {
                        kept.push(i);
                    }
                    i += 1;
                    keep
                });
                cx.left_out(&at, &kept);
            }
        }
    }
}

/// `minecraft:anger_level`: the entity's anger towards nuisances (the warden).
#[doc(alias = "minecraft:anger_level")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct AngerLevel {
    /// `anger_decrement_interval`: seconds between anger decreases; default 1.
    #[facet(default = 1.0_f32)]
    pub anger_decrement_interval: f32,
    /// `angry_boost`: anger added to a nuisance already above the threshold; default 20.
    #[facet(default = 20_i32)]
    pub angry_boost: i32,
    /// `angry_threshold`: the anger at which the entity is angry; default 80.
    #[facet(default = 80_i32)]
    pub angry_threshold: i32,
    /// `default_annoyingness`: anger added by a nuisance; at least 0, default 0.
    #[facet(default = 0_i32)]
    pub default_annoyingness: i32,
    /// `default_projectile_annoyingness`: anger added by a projectile; at least 0, default 0.
    #[facet(default = 0_i32)]
    pub default_projectile_annoyingness: i32,
    /// `max_anger`: the highest anger; default 100.
    #[facet(default = 100_i32)]
    pub max_anger: i32,
    /// `nuisance_filter`: which entities are nuisances; absent means all.
    pub nuisance_filter: Option<Filter>,
    /// `on_increase_sounds`: sounds played when the anger rises, each with a condition.
    #[facet(default)]
    pub on_increase_sounds: Vec<AngerSound>,
    /// `remove_targets_below_angry_threshold`: targets below the threshold are dropped; default
    /// `true`.
    #[facet(default = true)]
    pub remove_targets_below_angry_threshold: bool,
}

/// One sound of [`AngerLevel::on_increase_sounds`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct AngerSound {
    /// `condition`: when the sound plays, a Molang expression; required.
    #[facet(schema::molang = EXPRESSION)]
    pub condition: Molang,
    /// `sound`: the sound event; required.
    pub sound: String,
}

impl Typed for AngerLevel {
    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        // The game's schema minimums.
        cx.at_least(
            &JsonPath::new().child("default_annoyingness"),
            &mut self.default_annoyingness,
            0,
            0,
        );
        cx.at_least(
            &JsonPath::new().child("default_projectile_annoyingness"),
            &mut self.default_projectile_annoyingness,
            0,
            0,
        );
        if let Some(filter) = &self.nuisance_filter {
            cx.extend(filter.check(&JsonPath::new().child("nuisance_filter"), cx.unknown()));
        }
        // A sound whose condition the lenient reader rejects has no condition to fall back to:
        // the entry is dropped.
        let sounds = JsonPath::new().child("on_increase_sounds");
        let mut i = 0;
        let mut kept = Vec::with_capacity(self.on_increase_sounds.len());
        self.on_increase_sounds.retain(|sound| {
            let keep = lenient_value(&sound.condition, &sounds.index(i).child("condition"), cx);
            if keep {
                kept.push(i);
            }
            i += 1;
            keep
        });
        cx.left_out(&sounds, &kept);
    }
}

/// `minecraft:rideable`: the entity can be ridden, with its seats.
// A seat's `position` is a `Vec3`, `rotate_rider_by` an expression node built from 0.0,
// `lock_rider_rotation` defaults to 181, the two camera members are optional with ranges
// `1 ..= 64` and `1 ..= 32`, and `dismount_mode` is the `DismountMode` enum.
// `rotate_rider_by` is Molang.
#[doc(alias = "minecraft:rideable")]
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Rideable {
    /// `seat_count`: how many entities ride it at once; default 1.
    #[facet(default = 1_i32)]
    pub seat_count: i32,
    /// `controlling_seat`: the seat of the driver; default 0.
    #[facet(default = 0_i32)]
    pub controlling_seat: i32,
    /// `crouching_skip_interact`: a crouching entity cannot interact with it; default `true`.
    #[facet(default = true)]
    pub crouching_skip_interact: bool,
    /// `pull_in_entities`: it pulls entities of `family_types` into free seats; default
    /// `false`.
    #[facet(default = false)]
    pub pull_in_entities: bool,
    /// `rider_can_interact`: the rider can pick it when looking at it; default `false`.
    #[facet(default = false)]
    pub rider_can_interact: bool,
    /// `passenger_max_width`: the widest a rider may be, 0 for any; default 0.
    #[facet(default = 0.0_f32)]
    pub passenger_max_width: f32,
    /// `family_types`: the families that may ride it.
    #[facet(default)]
    pub family_types: Vec<String>,
    /// `interact_text`: the touch-screen text of the interaction; default none.
    #[facet(default = String::new())]
    pub interact_text: String,
    /// `dismount_mode`: where riders are placed when they dismount; default `default`.
    #[facet(default = DismountMode::Default)]
    pub dismount_mode: DismountMode,
    /// `on_rider_enter_event`: the event when an entity starts riding; default none.
    #[facet(default = String::new())]
    pub on_rider_enter_event: String,
    /// `on_rider_exit_event`: the event when an entity stops riding; default none.
    #[facet(default = String::new())]
    pub on_rider_exit_event: String,
    /// `seats`: the seats.
    #[facet(default)]
    pub seats: Vec<Seat>,
}

/// Where a [`Rideable`]'s riders go when they dismount.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Facet, Schema)]
#[repr(u8)]
#[non_exhaustive]
pub enum DismountMode {
    /// A free ground position around the entity, else the centre of its box (JSON `default`).
    #[default]
    #[facet(rename = "default")]
    Default,
    /// The centre of the top of its box (JSON `on_top_center`).
    #[facet(rename = "on_top_center")]
    OnTopCenter,
    /// The top of the neighbouring block in the rider's look direction (JSON `look_direction`).
    #[facet(rename = "look_direction")]
    LookDirection,
}

/// One seat of a [`Rideable`].
#[derive(Clone, Debug, PartialEq, Facet, Schema)]
#[facet(schema::unknown = "keep")]
pub struct Seat {
    /// `position`: the seat relative to the entity; default 0, 0, 0.
    #[facet(default = Vec3::new(0.0, 0.0, 0.0))]
    pub position: Vec3,
    /// `min_rider_count`: the riders needed before this seat is used; default 0.
    #[facet(default = 0_i32)]
    pub min_rider_count: i32,
    /// `max_rider_count`: the most riders for this seat to be valid; default 0.
    #[facet(default = 0_i32)]
    pub max_rider_count: i32,
    /// `rotate_rider_by`: the rider's rotation offset, a Molang expression; default 0.
    #[facet(default = Molang::Const(0.0), schema::molang = EXPRESSION)]
    pub rotate_rider_by: Molang,
    /// `lock_rider_rotation`: the angle the rider may turn, in degrees; above 180 (the default
    /// 181) is no limit.
    #[facet(default = 181.0_f32)]
    pub lock_rider_rotation: f32,
    /// `third_person_camera_radius`: the third-person camera's radius for a player in this
    /// seat, 1 to 64; absent keeps the camera's own.
    pub third_person_camera_radius: Option<f32>,
    /// `camera_relax_distance_smoothing`: the stiffness of the camera's return after an
    /// obstacle, 1 to 32; absent keeps the camera's own.
    pub camera_relax_distance_smoothing: Option<f32>,
}

impl Typed for Rideable {
    // Not KEEP_NESTED: the legacy reader is one of the game's schema readers, which validates
    // the whole tree before it parses: a nested value of another type never reaches this
    // read; from 1.15.0 it defaults the whole component.

    /// Below 1.26.60 a single `seats` object is the one seat (vanilla's older files write it so;
    /// the typed member is a list).
    fn prepare(json: &mut VObject, cx: &mut ComponentCheck<'_>) -> bool {
        let changed = matches!(cx.mode(), ConvertMode::Lenient)
            && json.get("seats").is_some_and(Value::is_object)
            && one_element(json, "seats", &JsonPath::new(), cx);
        if let Some(seats) = json.get("seats").and_then(Value::as_array) {
            let list = JsonPath::new().child("seats");
            for (i, seat) in seats.iter().enumerate() {
                if let Some(seat) = seat.as_object() {
                    tell_partial_vector(seat, "position", &list.index(i), cx);
                }
            }
        }
        changed
    }

    fn check(&mut self, cx: &mut ComponentCheck<'_>) {
        let seats = JsonPath::new().child("seats");
        for (i, seat) in self.seats.iter_mut().enumerate() {
            let at = seats.index(i);
            // The legacy schema reads the expression as the dynamic row did: a value it
            // rejects takes the default (0).
            if !lenient_value(&seat.rotate_rider_by, &at.child("rotate_rider_by"), cx) {
                seat.rotate_rider_by = Molang::Const(0.0);
            }
            // The typed reader bounds the seat limits to a range; the legacy schema has none.
            // By hand: a broken bound resets the member to the low end (`Some(1.0)`), where a
            // derived bound resets an `Option` to `None`.
            if matches!(cx.mode(), ConvertMode::Strict) {
                for (key, bounds, value) in [
                    (
                        "third_person_camera_radius",
                        (1.0, 64.0),
                        &mut seat.third_person_camera_radius,
                    ),
                    (
                        "camera_relax_distance_smoothing",
                        (1.0, 32.0),
                        &mut seat.camera_relax_distance_smoothing,
                    ),
                ] {
                    if let Some(number) = value {
                        cx.within(&at.child(key), number, bounds, bounds.0);
                    }
                }
            }
        }
    }
}
