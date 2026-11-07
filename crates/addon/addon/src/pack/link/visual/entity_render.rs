//! The materials and the render controllers the description of a client entity or an atomic
//! client entity names, and the render-controller features an atomic client entity cannot
//! use.

use bedrock_addon_core::diagnostic::builders;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity, VanillaText};
use bedrock_addon_core::id::RenderControllerId;
use bedrock_addon_core::molang::{Molang, MolangSource};

use super::super::{At, Findings, LinkCx};
use super::user::{Family, User};
use super::{plain_reference, user_render};
use crate::families::render_controller::RenderController;

pub(super) fn run(cx: &LinkCx<'_>, out: &mut Findings, entities: &[User<'_>]) {
    for user in entities {
        user_render::materials(cx, out, user);
    }
    for user in entities {
        user_render::material_shaders(cx, out, user);
    }
    for user in entities {
        user_render::render_controllers(cx, out, user);
    }
    for user in entities {
        if user.family == Family::Atomic {
            atomic_controllers(cx, out, user);
        }
    }
}

/// Each render controller an atomic client entity names, judged by what the game
/// says of it when it draws the entity. A controller with a feature atomic entities do not
/// support is skipped by the game for this entity; of one it draws, `filter_lighting` is
/// ignored and a `light_color_multiplier` that is an expression counts as 1.
///
/// The three features of the game's text are read from the controller's members: an array,
/// a `geometry`, `textures` or `materials` value that is no plain `geometry.<name>` /
/// `texture.<name>` / `material.<name>`, or a `part_visibility` entry.
// The mapping of the three features to the controller's members is inferred from the
// game's text.
fn atomic_controllers(cx: &LinkCx<'_>, out: &mut Findings, user: &User<'_>) {
    let controllers = cx.resource().render_controllers();
    for (index, entry) in user.render_controllers.iter().enumerate() {
        let Some(name) = entry.controller() else {
            continue;
        };
        // A controller nothing defines is told by the render-controller reference rule.
        let Some(document) = controllers.get(&RenderControllerId::parse(name.as_str())) else {
            continue;
        };
        let controller = document.value();
        let at = || {
            let mut path = user.member("render_controllers");
            path.push_index(index);
            // Logged at the warning level, once when the entity is drawn: the unsupported
            // controller, `filter_lighting` and `light_color_multiplier`: a warning.
            At::new(user.origin, path, Severity::Warning)
        };
        if !atomic_compatible(controller) {
            out.push(at().diagnostic(DiagnosticCode::from(
                &core_codes::ATOMIC_UNSUPPORTED_RENDER_CONTROLLER,
            )));
            continue;
        }
        if controller.filter_lighting() {
            out.push(at().diagnostic(DiagnosticCode::from(
                &core_codes::ATOMIC_FILTER_LIGHTING_IGNORED,
            )));
        }
        if let Some(Molang::Expr(source)) = &controller.light_color_multiplier
            && !is_constant(source)
        {
            out.push(at().diagnostic(DiagnosticCode::from(builders::worded(
                &core_codes::NO_EFFECT,
                &core_codes::NO_EFFECT_WORDED,
                Some(VanillaText::AtomicLightColorExpression).as_ref(),
            ))));
        }
    }
}

/// Whether `controller` uses none of the features an atomic client entity cannot.
fn atomic_compatible(controller: &RenderController) -> bool {
    let no_arrays = controller.arrays.as_ref().is_none_or(|arrays| {
        arrays.geometries.is_empty() && arrays.materials.is_empty() && arrays.textures.is_empty()
    });
    let plain = |source: &MolangSource, prefix| plain_reference(source.as_str(), prefix).is_some();
    no_arrays
        && controller.part_visibility.is_empty()
        && plain(&controller.geometry, "geometry.")
        && controller
            .textures
            .iter()
            .all(|texture| plain(texture, "texture."))
        && controller
            .materials
            .iter()
            .all(|(_, material)| plain(material, "material."))
}

/// Whether the expression `source` is a number: its text is one, or it compiles to one.
fn is_constant(source: &MolangSource) -> bool {
    if source.as_str().trim().parse::<f32>().is_ok() {
        return true;
    }
    folds_to_a_constant(source)
}

#[cfg(feature = "molang")]
fn folds_to_a_constant(source: &MolangSource) -> bool {
    use bedrock_addon_core::molang::{CheckOptions, CompileOptions, Side, check};
    let catalog = bedrock_addon_core::molang::molang::stdlib::queries(Side::Client).clone();
    let Some(version) = source.raw_version() else {
        return false;
    };
    // A rejection has no expression, nor does an expression that uses arrays or resources.
    let options = CheckOptions::new(CompileOptions::from_raw_version(catalog, version));
    check(source, &options)
        .expr()
        .is_some_and(|expression| expression.as_constant().is_some())
}

#[cfg(not(feature = "molang"))]
fn folds_to_a_constant(_: &MolangSource) -> bool {
    false
}
