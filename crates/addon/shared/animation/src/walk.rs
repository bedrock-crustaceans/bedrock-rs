//! What both animation loaders read their schema trees with on core's node-tree walker
//! (`json::node_tree`): the rules of the walk (member names that match case-insensitively and in
//! full, in the file's order), the key patterns only these trees have (time keys, axes), and
//! vanilla's `atof` the keys are read with.

use bedrock_addon_core::json::node_tree::{
    Failure, Lossy, Names, Order, Pattern, Rules, Validation,
};

/// The rules of the game's animation and animation controller schemas: names and prefixes
/// compared without regard to ASCII case, the members in the file's order, strict, an object
/// failed by any member, and a lossy float read silently.
pub const RULES: Rules = Rules {
    names: Names::IgnoreAsciiCase,
    order: Order::Document,
    validation: Validation::Strict,
    failure: Failure::MemberFailed,
    lossy: Lossy::Silent,
};

/// `[0-9]+(\.[0-9]+)?`: a time key.
pub const TIME: Pattern = Pattern::Custom {
    matches: is_time_key,
    regex: r"[0-9]+(\.[0-9]+)?",
};

/// `[xyz]`, in either case.
pub const AXIS: Pattern = Pattern::Custom {
    matches: is_axis,
    regex: "[xyz]",
};

/// `[0-9]+(\.[0-9]+)?`, in full.
pub(crate) fn is_time_key(key: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    match key.split_once('.') {
        Some((whole, fraction)) => digits(whole) && digits(fraction),
        None => digits(key),
    }
}

/// `[xyz]`, in either case.
fn is_axis(key: &str) -> bool {
    matches!(key, "x" | "y" | "z" | "X" | "Y" | "Z")
}

/// vanilla's `atof` of a key that matched `is_time_key` (or any key: `atof` reads the longest
/// numeric prefix and gives 0 for none).
pub fn atof(key: &str) -> f32 {
    let end = key
        .char_indices()
        .take_while(|(i, c)| c.is_ascii_digit() || *c == '.' || (*i == 0 && matches!(c, '-' | '+')))
        .last()
        .map_or(0, |(i, c)| i + c.len_utf8());
    let mut prefix = &key[..end];
    // `atof("1.")` is 1; Rust's parser wants digits after the point.
    while !prefix.is_empty() && prefix.parse::<f32>().is_err() {
        prefix = &prefix[..prefix.len() - 1];
    }
    prefix.parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::JsonPath;
    use bedrock_addon_core::json::node_tree::{Group, Hit, Node, Walk};

    #[test]
    fn time_keys_match_in_full() {
        for key in ["0", "0.5", "12.25"] {
            assert!(is_time_key(key), "{key}");
            assert!(TIME.matches(key, Names::IgnoreAsciiCase), "{key}");
        }
        for key in [".5", "-1", "1e2", "0.5 ", "1.", ""] {
            assert!(!is_time_key(key), "{key}");
        }
    }

    #[test]
    fn axes_match_in_either_case() {
        for key in ["x", "y", "z", "X", "Y", "Z"] {
            assert!(AXIS.matches(key, Names::IgnoreAsciiCase), "{key}");
        }
        assert!(!AXIS.matches("w", Names::IgnoreAsciiCase));
        assert!(!AXIS.matches("xy", Names::IgnoreAsciiCase));
    }

    #[test]
    fn atof_reads_the_numeric_prefix() {
        assert!((atof("0.25") - 0.25).abs() < f32::EPSILON);
        assert!(atof("x").abs() < f32::EPSILON);
        assert!((atof("1.") - 1.0).abs() < f32::EPSILON);
    }

    /// The keys `object` hands its callback, in the order it hands them, and the findings.
    fn walked(text: &str, groups: &[Group], read: Hit) -> (bool, Vec<String>, Vec<String>) {
        let mut walk = Walk::new(RULES, &JsonPath::new());
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

    /// W2: a member whose read fails fails the object, though it raised no finding, and the
    /// required check is then not made.
    #[test]
    fn a_member_failed_without_a_finding_fails_the_object_and_skips_the_required_check() {
        let groups = [Group::name("a"), Group::required("b")];
        let (ok, keys, findings) = walked(r#"{"a": 1}"#, &groups, Hit::Failed);
        assert!(!ok);
        assert_eq!(keys, ["a"]);
        assert!(findings.is_empty(), "{findings:?}");
    }

    /// W3: a number a float does not hold is read without a `LossyConversion` information.
    #[test]
    fn a_float_that_narrows_is_not_told_as_lossy() {
        let mut walk = Walk::new(RULES, &JsonPath::new());
        walk.enter("x");
        assert_eq!(
            walk.float(Node::root("0.123456789012").unwrap()),
            Some(0.123_456_79)
        );
        assert!(walk.diagnostics.is_empty());
    }
}
