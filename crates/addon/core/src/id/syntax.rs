//! The syntax check and the canonical form of an identifier, shared by `Id::new`,
//! `Id::from_static` (a `const` evaluation) and the comparison traits.

use std::hash::{Hash, Hasher};

use super::{IdError, Suffix, Syntax, VANILLA_NAMESPACE};
use crate::pack_identity::{PathFault, check_path};

/// The longest identifier text of a non-path kind, in bytes (a sanity bound of the crate's own).
const MAX_LEN: usize = u16::MAX as usize;

/// ASCII whitespace and control characters (U+0000 to U+0020, U+007F) are never part of a
/// namespaced or prefixed identifier.
const fn is_blank(b: u8) -> bool {
    b <= b' ' || b == 0x7f
}

/// The rejected byte is ASCII whenever this is called, so it is its own character.
const fn invalid(index: usize, b: u8) -> IdError {
    IdError::InvalidChar {
        index,
        ch: b as char,
    }
}

/// Checks `text` against `syntax`; `const` so `Id::from_static` runs it at compile time.
pub(super) const fn validate(text: &str, syntax: Syntax, suffix: Suffix) -> Result<(), IdError> {
    if text.is_empty() {
        return Err(IdError::Empty);
    }
    match syntax {
        Syntax::Namespaced { default_vanilla } => {
            namespaced(text.as_bytes(), default_vanilla, suffix)
        }
        Syntax::Prefixed { prefix, fold_case } => prefixed(text.as_bytes(), prefix, fold_case),
        Syntax::Path => match check_path(text) {
            Ok(0) => Err(IdError::Empty),
            Ok(_) => Ok(()),
            Err(PathFault::DotSegment(index)) => Err(invalid(index, b'.')),
            Err(PathFault::InvalidChar(index, ch)) => Err(IdError::InvalidChar { index, ch }),
            Err(PathFault::TooManySegments | PathFault::TooLong) => Err(IdError::TooLong),
        },
        Syntax::Free => {
            if text.len() > MAX_LEN {
                Err(IdError::TooLong)
            } else {
                Ok(())
            }
        }
    }
}

const fn namespaced(bytes: &[u8], default_vanilla: bool, suffix: Suffix) -> Result<(), IdError> {
    if bytes.len() > MAX_LEN {
        return Err(IdError::TooLong);
    }
    let mut end = bytes.len();
    // The event suffix: `<` … `>` at the end, non-empty, with no further brackets or blanks.
    if let Suffix::Event = suffix
        && let Some(open) = find(bytes, b'<', end)
    {
        let close = bytes.len() - 1;
        if bytes[close] != b'>' || close == open + 1 {
            return Err(invalid(open, b'<'));
        }
        let mut i = open + 1;
        while i < close {
            let b = bytes[i];
            if is_blank(b) || b == b'<' || b == b'>' {
                return Err(invalid(i, b));
            }
            i += 1;
        }
        end = open;
    }
    // The aux suffix: with two or more colons, the part after the last one is an `i16`.
    if let Suffix::Aux = suffix
        && count(bytes, b':', end) >= 2
        && let Some(colon) = rfind(bytes, b':', end)
    {
        if !is_i16(bytes, colon + 1, end) {
            return Err(invalid(colon, b':'));
        }
        end = colon;
    }
    let mut colon = None;
    let mut i = 0;
    while i < end {
        let b = bytes[i];
        if is_blank(b) || b == b'<' || b == b'>' || (b == b':' && colon.is_some()) {
            return Err(invalid(i, b));
        }
        if b == b':' {
            colon = Some(i);
        }
        i += 1;
    }
    match colon {
        Some(colon) if colon == 0 || colon + 1 == end => Err(IdError::Empty),
        Some(_) => Ok(()),
        None if end == 0 => Err(IdError::Empty),
        None if default_vanilla => Ok(()),
        None => Err(IdError::MissingNamespace),
    }
}

const fn prefixed(bytes: &[u8], prefix: &'static str, fold_case: bool) -> Result<(), IdError> {
    if bytes.len() > MAX_LEN {
        return Err(IdError::TooLong);
    }
    if !starts_with(bytes, prefix.as_bytes(), fold_case) {
        return Err(IdError::WrongPrefix { expected: prefix });
    }
    if bytes.len() == prefix.len() {
        return Err(IdError::Empty);
    }
    let mut i = prefix.len();
    while i < bytes.len() {
        if is_blank(bytes[i]) {
            return Err(invalid(i, bytes[i]));
        }
        i += 1;
    }
    Ok(())
}

const fn starts_with(bytes: &[u8], prefix: &[u8], fold_case: bool) -> bool {
    if bytes.len() < prefix.len() {
        return false;
    }
    let mut i = 0;
    while i < prefix.len() {
        let (a, b) = (bytes[i], prefix[i]);
        let same = if fold_case {
            a.eq_ignore_ascii_case(&b)
        } else {
            a == b
        };
        if !same {
            return false;
        }
        i += 1;
    }
    true
}

const fn find(bytes: &[u8], needle: u8, to: usize) -> Option<usize> {
    let mut i = 0;
    while i < to {
        if bytes[i] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

const fn rfind(bytes: &[u8], needle: u8, to: usize) -> Option<usize> {
    let mut i = to;
    while i > 0 {
        i -= 1;
        if bytes[i] == needle {
            return Some(i);
        }
    }
    None
}

const fn count(bytes: &[u8], needle: u8, to: usize) -> usize {
    let mut n = 0;
    let mut i = 0;
    while i < to {
        if bytes[i] == needle {
            n += 1;
        }
        i += 1;
    }
    n
}

/// `bytes[from..to]` is an optional `-` and one or more ASCII digits whose value fits an `i16`.
const fn is_i16(bytes: &[u8], from: usize, to: usize) -> bool {
    let mut i = from;
    let negative = i < to && bytes[i] == b'-';
    if negative {
        i += 1;
    }
    if i == to {
        return false;
    }
    // One past `i16::MAX` in magnitude, the most a negative value may reach.
    let limit: i32 = 32_768;
    let mut value: i32 = 0;
    while i < to {
        let b = bytes[i];
        if !b.is_ascii_digit() {
            return false;
        }
        value = value * 10 + (b - b'0') as i32;
        if value > limit {
            return false;
        }
        i += 1;
    }
    negative || value < limit
}

/// The text of a namespaced identifier without its suffix (the event from the first `<`; the
/// aux part after the last of two or more colons).
fn body(text: &str, suffix: Suffix) -> &str {
    match suffix {
        Suffix::Event => text.split_once('<').map_or(text, |(body, _)| body),
        Suffix::Aux if text.matches(':').count() >= 2 => {
            text.rsplit_once(':').map_or(text, |(body, _)| body)
        }
        _ => text,
    }
}

/// `(namespace, path)` of a namespaced text, its suffix removed; the namespace ends at the
/// first `:`.
pub(super) fn split(text: &str, suffix: Suffix) -> (Option<&str>, &str) {
    let body = body(text, suffix);
    match body.split_once(':') {
        Some((namespace, path)) => (Some(namespace), path),
        None => (None, body),
    }
}

/// What `Eq` and `Hash` compare.
pub(super) enum Canonical<'a> {
    /// A namespaced identifier: its namespace (defaulted where the kind says so) and path.
    Namespaced(Option<&'a str>, &'a str),
    /// Compared without ASCII case.
    Folded(&'a str),
    /// A pack path: compared by its non-empty segments, split at `/` or `\`.
    Path(&'a str),
    /// Compared as written.
    Exact(&'a str),
}

/// The non-empty segments of a path text, as `PackPath::new` normalises them.
fn segments(text: &str) -> impl Iterator<Item = &str> {
    text.split(['/', '\\'])
        .filter(|segment| !segment.is_empty())
}

/// The canonical form of `text` under `syntax`.
pub(super) fn canonical(text: &str, syntax: Syntax, suffix: Suffix) -> Canonical<'_> {
    match syntax {
        Syntax::Namespaced { default_vanilla } => {
            let (namespace, path) = split(text, suffix);
            let namespace = match namespace {
                None if default_vanilla => Some(VANILLA_NAMESPACE),
                namespace => namespace,
            };
            Canonical::Namespaced(namespace, path)
        }
        Syntax::Prefixed {
            fold_case: true, ..
        } => Canonical::Folded(text),
        Syntax::Path => Canonical::Path(text),
        _ => Canonical::Exact(text),
    }
}

impl PartialEq for Canonical<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Namespaced(a, p), Self::Namespaced(b, q)) => a == b && p == q,
            (Self::Folded(a), Self::Folded(b)) => a.eq_ignore_ascii_case(b),
            (Self::Path(a), Self::Path(b)) => segments(a).eq(segments(b)),
            (Self::Exact(a), Self::Exact(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Canonical<'_> {}

/// Hashes the bytes `eq` compares: `namespace:path` (the path alone when there is no
/// namespace; it then holds no `:`), the ASCII-lowercased text, or a path's segments joined
/// with `/`.
impl Hash for Canonical<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Namespaced(namespace, path) => {
                if let Some(namespace) = namespace {
                    state.write(namespace.as_bytes());
                    state.write_u8(b':');
                }
                state.write(path.as_bytes());
            }
            Self::Folded(text) => {
                for b in text.bytes() {
                    state.write_u8(b.to_ascii_lowercase());
                }
            }
            Self::Path(text) => {
                for (i, segment) in segments(text).enumerate() {
                    if i > 0 {
                        state.write_u8(b'/');
                    }
                    state.write(segment.as_bytes());
                }
            }
            Self::Exact(text) => state.write(text.as_bytes()),
        }
        state.write_u8(0xff);
    }
}
