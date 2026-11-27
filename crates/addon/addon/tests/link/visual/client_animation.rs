//! 20 R15, R16, R18 and R19: the effect keys of an animation and of an animation controller
//! against the client entities that play them, and the names nothing checks.

use bedrock_addon::diagnostic::{DiagnosticDetail, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{in_file, link_with, stacks};
use super::{Told, animation, animation_controller, client_entity, entity, says, told};

const ANIMATION: &str = "animations/x.animation.json";
const CONTROLLER: &str = "animation_controllers/x.animation_controllers.json";
const INFO: Severity = Severity::Info;
const UNRESOLVED: &str = "core/unresolved_reference";

/// The client entity `id` with the effect maps `effects`, playing `plays` (the value of its
/// one `animations` entry).
fn player(id: &str, effects: &str, plays: &str) -> String {
    let comma = if effects.is_empty() { "" } else { ", " };
    client_entity(
        id,
        &format!(r#"{effects}{comma}"animations": {{"a": "{plays}"}}"#),
    )
}

/// What linking says in `file` of a world with the client entities `entities` and the
/// animation or controller file `text` at `file`.
fn linked(entities: &[&str], file: &'static str, text: &str, lints: bool) -> Vec<Told> {
    const ENTITY_FILES: [&str; 2] = ["entity/one.entity.json", "entity/two.entity.json"];
    let pig = entity("minecraft:pig");
    let mut resource = vec![(file, text)];
    for (path, entity) in ENTITY_FILES.iter().zip(entities) {
        resource.push((*path, *entity));
    }
    let found = link_with(
        &[("entities/pig.json", &pig)],
        &resource,
        LoadOptions::default(),
        LinkOptions {
            lints,
            ..Default::default()
        },
    );
    told(&found, "rp 2", file)
}

const EFFECTS: &str =
    r#""particle_effects": {"Smoke": "demo:smoke"}, "sound_effects": {"Step": "demo.step"}"#;

// ---- R15: animations -------------------------------------------------------------------------

#[test]
fn r15_an_effect_that_is_a_key_of_the_playing_client_entity_resolves_in_lower_case() {
    // Both sides are lower-cased: the entity's key and the animation's effect.
    let text = animation(
        "animation.x",
        r#""particle_effects": {"0.0": {"effect": "smoke"}, "1.0": [{"effect": "SMOKE"}]}, "sound_effects": {"0.0": {"effect": "STEP"}}"#,
    );
    let pig = player("minecraft:pig", EFFECTS, "Animation.X");
    assert_eq!(linked(&[&pig], ANIMATION, &text, true), []);
}

#[test]
fn r15_an_effect_that_is_no_key_of_the_playing_client_entity_is_a_note_and_a_lint() {
    let text = animation(
        "animation.x",
        r#""particle_effects": {"0.0": {"effect": "Dust"}, "1.5": [{"effect": "smoke"}, {"effect": "ash"}]}, "sound_effects": {"0.0": [{"effect": "step"}, {"effect": "Nowhere"}], "2.0": {"effect": "roar"}}"#,
    );
    let pig = player("minecraft:pig", EFFECTS, "animation.x");
    let at = |tail: &str| format!("animations/animation.x/{tail}");
    assert_eq!(
        linked(&[&pig], ANIMATION, &text, true),
        [
            says(
                UNRESOLVED,
                INFO,
                &at("particle_effects/0.0/effect"),
                "particle effect",
                "dust"
            ),
            says(
                UNRESOLVED,
                INFO,
                &at("particle_effects/1.5/1/effect"),
                "particle effect",
                "ash"
            ),
            says(
                UNRESOLVED,
                INFO,
                &at("sound_effects/0.0/1/effect"),
                "sound effect",
                "nowhere"
            ),
            says(
                UNRESOLVED,
                INFO,
                &at("sound_effects/2.0/effect"),
                "sound effect",
                "roar"
            ),
        ]
    );
    assert_eq!(linked(&[&pig], ANIMATION, &text, false), []);
}

/// A map the entity does not have at all has no key.
#[test]
fn r15_an_effect_of_a_client_entity_without_the_map_is_a_note() {
    let text = animation(
        "animation.x",
        r#""particle_effects": {"0.0": {"effect": "smoke"}}"#,
    );
    let pig = player("minecraft:pig", "", "animation.x");
    assert_eq!(linked(&[&pig], ANIMATION, &text, true).len(), 1);
}

#[test]
fn r15_an_animation_no_client_entity_plays_is_not_checked() {
    let text = animation(
        "animation.x",
        r#""particle_effects": {"0.0": {"effect": "dust"}}"#,
    );
    let pig = player("minecraft:pig", EFFECTS, "animation.other");
    assert_eq!(linked(&[&pig], ANIMATION, &text, true), []);
    // A controller of the same name is another thing.
    let pig = player("minecraft:pig", EFFECTS, "controller.animation.x");
    assert_eq!(linked(&[&pig], ANIMATION, &text, true), []);
}

/// One finding per effect, however many of the entities that play the animation lack it.
#[test]
fn r15_an_effect_one_of_two_playing_client_entities_lacks_is_told_once() {
    let text = animation(
        "animation.x",
        r#""particle_effects": {"0.0": {"effect": "smoke"}, "1.0": {"effect": "dust"}}"#,
    );
    let pig = player("minecraft:pig", EFFECTS, "animation.x");
    let cow = player("minecraft:cow", "", "animation.x");
    let found = linked(&[&pig, &cow], ANIMATION, &text, true);
    let names: Vec<&str> = found.iter().map(|said| said.4.as_str()).collect();
    assert_eq!(names, ["smoke", "dust"]);
}

/// 20 R15 names the client entity: an attachable that plays the animation is not asked.
#[cfg(feature = "attachable")]
#[test]
fn r15_an_animation_only_an_attachable_plays_is_not_checked() {
    let text = animation(
        "animation.x",
        r#""particle_effects": {"0.0": {"effect": "dust"}}"#,
    );
    let wand = super::attachable("minecraft:bow", r#""animations": {"a": "animation.x"}"#);
    let found = link_with(
        &[],
        &[(ANIMATION, &text), ("attachables/bow.json", &wand)],
        LoadOptions::default(),
        LinkOptions::default(),
    );
    assert_eq!(told(&found, "rp 2", ANIMATION), []);
}

// ---- R18: animation controllers --------------------------------------------------------------

#[test]
fn r18_a_state_effect_that_is_a_key_of_the_playing_client_entity_resolves_in_lower_case() {
    let text = animation_controller(
        "controller.animation.x",
        r#""particle_effects": [{"effect": "SMOKE"}], "sound_effects": [{"effect": "step"}]"#,
    );
    let pig = player("minecraft:pig", EFFECTS, "Controller.Animation.X");
    assert_eq!(linked(&[&pig], CONTROLLER, &text, true), []);
}

#[test]
fn r18_a_state_effect_that_is_no_key_of_the_playing_client_entity_is_a_note_and_a_lint() {
    let text = animation_controller(
        "controller.animation.x",
        r#""particle_effects": [{"effect": "smoke"}, {"effect": "Dust"}], "sound_effects": [{"effect": "Nowhere"}, {"effect": "step"}]"#,
    );
    let pig = player("minecraft:pig", EFFECTS, "controller.animation.x");
    let at =
        |tail: &str| format!("animation_controllers/controller.animation.x/states/default/{tail}");
    assert_eq!(
        linked(&[&pig], CONTROLLER, &text, true),
        [
            says(
                UNRESOLVED,
                INFO,
                &at("particle_effects/1/effect"),
                "particle effect",
                "dust"
            ),
            says(
                UNRESOLVED,
                INFO,
                &at("sound_effects/0/effect"),
                "sound effect",
                "nowhere"
            ),
        ]
    );
    assert_eq!(linked(&[&pig], CONTROLLER, &text, false), []);
}

#[test]
fn r18_a_controller_no_client_entity_plays_is_not_checked() {
    let text = animation_controller(
        "controller.animation.x",
        r#""particle_effects": [{"effect": "dust"}]"#,
    );
    let pig = player("minecraft:pig", EFFECTS, "animation.x");
    assert_eq!(linked(&[&pig], CONTROLLER, &text, true), []);
}

// ---- R16 and R19: not checked ----------------------------------------------------------------

/// Whether a finding of the load or of the link pass in `file` names a reference.
fn any_reference(
    file: &'static str,
    text: &str,
    load: impl Fn(&bedrock_addon::pack::StackSet) -> Vec<bedrock_addon::diagnostic::Diagnostic>,
) -> bool {
    let pig = player(
        "minecraft:pig",
        EFFECTS,
        if file == ANIMATION {
            "animation.x"
        } else {
            "controller.animation.x"
        },
    );
    let behavior = entity("minecraft:pig");
    let set = stacks(
        &[("entities/pig.json", &behavior)],
        &[(file, text), ("entity/one.entity.json", &pig)],
        LoadOptions::default(),
    );
    let linked = set.link();
    let is_reference = |finding: &bedrock_addon::diagnostic::Diagnostic| {
        matches!(
            finding.detail.as_deref(),
            Some(DiagnosticDetail::Reference { .. })
        )
    };
    load(&set).iter().any(is_reference) || in_file(&linked, file).into_iter().any(is_reference)
}

/// 20 R16: the bones an animation moves and the locator of an effect name parts of a
/// geometry the animation does not know.
#[test]
fn r16_the_bones_and_the_locators_of_an_animation_are_not_checked() {
    let text = animation(
        "animation.x",
        r#""bones": {"no_such_bone": {"scale": 1}}, "particle_effects": {"0.0": {"effect": "smoke", "locator": "no_such_locator"}}"#,
    );
    assert!(!any_reference(ANIMATION, &text, |set| {
        let animations = set.resource.animations();
        animations
            .iter()
            .flat_map(|(_, document)| document.diagnostics().iter().cloned())
            .chain(animations.diagnostics().cloned())
            .collect()
    }));
}

/// 20 R19: the animations a controller state plays are keys of the client entity that plays
/// the controller, and the locators of its effects parts of a geometry.
#[test]
fn r19_the_animations_and_the_locators_of_a_controller_state_are_not_checked() {
    let text = animation_controller(
        "controller.animation.x",
        r#""animations": ["no_such_animation"], "particle_effects": [{"effect": "smoke", "locator": "no_such_locator"}]"#,
    );
    assert!(!any_reference(CONTROLLER, &text, |set| {
        let controllers = set.resource.animation_controllers();
        controllers
            .iter()
            .flat_map(|(_, document)| document.diagnostics().iter().cloned())
            .chain(controllers.diagnostics().cloned())
            .collect()
    }));
}
