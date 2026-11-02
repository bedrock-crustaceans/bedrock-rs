//! What the biome families share: the path of a member below another, and the colour pattern.

use bedrock_addon_core::diagnostic::JsonPath;

/// The path `at` continued by `steps`; a step of digits is an array index (no biome member is
/// named by digits).
#[must_use]
pub fn child(at: &JsonPath, steps: &[&str]) -> JsonPath {
    let mut path = at.clone();
    for step in steps {
        match step.parse::<usize>() {
            Ok(index) if step.bytes().all(|b| b.is_ascii_digit()) => path.push_index(index),
            _ => path.push(step),
        }
    }
    path
}

/// `^#[a-fA-F0-9]{digits}$`.
// The game's colour pattern for biome colour members.
#[must_use]
pub fn is_hex(text: &str, digits: usize) -> bool {
    text.strip_prefix('#')
        .is_some_and(|rest| rest.len() == digits && rest.bytes().all(|b| b.is_ascii_hexdigit()))
}
