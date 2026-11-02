//! The references a client entity makes.

use bedrock_addon::diagnostic::{DiagnosticDetail, Severity};
use bedrock_addon::id::EntityId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

// The material and atlas files of these worlds are a pack the game ships: the rows
// are tested at their own severity. `tests/link/view.rs` has the other views.
use super::super::common::{game_link_with as link_with, hosted, stacks};
use super::{Told, client_entity, entity, says, told};

const GHOST: &str = "entity/ghost.entity.json";
const INFO: Severity = Severity::Info;
const WARNING: Severity = Severity::Warning;
const ERROR: Severity = Severity::Error;
const UNRESOLVED: &str = "UnresolvedReference";

fn at(member: &str) -> String {
    format!("minecraft:client_entity/description/{member}")
}

/// What linking says about the client entity `demo:ghost` with `members`, in a world whose
/// behaviour pack defines the entity and whose resource pack also has `more`.
fn ghost_with(
    members: &str,
    more: &[(&'static str, &str)],
    load: LoadOptions,
    lints: bool,
) -> Vec<Told> {
    let behavior = entity("demo:ghost");
    let text = client_entity("demo:ghost", members);
    let mut resource = vec![(GHOST, text.as_str())];
    resource.extend_from_slice(more);
    let found = link_with(
        &[("entities/ghost.json", &behavior)],
        &resource,
        load,
        &LinkOptions {
            lints,
            ..Default::default()
        },
    );
    told(&found, "rp 2", GHOST)
}

/// [`ghost_with`], no host, every rule.
fn ghost(members: &str, more: &[(&'static str, &str)]) -> Vec<Told> {
    ghost_with(members, more, LoadOptions::default(), true)
}

// ---- R1: materials ---------------------------------------------------------------------------

#[cfg(all(feature = "render_controller", feature = "material"))]
mod r1 {
    use super::super::{ENTITY_MATERIAL, FANCY, materials, text_of};
    use super::*;

    #[test]
    fn r1_a_material_a_pack_defines_resolves_and_the_value_is_read_in_lower_case() {
        let file = materials(ENTITY_MATERIAL);
        let found = ghost(
            r#""materials": {"default": "entity_alphatest", "upper": "Entity_AlphaTest"}"#,
            &[FANCY, ("materials/entity.material", &file)],
        );
        assert_eq!(found, []);
    }

    /// The game logs the miss at level 3, skips the entry and goes on with the
    /// file: a kept error.
    #[test]
    fn r1_a_material_nothing_defines_is_a_kept_error_with_the_games_text() {
        let file = materials(ENTITY_MATERIAL);
        let behavior = entity("demo:ghost");
        let text = client_entity(
            "demo:ghost",
            r#""materials": {"default": "entity_alphatest", "Other": "No_Such_Material"}"#,
        );
        let found = link_with(
            &[("entities/ghost.json", &behavior)],
            &[(GHOST, &text), FANCY, ("materials/entity.material", &file)],
            LoadOptions::default(),
            // An error is no lint.
            &LinkOptions {
                lints: false,
                ..Default::default()
            },
        );
        assert_eq!(
            told(&found, "rp 2", GHOST),
            [says(
                UNRESOLVED,
                ERROR,
                &at("materials/Other"),
                "material",
                "no_such_material"
            )]
        );
        assert!(
            found
                .iter()
                .filter(|finding| finding.code.name() == UNRESOLVED)
                .all(|finding| finding.kept),
            "{found:#?}"
        );
        assert_eq!(
            text_of(&found, GHOST, UNRESOLVED),
            "Material \"no_such_material\" is referenced in file \"entity/ghost.entity.json\" but not defined."
        );
    }

    /// A `.material` file creates materials only when a list file names it.
    #[test]
    fn r1_a_material_of_a_file_no_list_names_is_not_defined() {
        let listed = materials(ENTITY_MATERIAL);
        let unlisted = materials(r#""decoy": {"vertexShader": "shaders/entity.vertex"}"#);
        let found = ghost(
            r#""materials": {"default": "entity_alphatest", "other": "decoy"}"#,
            &[
                FANCY,
                ("materials/entity.material", &listed),
                ("materials/other.material", &unlisted),
            ],
        );
        assert_eq!(
            found,
            [says(
                UNRESOLVED,
                ERROR,
                &at("materials/other"),
                "material",
                "decoy"
            )]
        );
    }

    /// The rule runs against the stack's material index: a stack that creates no material
    /// (a pack loaded without the game's `materials/`) has none, and nothing is judged. A
    /// host that answers for the game's materials is an index.
    #[test]
    fn r1_a_stack_that_creates_no_material_has_no_index_and_no_material_is_judged() {
        let members = r#""materials": {"default": "entity_alphatest"}"#;
        assert_eq!(ghost(members, &[]), []);
        // A `.material` file no list names creates nothing.
        let unlisted = materials(ENTITY_MATERIAL);
        assert_eq!(
            ghost(
                r#""materials": {"default": "nope"}"#,
                &[("materials/entity.material", &unlisted)]
            ),
            []
        );
        let host = hosted(&[("material", "another")]);
        assert_eq!(ghost_with(members, &[], host, true).len(), 1);
    }

    #[test]
    fn r1_a_material_of_the_game_resolves_with_a_host_and_is_a_warning_without_one() {
        let members = r#""materials": {"default": "entity_emissive", "spider": "Spider"}"#;
        // The skinning variant alone answers for its name.
        let host = hosted(&[
            ("material", "entity_emissive"),
            ("material", "spider.skinning"),
        ]);
        assert_eq!(ghost_with(members, &[], host, true), []);
        let file = materials(ENTITY_MATERIAL);
        let found = ghost(members, &[FANCY, ("materials/entity.material", &file)]);
        let names: Vec<&str> = found.iter().map(|said| said.4.as_str()).collect();
        assert_eq!(names, ["entity_emissive", "spider"]);
    }

    #[test]
    fn r26_a_material_whose_shader_is_no_actor_shader_is_an_invalid_reference() {
        let file = materials(&format!(
            r#"{ENTITY_MATERIAL}, "banner_pole": {{"vertexShader": "shaders/banner.vertex"}}, "pattern_mat": {{"vertexShader": "shaders/pattern.vertex"}}, "my_mat": {{"vertexShader": "shaders/my.vertex"}}"#
        ));
        let behavior = entity("demo:ghost");
        let text = client_entity(
            "demo:ghost",
            r#""materials": {"a": "entity_alphatest", "b": "banner_pole", "c": "pattern_mat", "D": "My_Mat"}"#,
        );
        let found = link_with(
            &[("entities/ghost.json", &behavior)],
            &[(GHOST, &text), FANCY, ("materials/entity.material", &file)],
            LoadOptions::default(),
            &LinkOptions {
                lints: false,
                ..Default::default()
            },
        );
        assert_eq!(
            told(&found, "rp 2", GHOST),
            // The game logs the shader at level 3
            //  and falls back to the `Actor`
            // variant: a kept error.
            [says(
                "InvalidReference",
                ERROR,
                &at("materials/D"),
                "material",
                "my_mat"
            )]
        );
        assert_eq!(
            text_of(&found, GHOST, "InvalidReference"),
            "Material shader \"shaders/my.vertex\" cannot be assigned to an entity"
        );
    }

    /// A material whose chain names no vertex shader has no shader that could be the wrong
    /// one: nothing is said.
    #[test]
    fn r26_a_material_without_a_vertex_shader_is_not_judged() {
        // Not written at all, inherited unwritten, and written as the empty text.
        let file = materials(
            r#""bare": {"+defines": ["FANCY"]}, "child:bare": {}, "empty": {"vertexShader": ""}"#,
        );
        let found = ghost(
            r#""materials": {"a": "bare", "b": "child", "c": "empty"}"#,
            &[FANCY, ("materials/entity.material", &file)],
        );
        assert_eq!(found, []);
    }

    /// A material the stack does not define has no shader to judge: R1 alone speaks.
    #[test]
    fn r26_a_material_nothing_defines_is_not_judged_by_its_shader() {
        let file = materials(ENTITY_MATERIAL);
        let found = ghost(
            r#""materials": {"default": "my_mat"}"#,
            &[FANCY, ("materials/entity.material", &file)],
        );
        let codes: Vec<&str> = found.iter().map(|said| said.0).collect();
        assert_eq!(codes, [UNRESOLVED]);
    }
}

#[cfg(not(all(feature = "render_controller", feature = "material")))]
#[test]
fn r1_r5_without_the_render_controller_and_material_features_nothing_is_said() {
    let found = ghost(
        r#""materials": {"default": "nope"}, "render_controllers": ["controller.render.nope"]"#,
        &[],
    );
    assert_eq!(found, []);
}

// ---- R2: particles ---------------------------------------------------------------------------

#[cfg(feature = "particle")]
mod r2 {
    use super::super::{particle, text_of};
    use super::*;

    #[test]
    fn r2_a_particle_a_pack_defines_resolves_and_the_value_is_read_in_lower_case() {
        let smoke = particle("demo:smoke");
        let found = ghost(
            r#""particle_effects": {"smoke": "demo:smoke", "upper": "Demo:Smoke"}"#,
            &[("particles/smoke.particle.json", &smoke)],
        );
        assert_eq!(found, []);
    }

    #[test]
    fn r2_a_particle_nothing_defines_is_a_kept_error_with_the_games_text() {
        let smoke = particle("demo:smoke");
        let behavior = entity("demo:ghost");
        let text = client_entity(
            "demo:ghost",
            r#""particle_effects": {"smoke": "demo:smoke", "X": "Demo:Nope"}"#,
        );
        let found = link_with(
            &[("entities/ghost.json", &behavior)],
            &[(GHOST, &text), ("particles/smoke.particle.json", &smoke)],
            LoadOptions::default(),
            &LinkOptions {
                lints: false,
                ..Default::default()
            },
        );
        assert_eq!(
            told(&found, "rp 2", GHOST),
            // The game logs the miss at level 3 and goes on with the next entry: a kept error.
            [says(
                UNRESOLVED,
                ERROR,
                &at("particle_effects/X"),
                "particle",
                "demo:nope"
            )]
        );
        // The file, then the value as written.
        assert_eq!(
            text_of(&found, GHOST, UNRESOLVED),
            "Entity .json 'entity/ghost.entity.json' referenced particle 'Demo:Nope' that doesn't exist"
        );
    }

    #[test]
    fn r2_a_particle_of_the_game_resolves_with_a_host_and_is_a_kept_error_without_one() {
        let members = r#""particle_effects": {"smoke": "minecraft:basic_smoke_particle"}"#;
        let host = hosted(&[("particle", "minecraft:basic_smoke_particle")]);
        assert_eq!(ghost_with(members, &[], host, true), []);
        assert_eq!(
            ghost(members, &[]),
            [says(
                UNRESOLVED,
                ERROR,
                &at("particle_effects/smoke"),
                "particle",
                "minecraft:basic_smoke_particle"
            )]
        );
    }
}

#[cfg(not(feature = "particle"))]
#[test]
fn r2_without_the_particle_feature_nothing_is_said() {
    assert_eq!(ghost(r#""particle_effects": {"x": "demo:nope"}"#, &[]), []);
}

// ---- R3: geometry ----------------------------------------------------------------------------

#[cfg(feature = "geometry")]
mod r3 {
    use super::super::geometry;
    use super::*;

    #[test]
    fn r3_a_geometry_a_pack_defines_resolves_and_the_value_is_read_in_lower_case() {
        let model = geometry("geometry.ghost");
        let found = ghost(
            r#""geometry": {"default": "geometry.ghost", "upper": "Geometry.Ghost"}"#,
            &[("models/entity/ghost.geo.json", &model)],
        );
        assert_eq!(found, []);
    }

    #[test]
    fn r3_a_geometry_nothing_defines_is_a_note_and_a_lint() {
        let model = geometry("geometry.ghost");
        let members = r#""geometry": {"default": "geometry.ghost", "Other": "Geometry.Nope"}"#;
        let more = [("models/entity/ghost.geo.json", model.as_str())];
        assert_eq!(
            ghost(members, &more),
            [says(
                UNRESOLVED,
                INFO,
                &at("geometry/Other"),
                "geometry",
                "geometry.nope"
            )]
        );
        assert_eq!(
            ghost_with(members, &more, LoadOptions::default(), false),
            []
        );
    }

    #[test]
    fn r3_a_geometry_of_the_game_resolves_with_a_host_and_is_a_note_without_one() {
        let members = r#""geometry": {"default": "geometry.humanoid"}"#;
        let host = hosted(&[("geometry", "geometry.humanoid")]);
        assert_eq!(ghost_with(members, &[], host, true), []);
        assert_eq!(
            ghost(members, &[]),
            [says(
                UNRESOLVED,
                INFO,
                &at("geometry/default"),
                "geometry",
                "geometry.humanoid"
            )]
        );
    }
}

#[cfg(not(feature = "geometry"))]
#[test]
fn r3_without_the_geometry_feature_nothing_is_said() {
    assert_eq!(
        ghost(r#""geometry": {"default": "geometry.nope"}"#, &[]),
        []
    );
}

// ---- R4: animations --------------------------------------------------------------------------

#[cfg(all(feature = "animation", feature = "animation_controller"))]
mod r4 {
    use super::super::{animation, animation_controller};
    use super::*;

    const FILES: [&str; 2] = [
        "animations/ghost.animation.json",
        "animation_controllers/ghost.animation_controllers.json",
    ];

    #[test]
    fn r4_an_animation_and_a_controller_a_pack_defines_resolve_by_their_prefix_in_lower_case() {
        let float = animation("animation.ghost.float", "");
        let moves = animation_controller("controller.animation.ghost.move", "");
        let found = ghost(
            r#""animations": {"float": "Animation.Ghost.Float", "move": "Controller.Animation.Ghost.Move", "neither": "ghost.neither"}"#,
            &[(FILES[0], &float), (FILES[1], &moves)],
        );
        // A value with neither prefix is ignored by the game, and by us.
        assert_eq!(found, []);
    }

    #[test]
    fn r4_an_animation_nothing_defines_is_a_note_and_a_lint() {
        let float = animation("animation.ghost.float", "");
        let moves = animation_controller("controller.animation.ghost.move", "");
        let more = [(FILES[0], float.as_str()), (FILES[1], moves.as_str())];
        // An animation does not answer for a controller of the same name, nor a controller
        // for an animation.
        let members = r#""animations": {"A": "Animation.Ghost.Nope", "b": "controller.animation.ghost.float", "c": "animation.ghost.move"}"#;
        assert_eq!(
            ghost(members, &more),
            [
                says(
                    UNRESOLVED,
                    INFO,
                    &at("animations/A"),
                    "animation",
                    "animation.ghost.nope"
                ),
                says(
                    UNRESOLVED,
                    INFO,
                    &at("animations/b"),
                    "animation",
                    "controller.animation.ghost.float"
                ),
                says(
                    UNRESOLVED,
                    INFO,
                    &at("animations/c"),
                    "animation",
                    "animation.ghost.move"
                ),
            ]
        );
        assert_eq!(
            ghost_with(members, &more, LoadOptions::default(), false),
            []
        );
    }

    #[test]
    fn r4_an_animation_of_the_game_resolves_with_a_host_and_is_a_note_without_one() {
        let members = r#""animations": {"base": "animation.humanoid.base_pose"}"#;
        let host = hosted(&[("animation", "animation.humanoid.base_pose")]);
        assert_eq!(ghost_with(members, &[], host, true), []);
        assert_eq!(
            ghost(members, &[]),
            [says(
                UNRESOLVED,
                INFO,
                &at("animations/base"),
                "animation",
                "animation.humanoid.base_pose"
            )]
        );
    }

    /// A document older than 1.10.0 lists its controllers in `animation_controllers`: the
    /// finding is where the file has the name.
    #[test]
    fn r4_a_controller_of_a_1_8_document_is_told_where_the_file_has_it() {
        let behavior = entity("demo:ghost");
        let moves = animation_controller("controller.animation.ghost.move", "");
        let text = r#"{"format_version": "1.8.0", "minecraft:client_entity": {"description": {"identifier": "demo:ghost", "animations": {"float": "animation.ghost.nope"}, "animation_controllers": [{"move": "controller.animation.ghost.move"}, {"turn": "controller.animation.ghost.nope"}]}}}"#;
        let found = link_with(
            &[("entities/ghost.json", &behavior)],
            &[(GHOST, text), (FILES[1], &moves)],
            LoadOptions::default(),
            &LinkOptions::default(),
        );
        assert_eq!(
            told(&found, "rp 2", GHOST),
            [
                says(
                    UNRESOLVED,
                    INFO,
                    &at("animations/float"),
                    "animation",
                    "animation.ghost.nope"
                ),
                says(
                    UNRESOLVED,
                    INFO,
                    &at("animation_controllers/1/turn"),
                    "animation",
                    "controller.animation.ghost.nope"
                ),
            ]
        );
    }
}

#[cfg(not(all(feature = "animation", feature = "animation_controller")))]
#[test]
fn r4_without_the_animation_feature_nothing_is_said() {
    assert_eq!(
        ghost(r#""animations": {"a": "animation.ghost.nope"}"#, &[]),
        []
    );
}

// ---- R5: render controllers ------------------------------------------------------------------

#[cfg(all(feature = "render_controller", feature = "material"))]
mod r5 {
    use super::super::render_controller;
    use super::*;

    const FILE: &str = "render_controllers/ghost.render_controllers.json";

    #[test]
    fn r5_a_render_controller_a_pack_defines_resolves_in_both_forms_and_in_lower_case() {
        let controller = render_controller("controller.render.ghost", "");
        let found = ghost(
            r#""render_controllers": ["controller.render.ghost", "Controller.Render.Ghost", {"Controller.Render.Ghost": "query.is_baby"}]"#,
            &[(FILE, &controller)],
        );
        assert_eq!(found, []);
    }

    #[test]
    fn r5_a_render_controller_nothing_defines_is_a_note_at_its_index_and_a_lint() {
        let controller = render_controller("controller.render.ghost", "");
        let more = [(FILE, controller.as_str())];
        let members = r#""render_controllers": ["controller.render.ghost", "Controller.Render.Nope", {"Controller.Render.Nope_Too": "1.0"}]"#;
        assert_eq!(
            ghost(members, &more),
            [
                says(
                    UNRESOLVED,
                    INFO,
                    &at("render_controllers/1"),
                    "render controller",
                    "controller.render.nope"
                ),
                // The object form: the member that names the controller, its key as written.
                says(
                    UNRESOLVED,
                    INFO,
                    &at("render_controllers/2/Controller.Render.Nope_Too"),
                    "render controller",
                    "controller.render.nope_too"
                ),
            ]
        );
        assert_eq!(
            ghost_with(members, &more, LoadOptions::default(), false),
            []
        );
    }

    #[test]
    fn r5_a_render_controller_of_the_game_resolves_with_a_host_and_is_a_note_without_one() {
        let members = r#""render_controllers": ["controller.render.default"]"#;
        let host = hosted(&[("render controller", "controller.render.default")]);
        assert_eq!(ghost_with(members, &[], host, true), []);
        assert_eq!(
            ghost(members, &[]),
            [says(
                UNRESOLVED,
                INFO,
                &at("render_controllers/0"),
                "render controller",
                "controller.render.default"
            )]
        );
    }
}

// ---- R7: textures ----------------------------------------------------------------------------

#[cfg(all(
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "texture_set"
))]
mod r7 {
    use super::*;

    const IMAGE: (&str, &str) = ("textures/entity/ghost.png", "an image; no load reads it");

    #[test]
    fn r7_a_texture_a_pack_has_resolves_as_written() {
        let found = ghost(
            r#""textures": {"default": "textures/entity/ghost"}"#,
            &[IMAGE],
        );
        assert_eq!(found, []);
    }

    #[test]
    fn r7_a_texture_no_pack_has_is_the_game_s_warning() {
        let behavior = entity("demo:ghost");
        let text = client_entity(
            "demo:ghost",
            r#""textures": {"default": "textures/entity/ghost", "Other": "textures/entity/nope"}"#,
        );
        let found = link_with(
            &[("entities/ghost.json", &behavior)],
            &[(GHOST, &text), IMAGE],
            LoadOptions::default(),
            // the game's content-log warning, told without the lints too
            &LinkOptions {
                lints: false,
                ..Default::default()
            },
        );
        assert_eq!(
            told(&found, "rp 2", GHOST),
            [says(
                "TextureNotFound",
                WARNING,
                &at("textures/Other"),
                "texture",
                "textures/entity/nope"
            )]
        );
    }

    /// A dedicated server reads no client entity, so a server-side link says nothing of its
    /// textures; the same world linked as the client tells the miss.
    #[test]
    fn r7_a_server_side_link_tells_no_texture_miss() {
        let behavior = entity("demo:ghost");
        let text = client_entity(
            "demo:ghost",
            r#""textures": {"default": "textures/entity/nope"}"#,
        );
        let linked = |client: bool| {
            let found = link_with(
                &[("entities/ghost.json", &behavior)],
                &[(GHOST, &text)],
                LoadOptions::default(),
                &LinkOptions {
                    client,
                    ..Default::default()
                },
            );
            told(&found, "rp 2", GHOST)
        };
        assert_eq!(linked(false), [], "a server-side link tells R7");
        assert_eq!(linked(true).len(), 1);
    }

    /// The text is the texture-set loader's, which the client entity's load reaches
    /// (the game's texture-set loader, logging at level 2).
    #[test]
    fn r7_the_finding_carries_the_texture_set_loader_s_text() {
        let found = link_with(
            &[("entities/ghost.json", &entity("demo:ghost"))],
            &[
                (
                    GHOST,
                    &client_entity(
                        "demo:ghost",
                        r#""textures": {"default": "textures/entity/nope"}"#,
                    ),
                ),
                IMAGE,
            ],
            LoadOptions::default(),
            &LinkOptions {
                lints: false,
                ..Default::default()
            },
        );
        let texts: Vec<String> = found
            .iter()
            .filter(|f| f.code.name() == "TextureNotFound")
            .map(|f| f.code.to_string())
            .collect();
        assert_eq!(
            texts,
            ["Failed to find either texture set or image file of name (textures/entity/nope)"]
        );
    }

    /// The path is handed on as written: nothing lower-cases it.
    #[test]
    fn r7_a_texture_path_is_not_lower_cased() {
        let found = ghost(
            r#""textures": {"default": "Textures/Entity/Ghost"}"#,
            &[IMAGE],
        );
        assert_eq!(
            found,
            [says(
                "TextureNotFound",
                WARNING,
                &at("textures/default"),
                "texture",
                "Textures/Entity/Ghost"
            )]
        );
    }

    /// A texture set that did not load is a texture the stack has: the load reported it.
    #[test]
    fn r7_a_texture_set_that_did_not_load_is_not_reported_again() {
        let found = ghost(
            r#""textures": {"default": "textures/entity/ghost"}"#,
            &[("textures/entity/ghost.texture_set.json", "{")],
        );
        assert_eq!(found, []);
    }

    /// The name of an atlas (`atlas.shield`, as the game's own shield attachable writes it;
    /// `atlas.terrain`, `atlas.items`) is no file of a pack: it is not judged. Only the
    /// prefix makes a name one.
    #[test]
    fn r7_the_name_of_an_atlas_is_no_texture_path_and_is_not_judged() {
        let found = ghost(
            r#""textures": {"pattern": "atlas.shield", "terrain": "atlas.terrain", "a": "atlas", "b": "atlases/shield", "c": "Atlas.shield"}"#,
            &[],
        );
        let names: Vec<&str> = found.iter().map(|said| said.4.as_str()).collect();
        assert_eq!(names, ["atlas", "atlases/shield", "Atlas.shield"]);
    }

    #[test]
    fn r7_a_texture_of_the_game_resolves_with_a_host_and_is_a_warning_without_one() {
        let members = r#""textures": {"default": "textures/entity/pig/pig"}"#;
        let host = hosted(&[("texture", "textures/entity/pig/pig")]);
        assert_eq!(ghost_with(members, &[], host, true), []);
        assert_eq!(ghost(members, &[]).len(), 1);
    }
}

#[cfg(not(all(
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "texture_set"
)))]
#[test]
fn r7_without_the_texture_feature_nothing_is_said() {
    assert_eq!(
        ghost(r#""textures": {"default": "textures/entity/nope"}"#, &[]),
        []
    );
}

// ---- R8: sound effects -----------------------------------------------------------------------

#[cfg(feature = "sound_definitions")]
mod r8 {
    use super::super::sound_definitions;
    use super::*;

    const FILE: &str = "sounds/sound_definitions.json";

    #[test]
    fn r8_a_sound_event_a_pack_defines_resolves() {
        let sounds = sound_definitions("ghost.hum");
        let found = ghost(
            r#""sound_effects": {"hum": "ghost.hum"}"#,
            &[(FILE, &sounds), ("sounds/demo/sound.ogg", "")],
        );
        assert_eq!(found, []);
    }

    #[test]
    fn r8_a_sound_event_nothing_defines_is_a_note_as_written_and_a_lint() {
        let sounds = sound_definitions("ghost.hum");
        let more = [(FILE, sounds.as_str()), ("sounds/demo/sound.ogg", "")];
        // The value is used as written: another case is another name.
        let members =
            r#""sound_effects": {"hum": "ghost.hum", "Upper": "Ghost.Hum", "s": "ghost.nope"}"#;
        assert_eq!(
            ghost(members, &more),
            [
                says(
                    "UnknownSound",
                    INFO,
                    &at("sound_effects/Upper"),
                    "sound event",
                    "Ghost.Hum"
                ),
                says(
                    "UnknownSound",
                    INFO,
                    &at("sound_effects/s"),
                    "sound event",
                    "ghost.nope"
                ),
            ]
        );
        assert_eq!(
            ghost_with(members, &more, LoadOptions::default(), false),
            []
        );
    }

    #[test]
    fn r8_a_sound_event_of_the_game_resolves_with_a_host_and_is_a_note_without_one() {
        let members = r#""sound_effects": {"click": "random.click"}"#;
        let host = hosted(&[("sound event", "random.click")]);
        assert_eq!(ghost_with(members, &[], host, true), []);
        assert_eq!(
            ghost(members, &[]),
            [says(
                "UnknownSound",
                INFO,
                &at("sound_effects/click"),
                "sound event",
                "random.click"
            )]
        );
    }
}

#[cfg(not(feature = "sound_definitions"))]
#[test]
fn r8_without_the_sound_feature_nothing_is_said() {
    assert_eq!(ghost(r#""sound_effects": {"s": "ghost.nope"}"#, &[]), []);
}

// ---- R10: the entity a client entity draws ---------------------------------------------------

/// What linking says about a client entity `id` in a world whose behaviour pack defines
/// `demo:ghost`.
fn identified(id: &str, load: LoadOptions, lints: bool) -> Vec<Told> {
    let behavior = entity("demo:ghost");
    let text = client_entity(id, "");
    let found = link_with(
        &[("entities/ghost.json", &behavior)],
        &[(GHOST, &text)],
        load,
        &LinkOptions {
            lints,
            ..Default::default()
        },
    );
    told(&found, "rp 2", GHOST)
}

#[test]
fn r10_a_client_entity_of_an_entity_a_pack_defines_resolves_in_lower_case() {
    assert_eq!(identified("demo:ghost", LoadOptions::default(), true), []);
    assert_eq!(identified("Demo:Ghost", LoadOptions::default(), true), []);
}

#[test]
fn r10_a_client_entity_of_an_entity_of_the_game_resolves_with_a_host_and_without_one() {
    // Without a host the crate's own list of the game's entities answers.
    assert_eq!(
        identified("minecraft:pig", LoadOptions::default(), true),
        []
    );
    let host = hosted(&[("entity", "demo:hosted")]);
    assert_eq!(identified("demo:hosted", host, true), []);
}

#[test]
fn r10_a_client_entity_of_no_entity_is_a_note_and_a_lint() {
    assert_eq!(
        identified("Demo:Nobody", LoadOptions::default(), true),
        [says(
            UNRESOLVED,
            INFO,
            &at("identifier"),
            "entity",
            "demo:nobody"
        )]
    );
    assert_eq!(identified("Demo:Nobody", LoadOptions::default(), false), []);
    // A host is believed when it says no.
    let host = hosted(&[("entity", "demo:hosted")]);
    assert_eq!(identified("minecraft:pig", host, true).len(), 1);
}

// ---- R6 (load) and R9 (not checked) ----------------------------------------------------------

/// 20 R9: `queryable_geometry` and `spawn_egg.texture` name a geometry slot and a texture
/// that neither the load nor the link pass resolves.
#[test]
fn r9_queryable_geometry_and_the_spawn_egg_texture_are_not_checked() {
    let behavior = entity("demo:ghost");
    let text = client_entity(
        "demo:ghost",
        r#""queryable_geometry": "nope", "spawn_egg": {"texture": "nope", "texture_index": 0}"#,
    );
    let set = stacks(
        &[("entities/ghost.json", &behavior)],
        &[(GHOST, &text)],
        LoadOptions::default(),
    );
    let entities = set.resource.client_entities();
    let document = entities
        .get(&EntityId::parse("demo:ghost"))
        .expect("the client entity loads");
    let reference = |finding: &bedrock_addon::diagnostic::Diagnostic| {
        matches!(
            finding.detail.as_deref(),
            Some(DiagnosticDetail::Reference { .. })
        )
    };
    assert!(!document.diagnostics().iter().any(reference));
    assert!(!entities.diagnostics().any(reference));
    assert_eq!(told(&set.link(), "rp 2", GHOST), []);
}
