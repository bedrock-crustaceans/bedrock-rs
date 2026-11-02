//! The generated round trip: a few thousand atomic client entity
//! documents from a fixed seed, over every member's forms. Every document that loads writes
//! text that loads again to an equal value and writes identically, or is refused with
//! `NotRepresentable` and a path; and the text written equals the source's, but for the
//! recorded differences, each applied by name below.

use bedrock_addon::document::Content;
use bedrock_addon::json::EncodeError;

use super::common::*;
use super::generate::{Canon, J, Rng, as_f32_written, string_map};

const PAYLOAD: &str = "minecraft:atomic_client_entity";
const DESCRIPTION: [&str; 2] = [PAYLOAD, "description"];
const BOX: [&str; 3] = [PAYLOAD, "components", "minecraft:aabb_dimensions"];

/// A list of strings: absent, empty, or up to three; rarely an element that is no string.
fn strings(rng: &mut Rng, absent: usize, values: &[&str]) -> Option<J> {
    if rng.chance(absent) {
        return None;
    }
    Some(J::Arr(
        (0..rng.below(4))
            .map(|_| match rng.below(100) {
                0 => J::Num("1"),
                1 => J::obj(&[("idle", J::str("1.0"))]),
                _ => J::str(rng.pick(values)),
            })
            .collect(),
    ))
}

/// One side of the box: whole and fractional numbers of either token kind, zero, numbers
/// that are no `f32` exactly, numbers beyond `f32`; rarely below zero or no number.
fn side(rng: &mut Rng) -> J {
    match rng.below(100) {
        0..=1 => J::Num(rng.pick(&["-1", "-0.5", "-1e39"])),
        2 => J::str("1"),
        _ => J::Num(rng.pick(&[
            "0",
            "0.0",
            "1",
            "0.5",
            "2.678",
            "0.1",
            "16777217",
            "3.4028235e38",
            "1e-50",
            "1e39",
            "1e999",
        ])),
    }
}

/// One atomic client entity document, as text.
fn document(rng: &mut Rng) -> String {
    let mut d: Vec<(String, J)> = Vec::new();
    let mut add = |name: &str, value: Option<J>| {
        if let Some(value) = value {
            d.push((name.to_owned(), value));
        }
    };
    let identifier = match rng.below(100) {
        0 => Some(J::str("wisp")),
        1 => Some(J::Num("5")),
        2 => None,
        _ => Some(J::str(rng.pick(&[
            "demo:wisp",
            "Demo:Wisp",
            "minecraft:wisp",
            "a:b:c",
        ]))),
    };
    add("identifier", identifier);
    let materials = ["entity_alphatest", "Entity_AlphaTest"];
    add("materials", string_map(rng, 35, &materials));
    let textures = ["textures/entity/wisp", "Textures/Entity/Wisp"];
    add("textures", string_map(rng, 35, &textures));
    let geometry = ["geometry.wisp", "Geometry.Wisp.V2"];
    add("geometry", string_map(rng, 35, &geometry));
    let animations = [
        "animation.wisp.idle",
        "Controller.Animation.Wisp.Idle",
        "controller.animation.wisp.idle",
    ];
    add("animations", string_map(rng, 35, &animations));
    let particles = ["minecraft:flame", "Demo:Spark"];
    add("particle_effects", string_map(rng, 60, &particles));
    let sounds = ["mob.wisp.say", "Mob.Wisp.Hurt"];
    add("sound_effects", string_map(rng, 60, &sounds));
    if rng.chance(70) {
        let mut scripts: Vec<(String, J)> = Vec::new();
        let statements = ["variable.x = 1;", "Variable.Y = Query.Is_Baby;", "1 +", ""];
        for name in ["initialize", "pre_animation"] {
            if let Some(list) = strings(rng, 55, &statements) {
                scripts.push((name.to_owned(), list));
            }
        }
        if let Some(list) = strings(rng, 55, &["idle", "Idle", "missing"]) {
            scripts.push(("animate".to_owned(), list));
        }
        if rng.chance(1) {
            scripts.push(("scale".to_owned(), J::str("1.0")));
        }
        add("scripts", Some(J::Obj(scripts)));
    }
    let controllers = ["controller.render.wisp", "Controller.Render.Wisp"];
    add("render_controllers", strings(rng, 35, &controllers));
    if rng.chance(1) {
        add("hide_armor", Some(J::Bool(true)));
    }
    let mut payload = vec![("description".to_owned(), J::Obj(d))];
    if rng.chance(70) {
        let mut components: Vec<(String, J)> = Vec::new();
        if rng.chance(85) {
            let mut sides = vec![("width".to_owned(), side(rng))];
            if !rng.chance(1) {
                sides.push(("height".to_owned(), side(rng)));
            }
            components.push(("minecraft:aabb_dimensions".to_owned(), J::Obj(sides)));
        }
        if rng.chance(1) {
            // The 1.26.45 key: an unknown component of 1.26.60.
            components.push(("minecraft:collision_box".to_owned(), J::Obj(Vec::new())));
        }
        payload.push(("components".to_owned(), J::Obj(components)));
    }
    let version = if rng.chance(1) { "1.26.0" } else { "beta" };
    J::obj(&[
        ("format_version", J::str(version)),
        (PAYLOAD, J::Obj(payload)),
    ])
    .text()
}

/// How many documents the test generates.
const GENERATED: usize = 2000;

/// What the run saw.
#[derive(Debug, Default)]
struct Counts {
    loaded: usize,
    round_tripped: usize,
    refused: usize,
    empty_members_kept: usize,
    sides_rewritten: usize,
    sides_clamped: usize,
}

/// The source as its write-back must read, each recorded difference applied by name.
fn expected(source: &str, counts: &mut Counts) -> Canon {
    let mut expected = Canon::of_text(source);
    // An authored empty member is written as it was read.
    for name in [
        "materials",
        "textures",
        "geometry",
        "animations",
        "render_controllers",
    ] {
        let at = [DESCRIPTION[0], DESCRIPTION[1], name];
        if matches!(
            expected.at_mut(&at),
            Some(Canon::Obj(members)) if members.is_empty()
        ) || matches!(expected.at_mut(&at), Some(Canon::Arr(elements)) if elements.is_empty())
        {
            counts.empty_members_kept += 1;
        }
    }
    for name in ["width", "height"] {
        // The sides are `f32`: a number is written as the shortest real that reads back to
        // its nearest `f32` (`1` as `1.0`, `16777217` as `16777216.0`), and a number
        // beyond `f32` as the largest finite float.
        let at = [BOX[0], BOX[1], BOX[2], name];
        if let Some(number) = expected.number(&at) {
            let (written, clamped) = as_f32_written(number);
            *expected.at_mut(&at).unwrap() = Canon::real(written);
            counts.sides_rewritten += 1;
            counts.sides_clamped += usize::from(clamped);
        }
    }
    expected
}

#[test]
fn every_generated_document_that_loads_round_trips_or_is_refused_with_a_path() {
    let mut rng = Rng::new(0x0a70_41c0_e471_7900);
    let mut counts = Counts::default();
    let cx = cx();
    for _ in 0..GENERATED {
        let source = document(&mut rng);
        let Ok(document) = atomic(&source, &cx) else {
            continue;
        };
        counts.loaded += 1;
        let written = match document.to_json() {
            Ok(written) => written,
            Err(EncodeError::NotRepresentable { at, .. }) => {
                assert!(!at.is_empty(), "a refusal without a path: {source}");
                counts.refused += 1;
                continue;
            }
            Err(other) => panic!("{other}: {source}"),
        };
        let again = match atomic(&written, &cx) {
            Ok(again) => again,
            Err(rejected) => panic!(
                "to_json gave text this loader refuses:\n{}\nwritten: {written}\nsource: {source}",
                lines(&rejected.diagnostics).join("\n")
            ),
        };
        assert_eq!(again.value(), document.value(), "{source}\n{written}");
        assert_eq!(again.to_json().unwrap(), written, "{source}");
        // What the first load replaced (a clamped side) is in the text: nothing is found
        // the second time.
        silent(again.diagnostics());
        assert_eq!(
            Canon::of_text(&written),
            expected(&source, &mut counts),
            "{source}\n{written}"
        );
        counts.round_tripped += 1;
    }
    eprintln!("generated atomic client entities: {GENERATED} generated, {counts:?}");
    assert!(counts.loaded * 2 > GENERATED, "{counts:?}");
    assert_eq!(counts.round_tripped + counts.refused, counts.loaded);
    assert!(counts.empty_members_kept > 0, "{counts:?}");
    assert!(counts.sides_rewritten > 0, "{counts:?}");
    assert!(counts.sides_clamped > 0, "{counts:?}");
}
