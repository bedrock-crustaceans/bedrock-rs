//! 20 R21 and materials V16: the slots a render controller draws, against the client
//! entities and attachables that use it; and the controller a folded name reaches.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

use super::super::common::link_with;
use super::{Told, render_controller, render_controller_of, says, told};

const FILE: &str = "render_controllers/x.render_controllers.json";
const INFO: Severity = Severity::Info;
const ERROR: Severity = Severity::Error;
const UNRESOLVED: &str = "UnresolvedReference";
const KEY: &str = "controller.render.x";

fn at(tail: &str) -> String {
    format!("render_controllers/{KEY}/{tail}")
}

/// What linking says in the controller file `text` of a world with the files `users`.
fn linked(text: &str, users: &[(&'static str, &str)], lints: bool) -> Vec<Told> {
    let mut resource = vec![(FILE, text)];
    resource.extend_from_slice(users);
    let found = link_with(
        &[],
        &resource,
        LoadOptions::default(),
        &LinkOptions {
            lints,
            ..Default::default()
        },
    );
    told(&found, "rp 2", FILE)
}

/// The description members of a user of [`KEY`] with the slot `slot` in each of its maps.
fn slots(slot: &str) -> String {
    format!(
        r#""materials": {{"{slot}": "entity_alphatest"}}, "textures": {{"{slot}": "textures/entity/x"}}, "geometry": {{"{slot}": "geometry.x"}}, "render_controllers": ["Controller.Render.X"]"#
    )
}

#[cfg(all(feature = "client_entity", feature = "atomic_client_entity"))]
mod with_client_entities {
    use super::super::{atomic, client_entity};
    use super::*;

    const PIG: &str = "entity/pig.entity.json";
    const COW: &str = "entity/cow.entity.json";

    #[test]
    fn r21_the_slots_of_a_using_client_entity_resolve_in_lower_case() {
        // Both sides are lower-cased: the file's text is folded, the slot names are too.
        let pig = client_entity("minecraft:pig", &slots("Default"));
        assert_eq!(
            linked(&render_controller(KEY, ""), &[(PIG, &pig)], false),
            []
        );
    }

    #[test]
    fn r21_a_slot_the_using_client_entity_lacks_is_a_kept_error() {
        let controller = render_controller_of(
            KEY,
            r#""geometry": "Geometry.Body", "materials": [{"*": "Material.default"}, {"head": "Material.Skin"}], "textures": ["Texture.default", "Texture.Glow"]"#,
        );
        let pig = client_entity("minecraft:pig", &slots("default"));
        // The draw evaluates the slot to a value of another type, and
        // The game logs it at level 3: told
        // without the lints too.
        assert_eq!(
            linked(&controller, &[(PIG, &pig)], false),
            linked(&controller, &[(PIG, &pig)], true)
        );
        assert_eq!(
            linked(&controller, &[(PIG, &pig)], true),
            [
                says(UNRESOLVED, ERROR, &at("geometry"), "geometry slot", "body"),
                says(UNRESOLVED, ERROR, &at("textures/1"), "texture slot", "glow"),
                says(
                    UNRESOLVED,
                    ERROR,
                    // At the entry: the element of `materials` and its bone pattern.
                    &at("materials/1/head"),
                    "material slot",
                    "skin"
                ),
            ]
        );
    }

    /// The game folds a controller file before it reads it; a finding names the controller,
    /// the array and the bone pattern as the file spells them.
    #[test]
    fn r21_a_finding_names_the_members_as_the_file_spells_them() {
        let controller = r#"{"format_version": "1.8.0", "render_controllers": {"Controller.Render.X": {
            "arrays": {"textures": {"Array.Skins": ["Texture.default", "Texture.Other"]}},
            "geometry": "Geometry.Body",
            "materials": [{"*": "Material.default", "Head": "Material.Skin"}, {"Arm": "Material.Skin"}],
            "textures": ["Array.Skins[query.variant]", "Texture.Glow"]}}}"#;
        let pig = client_entity("minecraft:pig", &slots("default"));
        let at = |tail: &str| format!("render_controllers/Controller.Render.X/{tail}");
        assert_eq!(
            linked(controller, &[(PIG, &pig)], true),
            [
                says(UNRESOLVED, ERROR, &at("geometry"), "geometry slot", "body"),
                says(UNRESOLVED, ERROR, &at("textures/1"), "texture slot", "glow"),
                says(
                    UNRESOLVED,
                    ERROR,
                    &at("materials/0/Head"),
                    "material slot",
                    "skin"
                ),
                says(
                    UNRESOLVED,
                    ERROR,
                    &at("materials/1/Arm"),
                    "material slot",
                    "skin"
                ),
                says(
                    UNRESOLVED,
                    ERROR,
                    &at("arrays/textures/Array.Skins/1"),
                    "texture slot",
                    "other"
                ),
            ]
        );
    }

    #[test]
    fn r21_the_elements_of_an_array_name_slots_too() {
        let controller = render_controller_of(
            KEY,
            r#""arrays": {"geometries": {"Array.geos": ["Geometry.default", "Geometry.Baby"]}, "textures": {"Array.skins": ["Texture.default", "Texture.Other"]}, "materials": {"Array.mats": ["Material.Shiny", "Material.default"]}}, "geometry": "Array.geos[query.is_baby]", "materials": [{"*": "Array.mats[0]"}], "textures": ["Array.skins[query.variant]"]"#,
        );
        let pig = client_entity("minecraft:pig", &slots("default"));
        assert_eq!(
            linked(&controller, &[(PIG, &pig)], true),
            [
                says(
                    UNRESOLVED,
                    ERROR,
                    &at("arrays/geometries/Array.geos/1"),
                    "geometry slot",
                    "baby"
                ),
                says(
                    UNRESOLVED,
                    ERROR,
                    &at("arrays/textures/Array.skins/1"),
                    "texture slot",
                    "other"
                ),
                says(
                    UNRESOLVED,
                    ERROR,
                    &at("arrays/materials/Array.mats/0"),
                    "material slot",
                    "shiny"
                ),
            ]
        );
    }

    /// a slot named inside an expression is the game's to resolve when the
    /// expression runs.
    #[test]
    fn r21_a_slot_named_inside_an_expression_stays_run() {
        let controller = render_controller_of(
            KEY,
            r#""geometry": "query.is_baby ? Geometry.baby : Geometry.adult", "materials": [{"*": "query.is_baby ? Material.baby : Material.adult"}], "textures": ["query.is_baby ? Texture.baby : Texture.adult"]"#,
        );
        let pig = client_entity("minecraft:pig", &slots("default"));
        assert_eq!(linked(&controller, &[(PIG, &pig)], true), []);
    }

    #[test]
    fn r21_a_controller_nothing_uses_is_not_checked() {
        let controller = render_controller_of(
            KEY,
            r#""geometry": "Geometry.nope", "textures": ["Texture.nope"]"#,
        );
        let pig = client_entity("minecraft:pig", "");
        assert_eq!(linked(&controller, &[(PIG, &pig)], true), []);
        // An atomic client entity's use is judged by R14, never by its slots.
        let wisp = atomic(
            "demo:wisp",
            r#""render_controllers": ["controller.render.x"]"#,
        );
        assert_eq!(
            linked(
                &controller,
                &[("atomic_client_entities/wisp.json", &wisp)],
                true
            ),
            []
        );
    }

    /// One finding per member, however many users lack the slot.
    #[test]
    fn r21_a_slot_one_of_two_users_lacks_is_told_once() {
        let controller = render_controller(KEY, "");
        let pig = client_entity("minecraft:pig", &slots("default"));
        let cow = client_entity("minecraft:cow", &slots("other"));
        let none = client_entity("minecraft:sheep", &slots("another"));
        let found = linked(
            &controller,
            &[
                (PIG, &pig),
                (COW, &cow),
                ("entity/sheep.entity.json", &none),
            ],
            true,
        );
        assert_eq!(
            found,
            [
                says(
                    UNRESOLVED,
                    ERROR,
                    &at("geometry"),
                    "geometry slot",
                    "default"
                ),
                says(
                    UNRESOLVED,
                    ERROR,
                    &at("textures/0"),
                    "texture slot",
                    "default"
                ),
                says(
                    UNRESOLVED,
                    ERROR,
                    &at("materials/0/*"),
                    "material slot",
                    "default"
                ),
            ]
        );
    }

    // ---- materials V16 -----------------------------------------------------------------------

    /// A world whose `tinted` material has `defines`, drawn by [`KEY`] with `textures`.
    fn glinting(defines: &str, textures: &str) -> Vec<Told> {
        told(&glinting_findings(defines, textures), "rp 2", FILE)
    }

    /// The findings of [`glinting`]'s world, before they are told.
    fn glinting_findings(defines: &str, textures: &str) -> bedrock_addon::diagnostic::Diagnostics {
        let file = super::super::materials(&format!(
            r#""tinted": {{"vertexShader": "shaders/entity.vertex", "+defines": [{defines}]}}"#
        ));
        let controller = render_controller_of(
            KEY,
            &format!(
                r#""geometry": "Geometry.default", "materials": [{{"*": "Material.default"}}], "textures": [{textures}]"#
            ),
        );
        let pig = client_entity(
            "minecraft:pig",
            r#""materials": {"Default": "Tinted"}, "textures": {"default": "textures/entity/x"}, "geometry": {"default": "geometry.x"}, "render_controllers": ["controller.render.x"]"#,
        );
        let mut resource = vec![(FILE, controller.as_str())];
        resource.extend_from_slice(&[
            (PIG, &pig),
            super::super::FANCY,
            ("materials/entity.material", &file),
        ]);
        link_with(
            &[],
            &resource,
            LoadOptions::default(),
            &LinkOptions {
                lints: false,
                ..Default::default()
            },
        )
    }

    const TWO: &str = r#""Texture.default", "Texture.default""#;

    #[test]
    fn v16_two_textures_for_a_glint_and_multiplicative_tint_material_is_a_kept_error() {
        // The game logs the line at level 3 at draw time and drops nothing: a kept error.
        assert_eq!(
            glinting(r#""GLINT", "MULTIPLICATIVE_TINT""#, TWO),
            [says("InvalidValue", ERROR, &at("textures"), "", "")]
        );
        assert!(
            glinting_findings(r#""GLINT", "MULTIPLICATIVE_TINT""#, TWO)
                .iter()
                .filter(|finding| finding.code.name() == "InvalidValue")
                .all(|finding| finding.kept)
        );
    }

    #[test]
    fn v16_one_or_three_textures_are_accepted() {
        let defines = r#""GLINT", "MULTIPLICATIVE_TINT""#;
        assert_eq!(glinting(defines, r#""Texture.default""#), []);
        assert_eq!(
            glinting(
                defines,
                r#""Texture.default", "Texture.default", "Texture.default""#
            ),
            []
        );
    }

    #[test]
    fn v16_a_material_with_one_of_the_two_defines_takes_two_textures() {
        assert_eq!(glinting(r#""GLINT""#, TWO), []);
        assert_eq!(glinting(r#""MULTIPLICATIVE_TINT""#, TWO), []);
    }
}

#[cfg(feature = "attachable")]
#[test]
fn r21_a_slot_a_using_attachable_lacks_is_a_kept_error() {
    let controller = render_controller_of(
        KEY,
        r#""geometry": "Geometry.default", "textures": ["Texture.Enchanted"]"#,
    );
    let bow = super::attachable("minecraft:bow", &slots("default"));
    assert_eq!(
        linked(&controller, &[("attachables/bow.json", &bow)], true),
        [says(
            UNRESOLVED,
            ERROR,
            &at("textures/0"),
            "texture slot",
            "enchanted"
        )]
    );
}

/// A controller whose key kept its capitals (an apostrophe before it stops the game's
/// lower-casing) is registered under its folded name: a reference resolves, and nothing more
/// is said than the load's own warning on the controller.
#[cfg(feature = "client_entity")]
#[test]
fn r5_a_controller_registered_under_a_folded_name_whose_key_kept_capitals_resolves() {
    let text = "{\"format_version\": \"1.8.0\", \"render_controllers\": {\n// it's\n\"controller.render.Foo\": {\"geometry\": \"geometry.default\", \"textures\": [\"texture.default\"]}}}";
    let pig = super::client_entity(
        "minecraft:pig",
        r#""textures": {"default": "textures/entity/x"}, "geometry": {"default": "geometry.x"}, "render_controllers": ["controller.render.foo", "Controller.Render.Foo"]"#,
    );
    let found = link_with(
        &[],
        &[(FILE, text), ("entity/pig.entity.json", &pig)],
        LoadOptions::default(),
        &LinkOptions::default(),
    );
    let about_controllers: Vec<Told> = told(&found, "rp 2", "entity/pig.entity.json")
        .into_iter()
        .filter(|said| said.3 == "render controller")
        .collect();
    assert_eq!(about_controllers, []);
    assert_eq!(told(&found, "rp 2", FILE), []);
}
