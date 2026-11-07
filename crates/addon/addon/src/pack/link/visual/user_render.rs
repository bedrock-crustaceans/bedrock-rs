//! `description.materials` and `description.render_controllers` of a client entity, an
//! attachable or an atomic client entity: each names a material or a render controller.

use bedrock_addon_core::diagnostic::{Severity, sentence};
use bedrock_addon_core::id::{MaterialId, RenderControllerId};
use bedrock_addon_shared_visual::ControllerUse;

use super::super::{At, Findings, LinkCx, Target, invalid, kinds, unresolved};
use super::user::{Family, User};

/// Vanilla's text for a material no group has: the name in lower case, then the file.
const MATERIAL_NOT_DEFINED: &str = "Material \"{}\" is referenced in file \"{}\" but not defined.";

/// Whether a material named `name` exists for a description: the material itself or its
/// skinning variant (the game looks up `<name>.skinning` and `<name>.skinning_color`, each
/// falling back to `<name>`, and complains only when both the variant and the name are
/// missing). The stack's three groups answer, then the host
/// ([`LinkCx::material_exists`]).
///
/// Costs the length of the name: the place of the material is not asked for.
pub(super) fn material_exists(cx: &LinkCx<'_>, name: &str) -> bool {
    cx.material_exists(&MaterialId::parse(name))
        || cx.material_exists(&MaterialId::parse(format!("{name}.skinning")))
}

/// `materials`: each value names a material. Nothing is judged in a stack
/// without a view of the materials, and a miss is an information with a partial one
/// ([`LinkCx::material_view`]).
pub(super) fn materials(cx: &LinkCx<'_>, out: &mut Findings, user: &User<'_>) {
    let lint = user.family == Family::Atomic;
    if lint && !cx.lints() {
        return;
    }
    if user.materials.is_none_or(indexmap::IndexMap::is_empty) {
        return;
    }
    let view = cx.material_view(out);
    if !view.judges() || (view == super::super::View::Partial && !cx.lints()) {
        return;
    }
    for (key, value) in user.materials.into_iter().flatten() {
        let name = user.folded(value.as_str());
        if material_exists(cx, &name) {
            continue;
        }
        let path = user.entry("materials", key);
        let at = if lint {
            // The atomic loader checks nothing when it loads: the crate's own check, an
            // information.
            At::new(user.origin, path, Severity::Info)
        } else {
            // The game logs the miss at the error level, skips the entry and goes on with the
            // file: a kept error, an information with a partial view.
            At::new(user.origin, path, view.severity(Severity::Error)).with_text(sentence(
                MATERIAL_NOT_DEFINED,
                &[name.to_string(), user.origin.path.to_string()],
            ))
        };
        let finding = unresolved(at, kinds::MATERIAL, &name, None);
        out.push(if finding.severity == Severity::Error {
            finding.into_kept()
        } else {
            finding
        });
    }
}

/// `render_controllers`: each entry names a render controller.
pub(super) fn render_controllers(cx: &LinkCx<'_>, out: &mut Findings, user: &User<'_>) {
    if !cx.lints() {
        return;
    }
    for (index, entry) in user.render_controllers.iter().enumerate() {
        // An empty object names nothing.
        let Some(written) = entry.controller() else {
            continue;
        };
        let name = user.folded(written.as_str());
        let Target::Missing(hint) = cx.render_controller(&RenderControllerId::parse(&*name)) else {
            continue;
        };
        let mut path = user.member("render_controllers");
        path.push_index(index);
        if matches!(entry, ControllerUse::Conditional(_)) {
            // The object form: the member that names the controller, its key as written.
            path.push(written.as_str());
        }
        let at = At::new(user.origin, path, Severity::Info);
        out.push(unresolved(at, kinds::RENDER_CONTROLLER, &name, hint));
    }
}

/// Vanilla's text for a material an actor cannot be drawn with: the material's vertex shader.
const SHADER_NOT_FOR_ACTORS: &str = "Material shader \"{}\" cannot be assigned to an entity";

/// The material `name` of the stack with everything it inherits applied: the common group's,
/// else the fancy group's, else the sad group's; for the name of a variant
/// (`<material>.<variant>`) its material.
pub(super) fn resolved<'c>(
    cx: &'c LinkCx<'_>,
    name: &str,
) -> Option<&'c crate::families::material::ResolvedMaterial> {
    use crate::families::material::MaterialGroupKind;
    MaterialGroupKind::ALL.into_iter().find_map(|kind| {
        let group = cx.resolved_materials(kind);
        std::iter::once(name)
            .chain(name.rmatch_indices('.').map(|(at, _)| &name[..at]))
            .find_map(|name| group.get(name))
    })
}

/// `materials` of a client entity or an attachable: a material that exists must be one an
/// actor can be drawn with, its vertex shader one of the actor shaders. The game
/// finds out when it first draws the actor and falls back to the default variant; this crate
/// tells it where the material is named.
pub(super) fn material_shaders(cx: &LinkCx<'_>, out: &mut Findings, user: &User<'_>) {
    use crate::families::material::ACTOR_SHADER_STEMS;
    if user.family == Family::Atomic {
        return;
    }
    for (key, value) in user.materials.into_iter().flatten() {
        let name = user.folded(value.as_str());
        let Some(material) = resolved(cx, &name) else {
            continue;
        };
        // A chain that names no vertex shader has none that could be the wrong one.
        if material.vertex_shader.is_empty()
            || ACTOR_SHADER_STEMS.contains(&material.vertex_shader_stem())
        {
            continue;
        }
        // The game logs the shader at the error level when the entity is drawn and falls back
        // to the `Actor` variant: a kept error.
        let at = At::new(user.origin, user.entry("materials", key), Severity::Error).with_text(
            sentence(SHADER_NOT_FOR_ACTORS, &[material.vertex_shader.as_str()]),
        );
        out.push(invalid(at, kinds::MATERIAL, &name).into_kept());
    }
}
