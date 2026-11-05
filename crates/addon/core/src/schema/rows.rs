//! The row table `#[derive(Schema)]` emits per struct, `Schema::FIELDS`: what each field
//! declares, as data, so it can be tested against the generated game tables.

use crate::version::FormatVersion;

/// Which of the three kinds a field is: its type and one attribute say it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FieldKind {
    /// A plain `T` with no facet `default`: a document that omits it is refused.
    Required,
    /// A plain `T` with facet's `#[facet(default = x)]`: filled on read, written back.
    Filled,
    /// An `Option<T>`: absence is kept, with or without a `schema::fallback`.
    Optional,
}

/// A value an attribute states, as written: a literal, or an expression the table cannot hold.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Lit {
    /// An integer literal (a leading `-` included).
    Int(i64),
    /// A float literal.
    Float(f64),
    /// `true` or `false`.
    Bool(bool),
    /// A string literal.
    Str(&'static str),
    /// A bare `#[facet(default)]`: the field type's `Default`.
    TypeDefault,
    /// Any other expression, as its tokens print; a test against a table skips it.
    Expr(&'static str),
}

/// The comparison of a bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BoundOp {
    /// `schema::min`: the value is `>=` the limit.
    Min,
    /// `schema::max`: `<=`.
    Max,
    /// `schema::above`: `>` (exclusive).
    Above,
    /// `schema::below`: `<` (exclusive).
    Below,
}

impl BoundOp {
    /// The key that states it.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Min => "min",
            Self::Max => "max",
            Self::Above => "above",
            Self::Below => "below",
        }
    }
}

/// What a bound compares with.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum Limit {
    /// A number.
    Num(f64),
    /// Another field of the struct, by its Rust name: a cross-field dependency, whose value is
    /// also what a failed member falls back to.
    Field(&'static str),
}

/// One bound of a field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bound {
    /// The comparison.
    pub op: BoundOp,
    /// What it compares with.
    pub limit: Limit,
}

/// What an unknown member of the struct (or below a field) is where it differs from the
/// slice's rule: `schema::unknown = "keep" | "report" | "reject" | "ignore"`, the four
/// dispositions of today's codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UnknownRule {
    /// Kept in the document's unknown table and written back (`UnrecognizedField`).
    Keep,
    /// Reported and not kept (`MemberNotKept`).
    Report,
    /// Refused (`DroppedField`).
    Reject,
    /// Neither kept nor reported (`IgnoredMember`).
    Ignore,
}

impl UnknownRule {
    /// The rule a stored `schema::unknown` string names; `None` for another word (which the
    /// derive refuses at compile time).
    pub fn from_word(word: &str) -> Option<Self> {
        match word {
            "keep" => Some(Self::Keep),
            "report" => Some(Self::Report),
            "reject" => Some(Self::Reject),
            "ignore" => Some(Self::Ignore),
            _ => None,
        }
    }
}

/// One field of a `#[derive(Schema)]` struct, in declaration order (`skip` fields left out).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldRow {
    /// The Rust name (`"0"` for a transparent newtype's field).
    pub name: &'static str,
    /// The JSON key: facet's `rename` or `rename_all` applied (empty for a transparent
    /// newtype's field and for a flattened one, which share their owner's object).
    pub json: &'static str,
    /// The field type, as its tokens print.
    pub ty: &'static str,
    /// Required, filled on read, or optional.
    pub kind: FieldKind,
    /// facet's `default`.
    pub default: Option<Lit>,
    /// `schema::fallback`.
    pub fallback: Option<Lit>,
    /// `schema::min`, `max`, `above`, `below`, in the order written.
    pub bounds: &'static [Bound],
    /// `schema::since_format`.
    pub since_format: Option<FormatVersion>,
    /// `schema::since_engine`.
    pub since_engine: Option<FormatVersion>,
    /// `schema::unknown`.
    pub unknown: Option<UnknownRule>,
    /// `schema::lowercase`.
    pub lowercase: bool,
    /// `schema::foreign_event`.
    pub foreign_event: bool,
    /// facet's `flatten`: the field's members are members of this struct's object.
    pub flatten: bool,
}

impl FieldRow {
    /// A row with nothing declared but its names, type and kind; the derive spells the rest
    /// with `..FieldRow::plain(..)`.
    pub const fn plain(
        name: &'static str,
        json: &'static str,
        ty: &'static str,
        kind: FieldKind,
    ) -> Self {
        Self {
            name,
            json,
            ty,
            kind,
            default: None,
            fallback: None,
            bounds: &[],
            since_format: None,
            since_engine: None,
            unknown: None,
            lowercase: false,
            foreign_event: false,
            flatten: false,
        }
    }
}
