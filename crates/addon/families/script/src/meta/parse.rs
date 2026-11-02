//! The strict read every metadata model shares: text checks, the typed parse, and the
//! unknown-member walk over the same text.

use facet::Facet;

use super::MetaError;
use bedrock_addon_core::json::{
    JsonSlice, MAX_JSON_DEPTH, Value, check_depth, parse_at, strip_bom, unknown_members,
};

/// `T` read from `bytes`, and the same text as a `Value`; the first member no field of `T`
/// takes is an error.
pub(crate) fn strict<T: for<'f> Facet<'f>>(bytes: &[u8]) -> Result<(T, Value), MetaError> {
    // A byte order mark is blanked first so that every offset stays the file's own.
    let bytes = &*bedrock_addon_core::json::blank_bom(bytes);
    let bytes = strip_bom(bytes);
    check_depth(bytes, MAX_JSON_DEPTH)?;
    let text = std::str::from_utf8(bytes)?;
    let typed: T = parse_at(JsonSlice::new(text))?;
    let value: Value = parse_at(JsonSlice::new(text))?;
    if let Some((path, _)) = unknown_members(T::SHAPE, &value).into_iter().next() {
        return Err(MetaError::UnknownMember { path });
    }
    Ok((typed, value))
}
