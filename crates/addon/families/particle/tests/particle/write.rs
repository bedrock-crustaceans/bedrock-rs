//! Writing back: what is written is what was read, a value the loader would
//! refuse has no text, and a few thousand generated documents round-trip.

use std::fmt::Write as _;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{Content, JsonDocument as _, Placed};
use bedrock_addon_core::id::ParticleId;
use bedrock_addon_core::json::{EncodeError, Unrepresentable, Value};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_family_particle::component::{
    DistanceEvent, EmitterLifetimeEvents, EmitterRateInstant, ParticleMotionCollision, VectorEvents,
};
use bedrock_addon_family_particle::{Curve, EventNode, ParticleEffect};

use super::common::{
    BASE, CEREAL, LEGACY, at, context, effect, effect_with, findings, json, load, loaded, number,
    rejected, written,
};

/// The path of a refusal.
#[track_caller]
fn refused(effect: &ParticleEffect) -> String {
    match effect.to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => at.to_string(),
        other => panic!("{other:?}"),
    }
}

/// A valid effect to change.
fn base() -> ParticleEffect {
    let mut effect = ParticleEffect::new(
        ParticleId::parse("demo:spark"),
        "particles_alpha",
        "textures/particle/particles",
    );
    effect.components.insert(EmitterRateInstant {
        num_particles: number(1.0),
        ..EmitterRateInstant::default()
    });
    effect
}

#[test]
fn a_built_effect_writes_text_that_loads_back_equal() {
    let effect = base();
    let text = effect.to_json().unwrap();
    let again = loaded(&text);
    assert_eq!(again.value(), &effect);
    assert_eq!(findings(again.diagnostics()), [] as [&str; 0]);
    assert!(text.contains(r#""format_version": "1.20.80""#), "{text}");
}

/// The default path of a Free key: the identifier with `:` as `_`, directly in `particles/`.
#[test]
fn the_default_path_is_the_identifier_in_particles() {
    let effect = ParticleEffect::new(
        ParticleId::parse("minecraft:basic_flame_particle"),
        "m",
        "t",
    );
    assert_eq!(
        effect.default_path().as_str(),
        "particles/minecraft_basic_flame_particle.json"
    );
    assert_eq!(base().default_path().as_str(), "particles/demo_spark.json");
}

/// Write-back writes no member the author did not write.
#[test]
fn a_member_the_document_does_not_write_is_not_written() {
    let text = effect(
        CEREAL,
        r#""minecraft:emitter_shape_sphere": {}, "minecraft:particle_appearance_billboard": {"size": [1, 1], "uv": {"flipbook": {"base_UV": [0, 0], "max_frame": 1}}}, "minecraft:particle_motion_collision": {"collision_radius": 0.5, "events": {"event": "e"}}, "minecraft:emitter_lifetime_events": {"timeline": {}, "looping_travel_distance_events": []}"#,
    );
    let effect = loaded(&text);
    let source = json(&text);
    assert_eq!(
        at(&written(&effect), "particle_effect"),
        at(&source, "particle_effect")
    );
    // Neither `curves` nor `events`; present but empty ones stay.
    assert!(at(&written(&effect), "particle_effect/curves").is_none());
    let empty = loaded(&effect_with(CEREAL, r#""curves": {}, "events": {}"#));
    assert_eq!(
        at(&written(&empty), "particle_effect/curves"),
        Some(&json("{}"))
    );
    assert_eq!(
        at(&written(&empty), "particle_effect/events"),
        Some(&json("{}"))
    );
}

/// An unknown member is told at every level: an error that refuses a 1.20.80 document,
/// a warning in a 1.10.0 one, which keeps none of them and writes without them.
#[test]
fn unknown_members_are_told_at_their_level_and_not_kept() {
    let members = r#"{"description": {"identifier": "demo:spark", "basic_render_parameters": {"material": "m", "texture": "t", "a": 1}, "b": 2}, "components": {"minecraft:emitter_rate_instant": {"num_particles": 1, "c": 3}}, "curves": {"variable.c": {"type": "linear", "input": 0, "nodes": [0], "d": 4}}, "events": {"e": {"log": "x", "f": 5, "randomize": [{"weight": 1, "g": 6}]}}, "extra": {"h": 7}}"#;
    let cereal = format!(r#"{{"format_version": "1.26.10", "particle_effect": {members}}}"#);
    let errors = rejected(&cereal);
    assert_eq!(
        findings(&errors)
            .iter()
            .filter(|line| line.starts_with("Error") && line.contains("core/unrecognized_field"))
            .count(),
        7,
        "{:#?}",
        findings(&errors)
    );
    let text = format!(r#"{{"format_version": "1.10.0", "particle_effect": {members}}}"#);
    let effect = loaded(&text);
    assert_eq!(
        findings(effect.diagnostics())
            .iter()
            // the legacy parser reads past them silently: informations
            .filter(|line| line.starts_with("Info") && line.contains("core/unrecognized_field"))
            .count(),
        7,
        "{:#?}",
        findings(effect.diagnostics())
    );
    assert!(effect.extra.is_empty());
    let written = written(&effect);
    assert_eq!(at(&written, "particle_effect/extra"), None);
    assert_eq!(
        at(
            &written,
            "particle_effect/description/basic_render_parameters/a"
        ),
        None
    );
    assert_eq!(loaded(&effect.to_json().unwrap()).value(), effect.value());
}

/// the game never stores an unknown key. A 1.10.0 document with an
/// unknown member and an unknown component loads with the legacy warnings, holds neither,
/// writes at 1.20.80 without them and reads back equal.
#[test]
fn a_legacy_document_with_unknown_keys_writes_without_them_and_reads_back_equal() {
    let text = super::common::document(
        LEGACY,
        r#""zz": 1, "components": {"minecraft:emitter_lifetime_once": {}, "minecraft:emitter_rate_instant": {"num_particles": 1, "c": 3}, "x:y": {"a": 1}}"#,
    );
    let document = loaded(&text);
    let codes: Vec<String> = findings(document.diagnostics());
    assert!(
        codes
            .iter()
            .any(|line| line.starts_with("Info particle_effect/zz core/unrecognized_field")),
        "{codes:?}"
    );
    assert!(
        codes.iter().any(|line| line
            .starts_with("Error particle_effect/components/x:y particle/unrecognized_component")),
        "{codes:?}"
    );
    assert!(document.extra.is_empty());
    let written = document.to_json().expect("the upgraded value writes");
    assert!(
        !written.contains("zz") && !written.contains("x:y") && !written.contains(r#""c""#),
        "{written}"
    );
    let again = loaded(&written);
    assert_eq!(again.value(), document.value());
    assert!(
        findings(again.diagnostics()).is_empty(),
        "{:?}",
        findings(again.diagnostics())
    );
}

#[test]
fn a_number_that_is_not_finite_has_no_text() {
    let mut effect = base();
    effect.components.insert(EmitterRateInstant {
        num_particles: Molang::Const(f32::NAN),
        ..EmitterRateInstant::default()
    });
    assert_eq!(
        refused(&effect),
        "particle_effect/components/minecraft:emitter_rate_instant/num_particles"
    );
    let mut effect = base();
    effect.components.insert(ParticleMotionCollision {
        collision_drag: Some(f32::INFINITY),
        ..ParticleMotionCollision::default()
    });
    assert_eq!(
        refused(&effect),
        "particle_effect/components/minecraft:particle_motion_collision/collision_drag"
    );
}

/// A Molang bool is read by no 1.20.80 field: it has no text.
#[test]
fn a_molang_bool_has_no_text() {
    let mut effect = base();
    effect.components.insert(EmitterRateInstant {
        num_particles: Molang::Bool(true),
        ..EmitterRateInstant::default()
    });
    assert_eq!(
        refused(&effect),
        "particle_effect/components/minecraft:emitter_rate_instant/num_particles"
    );
}

/// What the loader's constraints refuse is not written: `to_json` never returns text this
/// loader does not read.
#[test]
fn a_value_outside_a_constraint_of_the_loader_has_no_text() {
    let radius = "particle_effect/components/minecraft:particle_motion_collision/collision_radius";
    for (value, fits) in [(0.0, true), (0.5, true), (-0.001, false), (0.501, false)] {
        let mut effect = base();
        effect.components.insert(ParticleMotionCollision {
            collision_radius: value,
            ..ParticleMotionCollision::default()
        });
        if fits {
            drop(loaded(&effect.to_json().unwrap()));
        } else {
            assert_eq!(refused(&effect), radius, "{value}");
        }
    }
    for (value, fits) in [(0.0, true), (-0.5, false)] {
        let mut effect = base();
        effect.components.insert(EmitterLifetimeEvents {
            looping_travel_distance_events: Some(vec![DistanceEvent {
                distance: value,
                effects: VectorEvents::One("e".to_owned()),
                ..DistanceEvent::default()
            }]),
            ..EmitterLifetimeEvents::default()
        });
        if fits {
            drop(loaded(&effect.to_json().unwrap()));
        } else {
            assert_eq!(
                refused(&effect),
                "particle_effect/components/minecraft:emitter_lifetime_events/looping_travel_distance_events/0/distance"
            );
        }
    }
    for (length, fits) in [(256, true), (257, false)] {
        let mut effect = base();
        effect.description.basic_render_parameters.texture = "t".repeat(length);
        if fits {
            drop(loaded(&effect.to_json().unwrap()));
        } else {
            assert_eq!(
                refused(&effect),
                "particle_effect/description/basic_render_parameters/texture"
            );
        }
    }
}

/// An unknown member named like a known one would replace it in the text.
#[test]
fn an_unknown_member_named_like_a_known_one_has_no_text() {
    let mut effect = base();
    effect
        .extra
        .insert("components".to_owned(), Value::from(1_i64));
    assert_eq!(refused(&effect), "particle_effect/components");
    let mut effect = base();
    effect
        .description
        .extra
        .insert("identifier".to_owned(), Value::from("x"));
    assert_eq!(refused(&effect), "particle_effect/description/identifier");
}

/// A chain of nodes of length `depth`, each the only node of the last one's `sequence`.
fn chain(depth: usize) -> EventNode {
    let mut node = EventNode::default();
    for _ in 0..depth {
        let mut outer = EventNode::default();
        outer.sequence = Some(vec![node]);
        node = outer;
    }
    node
}

/// Events nest as deep as a file may be and no deeper: the deepest text the loader reads is
/// written, one level more is refused with a path, and a chain of any length is dropped
/// without using the stack.
#[test]
fn events_nest_to_the_depth_a_file_may_have_and_no_deeper() {
    // `{ particle_effect { events { e {` is 4 levels; each `sequence` step adds `[` and `{`.
    let deepest = (128 - 4) / 2;
    let mut effect = base();
    effect.events = Some([("e".to_owned(), chain(deepest))].into_iter().collect());
    let text = effect.to_json().unwrap();
    assert_eq!(loaded(&text).value(), &effect);
    effect.events = Some([("e".to_owned(), chain(deepest + 1))].into_iter().collect());
    assert!(refused(&effect).starts_with("particle_effect"));
    effect.events = Some([("e".to_owned(), chain(200_000))].into_iter().collect());
    assert!(refused(&effect).starts_with("particle_effect/events/e/sequence/0/"));
    drop(effect);
}

/// What `EventNode` documents of its derived impls: a tree a file can hold is shallow enough
/// to clone, compare and print, because the loader refuses a file that nests one node
/// deeper; a chain of 100 000 nodes built in memory is refused by the writer and dropped.
#[test]
fn a_read_event_tree_is_bounded_and_a_built_one_of_any_depth_is_refused_and_dropped() {
    // The text of an event that is a chain of `depth` nodes below it.
    let nested = |depth: usize| {
        let text = format!(
            r#"{{{}"log": "x"{}}}"#,
            r#""sequence": [{"#.repeat(depth),
            "}]".repeat(depth)
        );
        effect_with(CEREAL, &format!(r#""events": {{"e": {text}}}"#))
    };
    let deepest = (128 - 4) / 2;
    let read = loaded(&nested(deepest));
    let event = &read.events.as_ref().unwrap()["e"];
    let mut depth = 0;
    let mut node = event;
    while let Some(sequence) = &node.sequence {
        node = &sequence[0];
        depth += 1;
    }
    assert_eq!(depth, deepest);
    assert_eq!(node.log(), "x");
    // The derived impls, on the deepest tree a file gives.
    let copy = event.clone();
    assert_eq!(&copy, event);
    assert!(format!("{event:?}").matches("sequence").count() > deepest);
    // One node deeper is no file the loader reads.
    assert!(load(&nested(deepest + 1), &context()).is_err());

    let mut effect = base();
    effect.events = Some([("e".to_owned(), chain(100_000))].into_iter().collect());
    assert!(refused(&effect).starts_with("particle_effect/events/e/sequence/0/"));
    drop(effect);
}

/// A file nested deeper than the limit is refused before it is parsed.
#[test]
fn a_file_nested_past_the_limit_is_refused() {
    let nested = format!("{}1{}", "[".repeat(200), "]".repeat(200));
    let text = effect_with(CEREAL, &format!(r#""x": {nested}"#));
    assert!(load(&text, &context()).is_err());
}

// ---- generated documents ------------------------------------------------------------------

/// A small deterministic generator (xorshift): the same documents on every run.
///
/// A `clean` document takes only the forms the 1.20.80 reader accepts (those before the
/// [`BAD`] of a list) and leaves out no required member, so most of them load in that era
/// too; the others take every form.
struct Rng {
    state: u64,
    clean: bool,
    /// Whether the text is decorated ([`object`]), and the stream the decorations are drawn
    /// from: apart from the content's, so that a document can be had with and without.
    decorate: bool,
    style: u64,
}

/// Separates the forms the 1.20.80 reader accepts from those it refuses.
const BAD: &str = "|";

impl Rng {
    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn pick<'a>(&mut self, options: &[&'a str]) -> &'a str {
        let good = options
            .iter()
            .position(|o| *o == BAD)
            .unwrap_or(options.len());
        let options: Vec<&str> = if self.clean {
            options[..good].to_vec()
        } else {
            options.iter().copied().filter(|o| *o != BAD).collect()
        };
        options[usize::try_from(self.next() % options.len() as u64).unwrap()]
    }

    fn next_style(&mut self) -> u64 {
        self.style ^= self.style << 13;
        self.style ^= self.style >> 7;
        self.style ^= self.style << 17;
        self.style
    }

    fn chance(&mut self, of: u64) -> bool {
        self.next().is_multiple_of(of)
    }

    /// Whether to leave out a required member or break a shape: never in a clean document.
    fn spoil(&mut self, of: u64) -> bool {
        self.chance(of) && !self.clean
    }
}

/// The forms a Molang member takes.
const MOLANG: &[&str] = &[
    "0",
    "1",
    "-2.5",
    "1e39",
    "-1e39",
    "0.1",
    "16777217",
    "1.0",
    r#""variable.x""#,
    r#""1 +""#,
    r#""""#,
    r#"{"expression": "variable.y", "version": 12}"#,
    r#"{"expression": "", "version": 3}"#,
    r#"{"expression": "1", "version": -1}"#,
    BAD,
    "true",
    "null",
    "[]",
    "{}",
    r#"{"expression": 1}"#,
];
const FLOAT: &[&str] = &[
    "0", "0.5", "0.25", "0.1", BAD, "-1", "2", "1e39", r#""x""#, "null",
];
const BOOL: &[&str] = &["true", "false", BAD, "1", "null"];
const EVENTS: &[&str] = &[r#""e""#, r#"["e", "f"]"#, "[]", r#""""#, BAD, "[1]", "1"];
const DIRECTION: &[&str] = &[
    r#""inwards""#,
    r#""outwards""#,
    "[0, 1, 0]",
    BAD,
    r#""up""#,
    "[0, 1]",
    "1",
];
const COLOR: &[&str] = &[
    "[1, 0.5, 0]",
    "[1, 0.5, 0, 1]",
    r##""#FF00FF00""##,
    r#""red""#,
    r##"{"gradient": [[1, 1, 1], "#FFFFFF"], "interpolant": "variable.t"}"##,
    r##"{"gradient": {"0.0": [1, 1, 1, 1], "1.0": "#000000"}, "interpolant": 0.5}"##,
    BAD,
    "[1]",
    "1",
    r#"{"gradient": {}}"#,
    r#"{"interpolant": 1}"#,
];
const TIMELINE: &[&str] = &[r#"{"0.5": "e", "1": ["f"]}"#, "{}", BAD, "[]", "1"];
const LOOPING: &[&str] = &[
    r#"[{"distance": 1, "effects": "e"}]"#,
    "[]",
    BAD,
    r#"[{"distance": -1, "effects": "e"}]"#,
    r#"[{"effects": ["e"]}]"#,
    r#"[{"distance": 0}]"#,
    "[1]",
    "{}",
];
const PLANE: &[&str] = &[r#""x""#, "[0, 0, 1]", BAD, r#""w""#, "2"];
const BLOCKS: &[&str] = &[r#"["minecraft:air"]"#, "[]", BAD, "[1]", "1"];
const PLANE4: &[&str] = &[
    "[0, 1, 0, 0]",
    "[0, 1, 0, 1e39]",
    BAD,
    "[0, 1, 0]",
    r#"[0, "x", 0, 0]"#,
    "1",
];
const COLLISION_EVENTS: &[&str] = &[
    r#"{"event": "e"}"#,
    r#"[{"event": "e", "min_speed": 0.5}, {"event": "f", "min_speed": 3}]"#,
    "[]",
    BAD,
    r#"[{"min_speed": 1}]"#,
    r#"[{"event": "e", "min_speed": "x"}]"#,
    "[1]",
    "1",
];
const FACING: &[&str] = &[r#""lookat_xyz""#, r#""nonsense""#, BAD, "1"];
const BILLBOARD_DIRECTION: &[&str] = &[
    r#"{"mode": "custom", "custom_direction": [0, 1, 0]}"#,
    r#"{"mode": "custom"}"#,
    r#"{"mode": "derive_from_velocity", "min_speed_threshold": 0.1}"#,
    BAD,
    r#"{"mode": "sideways"}"#,
    r#"{"min_speed_threshold": 1e39}"#,
    "1",
];
const UV: &[&str] = &[
    r#"{"texture_width": 16, "texture_height": 16, "uv": [0, 0], "uv_size": [1, 1]}"#,
    r#"{"texturewidth": 16, "uv": ["variable.u", 0]}"#,
    r#"{"flipbook": {"base_UV": [0, 0], "size_UV": [1, 1], "step_UV": [1, 0], "frames_per_second": 8, "max_frame": "variable.f", "stretch_to_lifetime": true, "loop": false}}"#,
    BAD,
    r#"{"texture_width": 16.5}"#,
    r#"{"flipbook": {}}"#,
    r#"{"flipbook": 1}"#,
    "[]",
];
const LIGHTING: &[&str] = &["{}", r#"{"x": 1}"#, BAD, "1"];
const CHAIN_NODES: &[&str] = &[
    r#"{"0.0": {"value": 1, "slope": 0}, "1.0": {"left_value": 1, "right_value": 0, "left_slope": 1, "right_slope": 0}}"#,
    r#"{"0.0": {"left_value": 1}}"#,
    "{}",
    BAD,
    r#"{"0.0": 1}"#,
];
const EFFECT: &[&str] = &[
    r#"{"effect": "demo:other", "type": "emitter"}"#,
    r#"{"effect": "demo:other", "type": "particle_with_velocity", "pre_effect_expression": "variable.a = 1;"}"#,
    r#"{"effect": "demo:other", "type": "emitter_bound", "pre_effect_expression": 3}"#,
    BAD,
    r#"{"effect": "demo:other", "type": "none"}"#,
    r#"{"effect": 1, "type": "particle"}"#,
    r#"{"type": "particle"}"#,
    "1",
];
const SOUND: &[&str] = &[
    r#"{"event_name": "explode"}"#,
    BAD,
    r#"{"event_name": 1}"#,
    "{}",
    "1",
];
const WEIGHT: &[&str] = &[
    r#""weight": 1"#,
    r#""weight": -1"#,
    r#""weight": 0.5"#,
    BAD,
    r#""weight": "x""#,
    "",
];
const DESCRIPTION: &[&str] = &[
    r#"{"identifier": "demo:g", "basic_render_parameters": {"material": "m", "texture": "t"}}"#,
    r#"{"identifier": "demo:g", "basic_render_parameters": {"material": "", "texture": ""}}"#,
    BAD,
    r#"{"identifier": "demo:g", "basic_render_parameters": {"material": "m"}}"#,
    r#"{"identifier": "demo:g", "basic_render_parameters": {"material": 1, "texture": "t"}}"#,
    r#"{"identifier": "demo:g", "basic_render_parameters": {"material": "m", "texture": 1}}"#,
    r#"{"identifier": 1}"#,
    r#"{"identifier": "demo:g"}"#,
];

/// `"key": value` for an optional member, there two times in three.
fn member(rng: &mut Rng, key: &str, forms: &[&str], into: &mut Vec<String>) {
    if !rng.chance(3) {
        into.push(format!(r#""{key}": {}"#, rng.pick(forms)));
    }
}

/// `"key": value` for a member the 1.20.80 reader requires: left out only to spoil.
fn required(rng: &mut Rng, key: &str, forms: &[&str], into: &mut Vec<String>) {
    if !rng.spoil(4) {
        into.push(format!(r#""{key}": {}"#, rng.pick(forms)));
    }
}

/// An array of `count` Molang values, or (to spoil) another shape.
fn molang_array(rng: &mut Rng, count: usize) -> String {
    if rng.spoil(8) {
        return rng
            .pick(&["[0]", "1", r#""x""#, "[0, 0, 0, 0, 0]"])
            .to_owned();
    }
    let elements: Vec<&str> = (0..count).map(|_| rng.pick(MOLANG)).collect();
    format!("[{}]", elements.join(", "))
}

/// What a first copy of a member written twice holds: the last copy is the member.
const JUNK: &[&str] = &["1", "null", r#""junk""#, "{}", "[]", "[[1]]"];

/// The JSON object of `members` (`"key": value` each). In a decorated document one object of
/// four is written otherwise without saying anything else: a comment before a member, a
/// member written twice (the first copy holding junk), or a key whose first character is a
/// `\u` escape.
fn object(rng: &mut Rng, members: &[String]) -> String {
    if !rng.decorate || !rng.next_style().is_multiple_of(4) {
        return format!("{{{}}}", members.join(", "));
    }
    let mut out = Vec::new();
    for member in members {
        let (key, value) = member.split_once(r#"": "#).expect(r#""key": value"#);
        let key = &key[1..];
        // The version is the first textual `"format_version"` of the file (the game's
        // reader): a junk copy before it, or the key spelt with an escape, changes the version
        // read, not only the text. It is written plainly.
        if key == "format_version" {
            out.push(member.clone());
            continue;
        }
        let mut text = String::new();
        match rng.next_style() % 4 {
            0 => text.push_str("/* a comment, with \"quotes\" and { */ "),
            1 => text.push_str("// a comment to the end of its line: }\n"),
            2 => {
                let junk = JUNK[usize::try_from(rng.next_style() % JUNK.len() as u64).unwrap()];
                write!(text, r#""{key}": {junk}, "#).unwrap();
            }
            _ => {}
        }
        let mut letters = key.chars();
        match letters.next() {
            Some(first) if first.is_ascii() && rng.next_style().is_multiple_of(3) => {
                let rest = letters.as_str();
                write!(text, r#""\u{:04x}{rest}": {value}"#, first as u32).unwrap();
            }
            _ => text.push_str(member),
        }
        out.push(text);
    }
    format!("{{{}}}", out.join(", "))
}

/// What a generated component is written as.
enum Body {
    /// An object of these members.
    Members(Vec<String>),
    /// A value that is no object of members: an array, a number.
    Value(String),
}

/// `"key": [Molang; count]`, there two times in three.
fn array_member(rng: &mut Rng, key: &str, count: usize, into: &mut Vec<String>) {
    if !rng.chance(3) {
        into.push(format!(r#""{key}": {}"#, molang_array(rng, count)));
    }
}

/// An object of the optional members `keys`, each a Molang value.
fn molang_members(rng: &mut Rng, keys: &[&str]) -> Body {
    let mut m = Vec::new();
    for key in keys {
        member(rng, key, MOLANG, &mut m);
    }
    Body::Members(m)
}

/// Writes one component from the stream.
type Generator = fn(&mut Rng) -> Body;

/// The generator of each component of the family, by its name: every row of the family's
/// component table has one (`the_generator_names_every_component_of_the_family`).
const COMPONENTS: &[(&str, Generator)] = &[
    ("minecraft:emitter_initialization", |rng| {
        molang_members(rng, &["creation_expression", "per_update_expression"])
    }),
    ("minecraft:emitter_lifetime_events", |rng| {
        let mut m = Vec::new();
        member(rng, "creation_event", EVENTS, &mut m);
        member(rng, "expiration_event", EVENTS, &mut m);
        member(rng, "timeline", TIMELINE, &mut m);
        member(rng, "travel_distance_events", TIMELINE, &mut m);
        member(rng, "looping_travel_distance_events", LOOPING, &mut m);
        Body::Members(m)
    }),
    ("minecraft:emitter_lifetime_expression", |rng| {
        molang_members(rng, &["activation_expression", "expiration_expression"])
    }),
    ("minecraft:emitter_lifetime_looping", |rng| {
        molang_members(rng, &["active_time", "sleep_time"])
    }),
    ("minecraft:emitter_lifetime_once", |rng| {
        molang_members(rng, &["active_time"])
    }),
    ("minecraft:emitter_local_space", |rng| {
        let mut m = Vec::new();
        member(rng, "position", BOOL, &mut m);
        member(rng, "rotation", BOOL, &mut m);
        member(rng, "velocity", BOOL, &mut m);
        Body::Members(m)
    }),
    ("minecraft:emitter_rate_instant", |rng| {
        let mut m = Vec::new();
        required(rng, "num_particles", MOLANG, &mut m);
        Body::Members(m)
    }),
    ("minecraft:emitter_rate_manual", |rng| {
        let mut m = Vec::new();
        required(rng, "max_particles", MOLANG, &mut m);
        Body::Members(m)
    }),
    ("minecraft:emitter_rate_steady", |rng| {
        let mut m = Vec::new();
        required(rng, "spawn_rate", MOLANG, &mut m);
        required(rng, "max_particles", MOLANG, &mut m);
        Body::Members(m)
    }),
    ("minecraft:emitter_shape_box", |rng| {
        let mut m = Vec::new();
        if !rng.spoil(4) {
            m.push(format!(r#""half_dimensions": {}"#, molang_array(rng, 3)));
        }
        array_member(rng, "offset", 3, &mut m);
        member(rng, "surface_only", BOOL, &mut m);
        member(rng, "direction", DIRECTION, &mut m);
        Body::Members(m)
    }),
    ("minecraft:emitter_shape_custom", |rng| {
        let mut m = Vec::new();
        array_member(rng, "offset", 3, &mut m);
        array_member(rng, "direction", 3, &mut m);
        Body::Members(m)
    }),
    ("minecraft:emitter_shape_disc", |rng| {
        let mut m = Vec::new();
        array_member(rng, "offset", 3, &mut m);
        member(rng, "radius", MOLANG, &mut m);
        member(rng, "surface_only", BOOL, &mut m);
        member(rng, "plane_normal", PLANE, &mut m);
        member(rng, "direction", DIRECTION, &mut m);
        Body::Members(m)
    }),
    ("minecraft:emitter_shape_entity_aabb", |rng| {
        let mut m = Vec::new();
        member(rng, "surface_only", BOOL, &mut m);
        member(rng, "direction", DIRECTION, &mut m);
        Body::Members(m)
    }),
    ("minecraft:emitter_shape_point", |rng| {
        let mut m = Vec::new();
        array_member(rng, "offset", 3, &mut m);
        array_member(rng, "direction", 3, &mut m);
        Body::Members(m)
    }),
    ("minecraft:emitter_shape_sphere", |rng| {
        let mut m = Vec::new();
        array_member(rng, "offset", 3, &mut m);
        member(rng, "radius", MOLANG, &mut m);
        member(rng, "surface_only", BOOL, &mut m);
        member(rng, "direction", DIRECTION, &mut m);
        Body::Members(m)
    }),
    ("minecraft:particle_appearance_billboard", |rng| {
        let mut m = Vec::new();
        if !rng.spoil(4) {
            m.push(format!(r#""size": {}"#, molang_array(rng, 2)));
        }
        member(rng, "facing_camera_mode", FACING, &mut m);
        member(rng, "direction", BILLBOARD_DIRECTION, &mut m);
        member(rng, "uv", UV, &mut m);
        Body::Members(m)
    }),
    ("minecraft:particle_appearance_lighting", |rng| {
        Body::Value(rng.pick(LIGHTING).to_owned())
    }),
    ("minecraft:particle_appearance_tinting", |rng| {
        let mut m = Vec::new();
        required(rng, "color", COLOR, &mut m);
        Body::Members(m)
    }),
    ("minecraft:particle_expire_if_in_blocks", |rng| {
        Body::Value(rng.pick(BLOCKS).to_owned())
    }),
    ("minecraft:particle_expire_if_not_in_blocks", |rng| {
        Body::Value(rng.pick(BLOCKS).to_owned())
    }),
    ("minecraft:particle_initial_speed", |rng| {
        Body::Value(if rng.chance(2) {
            rng.pick(MOLANG).to_owned()
        } else {
            molang_array(rng, 3)
        })
    }),
    ("minecraft:particle_initial_spin", |rng| {
        molang_members(rng, &["rotation", "rotation_rate"])
    }),
    ("minecraft:particle_initialization", |rng| {
        molang_members(rng, &["per_update_expression", "per_render_expression"])
    }),
    ("minecraft:particle_kill_plane", |rng| {
        Body::Value(rng.pick(PLANE4).to_owned())
    }),
    ("minecraft:particle_lifetime_events", |rng| {
        let mut m = Vec::new();
        member(rng, "creation_event", EVENTS, &mut m);
        member(rng, "expiration_event", EVENTS, &mut m);
        member(rng, "timeline", TIMELINE, &mut m);
        Body::Members(m)
    }),
    ("minecraft:particle_lifetime_expression", |rng| {
        molang_members(rng, &["expiration_expression", "max_lifetime"])
    }),
    ("minecraft:particle_motion_collision", |rng| {
        let mut m = Vec::new();
        member(rng, "enabled", MOLANG, &mut m);
        member(rng, "collision_drag", FLOAT, &mut m);
        member(rng, "coefficient_of_restitution", FLOAT, &mut m);
        required(rng, "collision_radius", FLOAT, &mut m);
        member(rng, "expire_on_contact", BOOL, &mut m);
        member(rng, "events", COLLISION_EVENTS, &mut m);
        Body::Members(m)
    }),
    ("minecraft:particle_motion_dynamic", |rng| {
        let mut m = Vec::new();
        array_member(rng, "linear_acceleration", 3, &mut m);
        member(rng, "linear_drag_coefficient", MOLANG, &mut m);
        member(rng, "rotation_acceleration", MOLANG, &mut m);
        member(rng, "rotation_drag_coefficient", MOLANG, &mut m);
        Body::Members(m)
    }),
    ("minecraft:particle_motion_parametric", |rng| {
        let mut m = Vec::new();
        array_member(rng, "relative_position", 3, &mut m);
        array_member(rng, "direction", 3, &mut m);
        member(rng, "rotation", MOLANG, &mut m);
        Body::Members(m)
    }),
];

/// The generator's list is the family's component table: a component added to the family
/// without a generator fails here.
#[test]
fn the_generator_names_every_component_of_the_family() {
    use bedrock_addon_core::versioning::Family;
    use bedrock_addon_family_particle::history::ParticleFamily;
    let table: Vec<&str> = <ParticleFamily as Family>::components()
        .0
        .iter()
        .map(|row| row.name)
        .collect();
    let generated: Vec<&str> = COMPONENTS.iter().map(|(name, _)| *name).collect();
    assert_eq!(generated, table);
    assert_eq!(generated.len(), 29);
}

/// One generated component, as `"name": value`: one of the family's, or (one time in ten) a
/// key the family does not know.
fn component(rng: &mut Rng) -> String {
    if rng.chance(10) {
        let value = rng.pick(&["{}", "1", r#"{"a": [1]}"#]);
        let name = rng.pick(&[
            "demo:unknown",
            "minecraft:particle_appearance_camera_distance_fade",
            "",
        ]);
        return format!(r#""{name}": {value}"#);
    }
    let (name, body) = COMPONENTS[usize::try_from(rng.next() % COMPONENTS.len() as u64).unwrap()];
    let mut m = match body(rng) {
        Body::Members(m) => m,
        Body::Value(value) => return format!(r#""{name}": {value}"#),
    };
    if rng.spoil(12) {
        return format!(r#""{name}": {}"#, rng.pick(&["1", "[]", "null"]));
    }
    if rng.chance(10) {
        m.push(r#""unknown": 1"#.to_owned());
    }
    format!(r#""{name}": {}"#, object(rng, &m))
}

fn curve(rng: &mut Rng) -> String {
    let kind = rng.pick(&["linear", "bezier", "catmull_rom", "bezier_chain", "spiral"]);
    let mut m = Vec::new();
    if !rng.spoil(8) {
        m.push(format!(r#""type": "{kind}""#));
    }
    required(rng, "input", MOLANG, &mut m);
    member(rng, "horizontal_range", MOLANG, &mut m);
    let nodes = if kind == "bezier_chain" {
        rng.pick(CHAIN_NODES).to_owned()
    } else if rng.spoil(8) {
        "1".to_owned()
    } else {
        let count = (rng.next() % 6) as usize;
        let nodes: Vec<&str> = (0..count).map(|_| rng.pick(MOLANG)).collect();
        format!("[{}]", nodes.join(", "))
    };
    if !rng.spoil(8) {
        m.push(format!(r#""nodes": {nodes}"#));
    }
    object(rng, &m)
}

/// The forms of `log`: plain, and with every escape a JSON string has.
const LOG: &[&str] = &[
    r#""x""#,
    r#""a \"quoted\" \\ back\/slash,\ttab\nline é 😀 {}""#,
    r#""""#,
    BAD,
    "1",
];

fn event_members(rng: &mut Rng, depth: u32) -> Vec<String> {
    let mut m = Vec::new();
    member(rng, "expression", MOLANG, &mut m);
    member(rng, "log", LOG, &mut m);
    member(rng, "particle_effect", EFFECT, &mut m);
    member(rng, "sound_effect", SOUND, &mut m);
    if depth < 3 && rng.chance(3) {
        let nodes: Vec<String> = (0..rng.next() % 3)
            .map(|_| {
                let members = event_members(rng, depth + 1);
                object(rng, &members)
            })
            .collect();
        m.push(format!(r#""sequence": [{}]"#, nodes.join(", ")));
    }
    if depth < 3 && rng.chance(3) {
        let nodes: Vec<String> = (0..rng.next() % 3)
            .map(|_| {
                let mut option = event_members(rng, depth + 1);
                let weight = rng.pick(WEIGHT);
                if !weight.is_empty() {
                    option.insert(0, weight.to_owned());
                }
                object(rng, &option)
            })
            .collect();
        m.push(format!(r#""randomize": [{}]"#, nodes.join(", ")));
    }
    m
}

/// One generated document.
fn generated(rng: &mut Rng) -> String {
    rng.clean = !rng.chance(3);
    let version = rng.pick(&["1.10.0", "1.20.79", "1.20.80", "1.26.10", "1.26.30"]);
    let mut components: Vec<String> = Vec::new();
    if !rng.chance(6) {
        components.push(r#""minecraft:emitter_lifetime_once": {"active_time": 1}"#.to_owned());
    }
    if !rng.spoil(6) {
        components.push(format!(
            r#""minecraft:emitter_rate_instant": {{"num_particles": {}}}"#,
            rng.pick(MOLANG)
        ));
    }
    // A component may come again: a member written twice keeps its last copy before
    // anything reads it, so the value has the last one.
    for _ in 0..rng.next() % 8 {
        components.push(component(rng));
    }
    let mut payload = Vec::new();
    if !rng.spoil(20) {
        payload.push(format!(r#""description": {}"#, rng.pick(DESCRIPTION)));
    }
    if !rng.spoil(20) {
        payload.push(format!(r#""components": {}"#, object(rng, &components)));
    }
    if rng.chance(2) {
        let curves: Vec<String> = (0..rng.next() % 3)
            .map(|i| format!(r#""variable.c{i}": {}"#, curve(rng)))
            .collect();
        payload.push(format!(r#""curves": {}"#, object(rng, &curves)));
    }
    if rng.chance(2) {
        let events: Vec<String> = (0..rng.next() % 3)
            .map(|i| {
                let members = event_members(rng, 0);
                format!(r#""e{i}": {}"#, object(rng, &members))
            })
            .collect();
        payload.push(format!(r#""events": {}"#, object(rng, &events)));
    }
    if rng.chance(15) {
        payload.push(r#""unknown": [1]"#.to_owned());
    }
    let payload = object(rng, &payload);
    let root = vec![
        format!(r#""format_version": "{version}""#),
        format!(r#""particle_effect": {payload}"#),
    ];
    object(rng, &root)
}

/// The error-level findings.
/// The findings that refuse: a kept error refuses nothing.
fn error_count(d: &Diagnostics) -> usize {
    d.iter().filter(|f| f.rejects()).count()
}

/// One difference between a document as authored and as written back.
#[derive(Debug)]
enum Difference {
    /// A number written as another number.
    Number(f64, f64),
    /// A member of the input that the output lacks.
    Gone,
    /// A member of the output that the input lacks.
    Added,
    /// Any other change of a value.
    Changed,
}

/// The differences between `input` and `output` as JSON values, member for member, each at
/// its `/`-separated path. Members are matched by key, not by position: their order in the
/// written text is the model's.
fn differences(input: &Value, output: &Value, path: &str, into: &mut Vec<(String, Difference)>) {
    if let (Some(input), Some(output)) = (input.as_object(), output.as_object()) {
        for (key, value) in input {
            let at = format!("{path}/{}", key.as_str());
            match output.get(key.as_str()) {
                Some(written) => differences(value, written, &at, into),
                None => into.push((at, Difference::Gone)),
            }
        }
        for (key, _) in output {
            if input.get(key.as_str()).is_none() {
                into.push((format!("{path}/{}", key.as_str()), Difference::Added));
            }
        }
    } else if let (Some(input), Some(output)) = (input.as_array(), output.as_array()) {
        if input.len() == output.len() {
            for (i, (value, written)) in input.iter().zip(output.iter()).enumerate() {
                differences(value, written, &format!("{path}/{i}"), into);
            }
        } else {
            into.push((path.to_owned(), Difference::Changed));
        }
    } else if let (Some(input), Some(output)) = (input.as_number(), output.as_number()) {
        let (input, output) = (input.to_f64_lossy(), output.to_f64_lossy());
        #[expect(clippy::float_cmp, reason = "any difference of the two doubles is one")]
        let differ = input != output;
        if differ {
            into.push((path.to_owned(), Difference::Number(input, output)));
        }
    } else if input != output {
        into.push((path.to_owned(), Difference::Changed));
    }
}

/// How often each recorded rewrite was seen between a document as authored and as written.
#[derive(Debug, Default)]
struct Rewrites {
    /// Documents compared: read at 1.20.80 or above.
    compared: usize,
    /// Of them, written exactly as authored but for `format_version`.
    as_authored: usize,
    /// `format_version` written as the newest, 1.20.80.
    format_version: usize,
    /// Documents authored at 1.20.80, whose `format_version` is written as it is.
    already_newest: usize,
    /// A number written as its nearest `f32`, without a finding (`16777217` as `16777216`).
    nearest_f32: usize,
    /// A number beyond `f32` written as the largest finite float (a warning).
    clamped: usize,
    /// `emitter_local_space`: `rotation` or `velocity` written `false` (a warning).
    local_space: usize,
    /// A curve of an unknown type, left out (a warning).
    curve_left_out: usize,
    /// a curve whose node count the game does not take, written with the nodes it
    /// has (no difference; counted so that the case is known to occur).
    curve_nodes_as_written: usize,
}

/// Classes the difference at `path`, which must be one of the recorded rewrites, each of
/// which but the first two comes with a finding of the load.
fn record(rewrites: &mut Rewrites, path: &str, difference: &Difference, input: &Value, what: &str) {
    let steps: Vec<&str> = path.split('/').collect();
    match (difference, steps.as_slice()) {
        (Difference::Changed, ["", "format_version"]) => rewrites.format_version += 1,
        (Difference::Number(read, written), _) => {
            #[allow(
                clippy::cast_possible_truncation,
                reason = "the nearest f32 is the reading"
            )]
            let nearest = *read as f32;
            if nearest.is_finite() {
                // The shortest decimal of the nearest `f32`.
                let shortest: f64 = format!("{nearest:?}").parse().unwrap();
                assert_eq!(*written, shortest, "{path}: {what}");
                rewrites.nearest_f32 += 1;
            } else {
                let largest: f64 = format!("{:?}", f32::MAX.copysign(nearest)).parse().unwrap();
                assert_eq!(*written, largest, "{path}: {what}");
                rewrites.clamped += 1;
            }
        }
        (
            Difference::Changed,
            [
                "",
                "particle_effect",
                "components",
                "minecraft:emitter_local_space",
                "rotation" | "velocity",
            ],
        ) => rewrites.local_space += 1,
        (Difference::Gone, ["", "particle_effect", "curves", name]) => {
            let kind = at(input, &format!("particle_effect/curves/{name}/type"));
            let kind = kind
                .and_then(|kind| kind.as_string())
                .map(facet_value::VString::as_str);
            assert_eq!(kind, Some("spiral"), "{path}: {what}");
            rewrites.curve_left_out += 1;
        }
        _ => panic!("{path}: {difference:?} is no recorded rewrite\n{what}"),
    }
}

/// The written text compared with the text as authored, for a document read at 1.20.80 or
/// above (a 1.10.0 one is written upgraded, which `p06` and the step tests pin).
fn compare_as_authored(
    rewrites: &mut Rewrites,
    authored: &str,
    document: &ParticleEffect,
    written: &str,
    warnings: usize,
) {
    // Of a member written twice the last copy is the member, as for the loader.
    let input = json(&bedrock_addon_core::json::last_wins(authored));
    let output = json(written);
    let mut found = Vec::new();
    differences(&input, &output, "", &mut found);
    let what = format!("{authored}\n{written}");
    let before = (
        rewrites.clamped,
        rewrites.local_space,
        rewrites.curve_left_out,
    );
    for (path, difference) in &found {
        record(rewrites, path, difference, &input, &what);
    }
    rewrites.compared += 1;
    rewrites.already_newest += usize::from(authored.contains(r#""format_version": "1.20.80""#));
    rewrites.as_authored += usize::from(found.iter().all(|(path, _)| path == "/format_version"));
    // A rewrite with a finding is never silent.
    let with_finding = (
        rewrites.clamped,
        rewrites.local_space,
        rewrites.curve_left_out,
    );
    assert!(with_finding == before || warnings > 0, "{what}");
    let curves = document.curves.iter().flat_map(|curves| curves.values());
    for curve in curves {
        if let Curve::Simple(curve) = curve {
            rewrites.curve_nodes_as_written += usize::from(!curve.has_valid_node_count());
        }
    }
}

/// Six thousand generated documents over the forms of every member of every component,
/// valid and not, in both eras, each written plainly and with comments, members written
/// twice and escaped keys: the two texts load alike (the same value and findings, or both
/// rejected), and each one that loads writes text that loads again, without an error, to an
/// equal value, and writes the same text again; or its write is refused with a path.
#[test]
fn every_generated_document_that_loads_writes_text_that_loads_back_equal() {
    let mut rng = Rng {
        state: 0x9E37_79B9_7F4A_7C15,
        clean: false,
        decorate: false,
        style: 0xD1B5_4A32_D192_ED03,
    };
    let cx = context();
    let (mut loaded_count, mut rejected, mut refused, mut legacy, mut warned) = (0, 0, 0, 0, 0);
    let mut decorated_count = 0;
    let mut rewrites = Rewrites::default();
    for case in 0..6000 {
        // The same content twice: plain, then decorated.
        let state = rng.state;
        rng.decorate = false;
        let plain = generated(&mut rng);
        rng.state = state;
        rng.decorate = true;
        let text = generated(&mut rng);
        decorated_count += usize::from(text != plain);
        let (plain_loaded, loaded) = (load(&plain, &cx), load(&text, &cx));
        match (&plain_loaded, &loaded) {
            (Ok(plain_loaded), Ok(loaded)) => {
                assert_eq!(
                    loaded.value(),
                    plain_loaded.value(),
                    "case {case}\n{plain}\n{text}"
                );
                assert_eq!(
                    super::common::texts(loaded.diagnostics()),
                    super::common::texts(plain_loaded.diagnostics()),
                    "case {case}\n{plain}\n{text}"
                );
            }
            (Err(plain_rejected), Err(rejected)) => assert_eq!(
                super::common::texts(&rejected.diagnostics),
                super::common::texts(&plain_rejected.diagnostics),
                "case {case}\n{plain}\n{text}"
            ),
            _ => panic!("case {case}: one text loads and the other does not\n{plain}\n{text}"),
        }
        let Ok(document) = loaded else {
            rejected += 1;
            continue;
        };
        loaded_count += 1;
        legacy += usize::from(document.version().upgraded);
        warned += usize::from(!findings(document.diagnostics()).is_empty());
        match document.to_json() {
            Ok(rewritten) => {
                if !document.version().upgraded {
                    let warnings = findings(document.diagnostics()).len();
                    compare_as_authored(&mut rewrites, &plain, &document, &rewritten, warnings);
                }
                let again = load(&rewritten, &cx).unwrap_or_else(|rejected| {
                    panic!(
                        "case {case}: the text written does not load\n{text}\n{rewritten}\n{:#?}",
                        super::common::texts(&rejected.diagnostics)
                    )
                });
                assert_eq!(error_count(again.diagnostics()), 0, "case {case}");
                assert_eq!(
                    again.value(),
                    document.value(),
                    "case {case}\n{text}\n{rewritten}"
                );
                assert_eq!(again.to_json().unwrap(), rewritten, "case {case}");
            }
            Err(EncodeError::NotRepresentable { at, .. }) => {
                assert!(
                    !at.is_empty(),
                    "case {case}: a refusal without a path\n{text}"
                );
                refused += 1;
            }
            Err(other) => panic!("case {case}: {other:?}"),
        }
    }
    eprintln!(
        "generated: {loaded_count} loaded ({legacy} upgraded from 1.10.0, {warned} with findings), {rejected} rejected, {refused} refused by the writer, {decorated_count} decorated"
    );
    // Both eras, clean and with findings, and rejections; what loads has a text.
    let cereal = loaded_count - legacy;
    assert!(legacy > 1500, "{legacy}");
    // A 1.20.80 document with an unknown member or component is refused.
    assert!(cereal > 1000, "{cereal}");
    assert!(warned > 1000, "{warned}");
    assert!(rejected > 500, "{rejected}");
    assert!(decorated_count > 3000, "{decorated_count}");
    // As authored: every difference is a recorded rewrite, and each of them occurs.
    eprintln!("as authored: {rewrites:?}");
    assert_eq!(rewrites.compared, cereal);
    assert_eq!(rewrites.format_version, cereal - rewrites.already_newest);
    assert!(rewrites.as_authored > 300, "{rewrites:?}");
    assert!(rewrites.nearest_f32 > 100, "{rewrites:?}");
    assert!(rewrites.clamped > 100, "{rewrites:?}");
    assert!(rewrites.local_space > 20, "{rewrites:?}");
    assert!(rewrites.curve_left_out > 20, "{rewrites:?}");
    assert!(rewrites.curve_nodes_as_written > 100, "{rewrites:?}");
    assert_eq!(refused, 0);
}

/// A 1.10.0 file the legacy parser stops in is written whole: what the game
/// registered, in the newest format. The writer adds the members the 1.20.80 reader requires
/// and the author did not write (an empty `material` / `texture`, an empty `components`),
/// leaves out what the parser never reached, and the text it writes has no line left to give
/// on reload: one test case per place the parser stops at.
#[test]
fn a_legacy_effect_the_parser_stopped_in_is_written_whole_at_its_defaults() {
    let payload = |members: &str| {
        format!(r#"{{"format_version": "1.10.0", "particle_effect": {{{members}}}}}"#)
    };
    let rest = format!(
        r#""components": {{{BASE}}}, "curves": {{"variable.c": {{"type": "linear", "input": 0, "nodes": [0]}}}}, "events": {{"e": {{"log": "x"}}}}"#
    );
    let described = r#""description": {"identifier": "demo:x", "basic_render_parameters": {"material": "m", "texture": "t"}}"#;
    let with = |parameters: &str| {
        payload(&format!(
            r#""description": {{"identifier": "demo:x", "basic_render_parameters": {parameters}}}, {rest}"#
        ))
    };
    let brp = "particle_effect/description/basic_render_parameters";
    let not_found = format!(
        "Error {brp} particle/no_render_parameters: basic_render_parameters not found when reading in particle effect"
    );
    let no_components = "Error particle_effect/components particle/no_components: no components found when reading in particle effect".to_owned();
    // The written document of an effect that has only its description.
    let bare = |material: &str, texture: &str| {
        format!(
            r#"{{"format_version": "1.20.80", "particle_effect": {{"description": {{"identifier": "demo:x", "basic_render_parameters": {{"material": "{material}", "texture": "{texture}"}}}}, "components": {{}}}}}}"#
        )
    };
    // The curves and the event as the upgrade writes them (`expression: 0`).
    let read_before_components = r#"{"format_version": "1.20.80", "particle_effect": {"description": {"identifier": "demo:x", "basic_render_parameters": {"material": "m", "texture": "t"}}, "components": {}, "curves": {"variable.c": {"type": "linear", "input": 0, "nodes": [0]}}, "events": {"e": {"expression": 0, "log": "x"}}}}"#;
    for (stop, text, line, expected) in [
        (
            "no basic_render_parameters",
            payload(&format!(
                r#""description": {{"identifier": "demo:x"}}, {rest}"#
            )),
            not_found.clone(),
            bare("", ""),
        ),
        (
            "basic_render_parameters that is no object",
            with("[]"),
            not_found,
            bare("", ""),
        ),
        (
            "no material",
            with(r#"{"texture": "t"}"#),
            format!("Error {brp}/material particle/no_material: particle has no material!"),
            bare("", "t"),
        ),
        (
            "a material that is no string",
            with(r#"{"material": 1, "texture": "t"}"#),
            format!(
                "Error {brp}/material particle/material_not_string: particle material field isn't a string"
            ),
            bare("", "t"),
        ),
        (
            "no texture",
            with(r#"{"material": "m"}"#),
            format!("Error {brp}/texture particle/no_texture: particle has no texture!"),
            bare("m", ""),
        ),
        (
            "no components",
            payload(&format!(
                r#"{described}, "curves": {{"variable.c": {{"type": "linear", "input": 0, "nodes": [0]}}}}, "events": {{"e": {{"log": "x"}}}}"#
            )),
            no_components.clone(),
            read_before_components.to_owned(),
        ),
        (
            "components that is no object",
            payload(&format!(
                r#"{described}, "components": [], "curves": {{"variable.c": {{"type": "linear", "input": 0, "nodes": [0]}}}}, "events": {{"e": {{"log": "x"}}}}"#
            )),
            no_components,
            read_before_components.to_owned(),
        ),
    ] {
        let effect = loaded(&text);
        // Read: the parser's one line, where it stopped, and the note of each member it did
        // not get to.
        let (lines, notes): (Vec<String>, Vec<String>) = findings(effect.diagnostics())
            .into_iter()
            .partition(|found| found.starts_with("Error"));
        assert_eq!(lines, [line], "{stop}");
        assert!(
            notes
                .iter()
                .all(|note| note.starts_with("Info particle_effect/")
                    && note.ends_with(" core/dropped_field: field dropped")),
            "{stop}: {notes:#?}"
        );
        // Written: exactly this document.
        let text = effect.to_json().unwrap();
        assert_eq!(json(&text), json(&expected), "{stop}\n{text}");
        // Read again: the same effect, and nothing left to say, because the file now has
        // the members whose absence the line was about.
        let again = loaded(&text);
        assert_eq!(again.value(), effect.value(), "{stop}");
        assert_eq!(findings(again.diagnostics()), [] as [&str; 0], "{stop}");
        assert!(!again.version().upgraded, "{stop}");
    }
}

/// What is written back for an integer literal too long for 64 bits: the author's
/// digits are not kept. A typed member holds the nearest `f32` and writes its shortest
/// decimal; a member the family does not know is not kept.
#[test]
fn a_wide_integer_is_written_back_as_the_shortest_decimal_of_the_float_it_was_read_as() {
    const DIGITS: &str = "123456789012345678901";
    // The member the family does not know, in a 1.10.0 document: told, not kept.
    let legacy = loaded(&effect(
        LEGACY,
        &format!(
            r#""minecraft:particle_motion_collision": {{"collision_radius": 0.1, "unknown": {DIGITS}}}"#
        ),
    ));
    assert_eq!(
        at(
            &written(&legacy),
            "particle_effect/components/minecraft:particle_motion_collision/unknown"
        ),
        None
    );
    for version in [LEGACY, CEREAL] {
        let effect = loaded(&effect(
            version,
            &format!(
                r#""minecraft:particle_motion_collision": {{"collision_radius": 0.1, "collision_drag": {DIGITS}, "enabled": {DIGITS}}}"#
            ),
        ));
        let text = effect.to_json().unwrap();
        assert!(!text.contains(DIGITS), "{version}: {text}");
        let written = json(&text);
        let number = |member: &str| {
            let pointer =
                format!("particle_effect/components/minecraft:particle_motion_collision/{member}");
            at(&written, &pointer)
                .and_then(|value| value.as_number())
                .map(facet_value::VNumber::to_f64_lossy)
        };
        // The nearest `f32` is 1.2345679e20; its shortest decimal, read as a JSON number.
        assert_eq!(number("collision_drag"), Some(1.234_567_9e20), "{version}");
        assert_eq!(number("enabled"), Some(1.234_567_9e20), "{version}");
        // And stable from there: the written text reads to the same value and writes itself.
        let again = loaded(&text);
        assert_eq!(again.value(), effect.value(), "{version}");
        assert_eq!(again.to_json().unwrap(), text, "{version}");
    }
}

/// The same value read from both eras writes the same 1.20.80 members where the upgrade adds
/// none.
#[test]
fn a_legacy_document_is_written_as_a_1_20_80_document() {
    let legacy = loaded(&effect(LEGACY, ""));
    let text = legacy.to_json().unwrap();
    assert!(text.contains(r#""format_version": "1.20.80""#));
    let again = loaded(&text);
    assert_eq!(again.value(), legacy.value());
    assert!(!again.version().upgraded);
}

/// An unknown member may hold any value; one nested deeper than a file may be has no text.
#[test]
fn an_unknown_member_nested_past_the_limit_has_no_text() {
    use bedrock_addon_family_particle::history::ParticleHistory;
    let nested = |depth: usize| {
        let mut value = Value::from(1_i64);
        for _ in 0..depth {
            value = Value::from(facet_value::VArray::from_iter([value]));
        }
        value
    };
    // A 1.20.80 value has no place for an unknown member at any depth.
    let mut effect = base();
    effect.extra.insert("deep".to_owned(), nested(1));
    match effect.to_json() {
        Err(EncodeError::NotRepresentable { at, what }) => {
            assert_eq!(what, Unrepresentable::Unsupported);
            assert_eq!(at.to_string(), "particle_effect/deep");
        }
        other => panic!("{other:?}"),
    }
    // A 1.10.0 value built in code writes it (its own writer); the reader reads past it and
    // keeps nothing. `{ particle_effect {` is two levels above it.
    let legacy = effect_with(LEGACY, "");
    let read = || {
        let document =
            ParticleEffect::parse_as_written(&super::common::path(), legacy.as_bytes(), &context())
                .unwrap();
        match document.into_value() {
            ParticleHistory::V1_10_0(effect) => effect,
            other @ ParticleHistory::V1_20_80(_) => panic!("{other:?}"),
        }
    };
    let mut old = read();
    old.extra.insert("deep".to_owned(), nested(126));
    let text = ParticleHistory::V1_10_0(old.clone()).to_json().unwrap();
    let again =
        ParticleEffect::parse_as_written(&super::common::path(), text.as_bytes(), &context())
            .unwrap();
    assert_eq!(again.value(), &ParticleHistory::V1_10_0(read()));
    old.extra.insert("deep".to_owned(), nested(127));
    // The writer of core refuses it where the text would pass the limit: the array
    // 126 arrays below the member.
    match ParticleHistory::V1_10_0(old).to_json() {
        Err(EncodeError::NotRepresentable { at, what }) => {
            assert_eq!(what, Unrepresentable::TooDeep);
            assert_eq!(
                at.to_string(),
                format!("particle_effect/deep{}", "/0".repeat(126))
            );
        }
        other => panic!("{other:?}"),
    }
}
