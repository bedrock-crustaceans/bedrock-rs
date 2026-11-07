//! The rules on a loaded server sound definitions file.

use super::codes;
use super::{BROADCAST_RANGE_SINCE, SoundMetadataFile};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostic, Phase, Severity};
use bedrock_addon_core::versioning::ValidateCx;

/// The regex of the string constraint on a sound event name.
const NO_WHITESPACE_REGEX: &str = "^\\S+$";
/// The namespace no pack may define server sounds in, compared without case.
const DISALLOWED_NAMESPACE: &str = "minecraft";

/// Whether `text` matches `^\S+$`: not empty and without whitespace.
pub(crate) fn no_whitespace(text: &str) -> bool {
    !text.is_empty() && !text.chars().any(|c| c.is_whitespace() || c == '\u{feff}')
}

/// The namespace rule of the public area: `None` when `name` passes,
/// else the constraint's code.
// The game checks two case-insensitive patterns: the first forbids a `minecraft:` prefix, the
// second wants one `:` with text on both sides.
pub(crate) fn namespace_code(name: &str) -> Option<Code> {
    let bytes = name.as_bytes();
    // A `:` with at least one character on each side.
    let prefixed = bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| *byte == b':' && index >= 1 && index + 1 < bytes.len());
    if !prefixed {
        return Some(Code::from(&codes::NEEDS_NAMESPACE));
    }
    let disallowed = bedrock_addon_core::id::syntax::split_namespace(name)
        .is_some_and(|(namespace, _)| namespace.eq_ignore_ascii_case(DISALLOWED_NAMESPACE));
    disallowed.then(|| codes::INVALID_NAMESPACE.with([DISALLOWED_NAMESPACE]))
}

/// The rules on the loaded file. An error rejects it.
pub(crate) fn validate(file: &SoundMetadataFile, cx: &mut ValidateCx<'_>) {
    let base = cx.at.clone();
    let mut push = |code: Code, severity: Severity, index: usize, path: &[&str]| {
        let at = base.index(index).with(path);
        cx.diagnostics.push(
            Diagnostic::error(code)
                .at(at)
                .severity(severity)
                .phase(Phase::Validate),
        );
    };
    let pattern = || bedrock_addon_core::diagnostic::cereal::string_mismatch(NO_WHITESPACE_REGEX);
    for (index, definition) in file.0.iter().enumerate() {
        let name = definition.sound_event_name.as_str();
        if let Some(code) = namespace_code(name) {
            push(code, Severity::Error, index, &["sound_event_name"]);
        }
        if !no_whitespace(name) {
            push(pattern(), Severity::Error, index, &["sound_event_name"]);
        }
        for (key, value) in definition.tags.iter().flatten() {
            if !no_whitespace(key) || !value.iter().all(|value| no_whitespace(value)) {
                push(pattern(), Severity::Error, index, &["tags", key.as_str()]);
            }
        }
        if let Some(duration) = &definition.duration_info {
            let seconds = duration.seconds;
            if !seconds.is_finite() || seconds <= 0.0 {
                // The crate's own information: vanilla converts any value without a check.
                push(
                    Code::from(&core_codes::INVALID_VALUE),
                    Severity::Info,
                    index,
                    &["duration_info", "seconds"],
                );
            }
        }
        if let Some(range) = &definition.broadcast_range {
            if cx.load.engine_version < BROADCAST_RANGE_SINCE {
                // An engine below 1.26.50 does not bind the member.
                push(
                    Code::from(&core_codes::UNRECOGNIZED_FIELD),
                    Severity::Error,
                    index,
                    &["broadcast_range"],
                );
            } else if *range <= 0.0 {
                // The crate's own information: the game binds any number without a check.
                push(
                    Code::from(&core_codes::INVALID_VALUE),
                    Severity::Info,
                    index,
                    &["broadcast_range"],
                );
            }
        }
    }
}
