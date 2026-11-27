//! The generated round trip: a few thousand client entity documents
//! from a fixed seed, over every member's forms at the three structs. Every document that
//! loads writes text that loads again to an equal value and writes identically, or is refused
//! with `NotRepresentable` and a path. A 1.26.0 document is also written as it was read: its
//! text equals the source's, but for the recorded differences, each applied by name below.

use bedrock_addon_core::diagnostic::{Disposition, Kind, Severity};
use bedrock_addon_core::document::JsonDocument as _;
use bedrock_addon_core::json::EncodeError;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::versioning::VersionStruct;
use bedrock_addon_family_client_entity::ClientEntity;
use bedrock_addon_family_client_entity::history::ClientEntityHistory;

use super::common::*;
use super::generate::{Canon, J, Rng, as_f32_written, string_map};

/// Which struct reads a generated document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    V1_8,
    V1_10,
    V1_26,
}

const DESCRIPTION: [&str; 2] = ["minecraft:client_entity", "description"];
const SCRIPTS: [&str; 3] = ["minecraft:client_entity", "description", "scripts"];

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

/// A Molang member: an expression, a number of either tag, a boolean; rarely a text the
/// parser rejects or a wrong type.
fn molang(rng: &mut Rng) -> J {
    match rng.below(100) {
        0..=1 => J::str("1 +"),
        2 => J::Arr(Vec::new()),
        3..=12 => J::Num(rng.pick(&["1.5", "2", "2.0", "0", "-0.25"])),
        13..=20 => J::Bool(rng.chance(50)),
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
                    rng.pick(&[J::Num("1"), J::Num("1.5"), J::Bool(true)])
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
        ])),
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

/// One client entity document for `shape`, as text.
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
        1 => J::Num("5"),
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
    if rng.chance(1) {
        add("extra", Some(J::Num("1")));
    }
    shaped(rng, shape, J::Obj(d))
}

/// The versions each struct reads.
const V1_8: [[u16; 3]; 2] = [[1, 8, 0], [1, 9, 0]];
const V1_10: [[u16; 3]; 6] = [
    [1, 10, 0],
    [1, 16, 100],
    [1, 18, 10],
    [1, 18, 20],
    [1, 21, 0],
    [1, 25, 99],
];
const V1_26: [[u16; 3]; 3] = [[1, 26, 0], [1, 26, 30], [1, 26, 50]];

/// A version number as a JSON number token.
fn number(part: u16) -> J {
    J::Num(match part {
        0 => "0",
        1 => "1",
        8 => "8",
        9 => "9",
        10 => "10",
        16 => "16",
        18 => "18",
        20 => "20",
        21 => "21",
        25 => "25",
        26 => "26",
        30 => "30",
        50 => "50",
        99 => "99",
        100 => "100",
        other => panic!("no token for {other}"),
    })
}

/// `version` in one of its spellings: mostly in full, as vanilla's files have it;
/// `"1.26"` for a version whose patch is 0; `[1, 26, 0]`.
fn format_version(rng: &mut Rng, version: [u16; 3]) -> J {
    let [major, minor, patch] = version;
    match rng.below(100) {
        0..=14 if patch == 0 => J::Str(format!("{major}.{minor}")),
        15..=29 => J::Arr(version.iter().map(|part| number(*part)).collect()),
        _ => J::Str(format!("{major}.{minor}.{patch}")),
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
        "minecraft:client_entity".to_owned(),
        J::obj(&[("description", description)]),
    ));
    J::Obj(root).text()
}

/// The three numbers of a generated `format_version`, in any of its spellings, and whether
/// it is the short string or the array; `None` without the member.
fn version_of(source: &mut Canon) -> Option<((i64, i64, i64), bool, bool)> {
    let (parts, short, array): (Vec<i64>, bool, bool) = match source.at_mut(&["format_version"])? {
        Canon::Str(version) => {
            let parts: Vec<i64> = version
                .split('.')
                .map(|part| part.parse().unwrap())
                .collect();
            let short = parts.len() == 2;
            (parts, short, false)
        }
        Canon::Arr(parts) => (
            parts
                .iter()
                .map(|part| match part {
                    Canon::Int(part) => *part,
                    other => panic!("a generated version holds {other:?}"),
                })
                .collect(),
            false,
            true,
        ),
        other => panic!("a generated version is {other:?}"),
    };
    Some((
        (parts[0], parts[1], parts.get(2).copied().unwrap_or(0)),
        short,
        array,
    ))
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
    /// 1.26.0 documents whose written text was compared with the source.
    compared: usize,
    /// THE class "version respelt": a 1.26 document whose version was the short
    /// string or the array is written with the full string, `"1.26"` and `[1, 26, 0]` as
    /// `"1.26.0"`.
    short_versions_in_full: usize,
    array_versions_as_strings: usize,
    /// Older documents (stamped 1.26.0 by their steps) whose version was so spelt.
    older_versions_short: usize,
    older_versions_as_arrays: usize,
    empty_maps_kept: usize,
    scales_rewritten: usize,
    scales_clamped: usize,
    scripts_dropped: usize,
    constants_as_integers: usize,
    lower_cased: usize,
    case_kept: usize,
    elements_dropped: usize,
}

/// A written text must load, without an error.
#[track_caller]
fn reload(
    written: &str,
    cx: &LoadContext,
    source: &str,
) -> bedrock_addon_core::document::Document<ClientEntity> {
    match entity(written, cx) {
        Ok(document) => document,
        Err(rejected) => panic!(
            "to_json gave text this loader refuses:\n{}\nwritten: {written}\nsource: {source}",
            lines(&rejected.diagnostics).join("\n")
        ),
    }
}

/// The source of a 1.26.0 document as its write-back must read, each recorded difference
/// applied by name.
fn expected(
    source: &str,
    document: &bedrock_addon_core::document::Document<ClientEntity>,
    counts: &mut Counts,
) -> Canon {
    let mut expected = Canon::of_text(source);
    // The class "version respelt": the model holds the version, not its spelling,
    // and writes it in full, a string.
    if let Some(((major, minor, patch), short, array)) = version_of(&mut expected)
        && (short || array)
        && let Canon::Obj(root) = &mut expected
    {
        root.insert(
            "format_version".to_owned(),
            Canon::Str(format!("{major}.{minor}.{patch}")),
        );
        counts.short_versions_in_full += usize::from(short);
        counts.array_versions_as_strings += usize::from(array);
    }
    // an authored empty `animations` or `sound_effects` object is written as it was
    // read. (`materials`, `textures` and `geometry` never reach here empty: the schema
    // refuses them.)
    for name in ["animations", "sound_effects"] {
        let at = [DESCRIPTION[0], DESCRIPTION[1], name];
        if matches!(expected.at_mut(&at), Some(Canon::Obj(members)) if members.is_empty()) {
            counts.empty_maps_kept += 1;
        }
    }
    let scale = ["minecraft:client_entity", "description", "held_item_scale"];
    if let Some(number) = expected.number(&scale) {
        // The `f32` is written as the shortest number that reads back to it, where core would
        // widen it; the variant keeps no token kind, so an integer token is written as a real;
        // a number beyond `f32` is read as the largest finite float of its sign.
        let (written, clamped) = as_f32_written(number);
        *expected.at_mut(&scale).unwrap() = Canon::real(written);
        counts.scales_rewritten += 1;
        counts.scales_clamped += usize::from(clamped);
    }
    for name in ["initialize", "pre_animation"] {
        // a joined script text the parser rejects is dropped with "Failed to parse
        // '<name>'!" (with the `molang` feature); the member is not stored, so not written.
        let at = format!("minecraft:client_entity/description/scripts/{name}");
        let dropped = document.diagnostics().iter().any(|found| {
            found.code.id() == "visual/failed_to_parse"
                && found.location.json_path.to_string() == at
        });
        if dropped && expected.remove(&SCRIPTS, name) {
            counts.scripts_dropped += 1;
        }
    }
    // a Molang member that is a number is a constant, and an integral constant is
    // written as an integer (`2.0` as `2`). The Molang members are the named scripts and
    // the conditions of the object forms.
    let mut constants = 0;
    if let Some(Canon::Obj(scripts)) = expected.at_mut(&SCRIPTS) {
        for (name, member) in scripts.iter_mut() {
            match (name.as_str(), member) {
                ("variables" | "initialize" | "pre_animation", _) => {}
                ("animate", Canon::Arr(entries)) => {
                    constants += conditions_as_constants(entries);
                }
                (_, member) => constants += usize::from(as_constant(member)),
            }
        }
    }
    let controllers = [DESCRIPTION[0], DESCRIPTION[1], "render_controllers"];
    if let Some(Canon::Arr(entries)) = expected.at_mut(&controllers) {
        constants += conditions_as_constants(entries);
    }
    counts.constants_as_integers += constants;
    expected
}

/// An integral real as the integer a Molang constant is written as; whether it changed.
fn as_constant(member: &mut Canon) -> bool {
    let Canon::Real(bits) = member else {
        return false;
    };
    let number = f64::from_bits(*bits);
    if number.fract() != 0.0 {
        return false;
    }
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the generated constants are small"
    )]
    let int = number as i64;
    *member = Canon::Int(int);
    true
}

/// [`as_constant`] over the conditions of the `{ name: condition }` entries of a list.
fn conditions_as_constants(entries: &mut [Canon]) -> usize {
    let mut changed = 0;
    for entry in entries {
        if let Canon::Obj(members) = entry {
            for condition in members.values_mut() {
                changed += usize::from(as_constant(condition));
            }
        }
    }
    changed
}

/// One generated document through the invariant.
fn check(source: &str, shape: Shape, cx: &LoadContext, counts: &mut Counts) {
    counts.generated += 1;
    // The struct its version selects writes its own shape back, and that reads to the same
    // value.
    let path = PackPath::from_static(PATH);
    if let Ok(written) = ClientEntity::parse_as_written(&path, source.as_bytes(), cx) {
        let value = match written.value() {
            ClientEntityHistory::V1_8_0(entity) => entity.write(),
            ClientEntityHistory::V1_10_0(entity) => entity.write(),
            ClientEntityHistory::V1_26_0(entity) => entity.write(),
        };
        let text = bedrock_addon_core::json::to_json(&value.unwrap()).unwrap();
        let again =
            ClientEntity::parse_as_written(&path, text.as_bytes(), cx).unwrap_or_else(|rejected| {
                panic!(
                    "the as-written text is refused:\n{}\nwritten: {text}\nsource: {source}",
                    lines(&rejected.diagnostics).join("\n")
                )
            });
        assert_eq!(again.value(), written.value(), "{source}");
        counts.as_written += 1;
    }
    let Ok(document) = entity(source, cx) else {
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
    // second load has nothing at error level and no new drop.
    assert!(
        again.diagnostics().iter().all(|found| !found.rejects()),
        "{source}"
    );
    counts.round_tripped += 1;
    if shape == Shape::V1_26 {
        let expected = expected(source, &document, counts);
        assert_eq!(Canon::of_text(&written), expected, "{source}\n{written}");
        counts.compared += 1;
    } else {
        // an older document is written in the newest shape, with what its steps
        // did. Its text is not the source's; the value read back is the value loaded
        // (above). The two things the steps do to script text are checked on the text: a
        // document written below 1.18.20 has its statements lower-cased, a later one keeps
        // them; an element that is no string is dropped with a note.
        let dropped = |severity: Severity| {
            document.diagnostics().iter().any(|found| {
                found.code.kind() == Kind::UnknownMember(Disposition::Dropped)
                    && found.severity == severity
            })
        };
        // The version as the numbers it spells, whatever its spelling: the steps key on it.
        let version = version_of(&mut Canon::of_text(source));
        if let Some((_, short, array)) = version {
            counts.older_versions_short += usize::from(short);
            counts.older_versions_as_arrays += usize::from(array);
            // Stamped by the steps, in full.
            assert!(
                written.contains(r#""format_version": "1.26.0""#),
                "{source}\n{written}"
            );
        }
        if source.contains(MIXED_CASE) && !dropped(Severity::Warning) {
            let folds = version.is_none_or(|(version, _, _)| version < (1, 18, 20));
            assert_eq!(written.contains(MIXED_CASE), !folds, "{source}\n{written}");
            assert_eq!(
                written.contains(&MIXED_CASE.to_ascii_lowercase()),
                folds,
                "{source}\n{written}"
            );
            counts.lower_cased += usize::from(folds);
            counts.case_kept += usize::from(!folds);
        }
        counts.elements_dropped += usize::from(dropped(Severity::Info));
    }
}

#[test]
fn every_generated_document_that_loads_round_trips_or_is_refused_with_a_path() {
    let mut rng = Rng::new(0x00c1_1e47_e471_7900);
    let mut counts = Counts::default();
    let contexts = [rp(1, 21, 0), rp(1, 13, 0), rp(1, 26, 50)];
    for index in 0..GENERATED {
        let shape = [Shape::V1_8, Shape::V1_10, Shape::V1_26][index % 3];
        let source = document(&mut rng, shape);
        check(&source, shape, &contexts[rng.below(3)], &mut counts);
    }
    eprintln!("generated client entities: {counts:?}");
    // The generator's invalid forms are rare: most documents load, and every form of a
    // recorded difference was met.
    assert_eq!(counts.generated, GENERATED);
    assert!(counts.loaded * 2 > counts.generated, "{counts:?}");
    assert_eq!(counts.round_tripped + counts.refused, counts.loaded);
    // Nothing a load accepts is refused by the writer.
    assert_eq!(counts.refused, 0, "{counts:?}");
    assert!(counts.as_written >= counts.loaded, "{counts:?}");
    assert!(counts.compared > 500, "{counts:?}");
    // Every spelling of a version was met, in a document that is compared and in one that
    // is upgraded.
    assert!(counts.short_versions_in_full > 0, "{counts:?}");
    assert!(counts.array_versions_as_strings > 0, "{counts:?}");
    assert!(counts.older_versions_short > 0, "{counts:?}");
    assert!(counts.older_versions_as_arrays > 0, "{counts:?}");
    assert!(counts.empty_maps_kept > 0, "{counts:?}");
    assert!(counts.scales_rewritten > 0, "{counts:?}");
    assert!(counts.scales_clamped > 0, "{counts:?}");
    assert!(counts.constants_as_integers > 0, "{counts:?}");
    assert!(counts.lower_cased > 0, "{counts:?}");
    assert!(counts.case_kept > 0, "{counts:?}");
    assert!(counts.elements_dropped > 0, "{counts:?}");
    #[cfg(feature = "molang")]
    assert!(counts.scripts_dropped > 0, "{counts:?}");
}
