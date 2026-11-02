//! 20 R13 and R14: the references an atomic client entity makes, and the render-controller
//! features it cannot use.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::{hosted, link_with};
use super::{Told, atomic, says, told};

const WISP: &str = "atomic_client_entities/wisp.json";
const INFO: Severity = Severity::Info;
const WARNING: Severity = Severity::Warning;
const UNRESOLVED: &str = "UnresolvedReference";

fn at(member: &str) -> String {
    format!("minecraft:atomic_client_entity/description/{member}")
}

/// What linking says about the atomic client entity `demo:wisp` with `members`, in a world
/// whose resource pack also has `more`.
fn wisp_with(
    members: &str,
    more: &[(&'static str, &str)],
    load: LoadOptions,
    lints: bool,
) -> Vec<Told> {
    let text = atomic("demo:wisp", members);
    let mut resource = vec![(WISP, text.as_str())];
    resource.extend_from_slice(more);
    let found = link_with(
        &[],
        &resource,
        load,
        LinkOptions {
            lints,
            ..Default::default()
        },
    );
    told(&found, "rp 2", WISP)
}

/// [`wisp_with`], no host, every rule.
fn wisp(members: &str, more: &[(&'static str, &str)]) -> Vec<Told> {
    wisp_with(members, more, LoadOptions::default(), true)
}

// ---- R13: the names of the description -------------------------------------------------------

/// An atomic client entity has no behaviour-pack entity: its identifier is not looked up.
#[test]
fn r13_the_identifier_of_an_atomic_client_entity_names_no_entity() {
    assert_eq!(wisp("", &[]), []);
}

#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r13_a_material_is_looked_up_as_written_and_one_nothing_defines_is_a_note() {
    use super::{ENTITY_MATERIAL, FANCY, materials};
    let file = materials(ENTITY_MATERIAL);
    // Nothing is lower-cased for an atomic client entity: another case is another name.
    let members = r#""materials": {"default": "entity_alphatest", "upper": "Entity_AlphaTest", "other": "nope"}"#;
    let more = [FANCY, ("materials/entity.material", file.as_str())];
    assert_eq!(
        wisp(members, &more),
        [
            says(
                UNRESOLVED,
                INFO,
                &at("materials/upper"),
                "material",
                "Entity_AlphaTest"
            ),
            says(UNRESOLVED, INFO, &at("materials/other"), "material", "nope"),
        ]
    );
    assert_eq!(wisp_with(members, &more, LoadOptions::default(), false), []);
    let host = hosted(&[("material", "Entity_AlphaTest"), ("material", "nope")]);
    assert_eq!(wisp_with(members, &more, host, true), []);
}

#[cfg(feature = "particle")]
#[test]
fn r13_a_particle_is_looked_up_as_written_and_one_nothing_defines_is_a_note() {
    let smoke = super::particle("demo:smoke");
    let members = r#""particle_effects": {"smoke": "demo:smoke", "upper": "Demo:Smoke"}"#;
    let more = [("particles/smoke.particle.json", smoke.as_str())];
    assert_eq!(
        wisp(members, &more),
        [says(
            UNRESOLVED,
            INFO,
            &at("particle_effects/upper"),
            "particle",
            "Demo:Smoke"
        )]
    );
    assert_eq!(wisp_with(members, &more, LoadOptions::default(), false), []);
}

/// The geometry kind compares without case wherever it is looked up.
#[cfg(feature = "geometry")]
#[test]
fn r13_a_geometry_nothing_defines_is_a_note_and_another_case_resolves_open_o3() {
    let model = super::geometry("geometry.wisp");
    let members = r#""geometry": {"default": "geometry.wisp", "upper": "Geometry.Wisp", "other": "Geometry.Nope"}"#;
    let more = [("models/entity/wisp.geo.json", model.as_str())];
    assert_eq!(
        wisp(members, &more),
        [says(
            UNRESOLVED,
            INFO,
            &at("geometry/other"),
            "geometry",
            "Geometry.Nope"
        )]
    );
    assert_eq!(wisp_with(members, &more, LoadOptions::default(), false), []);
}

#[cfg(all(feature = "animation", feature = "animation_controller"))]
#[test]
fn r13_an_animation_is_looked_up_as_written_and_one_nothing_defines_is_a_note() {
    let float = super::animation("animation.wisp.float", "");
    let members = r#""animations": {"float": "animation.wisp.float", "upper": "Animation.Wisp.Float", "c": "controller.animation.wisp.nope"}"#;
    let more = [("animations/wisp.animation.json", float.as_str())];
    // A value whose prefix is in another case has neither prefix: ignored.
    assert_eq!(
        wisp(members, &more),
        [says(
            UNRESOLVED,
            INFO,
            &at("animations/c"),
            "animation",
            "controller.animation.wisp.nope"
        )]
    );
    assert_eq!(wisp_with(members, &more, LoadOptions::default(), false), []);
}

#[cfg(feature = "sound_definitions")]
#[test]
fn r13_a_sound_event_nothing_defines_is_an_unknown_sound_note() {
    let sounds = super::sound_definitions("wisp.hum");
    let members = r#""sound_effects": {"hum": "wisp.hum", "s": "wisp.nope"}"#;
    let more = [
        ("sounds/sound_definitions.json", sounds.as_str()),
        ("sounds/demo/sound.ogg", ""),
    ];
    assert_eq!(
        wisp(members, &more),
        [says(
            "UnknownSound",
            INFO,
            &at("sound_effects/s"),
            "sound event",
            "wisp.nope"
        )]
    );
    assert_eq!(wisp_with(members, &more, LoadOptions::default(), false), []);
}

#[cfg(all(
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "texture_set"
))]
#[test]
fn r7_a_texture_of_an_atomic_client_entity_no_pack_has_is_the_game_s_warning() {
    let found = wisp_with(
        r#""textures": {"default": "textures/entity/wisp", "other": "textures/entity/nope"}"#,
        &[("textures/entity/wisp.png", "an image")],
        LoadOptions::default(),
        // the game's content-log warning, told without the lints too
        false,
    );
    assert_eq!(
        found,
        [says(
            "TextureNotFound",
            WARNING,
            &at("textures/other"),
            "texture",
            "textures/entity/nope"
        )]
    );
}

// ---- R13 and R14: render controllers ---------------------------------------------------------

#[cfg(all(feature = "render_controller", feature = "material"))]
mod controllers {
    use super::super::{render_controller, render_controller_of, text_of};
    use super::*;

    const FILE: &str = "render_controllers/wisp.render_controllers.json";
    const UNSUPPORTED: &str = "AtomicUnsupportedRenderController";

    fn named(controller: &str) -> Vec<Told> {
        wisp(
            r#""render_controllers": ["controller.render.wisp"]"#,
            &[(FILE, controller)],
        )
    }

    #[test]
    fn r13_a_render_controller_nothing_defines_is_a_note_and_no_feature_is_judged() {
        let members = r#""render_controllers": ["controller.render.nope"]"#;
        assert_eq!(
            wisp(members, &[]),
            [says(
                UNRESOLVED,
                INFO,
                &at("render_controllers/0"),
                "render controller",
                "controller.render.nope"
            )]
        );
        assert_eq!(wisp_with(members, &[], LoadOptions::default(), false), []);
        let host = hosted(&[("render controller", "controller.render.nope")]);
        assert_eq!(wisp_with(members, &[], host, true), []);
    }

    #[test]
    fn r14_a_controller_of_plain_references_is_supported() {
        assert_eq!(named(&render_controller("controller.render.wisp", "")), []);
    }

    #[test]
    fn r14_a_geometry_a_texture_or_a_material_that_is_no_plain_reference_is_unsupported() {
        for members in [
            r#""geometry": "query.is_baby ? Geometry.baby : Geometry.default", "textures": ["Texture.default"]"#,
            r#""geometry": "Geometry.default", "textures": ["query.is_baby ? Texture.baby : Texture.default"]"#,
            r#""geometry": "Geometry.default", "textures": ["Texture.default"], "materials": [{"*": "query.is_baby ? Material.baby : Material.default"}]"#,
        ] {
            let found = wisp_with(
                r#""render_controllers": ["controller.render.wisp"]"#,
                &[(
                    FILE,
                    &render_controller_of("controller.render.wisp", members),
                )],
                LoadOptions::default(),
                false,
            );
            assert_eq!(
                found,
                [says(
                    UNSUPPORTED,
                    WARNING,
                    &at("render_controllers/0"),
                    "",
                    ""
                )],
                "{members}"
            );
        }
    }

    /// The game skips an unsupported controller for the entity: its lighting is not judged.
    #[test]
    fn r14_an_unsupported_controller_is_told_once() {
        let controller = render_controller(
            "controller.render.wisp",
            r#""part_visibility": [{"head": "query.is_baby"}], "filter_lighting": true, "light_color_multiplier": "query.is_baby ? 2.0 : 1.0""#,
        );
        let found = named(&controller);
        assert_eq!(
            found,
            [says(
                UNSUPPORTED,
                WARNING,
                &at("render_controllers/0"),
                "",
                ""
            )]
        );
    }

    #[test]
    fn r14_the_three_findings_carry_the_games_texts() {
        let text = atomic(
            "demo:wisp",
            r#""render_controllers": ["controller.render.a", "controller.render.b", "controller.render.c"]"#,
        );
        let controllers = r#"{"format_version": "1.8.0", "render_controllers": {
            "controller.render.a": {"geometry": "Geometry.default", "textures": ["Texture.default"], "part_visibility": [{"head": false}]},
            "controller.render.b": {"geometry": "Geometry.default", "textures": ["Texture.default"], "filter_lighting": true},
            "controller.render.c": {"geometry": "Geometry.default", "textures": ["Texture.default"], "light_color_multiplier": "query.is_baby ? 2.0 : 1.0"}}}"#;
        let found = link_with(
            &[],
            &[(WISP, &text), (FILE, controllers)],
            LoadOptions::default(),
            LinkOptions {
                lints: false,
                ..Default::default()
            },
        );
        assert_eq!(
            told(&found, "rp 2", WISP),
            [
                says(UNSUPPORTED, WARNING, &at("render_controllers/0"), "", ""),
                says(
                    "AtomicFilterLightingIgnored",
                    WARNING,
                    &at("render_controllers/1"),
                    "",
                    ""
                ),
                says("NoEffect", WARNING, &at("render_controllers/2"), "", ""),
            ]
        );
        assert_eq!(
            text_of(&found, WISP, UNSUPPORTED),
            "Skipping an Atomic Entity - Unsupported render controller features (conditional enable, dynamic resources, multiple materials, or part visibility)."
        );
        assert_eq!(
            text_of(&found, WISP, "AtomicFilterLightingIgnored"),
            "Atomic Entity - filter_lighting is not supported."
        );
        assert_eq!(
            text_of(&found, WISP, "NoEffect"),
            "Atomic Entity - light_color_multiplier expressions are not supported."
        );
    }
}
