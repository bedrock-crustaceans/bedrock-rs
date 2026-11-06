//! [`JsonPath`]: the breadcrumb of a finding inside a JSON document.

use std::fmt;

/// A breadcrumb inside a JSON document: `minecraft:block/components/minecraft:friction`.
///
/// Member names and array indices, from the document root down, as vanilla's error path stack
/// records them while it reads. `Display` joins them with `/`; an index prints as its number,
/// and a `/` or `~` inside a key is written `~1` or `~0` (the JSON pointer escapes).
///
/// Ordered segment by segment, a key before an index, a path before the paths below it.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonPath {
    segments: Vec<Segment>,
}

/// One step of a [`JsonPath`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Segment {
    Key(Box<str>),
    Index(usize),
}

impl JsonPath {
    /// The empty path: the document root.
    pub fn new() -> Self {
        Self::default()
    }

    /// Steps into the member `key`.
    pub fn push(&mut self, key: &str) {
        self.segments.push(Segment::Key(key.into()));
    }

    /// Steps into the array element `index`.
    pub fn push_index(&mut self, index: usize) {
        self.segments.push(Segment::Index(index));
    }

    /// Continues the path with every segment of `other` (a path relative to this one, such as
    /// `SliceError::path` below a component key).
    pub fn append(&mut self, other: &JsonPath) {
        self.segments.extend(other.segments.iter().cloned());
    }

    /// The path of the member `key` below this one.
    #[must_use]
    pub fn child(&self, key: &str) -> Self {
        let mut path = self.clone();
        path.push(key);
        path
    }

    /// The path of the array element `index` below this one.
    #[must_use]
    pub fn index(&self, index: usize) -> Self {
        let mut path = self.clone();
        path.push_index(index);
        path
    }

    /// This path continued by every segment of `relative`.
    #[must_use]
    pub fn join(&self, relative: &JsonPath) -> Self {
        let mut path = self.clone();
        path.append(relative);
        path
    }

    /// The path of the members `keys` below this one, from the outermost down, each taken as
    /// written: a key of digits stays a key (an array element is [`index`](Self::index)).
    #[must_use]
    pub fn with<K: AsRef<str>>(&self, keys: &[K]) -> Self {
        let mut path = self.clone();
        for key in keys {
            path.push(key.as_ref());
        }
        path
    }

    /// Steps out of the last segment; `false` when the path was already the root.
    pub fn pop(&mut self) -> bool {
        self.segments.pop().is_some()
    }

    /// The number of segments.
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// The path is the document root.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// The first `length` segments of the path.
    #[must_use]
    pub fn prefix(&self, length: usize) -> JsonPath {
        JsonPath {
            segments: self.segments[..length.min(self.segments.len())].to_vec(),
        }
    }

    /// The rest of the path below `prefix`; `None` when the path is not at or below it.
    #[must_use]
    pub fn strip_prefix(&self, prefix: &JsonPath) -> Option<JsonPath> {
        self.segments
            .strip_prefix(prefix.segments.as_slice())
            .map(|rest| JsonPath {
                segments: rest.to_vec(),
            })
    }

    /// The path of the members `keys`, from the document root down, each taken as written.
    #[must_use]
    pub fn of_keys<K: AsRef<str>>(keys: impl IntoIterator<Item = K>) -> Self {
        let mut path = Self::new();
        for key in keys {
            path.push(key.as_ref());
        }
        path
    }

    /// The last segment when it is a member key, as the file has it; `None` for the root and
    /// for a path that ends at an array element.
    #[must_use]
    pub fn last_key(&self) -> Option<&str> {
        match self.segments.last()? {
            Segment::Key(key) => Some(key),
            Segment::Index(_) => None,
        }
    }

    /// Whether the path is the one member `key` of the root.
    #[must_use]
    pub fn is_key(&self, key: &str) -> bool {
        matches!(self.segments.as_slice(), [Segment::Key(only)] if **only == *key)
    }

    /// Whether the path is `prefix` or lies below it, segment for segment (the member
    /// `ab` is not below `a`).
    #[must_use]
    pub fn starts_with(&self, prefix: &JsonPath) -> bool {
        self.segments.starts_with(&prefix.segments)
    }

    /// Whether the path's last segments are those of `suffix`, segment for segment.
    #[must_use]
    pub fn ends_with(&self, suffix: &JsonPath) -> bool {
        self.segments.ends_with(&suffix.segments)
    }

    /// The array index that follows the first member `key` on the path
    /// (`permutations/3/condition` has 3 after `permutations`); `None` when no segment is
    /// that key or an index does not follow it.
    #[must_use]
    pub fn index_after(&self, key: &str) -> Option<usize> {
        let at = self
            .segments
            .iter()
            .position(|segment| matches!(segment, Segment::Key(k) if **k == *key))?;
        match self.segments.get(at + 1)? {
            Segment::Index(index) => Some(*index),
            Segment::Key(_) => None,
        }
    }

    /// The segments, from the document root down: each key as the file has it (the `Display`
    /// text escapes `/` and `~` in a key) and each index as a number.
    pub fn segments(&self) -> impl ExactSizeIterator<Item = JsonPathSegment<'_>> + '_ {
        self.segments.iter().map(|segment| match segment {
            Segment::Key(key) => JsonPathSegment::Key(key),
            Segment::Index(index) => JsonPathSegment::Index(*index),
        })
    }
}

/// One step of a [`JsonPath`], as [`JsonPath::segments`] lists it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JsonPathSegment<'a> {
    /// The member of this key.
    Key(&'a str),
    /// The array element at this index.
    Index(usize),
}

/// Writes a member key as a JSON pointer token (RFC 6901): `~` as `~0` and `/` as `~1`, so a
/// key that holds the separator stays one segment of the text.
///
/// Nearly every key holds neither and is written in one piece. It is looked through as bytes
/// first: the search of a `str` for one of two characters goes a character at a time and is
/// built with this crate, and in the dev profile it took ten times the writing of the key.
fn write_key(f: &mut fmt::Formatter<'_>, key: &str) -> fmt::Result {
    let bytes = key.as_bytes();
    if !bytes.contains(&b'~') && !bytes.contains(&b'/') {
        return f.write_str(key);
    }
    let mut rest = key;
    while let Some(at) = rest.find(['~', '/']) {
        f.write_str(&rest[..at])?;
        f.write_str(if rest.as_bytes()[at] == b'~' {
            "~0"
        } else {
            "~1"
        })?;
        rest = &rest[at + 1..];
    }
    f.write_str(rest)
}

/// The segments joined with `/`; an index as its number, a key as a JSON pointer token (RFC
/// 6901: `~` as `~0`, `/` as `~1`), without the pointer's leading `/`.
///
/// The text is for a reader, not for code: splitting it at `/` and undoing the two escapes
/// gives the keys back, with two exceptions the text cannot tell apart. A key made of digits
/// prints like an index (`["0"]` and element 0 are both `0`), and a path of one empty key
/// prints like the root (both are the empty text; an empty key after other segments is an
/// empty piece, `a//b`). Code that needs a key, a prefix or a step reads
/// [`JsonPath::segments`], [`last_key`](JsonPath::last_key),
/// [`starts_with`](JsonPath::starts_with) or [`strip_prefix`](JsonPath::strip_prefix).
impl fmt::Display for JsonPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, segment) in self.segments.iter().enumerate() {
            if i > 0 {
                f.write_str("/")?;
            }
            match segment {
                Segment::Key(key) => write_key(f, key)?,
                Segment::Index(index) => write!(f, "{index}")?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn components() -> JsonPath {
        JsonPath::of_keys(["minecraft:block", "components"])
    }

    #[test]
    fn child_steps_into_a_member_and_leaves_the_base() {
        let base = components();
        let child = base.child("minecraft:friction");
        assert_eq!(
            child.to_string(),
            "minecraft:block/components/minecraft:friction"
        );
        assert_eq!(base, components());
        let mut pushed = components();
        pushed.push("minecraft:friction");
        assert_eq!(child, pushed);
    }

    #[test]
    fn index_steps_into_an_element_unlike_a_key_of_its_digits() {
        let element = components().index(0);
        assert_eq!(element.to_string(), "minecraft:block/components/0");
        let mut pushed = components();
        pushed.push_index(0);
        assert_eq!(element, pushed);
        assert_ne!(element, components().child("0"), "an index is no key");
    }

    #[test]
    fn join_continues_the_base_with_a_relative_path() {
        let relative = JsonPath::new().child("a").index(2);
        let joined = components().join(&relative);
        assert_eq!(joined.to_string(), "minecraft:block/components/a/2");
        let mut appended = components();
        appended.append(&relative);
        assert_eq!(joined, appended);
        assert_eq!(components().join(&JsonPath::new()), components());
    }

    #[test]
    fn with_steps_into_each_key_as_written() {
        let path = JsonPath::new().with(&["minecraft:block", "components", "0"]);
        assert_eq!(path, components().child("0"));
        assert_eq!(path.to_string(), "minecraft:block/components/0");
        assert_eq!(JsonPath::new().with(&["a/b"]).to_string(), "a~1b");
        assert_eq!(components().with::<&str>(&[]), components());
        let owned = vec![String::from("x")];
        assert_eq!(JsonPath::new().with(&owned), JsonPath::of_keys(["x"]));
    }
}
