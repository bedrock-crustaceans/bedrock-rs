//! Reading the way jsoncpp does, for the families that read a file as vanilla's JSON reader
//! does: a BOM skipped, comments accepted, the last of repeated members kept.

pub mod fork;
pub mod lenient;

use std::str::Utf8Error;
use std::sync::Arc;

use super::{MAX_JSON_DEPTH, TooDeep, Value, check_depth, last_wins, strip_bom};

/// Why a file is no JSON for the jsoncpp readers ([`read`], and the lenient and merge
/// readers of the texture and sound files).
///
/// The messages are vanilla-facing diagnostics text and stay as they are.
#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ReadError {
    /// [`read`] met nesting past [`MAX_JSON_DEPTH`].
    #[error("nesting too deep at byte {offset}")]
    TooDeep {
        /// The byte offset of the first bracket past the limit.
        offset: usize,
    },
    /// The lenient and merge readers met nesting past [`MAX_JSON_DEPTH`].
    #[error(transparent)]
    Nested(#[from] TooDeep),
    /// The bytes are no UTF-8.
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    /// The text is no JSON.
    #[error(transparent)]
    Syntax(Arc<facet_json::DeserializeError>),
}

impl From<facet_json::DeserializeError> for ReadError {
    fn from(error: facet_json::DeserializeError) -> Self {
        Self::Syntax(Arc::new(error))
    }
}

/// The member `key` of an object when it is present and not null.
pub fn member<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value
        .as_object()
        .and_then(|object| object.get(key))
        .filter(|member| !member.is_null())
}

/// The number of a value, as a float.
pub fn number(value: &Value) -> Option<f64> {
    value.as_number().map(facet_value::VNumber::to_f64_lossy)
}

/// Reads the bytes of one file as a JSON value the way the game's JSON reader does: a BOM skipped,
/// comments accepted, the last of repeated members kept.
///
/// # Errors
///
/// A [`ReadError`] for nesting past the limit, bytes that are no UTF-8, or text that is no JSON.
// jsoncpp's lenient parse: comments allowed, repeated members last-wins.
pub fn read(bytes: &[u8]) -> Result<Value, ReadError> {
    // The BOM is blanked, not cut, so every offset is an offset into the file as written.
    let bytes = &*super::blank_bom(bytes);
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH).map_err(|deep| ReadError::TooDeep {
        offset: deep.offset,
    })?;
    let text = std::str::from_utf8(bytes)?;
    Ok(super::from_str_jsonc::<Value>(&last_wins(text))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn read_names_each_failure_with_its_variant() {
        let deep = "[".repeat(MAX_JSON_DEPTH + 1);
        assert!(matches!(
            read(deep.as_bytes()),
            Err(ReadError::TooDeep { offset }) if offset == MAX_JSON_DEPTH
        ));
        assert!(matches!(read(&[b'"', 0xff, b'"']), Err(ReadError::Utf8(_))));
        assert!(matches!(read(b"{"), Err(ReadError::Syntax(_))));
        assert!(read(b"\xef\xbb\xbf{/* c */ \"a\": 1, \"a\": 2}").is_ok());
    }

    #[test]
    fn read_error_keeps_the_texts_of_the_readers() {
        let deep = "[".repeat(MAX_JSON_DEPTH + 1);
        let error = read(deep.as_bytes()).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("nesting too deep at byte {MAX_JSON_DEPTH}")
        );
        let syntax = read(b"{").unwrap_err();
        assert!(!syntax.to_string().is_empty());
    }

    /// The texture readers' `asInt` and the sound fork's agree but on an integer token past `i64`:
    /// its low 32 bits for the texture readers, 0 for the sound fork.
    #[test]
    fn the_two_as_int_copies_and_their_one_difference() {
        use super::fork::as_int as fork;
        use super::lenient::as_int as texture;
        let json = |text: &str| super::super::from_str_jsonc::<Value>(text).unwrap();
        let wide = (u64::MAX - 1).to_string();
        for (text, low_word, zero) in [
            ("7", 7, 7),
            ("-7", -7, -7),
            ("4294967297", 1, 1),
            (wide.as_str(), -2, 0),
            ("2.9", 2, 2),
            ("-2.9", -2, -2),
            ("1e300", i32::MAX, i32::MAX),
            ("-1e300", i32::MIN, i32::MIN),
            ("true", 1, 1),
            ("false", 0, 0),
            ("\"7\"", 0, 0),
            ("[1]", 0, 0),
            ("{}", 0, 0),
            ("null", 5, 5),
        ] {
            let value = json(text);
            assert_eq!(texture(Some(&value), 5), low_word, "texture {text}");
            assert_eq!(fork(&value, 5), zero, "fork {text}");
        }
        assert_eq!(texture(None, 5), 5, "a missing member is the default");
    }

    #[test]
    fn read_error_is_send_and_sync() {
        fn assert_traits<T: Send + Sync + Clone + std::fmt::Debug + Error>() {}
        assert_traits::<ReadError>();
    }
}
