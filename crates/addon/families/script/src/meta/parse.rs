//! The strict read every metadata model shares: text checks, the typed parse, and the
//! unknown-member walk over the same text.

use facet::Facet;

use super::MetaError;
use bedrock_addon_core::json::{
    JsonSlice, MAX_JSON_DEPTH, PrepareError, Prepared, Value, blank_bom, check_depth, parse_at,
    unknown_members,
};

/// `T` read from `bytes`, and the same text as a `Value`; the first member no field of `T`
/// takes is an error.
pub(crate) fn strict<T: for<'f> Facet<'f>>(bytes: &[u8]) -> Result<(T, Value), MetaError> {
    // A byte order mark is blanked, so that every offset stays the file's own. The depth
    // before the encoding: a text both too deep and not UTF-8 is `TooDeep`. The mark is
    // blanked before the scan too, which would stop at it.
    let bytes = &*blank_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH)?;
    let text = match Prepared::new(bytes) {
        Ok(prepared) => prepared.into_text(),
        Err(PrepareError::TooDeep(error)) => return Err(MetaError::TooDeep(error)),
        Err(PrepareError::NotUtf8(error)) => return Err(MetaError::NotUtf8(error)),
        // `new` reads zero bytes as the empty text; were it to refuse them, they are that
        // text all the same, which the parse below refuses.
        Err(PrepareError::Empty) => std::borrow::Cow::Borrowed(""),
    };
    let text = &*text;
    let typed: T = parse_at(JsonSlice::new(text))?;
    let value: Value = parse_at(JsonSlice::new(text))?;
    if let Some((path, _)) = unknown_members(T::SHAPE, &value).into_iter().next() {
        return Err(MetaError::UnknownMember { path });
    }
    Ok((typed, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The nesting is checked before the encoding: a text that fails both is `TooDeep`.
    #[test]
    fn a_text_too_deep_and_not_utf8_is_too_deep() {
        let mut bytes = vec![b'['; MAX_JSON_DEPTH + 1];
        bytes.push(0xff);
        let error = strict::<Value>(&bytes).map(|_| ()).expect_err("refused");
        assert!(matches!(error, MetaError::TooDeep(_)), "{error:?}");
    }

    /// Behind a byte-order mark too, at the offset in the file as written.
    #[test]
    fn a_text_too_deep_and_not_utf8_behind_a_bom_is_too_deep() {
        let mut bytes = b"\xEF\xBB\xBF".to_vec();
        bytes.extend(vec![b'['; MAX_JSON_DEPTH + 1]);
        bytes.push(0xff);
        let error = strict::<Value>(&bytes).map(|_| ()).expect_err("refused");
        assert!(
            matches!(error, MetaError::TooDeep(ref e) if e.offset == MAX_JSON_DEPTH + 3),
            "{error:?}"
        );
    }
}
