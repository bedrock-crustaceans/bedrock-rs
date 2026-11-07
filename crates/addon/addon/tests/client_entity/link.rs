//! `l06`, and "Reference checks we
//! add": the references a client entity makes into other families (particles, materials,
//! geometry, animations and animation controllers, render controllers). They are the link
//! pass's, so the rows are written against `StackSet::link` and
//! ignored until the link pass exists. What loading alone says is pinned below, not
//! ignored.

use bedrock_addon::diagnostic::{DiagnosticDetail, Diagnostics, Kind, Severity};
use bedrock_addon::document::Document;
use bedrock_addon::families::client_entity::{AnimationKind, ClientEntity};
use bedrock_addon::id::EntityId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{
    OpenOptions, Pack, PackOrigin, PackRepository, PackSelection, StackSet, WorldPacks,
};
use bedrock_addon::storage::PackPath;

use super::common::*;

const GHOST: &str = "entity/ghost.entity.json";

/// One pack of `module` (`data`, `resources`) with the header UUID `n` and `files`.
fn pack_of(n: u8, module: &str, files: &[(&str, &str)]) -> Pack {
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "pack {n}", "description": "", "uuid": "00000000-0000-4000-8000-0000000000{n:02}", "version": [1, 0, 0], "min_engine_version": [1, 21, 0]}}, "modules": [{{"type": "{module}", "uuid": "00000000-0000-4000-9000-0000000000{n:02}", "version": [1, 0, 0]}}]}}"#
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

/// A world whose resource pack holds `files` and whose behaviour pack defines the entity
/// `demo:ghost`: a client entity of no entity is a finding of its own, which is not
/// what these tests are about.
fn world(files: &[(&str, &str)]) -> StackSet {
    const ENTITY: &str = r#"{"format_version": "1.21.0", "minecraft:entity": {"description": {"identifier": "demo:ghost", "is_spawnable": true, "is_summonable": true, "is_experimental": false}, "components": {}}}"#;
    let packs = [
        pack_of(1, "data", &[("entities/ghost.json", ENTITY)]),
        pack_of(2, "resources", files),
    ];
    let select = |pack: &Pack| {
        vec![PackSelection {
            pack_id: pack.id().uuid,
            version: pack.id().version.clone(),
            subpack: None,
        }]
    };
    let selected = WorldPacks {
        behavior: select(&packs[0]),
        resource: select(&packs[1]),
    };
    let mut repository = PackRepository::new();
    // these tests, is one the game ships)
    let [behavior, resource] = packs;
    repository.add_pack(behavior, PackOrigin::World);
    repository.add_pack(resource, PackOrigin::Package);
    repository.stacks(&selected, LoadOptions::default())
}

/// The link findings about `entity/ghost.entity.json` of a pack that holds it and `others`.
fn linked(ghost: &str, others: &[(&str, &str)]) -> Diagnostics {
    let mut files = vec![(GHOST, ghost)];
    files.extend_from_slice(others);
    linked_in(&world(&files))
}

/// The link findings about `entity/ghost.entity.json` of `set`.
fn linked_in(set: &StackSet) -> Diagnostics {
    set.link()
        .into_iter()
        .filter(|finding| {
            finding
                .location
                .path
                .as_ref()
                .is_some_and(|path| path.as_str() == "entity/ghost.entity.json")
        })
        .collect()
}

const PARTICLE: &str = r#"{"format_version": "1.10.0", "particle_effect": {"description": {"identifier": "demo:smoke", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}}, "components": {"minecraft:emitter_rate_instant": {"num_particles": 1}, "minecraft:emitter_lifetime_once": {"active_time": 1}}}}"#;

// The rule is compiled with its target's feature.
#[cfg(feature = "particle")]
#[test]
fn l06_unknown_particle() {
    let ghost = ghost(
        "1.10.0",
        r#""particle_effects": {"x": "demo:nope", "smoke": "demo:smoke"}"#,
    );
    let d = linked(&ghost, &[("particles/smoke.particle.json", PARTICLE)]);
    // vanilla logs at level 3, skips the entry and loads the file: a
    // kept error.
    expect(
        &d,
        "core/unresolved_reference_worded",
        Severity::Error,
        "Entity .json 'entity/ghost.entity.json' referenced particle 'demo:nope' that doesn't exist",
    );
    expect_at(
        &d,
        "core/unresolved_reference_worded",
        Severity::Error,
        "minecraft:client_entity/description/particle_effects/x",
    );
    assert!(
        d.iter()
            .filter(|f| f.code.id() == "core/unresolved_reference_worded")
            .all(|f| f.kept),
        "{d:#?}"
    );
    assert_eq!(
        d.iter()
            .filter(|f| f.code.id() == "core/unresolved_reference_worded")
            .count(),
        1
    );
}

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn rule_18_undefined_material() {
    let ghost = ghost("1.10.0", r#""materials": {"default": "No_Such_Material"}"#);
    let materials = r#"{"materials": {"version": "1.0.0", "entity_alphatest": {}}}"#;
    // A `.material` file creates materials only when a list names it.
    let list = (
        "materials/fancy.json",
        r#"[{"path": "materials/entity.material"}]"#,
    );
    let d = linked(&ghost, &[list, ("materials/entity.material", materials)]);
    // The lower-cased value, then the file; the game's level-3 line, the entry skipped and the
    // file kept: a kept error.
    expect(
        &d,
        "core/unresolved_reference_worded",
        Severity::Error,
        "Material \"no_such_material\" is referenced in file \"entity/ghost.entity.json\" but not defined.",
    );
}

/// Until the link pass runs, a document with references nothing defines loads, and loading
/// says nothing of them: vanilla resolves geometry, animations and render controllers lazily
/// and checks particles and materials against other groups.
#[test]
fn l06_the_document_loads_and_loading_does_not_resolve_references() {
    let ghost = ghost(
        "1.10.0",
        r#""materials": {"default": "no_such_material"}, "geometry": {"default": "geometry.nope"}, "particle_effects": {"x": "demo:nope"}, "animations": {"a": "animation.nope"}, "render_controllers": ["controller.render.nope"], "sound_effects": {"s": "nope"}"#,
    );
    let files = [("entity/ghost.entity.json", ghost.as_str())];
    let set = stack(&[pack(&files)]);
    let entities = set.resource.client_entities();
    let entity = entities.get(&EntityId::parse("demo:ghost")).unwrap();
    silent(entity.diagnostics());
    assert!(entities.diagnostics().next().is_none());
    // The entry vanilla would skip is kept: nothing here knows the particle group.
    assert!(
        entity
            .description
            .slots
            .particle_effects
            .as_ref()
            .unwrap()
            .contains_key("x")
    );
}

// ---- "Reference checks we add" ----

/// The names the `UnresolvedReference` findings of `d` are about, in order.
fn unresolved(d: &Diagnostics) -> Vec<String> {
    d.iter()
        .filter(|f| f.code.id() == "core/unresolved_reference")
        .filter_map(|f| match f.detail.as_deref() {
            Some(DiagnosticDetail::Reference { id, .. }) => Some(id.clone()),
            _ => None,
        })
        .collect()
}

/// Requires the one `UnresolvedReference` of `d` to be a note at `path` about `id`: what
/// vanilla resolves lazily and never logs is kept and reported at info.
#[track_caller]
fn one_lazy_reference(d: &Diagnostics, path: &str, id: &str) {
    expect_at(d, "core/unresolved_reference", Severity::Info, path);
    assert_eq!(unresolved(d), [id.to_owned()], "{}", lines(d).join("\n"));
    assert!(
        d.iter()
            .filter(|f| f.code.id() == "core/unresolved_reference")
            .all(|f| f.severity == Severity::Info),
        "{}",
        lines(d).join("\n")
    );
}

/// The loaded `demo:ghost` of `set`, for what the link pass leaves as written.
#[track_caller]
fn ghost_of(set: &StackSet) -> Document<ClientEntity> {
    set.resource
        .client_entities()
        .get(&EntityId::parse("demo:ghost"))
        .unwrap()
        .clone()
}

const GEOMETRY: &str = r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.ghost", "texture_width": 16, "texture_height": 16}, "bones": [{"name": "root"}]}]}"#;

// The rule is compiled with its target's feature.
#[cfg(feature = "geometry")]
#[test]
fn reference_check_unknown_geometry() {
    // R3: the value is looked up in lower case; a geometry nothing defines is kept, and
    // vanilla logs nothing (it resolves lazily): a note.
    let text = ghost(
        "1.10.0",
        r#""geometry": {"default": "Geometry.Ghost", "other": "Geometry.Nope"}"#,
    );
    let files = [
        (GHOST, text.as_str()),
        ("models/entity/ghost.geo.json", GEOMETRY),
    ];
    let set = world(&files);
    let d = linked_in(&set);
    one_lazy_reference(
        &d,
        "minecraft:client_entity/description/geometry/other",
        "geometry.nope",
    );
    // The entry is kept as written.
    assert_eq!(
        ghost_of(&set).description.slots.geometry["other"].as_str(),
        "Geometry.Nope"
    );
    // With every geometry defined nothing is said.
    let text = ghost("1.10.0", r#""geometry": {"default": "geometry.ghost"}"#);
    let d = linked(&text, &[("models/entity/ghost.geo.json", GEOMETRY)]);
    assert!(
        d.iter().all(|f| f.code.kind() != Kind::UnresolvedReference),
        "{}",
        lines(&d).join("\n")
    );
}

const ANIMATION: &str =
    r#"{"format_version": "1.8.0", "animations": {"animation.ghost.float": {"loop": true}}}"#;
const ANIMATION_CONTROLLER: &str = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.ghost.move": {"states": {"default": {}}}}}"#;

// The rule is compiled with its target's feature.
#[cfg(feature = "animation")]
#[test]
fn reference_check_unknown_animation() {
    // R4: the prefix of the value, in lower case, picks the kind it is looked up as
    // (`ClientEntity::animation_kind`); a value with neither prefix is ignored silently.
    // Nothing defined: a note each, entries kept.
    let text = ghost(
        "1.10.0",
        r#""animations": {"float": "Animation.Ghost.Float", "move": "Controller.Animation.Ghost.Move", "nope": "animation.ghost.nope", "nope_too": "controller.animation.ghost.nope", "neither": "ghost.neither"}"#,
    );
    let others = [
        ("animations/ghost.animation.json", ANIMATION),
        (
            "animation_controllers/ghost.animation_controllers.json",
            ANIMATION_CONTROLLER,
        ),
    ];
    let mut files = vec![(GHOST, text.as_str())];
    files.extend_from_slice(&others);
    let set = world(&files);
    let d = linked_in(&set);
    expect_at(
        &d,
        "core/unresolved_reference",
        Severity::Info,
        "minecraft:client_entity/description/animations/nope",
    );
    expect_at(
        &d,
        "core/unresolved_reference",
        Severity::Info,
        "minecraft:client_entity/description/animations/nope_too",
    );
    assert_eq!(
        unresolved(&d),
        ["animation.ghost.nope", "controller.animation.ghost.nope"],
        "{}",
        lines(&d).join("\n")
    );
    assert!(d.iter().all(|f| f.severity < Severity::Warning));
    // An animation is no animation controller of the same name, and the other way round.
    let crossed = ghost(
        "1.10.0",
        r#""animations": {"a": "animation.ghost.move", "b": "controller.animation.ghost.float"}"#,
    );
    assert_eq!(
        unresolved(&linked(&crossed, &others)),
        ["animation.ghost.move", "controller.animation.ghost.float"]
    );
    assert_eq!(
        ghost_of(&set)
            .description
            .slots
            .animations
            .as_ref()
            .unwrap()
            .len(),
        5
    );
}

const RENDER_CONTROLLER: &str = r#"{"format_version": "1.8.0", "render_controllers": {"controller.render.ghost": {"geometry": "Geometry.default", "materials": [{"*": "Material.default"}], "textures": ["Texture.default"]}}}"#;

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn reference_check_unknown_render_controller() {
    // R5: the string form and the key of the object form, each looked up in lower case; a
    // controller nothing defines is kept, and vanilla logs nothing: a note.
    let text = ghost(
        "1.10.0",
        r#""render_controllers": ["Controller.Render.Ghost", "controller.render.nope", {"controller.render.ghost": "query.is_baby"}, {"Controller.Render.Nope_Too": "1.0"}]"#,
    );
    let others = [(
        "render_controllers/ghost.render_controllers.json",
        RENDER_CONTROLLER,
    )];
    let mut files = vec![(GHOST, text.as_str())];
    files.extend_from_slice(&others);
    let set = world(&files);
    let d = linked_in(&set);
    expect_at(
        &d,
        "core/unresolved_reference",
        Severity::Info,
        "minecraft:client_entity/description/render_controllers/1",
    );
    expect_at(
        &d,
        "core/unresolved_reference",
        Severity::Info,
        "minecraft:client_entity/description/render_controllers/3/Controller.Render.Nope_Too",
    );
    assert_eq!(
        unresolved(&d),
        ["controller.render.nope", "controller.render.nope_too"],
        "{}",
        lines(&d).join("\n")
    );
    assert!(d.iter().all(|f| f.severity < Severity::Warning));
    assert_eq!(
        ghost_of(&set)
            .description
            .slots
            .render_controllers
            .as_ref()
            .unwrap()
            .len(),
        4
    );
    // One defined controller, named once: nothing is said; with no file that defines it,
    // the same entry is the note.
    let text = ghost(
        "1.10.0",
        r#""render_controllers": ["controller.render.ghost"]"#,
    );
    let d = linked(&text, &others);
    assert!(unresolved(&d).is_empty(), "{}", lines(&d).join("\n"));
    one_lazy_reference(
        &linked(&text, &[]),
        "minecraft:client_entity/description/render_controllers/0",
        "controller.render.ghost",
    );
}

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn l21_a_link_finding_beside_an_empty_conditional_object_is_at_the_index_of_the_file() {
    const AT: &str = "minecraft:client_entity/description/render_controllers";
    let text = ghost(
        "1.10.0",
        r#""render_controllers": [{}, "controller.render.missing"]"#,
    );
    one_lazy_reference(
        &linked(&text, &[]),
        &format!("{AT}/1"),
        "controller.render.missing",
    );
    let text = ghost(
        "1.10.0",
        r#""render_controllers": ["controller.render.missing0", {}, {"controller.render.missing2": "1"}, {}, "controller.render.missing4"]"#,
    );
    let d = linked(&text, &[]);
    let at: Vec<String> = d
        .iter()
        .filter(|f| f.code.id() == "core/unresolved_reference")
        .map(|f| f.location.json_path.to_string())
        .collect();
    assert_eq!(
        at,
        [
            format!("{AT}/0"),
            format!("{AT}/2/controller.render.missing2"),
            format!("{AT}/4")
        ],
        "{}",
        lines(&d).join("\n")
    );
}

/// The definitions the reference-check tests resolve against are definitions to their own
/// families: each loads, alone under its identifier and without a warning. (An ignored test
/// stops at the link pass and could not tell.) The document that names them loads too, its
/// entries as written.
#[cfg(all(
    feature = "geometry",
    feature = "animation",
    feature = "render_controller",
    feature = "material"
))]
#[test]
fn the_definitions_the_reference_check_tests_resolve_against_load() {
    let text = ghost(
        "1.10.0",
        r#""geometry": {"default": "Geometry.Ghost"}, "animations": {"float": "Animation.Ghost.Float", "move": "Controller.Animation.Ghost.Move"}, "render_controllers": ["Controller.Render.Ghost"]"#,
    );
    let files = [
        (GHOST, text.as_str()),
        ("models/entity/ghost.geo.json", GEOMETRY),
        ("animations/ghost.animation.json", ANIMATION),
        (
            "animation_controllers/ghost.animation_controllers.json",
            ANIMATION_CONTROLLER,
        ),
        (
            "render_controllers/ghost.render_controllers.json",
            RENDER_CONTROLLER,
        ),
    ];
    let set = stack(&[pack(&files)]);
    let geometries = set.resource.geometries();
    assert_eq!(geometries.len(), 1);
    assert!(
        geometries
            .diagnostics()
            .all(|f| f.severity < Severity::Warning)
    );
    let animations = set.resource.animations();
    assert_eq!(animations.len(), 1);
    assert!(
        animations
            .diagnostics()
            .all(|f| f.severity < Severity::Warning)
    );
    let controllers = set.resource.animation_controllers();
    assert_eq!(controllers.len(), 1);
    assert!(
        controllers
            .diagnostics()
            .all(|f| f.severity < Severity::Warning)
    );
    let render_controllers = set.resource.render_controllers();
    assert_eq!(render_controllers.len(), 1);
    assert!(
        render_controllers
            .diagnostics()
            .all(|f| f.severity < Severity::Warning)
    );
    let entity = ghost_of(&set);
    silent(entity.diagnostics());
    assert_eq!(
        entity.description.slots.geometry["default"].as_str(),
        "Geometry.Ghost"
    );
    assert_eq!(
        entity.animation_kind("move"),
        Some(AnimationKind::Controller)
    );
}
