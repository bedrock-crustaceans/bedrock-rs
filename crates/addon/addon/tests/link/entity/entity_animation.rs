//! 20 E8: the animations and animation controllers an entity's `description.animations`
//! names, on the behaviour stack.

#![allow(unused_imports, reason = "each feature set compiles part of the tests")]

use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link, link_with};
use super::{FILE, kept_unresolved, said_of};

const WALK: &str = r#"{"format_version": "1.10.0", "animations": {"animation.demo.walk": {"loop": true, "timeline": {"0.0": "/say hi"}}}}"#;
const MOVE: &str = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.demo.move": {"initial_state": "default", "states": {"default": {}}}}}"#;
/// The client's animation of the same world: no answer for a behaviour-pack entity.
const FLY: &str =
    r#"{"format_version": "1.8.0", "animations": {"animation.demo.fly": {"loop": true}}}"#;

fn pet(animations: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.50", "minecraft:entity": {{"description": {{"identifier": "demo:pet", "animations": {{{animations}}}, "scripts": {{"animate": ["walk"]}}}}, "components": {{}}}}}}"#
    )
}

/// The prefix of the name picks the kind: `animation.` an animation, `controller.animation.`
/// a controller. Each is looked up among the behaviour packs' own, as written. A name nothing
/// defines is "Error: can't find animation %s" when the key is played
/// (the game's animation player),
/// and the entity goes on: a kept error.
#[cfg(all(feature = "animation", feature = "animation_controller"))]
#[test]
fn e8_an_animation_or_controller_no_behavior_pack_defines_is_a_kept_error() {
    let text = pet(
        r#""walk": "animation.demo.walk", "move": "controller.animation.demo.move",
           "none": "animation.demo.none", "lost": "controller.animation.demo.lost",
           "fly": "animation.demo.fly", "case": "animation.demo.Walk",
           "swap": "controller.animation.demo.walk""#,
    );
    let found = link(
        &[
            (FILE, &text),
            ("animations/walk.json", WALK),
            ("animation_controllers/move.json", MOVE),
        ],
        &[("animations/fly.animation.json", FLY)],
    );
    let a = |key: &str, id: &str| {
        kept_unresolved(&format!("description/animations/{key}"), "animation", id)
    };
    assert_eq!(
        said_of(&found, "animation"),
        [
            a("none", "animation.demo.none"),
            a("lost", "controller.animation.demo.lost"),
            a("fly", "animation.demo.fly"),
            a("case", "animation.demo.Walk"),
            a("swap", "controller.animation.demo.walk"),
        ]
    );
    assert!(found.iter().all(|finding| finding.kept), "{found:?}");
}

/// A name with neither prefix is no animation reference the pass can place: nothing is told.
/// With a host, the host answers for the game's own animations.
#[cfg(all(feature = "animation", feature = "animation_controller"))]
#[test]
fn e8_a_name_without_a_prefix_is_not_checked_and_a_host_answers_for_the_games() {
    let text = pet(r#""odd": "demo.walk", "sit": "animation.game.sit""#);
    let files = [(FILE, &*text)];
    let found = link(&files, &[]);
    assert_eq!(
        said_of(&found, "animation"),
        [kept_unresolved(
            "description/animations/sit",
            "animation",
            "animation.game.sit"
        )]
    );
    let found = link_with(
        &files,
        &[],
        hosted(&[("animation", "animation.game.sit")]),
        LinkOptions::default(),
    );
    assert!(said_of(&found, "animation").is_empty(), "{found:?}");
}

#[cfg(not(all(feature = "animation", feature = "animation_controller")))]
#[test]
fn e8_without_the_animation_feature_no_animation_is_told() {
    let text = pet(r#""none": "animation.demo.none""#);
    let found = link(&[(FILE, &text)], &[]);
    assert!(said_of(&found, "animation").is_empty(), "{found:?}");
}
