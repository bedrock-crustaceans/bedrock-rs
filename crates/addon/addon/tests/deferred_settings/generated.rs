//! Round trips of generated documents: for each payload a few thousand
//! documents from fixed seeds, over the forms its members take (a single value or keyframes,
//! a colour as hex or as an array, numbers at and beyond the ends of their ranges, integers
//! of twenty, forty and four hundred digits, `-0`, `1e999`, unknown members, a missing or
//! empty identifier, every struct version of the chain in its full, short, zero-padded and
//! array spelling, members written twice and in no canonical order).
//!
//! Every document that loads is either written as text that loads again to an equal value
//! and is rewritten identically, or refused with `EncodeError::NotRepresentable` at a path,
//! for a recorded reason. And every written one is held against its input as JSON values,
//! member for member (`compare.rs`: numbers as numbers, each with its kind and its token):
//! it is its input, or differs from it by differences that each have a recorded [`Class`].
//! A difference of no class fails the test; so does a class whose finding the load did not
//! report. The classes and their counts are asserted per payload.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Document};
use bedrock_addon::families::deferred_settings::{
    AtmosphereSettings, ColorGradingSettings, CubemapSettings, LightingSettings,
    LocalLightSettings, PbrFallbackSettings, ShadowSettings, WaterSettings,
};
use bedrock_addon::json::EncodeError;

use super::common::*;
use super::compare::{Change, Difference, Json, Number, Step, differences, shown};
use super::ranges::{exact, next_down, next_up};

/// The documents generated for each payload.
const DOCUMENTS: usize = 4_000;

/// `SplitMix64`: the same documents on every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number below `n`.
    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).unwrap()
    }

    /// True once in `n` calls.
    fn one_in(&mut self, n: usize) -> bool {
        self.below(n) == 0
    }

    /// True `percent` times in a hundred.
    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    fn pick<'a>(&mut self, of: &[&'a str]) -> &'a str {
        of[self.below(of.len())]
    }
}

/// What an unknown member may hold: every JSON type, nesting, numbers a float or a double
/// does not hold, an integer beyond 64 bits.
const UNKNOWN: &[&str] = &[
    "1",
    "-2.5",
    r#""s""#,
    "true",
    "null",
    r#"[1, 2.5, "x", null]"#,
    r#"{"a": null, "b": {"c": []}}"#,
    "1e39",
    "1e999",
    "-1e999",
    r#"{"n": [1e400, {"m": -1e400}]}"#,
    "18446744073709551616",
    "-0",
    "-0.0",
    "1e2",
    "1.50",
    "007",
    WIDE_20,
    WIDE_40,
    WIDE_400,
    r#"{"b": 1, "a": [-0, 1E+2], "b": 2}"#,
];

/// Integers no 64-bit integer holds: twenty digits (just beyond the unsigned one), forty (a
/// double holds it, a float does not), four hundred (no double holds it).
const WIDE_20: &str = "99999999999999999999";
const WIDE_40: &str = "-1234567890123456789012345678901234567890";
const WIDE_400: &str = concat!(
    "1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890",
    "1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890",
    "1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890",
    "1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890",
);
const WIDE: &[&str] = &[WIDE_20, WIDE_40, WIDE_400];

/// A value of no member's type.
const WRONG: &[&str] = &[r#""1""#, "true", "[]", "{}", r#"[1, "x"]"#];

/// The members of one object while it is generated.
struct Object(Vec<String>);

impl Object {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn member(&mut self, key: &str, value: &str) {
        self.0.push(format!(r#""{key}": {value}"#));
    }

    /// The member `key`, in `percent` of the documents.
    fn some(
        &mut self,
        rng: &mut Rng,
        percent: usize,
        key: &str,
        value: impl FnOnce(&mut Rng) -> String,
    ) {
        if rng.chance(percent) {
            let value = value(rng);
            self.member(key, &value);
        }
    }

    /// The object's text: now and then with an unknown member, the members in the order they
    /// were added or turned about a random one.
    fn finish(mut self, rng: &mut Rng) -> String {
        if rng.one_in(12) {
            let key = rng.pick(&["zz", "extra", "note", "identifier", "x"]);
            let value = rng.pick(UNKNOWN);
            self.member(key, value);
        }
        // A member written twice (the last stands): the same text again, or a first
        // value of another type that the later one replaces.
        if rng.one_in(12) && !self.0.is_empty() {
            let member = self.0[rng.below(self.0.len())].clone();
            if rng.one_in(2) {
                self.0.push(member);
            } else {
                let key = member.split(": ").next().unwrap();
                let first = format!("{key}: {}", rng.pick(WRONG));
                self.0.insert(0, first);
            }
        }
        // No canonical order: turned about a random member, or back to front.
        if rng.one_in(3) && !self.0.is_empty() {
            let by = rng.below(self.0.len());
            self.0.rotate_left(by);
        } else if rng.one_in(8) {
            self.0.reverse();
        }
        format!("{{{}}}", self.0.join(", "))
    }
}

/// A number of the range `[min, max]`: mostly inside it (its ends, an integer token, a
/// fraction), now and then the next float beyond an end, a number no float holds, another
/// type.
fn number(rng: &mut Rng, min: f32, max: f32) -> String {
    match rng.below(60) {
        0 => exact(next_up(max)),
        1 => exact(next_down(min)),
        2 => rng
            .pick(&[
                "1e39", "-1e39", "1e999", "-1e999", WIDE_20, WIDE_40, WIDE_400,
            ])
            .to_owned(),
        3 => rng.pick(WRONG).to_owned(),
        // The integer zero with a sign, where zero is in the range.
        4 if min <= 0.0 && max >= 0.0 => "-0".to_owned(),
        4..=11 => exact(min),
        12..=19 => exact(max),
        20..=29 => {
            // An integer token inside the range, when there is one.
            #[allow(clippy::cast_possible_truncation, reason = "small bounds")]
            let (low, high) = (min.ceil() as i64, max.floor() as i64);
            if low > high {
                exact(max)
            } else {
                let span = usize::try_from(high - low + 1).unwrap();
                (low + i64::try_from(rng.below(span)).unwrap()).to_string()
            }
        }
        _ => {
            #[allow(clippy::cast_precision_loss, reason = "below 1000")]
            let part = rng.below(1001) as f32 / 1000.0;
            exact((min + (max - min) * part).clamp(min, max))
        }
    }
}

/// The one number without a range: any, the ones no float holds included.
fn unbounded(rng: &mut Rng) -> String {
    rng.pick(&[
        "0",
        "1.5",
        "-3",
        "100000",
        "1e39",
        "-1e39",
        "1e999",
        "3.4028235e38",
        r#""x""#,
        "-0",
        "1e2",
        WIDE_20,
        WIDE_40,
        WIDE_400,
    ])
    .to_owned()
}

/// An integer of `[min, max]`; now and then just outside it, with a fraction, a real token
/// without one, another type.
fn integer(rng: &mut Rng, min: i64, max: i64) -> String {
    match rng.below(30) {
        0 => (min - 1).to_string(),
        1 => (max + 1).to_string(),
        2 => format!("{min}.5"),
        3 => rng.pick(WRONG).to_owned(),
        4 => format!("{max}.0"),
        5 => rng.pick(WIDE).to_owned(),
        6 => rng.pick(&["-0", "1e999", "1e1"]).to_owned(),
        _ => {
            let span = usize::try_from(max - min + 1).unwrap();
            (min + i64::try_from(rng.below(span)).unwrap()).to_string()
        }
    }
}

fn boolean(rng: &mut Rng) -> String {
    match rng.below(40) {
        0 => r#""true""#.to_owned(),
        1 => "1".to_owned(),
        n => (n % 2 == 0).to_string(),
    }
}

/// One of `names`, quoted; now and then no name of the table, or no string.
fn choice(rng: &mut Rng, names: &[&str]) -> String {
    match rng.below(40) {
        0 => r#""nope""#.to_owned(),
        1 => "3".to_owned(),
        _ => format!(r#""{}""#, rng.pick(names)),
    }
}

/// `count` channels as hex digits or as an array; now and then a channel outside 0–255
/// (clamped), a fraction, the other colour's length, another type.
fn channels(rng: &mut Rng, count: usize) -> String {
    let other = if count == 3 { 4 } else { 3 };
    let hex = |rng: &mut Rng, count: usize| {
        let digits: String = (0..2 * count)
            .map(|_| char::from(b"0123456789abcdefABCDEF"[rng.below(22)]))
            .collect();
        format!(r##""#{digits}""##)
    };
    let array = |rng: &mut Rng, count: usize| {
        let elements: Vec<String> = (0..count)
            .map(|_| match rng.below(40) {
                0 => "300".to_owned(),
                1 => "-5".to_owned(),
                2 => "12.0".to_owned(),
                3 => "1e39".to_owned(),
                4 => "-0".to_owned(),
                5 => rng.pick(WIDE).to_owned(),
                6 => rng.pick(&["1e999", "-1e999", "1e2", "007"]).to_owned(),
                _ => rng.below(256).to_string(),
            })
            .collect();
        format!("[{}]", elements.join(", "))
    };
    match rng.below(50) {
        0 => hex(rng, other),
        1 => array(rng, other),
        2 => if count == 3 {
            "[1, 2.5, 3]"
        } else {
            "[1, 2.5, 3, 4]"
        }
        .to_owned(),
        3 => rng
            .pick(&[r#""red""#, r##""#abc""##, r#""FFFFFF""#, "7", "{}"])
            .to_owned(),
        n if n % 2 == 0 => hex(rng, count),
        _ => array(rng, count),
    }
}

fn rgb(rng: &mut Rng) -> String {
    channels(rng, 3)
}

fn rgba(rng: &mut Rng) -> String {
    channels(rng, 4)
}

/// Keys of keyframes: times of the day in several spellings (some of them one time), and now
/// and then one outside the day, one that is no number.
const TIMES: &[&str] = &[
    "0", "0.0", "-0", "0.25", "0.250000", "0.5", "0.50", "5e-1", "0x.8p0", " 0.75", "0.75", "1",
    "1.0", "0x1", "0.1", "0.5x",
];
const NO_TIMES: &[&str] = &["1.5", "-0.1", "nan", "inf", "0x10", "noon", "", "1e39"];

/// A keyframed member: its single value, or an object of one to four keyframes (now and then
/// of none).
fn keyframed(rng: &mut Rng, value: impl Fn(&mut Rng) -> String) -> String {
    if rng.one_in(3) {
        return value(rng);
    }
    if rng.one_in(60) {
        return rng.pick(&["{}", "[]", r#""0.5""#]).to_owned();
    }
    let frames: Vec<String> = (0..=rng.below(4))
        .map(|_| {
            let key = if rng.one_in(25) {
                rng.pick(NO_TIMES)
            } else {
                rng.pick(TIMES)
            };
            format!(r#""{key}": {}"#, value(rng))
        })
        .collect();
    format!("{{{}}}", frames.join(", "))
}

/// Three numbers of `[min, max]`: an array, or an object of some of `x`, `y`, `z`.
fn vec3(rng: &mut Rng, min: f32, max: f32) -> String {
    match rng.below(30) {
        0 => format!("[{}, {}]", number(rng, min, max), number(rng, min, max)),
        1 => rng.pick(WRONG).to_owned(),
        n if n % 3 == 0 => {
            let mut object = Object::new();
            for name in ["x", "y", "z"] {
                object.some(rng, 70, name, |rng| number(rng, min, max));
            }
            format!("{{{}}}", object.0.join(", "))
        }
        _ => format!(
            "[{}, {}, {}]",
            number(rng, min, max),
            number(rng, min, max),
            number(rng, min, max)
        ),
    }
}

/// A `description`, or none: mostly a namespaced identifier; now and then one without a
/// namespace, an empty one, none, one of another type, an unknown member beside it, no object.
fn describe(rng: &mut Rng, object: &mut Object) {
    let value = match rng.below(24) {
        0 => return,
        1 => "{}".to_owned(),
        2 => r#"{"identifier": ""}"#.to_owned(),
        3 => r#"{"identifier": "plain"}"#.to_owned(),
        4 => r#"{"identifier": 5}"#.to_owned(),
        5 => r#"{"identifier": "t:a", "zz": [1e999]}"#.to_owned(),
        6 => "[]".to_owned(),
        _ => format!(r#"{{"identifier": "t:{}"}}"#, rng.below(50)),
    };
    object.member("description", &value);
}

/// The document of `payload` under `key` at `version`; now and then with another version
/// (between two structs, below the first, a pre-release), none, a third root member, or a
/// payload that is no object.
fn envelope(rng: &mut Rng, key: &str, version: &str, payload: &str) -> String {
    let version = if rng.one_in(40) {
        rng.pick(&["1.21.50", "1.0.0", "1.99.0", "1.26.20-beta", "1.21", ""])
    } else {
        version
    };
    let payload = if rng.one_in(80) {
        rng.pick(&["[]", "5", "null", r#""x""#])
    } else {
        payload
    };
    let version = spelt(rng, version);
    match rng.below(120) {
        0 => format!(r#"{{"{key}": {payload}}}"#),
        1 => format!(r#"{{"format_version": {version}, "{key}": {payload}, "zz": 1}}"#),
        2..=20 => format!(r#"{{"{key}": {payload}, "format_version": {version}}}"#),
        // A root member written twice.
        21 => format!(r#"{{"format_version": {version}, "{key}": 5, "{key}": {payload}}}"#),
        22 => format!(
            r#"{{"format_version": "1.0.0", "format_version": {version}, "{key}": {payload}}}"#
        ),
        _ => format!(r#"{{"format_version": {version}, "{key}": {payload}}}"#),
    }
}

/// The JSON of `version`: mostly the string in full; now and then (where it has three
/// numbers) its short string when the patch is 0, the array, a string or an array with
/// leading zeros, an array with a wide or a real element.
fn spelt(rng: &mut Rng, version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    let full = format!(r#""{version}""#);
    let [major, minor, patch] = parts[..] else {
        return full;
    };
    if patch.parse::<u32>().is_err() {
        return full;
    }
    match rng.below(120) {
        0..=11 if patch == "0" => format!(r#""{major}.{minor}""#),
        12..=23 => format!(r#""0{major}.0{minor}.00{patch}""#),
        24 | 25 => format!("[{major}, {minor}, {patch}]"),
        26 => format!("[0{major}, {minor}, 0{patch}]"),
        27 => format!("[{major}, {minor}, {WIDE_20}]"),
        28 => format!("[{major}, {minor}.0, {patch}]"),
        29 => format!("[{major}, {minor}]"),
        _ => full,
    }
}

/// Whether `version` (one of the chain's) is at or above `floor`.
fn from(version: &str, floor: (u32, u32, u32)) -> bool {
    let mut parts = version.split('.').map(|part| part.parse::<u32>().unwrap());
    let version = (
        parts.next().unwrap(),
        parts.next().unwrap(),
        parts.next().unwrap(),
    );
    version >= floor
}

/// A member a struct declares, or (now and then) one it does not declare yet.
fn declared(rng: &mut Rng, declares: bool) -> bool {
    declares || rng.one_in(15)
}

fn lighting(rng: &mut Rng) -> String {
    let version = rng.pick(&["1.21.40", "1.21.60", "1.21.70", "1.21.80", "1.26.0"]);
    let nested = from(version, (1, 21, 80));
    let keyframed_ambient = from(version, (1, 26, 0));
    let alpha = version == "1.21.40";
    let colour = |rng: &mut Rng| {
        // The struct's own colour form, and now and then the other one.
        if alpha == rng.one_in(30) {
            rgb(rng)
        } else {
            rgba(rng)
        }
    };
    let light = |rng: &mut Rng| {
        let mut light = Object::new();
        light.some(rng, 70, "illuminance", |rng| {
            keyframed(rng, |rng| number(rng, 0.0, 110_000.0))
        });
        light.some(rng, 70, "color", |rng| keyframed(rng, colour));
        light.finish(rng)
    };
    let orbital = |rng: &mut Rng| {
        let mut orbital = Object::new();
        orbital.some(rng, 60, "sun", light);
        orbital.some(rng, 60, "moon", light);
        orbital.some(rng, 50, "orbital_offset_degrees", |rng| {
            keyframed(rng, |rng| number(rng, 0.0, 359.99))
        });
        orbital
    };
    let flash = |rng: &mut Rng| {
        let mut flash = Object::new();
        flash.some(rng, 70, "color", rgb);
        flash.some(rng, 70, "illuminance", unbounded);
        flash.finish(rng)
    };
    let mut payload = Object::new();
    describe(rng, &mut payload);
    payload.some(rng, 75, "directional_lights", |rng| {
        if nested {
            let mut lights = Object::new();
            lights.some(rng, 80, "orbital", |rng| orbital(rng).finish(rng));
            lights.some(rng, 50, "flash", flash);
            lights.finish(rng)
        } else {
            let mut lights = orbital(rng);
            // Members a newer struct declares at this level.
            if rng.one_in(15) {
                lights.member("flash", &flash(rng));
            }
            if rng.one_in(15) {
                let nested = orbital(rng).finish(rng);
                lights.member("orbital", &nested);
            }
            lights.finish(rng)
        }
    });
    payload.some(rng, 50, "emissive", |rng| {
        let mut emissive = Object::new();
        emissive.some(rng, 80, "desaturation", |rng| number(rng, 0.0, 1.0));
        emissive.finish(rng)
    });
    payload.some(rng, 60, "ambient", |rng| {
        let mut ambient = Object::new();
        // Both are required.
        ambient.some(rng, 95, "color", |rng| {
            if keyframed_ambient {
                keyframed(rng, rgb)
            } else {
                rgb(rng)
            }
        });
        ambient.some(rng, 95, "illuminance", |rng| {
            if keyframed_ambient {
                keyframed(rng, |rng| number(rng, 0.0, 5.0))
            } else {
                number(rng, 0.0, 5.0)
            }
        });
        ambient.finish(rng)
    });
    if declared(rng, from(version, (1, 21, 70))) && rng.chance(60) {
        let mut sky = Object::new();
        sky.some(rng, 80, "intensity", |rng| {
            if keyframed_ambient {
                keyframed(rng, |rng| number(rng, 0.1, 1.0))
            } else {
                number(rng, 0.1, 1.0)
            }
        });
        let sky = sky.finish(rng);
        payload.member("sky", &sky);
    }
    let payload = payload.finish(rng);
    envelope(rng, LIGHTING, version, &payload)
}

fn water(rng: &mut Rng) -> String {
    let version = rng.pick(&["1.21.40", "1.21.80", "1.21.120", "1.26.0", "1.26.20"]);
    let from_80 = from(version, (1, 21, 80));
    let mut payload = Object::new();
    describe(rng, &mut payload);
    payload.some(rng, 60, "particle_concentrations", |rng| {
        let mut particles = Object::new();
        particles.some(rng, 70, "chlorophyll", |rng| number(rng, 0.0, 10.0));
        particles.some(rng, 70, "suspended_sediment", |rng| number(rng, 0.0, 300.0));
        particles.some(rng, 70, "cdom", |rng| number(rng, 0.0, 15.0));
        particles.finish(rng)
    });
    payload.some(rng, 70, "waves", |rng| {
        let mut waves = Object::new();
        waves.some(rng, 50, "enabled", boolean);
        waves.some(rng, 50, "frequency", |rng| number(rng, 0.01, 3.0));
        waves.some(rng, 50, "octaves", |rng| integer(rng, 1, 30));
        waves.some(rng, 50, "depth", |rng| number(rng, 0.0, 3.0));
        if declared(rng, !from(version, (1, 21, 120))) {
            waves.some(rng, 50, "sampleWidth", |rng| number(rng, 0.01, 1.0));
        }
        waves.some(rng, 50, "speed", |rng| number(rng, 0.01, 10.0));
        waves.some(rng, 50, "pull", |rng| number(rng, -1.0, 1.0));
        waves.some(rng, 50, "mix", |rng| number(rng, 0.0, 1.0));
        waves.some(rng, 50, "frequency_scaling", |rng| number(rng, 0.0, 2.0));
        waves.some(rng, 50, "speed_scaling", |rng| number(rng, 0.0, 2.0));
        if declared(rng, from_80) {
            waves.some(rng, 50, "direction_increment", |rng| {
                number(rng, 0.0, 360.0)
            });
        }
        waves.some(rng, 50, "shape", |rng| number(rng, 1.0, 10.0));
        waves.finish(rng)
    });
    if declared(rng, from_80) {
        payload.some(rng, 60, "caustics", |rng| {
            let mut caustics = Object::new();
            caustics.some(rng, 60, "enabled", boolean);
            caustics.some(rng, 60, "frame_length", |rng| number(rng, 0.01, 5.0));
            caustics.some(rng, 60, "texture", |rng| {
                rng.pick(&[r#""textures/environment/caustics""#, r#""""#, "5"])
                    .to_owned()
            });
            caustics.some(rng, 60, "scale", |rng| number(rng, 0.1, 5.0));
            caustics.some(rng, 60, "power", |rng| number(rng, 1.0, 6.0));
            caustics.finish(rng)
        });
    }
    if declared(rng, from(version, (1, 26, 0))) {
        payload.some(rng, 60, "biome_water_color_contribution", |rng| {
            number(rng, 0.0, 1.0)
        });
    }
    let payload = payload.finish(rng);
    envelope(rng, WATER, version, &payload)
}

fn atmosphere(rng: &mut Rng) -> String {
    let version = rng.pick(&["1.21.40", "1.26.20"]);
    let ranged = |object: &mut Object, rng: &mut Rng, key: &str, max: f32| {
        object.some(rng, 60, key, |rng| {
            keyframed(rng, |rng| number(rng, 0.0, max))
        });
    };
    let mut payload = Object::new();
    describe(rng, &mut payload);
    payload.some(rng, 60, "horizon_blend_stops", |rng| {
        let mut stops = Object::new();
        ranged(&mut stops, rng, "min", 1.0);
        ranged(&mut stops, rng, "start", 1.0);
        ranged(&mut stops, rng, "mie_start", 1.2);
        ranged(&mut stops, rng, "max", 1.0);
        stops.finish(rng)
    });
    ranged(&mut payload, rng, "rayleigh_strength", 50.0);
    ranged(&mut payload, rng, "sun_mie_strength", 60.0);
    ranged(&mut payload, rng, "moon_mie_strength", 20.0);
    ranged(&mut payload, rng, "sun_glare_shape", 50.0);
    payload.some(rng, 60, "sky_zenith_color", |rng| keyframed(rng, rgb));
    payload.some(rng, 60, "sky_horizon_color", |rng| keyframed(rng, rgb));
    let payload = payload.finish(rng);
    envelope(rng, ATMOSPHERE, version, &payload)
}

fn grading(rng: &mut Rng) -> String {
    let version = rng.pick(&["1.21.40", "1.21.90"]);
    let controls = |object: &mut Object, rng: &mut Rng| {
        for (key, min, max) in [
            ("contrast", 0.0, 4.0),
            ("gain", 0.0, 10.0),
            ("gamma", 0.0, 4.0),
            ("offset", -1.0, 1.0),
            ("saturation", 0.0, 10.0),
        ] {
            object.some(rng, 40, key, |rng| vec3(rng, min, max));
        }
    };
    let mut payload = Object::new();
    describe(rng, &mut payload);
    payload.some(rng, 80, "color_grading", |rng| {
        let mut grading = Object::new();
        grading.some(rng, 60, "midtones", |rng| {
            let mut tones = Object::new();
            controls(&mut tones, rng);
            tones.finish(rng)
        });
        grading.some(rng, 60, "highlights", |rng| {
            let mut tones = Object::new();
            tones.some(rng, 50, "enabled", boolean);
            tones.some(rng, 50, "highlightsMin", |rng| number(rng, 1.0, 20.0));
            controls(&mut tones, rng);
            tones.finish(rng)
        });
        grading.some(rng, 60, "shadows", |rng| {
            let mut tones = Object::new();
            tones.some(rng, 50, "enabled", boolean);
            tones.some(rng, 50, "shadowsMax", |rng| number(rng, 0.0, 1.0));
            controls(&mut tones, rng);
            tones.finish(rng)
        });
        if declared(rng, version == "1.21.90") {
            grading.some(rng, 60, "temperature", |rng| {
                let mut temperature = Object::new();
                temperature.some(rng, 60, "enabled", boolean);
                temperature.some(rng, 60, "temperature", |rng| number(rng, 1000.0, 15000.0));
                temperature.some(rng, 60, "type", |rng| {
                    choice(rng, &["white_balance", "color_temperature"])
                });
                temperature.finish(rng)
            });
        }
        grading.finish(rng)
    });
    payload.some(rng, 60, "tone_mapping", |rng| {
        let mut mapping = Object::new();
        mapping.some(rng, 80, "operator", |rng| {
            choice(
                rng,
                &[
                    "reinhard",
                    "reinhard_luma",
                    "reinhard_luminance",
                    "hable",
                    "aces",
                    "generic",
                ],
            )
        });
        mapping.finish(rng)
    });
    let payload = payload.finish(rng);
    envelope(rng, GRADING, version, &payload)
}

fn cubemap(rng: &mut Rng) -> String {
    let mut payload = Object::new();
    describe(rng, &mut payload);
    payload.some(rng, 85, "lighting", |rng| {
        let mut lighting = Object::new();
        lighting.some(rng, 70, "ambient_light_illuminance", |rng| {
            keyframed(rng, |rng| number(rng, 0.0, 100_000.0))
        });
        lighting.some(rng, 60, "sky_light_contribution", |rng| {
            number(rng, 0.0, 1.0)
        });
        lighting.some(rng, 60, "directional_light_contribution", |rng| {
            number(rng, 0.0, 1.0)
        });
        lighting.some(rng, 50, "affected_by_atmospheric_scattering", boolean);
        lighting.some(rng, 50, "affected_by_volumetric_scattering", boolean);
        lighting.finish(rng)
    });
    let payload = payload.finish(rng);
    envelope(rng, CUBEMAP, "1.21.130", &payload)
}

fn shadows(rng: &mut Rng) -> String {
    let mut payload = Object::new();
    payload.some(rng, 70, "shadow_style", |rng| {
        choice(rng, &["soft_shadows", "blocky_shadows"])
    });
    payload.some(rng, 70, "texel_size", |rng| integer(rng, 1, 1024));
    let payload = payload.finish(rng);
    envelope(rng, SHADOWS, "1.21.80", &payload)
}

fn pbr(rng: &mut Rng) -> String {
    let mut payload = Object::new();
    for kind in ["blocks", "actors", "particles", "items"] {
        payload.some(rng, 60, kind, |rng| {
            let mut kind = Object::new();
            // The four-value member, the three-value one, both, neither.
            let form = rng.below(8);
            if form < 4 || form == 6 {
                let value = rgba(rng);
                kind.member("global_metalness_emissive_roughness_subsurface", &value);
            }
            if (4..7).contains(&form) {
                let value = rgb(rng);
                kind.member("global_metalness_emissive_roughness", &value);
            }
            kind.finish(rng)
        });
    }
    let payload = payload.finish(rng);
    envelope(rng, PBR, "1.21.40", &payload)
}

fn local_lights(rng: &mut Rng) -> String {
    let mut entries = Vec::new();
    for _ in 0..rng.below(6) {
        // Two spellings of one block, a block of another namespace, the empty name.
        let block = rng.pick(&[
            "minecraft:torch",
            "torch",
            "minecraft:lantern",
            "t:lamp",
            "t:other",
            "",
        ]);
        let mut light = Object::new();
        light.some(rng, 70, "light_type", |rng| {
            choice(rng, &["static_light", "point_light"])
        });
        light.some(rng, 70, "light_color", rgb);
        entries.push(format!(r#""{block}": {}"#, light.finish(rng)));
    }
    let payload = format!("{{{}}}", entries.join(", "));
    envelope(rng, LOCAL, "1.21.120", &payload)
}

/// Why the writer may refuse a document that loaded: the rule that records the class.
fn refusal(key: &str, at: &str) -> Option<&'static str> {
    let below = at.strip_prefix(key)?.strip_prefix('/')?;
    match (key, below) {
        // a water or atmosphere document of a struct older than 1.26.20 loads
        // without `description`; the value is the 1.26.20 one, in which it is required.
        (WATER | ATMOSPHERE, "description") => Some("description-required"),
        _ => None,
    }
}

/// One recorded way in which a written document is not its input, member for member. Each is
/// a recorded rule. A difference of no class fails the test.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    /// The document was read by an older struct: it is written at the newest one's version.
    VersionRestamped,
    /// "version respelt": the same version, written in full.
    VersionRespelt,
    /// The members stand in the writer's order: `format_version` first, a struct's members in
    /// the order it declares them, unknown members after them, keyframes by time.
    MemberOrder,
    /// A member written twice: the last value stands. Counted by document.
    MemberTwice,
    /// "number respelt": a number of an unknown member, in the shortest spelling of
    /// its value in its kind (`1e2` → `100.0`, `-0` → `0`, `007` → `7`).
    NumberRespelt,
    /// The part of "number respelt" that changes the token's kind: an integer beyond 64 bits
    /// in an unknown member is the real it spells.
    WideIntegerAsReal,
    /// A number of an unknown member that no double holds: the largest finite double of its
    /// sign, with a warning.
    BeyondADouble,
    /// A number member: the shortest text of its `f32`, a real.
    ShortestFloat,
    /// An integer member or colour channel: the integer, whatever token spelt it (`12.0`,
    /// `007`, `-0`).
    ShortestInteger,
    /// The one number without a range, beyond `f32`: the largest finite float of its sign,
    /// with a warning.
    BeyondAFloat,
    /// A colour channel outside 0–255: clamped, with an info.
    ChannelClamped,
    /// A colour written as hex digits: its array.
    ColourAsArray,
    /// Three numbers written as an object: the array, an absent member 0.
    Vec3AsArray,
    /// A keyframe key in another spelling of its time: the shortest text of the `f32` that
    /// `strtof` reads.
    KeyRespelt,
    /// A keyframe outside the day: dropped, with a warning.
    KeyOutsideTheDay,
    /// Two keys of one time: the later stands, with a warning; the other is dropped, with an
    /// info.
    KeyOfOneTimeTwice,
    /// A member of a newer struct in an older document: unknown, with the version it needs,
    /// and not kept by the step that declares it.
    NewerMemberDropped,
    /// Lighting 1.21.40 → 1.21.60: the alpha of a directional light's colour.
    StepDroppedAlpha,
    /// Lighting 1.21.70 → 1.21.80: sun, moon and offset moved under `orbital`.
    StepMoved,
    /// A member a step writes: lighting's `sky.intensity`, `ambient` and its two members,
    /// colour grading's `temperature`.
    StepInserted,
    /// Water 1.21.80 → 1.21.120: `waves.sampleWidth`, with an info.
    StepDropped,
    /// Lighting 1.21.80 → 1.26.0: `ambient.color`, `ambient.illuminance` and `sky.intensity`
    /// as one keyframe at `0.0`.
    StepWrapped,
    /// Two keys of a local-light file that name one block (`torch`, `minecraft:torch`): the
    /// later entry stands under the first key's spelling, with a warning.
    BlockOfTwoSpellings,
}

/// The newest struct of each payload: the version every written document has.
fn newest(key: &str) -> (u32, u32, u32) {
    match key {
        LIGHTING => (1, 26, 0),
        WATER | ATMOSPHERE => (1, 26, 20),
        GRADING => (1, 21, 90),
        CUBEMAP => (1, 21, 130),
        SHADOWS => (1, 21, 80),
        PBR => (1, 21, 40),
        LOCAL => (1, 21, 120),
        other => panic!("{other}"),
    }
}

/// The version a `format_version` string spells: two or three numbers, leading zeros or not.
fn version_of(text: &str) -> Option<(u32, u32, u32)> {
    let mut parts = text.split('.').map(|part| part.parse::<u32>().ok());
    let version = (
        parts.next()??,
        parts.next()??,
        parts.next().unwrap_or(Some(0))?,
    );
    parts.next().is_none().then_some(version)
}

/// What a keyframe key is written as (the shortest text of the time `strtof` reads from it),
/// or `None` for a key outside the day.
fn written_key(key: &str) -> Option<&'static str> {
    Some(match key {
        "0" | "0.0" | "-0" => "0.0",
        "0.25" | "0.250000" => "0.25",
        "0.5" | "0.50" | "5e-1" | "0x.8p0" | "0.5x" => "0.5",
        " 0.75" | "0.75" => "0.75",
        "1" | "1.0" | "0x1" => "1.0",
        "0.1" => "0.1",
        _ => return None,
    })
}

/// An object of keyframes, as the generator writes them.
fn is_keyframes(members: &[(String, Json)]) -> bool {
    !members.is_empty()
        && members
            .iter()
            .all(|(key, _)| TIMES.contains(&key.as_str()) || NO_TIMES.contains(&key.as_str()))
}

/// The channels of a colour written as hex digits: `#RRGGBB`, or `#AARRGGBB` as red, green,
/// blue, alpha.
fn hex_channels(text: &str) -> Option<Json> {
    let digits = text.strip_prefix('#')?;
    if !matches!(digits.len(), 6 | 8) || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let mut channels: Vec<Json> = (0..digits.len() / 2)
        .map(|at| {
            let value = u8::from_str_radix(&digits[2 * at..2 * at + 2], 16).unwrap();
            Json::Num(Number {
                int: Some(i128::from(value)),
                real: f64::from(value),
                token: value.to_string(),
            })
        })
        .collect();
    if channels.len() == 4 {
        channels.rotate_left(1);
    }
    Some(Json::Arr(channels))
}

fn zero() -> Json {
    Json::Num(Number {
        int: Some(0),
        real: 0.0,
        token: "0".to_owned(),
    })
}

/// A member a struct declares from a version on: the version, the keys to its object below
/// the payload, its name.
type Newer = ((u32, u32, u32), &'static [&'static str], &'static str);

/// One document's comparison: what is known of it, and what was found.
struct Compared<'a> {
    key: &'a str,
    /// The version the input was written at.
    version: (u32, u32, u32),
    /// `(severity, code name, JSON path)` of the load's findings.
    findings: Vec<(Severity, &'static str, String)>,
    /// The paths [`Self::expect`] moved: `(in the input, in the written text)`, in the order
    /// they were moved in.
    moved: Vec<(String, String)>,
    classes: Vec<Class>,
    /// Differences of no class, or of a class whose finding is missing.
    unrecorded: Vec<(String, String)>,
}

impl Compared<'_> {
    fn found(&self, severity: Severity, code: &str, path: &str) -> bool {
        self.findings
            .iter()
            .any(|(s, c, p)| *s == severity && *c == code && p == path)
    }

    /// Whether `path` is an unknown member, kept as it was read.
    fn unknown(&self, path: &str) -> bool {
        self.found(Severity::Info, "UnrecognizedField", path)
    }

    /// Counts `class`; when `finding` is given, only if the load reported it.
    fn count(&mut self, class: Class, finding: Option<(Severity, &str, &str)>) {
        match finding {
            Some((severity, code, path)) if !self.found(severity, code, path) => {
                self.unrecorded.push((
                    format!("{class:?} without its finding"),
                    format!("no {severity:?} {code} at {path}"),
                ));
            }
            _ => self.classes.push(class),
        }
    }

    /// Gives each finding the path its place has in the written text: a finding names a
    /// keyframe by the key of the file, and a member a step moved by where it was.
    fn follow_the_moves(&mut self) {
        for (_, _, path) in &mut self.findings {
            for (from, to) in &self.moved {
                if let Some(rest) = path.strip_prefix(from.as_str())
                    && (rest.is_empty() || rest.starts_with('/'))
                {
                    *path = format!("{to}{rest}");
                }
            }
        }
    }

    /// Applies to the input the recorded rewrites that move, rename or drop members, each by
    /// name: after it, the input and the written text have the same members at the same
    /// paths and differ only value by value.
    fn expect(&mut self, input: &mut Json) {
        let key = self.key;
        let v = self.version;
        let newer: &[Newer] = match key {
            LIGHTING => &[
                ((1, 21, 70), &[], "sky"),
                ((1, 21, 80), &["directional_lights"], "flash"),
                ((1, 21, 80), &["directional_lights"], "orbital"),
            ],
            WATER => &[
                ((1, 21, 80), &[], "caustics"),
                ((1, 21, 80), &["waves"], "direction_increment"),
                ((1, 26, 0), &[], "biome_water_color_contribution"),
            ],
            GRADING => &[((1, 21, 90), &["color_grading"], "temperature")],
            _ => &[],
        };
        let Some(payload) = input.at_mut(&[key]) else {
            return;
        };
        for (since, parent, name) in newer {
            if v >= *since {
                continue;
            }
            let Some(object) = payload.at_mut(parent) else {
                continue;
            };
            if object.take(name).is_some() {
                let mut path = vec![key];
                path.extend_from_slice(parent);
                path.push(name);
                let path = path.join("/");
                self.count(
                    Class::NewerMemberDropped,
                    Some((Severity::Info, "UnrecognizedField", &path)),
                );
                // The member is gone: what a step puts at its path is no unknown member.
                self.findings
                    .retain(|(_, code, at)| !(*code == "UnrecognizedField" && *at == path));
            }
        }
        self.forms(payload, &mut vec![key.to_owned()]);
        match key {
            LIGHTING => self.lighting_steps(payload),
            WATER if v < (1, 21, 120) => {
                let dropped = payload
                    .at_mut(&["waves"])
                    .and_then(|waves| waves.take("sampleWidth"));
                if dropped.is_some() {
                    let path = format!("{key}/waves/sampleWidth");
                    self.count(
                        Class::StepDropped,
                        Some((Severity::Info, "DroppedField", &path)),
                    );
                }
            }
            LOCAL => {
                let block = |name: &str| {
                    if name.contains(':') {
                        name.to_owned()
                    } else {
                        format!("minecraft:{name}")
                    }
                };
                let mut standing: Vec<(String, Json)> = Vec::new();
                for (name, light) in std::mem::take(payload.members_mut().unwrap()) {
                    match standing
                        .iter_mut()
                        .find(|(other, _)| block(other) == block(&name))
                    {
                        Some((first, earlier)) => {
                            let path = format!("{key}/{name}");
                            self.moved.push((path.clone(), format!("{key}/{first}")));
                            *earlier = light;
                            self.count(
                                Class::BlockOfTwoSpellings,
                                Some((Severity::Info, "DuplicateIdentifier", &path)),
                            );
                        }
                        None => standing.push((name, light)),
                    }
                }
                *payload.members_mut().unwrap() = standing;
            }
            _ => {}
        }
    }

    /// The forms of a value that are written as another form, everywhere below `value` but
    /// in an unknown member: keyframe keys, a colour as hex digits, three numbers as an
    /// object.
    fn forms(&mut self, value: &mut Json, path: &mut Vec<String>) {
        let at = path.join("/");
        if self.unknown(&at) {
            return;
        }
        if let Json::Str(text) = value {
            if let Some(channels) = hex_channels(text) {
                *value = channels;
                self.classes.push(Class::ColourAsArray);
            }
            return;
        }
        let Json::Obj(members) = value else {
            return;
        };
        let vec3 = self.key == GRADING
            && ["contrast", "gain", "gamma", "offset", "saturation"]
                .contains(&path.last().unwrap().as_str());
        if vec3 {
            let elements = ["x", "y", "z"]
                .iter()
                .map(|name| value.take(name).unwrap_or_else(zero))
                .collect();
            *value = Json::Arr(elements);
            self.classes.push(Class::Vec3AsArray);
            return;
        }
        if self.key != LOCAL && is_keyframes(members) {
            let mut frames: Vec<(String, Json)> = Vec::new();
            for (key, frame) in std::mem::take(members) {
                let Some(written) = written_key(&key) else {
                    let path = format!("{at}/{key}");
                    self.count(
                        Class::KeyOutsideTheDay,
                        Some((Severity::Info, "KeyframeOutOfDomain", &path)),
                    );
                    continue;
                };
                frames.push((format!("{key}\u{0}{written}"), frame));
            }
            // Of two keys of one time the later stands.
            let mut standing: Vec<(String, Json)> = Vec::new();
            for (both, frame) in frames {
                let (key, written) = both.split_once('\u{0}').unwrap();
                if let Some(earlier) = standing
                    .iter()
                    .position(|(other, _)| other.split_once('\u{0}').unwrap().1 == written)
                {
                    let (lost, _) = standing.remove(earlier);
                    let lost = format!("{at}/{}", lost.split_once('\u{0}').unwrap().0);
                    let stands = format!("{at}/{key}");
                    if self.found(Severity::Info, "InvalidValue", &stands) {
                        self.count(
                            Class::KeyOfOneTimeTwice,
                            Some((Severity::Info, "DroppedField", &lost)),
                        );
                    } else {
                        self.count(
                            Class::KeyOfOneTimeTwice,
                            Some((Severity::Info, "InvalidValue", &stands)),
                        );
                    }
                }
                standing.push((both.clone(), frame));
            }
            for (both, mut frame) in standing {
                let (key, written) = both.split_once('\u{0}').unwrap();
                // The findings name the key as the file spells it.
                path.push(key.to_owned());
                self.forms(&mut frame, path);
                path.pop();
                if key != written {
                    self.classes.push(Class::KeyRespelt);
                    self.moved
                        .push((format!("{at}/{key}"), format!("{at}/{written}")));
                }
                members.push((written.to_owned(), frame));
            }
            return;
        }
        for (key, member) in members {
            path.push(key.clone());
            self.forms(member, path);
            path.pop();
        }
    }

    /// What lighting's steps move and drop.
    fn lighting_steps(&mut self, payload: &mut Json) {
        let v = self.version;
        let key = self.key;
        if v < (1, 26, 0) {
            for path in [
                ["ambient", "color"],
                ["ambient", "illuminance"],
                ["sky", "intensity"],
            ] {
                if let Some(value) = payload.at_mut(&path) {
                    *value = Json::Obj(vec![("0.0".to_owned(), value.clone())]);
                    self.classes.push(Class::StepWrapped);
                    let path = format!("{key}/{}", path.join("/"));
                    self.moved.push((path.clone(), format!("{path}/0.0")));
                }
            }
        }
        let Some(lights) = payload.at_mut(&["directional_lights"]) else {
            return;
        };
        if v < (1, 21, 60) {
            for light in ["sun", "moon"] {
                let Some(colour) = lights.at_mut(&[light, "color"]) else {
                    continue;
                };
                let mut colours: Vec<&mut Json> = match colour {
                    Json::Obj(frames) => frames.iter_mut().map(|(_, frame)| frame).collect(),
                    single => vec![single],
                };
                for colour in &mut colours {
                    if let Json::Arr(channels) = colour
                        && channels.len() == 4
                    {
                        channels.pop();
                        self.classes.push(Class::StepDroppedAlpha);
                    }
                }
            }
        }
        if v < (1, 21, 80) {
            let moved: Vec<(String, Json)> = ["sun", "moon", "orbital_offset_degrees"]
                .iter()
                .filter_map(|name| Some(((*name).to_owned(), lights.take(name)?)))
                .collect();
            // `orbital` is written whatever the older `directional_lights` held.
            {
                self.classes.push(Class::StepMoved);
                for (name, _) in &moved {
                    let lights = format!("{key}/directional_lights");
                    self.moved.push((
                        format!("{lights}/{name}"),
                        format!("{lights}/orbital/{name}"),
                    ));
                }
                lights
                    .members_mut()
                    .unwrap()
                    .insert(0, ("orbital".to_owned(), Json::Obj(moved)));
            }
        }
    }

    /// The class of one difference that is left after [`Self::expect`].
    #[expect(clippy::float_cmp, reason = "the clamp to `f64::MAX` is exact")]
    fn classify(&mut self, difference: &Difference) {
        let at = shown(&difference.path);
        let below = at
            .strip_prefix(self.key)
            .map(|rest| rest.trim_start_matches('/'));
        let raw = (1..=difference.path.len())
            .any(|length| self.unknown(&shown(&difference.path[..length])));
        let v = self.version;
        let class = match (&difference.change, below) {
            (Change::Reordered, _) => Some((Class::MemberOrder, None)),
            (Change::Replaced(Json::Str(before), Json::Str(after)), None)
                if at == "format_version" =>
            {
                let (before, after) = (version_of(before), version_of(after));
                assert_eq!(after, Some(newest(self.key)));
                if before == after {
                    Some((Class::VersionRespelt, None))
                } else {
                    assert_eq!(before, Some(v));
                    Some((Class::VersionRestamped, None))
                }
            }
            (Change::Respelt(before, after), _) if raw => {
                let real = |number: &Number| number.token.contains(['.', 'e', 'E']);
                if !real(before) && real(after) {
                    Some((Class::WideIntegerAsReal, None))
                } else {
                    Some((Class::NumberRespelt, None))
                }
            }
            (Change::Replaced(Json::Num(before), Json::Num(after)), _) if raw => {
                (before.real.is_infinite() && after.real == f64::MAX.copysign(before.real))
                    .then_some((Class::BeyondADouble, Some((Severity::Info, "InvalidValue"))))
            }
            (Change::Respelt(before, after), _) => {
                number_class(before, after).map(|class| (class, None))
            }
            (Change::Replaced(Json::Num(before), Json::Num(after)), _) => {
                number_class(before, after).map(|class| {
                    let finding = match class {
                        Class::BeyondAFloat => Some((Severity::Info, "InvalidValue")),
                        Class::ChannelClamped => Some((Severity::Info, "ClampedValue")),
                        _ => None,
                    };
                    (class, finding)
                })
            }
            (Change::Added(added), Some(below)) => {
                // What the step writes where the older document has nothing.
                const INTENSITY: &str = r#"{"0.0": 1.0}"#;
                const AMBIENT: &str =
                    r#"{"color": {"0.0": [255, 255, 255]}, "illuminance": {"0.0": 0.02}}"#;
                const TEMPERATURE: &str =
                    r#"{"enabled": false, "temperature": 6500.0, "type": "white_balance"}"#;
                let inserted = match (self.key, below) {
                    (LIGHTING, "sky") if v < (1, 26, 0) => {
                        format!(r#"{{"intensity": {INTENSITY}}}"#)
                    }
                    (LIGHTING, "sky/intensity") if v < (1, 26, 0) => INTENSITY.to_owned(),
                    (LIGHTING, "ambient") if v < (1, 26, 0) => AMBIENT.to_owned(),
                    (GRADING, "color_grading") if v < (1, 21, 90) => {
                        format!(r#"{{"temperature": {TEMPERATURE}}}"#)
                    }
                    (GRADING, "color_grading/temperature") if v < (1, 21, 90) => {
                        TEMPERATURE.to_owned()
                    }
                    _ => String::new(),
                };
                (!inserted.is_empty() && *added == Json::of_text(&inserted))
                    .then_some((Class::StepInserted, None))
            }
            _ => None,
        };
        match class {
            Some((class, finding)) => self.count(
                class,
                finding.map(|(severity, code)| (severity, code, at.as_str())),
            ),
            None => self
                .unrecorded
                .push((sketch(difference), format!("{:?}", difference.change))),
        }
    }
}

/// The class of a number member written as another number or another token.
#[expect(
    clippy::float_cmp,
    reason = "an integer and a double are the same number when equal"
)]
fn number_class(before: &Number, after: &Number) -> Option<Class> {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the nearest f32 is the question"
    )]
    let float = before.real as f32;
    if let Some(int) = after.int {
        #[allow(clippy::cast_precision_loss, reason = "channels and small integers")]
        let same = before.real == int as f64;
        if same {
            Some(Class::ShortestInteger)
        } else if (int == 255 && before.real > 255.0) || (int == 0 && before.real < 0.0) {
            Some(Class::ChannelClamped)
        } else {
            None
        }
    } else if float.is_infinite() {
        (after.token == format!("{:?}", f32::MAX.copysign(float))).then_some(Class::BeyondAFloat)
    } else {
        (after.token == format!("{float:?}")).then_some(Class::ShortestFloat)
    }
}

/// A difference without what is particular to its document: the path with keyframe keys,
/// indices and the generator's unknown members named alike, and the kinds of the values.
fn sketch(difference: &Difference) -> String {
    let path: Vec<String> = difference
        .path
        .iter()
        .map(|step| match step {
            Step::Key(key) if written_key(key).is_some() || NO_TIMES.contains(&key.as_str()) => {
                "<time>".to_owned()
            }
            Step::Key(key) if ["zz", "note", "extra"].contains(&key.as_str()) => {
                "<unknown>".to_owned()
            }
            Step::Key(key) => key.clone(),
            Step::Index(_) => "#".to_owned(),
        })
        .collect();
    let kind = |value: &Json| match value {
        Json::Null => "null",
        Json::Bool(_) => "a boolean",
        Json::Num(number) if number.is_int() => "an integer",
        Json::Num(_) => "a real",
        Json::Str(_) => "a string",
        Json::Arr(_) => "an array",
        Json::Obj(_) => "an object",
    };
    let what = match &difference.change {
        Change::Removed(value) => format!("{} removed", kind(value)),
        Change::Added(value) => format!("{} added", kind(value)),
        Change::Replaced(before, after) => format!("{} for {}", kind(after), kind(before)),
        Change::Respelt(..) => "respelt".to_owned(),
        Change::Reordered => "reordered".to_owned(),
    };
    format!("{}: {what}", path.join("/"))
}

/// What became of the documents of one payload.
#[derive(Debug, Default, PartialEq)]
struct Tally {
    /// Refused by the loader.
    rejected: usize,
    /// Loaded, written, read back equal and rewritten identically.
    written: usize,
    /// Of the written ones: equal to their input as JSON, member for member.
    as_authored: usize,
    /// Loaded and refused by the writer, by the decision that records why.
    refused: BTreeMap<&'static str, usize>,
    /// The differences of the written documents from their inputs, by class.
    classes: BTreeMap<Class, usize>,
}

/// Compares a written document with its input as JSON values and counts each difference by
/// its class. A difference of no class is returned, with its document.
fn compare<T: Content>(
    key: &str,
    text: &str,
    written: &str,
    document: &Document<T>,
    tally: &mut Tally,
) -> Vec<(String, String)> {
    // What every loader reads: of a member written twice, the last.
    let cleaned = bedrock_addon::json::last_wins(text);
    let mut input = Json::of_text(&cleaned);
    let output = Json::of_text(written);
    let version = match input.get("format_version") {
        Some(Json::Str(version)) => version_of(version).expect("a loaded version"),
        other => panic!("loaded with the version {other:?}\nfrom: {text}"),
    };
    let mut compared = Compared {
        key,
        version,
        findings: document
            .diagnostics()
            .iter()
            .map(|finding| {
                (
                    finding.severity,
                    finding.code.name(),
                    finding.location.json_path.to_string(),
                )
            })
            .collect(),
        moved: Vec::new(),
        classes: Vec::new(),
        unrecorded: Vec::new(),
    };
    if cleaned != text {
        compared.classes.push(Class::MemberTwice);
    }
    compared.expect(&mut input);
    compared.follow_the_moves();
    let found = differences(&input, &output);
    for difference in &found {
        compared.classify(difference);
    }
    if compared.classes.is_empty() && compared.unrecorded.is_empty() {
        assert_eq!(Json::of_text(text), output, "from: {text}");
        tally.as_authored += 1;
    }
    for class in compared.classes {
        *tally.classes.entry(class).or_default() += 1;
    }
    compared.unrecorded
}

/// Generates the payload's documents and holds each to the round trip, and each written one
/// to its input.
fn round_trips<T>(key: &str, seed: u64, generate: impl Fn(&mut Rng) -> String) -> Tally
where
    T: Content + std::fmt::Debug + PartialEq,
{
    let mut rng = Rng(seed);
    let mut tally = Tally::default();
    // The differences of no recorded class: how many of each sketch, and the first of them.
    let mut unrecorded: BTreeMap<String, (usize, String)> = BTreeMap::new();
    for _ in 0..DOCUMENTS {
        let text = generate(&mut rng);
        let Ok(document) = parse::<T>(&text) else {
            tally.rejected += 1;
            continue;
        };
        match document.to_json() {
            Ok(written) => {
                let again = match parse::<T>(&written) {
                    Ok(again) => again,
                    Err(rejected) => panic!(
                        "written text does not load\nfrom: {text}\nwritten: {written}\n{:#?}",
                        rejected.diagnostics
                    ),
                };
                assert_eq!(
                    again.value(),
                    document.value(),
                    "from: {text}\nwritten: {written}"
                );
                assert_eq!(again.to_json().expect("rewritten"), written, "from: {text}");
                tally.written += 1;
                for (sketch, detail) in compare(key, &text, &written, &document, &mut tally) {
                    let entry = unrecorded.entry(sketch).or_default();
                    entry.0 += 1;
                    if entry.1.is_empty() {
                        entry.1 = format!(
                            "{detail}\n    from: {text}\n    written: {written}\n    findings: {:?}",
                            found(document.diagnostics(), Severity::Warning)
                                .iter()
                                .chain(&found(document.diagnostics(), Severity::Info))
                                .map(|(code, path, _)| format!("{code} {path}"))
                                .collect::<Vec<_>>()
                        );
                    }
                }
            }
            Err(EncodeError::NotRepresentable { at, .. }) => {
                let at = at.to_string();
                assert!(!at.is_empty(), "a refusal without a path\nfrom: {text}");
                let Some(class) = refusal(key, &at) else {
                    panic!("a refusal of no recorded class at {at}\nfrom: {text}");
                };
                *tally.refused.entry(class).or_default() += 1;
            }
            Err(other) => panic!("{other:?}\nfrom: {text}"),
        }
    }
    eprintln!("{key}: {DOCUMENTS} generated, {tally:#?}");
    if !unrecorded.is_empty() {
        let mut report = String::new();
        for (sketch, (count, first)) in unrecorded.iter().take(40) {
            writeln!(report, "{count:5} {sketch}\n    {first}").unwrap();
        }
        panic!(
            "{key}: {} kinds of difference between a written document and its input that no \
             decision records:\n{report}",
            unrecorded.len()
        );
    }
    tally
}

/// The tally a payload must have: `rejected` of its documents refused by the loader, `refused`
/// of the loaded ones refused by the writer for their missing `description`, every
/// other one written; `as_authored` of those equal to their input, the others differing from
/// it by `classes` alone.
fn tally(rejected: usize, refused: usize, as_authored: usize, classes: &[(Class, usize)]) -> Tally {
    Tally {
        rejected,
        written: DOCUMENTS - rejected - refused,
        as_authored,
        refused: if refused == 0 {
            BTreeMap::new()
        } else {
            BTreeMap::from([("description-required", refused)])
        },
        classes: classes.iter().copied().collect(),
    }
}

// The counts are those of the seeds: they move only with the generator, with what the loader
// accepts or with what the writer writes, and then they are looked at. Every class is counted
// by difference (a document may have several of one class), but `MemberTwice`, by document.
// Not among them, because the document does not load: `format_version` as an array (the
// envelope takes it, the struct's string member refuses it: validation.md E5a), a root member
// written twice (the envelope counts the root's members as written: E2 to E5), a ranged number
// beyond `f32`.

#[test]
fn generated_lighting_documents_are_written_as_authored_but_for_the_recorded_classes() {
    let found = round_trips::<LightingSettings>(LIGHTING, 0x11, lighting);
    let expected = tally(
        2_364,
        0,
        17,
        &[
            (Class::VersionRestamped, 1_334),
            (Class::VersionRespelt, 28),
            (Class::MemberOrder, 1_836),
            (Class::MemberTwice, 535),
            (Class::NumberRespelt, 163),
            (Class::WideIntegerAsReal, 65),
            (Class::BeyondADouble, 198),
            (Class::ShortestFloat, 3_913),
            (Class::ShortestInteger, 182),
            (Class::BeyondAFloat, 56),
            (Class::ChannelClamped, 274),
            (Class::ColourAsArray, 1_069),
            (Class::KeyRespelt, 1_669),
            (Class::KeyOutsideTheDay, 83),
            (Class::KeyOfOneTimeTwice, 330),
            (Class::NewerMemberDropped, 118),
            (Class::StepDroppedAlpha, 222),
            (Class::StepMoved, 650),
            (Class::StepInserted, 1_665),
            (Class::StepWrapped, 1_707),
        ],
    );
    assert_eq!(found, expected);
}

#[test]
fn generated_water_documents_are_written_as_authored_but_for_the_recorded_classes() {
    let found = round_trips::<WaterSettings>(WATER, 0x22, water);
    let expected = tally(
        2_488,
        51,
        9,
        &[
            (Class::VersionRestamped, 1_185),
            (Class::MemberOrder, 1_551),
            (Class::MemberTwice, 266),
            (Class::NumberRespelt, 85),
            (Class::WideIntegerAsReal, 36),
            (Class::BeyondADouble, 135),
            (Class::ShortestFloat, 6_733),
            (Class::ShortestInteger, 21),
            (Class::NewerMemberDropped, 63),
            (Class::StepDropped, 204),
        ],
    );
    assert_eq!(found, expected);
}

#[test]
fn generated_atmosphere_documents_are_written_as_authored_but_for_the_recorded_classes() {
    let found = round_trips::<AtmosphereSettings>(ATMOSPHERE, 0x33, atmosphere);
    let expected = tally(
        2_873,
        30,
        2,
        &[
            (Class::VersionRestamped, 560),
            (Class::MemberOrder, 1_926),
            (Class::MemberTwice, 396),
            (Class::NumberRespelt, 24),
            (Class::WideIntegerAsReal, 18),
            (Class::BeyondADouble, 79),
            (Class::ShortestFloat, 5_640),
            (Class::ShortestInteger, 176),
            (Class::ChannelClamped, 236),
            (Class::ColourAsArray, 894),
            (Class::KeyRespelt, 3_548),
            (Class::KeyOutsideTheDay, 197),
            (Class::KeyOfOneTimeTwice, 756),
        ],
    );
    assert_eq!(found, expected);
}

#[test]
fn generated_colour_grading_documents_are_written_as_authored_but_for_the_recorded_classes() {
    let found = round_trips::<ColorGradingSettings>(GRADING, 0x44, grading);
    let expected = tally(
        2_478,
        0,
        181,
        &[
            (Class::VersionRestamped, 780),
            (Class::MemberOrder, 1_099),
            (Class::MemberTwice, 337),
            (Class::NumberRespelt, 111),
            (Class::WideIntegerAsReal, 50),
            (Class::BeyondADouble, 156),
            (Class::ShortestFloat, 7_121),
            (Class::Vec3AsArray, 764),
            (Class::NewerMemberDropped, 19),
            (Class::StepInserted, 780),
        ],
    );
    assert_eq!(found, expected);
}

#[test]
fn generated_cubemap_documents_are_written_as_authored_but_for_the_recorded_classes() {
    let found = round_trips::<CubemapSettings>(CUBEMAP, 0x55, cubemap);
    let expected = tally(
        1_728,
        0,
        330,
        &[
            (Class::MemberOrder, 1_719),
            (Class::MemberTwice, 425),
            (Class::NumberRespelt, 103),
            (Class::WideIntegerAsReal, 54),
            (Class::BeyondADouble, 214),
            (Class::ShortestFloat, 4_183),
            (Class::KeyRespelt, 935),
            (Class::KeyOutsideTheDay, 50),
            (Class::KeyOfOneTimeTwice, 229),
        ],
    );
    assert_eq!(found, expected);
}

#[test]
fn generated_shadow_documents_are_written_as_authored_but_for_the_recorded_classes() {
    let found = round_trips::<ShadowSettings>(SHADOWS, 0x66, shadows);
    let expected = tally(
        1_335,
        0,
        1_672,
        &[
            (Class::MemberOrder, 771),
            (Class::MemberTwice, 215),
            (Class::NumberRespelt, 52),
            (Class::WideIntegerAsReal, 21),
            (Class::BeyondADouble, 52),
            (Class::ShortestInteger, 99),
        ],
    );
    assert_eq!(found, expected);
}

#[test]
fn generated_pbr_documents_are_written_as_authored_but_for_the_recorded_classes() {
    let found = round_trips::<PbrFallbackSettings>(PBR, 0x77, pbr);
    let expected = tally(
        1_580,
        0,
        227,
        &[
            (Class::MemberOrder, 1_228),
            (Class::MemberTwice, 551),
            (Class::NumberRespelt, 250),
            (Class::WideIntegerAsReal, 98),
            (Class::BeyondADouble, 195),
            (Class::ShortestInteger, 503),
            (Class::ChannelClamped, 798),
            (Class::ColourAsArray, 2_449),
        ],
    );
    assert_eq!(found, expected);
}

#[test]
fn generated_local_light_documents_are_written_as_authored_but_for_the_recorded_classes() {
    let found = round_trips::<LocalLightSettings>(LOCAL, 0x88, local_lights);
    let expected = tally(
        1_492,
        0,
        659,
        &[
            (Class::MemberOrder, 976),
            (Class::MemberTwice, 1_038),
            (Class::NumberRespelt, 121),
            (Class::WideIntegerAsReal, 58),
            (Class::BeyondADouble, 84),
            (Class::ShortestInteger, 276),
            (Class::ChannelClamped, 386),
            (Class::ColourAsArray, 1_658),
            (Class::BlockOfTwoSpellings, 272),
        ],
    );
    assert_eq!(found, expected);
}
