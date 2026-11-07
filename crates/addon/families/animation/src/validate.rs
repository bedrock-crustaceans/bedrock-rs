//! The findings that are not schema failures: the crate's own lints, where the game says nothing.

use super::{AnimationFile, OneOrMany};
use bedrock_addon_core::diagnostic::DiagnosticCode;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Severity};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_shared_animation::validate::{command_version_unset, events, push};

/// The crate's own findings on a loaded animation file: bones that collide after lower-casing
/// (`DuplicateBone`), unrecognized `@` event strings, and events in a pack without
/// `min_engine_version`.
pub(crate) fn animation_findings(
    file: &AnimationFile,
    cx: &LoadContext,
    diagnostics: &mut Diagnostics,
) {
    let mut any_event = false;
    for (key, animation) in &file.animations {
        let base = JsonPath::new().child("animations").child(key);
        if let Some(bones) = &animation.bones {
            let mut seen = Vec::new();
            for name in bones.keys() {
                let lower = name.to_ascii_lowercase();
                if seen.contains(&lower) {
                    let at = base.child("bones").child(name);
                    push(
                        diagnostics,
                        DiagnosticCode::from(&core_codes::DUPLICATE_BONE),
                        Severity::Info,
                        at,
                    );
                } else {
                    seen.push(lower);
                }
            }
        }
        if let Some(timeline) = &animation.timeline {
            let mut list = Vec::new();
            for (time, entries) in timeline {
                let at = base.child("timeline").child(time);
                match entries {
                    OneOrMany::One(e) => list.push((at, e)),
                    OneOrMany::Many(many) => {
                        for (index, e) in many.iter().enumerate() {
                            list.push((at.index(index), e));
                        }
                    }
                }
            }
            any_event |= events(list, diagnostics);
        }
    }
    command_version_unset(cx, any_event, diagnostics);
}
