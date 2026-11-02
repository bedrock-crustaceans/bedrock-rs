//! The link tests of the client visuals (20 rows R1 to R26). One file per source
//! family; the worlds are built here from literals.

#![allow(dead_code, reason = "each feature set uses part of the fixtures")]

use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail, Diagnostics, Phase, Severity};

#[cfg(feature = "atomic_client_entity")]
mod atomic_client_entity;
#[cfg(all(feature = "attachable", feature = "item"))]
mod attachable;
#[cfg(all(
    feature = "animation",
    feature = "animation_controller",
    feature = "client_entity"
))]
mod client_animation;
#[cfg(all(feature = "entity", feature = "client_entity"))]
mod client_entity;
#[cfg(feature = "geometry")]
mod geometry;
#[cfg(all(feature = "render_controller", feature = "material"))]
mod render_controller;

/// What a finding says: the code's name, the severity, the JSON path, and the kind and the
/// name of its reference detail (empty without one).
pub type Told = (&'static str, Severity, String, &'static str, String);

/// [`Told`] of one finding.
pub fn told_of(finding: &Diagnostic) -> Told {
    let (kind, id) = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => (*kind, id.clone()),
        _ => ("", String::new()),
    };
    (
        finding.code.name(),
        finding.severity,
        finding.location.json_path.to_string(),
        kind,
        id,
    )
}

/// The findings in the file `path`, each of the link phase and of the pack `pack`.
#[track_caller]
pub fn told(findings: &Diagnostics, pack: &str, path: &str) -> Vec<Told> {
    super::common::in_file(findings, path)
        .into_iter()
        .map(|finding| {
            assert_eq!(finding.phase, Phase::Link, "{finding:?}");
            let from = finding.location.pack.as_ref().map(|pack| pack.name());
            assert_eq!(from, Some(pack), "{finding:?}");
            told_of(finding)
        })
        .collect()
}

/// A finding as a test writes it.
pub fn says(
    code: &'static str,
    severity: Severity,
    path: &str,
    kind: &'static str,
    id: &str,
) -> Told {
    (code, severity, path.to_owned(), kind, id.to_owned())
}

/// The text of the one finding in `path` whose code is called `code`.
#[track_caller]
pub fn text_of(findings: &Diagnostics, path: &str, code: &str) -> String {
    let found: Vec<_> = super::common::in_file(findings, path)
        .into_iter()
        .filter(|finding| finding.code.name() == code)
        .collect();
    assert_eq!(found.len(), 1, "{found:#?}");
    found[0].code.to_string()
}

/// A behaviour-pack entity `id`.
pub fn entity(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.0", "minecraft:entity": {{"description": {{"identifier": "{id}", "is_spawnable": true, "is_summonable": true, "is_experimental": false}}, "components": {{}}}}}}"#
    )
}

/// A client entity `id` at 1.10.0 with `members` after the identifier.
pub fn client_entity(id: &str, members: &str) -> String {
    let comma = if members.is_empty() { "" } else { ", " };
    format!(
        r#"{{"format_version": "1.10.0", "minecraft:client_entity": {{"description": {{"identifier": "{id}"{comma}{members}}}}}}}"#
    )
}

/// An attachable `id` at 1.10.0 with `members` after the identifier.
pub fn attachable(id: &str, members: &str) -> String {
    let comma = if members.is_empty() { "" } else { ", " };
    format!(
        r#"{{"format_version": "1.10.0", "minecraft:attachable": {{"description": {{"identifier": "{id}"{comma}{members}}}}}}}"#
    )
}

/// An atomic client entity `id` with `members` after the identifier.
pub fn atomic(id: &str, members: &str) -> String {
    let comma = if members.is_empty() { "" } else { ", " };
    format!(
        r#"{{"format_version": "beta", "minecraft:atomic_client_entity": {{"description": {{"identifier": "{id}"{comma}{members}}}}}}}"#
    )
}

/// A geometry file with the one geometry `id`.
pub fn geometry(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.12.0", "minecraft:geometry": [{{"description": {{"identifier": "{id}", "texture_width": 16, "texture_height": 16}}, "bones": [{{"name": "root"}}]}}]}}"#
    )
}

/// An animation file with the one animation `id` and the members `members`.
pub fn animation(id: &str, members: &str) -> String {
    let comma = if members.is_empty() { "" } else { ", " };
    format!(
        r#"{{"format_version": "1.8.0", "animations": {{"{id}": {{"loop": true{comma}{members}}}}}}}"#
    )
}

/// An animation controller file with the one controller `id`, whose state `default` has
/// `members`.
pub fn animation_controller(id: &str, members: &str) -> String {
    format!(
        r#"{{"format_version": "1.10.0", "animation_controllers": {{"{id}": {{"states": {{"default": {{{members}}}}}}}}}}}"#
    )
}

/// A render controller file with the one controller `id` drawing the `default` slots, and
/// `extra` members after them.
pub fn render_controller(id: &str, extra: &str) -> String {
    let comma = if extra.is_empty() { "" } else { ", " };
    format!(
        r#"{{"format_version": "1.8.0", "render_controllers": {{"{id}": {{"geometry": "Geometry.default", "materials": [{{"*": "Material.default"}}], "textures": ["Texture.default"]{comma}{extra}}}}}}}"#
    )
}

/// A render controller file with the one controller `id` whose members are `members`.
pub fn render_controller_of(id: &str, members: &str) -> String {
    format!(r#"{{"format_version": "1.8.0", "render_controllers": {{"{id}": {{{members}}}}}}}"#)
}

/// A particle effect `id` that loads without a finding of its own.
pub fn particle(id: &str) -> String {
    format!(
        r#"{{"format_version": "1.10.0", "particle_effect": {{"description": {{"identifier": "{id}", "basic_render_parameters": {{"material": "particles_alpha", "texture": "textures/particle/particles"}}}}, "components": {{"minecraft:emitter_rate_instant": {{"num_particles": 1}}, "minecraft:emitter_lifetime_once": {{"active_time": 1}}}}}}}}"#
    )
}

/// `sounds/sound_definitions.json` with the one sound event `name`.
pub fn sound_definitions(name: &str) -> String {
    format!(
        r#"{{"format_version": "1.14.0", "sound_definitions": {{"{name}": {{"category": "neutral", "sounds": ["sounds/demo/sound"]}}}}}}"#
    )
}

/// The list file that makes the materials of `materials/entity.material` exist.
pub const FANCY: (&str, &str) = (
    "materials/fancy.json",
    r#"[{"path": "materials/entity.material"}]"#,
);

/// `materials/entity.material` with the materials `members`.
pub fn materials(members: &str) -> String {
    format!(r#"{{"materials": {{"version": "1.0.0", {members}}}}}"#)
}

/// A material an actor can be drawn with.
pub const ENTITY_MATERIAL: &str = r#""entity_alphatest": {"vertexShader": "shaders/entity.vertex", "fragmentShader": "shaders/entity.fragment"}"#;
