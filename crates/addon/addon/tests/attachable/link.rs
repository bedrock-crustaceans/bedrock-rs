//! The rows whose finding comes from the link pass: an `item` that names no known
//! item (`a17`, `a18`), an identifier a client entity also has (`a24`) and the references into
//! other families (materials, particles, geometry, animations, render controllers; `animate`
//! is checked at load). They are
//! written against `StackSet::link` and ignored until the link pass exists. What
//! loading alone says is pinned below, not ignored.

use bedrock_addon::diagnostic::{DiagnosticDetail, Diagnostics, Kind, ReferenceHint, Severity};
use bedrock_addon::families::attachable::ItemBindings;
use bedrock_addon::id::AttachableId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::StackSet;

use super::common::*;

const WAND: &str = "attachables/wand.json";

/// `example:wand` at 1.10.0 with `members`.
fn wand(members: &str) -> String {
    let comma = if members.is_empty() { "" } else { ", " };
    doc(
        "1.10.0",
        &format!(r#"{{"identifier": "example:wand"{comma}{members}}}"#),
    )
}

/// Requires no finding of `kind`.
#[track_caller]
fn none_of_kind(d: &Diagnostics, kind: Kind) {
    assert!(
        d.iter().all(|f| f.code.kind() != kind),
        "{}",
        lines(d).join("\n")
    );
}

/// The link findings about `attachables/wand.json` of `set`.
fn linked(set: &StackSet) -> Diagnostics {
    set.link()
        .into_iter()
        .filter(|finding| {
            finding
                .location
                .path
                .as_ref()
                .is_some_and(|path| path.as_str() == WAND)
        })
        .collect()
}

/// A behaviour-pack item `identifier`.
fn item(identifier: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.0", "minecraft:item": {{"description": {{"identifier": "{identifier}"}}, "components": {{}}}}}}"#
    )
}

/// The reference a finding is about: its kind, name and hint.
fn reference(d: &Diagnostics) -> Vec<(&'static str, String, Option<ReferenceHint>)> {
    d.iter()
        .filter(|f| f.code.kind() == Kind::UnresolvedReference)
        .filter_map(|f| match f.detail.as_deref() {
            Some(DiagnosticDetail::Reference { kind, id, hint }) => {
                Some((*kind, id.clone(), hint.clone()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a17_item_case_mismatch() {
    // Ours: the game says nothing here, an information.
    // Rule 22: the key differs from a known item only by case. Vanilla compares bytes, so
    // the binding never matches; the hint names the item meant.
    let wand = wand(r#""item": {"Minecraft:Bow": "1"}"#);
    let set = stack(&[pack(&[(WAND, &wand)])]);
    let d = linked(&set);
    expect_at(
        &d,
        "core/unresolved_reference",
        Severity::Info,
        "minecraft:attachable/description/item/Minecraft:Bow",
    );
    assert_eq!(
        reference(&d),
        [(
            "item",
            "Minecraft:Bow".to_owned(),
            Some(ReferenceHint::CaseMismatch("minecraft:bow".to_owned()))
        )]
    );
    // The binding is kept.
    assert_eq!(
        ItemBindings::from_registry(set.resource.attachables())
            .get("Minecraft:Bow")
            .len(),
        1
    );
    // The string form is checked alike, at `item` itself.
    let wand = self::wand(r#""item": "Minecraft:Bow""#);
    let set = stack(&[pack(&[(WAND, &wand)])]);
    expect_at(
        &linked(&set),
        "core/unresolved_reference",
        Severity::Info,
        "minecraft:attachable/description/item",
    );
    // The item as vanilla spells it resolves.
    let wand = self::wand(r#""item": {"minecraft:bow": "1"}"#);
    let set = stack(&[pack(&[(WAND, &wand)])]);
    none_of_kind(&linked(&set), Kind::UnresolvedReference);
}

#[test]
fn a18_item_unknown() {
    // Ours: the game says nothing here, an information.
    // Rule 21: a behaviour pack defines `example:wand` only.
    let wand = wand(r#""item": {"example:nothing": "1"}"#);
    let known = item("example:wand");
    let behavior = [("items/wand.json", known.as_str())];
    let resource = [(WAND, wand.as_str())];
    let set = stacks(
        &[pack(&behavior)],
        &[pack(&resource)],
        LoadOptions::default(),
    );
    let d = linked(&set);
    expect_at(
        &d,
        "core/unresolved_reference",
        Severity::Info,
        "minecraft:attachable/description/item/example:nothing",
    );
    assert_eq!(
        reference(&d),
        [("item", "example:nothing".to_owned(), None)]
    );
    // The pack's own item resolves, byte for byte: no `minecraft:` is assumed, and a member
    // that does not bind is not looked up.
    let ok = self::wand(r#""item": {"example:wand": "1", "example:zzz": "1"}"#);
    let resource = [(WAND, ok.as_str())];
    let set = stacks(
        &[pack(&behavior)],
        &[pack(&resource)],
        LoadOptions::default(),
    );
    none_of_kind(&linked(&set), Kind::UnresolvedReference);
    let bare = self::wand(r#""item": "wand""#);
    let resource = [(WAND, bare.as_str())];
    let set = stacks(
        &[pack(&behavior)],
        &[pack(&resource)],
        LoadOptions::default(),
    );
    assert_eq!(reference(&linked(&set)).len(), 1);
}

const ENTITY: &str = r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "Example:Wand", "textures": {"default": "textures/entity"}}}}"#;

// The rule is compiled with its target's feature.
#[cfg(feature = "entity")]
#[test]
fn a24_identifier_collides_with_client_entity_deviation() {
    // Ours: the game says nothing here, an information.
    // Vanilla keeps one shared identifier map: the attachable merges
    // into / replaces / is ignored against the client entity by min_engine_version"`.
    let wand = wand(r#""textures": {"other": "textures/attachable"}"#);
    let files = [("entity/wand.entity.json", ENTITY), (WAND, wand.as_str())];
    let set = stack(&[pack(&files)]);
    let d = linked(&set);
    // The identifiers compare in lower case; the finding is at the attachable.
    expect_at(
        &d,
        "core/identifier_collision",
        Severity::Info,
        "minecraft:attachable/description/identifier",
    );
    assert_eq!(
        d.iter()
            .filter(|f| f.code.id() == "core/identifier_collision")
            .count(),
        1
    );
}

#[test]
fn a24_the_attachable_and_the_client_entity_load_in_their_own_namespaces() {
    // What loading does today, with or without the link pass: neither merges into the other.
    let wand = wand(r#""textures": {"other": "textures/attachable"}"#);
    let files = [("entity/wand.entity.json", ENTITY), (WAND, wand.as_str())];
    let set = stack(&[pack(&files)]);
    let attachables = set.resource.attachables();
    let own = attachables
        .get(&AttachableId::parse("example:wand"))
        .unwrap();
    assert_eq!(own.description.slots.textures.len(), 1);
    assert_eq!(
        own.description.slots.textures["other"].as_str(),
        "textures/attachable"
    );
    assert_eq!(attachables.len(), 1);
    assert!(
        attachables
            .diagnostics()
            .all(|f| f.severity < Severity::Warning)
    );
    #[cfg(feature = "entity")]
    {
        let entities = set.resource.client_entities();
        let entity = entities
            .get(&bedrock_addon::id::EntityId::parse("example:wand"))
            .unwrap();
        assert_eq!(entity.description.slots.textures.len(), 1);
        assert!(entity.description.slots.textures.contains_key("default"));
    }
}

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r12_undefined_material() {
    let wand = wand(r#""materials": {"default": "No_Such_Material"}"#);
    let materials = r#"{"materials": {"version": "1.0.0", "entity_alphatest": {}}}"#;
    let files = [
        (WAND, wand.as_str()),
        // A `.material` file creates materials only when a list names it.
        (
            "materials/fancy.json",
            r#"[{"path": "materials/entity.material"}]"#,
        ),
        ("materials/entity.material", materials),
    ];
    let d = linked(&stack(&[pack(&files)]));
    // The lower-cased value, then the file (the client entity's text). The material list
    // is the add-on's own and no pack of the stack is the game's: the name may be the game's,
    // so the miss is an information (case (c)); the row's warning, with the game's
    // materials on the stack, is `tests/link/visual/attachable.rs`.
    expect(
        &d,
        "core/unresolved_reference_worded",
        Severity::Info,
        "Material \"no_such_material\" is referenced in file \"attachables/wand.json\" but not defined.",
    );
    expect_at(
        &d,
        "core/unresolved_reference_worded",
        Severity::Info,
        "minecraft:attachable/description/materials/default",
    );
    assert_eq!(kinds(&d), ["material"]);
}

const PARTICLE: &str = r#"{"format_version": "1.10.0", "particle_effect": {"description": {"identifier": "demo:smoke", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}}, "components": {"minecraft:emitter_rate_instant": {"num_particles": 1}, "minecraft:emitter_lifetime_once": {"active_time": 1}}}}"#;

// The rule is compiled with its target's feature.
#[cfg(feature = "particle")]
#[test]
fn r12_unknown_particle() {
    let wand = wand(r#""particle_effects": {"x": "demo:nope", "smoke": "demo:smoke"}"#);
    let files = [
        (WAND, wand.as_str()),
        ("particles/smoke.particle.json", PARTICLE),
    ];
    let d = linked(&stack(&[pack(&files)]));
    expect(
        &d,
        "core/unresolved_reference_worded",
        Severity::Error,
        "Entity .json 'attachables/wand.json' referenced particle 'demo:nope' that doesn't exist",
    );
    expect_at(
        &d,
        "core/unresolved_reference_worded",
        Severity::Error,
        "minecraft:attachable/description/particle_effects/x",
    );
    // Level 3, the entry skipped, the file kept: a kept error.
    assert!(
        d.iter()
            .filter(|f| f.code.id() == "core/unresolved_reference_worded")
            .all(|f| f.kept)
    );
    assert_eq!(kinds(&d), ["particle"]);
    assert_eq!(
        d.iter()
            .filter(|f| f.code.id() == "core/unresolved_reference_worded")
            .count(),
        1
    );
}

/// The names the `UnresolvedReference` findings of `d` are about, in order.
fn unresolved(d: &Diagnostics) -> Vec<String> {
    reference(d).into_iter().map(|(_, id, _)| id).collect()
}

/// The kinds of target the `UnresolvedReference` findings of `d` name, in order.
fn kinds(d: &Diagnostics) -> Vec<&'static str> {
    reference(d).into_iter().map(|(kind, _, _)| kind).collect()
}

/// Requires the one `UnresolvedReference` of `d` to be a note at `path` about `id`: what
/// vanilla resolves lazily and never logs is kept and reported at info.
#[track_caller]
fn one_lazy_reference(d: &Diagnostics, path: &str, id: &str) {
    expect_at(d, "core/unresolved_reference", Severity::Info, path);
    assert_eq!(unresolved(d), [id.to_owned()], "{}", lines(d).join("\n"));
    assert!(
        d.iter()
            .filter(|f| f.code.kind() == Kind::UnresolvedReference)
            .all(|f| f.severity == Severity::Info),
        "{}",
        lines(d).join("\n")
    );
}

const GEOMETRY: &str = r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.wand", "texture_width": 16, "texture_height": 16}, "bones": [{"name": "root"}]}]}"#;

// The rule is compiled with its target's feature.
#[cfg(feature = "geometry")]
#[test]
fn r12_unknown_geometry() {
    // R3: the value is looked up in lower case; a geometry nothing defines is kept, and
    // vanilla logs nothing (it resolves lazily): a note.
    let wand = wand(r#""geometry": {"default": "Geometry.Wand", "other": "Geometry.Nope"}"#);
    let files = [
        (WAND, wand.as_str()),
        ("models/entity/wand.geo.json", GEOMETRY),
    ];
    let set = stack(&[pack(&files)]);
    let d = linked(&set);
    one_lazy_reference(
        &d,
        "minecraft:attachable/description/geometry/other",
        "geometry.nope",
    );
    assert_eq!(kinds(&d), ["geometry"]);
    // The entry is kept as written.
    let attachables = set.resource.attachables();
    let own = attachables
        .get(&AttachableId::parse("example:wand"))
        .unwrap();
    assert_eq!(
        own.description.slots.geometry["other"].as_str(),
        "Geometry.Nope"
    );
    // With every geometry defined nothing is said.
    let wand = self::wand(r#""geometry": {"default": "geometry.wand"}"#);
    let files = [
        (WAND, wand.as_str()),
        ("models/entity/wand.geo.json", GEOMETRY),
    ];
    none_of_kind(&linked(&stack(&[pack(&files)])), Kind::UnresolvedReference);
}

const ANIMATION: &str =
    r#"{"format_version": "1.8.0", "animations": {"animation.wand.hold": {"loop": true}}}"#;
const ANIMATION_CONTROLLER: &str = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.wand.move": {"states": {"default": {}}}}}"#;

// The rule is compiled with its target's feature.
#[cfg(feature = "animation")]
#[test]
fn r12_unknown_animation() {
    // R4: the prefix of the value, in lower case, picks the kind it is looked up as; a value
    // with neither prefix is ignored silently. Nothing defined: a note each, entries kept.
    let wand = wand(
        r#""animations": {"hold": "Animation.Wand.Hold", "move": "Controller.Animation.Wand.Move", "nope": "animation.wand.nope", "nope_too": "controller.animation.wand.nope", "neither": "wand.neither"}"#,
    );
    let files = [
        (WAND, wand.as_str()),
        ("animations/wand.animation.json", ANIMATION),
        (
            "animation_controllers/wand.animation_controllers.json",
            ANIMATION_CONTROLLER,
        ),
    ];
    let set = stack(&[pack(&files)]);
    let d = linked(&set);
    expect_at(
        &d,
        "core/unresolved_reference",
        Severity::Info,
        "minecraft:attachable/description/animations/nope",
    );
    expect_at(
        &d,
        "core/unresolved_reference",
        Severity::Info,
        "minecraft:attachable/description/animations/nope_too",
    );
    assert_eq!(
        unresolved(&d),
        ["animation.wand.nope", "controller.animation.wand.nope"],
        "{}",
        lines(&d).join("\n")
    );
    assert_eq!(kinds(&d), ["animation", "animation"]);
    assert!(d.iter().all(|f| f.severity < Severity::Warning));
    // An animation is no animation controller of the same name, and the other way round.
    let wand = self::wand(
        r#""animations": {"a": "animation.wand.move", "b": "controller.animation.wand.hold"}"#,
    );
    let crossed = [
        (WAND, wand.as_str()),
        ("animations/wand.animation.json", ANIMATION),
        (
            "animation_controllers/wand.animation_controllers.json",
            ANIMATION_CONTROLLER,
        ),
    ];
    assert_eq!(
        unresolved(&linked(&stack(&[pack(&crossed)]))),
        ["animation.wand.move", "controller.animation.wand.hold"]
    );
    let attachables = set.resource.attachables();
    let own = attachables
        .get(&AttachableId::parse("example:wand"))
        .unwrap();
    assert_eq!(own.description.slots.animations.as_ref().unwrap().len(), 5);
}

const RENDER_CONTROLLER: &str = r#"{"format_version": "1.8.0", "render_controllers": {"controller.render.wand": {"geometry": "Geometry.default", "materials": [{"*": "Material.default"}], "textures": ["Texture.default"]}}}"#;

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r12_unknown_render_controller() {
    // R5: the string form and the key of the object form, each looked up in lower case; a
    // controller nothing defines is kept, and vanilla logs nothing: a note.
    let wand = wand(
        r#""render_controllers": ["Controller.Render.Wand", "controller.render.nope", {"controller.render.wand": "query.is_baby"}, {"Controller.Render.Nope_Too": "1.0"}]"#,
    );
    let files = [
        (WAND, wand.as_str()),
        (
            "render_controllers/wand.render_controllers.json",
            RENDER_CONTROLLER,
        ),
    ];
    let set = stack(&[pack(&files)]);
    let d = linked(&set);
    expect_at(
        &d,
        "core/unresolved_reference",
        Severity::Info,
        "minecraft:attachable/description/render_controllers/1",
    );
    expect_at(
        &d,
        "core/unresolved_reference",
        Severity::Info,
        "minecraft:attachable/description/render_controllers/3/Controller.Render.Nope_Too",
    );
    assert_eq!(
        unresolved(&d),
        ["controller.render.nope", "controller.render.nope_too"],
        "{}",
        lines(&d).join("\n")
    );
    assert_eq!(kinds(&d), ["render controller", "render controller"]);
    assert!(d.iter().all(|f| f.severity < Severity::Warning));
    let attachables = set.resource.attachables();
    let own = attachables
        .get(&AttachableId::parse("example:wand"))
        .unwrap();
    assert_eq!(
        own.description
            .slots
            .render_controllers
            .as_ref()
            .unwrap()
            .len(),
        4
    );
    // One defined controller, named once: nothing is said.
    let wand = self::wand(r#""render_controllers": ["controller.render.wand"]"#);
    let files = [
        (WAND, wand.as_str()),
        (
            "render_controllers/wand.render_controllers.json",
            RENDER_CONTROLLER,
        ),
    ];
    let d = linked(&stack(&[pack(&files)]));
    assert!(unresolved(&d).is_empty(), "{}", lines(&d).join("\n"));
    one_lazy_reference(
        &linked(&stack(&[pack(&[(WAND, wand.as_str())])])),
        "minecraft:attachable/description/render_controllers/0",
        "controller.render.wand",
    );
}

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn l21_a_link_finding_beside_an_empty_conditional_object_is_at_the_index_of_the_file() {
    const AT: &str = "minecraft:attachable/description/render_controllers";
    let wand = wand(r#""render_controllers": [{}, "controller.render.missing"]"#);
    one_lazy_reference(
        &linked(&stack(&[pack(&[(WAND, wand.as_str())])])),
        &format!("{AT}/1"),
        "controller.render.missing",
    );
    let wand = self::wand(
        r#""render_controllers": ["controller.render.missing0", {}, {"controller.render.missing2": "1"}, {}, "controller.render.missing4"]"#,
    );
    let d = linked(&stack(&[pack(&[(WAND, wand.as_str())])]));
    let at: Vec<String> = d
        .iter()
        .filter(|f| f.code.kind() == Kind::UnresolvedReference)
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

/// The definitions the tests below resolve against are definitions to their own families:
/// each loads, alone under its identifier and without a warning. (An ignored test stops at
/// the link pass and could not tell.)
#[cfg(all(
    feature = "geometry",
    feature = "animation",
    feature = "render_controller",
    feature = "material"
))]
#[test]
fn the_definitions_the_r12_tests_resolve_against_load() {
    let files = [
        ("models/entity/wand.geo.json", GEOMETRY),
        ("animations/wand.animation.json", ANIMATION),
        (
            "animation_controllers/wand.animation_controllers.json",
            ANIMATION_CONTROLLER,
        ),
        (
            "render_controllers/wand.render_controllers.json",
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
    let render = set.resource.render_controllers();
    assert_eq!(render.len(), 1);
    assert!(render.diagnostics().all(|f| f.severity < Severity::Warning));
}

/// Until the link pass runs, a document with references nothing defines loads, and loading
/// says nothing of them.
#[test]
fn the_document_loads_and_loading_resolves_no_reference() {
    let wand = wand(
        r#""item": {"example:nothing": "1"}, "materials": {"default": "no_such_material"}, "geometry": {"default": "geometry.nope"}, "particle_effects": {"x": "demo:nope"}, "animations": {"a": "animation.nope"}, "render_controllers": ["controller.render.nope"], "sound_effects": {"s": "nope"}"#,
    );
    let files = [(WAND, wand.as_str())];
    let set = stack(&[pack(&files)]);
    let attachables = set.resource.attachables();
    let own = attachables
        .get(&AttachableId::parse("example:wand"))
        .unwrap();
    assert!(
        own.diagnostics()
            .iter()
            .all(|f| f.code.id() == "core/molang_not_checked")
    );
    assert!(
        attachables
            .diagnostics()
            .all(|f| f.code.id() == "core/molang_not_checked")
    );
    assert_eq!(own.bound_items().len(), 1);
    assert!(
        own.description
            .slots
            .particle_effects
            .as_ref()
            .unwrap()
            .contains_key("x")
    );
}
