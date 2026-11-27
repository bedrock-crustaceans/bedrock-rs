//! The texts of the game's linked-asset validator, the link pass of the crate: core vocabulary,
//! so a content module that mirrors a reference the validator holds (a camera preset, a
//! catalogue item) gives its line without naming the link module.

use super::{JsonPath, JsonPathSegment, sentence};

/// What the game's validator says of a reference it cannot resolve.
// hidden: the game's validator wording, used by the families' readers
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Validator {
    /// `'{} Missing referenced asset {}'`: a named asset nothing defines.
    Missing,
    /// `'{} Invalid asset path {}'`: a file path (a loot table, a structure,
    /// a texture) no pack holds.
    InvalidPath,
}

/// The validator's breadcrumb of `path` from its segment `from` on, as the reference's
/// context writes it: ` -> <key>` for a member, `[<i>]` for an element, the leading space
/// trimmed (`-> components -> minecraft:placement_filter -> conditions[0] -> block_filter[0]`).
// hidden: the game's validator wording, used by the families' readers
#[doc(hidden)]
#[must_use]
pub fn breadcrumb(path: &JsonPath, from: usize) -> String {
    let mut out = String::new();
    for segment in path.segments().skip(from) {
        match segment {
            JsonPathSegment::Index(index) => {
                out.push('[');
                out.push_str(&index.to_string());
                out.push(']');
            }
            JsonPathSegment::Key(key) => {
                out.push_str(" -> ");
                out.push_str(key);
            }
        }
    }
    out.trim_start().to_owned()
}

/// The validator's line for `name` at `path` (from the segment `from`): the breadcrumb, a
/// colon, two spaces and the miss (the game writes every one when it finishes loading linked
/// assets, at the error level, area Json, and drops nothing: the crate's severity stays the rule's
/// own grading).
// hidden: the game's validator wording, used by the families' readers
#[doc(hidden)]
#[must_use]
pub fn validator_text(says: Validator, path: &JsonPath, from: usize, name: &str) -> String {
    sentence(
        match says {
            Validator::Missing => "{}:  Missing referenced asset {}",
            Validator::InvalidPath => "{}:  Invalid asset path {}",
        },
        &[breadcrumb(path, from), name.to_owned()],
    )
}
