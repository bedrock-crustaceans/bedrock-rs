//! The public part of the JSON helpers over facet-value: the untyped value, the scalar shape
//! ([`ScalarShape`]) and its error ([`StrictError`]), the nesting limit ([`MAX_JSON_DEPTH`]),
//! the envelope of a versioned document ([`RootProbe`], [`EnvelopeVersion`],
//! [`probe_envelope`]), the write style and the encode error.
//!
//! The rest is internal to the crate family (hidden from the documentation, outside `SemVer`).
//! Its rules:
//! - every text is read by [`parse`]: a `Value` and the spans of its parts, or the [`Fault`](parse::Fault)
//!   of its first syntax error (`from_str_jsonc` and `from_str` are its value alone). It
//!   refuses a string escape outside RFC 8259 and nesting past `MAX_JSON_DEPTH`, reads an
//!   integer token too large for 64 bits as the double it spells, as vanilla's readers read
//!   it, and stops at the end of the root value; without a call of itself for each level;
//! - before a parse of bytes, `blank_bom` (vanilla skips a BOM; blanked, the spans stay
//!   offsets into the file as written) and `check_depth` with `MAX_JSON_DEPTH`;
//! - typed values are read from the parsed value by the reader (`schema::FromJson`, the
//!   derive's `Schema`, through `kit`'s `Node` and `Read`), under the slice's `Reading`, which
//!   says where vanilla rejects a value of the wrong JSON type; a modern document's root
//!   through `probe_envelope`;
//! - unknown members are told and kept by the reader in a document's `Unknown` table for the
//!   structs marked with `document::Extra`, and written back by `merge_unknown` /
//!   `write_json_with`;
//! - writing through `write_json` / `to_json`, and a `Value` on one line through
//!   `to_facet_json`; no module calls a facet-json reader or writer.

// hidden: API not yet reviewed
pub(crate) mod arms;
#[doc(hidden)]
pub mod build;
#[cfg(test)]
mod characterize_lex;
mod clamp_text;
mod coerce;
pub(crate) use clamp_text::first_beyond_double;
mod duplicates;
mod envelope;
pub(crate) mod extra;
mod finite;
mod finite_bytes;
// hidden: API not yet reviewed
#[doc(hidden)]
pub mod jsoncpp;
pub mod kit;
mod lex;
// hidden: API not yet reviewed
#[doc(hidden)]
pub mod lossy;
mod null;
// hidden: API not yet reviewed
#[doc(hidden)]
pub mod parse;
mod prepared;
mod read; // the text entry points over the parser
mod scalar;
mod slice;
mod stack;
mod strict;
mod text;
mod write;

// hidden: API not yet reviewed
#[doc(hidden)]
pub use clamp_text::{
    ClampPlan, ClampedText, Width, clamp_text, clamp_tokens, drop_spoken_for, spoken_for,
};
pub use coerce::{Coerce, Coercing, IsIntRule, WideInt};
// hidden: API not yet reviewed
#[doc(hidden)]
pub use duplicates::{Winner, blank_repeats, first_wins, last_wins, last_wins_except};
pub(crate) use envelope::slice_fault_span;
pub use envelope::{EnvelopeError, EnvelopeVersion, RootProbe, probe_envelope};
// hidden: API not yet reviewed
#[doc(hidden)]
pub use finite::{Clamped, clamp_non_finite, clamp_non_finite_warned};
pub use finite_bytes::FiniteBytes;
// hidden: API not yet reviewed
#[doc(hidden)]
pub use null::{
    NULL_MEMBER_READ_AS_ABSENT, NullRule, null_members_of_shape, remove_null_members,
    tell_null_members,
};
pub use prepared::{PrepareError, Prepared};
// hidden: API not yet reviewed
#[doc(hidden)]
pub use lex::{Scanned, UnsafeText, scan};
// hidden: API not yet reviewed
#[doc(hidden)]
pub use read::{
    DOCUMENT_NUMBER_TOO_BIG, DOCUMENT_ROOT_NOT_SINGULAR, TRAILING_CONTENT, document_json_refusal,
    from_str, from_str_jsonc, trailing_content, trailing_content_finding,
};
pub use scalar::{
    BeyondWidth, Declared, JsonType, Refusal, Respelt, ScalarRules, ScalarShape, Scalars,
    StrictError, StrictIntType, Token, Verdict, judge,
};
// hidden: API not yet reviewed
#[doc(hidden)]
pub use slice::JsonSlice;
// Hidden: what still holds a wrapper is listed, with the gap it waits on, in `strict`'s
// module doc.
#[doc(hidden)]
pub use strict::StrictBool;
pub use text::MAX_JSON_DEPTH;
// hidden: API not yet reviewed
pub(crate) use stack::with_stack_for;
#[doc(hidden)]
pub use text::{TooDeep, blank_bom, check_depth, strip_bom};
pub(crate) use write::levels_of;
// hidden: API not yet reviewed
#[doc(hidden)]
pub use kit::{BEYOND_FLOAT_MEMBER, Misread, Misreading};
pub use write::{Precedence, merge_unknown, merge_unknown_with};
// hidden: API not yet reviewed
#[doc(hidden)]
pub use write::{
    UNKNOWN_MEMBER_NOT_WRITTEN, to_facet_json, to_json, write_json, write_json_with,
    write_json_with_precedence,
};

/// An untyped JSON value: opaque members (`events`, `extra`, raw layers) are held as this.
pub use facet_value::Value;

/// What core adds to [`Value`], a foreign type that takes no inherent method.
pub trait ValueExt: value_seal::Sealed {
    /// The JSON type of the value, as jsoncpp's type tests see it: one of the six, a number
    /// whether integral or not.
    ///
    /// The molang crate's [`JsonType`](crate::molang::JsonType) (the six), not
    /// [`json::JsonType`](JsonType), which is the strict reader's scalars only, integer and real apart.
    fn json_type(&self) -> crate::molang::JsonType;
}

mod value_seal {
    /// Only [`Value`](super::Value) has the extension.
    pub trait Sealed {}
    impl Sealed for super::Value {}
}

impl ValueExt for Value {
    fn json_type(&self) -> crate::molang::JsonType {
        use crate::molang::JsonType;
        if self.is_null() {
            JsonType::Null
        } else if self.is_bool() {
            JsonType::Bool
        } else if self.is_number() {
            JsonType::Number
        } else if self.is_string() {
            JsonType::String
        } else if self.is_array() {
            JsonType::Array
        } else {
            JsonType::Object
        }
    }
}

/// How `Content::write_json` lays the text out; default two spaces and a trailing newline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WriteStyle {
    /// The indentation per level.
    pub indent: Indent,
    /// End the text with a newline.
    pub trailing_newline: bool,
}

impl Default for WriteStyle {
    fn default() -> Self {
        Self {
            indent: Indent::Spaces(2),
            trailing_newline: true,
        }
    }
}

/// The indentation of written JSON.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Indent {
    /// One member or element per line, indented by this many spaces per level.
    Spaces(u8),
    /// One member or element per line, indented by a tab per level.
    Tab,
    /// Everything on one line, without spaces.
    Compact,
}

/// Why a value could not be written as JSON.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum EncodeError {
    /// A value with no JSON spelling (a non-finite number).
    #[error("{what:?} cannot be written as JSON (at `{at}`)")]
    NotRepresentable {
        /// Where in the document.
        at: crate::diagnostic::JsonPath,
        /// What could not be written.
        what: Unrepresentable,
    },
}

/// What [`EncodeError::NotRepresentable`] found.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Unrepresentable {
    /// A NaN: JSON has no spelling for it.
    NaN,
    /// Positive infinity.
    Infinity,
    /// Negative infinity.
    NegativeInfinity,
    /// An array or object nested deeper than the readers follow a text (128 levels): the
    /// text would be one no reader of the crate reads back.
    TooDeep,
    /// A value whose type refuses to be written: a Molang constant that is not finite (its
    /// proxy refuses it), or a type the JSON writer has no form for.
    Unsupported,
}
