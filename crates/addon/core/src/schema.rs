//! The `schema::` attribute namespace and what `#[derive(Schema)]` implements.
//!
//! The grammar of `bedrock_addon_schema` is re-exported whole, so that `#[facet(schema::min =
//! 0.0)]` resolves through `use bedrock_addon_core::schema::{self, Schema};`, which imports
//! the namespace, the derive and the trait at once. Beside it: the [`Schema`] trait, its row
//! table ([`FieldRow`]), the context a derived check reports to ([`SchemaCx`]), the grading of
//! a failed member both it and the reader kit apply ([`grade`]), and what
//! `#[derive(Components)]` emits for a goal row ([`GoalMeta`]) and per row
//! ([`ComponentRow`]).
//!
//! A module that uses the attributes imports the namespace by name; one that can see another
//! item named `schema` cannot use them. Inside this crate a module writes `use crate::schema;`.

mod component;
mod cx;
mod goal;
mod grading;
mod molang_leaves;
mod rows;
mod sides;
mod walk;

// The grammar's dispatcher macros (`__attr`, `__parse_attr`) and its helper re-exports come
// with the glob: facet's derive expands `schema::min` to `schema::__attr!(..)`. The derive
// macro `Schema` comes with it too; the trait of the same name is below.
#[doc(inline)]
pub use bedrock_addon_schema::*;

// hidden: emitted by the schema derives
#[doc(hidden)]
pub use component::find_row;
pub use component::shape_of;
pub use component::{
    ComponentRow, Shorthand, ShorthandRows, ShorthandShape, misshapen_shorthand_rows,
    shorthand_shape,
};
pub use cx::{SchemaCx, Verdict};
// hidden: emitted by the schema derives
#[doc(hidden)]
pub use goal::int_fallback;
pub use goal::{GoalMeta, VersionedParseFn};
pub use grading::{Grading, grade};
pub use rows::{Bound, BoundOp, FieldKind, FieldRow, Limit, Lit, UnknownRule};
// hidden: schema support the families' hand-written rows name; API not yet reviewed
#[doc(hidden)]
pub use sides::{Side, Sides};
// hidden: emitted by the schema derives; hand-written rows name it too
#[doc(hidden)]
pub use walk::{Number, SchemaFields, member};
pub use walk::{Schema, UnknownName};
