//! a client entity's and a
//! render controller's use of a material, checked against the material index.
//! The rules need two collections of the stack, so they run in the link pass. Written with
//! their assertions, ignored until `StackSet::link` is implemented.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{
    OpenOptions, Pack, PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks,
};
use bedrock_addon::storage::PackPath;

use crate::common::{fixture_text, wrapped};

const ENTITY_FILE: &str = "entity/thing.entity.json";
const MATERIALS: &str = "materials/entity.material";

/// A client entity `demo:thing` whose `default` material is `material`.
fn client_entity(material: &str) -> String {
    format!(
        r#"{{ "format_version": "1.10.0", "minecraft:client_entity": {{ "description": {{
            "identifier": "demo:thing",
            "materials": {{ "default": "{material}" }},
            "textures": {{ "default": "textures/entity/thing" }},
            "geometry": {{ "default": "geometry.thing" }},
            "render_controllers": [ "controller.render.default" ] }} }} }}"#
    )
}

/// What the client entity of [`client_entity`] names beside its material, each defined, so
/// that the material is the one thing a test's finding can be about: its texture,
/// its geometry (R3) and its render controller (R5).
const NAMED: [(&str, &str); 3] = [
    ("textures/entity/thing.png", "an image; no load reads it"),
    (
        "models/entity/thing.geo.json",
        r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.thing", "texture_width": 16, "texture_height": 16}, "bones": [{"name": "root"}]}]}"#,
    ),
    (
        "render_controllers/default.render_controllers.json",
        r#"{"format_version": "1.8.0", "render_controllers": {"controller.render.default": {"geometry": "Geometry.default", "materials": [{"*": "Material.default"}], "textures": ["Texture.default"]}}}"#,
    ),
];

/// The behaviour-pack entity the client entity draws.
const ENTITY: &str = r#"{"format_version": "1.21.0", "minecraft:entity": {"description": {"identifier": "demo:thing", "is_spawnable": true, "is_summonable": true, "is_experimental": false}, "components": {}}}"#;

/// One pack of `module` (`data`, `resources`) with the header UUID ending in `n` and `files`.
fn pack_of(n: &str, module: &str, files: &[(&str, &str)]) -> Pack {
    let manifest = format!(
        r#"{{ "format_version": 2,
        "header": {{ "name": "pack", "description": "", "uuid": "00000000-0000-0000-0000-0000000000{n}1",
                    "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
        "modules": [{{ "type": "{module}", "uuid": "00000000-0000-0000-0000-0000000000{n}2", "version": [1, 0, 0] }}] }}"#
    );
    let files = std::iter::once(("manifest.json", manifest.as_str()))
        .chain(files.iter().copied())
        .map(|(path, text)| (PackPath::new(path).unwrap(), text.as_bytes().to_vec()))
        .collect::<Vec<_>>();
    let options = OpenOptions {
        origin: PackOrigin::World,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("a pack")
}

/// The stacks of one resource pack holding the vanilla fancy list, `materials` as its
/// `materials/entity.material`, what [`NAMED`] defines and the files `more`, and of one
/// behaviour pack with the entity `demo:thing`.
fn stacks(materials: &str, more: &[(&str, &str)]) -> StackSet {
    let fancy = fixture_text("vanilla/fancy.json");
    let mut files = vec![
        ("materials/fancy.json", fancy.as_str()),
        (MATERIALS, materials),
    ];
    files.extend_from_slice(&NAMED);
    files.extend_from_slice(more);
    let rp = pack_of("a", "resources", &files);
    let bp = pack_of("b", "data", &[("entities/thing.json", ENTITY)]);
    let select = |pack: &Pack| {
        vec![PackSelection {
            pack_id: pack.id().uuid,
            version: pack.id().version.clone(),
            subpack: None,
        }]
    };
    let world = WorldPacks {
        behavior: select(&bp),
        resource: select(&rp),
    };
    let mut repository = PackRepository::new();
    repository.add_pack(rp, PackOrigin::Package);
    repository.add_pack(bp, PackOrigin::World);
    repository.stacks(&world, LoadOptions::default())
}

/// The findings at `file` above debug, as code, severity and text.
fn found(d: &Diagnostics, file: &str) -> Vec<(&'static str, Severity, String)> {
    d.iter()
        .filter(|f| {
            f.severity > Severity::Debug
                && f.location.path.as_ref().is_some_and(|p| p.as_str() == file)
        })
        .map(|f| (f.code.id(), f.severity, f.code.to_string()))
        .collect()
}

// The rule is compiled with its target's feature.
#[cfg(feature = "entity")]
#[test]
fn x01_a_client_entity_naming_an_undefined_material_is_reported() {
    let entity = client_entity("no_such_material");
    let set = stacks(
        &fixture_text("vanilla/entity.material"),
        &[(ENTITY_FILE, &entity)],
    );
    assert!(
        !set.resource
            .materials()
            .contains(&bedrock_addon::id::MaterialId::parse("no_such_material"))
    );
    let d = set.link();
    assert_eq!(
        found(&d, ENTITY_FILE),
        [(
            "core/unresolved_reference_worded",
            Severity::Error,
            format!(
                "Material \"no_such_material\" is referenced in file \"{ENTITY_FILE}\" but not defined."
            )
        )]
    );
}

// The rule is compiled with its target's feature.
#[cfg(feature = "entity")]
#[test]
fn x01_a_material_of_either_switchable_group_or_a_variant_is_defined() {
    // MG.3: a name the common group, the fancy group or the sad group has is defined; the
    // skinning variants are names of the index too.
    for name in [
        "entity_alphatest",
        "entity.skinning",
        "entity_static.skinning_color",
    ] {
        let entity = client_entity(name);
        let set = stacks(
            &fixture_text("vanilla/entity.material"),
            &[(ENTITY_FILE, &entity)],
        );
        assert!(
            set.resource
                .materials()
                .contains(&bedrock_addon::id::MaterialId::parse(name)),
            "{name}"
        );
        let d = set.link();
        assert_eq!(found(&d, ENTITY_FILE), [], "{name}");
    }
}

// The rule is compiled with its target's feature.
#[cfg(feature = "entity")]
#[test]
fn x02_a_material_with_a_custom_shader_stem_cannot_be_drawn_for_an_entity_deviation() {
    let materials = wrapped(r#""my_mat": {"vertexShader": "shaders/my.vertex"}"#);
    let entity = client_entity("my_mat");
    let set = stacks(&materials, &[(ENTITY_FILE, &entity)]);
    // The index has the name: the reference resolves, to something an entity cannot use.
    let resolved = set
        .resource
        .materials()
        .resolve(
            bedrock_addon::families::material::MaterialGroupKind::Fancy,
            "my_mat",
        )
        .expect("my_mat");
    assert_eq!(resolved.vertex_shader_stem(), "my");
    let d = set.link();
    let found = found(&d, ENTITY_FILE);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        (found[0].0, found[0].1),
        ("core/invalid_reference_worded", Severity::Error)
    );
    // Level 3 at draw time, the `Actor` variant drawn: a kept error.
    assert!(d.iter().any(|f| f.kept), "{d:#?}");
    assert_eq!(
        found[0].2,
        "Material shader \"shaders/my.vertex\" cannot be assigned to an entity"
    );
}

// The rule is compiled with its target's feature.
#[cfg(feature = "entity")]
#[test]
fn x03_a_custom_material_that_inherits_a_vanilla_one_is_accepted() {
    // The supported way to make a custom material: a child of a vanilla material, in the
    // pack's copy of the vanilla file, so that it keeps the `entity` shader stem.
    let vanilla = fixture_text("vanilla/entity.material");
    let custom = r#""my_mat:entity_alphatest": {"+defines": ["USE_EMISSIVE"]}, "#;
    let materials = vanilla.replacen(
        "\"version\": \"1.0.0\",",
        &format!("\"version\": \"1.0.0\", {custom}"),
        1,
    );
    assert_ne!(materials, vanilla);
    let entity = client_entity("my_mat");
    let set = stacks(&materials, &[(ENTITY_FILE, &entity)]);
    let d = set.link();
    assert_eq!(found(&d, ENTITY_FILE), []);
    assert_eq!(found(&d, MATERIALS), []);
}

// The rule is compiled with its target's feature.
#[cfg(feature = "entity")]
#[test]
fn v16_a_glint_and_multiplicative_tint_material_needs_three_textures() {
    // `USE_MULTITEXTURE`, `MULTIPLICATIVE_TINT` and `MASKED_MULTITEXTURE` set the number of
    // textures a material requires; a controller that gives two to one that requires three
    // is reported.
    let materials = wrapped(
        r#""tinted_glint": {"vertexShader": "shaders/entity.vertex",
                            "+defines": ["GLINT", "MULTIPLICATIVE_TINT"]}"#,
    );
    let controller = r#"{ "format_version": "1.8.0", "render_controllers": { "controller.render.thing": {
        "geometry": "Geometry.default", "materials": [ { "*": "Material.default" } ],
        "textures": [ "Texture.default", "Texture.default" ] } } }"#;
    let entity = client_entity("tinted_glint")
        .replace("controller.render.default", "controller.render.thing");
    let controller_file = "render_controllers/thing.render_controllers.json";
    let set = stacks(
        &materials,
        &[(ENTITY_FILE, &entity), (controller_file, controller)],
    );
    let d = set.link();
    let found = found(&d, controller_file);
    assert_eq!(found.len(), 1, "{found:#?}");
    // The controller's `textures` has a length the material does not accept: the code of a
    // value outside what its member takes, with the game's text.
    assert_eq!(
        (found[0].0, found[0].1),
        ("core/invalid_value_worded", Severity::Error)
    );
    // The material's texture validation, level 3 at draw time: a kept error.
    assert!(d.iter().any(|f| f.kept), "{d:#?}");
    assert_eq!(
        found[0].2,
        "Render Controller: controller.render.thing provides 2 textures, but using a Glint and Multiplicative-Tint material which requires 3 textures."
    );
}
