//! The references
//! an attachable makes, and the identifier it may share with a client entity.

use bedrock_addon::diagnostic::{DiagnosticDetail, ReferenceHint, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

// The material and atlas files of these worlds are a pack the game ships: the rows
// are tested at their own severity. `tests/link/view.rs` has the other views.
use super::super::common::{
    behavior_pack, game_link_with as link_with, hosted, in_file, resource_pack, stacks_of,
};
use super::{Told, attachable, says, told};

const WAND: &str = "attachables/wand.json";
const INFO: Severity = Severity::Info;
const WARNING: Severity = Severity::Warning;
const ERROR: Severity = Severity::Error;
const UNRESOLVED: &str = "UnresolvedReference";

fn at(member: &str) -> String {
    format!("minecraft:attachable/description/{member}")
}

/// A behaviour-pack item `id`.
fn item(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.0", "minecraft:item": {{"description": {{"identifier": "{id}"}}, "components": {{}}}}}}"#
    )
}

/// What linking says about the attachable `example:wand` with `members`, in a world whose
/// behaviour pack defines the item `example:wand` and whose resource pack also has `more`.
fn wand_with(
    members: &str,
    more: &[(&'static str, &str)],
    load: LoadOptions,
    lints: bool,
) -> Vec<Told> {
    let known = item("example:wand");
    let text = attachable("example:wand", members);
    let mut resource = vec![(WAND, text.as_str())];
    resource.extend_from_slice(more);
    let found = link_with(
        &[("items/wand.json", &known)],
        &resource,
        load,
        LinkOptions {
            lints,
            ..Default::default()
        },
    );
    told(&found, "rp 2", WAND)
}

/// [`wand_with`], no host, every rule.
fn wand(members: &str, more: &[(&'static str, &str)]) -> Vec<Told> {
    wand_with(members, more, LoadOptions::default(), true)
}

// ---- R11: the item ---------------------------------------------------------------------------

/// The hints of the `UnresolvedReference` findings about an item in the attachable's file.
fn item_hints(members: &str, load: LoadOptions) -> Vec<(String, String, Option<ReferenceHint>)> {
    let known = item("Example:Wand");
    let text = attachable("example:wand", members);
    // Ours: the game's attachable never matches an item at render time without a line
    // (attachable rules 21/22): an information, a lint.
    let found = link_with(
        &[("items/wand.json", &known)],
        &[(WAND, &text)],
        load,
        LinkOptions::default(),
    );
    in_file(&found, WAND)
        .into_iter()
        .filter_map(|finding| match finding.detail.as_deref() {
            Some(DiagnosticDetail::Reference {
                kind: "item",
                id,
                hint,
            }) => {
                assert_eq!(finding.code.name(), UNRESOLVED);
                assert_eq!(finding.severity, INFO);
                Some((
                    finding.location.json_path.to_string(),
                    id.clone(),
                    hint.clone(),
                ))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn r11_an_item_a_pack_defines_or_the_game_has_resolves_byte_for_byte() {
    // The pack's item as it is written there, and one of the game's.
    assert_eq!(
        item_hints(r#""item": "Example:Wand""#, LoadOptions::default()),
        []
    );
    assert_eq!(
        item_hints(
            r#""item": {"minecraft:bow": "1.0"}"#,
            LoadOptions::default()
        ),
        []
    );
    // No member, no reference: the attachable is drawn for the item of its own name.
    assert_eq!(item_hints("", LoadOptions::default()), []);
}

#[test]
fn r11_an_item_that_differs_in_case_is_a_warning_with_the_item_meant() {
    assert_eq!(
        item_hints(r#""item": "example:wand""#, LoadOptions::default()),
        [(
            at("item"),
            "example:wand".to_owned(),
            Some(ReferenceHint::CaseMismatch("Example:Wand".to_owned()))
        )]
    );
    assert_eq!(
        item_hints(
            r#""item": {"Minecraft:Bow": "1.0"}"#,
            LoadOptions::default()
        ),
        [(
            at("item/Minecraft:Bow"),
            "Minecraft:Bow".to_owned(),
            Some(ReferenceHint::CaseMismatch("minecraft:bow".to_owned()))
        )]
    );
}

#[test]
fn r11_an_item_without_its_namespace_is_a_warning_no_minecraft_is_assumed() {
    assert_eq!(
        item_hints(r#""item": "bow""#, LoadOptions::default()),
        [(
            at("item"),
            "bow".to_owned(),
            Some(ReferenceHint::MissingNamespace)
        )]
    );
}

#[test]
fn r11_an_item_nothing_has_is_a_warning_without_a_hint() {
    assert_eq!(
        item_hints(
            r#""item": {"example:nothing": "1.0"}"#,
            LoadOptions::default()
        ),
        [(
            at("item/example:nothing"),
            "example:nothing".to_owned(),
            None
        )]
    );
}

#[test]
fn r11_a_host_answers_for_the_items_of_the_game_yes_or_no() {
    let host = || hosted(&[("item", "demo:hosted")]);
    assert_eq!(item_hints(r#""item": "demo:hosted""#, host()), []);
    // The host is believed when it says no: the crate's own list is not asked.
    assert_eq!(item_hints(r#""item": "minecraft:bow""#, host()).len(), 1);
    assert_eq!(
        item_hints(r#""item": "Demo:Hosted""#, host()),
        [(
            at("item"),
            "Demo:Hosted".to_owned(),
            Some(ReferenceHint::CaseMismatch("demo:hosted".to_owned()))
        )]
    );
}

/// Every block has an item of its name.
#[cfg(feature = "block")]
#[test]
fn r11_the_item_of_a_block_of_the_stack_resolves() {
    let lamp = r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp"}, "components": {}}}"#;
    let text = attachable("example:wand", r#""item": "demo:lamp""#);
    let found = link_with(
        &[("blocks/lamp.json", lamp)],
        &[(WAND, &text)],
        LoadOptions::default(),
        LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert_eq!(told(&found, "rp 2", WAND), []);
}

/// A binding outlives the document it came from: the definition a higher pack replaces
/// leaves its `item` behind, and the finding is at the file that wrote it.
#[test]
fn r11_a_binding_of_a_replaced_document_is_told_at_its_own_file() {
    let lower = attachable("example:wand", r#""item": "example:lost""#);
    let higher = attachable(
        "example:wand",
        r#""min_engine_version": "1.20.0", "item": "minecraft:bow""#,
    );
    let set = stacks_of(
        vec![behavior_pack(1, &[])],
        vec![
            resource_pack(2, &[("attachables/new.json", &higher)]),
            resource_pack(3, &[(WAND, &lower)]),
        ],
        LoadOptions::default(),
    );
    let found = set.link();
    assert_eq!(
        told(&found, "rp 3", WAND),
        [says(UNRESOLVED, INFO, &at("item"), "item", "example:lost")]
    );
    assert_eq!(told(&found, "rp 2", "attachables/new.json"), []);
}

// ---- R12: as R1 to R5, and R7 ----------------------------------------------------------------

#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r12_a_material_resolves_in_lower_case_and_one_nothing_defines_is_a_kept_error() {
    use super::{ENTITY_MATERIAL, FANCY, materials, text_of};
    let file = materials(ENTITY_MATERIAL);
    let text = attachable(
        "example:wand",
        r#""materials": {"default": "Entity_AlphaTest", "Other": "No_Such_Material"}"#,
    );
    let known = item("example:wand");
    let found = link_with(
        &[("items/wand.json", &known)],
        &[(WAND, &text), FANCY, ("materials/entity.material", &file)],
        LoadOptions::default(),
        LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert_eq!(
        told(&found, "rp 2", WAND),
        [says(
            UNRESOLVED,
            ERROR,
            &at("materials/Other"),
            "material",
            "no_such_material"
        )]
    );
    // The game's level-3 line, the file kept.
    assert!(
        found
            .iter()
            .filter(|finding| finding.code.name() == UNRESOLVED)
            .all(|finding| finding.kept),
        "{found:#?}"
    );
    assert_eq!(
        text_of(&found, WAND, UNRESOLVED),
        "Material \"no_such_material\" is referenced in file \"attachables/wand.json\" but not defined."
    );
}

#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r12_a_material_whose_shader_is_no_actor_shader_is_an_invalid_reference() {
    use super::{FANCY, materials, text_of};
    let file = materials(r#""my_mat": {"vertexShader": "shaders/my.vertex"}"#);
    let text = attachable("example:wand", r#""materials": {"default": "my_mat"}"#);
    let known = item("example:wand");
    let found = link_with(
        &[("items/wand.json", &known)],
        &[(WAND, &text), FANCY, ("materials/entity.material", &file)],
        LoadOptions::default(),
        LinkOptions::default(),
    );
    assert_eq!(
        told(&found, "rp 2", WAND),
        // Level 3 at draw time, the `Actor` variant drawn: a
        // kept error.
        [says(
            "InvalidReference",
            ERROR,
            &at("materials/default"),
            "material",
            "my_mat"
        )]
    );
    assert!(
        found
            .iter()
            .filter(|finding| finding.code.name() == "InvalidReference")
            .all(|finding| finding.kept),
        "{found:#?}"
    );
    assert_eq!(
        text_of(&found, WAND, "InvalidReference"),
        "Material shader \"shaders/my.vertex\" cannot be assigned to an entity"
    );
}

#[cfg(feature = "particle")]
#[test]
fn r12_a_particle_resolves_in_lower_case_and_one_nothing_defines_is_a_kept_error() {
    let smoke = super::particle("demo:smoke");
    let found = wand_with(
        r#""particle_effects": {"smoke": "Demo:Smoke", "X": "demo:nope"}"#,
        &[("particles/smoke.particle.json", &smoke)],
        LoadOptions::default(),
        false,
    );
    assert_eq!(
        found,
        // Level 3, the entry skipped, the file kept.
        [says(
            UNRESOLVED,
            ERROR,
            &at("particle_effects/X"),
            "particle",
            "demo:nope"
        )]
    );
}

#[cfg(feature = "geometry")]
#[test]
fn r12_a_geometry_resolves_in_lower_case_and_one_nothing_defines_is_a_note() {
    let model = super::geometry("geometry.wand");
    let members = r#""geometry": {"default": "Geometry.Wand", "Other": "Geometry.Nope"}"#;
    let more = [("models/entity/wand.geo.json", model.as_str())];
    assert_eq!(
        wand(members, &more),
        [says(
            UNRESOLVED,
            INFO,
            &at("geometry/Other"),
            "geometry",
            "geometry.nope"
        )]
    );
    assert_eq!(wand_with(members, &more, LoadOptions::default(), false), []);
    let host = hosted(&[("geometry", "geometry.nope"), ("item", "example:wand")]);
    assert_eq!(wand_with(members, &more, host, true), []);
}

#[cfg(all(feature = "animation", feature = "animation_controller"))]
#[test]
fn r12_an_animation_resolves_by_its_prefix_and_one_nothing_defines_is_a_note() {
    let hold = super::animation("animation.wand.hold", "");
    let moves = super::animation_controller("controller.animation.wand.move", "");
    let members = r#""animations": {"hold": "Animation.Wand.Hold", "move": "Controller.Animation.Wand.Move", "neither": "wand.neither", "N": "animation.wand.nope", "c": "controller.animation.wand.hold"}"#;
    let more = [
        ("animations/wand.animation.json", hold.as_str()),
        (
            "animation_controllers/wand.animation_controllers.json",
            moves.as_str(),
        ),
    ];
    assert_eq!(
        wand(members, &more),
        [
            says(
                UNRESOLVED,
                INFO,
                &at("animations/N"),
                "animation",
                "animation.wand.nope"
            ),
            says(
                UNRESOLVED,
                INFO,
                &at("animations/c"),
                "animation",
                "controller.animation.wand.hold"
            ),
        ]
    );
    assert_eq!(wand_with(members, &more, LoadOptions::default(), false), []);
}

#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r12_a_render_controller_resolves_in_lower_case_and_one_nothing_defines_is_a_note() {
    let controller = super::render_controller("controller.render.wand", "");
    let members = r#""render_controllers": ["Controller.Render.Wand", {"Controller.Render.Nope": "1.0"}, "controller.render.nope_too"]"#;
    let more = [(
        "render_controllers/wand.render_controllers.json",
        controller.as_str(),
    )];
    assert_eq!(
        wand(members, &more),
        [
            says(
                UNRESOLVED,
                INFO,
                &at("render_controllers/1/Controller.Render.Nope"),
                "render controller",
                "controller.render.nope"
            ),
            says(
                UNRESOLVED,
                INFO,
                &at("render_controllers/2"),
                "render controller",
                "controller.render.nope_too"
            ),
        ]
    );
    assert_eq!(wand_with(members, &more, LoadOptions::default(), false), []);
}

#[cfg(all(
    feature = "texture_atlas",
    feature = "texture_file",
    feature = "texture_set"
))]
#[test]
fn r7_a_texture_of_an_attachable_resolves_as_written_and_one_no_pack_has_is_the_game_s_warning() {
    let found = wand_with(
        r#""textures": {"default": "textures/entity/wand", "Other": "textures/entity/nope"}"#,
        &[("textures/entity/wand.png", "an image")],
        LoadOptions::default(),
        // the game's content-log warning, told without the lints too
        false,
    );
    assert_eq!(
        found,
        [says(
            "TextureNotFound",
            WARNING,
            &at("textures/Other"),
            "texture",
            "textures/entity/nope"
        )]
    );
}

/// 20 R12 lists no sound: an attachable's `sound_effects` are not resolved.
#[test]
fn r12_the_sound_effects_of_an_attachable_are_not_checked() {
    assert_eq!(wand(r#""sound_effects": {"s": "wand.nope"}"#, &[]), []);
}

// ---- rule 28: one identifier, a client entity and an attachable -----------------------------

#[cfg(feature = "client_entity")]
mod rule_28 {
    use super::super::client_entity;
    use super::*;

    fn collisions(entity_id: &str, attachable_id: &str) -> Vec<Told> {
        let entity = client_entity(entity_id, "");
        let text = attachable(attachable_id, "");
        let found = link_with(
            &[],
            &[("entity/thing.entity.json", &entity), (WAND, &text)],
            LoadOptions::default(),
            // Ours: the shared map merges in silence: a lint.
            LinkOptions::default(),
        );
        told(&found, "rp 2", WAND)
            .into_iter()
            .filter(|said| said.0 == "IdentifierCollision")
            .collect()
    }

    #[test]
    fn rule_28_an_attachable_and_a_client_entity_of_one_identifier_collide_in_any_case_deviation() {
        let expected = |id: &str| {
            [says(
                "IdentifierCollision",
                INFO,
                &at("identifier"),
                "entity",
                id,
            )]
        };
        assert_eq!(
            collisions("minecraft:pig", "minecraft:pig"),
            expected("minecraft:pig")
        );
        assert_eq!(
            collisions("Minecraft:Pig", "minecraft:PIG"),
            expected("minecraft:PIG")
        );
    }

    #[test]
    fn rule_28_identifiers_that_differ_do_not_collide() {
        assert_eq!(collisions("minecraft:pig", "minecraft:pig_saddle"), []);
    }
}
