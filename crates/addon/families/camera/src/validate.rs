//! The family rules of a loaded preset that vanilla's client logs when it sets the camera up.
//! The server loads such a preset silently, so each finding is raised at load and the preset is
//! kept with it.

use super::{ALLOWED_MC_INHERIT, CameraPreset, VANILLA_CAMERAS, texts};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use bedrock_addon_core::versioning::ValidateCx;

/// The findings of one preset.
pub(crate) fn preset(preset: &CameraPreset, cx: &mut ValidateCx<'_>) {
    let at = cx.at.child("minecraft:camera_preset");
    let name = preset.identifier.as_str();
    // A preset that is none of the client's cameras has nothing to start from unless it
    // inherits. The client logs the line as an error at camera setup and goes on with the next
    // preset: the preset is not set up, but the document stays loaded, so this is a kept
    // error.
    let vanilla_camera = ALLOWED_MC_INHERIT.contains(&name) || VANILLA_CAMERAS.contains(&name);
    let inherits = preset
        .inherit_from
        .as_ref()
        .is_some_and(|parent| !parent.as_str().is_empty());
    if !vanilla_camera && !inherits {
        let error = Diagnostic::error(DiagnosticCode::MissingRequiredField {
            text: texts::formatted(texts::MISSING_INHERIT, &[name]),
        })
        .at(at.child("inherit_from"))
        .phase(Phase::Validate);
        cx.diagnostics.push_kept_error(error);
    }
    // The client logs the line as an error when `min > max` (equal limits say nothing), skips
    // the limits and goes on with the preset: a kept error.
    if let (Some(min), Some(max)) = (preset.yaw_limits_min, preset.yaw_limits_max)
        && min.0 > max.0
    {
        let error = Diagnostic::error(DiagnosticCode::InvalidValue {
            text: texts::formatted(texts::YAW_ORDER, &[name]),
        })
        .at(at.child("yaw_limits_min"))
        .phase(Phase::Validate);
        cx.diagnostics.push_kept_error(error);
    }
    // A vanilla camera keeps its own starting rotation, and the client ignores the value of a
    // preset named like one. The game logs a line only for a camera entity with a third-person
    // boom, which none of the four vanilla documents has, so the game is silent where this
    // tells: the crate's own note is an information.
    if VANILLA_CAMERAS.contains(&name) && preset.starting_rot.is_some() {
        let info = Diagnostic::info(DiagnosticCode::InvalidValue {
            text: texts::formatted(texts::VANILLA_STARTING_ROT, &[name]),
        })
        .at(at.child("starting_rot"))
        .phase(Phase::Validate);
        cx.diagnostics.push(info);
    }
}
