//! What both animation loaders read their schema trees with on the derived read: the reading and
//! the profile of the game's node tree for them (member names that match ignoring ASCII case, in
//! the file's order), the key patterns only these trees have (time keys, axes), and vanilla's
//! `atof` the keys are read with.

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::json::kit::{Names, Profile, Read, Texts};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::schema::{KeyPattern, Keys};
use bedrock_addon_core::versioning::Reading;

/// The profile of those schemas: the node tree's, names and prefixes compared without regard to
/// ASCII case, the members in the file's order, a lossy float read silently.
pub const PROFILE: Profile = Profile::TREE.with_names(Names::IgnoreAsciiCase);

/// The read of a file of a pack with context `cx`, telling into `findings`.
pub fn read<'d>(cx: &'d LoadContext, findings: &'d mut Diagnostics) -> Read<'d> {
    Read::new(Reading::TREE, findings)
        .profile(PROFILE)
        .texts(Texts::TREE)
        .load(cx)
}

/// `[0-9]+(\.[0-9]+)?`: a time key.
pub const TIME: KeyPattern = KeyPattern::Custom {
    matches: is_time_key,
    regex: r"[0-9]+(\.[0-9]+)?",
};

/// Members by their time key, none required.
pub const TIMES: Keys = Keys::new(TIME);

/// `[xyz]`, in either case.
pub const AXIS: KeyPattern = KeyPattern::Custom {
    matches: is_axis,
    regex: "[xyz]",
};

/// Members by an identifier (`[a-zA-Z0-9_.-]+`), at least one.
pub const IDENTIFIERS: Keys = Keys::new(KeyPattern::IDENTIFIER).required();

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
    use bedrock_addon_core::json::kit::Node;
    use bedrock_addon_core::json::parse::{Options, parse};
    use bedrock_addon_core::schema::{FromJson, read_document};
    use facet::Facet;

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

    /// Three members, each read where the file has it.
    #[derive(Debug, PartialEq, Facet, bedrock_addon_core::schema::Schema)]
    struct Three {
        a: Option<f32>,
        b: Option<f32>,
        c: Option<f32>,
    }

    /// The value and the findings of `text` read as `T` under the schemas' read.
    fn read_as<T: FromJson>(text: &str) -> (Option<T>, Vec<String>) {
        let parsed = parse(text, Options::default()).unwrap();
        let cx = LoadContext::detached();
        let mut findings = Diagnostics::default();
        let mut reader = read(&cx, &mut findings);
        let value = read_document::<T>(&Node::parsed(&parsed, JsonPath::new()), &mut reader);
        drop(reader);
        let told = findings
            .iter()
            .map(|d| format!("{:?} {} at {}", d.severity, d.code, d.location.json_path))
            .collect();
        (value, told)
    }

    /// W1: the members of an object are read in the file's order.
    #[test]
    fn members_are_read_in_document_order() {
        let (value, told) = read_as::<Three>(r#"{"c": true, "a": "x", "b": 1}"#);
        assert_eq!(value, None);
        let paths: Vec<&str> = told.iter().map(|t| t.rsplit(' ').next().unwrap()).collect();
        assert_eq!(paths, ["c", "a"]);
    }

    /// Two members, the second required.
    #[derive(Debug, PartialEq, Facet, bedrock_addon_core::schema::Schema)]
    struct Two {
        a: Option<bool>,
        b: bool,
    }

    /// W2: a member whose read fails fails the object, and the required check is then not made.
    #[test]
    fn a_failed_member_fails_the_object_and_skips_the_required_check() {
        let (value, told) = read_as::<Two>(r#"{"a": 1}"#);
        assert_eq!(value, None);
        let paths: Vec<&str> = told.iter().map(|t| t.rsplit(' ').next().unwrap()).collect();
        assert_eq!(paths, ["a"], "{told:?}");
    }

    /// W3: a number a float does not hold is read without a `LossyConversion` information.
    #[test]
    fn a_float_that_narrows_is_not_told_as_lossy() {
        let (value, told) = read_as::<Three>(r#"{"a": 0.123456789012}"#);
        assert_eq!(value.and_then(|t| t.a), Some(0.123_456_79));
        assert!(told.is_empty(), "{told:?}");
    }
}
