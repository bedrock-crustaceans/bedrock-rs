//! Geometry's schema walk on core's node-tree walker (`json::node_tree`): vanilla validates these
//! files with JSON schema node trees, leniently below 1.19.30 (an undeclared or wrong-typed member
//! is removed and the file loads) and strictly from 1.19.30 (it fails the file). This module holds
//! the rules of that walk, the legacy geometry-name pattern and the leaves only geometry reads.

use bedrock_addon_core::diagnostic::Severity;
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::json::node_tree::{
    Failure, Lossy, Names, Node, Order, Pattern, Rules, Validation, Walk,
};

use crate::codes;

/// The crate's own text for a member below 1.19.30 that vanilla removes without a message: the
/// lenient walk of core's node tree takes it as its text and raises it.
const CHILD_REMOVED: &str = "child '{}' not valid here (removed; format_version < 1.19.30)";

/// The rules of the game's geometry schemas: names as written, the members in the file's order
/// (the loader runs `json::last_wins` first), lenient (`lenient`, below 1.19.30) or strict, an
/// object failed only by an error raised in it, and a lossy float told.
pub(crate) const fn rules(lenient: bool) -> Rules {
    Rules {
        names: Names::Exact,
        order: Order::Document,
        validation: if lenient {
            Validation::Lenient {
                removed: CHILD_REMOVED,
            }
        } else {
            Validation::Strict
        },
        failure: Failure::ErrorRaised,
        lossy: Lossy::Told,
    }
}

/// `geometry\.[a-zA-Z0-9_.'-:]+`, searched in the key (the legacy geometry members).
pub(crate) const GEOMETRY: Pattern = Pattern::Custom {
    matches: has_geometry_name,
    regex: r"geometry\.[a-zA-Z0-9_.'-:]+",
};

/// A `std::regex_search` for `geometry\.[a-zA-Z0-9_.'-:]+`: the text holds `geometry.` followed
/// by a letter, a digit, `_`, `.` or a character of the range `'` to `:` (which also admits
/// `( ) * + , - /`).
pub(crate) fn has_geometry_name(text: &str) -> bool {
    const PREFIX: &str = "geometry.";
    text.match_indices(PREFIX).any(|(at, _)| {
        text.as_bytes()
            .get(at + PREFIX.len())
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_' || (b'\''..=b':').contains(b))
    })
}

/// The leaves only geometry's schemas have.
pub(crate) trait Leaves {
    /// An int node: a number without a fraction that fits an `i32`.
    fn int(&mut self, node: Node<'_>) -> Option<i32>;
    /// An array of exactly `N` floats (`addChildArray(name, N, …)`): a wrong length is "Array
    /// too small/large" and fails the file in either mode.
    fn floats<const N: usize>(&mut self, node: Node<'_>) -> Option<[f32; N]>;
    /// A box `uv`: two floats, of which vanilla keeps the whole part (`Json::Value::asInt`); a
    /// fraction is told where it stands, an `InvalidValue` info as the clamp of the bounds is.
    fn texels(&mut self, node: Node<'_>) -> Option<[f32; 2]>;
    /// The three indices of a `poly_mesh` vertex: floats to the schema, read the way the game's
    /// unsigned read takes them. An index that is not written as it was given is told at the
    /// element with an `InvalidValue` information, whether a fraction goes (the game's reading)
    /// or the value is below zero or beyond 32 bits (the bound is the crate's).
    fn indices(&mut self, node: Node<'_>) -> Option<[u32; 3]>;
    /// An array of floats of any length (the locator vectors, which vanilla declares unsized).
    fn float_list(&mut self, node: Node<'_>) -> Option<Vec<f32>>;
}

impl Leaves for Walk {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the range is checked first"
    )]
    fn int(&mut self, node: Node<'_>) -> Option<i32> {
        let integral = node.number().filter(|x| {
            x.fract() == 0.0 && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(x)
        });
        if integral.is_none() {
            self.bad_type("int", node);
        }
        integral.map(|x| x as i32)
    }

    fn floats<const N: usize>(&mut self, node: Node<'_>) -> Option<[f32; N]> {
        let values = self.float_list(node)?;
        if !sized(self, values.len(), N, node) {
            return None;
        }
        values.try_into().ok()
    }

    fn texels(&mut self, node: Node<'_>) -> Option<[f32; 2]> {
        let uv = self.floats::<2>(node)?;
        for (index, (element, value)) in self
            .elements(node)
            .unwrap_or_default()
            .into_iter()
            .zip(uv)
            .enumerate()
        {
            if value.fract() != 0.0 {
                self.enter_index(index);
                self.push(
                    codes::UV_TRUNCATED.with([value.to_string(), value.trunc().to_string()]),
                    Severity::Info,
                    element.span(),
                );
                self.leave();
            }
        }
        Some(uv)
    }

    fn indices(&mut self, node: Node<'_>) -> Option<[u32; 3]> {
        // Read as `f64`, not as the floats of the schema: an index is not rounded to an `f32`,
        // and a number beyond `f32` is told once, by the index.
        let Some(elements) = self.elements(node) else {
            self.bad_type("array", node);
            return None;
        };
        let mut read = [0_u32; 3];
        let mut givens = Vec::with_capacity(elements.len());
        for (index, element) in elements.iter().enumerate() {
            self.enter_index(index);
            let given = self.number(*element);
            self.leave();
            givens.push(given?);
        }
        if !sized(self, givens.len(), 3, node) {
            return None;
        }
        for (index, ((element, slot), given)) in elements
            .into_iter()
            .zip(read.iter_mut())
            .zip(givens)
            .enumerate()
        {
            // What truncates to an index of 32 bits; beyond it the cast's bound is the crate's own.
            let in_range = given > -1.0 && given < 4_294_967_296.0;
            let value = as_index(given);
            *slot = value;
            if in_range && given.fract() == 0.0 {
                continue;
            }
            // The crate's own information either way.
            self.enter_index(index);
            self.push(
                codes::INDEX_NOT_WHOLE.with([given.to_string(), value.to_string()]),
                Severity::Info,
                element.span(),
            );
            self.leave();
        }
        Some(read)
    }

    fn float_list(&mut self, node: Node<'_>) -> Option<Vec<f32>> {
        let Some(elements) = self.elements(node) else {
            self.bad_type("array", node);
            return None;
        };
        let mut values = Vec::with_capacity(elements.len());
        for (index, element) in elements.into_iter().enumerate() {
            self.enter_index(index);
            let value = self.float(element);
            self.leave();
            values.push(value?);
        }
        Some(values)
    }
}

/// Whether an array of `n` elements is the `want` the schema declares; if not, the size error
/// at the array.
fn sized(walk: &mut Walk, n: usize, want: usize, node: Node<'_>) -> bool {
    if n == want {
        return true;
    }
    let def = if n < want {
        &core_codes::ARRAY_TOO_SMALL
    } else {
        &core_codes::ARRAY_TOO_LARGE
    };
    walk.invalid(def.with([n.to_string(), want.to_string()]), node);
    false
}

/// `Json::Value::asUInt` of an index the schema declares as a float.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a float to integer cast saturates, as the index read does not need more"
)]
fn as_index(x: f64) -> u32 {
    x as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::JsonPath;
    use bedrock_addon_core::json::node_tree::{Group, Hit};

    #[test]
    fn the_geometry_pattern_is_searched_and_its_range_runs_from_quote_to_colon() {
        assert!(has_geometry_name("geometry.pig"));
        assert!(has_geometry_name("foo.geometry.bar"));
        assert!(has_geometry_name("geometry.a:geometry.b"));
        assert!(has_geometry_name("geometry.'"));
        assert!(has_geometry_name("geometry.:"));
        assert!(has_geometry_name("geometry.(x)"));
        assert!(!has_geometry_name("geometry."));
        assert!(!has_geometry_name("geometry.&"));
        assert!(!has_geometry_name("geometry.;"));
        assert!(!has_geometry_name("geometry pig"));
        assert!(!has_geometry_name("Geometry.pig"));
        assert!(GEOMETRY.matches("x.geometry.pig", Names::Exact));
    }

    /// The keys `object` hands its callback, in the order it hands them, and the findings.
    fn walked(text: &str, groups: &[Group], read: Hit) -> (bool, Vec<String>, Vec<String>) {
        let mut walk = Walk::new(rules(false), &JsonPath::new());
        let mut keys = Vec::new();
        let ok = walk.object(Node::root(text).unwrap(), groups, |_, _, key, _| {
            keys.push(key.to_owned());
            read
        });
        let findings = walk
            .diagnostics
            .iter()
            .map(|d| format!("{:?} {} at {}", d.severity, d.code, d.location.json_path))
            .collect();
        (ok, keys, findings)
    }

    /// W1: the members of an object are walked in the file's order.
    #[test]
    fn members_are_walked_in_document_order() {
        let groups = [Group::name("a"), Group::name("b"), Group::name("c")];
        let (ok, keys, _) = walked(r#"{"b": 1, "a": 2, "c": 3}"#, &groups, Hit::Ok);
        assert!(ok);
        assert_eq!(keys, ["b", "a", "c"]);
    }

    /// W2: a member whose read fails without an error fails nothing: the object is judged by
    /// the errors raised, and the required check is made.
    #[test]
    fn a_member_failed_without_a_finding_fails_nothing_and_the_required_check_is_made() {
        let (ok, keys, findings) = walked(r#"{"a": 1}"#, &[Group::name("a")], Hit::Failed);
        assert!(ok);
        assert_eq!(keys, ["a"]);
        assert!(findings.is_empty(), "{findings:?}");
        let groups = [Group::name("a"), Group::required("b")];
        let (ok, _, findings) = walked(r#"{"a": 1}"#, &groups, Hit::Failed);
        assert!(!ok);
        assert_eq!(findings, ["Error Required child b not found at "]);
    }

    /// W3: a number a float does not hold is read with a `LossyConversion` information.
    #[test]
    fn a_float_that_narrows_is_told_as_lossy() {
        let mut walk = Walk::new(rules(false), &JsonPath::new());
        walk.enter("x");
        let node = Node::root("0.123456789012").unwrap();
        assert_eq!(walk.float(node), Some(0.123_456_79));
        let findings: Vec<_> = walk
            .diagnostics
            .iter()
            .map(|d| {
                format!(
                    "{:?} {} at {}",
                    d.severity,
                    d.code.name(),
                    d.location.json_path
                )
            })
            .collect();
        assert_eq!(findings, ["Info LossyConversion at x"]);
    }
}
