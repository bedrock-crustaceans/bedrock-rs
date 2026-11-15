//! The strict read every metadata model shares: text checks, the typed parse, and the
//! unknown-member walk over the same text.

use super::{JsonFault, MetaError};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::parse::{Options, parse};
use bedrock_addon_core::json::{
    MAX_JSON_DEPTH, PrepareError, Prepared, Value, blank_bom, check_depth,
};
use bedrock_addon_core::schema::{FromJson, read_document};
use bedrock_addon_core::versioning::Reading;

/// `T` read from `bytes`, and the same text as a `Value`; the first member no field of `T`
/// takes is an error.
pub(crate) fn strict<T: FromJson>(bytes: &[u8]) -> Result<(T, Value), MetaError> {
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
    let value: Value = parse(text, Options::default())?.value;
    let mut told = Diagnostics::default();
    let reading = Reading::OWN.strict();
    let mut read = Read::new(reading, &mut told).profile(Profile::FIRST_FAILURE);
    let typed = read_document::<T>(&Node::new(&value, JsonPath::new()), &mut read);
    let Some(typed) = typed else {
        let path = told
            .iter()
            .find(|finding| finding.severity == Severity::Error)
            .map(|finding| finding.location.json_path.clone())
            .unwrap_or_default();
        return Err(MetaError::Json(JsonFault::Shape { path }));
    };
    // The members no field takes are told after the read, in the file's order.
    let unknown = Code::from(&core_codes::UNRECOGNIZED_FIELD);
    if let Some(finding) = told.iter().find(|finding| finding.code == unknown) {
        return Err(MetaError::UnknownMember {
            path: finding.location.json_path.clone(),
        });
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
