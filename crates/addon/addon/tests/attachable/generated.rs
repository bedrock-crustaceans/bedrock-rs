//! The generated round trip: a few thousand attachable documents
//! from a fixed seed, over every member's forms at the three structs. Every document that
//! loads writes text that loads again to an equal value and writes identically, or is refused
//! with `NotRepresentable` and a path. And every one is written as it was authored: its text
//! equals the source's as JSON, member for member, but for the recorded differences, each
//! applied by name below (the version stamp, the spelling of a version that is not stamped
//! and the two things the 1.10 step does to script text among them, so the older shapes are
//! compared too).

use bedrock_addon::document::{Document, JsonDocument as _};
use bedrock_addon::families::attachable::Attachable;
use bedrock_addon::families::attachable::history::AttachableHistory;
use bedrock_addon::json::EncodeError;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::versioning::VersionStruct;

use super::common::*;
use super::generate::{Canon, J, Rng, WIDE_FOR_A_STRING, as_f32_written, string_map};

/// Which struct reads a generated document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    V1_8,
    V1_10,
    V1_26,
}

const DESCRIPTION: [&str; 2] = ["minecraft:attachable", "description"];
const SCRIPTS: [&str; 3] = ["minecraft:attachable", "description", "scripts"];

const MOLANG: [&str; 6] = [
    "1.5",
    "query.is_baby ? 0.5 : 1.0",
    "Query.Is_Baby ? 0.5 : 1.0",
    "variable.x + Math.sin(query.life_time * 20.0)",
    "query.variant == 2",
    "q.is_moving",
];
/// A statement in mixed case; no other generated text holds it.
const MIXED_CASE: &str = "Variable.Y = Query.Is_Baby;";
const STATEMENTS: [&str; 5] = [
    "variable.x = 1;",
    MIXED_CASE,
    "variable.z = 'Hello World';",
    "v.w = math.sin(query.life_time * 20.0);",
    "",
];

/// Numbers no `f32` holds exactly: read as the nearest float and written as that float's
/// token (`0.12345679`, `16777216.0`, `0`).
const LONG_DECIMAL: &str = "0.123456789012";
const ODD_INTEGER: &str = "16777217";
const TINY: &str = "1e-50";

/// Integer tokens beyond 64 bits, generated where a number is declared (a Molang member,
/// `held_item_scale`, an element of `min_engine_version`): each is the number it spells
/// 23 digits, within `f32`; 39 digits between `f32::MAX` and `u128::MAX`; 400
/// digits, beyond a double.
const WIDE: &str = "99999999999999999999999";
const WIDE_ABOVE_F32: &str = "340282360000000000000000000000000000000";
const WIDE_BEYOND_A_DOUBLE: &str = concat!(
    "99999999999999999999999999999999999999999999999999",
    "99999999999999999999999999999999999999999999999999",
    "99999999999999999999999999999999999999999999999999",
    "99999999999999999999999999999999999999999999999999",
    "99999999999999999999999999999999999999999999999999",
    "99999999999999999999999999999999999999999999999999",
    "99999999999999999999999999999999999999999999999999",
    "99999999999999999999999999999999999999999999999999",
);
const WIDE_NUMBERS: [&str; 3] = [WIDE, WIDE_ABOVE_F32, WIDE_BEYOND_A_DOUBLE];

/// A Molang member: an expression, a number of either tag, a boolean; rarely a text the
/// parser rejects or a wrong type.
fn molang(rng: &mut Rng) -> J {
    match rng.below(100) {
        0..=1 => J::str("1 +"),
        2 => J::Arr(Vec::new()),
        3..=14 => J::Num(rng.pick(&[
            "1.5",
            "2",
            "2.0",
            "0",
            "-0.25",
            "1e39",
            "-1e999",
            // No `f32` is exactly these: a decimal longer than a float holds, an
            // integer beyond 2^24, a number below the smallest float.
            LONG_DECIMAL,
            ODD_INTEGER,
            TINY,
            WIDE,
            WIDE_ABOVE_F32,
            WIDE_BEYOND_A_DOUBLE,
        ])),
        15..=20 => J::Bool(rng.chance(50)),
        _ => J::str(rng.pick(&MOLANG)),
    }
}

/// A list of script statements; before 1.26.0 an element may be a number or a boolean.
fn statements(rng: &mut Rng, shape: Shape) -> J {
    if rng.chance(2) {
        return J::str("variable.x = 1;");
    }
    let other = if shape == Shape::V1_26 { 2 } else { 20 };
    J::Arr(
        (0..rng.below(4))
            .map(|_| {
                if rng.chance(other) {
                    rng.pick(&[J::Num("1"), J::Num("1.5"), J::Bool(true), J::Num("1e999")])
                        .clone()
                } else if rng.chance(3) {
                    // A text the parser rejects.
                    J::str("variable.x = (;")
                } else {
                    J::str(rng.pick(&STATEMENTS))
                }
            })
            .collect(),
    )
}

/// One `{ name: condition }` object, rarely with two members or none.
fn conditional(rng: &mut Rng, names: &[&str]) -> J {
    match rng.below(100) {
        0..=1 => J::Obj(Vec::new()),
        2..=9 => J::obj(&[("b.second", molang(rng)), ("a.first", molang(rng))]),
        _ => J::obj(&[(rng.pick(names), molang(rng))]),
    }
}

fn scripts(rng: &mut Rng, shape: Shape) -> Option<J> {
    if rng.chance(25) {
        return None;
    }
    let mut members: Vec<(String, J)> = Vec::new();
    if rng.chance(5) {
        return Some(J::Obj(members));
    }
    let newer = shape != Shape::V1_8;
    // A member its struct does not declare is generated rarely: the file is refused.
    let declared = |rng: &mut Rng, is: bool| if is { true } else { rng.chance(2) };
    if rng.chance(20) && declared(rng, newer) {
        let variables = match rng.below(20) {
            0 => J::Obj(Vec::new()),
            1 => J::obj(&[("variable.x", J::str("private"))]),
            2 => J::obj(&[("not_a_variable", J::str("public"))]),
            _ => J::obj(&[
                ("variable.x", J::str("public")),
                ("variable.Foo_1", J::str("public")),
            ]),
        };
        members.push(("variables".to_owned(), variables));
    }
    if rng.chance(40) && declared(rng, newer) {
        members.push(("initialize".to_owned(), statements(rng, shape)));
    }
    if rng.chance(40) {
        members.push(("pre_animation".to_owned(), statements(rng, shape)));
    }
    for name in [
        "scale",
        "scaleX",
        "scalex",
        "scaleY",
        "scaley",
        "scaleZ",
        "scalez",
        "parent_setup",
    ] {
        if rng.chance(12) {
            members.push((name.to_owned(), molang(rng)));
        }
    }
    for name in [
        "should_update_bones_and_effects_offscreen",
        "should_update_effects_offscreen",
    ] {
        if rng.chance(12) && declared(rng, newer) {
            members.push((name.to_owned(), molang(rng)));
        }
    }
    if rng.chance(10) && declared(rng, shape == Shape::V1_26) {
        members.push(("hide_held_items".to_owned(), molang(rng)));
    }
    if rng.chance(35) && declared(rng, newer) {
        let entries = (0..rng.below(4))
            .map(|_| match rng.below(100) {
                0 => J::Num("3"),
                1..=35 => conditional(rng, &["walk", "Walk", "demo:walk"]),
                _ => J::str(rng.pick(&["walk", "Walk", "move", "controller__move", "missing"])),
            })
            .collect();
        members.push(("animate".to_owned(), J::Arr(entries)));
    }
    Some(J::Obj(members))
}

fn spawn_egg(rng: &mut Rng) -> Option<J> {
    if rng.chance(55) {
        return None;
    }
    let mut members: Vec<(String, J)> = Vec::new();
    if rng.chance(50) {
        members.push(("texture".to_owned(), J::str("spawn_egg")));
    }
    if rng.chance(50) {
        let index = match rng.below(50) {
            0 => J::Num("1.5"),
            1 => J::str("3"),
            _ => J::Num(rng.pick(&["0", "3", "40", "-1"])),
        };
        members.push(("texture_index".to_owned(), index));
    }
    if rng.chance(40) {
        members.push((
            "base_color".to_owned(),
            J::str(rng.pick(&["#53443E", "#fff", "red"])),
        ));
    }
    if rng.chance(40) {
        members.push(("overlay_color".to_owned(), J::str("#2e6854")));
    }
    Some(J::Obj(members))
}

fn held_item_scale(rng: &mut Rng) -> J {
    match rng.below(100) {
        0 => J::Bool(true),
        1 => J::Arr(vec![J::Num("1"), J::Num("2")]),
        2..=4 => J::str("1 +"),
        5..=25 => J::str(rng.pick(&MOLANG)),
        _ => J::Num(rng.pick(&[
            "1.5",
            "2",
            "2.678",
            "0.1",
            "0",
            "0.0",
            "-1",
            "-0.5",
            "16777217",
            "3.4028235e38",
            "1e-50",
            "1e39",
            "-1e39",
            "1e999",
            WIDE,
            WIDE_ABOVE_F32,
            WIDE_BEYOND_A_DOUBLE,
        ])),
    }
}

/// `item`: the string form, the object form with one or two members, and rarely a shape the
/// schema refuses.
fn item(rng: &mut Rng) -> J {
    match rng.below(100) {
        0 => J::Obj(Vec::new()),
        1 => J::Num(rng.pick(&["5", WIDE_FOR_A_STRING])),
        2 => J::Null,
        3 => J::Arr(Vec::new()),
        4 => J::obj(&[("demo:wand sword", J::str("1.0"))]),
        5 => J::obj(&[("demo:wand", J::Null)]),
        6 => J::obj(&[("demo:a", J::str("1.0")), ("demo:b", J::Obj(Vec::new()))]),
        7..=9 => J::str(rng.pick(&["demo:wand sword", "", "wand!"])),
        10..=40 => J::str(rng.pick(&["demo:wand", "Minecraft:Bow", "bow", "a:b:c"])),
        41..=55 => J::obj(&[("demo:b", molang(rng)), ("demo:a", molang(rng))]),
        _ => J::obj(&[(
            *rng.pick(&["demo:wand", "Demo:Wand", "minecraft:diamond_helmet"]),
            molang(rng),
        )]),
    }
}

/// The `animation_controllers` member, when the file has one.
fn animation_controllers(rng: &mut Rng, shape: Shape) -> Option<J> {
    if !(rng.chance(40) && (shape == Shape::V1_8 || rng.chance(3))) {
        return None;
    }
    let controllers: Vec<J> = (0..rng.below(3))
        .map(|_| {
            if rng.chance(25) {
                J::obj(&[
                    ("b", J::str("controller.animation.ghost.b")),
                    ("a", J::str("controller.animation.ghost.a")),
                ])
            } else {
                J::obj(&[(
                    *rng.pick(&["move", "Look"]),
                    J::str("controller.animation.ghost.move"),
                )])
            }
        })
        .collect();
    Some(J::Arr(controllers))
}

/// The `render_controllers` member, when the file has one.
fn render_controllers(rng: &mut Rng, newer: bool) -> Option<J> {
    if !rng.chance(65) {
        return None;
    }
    let names = ["controller.render.ghost", "Controller.Render.Ghost"];
    let controllers: Vec<J> = (0..rng.below(4))
        .map(|_| {
            let object = if newer { 30 } else { 2 };
            if rng.chance(object) {
                conditional(rng, &names)
            } else {
                J::str(rng.pick(&names))
            }
        })
        .collect();
    Some(J::Arr(controllers))
}

/// One attachable document for `shape`, as text.
fn document(rng: &mut Rng, shape: Shape) -> String {
    let newer = shape != Shape::V1_8;
    let mut d: Vec<(String, J)> = Vec::new();
    let mut add = |name: &str, value: Option<J>| {
        if let Some(value) = value {
            d.push((name.to_owned(), value));
        }
    };
    let identifier = match rng.below(100) {
        0 => J::str(""),
        1 => J::Num(rng.pick(&["5", WIDE_FOR_A_STRING])),
        2 => return shaped(rng, shape, J::Obj(Vec::new())),
        _ => J::str(rng.pick(&[
            "demo:ghost",
            "Demo:Ghost",
            "minecraft:PIG",
            "ghost",
            "a:b:c",
        ])),
    };
    add("identifier", Some(identifier));
    let min_engine_version = match rng.below(100) {
        0 => Some(J::Num("5")),
        1..=8 => Some(J::str(rng.pick(&["1.8.0", "1.16.0", "1.26.0", "nonsense"]))),
        9..=14 => Some(J::Arr(vec![J::Num("1"), J::Num("16"), J::Num("0")])),
        // Kept as written, a double: the number the token spells.
        15..=16 => Some(J::Arr(vec![J::Num(WIDE), J::Num("0"), J::Num("0")])),
        _ => None,
    };
    add("min_engine_version", min_engine_version);
    let materials = ["entity_alphatest", "Entity_AlphaTest", "ghost"];
    add("materials", string_map(rng, 35, &materials));
    let textures = ["textures/entity/ghost", "Textures/Entity/Ghost"];
    add("textures", string_map(rng, 35, &textures));
    let geometry = ["geometry.ghost", "Geometry.Ghost.V2"];
    add("geometry", string_map(rng, 35, &geometry));
    if rng.chance(15) && (newer || rng.chance(5)) {
        add(
            "queryable_geometry",
            Some(J::str(rng.pick(&["default", "Default"]))),
        );
    }
    add("spawn_egg", spawn_egg(rng));
    add("scripts", scripts(rng, shape));
    let particles = ["minecraft:flame", "Demo:Spark"];
    add("particle_effects", string_map(rng, 60, &particles));
    add("particle_emitters", string_map(rng, 85, &particles));
    // With and without the 21-character controller prefix, in either case.
    let animations = [
        "animation.ghost.walk",
        "Animation.Ghost.Walk",
        "controller.animation.ghost.move",
        "Controller.Animation.Ghost.Move",
        "controller.animationx",
        "other",
    ];
    add("animations", string_map(rng, 35, &animations));
    add("animation_controllers", animation_controllers(rng, shape));
    add("render_controllers", render_controllers(rng, newer));
    let sounds = ["mob.ghost.say", "Mob.Ghost.Hurt"];
    add("sound_effects", string_map(rng, 60, &sounds));
    for name in [
        "enable_attachables",
        "held_item_ignores_lighting",
        "hide_armor",
    ] {
        if rng.chance(25) {
            let flag = if rng.chance(2) {
                J::Num("1")
            } else {
                J::Bool(rng.chance(50))
            };
            add(name, Some(flag));
        }
    }
    if rng.chance(40) && (shape == Shape::V1_26 || rng.chance(3)) {
        add("held_item_scale", Some(held_item_scale(rng)));
    }
    // `item` is a member from 1.10.0; at 1.8.0 it is generated rarely, and refused.
    if rng.chance(45) && (newer || rng.chance(4)) {
        add("item", Some(item(rng)));
    }
    // A `null` where a member is declared: refused, never read as absent.
    if rng.chance(2) {
        let member = *rng.pick(&[
            "scripts",
            "animations",
            "hide_armor",
            "spawn_egg",
            "sound_effects",
            "min_engine_version",
        ]);
        add(member, Some(J::Null));
    }
    if rng.chance(1) {
        add("extra", Some(J::Num("1")));
    }
    shaped(rng, shape, J::Obj(d))
}

/// A `format_version` as generated: its three numbers and how it is spelled.
#[derive(Clone, Copy)]
enum Spelling {
    /// `"1.26.0"`.
    Full,
    /// `"1.26"`: the patch left out (written for a version whose patch is 0).
    Short,
    /// `[1, 26, 0]`.
    Array,
}

/// The versions each struct reads, a patch of 0 first.
const V1_8: [[u32; 3]; 2] = [[1, 8, 0], [1, 9, 0]];
const V1_10: [[u32; 3]; 7] = [
    [1, 10, 0],
    [1, 21, 0],
    [1, 16, 100],
    [1, 18, 10],
    [1, 18, 20],
    [1, 25, 99],
    [1, 12, 0],
];
const V1_26: [[u32; 3]; 3] = [[1, 26, 0], [1, 26, 30], [1, 26, 50]];

/// `version` in one of its spellings: mostly in full, as vanilla's files have it.
fn format_version(rng: &mut Rng, version: [u32; 3]) -> J {
    let [major, minor, patch] = version;
    let spelling = match rng.below(100) {
        0..=14 if patch == 0 => Spelling::Short,
        15..=29 => Spelling::Array,
        _ => Spelling::Full,
    };
    match spelling {
        Spelling::Full => J::Str(format!("{major}.{minor}.{patch}")),
        Spelling::Short => J::Str(format!("{major}.{minor}")),
        Spelling::Array => J::Arr(
            version
                .iter()
                .map(|part| J::Int(i64::from(*part)))
                .collect(),
        ),
    }
}

/// The document of `description` at a `format_version` its shape's struct reads, in any of
/// the version's spellings.
fn shaped(rng: &mut Rng, shape: Shape, description: J) -> String {
    let version = match shape {
        Shape::V1_8 if rng.chance(33) => None,
        Shape::V1_8 => Some(*rng.pick(&V1_8)),
        Shape::V1_10 => Some(*rng.pick(&V1_10)),
        Shape::V1_26 => Some(*rng.pick(&V1_26)),
    };
    let mut root: Vec<(String, J)> = Vec::new();
    if let Some(version) = version {
        root.push(("format_version".to_owned(), format_version(rng, version)));
    }
    root.push((
        "minecraft:attachable".to_owned(),
        J::obj(&[("description", description)]),
    ));
    J::Obj(root).text()
}

/// How many documents the test generates, a third for each struct.
const GENERATED: usize = 3000;

/// What the run saw.
#[derive(Debug, Default)]
struct Counts {
    generated: usize,
    loaded: usize,
    round_tripped: usize,
    refused: usize,
    /// As-written documents (`parse_as_written`) written and read back.
    as_written: usize,
    rejected: usize,
    /// Documents whose written text was compared with the source.
    compared: usize,
    /// Of those, the ones written at a newer version than they were read at.
    restamped: usize,
    /// Of the restamped ones, those whose version was spelled short and as an array, by the
    /// era of the struct they were read as: `"1.8"` and `[1, 8, 0]`, `"1.10"` and `[1, 10, 0]`.
    v1_8_versions_short: usize,
    v1_10_versions_short: usize,
    v1_8_versions_as_arrays: usize,
    v1_10_versions_as_arrays: usize,
    /// Documents that were not upgraded and whose version is written in another spelling
    /// than the author's: `"1.26"` and `[1, 26, 0]`, both written `"1.26.0"`.
    short_versions_in_full: usize,
    array_versions_as_strings: usize,
    /// Compared documents with an `item`, with a binding, and with one that never holds.
    items: usize,
    bound: usize,
    never: usize,
    empty_maps_kept: usize,
    scales_rewritten: usize,
    scales_clamped: usize,
    /// Molang constants beyond `f32`, written as the largest finite float.
    constants_clamped: usize,
    scripts_dropped: usize,
    constants_as_integers: usize,
    /// Molang constants no `f32` is exactly, written as the nearest float: a long
    /// decimal, an integer beyond 2^24, a number below the smallest float.
    constants_narrowed: usize,
    constants_odd_integers: usize,
    constants_tiny: usize,
    /// Documents with an integer token beyond 64 bits where a string is declared: each is
    /// refused.
    wide_where_a_string_is: usize,
    /// Compared documents with such a token where a number is declared: written as a number.
    wide_numbers: usize,
    lower_cased: usize,
    elements_dropped: usize,
}

/// A written text must load, without an error.
#[track_caller]
fn reload(written: &str, cx: &LoadContext, source: &str) -> Document<Attachable> {
    match attachable(written, cx) {
        Ok(document) => document,
        Err(rejected) => panic!(
            "to_json gave text this loader refuses:\n{}\nwritten: {written}\nsource: {source}",
            lines(&rejected.diagnostics).join("\n")
        ),
    }
}

/// The three numbers of a generated `format_version`, in any of its spellings; `None`
/// without the member.
fn version_of(source: &mut Canon) -> Option<(i64, i64, i64)> {
    let parts: Vec<i64> = match source.at_mut(&["format_version"])? {
        Canon::Str(version) => version
            .split('.')
            .map(|part| part.parse().unwrap())
            .collect(),
        Canon::Arr(parts) => parts
            .iter()
            .map(|part| match part {
                Canon::Int(part) => *part,
                other => panic!("a generated version holds {other:?}"),
            })
            .collect(),
        other => panic!("a generated version is {other:?}"),
    };
    Some((parts[0], parts[1], parts.get(2).copied().unwrap_or(0)))
}

/// Whether the document's named scripts are lower-cased when it loads: it has no
/// `format_version`, or one below 1.18.20.
fn folds(source: &mut Canon) -> bool {
    version_of(source).is_none_or(|version| version < (1, 18, 20))
}

/// The source of a document as its write-back must read: the text as authored, but for the
/// recorded differences, each applied by name.
fn expected(source: &str, document: &Document<Attachable>, counts: &mut Counts) -> Canon {
    let mut expected = Canon::of_text(source);
    let fold = folds(&mut expected);
    // A document is written in the newest shape, so an older one (or one
    // without a version) is stamped 1.26.0, with what its steps did.
    let upgraded = document.version().upgraded;
    // How the version was spelled, before it is stamped or spelled in full.
    let (short, array) = match expected.at_mut(&["format_version"]) {
        Some(Canon::Str(version)) => (version.split('.').count() == 2, false),
        Some(Canon::Arr(_)) => (false, true),
        _ => (false, false),
    };
    let version = version_of(&mut expected);
    if let Canon::Obj(root) = &mut expected {
        if upgraded {
            root.insert("format_version".to_owned(), Canon::Str("1.26.0".to_owned()));
            counts.restamped += 1;
            // The 1.8 struct reads every version below 1.10.0, the 1.10 struct the rest.
            if version.is_some_and(|version| version < (1, 10, 0)) {
                counts.v1_8_versions_short += usize::from(short);
                counts.v1_8_versions_as_arrays += usize::from(array);
            } else {
                counts.v1_10_versions_short += usize::from(short);
                counts.v1_10_versions_as_arrays += usize::from(array);
            }
        } else if let Some((major, minor, patch)) = version
            && (short || array)
        {
            // the model holds the version, not its spelling, and the
            // version is written in full, a string: `"1.26"` and `[1, 26, 0]` as `"1.26.0"`.
            let full = format!("{major}.{minor}.{patch}");
            root.insert("format_version".to_owned(), Canon::Str(full));
            counts.short_versions_in_full += usize::from(short);
            counts.array_versions_as_strings += usize::from(array);
        }
    }
    // An empty `animations` or `sound_effects` object is written as it was read (`materials`,
    // `textures` and `geometry` never reach here empty: the schema refuses them).
    for name in ["animations", "sound_effects"] {
        let at = [DESCRIPTION[0], DESCRIPTION[1], name];
        if matches!(expected.at_mut(&at), Some(Canon::Obj(members)) if members.is_empty()) {
            counts.empty_maps_kept += 1;
        }
    }
    let scale = ["minecraft:attachable", "description", "held_item_scale"];
    if let Some(number) = expected.number(&scale) {
        // The `f32` is written as the shortest number that reads back to it, where core would
        // widen it; the variant keeps no token kind, so an integer token is written as a real;
        // a number beyond `f32` is read as the largest finite float of its sign, with a
        // warning.
        let (written, clamped) = as_f32_written(number);
        *expected.at_mut(&scale).unwrap() = Canon::real(written);
        counts.scales_rewritten += 1;
        counts.scales_clamped += usize::from(clamped);
    }
    // The 1.10 step: an `initialize` / `pre_animation` element that is no string is dropped
    // with a note, and below 1.18.20 the text of every named script is lower-cased.
    if upgraded && let Some(Canon::Obj(scripts)) = expected.at_mut(&SCRIPTS) {
        for (name, member) in scripts.iter_mut() {
            match (name.as_str(), member) {
                ("variables" | "animate", _) => {}
                ("initialize" | "pre_animation", Canon::Arr(elements)) => {
                    let before = elements.len();
                    elements.retain(|element| matches!(element, Canon::Str(_)));
                    counts.elements_dropped += before - elements.len();
                    for element in elements {
                        counts.lower_cased += usize::from(lower_case(element, fold));
                    }
                }
                (_, member) => counts.lower_cased += usize::from(lower_case(member, fold)),
            }
        }
    }
    for name in ["initialize", "pre_animation"] {
        // a joined script text the parser rejects is dropped with "Failed to parse
        // '<name>'!" (with the `molang` feature); the member is not stored, so not written.
        let at = format!("minecraft:attachable/description/scripts/{name}");
        let dropped = document.diagnostics().iter().any(|found| {
            found.code.id() == "visual/failed_to_parse"
                && found.kept
                && found.location.json_path.to_string() == at
        });
        if dropped && expected.remove(&SCRIPTS, name) {
            counts.scripts_dropped += 1;
        }
    }
    // a Molang member that is a number is a constant, written as the token of the
    // `f32` it is: an integral one as an integer (`2.0` as `2`), one no `f32` is exactly as
    // the nearest float, one beyond `f32` as the largest. The Molang
    // members are the named scripts and the conditions of the object forms, `item`'s among
    // them.
    let mut constants = Constants::default();
    if let Some(Canon::Obj(scripts)) = expected.at_mut(&SCRIPTS) {
        for (name, member) in scripts.iter_mut() {
            match (name.as_str(), member) {
                ("variables" | "initialize" | "pre_animation", _) => {}
                ("animate", Canon::Arr(entries)) => {
                    conditions_as_constants(entries, &mut constants);
                }
                (_, member) => as_constant(member, &mut constants),
            }
        }
    }
    let controllers = [DESCRIPTION[0], DESCRIPTION[1], "render_controllers"];
    if let Some(Canon::Arr(entries)) = expected.at_mut(&controllers) {
        conditions_as_constants(entries, &mut constants);
    }
    let item = [DESCRIPTION[0], DESCRIPTION[1], "item"];
    if let Some(item) = expected.at_mut(&item) {
        counts.items += 1;
        conditions_as_constants(std::slice::from_mut(item), &mut constants);
    }
    counts.constants_as_integers += constants.as_integers;
    counts.constants_clamped += constants.clamped;
    counts.constants_narrowed += constants.narrowed;
    counts.constants_odd_integers += constants.odd_integers;
    counts.constants_tiny += constants.tiny;
    expected
}

/// A string lower-cased when `fold` is set; whether that changed it.
fn lower_case(member: &mut Canon, fold: bool) -> bool {
    let Canon::Str(text) = member else {
        return false;
    };
    let lower = text.to_ascii_lowercase();
    if !fold || lower == *text {
        return false;
    }
    *text = lower;
    true
}

/// What the Molang constants of a document were written as, where not as authored.
#[derive(Debug, Default)]
struct Constants {
    /// An integral real written as an integer (`2.0` as `2`).
    as_integers: usize,
    /// Beyond `f32`: the largest finite float of its sign.
    clamped: usize,
    /// Not exact in `f32`: a decimal narrowed (`0.123456789012` as `0.12345679`),
    /// an integer beyond 2^24 (`16777217` as `16777216.0`), a number below the smallest
    /// float (`1e-50` as `0`).
    narrowed: usize,
    odd_integers: usize,
    tiny: usize,
}

/// A number as the token core writes for the Molang constant it is read as (`ConstToken`):
/// the nearest `f32`, as an integer when it is integral and below 2^24 in magnitude,
/// else as the shortest real that reads back to it. A number beyond `f32` is the largest
/// finite float of its sign. Each difference from the authored token is counted.
#[expect(
    clippy::float_cmp,
    reason = "the float is the exact double the document spelled"
)]
fn as_constant(member: &mut Canon, constants: &mut Constants) {
    #[allow(
        clippy::cast_precision_loss,
        reason = "the generated integers are below 2^53"
    )]
    let (number, was_real) = match member {
        Canon::Real(bits) => (f64::from_bits(*bits), true),
        Canon::Int(int) => (*int as f64, false),
        _ => return,
    };
    let (written, beyond) = as_f32_written(number);
    if beyond {
        *member = Canon::real(written);
        constants.clamped += 1;
        return;
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the nearest f32 is the question"
    )]
    let float = number as f32;
    #[allow(clippy::float_cmp, reason = "whether the float is the number, exactly")]
    if f64::from(float) != number {
        if float == 0.0 {
            constants.tiny += 1;
        } else if number.fract() == 0.0 {
            constants.odd_integers += 1;
        } else {
            constants.narrowed += 1;
        }
    }
    let integral = float.fract() == 0.0 && float.abs() < 16_777_216.0;
    if integral && !(float == 0.0 && float.is_sign_negative()) {
        #[allow(clippy::cast_possible_truncation, reason = "integral and below 2^24")]
        let int = float as i64;
        *member = Canon::Int(int);
        constants.as_integers += usize::from(was_real && f64::from(float) == number);
    } else {
        *member = Canon::real(written);
    }
}

/// [`as_constant`] over the conditions of the `{ name: condition }` entries of a list.
fn conditions_as_constants(entries: &mut [Canon], constants: &mut Constants) {
    for entry in entries {
        if let Canon::Obj(members) = entry {
            for condition in members.values_mut() {
                as_constant(condition, constants);
            }
        }
    }
}

/// One generated document through the invariant.
fn check(source: &str, cx: &LoadContext, counts: &mut Counts) {
    counts.generated += 1;
    // The struct its version selects writes its own shape back, and that reads to the same
    // value.
    let path = PackPath::from_static(PATH);
    if let Ok(written) = Attachable::parse_as_written(&path, source.as_bytes(), cx) {
        let value = match written.value() {
            AttachableHistory::V1_8_0(attachable) => attachable.write(),
            AttachableHistory::V1_10_0(attachable) => attachable.write(),
            AttachableHistory::V1_26_0(attachable) => attachable.write(),
        };
        let text = bedrock_addon::json::to_json(&value.unwrap()).unwrap();
        let again =
            Attachable::parse_as_written(&path, text.as_bytes(), cx).unwrap_or_else(|rejected| {
                panic!(
                    "the as-written text is refused:\n{}\nwritten: {text}\nsource: {source}",
                    lines(&rejected.diagnostics).join("\n")
                )
            });
        assert_eq!(again.value(), written.value(), "{source}");
        counts.as_written += 1;
    }
    let loaded = attachable(source, cx);
    if source.contains(WIDE_FOR_A_STRING) {
        // A number where a string is declared, whatever its width.
        counts.wide_where_a_string_is += 1;
        assert!(
            loaded.is_err(),
            "an integer beyond 64 bits is read where a string is declared: {source}"
        );
    }
    let Ok(document) = loaded else {
        counts.rejected += 1;
        return;
    };
    counts.loaded += 1;
    let written = match document.to_json() {
        Ok(written) => written,
        Err(EncodeError::NotRepresentable { at, .. }) => {
            assert!(!at.is_empty(), "a refusal without a path: {source}");
            counts.refused += 1;
            return;
        }
        Err(other) => panic!("{other}: {source}"),
    };
    let again = reload(&written, cx, source);
    assert_eq!(again.value(), document.value(), "{source}\n{written}");
    assert_eq!(again.to_json().unwrap(), written, "{source}");
    // What the first load warned about and dropped or replaced is gone from the text: the
    // second load has nothing at error level.
    assert!(
        again.diagnostics().iter().all(|found| !found.rejects()),
        "{source}"
    );
    counts.round_tripped += 1;
    // As authored: the written text is the source, member for member, at every version.
    let expected = expected(source, &document, counts);
    assert_eq!(Canon::of_text(&written), expected, "{source}\n{written}");
    counts.compared += 1;
    // An integer beyond 64 bits in a number's place was read as a number, and a number is
    // written: nowhere its digits as a string.
    let wide: Vec<&str> = WIDE_NUMBERS
        .into_iter()
        .filter(|token| source.contains(token))
        .collect();
    for token in &wide {
        assert!(
            !written.contains(&format!("\"{token}\"")),
            "{source}\n{written}"
        );
    }
    counts.wide_numbers += usize::from(!wide.is_empty());
    counts.bound += usize::from(document.binding().is_some());
    counts.never += usize::from(document.binding().is_some_and(|(_, c)| c.is_never()));
}

#[test]
fn every_generated_document_that_loads_is_written_as_authored_or_refused_with_a_path() {
    let mut rng = Rng::new(0x00a7_7ac4_ab1e_5eed);
    let mut counts = Counts::default();
    let contexts = [rp(1, 21, 0), rp(1, 13, 0), rp(1, 26, 50)];
    for index in 0..GENERATED {
        let shape = [Shape::V1_8, Shape::V1_10, Shape::V1_26][index % 3];
        let source = document(&mut rng, shape);
        check(&source, &contexts[rng.below(3)], &mut counts);
    }
    eprintln!("generated attachables: {counts:?}");
    // The refusals are of the forms the generator writes wrong on purpose (a wrong type, an
    // integer beyond 64 bits where a string is declared, a missing member); 1387 of the 3000
    // load. Two in five is a floor below that count: it fails when a change of the generator
    // or of the loader leaves a stream that is mostly refused and tests little.
    // Every form of a recorded difference was met.
    assert_eq!(counts.generated, GENERATED);
    assert_eq!(counts.loaded + counts.rejected, GENERATED);
    assert!(counts.loaded * 5 > counts.generated * 2, "{counts:?}");
    assert!(counts.rejected * 10 > counts.generated, "{counts:?}");
    assert_eq!(counts.round_tripped + counts.refused, counts.loaded);
    // Nothing a load accepts is refused by the writer.
    assert_eq!(counts.refused, 0, "{counts:?}");
    assert_eq!(counts.compared, counts.round_tripped);
    assert!(counts.as_written >= counts.loaded, "{counts:?}");
    assert!(counts.restamped > 500, "{counts:?}");
    assert!(counts.compared - counts.restamped > 400, "{counts:?}");
    // Every spelling of the version at every struct, and the one recorded loss.
    assert!(counts.v1_8_versions_short > 10, "{counts:?}");
    assert!(counts.v1_10_versions_short > 10, "{counts:?}");
    assert!(counts.v1_8_versions_as_arrays > 10, "{counts:?}");
    assert!(counts.v1_10_versions_as_arrays > 10, "{counts:?}");
    assert!(counts.short_versions_in_full > 10, "{counts:?}");
    assert!(counts.array_versions_as_strings > 20, "{counts:?}");
    assert!(counts.items > 300, "{counts:?}");
    assert!(counts.bound > 300, "{counts:?}");
    assert!(counts.never > 0, "{counts:?}");
    assert!(counts.empty_maps_kept > 0, "{counts:?}");
    assert!(counts.scales_rewritten > 0, "{counts:?}");
    assert!(counts.scales_clamped > 0, "{counts:?}");
    assert!(counts.constants_clamped > 20, "{counts:?}");
    assert!(counts.constants_as_integers > 0, "{counts:?}");
    assert!(counts.constants_narrowed > 5, "{counts:?}");
    assert!(counts.constants_odd_integers > 5, "{counts:?}");
    assert!(counts.constants_tiny > 5, "{counts:?}");
    assert!(counts.wide_where_a_string_is > 20, "{counts:?}");
    assert!(counts.wide_numbers > 40, "{counts:?}");
    assert!(counts.lower_cased > 0, "{counts:?}");
    assert!(counts.elements_dropped > 0, "{counts:?}");
    #[cfg(feature = "molang")]
    assert!(counts.scripts_dropped > 0, "{counts:?}");
}
