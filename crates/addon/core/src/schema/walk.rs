//! The `Schema` trait `#[derive(Schema)]` implements, and `SchemaFields`, the walk into a
//! field's type that its `check` and its trigger list recurse through.

use std::collections::{BTreeMap, HashMap};

use indexmap::IndexMap;

use super::cx::SchemaCx;
use super::rows::{FieldRow, UnknownRule};

/// A typed document struct (or string enum) whose per-field facts are stated as attributes:
/// implemented by `#[derive(Schema)]`, never by hand.
///
/// `FIELDS` is the row table, to be tested against the generated game tables; `check` applies
/// the bounds under the slice's rule, lower-cases the `lowercase` members (in silence, as the
/// game does) and walks into every field;
/// `trigger_members` lists the members that hold a trigger of this entity.
pub trait Schema {
    /// The fields in declaration order, `skip` fields left out (empty for a string enum).
    const FIELDS: &'static [FieldRow];

    /// The container's `schema::unknown`.
    const UNKNOWN: Option<UnknownRule> = None;

    /// The bounds of every field, at the members' paths below `cx`'s, and the ASCII
    /// lower-casing of each `lowercase` one, then the same for what each field holds.
    fn check(&mut self, cx: &mut SchemaCx<'_>);

    /// Appends the trigger members below `prefix` (keys joined by `.`, `[]` after a list, `*`
    /// for the values of a map; the empty pattern is the value itself): every `EventTrigger`
    /// field without `schema::foreign_event`, at any depth.
    fn trigger_members(prefix: &str, out: &mut Vec<String>);

    /// The trigger members of the type, from its root.
    fn triggers() -> Vec<String>
    where
        Self: Sized,
    {
        let mut out = Vec::new();
        Self::trigger_members("", &mut out);
        out
    }
}

/// The walk of a derived `check` and trigger list into a field's type: every `Schema` type, the
/// leaf types (numbers, `bool`, `char`, `String`, `json::Value`, the Molang values; what a family names with
/// `impl_schema_leaf!`), identifiers (`Id<K>`), `EventTrigger` (a trigger at its own path), and
/// `Option`, `Vec`, fixed-length arrays and `IndexMap<String, _>` of them.
// hidden: emitted by the schema derives
#[doc(hidden)]
pub trait SchemaFields {
    /// `Schema::check`, or nothing for a leaf.
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>);

    /// `Schema::trigger_members`, or nothing for a leaf.
    fn trigger_members(prefix: &str, out: &mut Vec<String>);
}

impl<T: Schema> SchemaFields for T {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        Schema::check(self, cx);
    }

    fn trigger_members(prefix: &str, out: &mut Vec<String>) {
        <T as Schema>::trigger_members(prefix, out);
    }
}

impl<T: SchemaFields> SchemaFields for Option<T> {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        if let Some(inner) = self {
            inner.check_fields(cx);
        }
    }

    fn trigger_members(prefix: &str, out: &mut Vec<String>) {
        T::trigger_members(prefix, out);
    }
}

impl<T: SchemaFields> SchemaFields for Vec<T> {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        for (index, inner) in self.iter_mut().enumerate() {
            cx.enter_index(index);
            inner.check_fields(cx);
            cx.leave();
        }
    }

    fn trigger_members(prefix: &str, out: &mut Vec<String>) {
        T::trigger_members(&format!("{prefix}[]"), out);
    }
}

/// A fixed-length array (`[x, y, z]`) walks its elements as a list does.
impl<T: SchemaFields, const N: usize> SchemaFields for [T; N] {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        for (index, inner) in self.iter_mut().enumerate() {
            cx.enter_index(index);
            inner.check_fields(cx);
            cx.leave();
        }
    }

    fn trigger_members(prefix: &str, out: &mut Vec<String>) {
        T::trigger_members(&format!("{prefix}[]"), out);
    }
}

/// An identifier is a leaf: its syntax is the identifier's own read, and it holds no trigger.
impl<K: crate::id::Kind> SchemaFields for crate::id::Id<K> {
    fn check_fields(&mut self, _: &mut SchemaCx<'_>) {}

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}

// The shared value types a document holds whole: each reads its own syntax and holds no
// bound the derive checks and no trigger.
crate::impl_schema_leaf!(crate::shared::Filter, crate::shared::FilterSubject);

impl<T: crate::shared::RangeEnd> SchemaFields for crate::shared::Range<T> {
    fn check_fields(&mut self, _: &mut SchemaCx<'_>) {}

    fn trigger_members(_: &str, _: &mut Vec<String>) {}
}

/// A boxed `Schema` type is that type (a recursive struct holds itself in a `Box`). `Box` is
/// fundamental, so the walk reaches it through this impl and the blanket `SchemaFields` one,
/// not through an impl of its own.
impl<T: Schema> Schema for Box<T> {
    const FIELDS: &'static [FieldRow] = T::FIELDS;
    const UNKNOWN: Option<UnknownRule> = T::UNKNOWN;

    fn check(&mut self, cx: &mut SchemaCx<'_>) {
        T::check(self, cx);
    }

    fn trigger_members(prefix: &str, out: &mut Vec<String>) {
        T::trigger_members(prefix, out);
    }
}

/// The walk into each value of a map keyed by member name, at the member's path.
macro_rules! map_fields {
    ($($map:ident),+) => {
        $(
            impl<T: SchemaFields, S> SchemaFields for $map<String, T, S>
            where
                S: core::hash::BuildHasher,
            {
                fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
                    for (key, inner) in self.iter_mut() {
                        cx.enter(key);
                        inner.check_fields(cx);
                        cx.leave();
                    }
                }

                fn trigger_members(prefix: &str, out: &mut Vec<String>) {
                    T::trigger_members(&member(prefix, "*"), out);
                }
            }
        )+
    };
}

map_fields!(IndexMap, HashMap);

impl<T: SchemaFields> SchemaFields for BTreeMap<String, T> {
    fn check_fields(&mut self, cx: &mut SchemaCx<'_>) {
        for (key, inner) in self.iter_mut() {
            cx.enter(key);
            inner.check_fields(cx);
            cx.leave();
        }
    }

    fn trigger_members(prefix: &str, out: &mut Vec<String>) {
        T::trigger_members(&member(prefix, "*"), out);
    }
}

/// An event trigger is a trigger at its own path; its own findings are `EventTrigger::check`'s.
impl SchemaFields for crate::shared::EventTrigger {
    fn check_fields(&mut self, _: &mut SchemaCx<'_>) {}

    fn trigger_members(prefix: &str, out: &mut Vec<String>) {
        out.push(prefix.to_owned());
    }
}

/// `prefix.key`, or `key` at the root: the pattern of the member `key` below `prefix`.
// hidden: emitted by the schema derives
#[doc(hidden)]
pub fn member(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_owned()
    } else {
        format!("{prefix}.{key}")
    }
}

/// The empty `SchemaFields` impl of each type named: a type with no bound, no `lowercase` and
/// no trigger below it, which a `#[derive(Schema)]` struct holds without deriving `Schema` on
/// it.
///
/// ```
/// pub struct Opaque(pub u8);
///
/// bedrock_addon_core::impl_schema_leaf!(Opaque);
/// ```
// hidden: macro support: named by the expansion of the exported macro
#[doc(hidden)]
#[macro_export]
macro_rules! impl_schema_leaf {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::schema::SchemaFields for $ty {
                fn check_fields(&mut self, _: &mut $crate::schema::SchemaCx<'_>) {}

                fn trigger_members(_: &str, _: &mut ::std::vec::Vec<::std::string::String>) {}
            }
        )+
    };
}

crate::impl_schema_leaf!(
    i8,
    i16,
    i32,
    i64,
    isize,
    u8,
    u16,
    u32,
    u64,
    usize,
    f32,
    f64,
    bool,
    char,
    String,
    crate::json::Value,
    crate::molang::Molang,
    crate::molang::MolangSource,
    crate::molang::MolangList,
);

/// A number a bound compares: the derive converts the value and the limit with `to_f64` for
/// the finding's text.
// hidden: schema support the families' hand-written rows name; API not yet reviewed
#[doc(hidden)]
pub trait Number: PartialOrd + Copy {
    /// The value as an `f64` (rounded where the type is wider).
    fn to_f64(self) -> f64;
}

macro_rules! number {
    ($($ty:ty),+) => {
        $(
            impl Number for $ty {
                #[allow(clippy::cast_precision_loss, clippy::cast_lossless, reason = "the text of a finding")]
                fn to_f64(self) -> f64 {
                    self as f64
                }
            }
        )+
    };
}

number!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64);

/// A name a string enum does not know: the error of the derived `parse_name`.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("expected {expected}, found `{found}`")]
pub struct UnknownName {
    /// What the enum takes, in words (`schema::expected`, or its names listed).
    pub expected: &'static str,
    /// The name read.
    pub found: String,
}
