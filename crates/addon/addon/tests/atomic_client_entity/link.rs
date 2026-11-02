//! `r01`–`r07`: the render-controller lint, and the cross-references (ours, lint only). The controller a
//! document names is another family's, resolved in the link pass (when the link pass runs), and so are the five maps of names, so the rows are written against
//! `StackSet::link` and ignored until the link pass is implemented. What loading alone says
//! of them is pinned below, not ignored.

use bedrock_addon::diagnostic::{DiagnosticDetail, Diagnostics, Severity};
use bedrock_addon::id::AtomicClientEntityId;
use bedrock_addon::pack::StackSet;

use super::common::*;

const UNSUPPORTED: &str = "Skipping an Atomic Entity - Unsupported render controller features (conditional enable, dynamic resources, multiple materials, or part visibility).";
const FILTER_LIGHTING: &str = "Atomic Entity - filter_lighting is not supported.";
const LIGHT_EXPRESSION: &str =
    "Atomic Entity - light_color_multiplier expressions are not supported.";

/// The static controller of `r01`, with `extra` members after its own.
fn controller(name: &str, extra: &str) -> String {
    let comma = if extra.is_empty() { "" } else { ", " };
    format!(
        r#""{name}": {{"geometry": "Geometry.default", "materials": [{{"*": "Material.default"}}], "textures": ["Texture.default"]{comma}{extra}}}"#
    )
}

/// A render-controller file holding `controllers`.
fn controllers(controllers: &[String]) -> String {
    format!(
        r#"{{"format_version": "1.8.0", "render_controllers": {{{}}}}}"#,
        controllers.join(", ")
    )
}

/// An atomic `demo:wisp` that names `names`.
fn wisp(names: &[&str]) -> String {
    let names: Vec<String> = names.iter().map(|name| format!("\"{name}\"")).collect();
    format!(
        r#"{{"format_version": "beta", "minecraft:atomic_client_entity": {{"description": {{"identifier": "demo:wisp", "geometry": {{"default": "geometry.wisp"}}, "render_controllers": [{}]}}}}}}"#,
        names.join(", ")
    )
}

/// The link findings about the atomic document of a pack with `wisp` and `controllers`.
fn linked(wisp: &str, controllers: &str) -> Diagnostics {
    let files = [
        ("atomic_client_entities/wisp.json", wisp),
        (
            "render_controllers/wisp.render_controllers.json",
            controllers,
        ),
    ];
    let set = stack(&[pack(&files)]);
    set.link()
        .into_iter()
        .filter(|finding| {
            finding
                .location
                .path
                .as_ref()
                .is_some_and(|path| path.as_str().starts_with("atomic_client_entities/"))
        })
        .collect()
}

const IGNORED: &str = "the render controller an atomic client entity names is resolved in the link pass; un-ignored when the link pass is implemented";

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r01_static_controller() {
    let _ = IGNORED;
    let d = linked(
        &wisp(&["controller.render.wisp"]),
        &controllers(&[controller("controller.render.wisp", "")]),
    );
    clean(&d);
}

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r02_arrays() {
    let dynamic = r#""controller.render.wisp": {"arrays": {"textures": {"Array.skins": ["Texture.default", "Texture.other"]}}, "geometry": "Geometry.default", "materials": [{"*": "Material.default"}], "textures": ["Array.skins[query.variant]"]}"#;
    let d = linked(
        &wisp(&["controller.render.wisp"]),
        &controllers(&[dynamic.to_owned()]),
    );
    expect(
        &d,
        "AtomicUnsupportedRenderController",
        Severity::Warning,
        UNSUPPORTED,
    );
}

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r03_part_visibility() {
    let d = linked(
        &wisp(&["controller.render.wisp"]),
        &controllers(&[controller(
            "controller.render.wisp",
            r#""part_visibility": [{"head": "query.is_baby"}]"#,
        )]),
    );
    expect(
        &d,
        "AtomicUnsupportedRenderController",
        Severity::Warning,
        UNSUPPORTED,
    );
}

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r04_filter_lighting() {
    let d = linked(
        &wisp(&["controller.render.wisp"]),
        &controllers(&[controller(
            "controller.render.wisp",
            r#""filter_lighting": true"#,
        )]),
    );
    expect(
        &d,
        "AtomicFilterLightingIgnored",
        Severity::Warning,
        FILTER_LIGHTING,
    );
    // `false` leaves the flag clear.
    let d = linked(
        &wisp(&["controller.render.wisp"]),
        &controllers(&[controller(
            "controller.render.wisp",
            r#""filter_lighting": false"#,
        )]),
    );
    none(&d, "AtomicFilterLightingIgnored");
}

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r05_light_multiplier_expression() {
    let d = linked(
        &wisp(&["controller.render.wisp"]),
        &controllers(&[controller(
            "controller.render.wisp",
            r#""light_color_multiplier": "query.is_baby ? 2.0 : 1.0""#,
        )]),
    );
    expect(&d, "NoEffect", Severity::Warning, LIGHT_EXPRESSION);
}

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r06_light_multiplier_constant_and_r06b_constant_string() {
    for value in ["2.0", "\"2.0\""] {
        let d = linked(
            &wisp(&["controller.render.wisp"]),
            &controllers(&[controller(
                "controller.render.wisp",
                &format!(r#""light_color_multiplier": {value}"#),
            )]),
        );
        clean(&d);
    }
}

// The rule is compiled with its target's feature.
#[cfg(all(feature = "render_controller", feature = "material"))]
#[test]
fn r07_two_controllers_one_bad() {
    let d = linked(
        &wisp(&["controller.render.ok", "controller.render.arrays"]),
        &controllers(&[
            controller("controller.render.ok", ""),
            controller(
                "controller.render.arrays",
                r#""arrays": {"textures": {"Array.skins": ["Texture.default"]}}"#,
            ),
        ]),
    );
    let unsupported: Vec<_> = d
        .iter()
        .filter(|f| f.code.name() == "AtomicUnsupportedRenderController")
        .collect();
    assert_eq!(unsupported.len(), 1);
    // At the second name of the document's list.
    assert_eq!(
        unsupported[0].location.json_path.to_string(),
        "minecraft:atomic_client_entity/description/render_controllers/1"
    );
}

/// Until the link pass runs, the documents of `r02`–`r07` load, and loading says nothing of
/// the controllers they name: vanilla finds these at render time, never at load.
#[test]
fn r01_to_r07_the_documents_load_and_loading_does_not_resolve_controllers() {
    let wisp = wisp(&["controller.render.ok", "controller.render.absent"]);
    let files = [("atomic_client_entities/wisp.json", wisp.as_str())];
    let set = stack(&[pack(&files)]);
    let entities = set.resource.atomic_client_entities();
    let entity = entities
        .get(&AtomicClientEntityId::parse("demo:wisp"))
        .unwrap();
    silent(entity.diagnostics());
    assert!(entities.diagnostics().next().is_none());
}

// ---- R5 and "Cross-references (ours, lint only)" ----

const WISP: &str = "atomic_client_entities/wisp.json";

/// An atomic `demo:wisp` with `members` after the identifier.
fn wisp_with(members: &str) -> String {
    format!(
        r#"{{"format_version": "beta", "minecraft:atomic_client_entity": {{"description": {{"identifier": "demo:wisp", {members}}}}}}}"#
    )
}

/// The link findings about the atomic document of `set`.
fn linked_in(set: &StackSet) -> Diagnostics {
    set.link()
        .into_iter()
        .filter(|finding| {
            finding
                .location
                .path
                .as_ref()
                .is_some_and(|path| path.as_str() == WISP)
        })
        .collect()
}

/// The names the `UnresolvedReference` findings of `d` are about, in order.
fn unresolved(d: &Diagnostics) -> Vec<String> {
    d.iter()
        .filter(|f| f.code.name() == "UnresolvedReference")
        .filter_map(|f| match f.detail.as_deref() {
            Some(DiagnosticDetail::Reference { id, .. }) => Some(id.clone()),
            _ => None,
        })
        .collect()
}

/// Requires the one `UnresolvedReference` of `d` to be a note at `path` about `id`: the
/// atomic loader has no existence check at load, so the name is kept and the finding is
/// ours, at info.
#[track_caller]
fn one_lint(d: &Diagnostics, path: &str, id: &str) {
    expect_at(d, "UnresolvedReference", Severity::Info, path);
    assert_eq!(unresolved(d), [id.to_owned()], "{}", lines(d).join("\n"));
    assert!(
        d.iter().all(|f| f.severity < Severity::Warning),
        "{}",
        lines(d).join("\n")
    );
}

const R13: &str = "the names an atomic client entity refers to are resolved in the link pass; un-ignored when the link pass is implemented";

// The rule is compiled with its target's feature.
#[cfg(all(
    feature = "render_controller",
    feature = "material",
    feature = "geometry"
))]
#[test]
fn r5_a_named_controller_that_does_not_exist() {
    let _ = R13;
    // R5: "the cross-reference warning client_entity uses for unknown controllers", at info;
    // the features of the controller that IS defined are R1 to R4's (r01–r07).
    let text = wisp(&["controller.render.wisp", "controller.render.absent"]);
    let set = stack(&[pack(&[
        (WISP, text.as_str()),
        (
            "render_controllers/wisp.render_controllers.json",
            &controllers(&[controller("controller.render.wisp", "")]),
        ),
        // The geometry `wisp()` names: undefined, it would be a second lint.
        ("models/entity/wisp.geo.json", GEOMETRY),
    ])]);
    let d = linked_in(&set);
    one_lint(
        &d,
        "minecraft:atomic_client_entity/description/render_controllers/1",
        "controller.render.absent",
    );
    // The name is kept: nothing is skipped at load.
    let entities = set.resource.atomic_client_entities();
    let entity = entities
        .get(&AtomicClientEntityId::parse("demo:wisp"))
        .unwrap();
    assert_eq!(
        entity
            .description
            .render_controllers
            .as_ref()
            .unwrap()
            .len(),
        2
    );
}

const GEOMETRY: &str = r#"{"format_version": "1.12.0", "minecraft:geometry": [{"description": {"identifier": "geometry.wisp", "texture_width": 16, "texture_height": 16}, "bones": [{"name": "root"}]}]}"#;
const ANIMATION: &str =
    r#"{"format_version": "1.8.0", "animations": {"animation.wisp.float": {"loop": true}}}"#;
const ANIMATION_CONTROLLER: &str = r#"{"format_version": "1.10.0", "animation_controllers": {"controller.animation.wisp.move": {"states": {"default": {}}}}}"#;
const MATERIALS: &str = r#"{"materials": {"version": "1.0.0", "entity_alphatest": {}}}"#;
const PARTICLE: &str = r#"{"format_version": "1.10.0", "particle_effect": {"description": {"identifier": "demo:smoke", "basic_render_parameters": {"material": "particles_alpha", "texture": "textures/particle/particles"}}, "components": {"minecraft:emitter_rate_instant": {"num_particles": 1}, "minecraft:emitter_lifetime_once": {"active_time": 1}}}}"#;
const SOUNDS: &str = r#"{"format_version": "1.14.0", "sound_definitions": {"wisp.hum": {"category": "neutral", "sounds": ["sounds/wisp/hum"]}}}"#;

/// The files that define one name of each of the five maps (with the list that names the material file and the audio the sound plays).
const DEFINITIONS: [(&str, &str); 8] = [
    // A `.material` file creates materials only when a list names it.
    (
        "materials/fancy.json",
        r#"[{"path": "materials/entity.material"}]"#,
    ),
    ("sounds/wisp/hum.ogg", ""),
    ("models/entity/wisp.geo.json", GEOMETRY),
    ("animations/wisp.animation.json", ANIMATION),
    (
        "animation_controllers/wisp.animation_controllers.json",
        ANIMATION_CONTROLLER,
    ),
    ("materials/entity.material", MATERIALS),
    ("particles/smoke.particle.json", PARTICLE),
    ("sounds/sound_definitions.json", SOUNDS),
];

/// The description members that name what [`DEFINITIONS`] define, each as written there.
const DEFINED: &str = r#""geometry": {"default": "geometry.wisp"}, "animations": {"float": "animation.wisp.float", "move": "controller.animation.wisp.move"}, "materials": {"default": "entity_alphatest"}, "particle_effects": {"smoke": "demo:smoke"}, "sound_effects": {"hum": "wisp.hum"}"#;

// The rule is compiled with its target's feature.
#[cfg(all(
    feature = "geometry",
    feature = "animation",
    feature = "render_controller",
    feature = "material",
    feature = "particle",
    feature = "sound_definitions"
))]
#[test]
fn cross_references_of_the_five_maps() {
    let link = |members: &str| {
        let text = wisp_with(members);
        let mut files = vec![(WISP, text.as_str())];
        files.extend_from_slice(&DEFINITIONS);
        linked_in(&stack(&[pack(&files)]))
    };
    // Every name defined: nothing is said.
    let d = link(DEFINED);
    assert!(unresolved(&d).is_empty(), "{}", lines(&d).join("\n"));
    // One unknown name per map, beside a defined one where the map takes two: a note at
    // the entry, and the document loads (info for every map, the material's too,
    // because the atomic callback has no load-time check).
    for (members, path, id) in [
        (
            r#""geometry": {"default": "geometry.wisp", "other": "geometry.nope"}"#,
            "geometry/other",
            "geometry.nope",
        ),
        (
            r#""animations": {"float": "animation.wisp.float", "nope": "animation.wisp.nope"}"#,
            "animations/nope",
            "animation.wisp.nope",
        ),
        (
            r#""animations": {"move": "controller.animation.wisp.move", "nope": "controller.animation.wisp.nope"}"#,
            "animations/nope",
            "controller.animation.wisp.nope",
        ),
        (
            r#""materials": {"default": "entity_alphatest", "other": "no_such_material"}"#,
            "materials/other",
            "no_such_material",
        ),
        (
            r#""particle_effects": {"smoke": "demo:smoke", "x": "demo:nope"}"#,
            "particle_effects/x",
            "demo:nope",
        ),
        (
            r#""sound_effects": {"hum": "wisp.hum", "s": "wisp.nope"}"#,
            "sound_effects/s",
            "wisp.nope",
        ),
    ] {
        let d = link(members);
        let path = format!("minecraft:atomic_client_entity/description/{path}");
        if path.contains("sound_effects") {
            // One code per kind of miss: a sound event is `UnknownSound`, as
            // for a client entity (R8).
            expect_at(&d, "UnknownSound", Severity::Info, &path);
            let about: Vec<_> = d
                .iter()
                .filter_map(|f| match f.detail.as_deref() {
                    Some(DiagnosticDetail::Reference { kind, id, .. }) => {
                        Some((f.code.name(), *kind, id.as_str()))
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(about, [("UnknownSound", "sound event", id)]);
            assert!(d.iter().all(|f| f.severity < Severity::Warning));
            continue;
        }
        one_lint(&d, &path, id);
    }
    // An animation is no animation controller of the same name, and the other way round.
    let d = link(
        r#""animations": {"a": "animation.wisp.move", "b": "controller.animation.wisp.float"}"#,
    );
    assert_eq!(
        unresolved(&d),
        ["animation.wisp.move", "controller.animation.wisp.float"]
    );
}

/// The definitions the cross-reference tests resolve against are definitions to their own
/// families: each loads, alone under its name and without a warning; and the document that
/// names them all loads and says nothing. (An ignored test stops at the link pass and could
/// not tell.)
#[cfg(all(
    feature = "geometry",
    feature = "animation",
    feature = "render_controller",
    feature = "material",
    feature = "particle",
    feature = "sound_definitions"
))]
#[test]
fn the_definitions_the_cross_reference_tests_resolve_against_load() {
    let text = wisp_with(DEFINED);
    let mut files = vec![(WISP, text.as_str())];
    files.extend_from_slice(&DEFINITIONS);
    let set = stack(&[pack(&files)]);
    let rp = &set.resource;
    assert_eq!(rp.geometries().len(), 1);
    assert!(
        rp.geometries()
            .diagnostics()
            .all(|f| f.severity < Severity::Warning)
    );
    assert_eq!(rp.animations().len(), 1);
    assert!(
        rp.animations()
            .diagnostics()
            .all(|f| f.severity < Severity::Warning)
    );
    assert_eq!(rp.animation_controllers().len(), 1);
    assert!(
        rp.animation_controllers()
            .diagnostics()
            .all(|f| f.severity < Severity::Warning)
    );
    assert_eq!(rp.particles().len(), 1);
    assert!(
        rp.particles()
            .diagnostics()
            .all(|f| f.severity < Severity::Warning)
    );
    assert!(rp.materials().get("entity_alphatest").is_some());
    assert!(
        rp.materials()
            .diagnostics()
            .all(|f| f.severity < Severity::Warning),
        "{:?}",
        rp.materials()
            .diagnostics()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert!(
        rp.sound_definitions()
            .value()
            .sound_definitions
            .keys()
            .any(|name| name.as_str() == "wisp.hum")
    );
    assert!(
        rp.sound_definitions()
            .diagnostics()
            .all(|f| f.severity < Severity::Warning),
        "{:?}",
        rp.sound_definitions()
            .diagnostics()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    let entities = rp.atomic_client_entities();
    let entity = entities
        .get(&AtomicClientEntityId::parse("demo:wisp"))
        .unwrap();
    silent(entity.diagnostics());
    assert!(entities.diagnostics().next().is_none());
}
