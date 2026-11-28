//! A map whose member names follow a pattern (`animation.<name>`, a bone name, a time key):
//! [`Keys`], the rule a map field states with `schema::keys`, and [`read_keyed`], its read.
//!
//! The game's schema node trees name the members of an object by name groups, a literal name or
//! a regex: a struct's fields are the literal ones, and a group of a pattern is a map of the
//! names it takes. A name no pattern takes is an unknown member of the map, and a map that must
//! have one tells it missing when none is read.

use indexmap::IndexMap;

use crate::json::kit::{Names, Node, Objects, Read};
use crate::schema::UnknownRule;

/// What member names a map takes, matched in full unless it says otherwise, a prefix as the
/// read's [`Names`] compares it.
///
/// Two patterns are equal when they are one pattern; a [`Custom`](Self::Custom) one compares
/// its function by address. Rust does not promise one address per function, so that equality is
/// best-effort and serves only the hashability bound; no read decides anything by it.
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "a `Custom` pattern is a family's `const`, compared with itself"
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KeyPattern {
    /// `<prefix>` then one or more of `[a-zA-Z0-9]` and `extra`.
    Prefixed {
        /// The prefix, unescaped (`controller.render.`).
        prefix: &'static str,
        /// The bytes the tail takes beside letters and digits.
        extra: &'static [u8],
        /// The regex as vanilla prints it.
        regex: &'static str,
    },
    /// One or more of `[a-zA-Z0-9]` and `extra`.
    Charset {
        /// The bytes the name takes beside letters and digits.
        extra: &'static [u8],
        /// The regex as vanilla prints it.
        regex: &'static str,
    },
    /// A family's own test of a name (a regex search, a time key), whatever the read's
    /// [`Names`].
    Custom {
        /// Whether a name is one the pattern accepts.
        matches: fn(&str) -> bool,
        /// The regex as vanilla prints it.
        regex: &'static str,
    },
}

impl KeyPattern {
    /// `[a-zA-Z0-9_.-]+`.
    pub const IDENTIFIER: Self = Self::Charset {
        extra: b"_.-",
        regex: "[a-zA-Z0-9_.-]+",
    };

    /// Whether `key` is a name this pattern accepts, a prefix compared as `names` says.
    #[must_use]
    pub fn matches(self, key: &str, names: Names) -> bool {
        match (self, names) {
            (Self::Prefixed { prefix, extra, .. }, Names::IgnoreAsciiCase) => {
                key.get(..prefix.len())
                    .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
                    && charset(&key[prefix.len()..], extra)
            }
            (Self::Prefixed { prefix, extra, .. }, _) => key
                .strip_prefix(prefix)
                .is_some_and(|tail| charset(tail, extra)),
            (Self::Charset { extra, .. }, _) => charset(key, extra),
            (Self::Custom { matches, .. }, _) => matches(key),
        }
    }

    /// The pattern as vanilla prints it in "Required child %s not found".
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Self::Prefixed { regex, .. }
            | Self::Charset { regex, .. }
            | Self::Custom { regex, .. } => regex,
        }
    }

    /// Whether `key`, which the pattern takes, spells its prefix in another case.
    fn respelled(self, key: &str) -> bool {
        matches!(self, Self::Prefixed { prefix, .. } if !key.starts_with(prefix))
    }
}

/// `[a-zA-Z0-9<extra>]+`, in full.
fn charset(key: &str, extra: &[u8]) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || extra.contains(&b))
}

/// The rule of a map's member names (`schema::keys`): the pattern they follow, and whether the
/// map must have one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Keys {
    /// The names the map takes.
    pub pattern: KeyPattern,
    /// The map must hold a member: one read is "Required child `<pattern>` not found".
    pub required: bool,
}

impl Keys {
    /// The names `pattern` takes, none required.
    #[must_use]
    pub const fn new(pattern: KeyPattern) -> Self {
        Self {
            pattern,
            required: false,
        }
    }

    /// The rule with at least one member required.
    #[must_use]
    pub const fn required(self) -> Self {
        Self {
            required: true,
            ..self
        }
    }
}

/// The members of the object `node` whose names `keys` takes, each value read by `value` at its
/// member's path, in the read's order ([`Read::ordered`]): the read of a `schema::keys` map. A
/// name the pattern does not take is an unknown member (by the word `report`: told, not kept);
/// a prefix spelled in another case under [`Names::IgnoreAsciiCase`] is noted
/// ([`Read::take_respelled`]). A failed value fails the map (every value is still read and
/// told); under the game's schema tree ([`Objects::Tree`]) one that failed without an error is
/// left out, and an error raised while the members were read fails the map. A map that must
/// have a member and read none (that did not fail) tells it missing, the pattern's text its
/// name.
///
/// ```
/// use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath};
/// use bedrock_addon_core::json::kit::{Node, Read};
/// use bedrock_addon_core::json::{Value, from_str_jsonc};
/// use bedrock_addon_core::schema::{FromJson, KeyPattern, Keys, read_keyed};
/// use bedrock_addon_core::versioning::Reading;
///
/// const BONES: Keys = Keys::new(KeyPattern::IDENTIFIER).required();
/// let json: Value = from_str_jsonc(r#"{"arm": 1, "left leg": 2}"#).unwrap();
/// let mut found = Diagnostics::default();
/// let mut read = Read::new(Reading::DOCUMENT, &mut found);
/// let bones = read_keyed(&Node::new(&json, JsonPath::new()), &mut read, &BONES, u8::from_json);
/// assert_eq!(bones.unwrap().keys().collect::<Vec<_>>(), ["arm"]);
/// let told: Vec<String> = found.iter().map(|f| format!("{} {}", f.location.json_path, f.code)).collect();
/// assert_eq!(told, ["left leg unrecognized field"]);
/// ```
pub fn read_keyed<V>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    keys: &Keys,
    mut value: impl FnMut(&Node<'_>, &mut Read<'_>) -> Option<V>,
) -> Option<IndexMap<String, V>> {
    if read.stopped() {
        return None;
    }
    let members = read.object(node)?;
    let names = read.telling().names;
    let mark = read.mark();
    let mut out = IndexMap::new();
    let mut failed = false;
    for (key, member) in read.ordered_entries(&members) {
        if read.stopped() {
            return None;
        }
        if !keys.pattern.matches(key, names) {
            read.unknown_by_rule(&member, UnknownRule::Report);
            continue;
        }
        if names == Names::IgnoreAsciiCase && keys.pattern.respelled(key) {
            read.respelled(&member);
        }
        let before = read.mark();
        match read.unkeepable(|read| read.in_map_value(true, |read| value(&member, read))) {
            Some(read) => {
                out.insert(key.to_owned(), read);
            }
            None if read.left_out_since(before) => {}
            None => failed = true,
        }
    }
    if read.telling().objects == Objects::Tree && read.raised_since(mark) {
        return None;
    }
    if failed {
        return None;
    }
    if keys.required && out.is_empty() {
        read.missing_member(&members, keys.pattern.text());
        return None;
    }
    Some(out)
}
