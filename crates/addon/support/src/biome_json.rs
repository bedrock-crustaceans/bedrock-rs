//! What the biome families share: the path of a member below another, and the colour pattern.

use bedrock_addon_core::diagnostic::JsonPath;

/// The path `at` continued by `steps`; a step of digits is an array index (no biome member is
/// named by digits), any other a member. Unlike [`JsonPath::with`], which takes every key as
/// written: a caller that knows a step is an element names it with [`JsonPath::index`].
pub fn child(at: &JsonPath, steps: &[&str]) -> JsonPath {
    steps
        .iter()
        .fold(at.clone(), |path, step| match array_index(step) {
            Some(index) => path.index(index),
            None => path.child(step),
        })
}

/// The array index a step of ASCII digits names, `None` for any other step (and for digits
/// past `usize`).
fn array_index(step: &str) -> Option<usize> {
    step.bytes()
        .all(|b| b.is_ascii_digit())
        .then(|| step.parse().ok())
        .flatten()
}

/// `^#[a-fA-F0-9]{digits}$`.
// The game's colour pattern for biome colour members.
pub fn is_hex(text: &str, digits: usize) -> bool {
    text.strip_prefix('#')
        .is_some_and(|rest| rest.len() == digits && rest.bytes().all(|b| b.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The path of `keys` written member by member, for a golden value.
    fn members(keys: &[&str]) -> JsonPath {
        let mut path = JsonPath::new();
        for key in keys {
            path.push(key);
        }
        path
    }

    #[test]
    fn a_step_of_digits_is_an_array_index() {
        let at = members(&["minecraft:client_biome"]);
        let mut expected = members(&["minecraft:client_biome", "fog_ids_to_merge"]);
        expected.push_index(2);
        assert_eq!(child(&at, &["fog_ids_to_merge", "2"]), expected);
        // Whatever holds it: a biome named by digits under `biomes` is an index too.
        let mut biome = members(&["biomes"]);
        biome.push_index(123);
        assert_eq!(child(&JsonPath::new(), &["biomes", "123"]), biome);
        // Leading zeros are digits all the same.
        let mut zero = members(&["a"]);
        zero.push_index(1);
        assert_eq!(child(&JsonPath::new(), &["a", "01"]), zero);
    }

    #[test]
    fn any_other_step_is_a_member() {
        for step in ["", "+1", "-1", "1.0", "1e2", "x1", " 1", "١"] {
            assert_eq!(
                child(&JsonPath::new(), &["a", step]),
                members(&["a", step]),
                "{step:?}"
            );
        }
        // Digits past `usize` do not parse as an index: a member.
        let huge = "9".repeat(40);
        assert_eq!(
            child(&JsonPath::new(), &[huge.as_str()]),
            members(&[huge.as_str()])
        );
    }
}
