//! What a goal row of `#[derive(Components)]` emits for its goal type: [`GoalMeta`], and the
//! goal parse table's function type.

use crate::diagnostic::Diagnostics;
use crate::json::JsonSlice;
use crate::load::LoadContext;
use crate::version::FormatVersion;

/// The two facts of a goal the component row states: its typed version (`typed = ".."` on the
/// row) and its `priority`, which the goal's `Schema` getter answers (`schema::fallback` on
/// the goal's `priority` field). Implemented by `#[derive(Components)]` for each `goal` row.
pub trait GoalMeta {
    /// The format version from which the game reads the goal with its typed struct.
    const TYPED: FormatVersion;

    /// The goal's priority, its fallback where the document has none.
    fn priority(&self) -> i32;
}

/// The parse of a goal by the document's format version, `C` the family's component enum: what
/// a `goal` row puts in the enum's `VERSIONED` table (`versioned::<T>` of the enum's
/// `versioned = path`).
pub type VersionedParseFn<C> =
    fn(JsonSlice<'_>, FormatVersion, &LoadContext, &mut Diagnostics) -> Option<C>;

/// The `schema::fallback` integer of the field `name` among `fields`: what a goal row's
/// `GoalMeta::priority` answers for an absent `priority`. A `const fn`, so a goal struct whose
/// `priority` declares no integer fallback fails to compile where the row's impl is emitted.
// hidden: emitted by the schema derives
#[doc(hidden)]
pub const fn int_fallback(fields: &[super::FieldRow], name: &str) -> i64 {
    let mut index = 0;
    while index < fields.len() {
        if same(fields[index].name, name) {
            if let Some(super::Lit::Int(value)) = fields[index].fallback {
                return value;
            }
            panic!("a goal's `priority` field declares no `schema::fallback = <integer>`");
        }
        index += 1;
    }
    panic!("a goal struct has no `priority` field in its `Schema::FIELDS`");
}

/// `a == b`, in a `const fn`.
const fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut index = 0;
    while index < a.len() {
        if a[index] != b[index] {
            return false;
        }
        index += 1;
    }
    true
}
