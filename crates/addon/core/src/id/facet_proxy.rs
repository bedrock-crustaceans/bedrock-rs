//! The `facet` impl of `Id<K>`: a proxy over the JSON shapes a deserializer can offer; only a
//! string is an identifier, and any string is kept as written (the lenient parse).

use std::sync::Arc;

use super::{Id, Kind};

/// Every JSON shape a deserializer can offer. The shapes an identifier does not take exist so
/// the conversion can reject them: without them the untagged solver of `facet-json` coerces a
/// number into a string.
#[derive(Debug, facet::Facet)]
#[facet(untagged)]
#[repr(u8)]
#[allow(
    dead_code,
    reason = "the payloads of the rejected shapes are never read"
)]
pub enum IdProxy {
    /// `null`.
    Null,
    /// A string: the identifier.
    String(String),
    /// A boolean.
    Bool(bool),
    /// A non-negative integer.
    UInt(u64),
    /// A negative integer.
    Int(i64),
    /// Any other number.
    Number(f64),
}

/// Why a JSON value is not an identifier.
#[derive(Debug, thiserror::Error)]
#[error("an identifier must be a JSON string")]
pub struct NotAString;

impl<K: Kind> TryFrom<IdProxy> for Id<K> {
    type Error = NotAString;

    fn try_from(proxy: IdProxy) -> Result<Self, NotAString> {
        match proxy {
            IdProxy::String(text) => Ok(Id::parse(Arc::<str>::from(text))),
            _ => Err(NotAString),
        }
    }
}

impl<K: Kind> From<&Id<K>> for IdProxy {
    fn from(id: &Id<K>) -> Self {
        IdProxy::String(id.as_str().to_owned())
    }
}
