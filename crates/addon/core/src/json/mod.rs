//! The public part of the thin helpers over facet-json and facet-value: the untyped value, the
//! strict scalar wrappers component members are made of ([`StrictInt`], [`StrictFloat`],
//! [`StrictBool`], [`StrictString`], with the scalar shape they are read through,
//! [`ScalarShape`], and their error, [`StrictError`]), the nesting limit ([`MAX_JSON_DEPTH`]),
//! the envelope of a versioned document ([`RootProbe`], [`EnvelopeVersion`],
//! [`probe_envelope`]), the write style and the encode error. Parsing is facet's; the crate
//! adds no JSON tree of its own.
//!
//! The rest is internal to the crate family (hidden from the documentation, outside `SemVer`).
//! Its rules:
//! - no module but `read` calls a facet-json parser: its `from_str_jsonc` and siblings scan the
//!   text first and refuse a string escape outside RFC 8259 (facet decodes `"\é"` out of
//!   bounds), nesting past `MAX_JSON_DEPTH`, and give facet nothing after the root value; an
//!   integer token too large for 64 bits is read as the double it spells, as vanilla's readers
//!   read it (facet alone makes it a string, or refuses the file past 128 bits);
//! - before every parse, `blank_bom` (facet rejects a BOM, vanilla skips it; blanked, the
//!   spans stay offsets into the file as written) and `check_depth`
//!   with `MAX_JSON_DEPTH` (facet has no nesting limit and overflows the stack); an untyped
//!   `Value` is read by the door itself, on the same stack at any depth, and a typed target
//!   by facet on a stack the door sizes by the depth of the text;
//! - typed structs are deserialised from text with the jsonc entry points, a slice at a time
//!   through `JsonSlice` and `parse_at`, which report failures at file offsets with the JSON
//!   member names; a modern document's root through `probe_envelope`;
//! - where vanilla rejects a value of the wrong JSON type, the strict wrappers `StrictInt`,
//!   `StrictFloat`, `StrictBool`, `StrictString`;
//! - unknown members through `unknown_members` / `keep_unknown_members`, kept in
//!   `document::Extra`; every `#[facet(skip)]` field is renamed `"\u{0}<name>"`, since facet
//!   binds a member to a skipped field of its name (see `Extra`'s "Skipped fields");
//! - writing through `write_json` / `to_json`, and a `Value` as facet-json itself spells it
//!   through `to_facet_json`: no other module calls a facet-json writer, as none calls a
//!   parser (its writers call themselves for each level of a value).

mod clamp_text;
mod coerce;
pub(crate) use clamp_text::first_beyond_double;
mod duplicates;
mod envelope;
pub(crate) mod extra;
mod finite;
pub mod kit;
mod null;
mod path;
mod prepared;
mod read; // the only caller of facet-json's parsers
mod slice;
mod stack;
mod strict;
mod text;
mod unknown;
mod write;

#[doc(hidden)]
pub use clamp_text::{
    ClampedText, Width, clamp_beyond_double, clamp_beyond_double_first_kept, clamp_text,
    clamp_tokens, drop_spoken_for, spoken_for,
};
pub use coerce::{Coerce, Coercing, IsIntRule, WideInt};
#[doc(hidden)]
pub use duplicates::{last_wins, last_wins_except};
pub use envelope::{EnvelopeVersion, RootProbe, probe_envelope};
#[doc(hidden)]
pub use finite::{Clamped, clamp_non_finite, clamp_non_finite_warned};
#[doc(hidden)]
pub use null::{
    NULL_MEMBER_READ_AS_ABSENT, NullRule, null_members, nulls_read, parse_at_null_absent,
    reject_nulls, remove_null_members, tell_null_members,
};
pub use prepared::{PrepareError, Prepared};
#[doc(hidden)]
pub use read::{
    DOCUMENT_NUMBER_TOO_BIG, DOCUMENT_ROOT_NOT_SINGULAR, Scanned, TRAILING_CONTENT, UnsafeText,
    document_json_refusal, from_slice_jsonc, from_str, from_str_borrowed_jsonc, from_str_jsonc,
    scan, trailing_content, trailing_content_finding,
};
#[doc(hidden)]
pub use slice::{JsonSlice, SliceError, parse_at, span_at};
pub use strict::{
    JsonType, ReaderMode, ScalarShape, StrictBool, StrictError, StrictFloat, StrictFloatType,
    StrictInt, StrictIntType, StrictString, with_reader_mode,
};
pub use text::MAX_JSON_DEPTH;
#[doc(hidden)]
pub use text::{TooDeep, blank_bom, check_depth, strip_bom};
#[doc(hidden)]
pub use unknown::{
    BEYOND_FLOAT_MEMBER, Misread, Misreading, keep_members, keep_members_and_range_members,
    keep_unknown_members, too_deep_to_walk, unknown_members,
};
#[doc(hidden)]
pub use write::{to_facet_json, to_facet_json_signed, to_json, write_json};

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
