//! The keep step: which unknown members of a walk are kept in a document's `Unknown` table.

use facet::{Facet, Peek};

use super::{Misread, Step, Table, Unknown, Value, Walk};
use crate::diagnostic::JsonPath;
use crate::schema::UnknownRule;

/// [`unknown_members`](super::unknown_members) for `target`, which was deserialised from the same text as `json`, and
/// keeps each unknown member of a struct marked with an `extra` field in `kept`, by the path
/// of its object from the root of `json` (replacing a member of the same key). Enums are
/// walked through the variant `target` holds. Returns every unknown member, kept or not: one
/// inside a map value or behind a pointer (`Box`) is reported but not kept, as the walk once
/// stored them in the struct and could not reach those mutably; neither is one in a struct
/// without an `extra` field.
pub fn keep_unknown_members<'f, T: Facet<'f>>(
    target: &T,
    json: &Value,
    kept: &mut Table,
) -> Vec<(JsonPath, Value)> {
    keep_unknown_members_with(target, json, false, kept)
}

/// [`keep_unknown_members`] under a reading: when `keeps_unknown`
/// (`Reading::keeps_unknown`), a struct that says nothing keeps its unknown members too, as the
/// door decides for the slice (a struct's `extra` mark or its `schema::unknown` word decides
/// first). For a hand reader that reads a member of its slice by facet outside the door: it
/// keeps what its reading says, with no `schema::unknown = "keep"` on each struct.
pub fn keep_unknown_members_with<'f, T: Facet<'f>>(
    target: &T,
    json: &Value,
    keeps_unknown: bool,
    kept: &mut Table,
) -> Vec<(JsonPath, Value)> {
    // The misread scalars are not asked for: the walk does not look for them.
    let walked = Walk::of(target, json).run();
    members(keep(
        walked.unknowns,
        Peek::new(target),
        keeps_unknown,
        kept,
    ))
}

/// [`keep_unknown_members`] for the door (`parse_payload`, `parse_component`): under a reading
/// that keeps unknown members when `keeps_unknown`, with each member's value left out and its
/// disposition beside it: [`UnknownRule::Keep`] for one put in `kept`, else what its struct
/// says (`Report` where it says nothing, or keeps it where the member cannot be kept).
pub(crate) fn door_unknown_members<'f, T: Facet<'f>>(
    target: &T,
    json: &Value,
    keeps_unknown: bool,
    kept: &mut Table,
) -> Vec<(JsonPath, UnknownRule)> {
    let walked = Walk::of(target, json).run();
    keep(walked.unknowns, Peek::new(target), keeps_unknown, kept)
        .into_iter()
        .map(|((path, _), rule)| (path, rule))
        .collect()
}

/// [`keep_unknown_members`], and with the unknown members every scalar of `json` that its
/// member of `target` read as another value than the document writes ([`Misread`]).
pub fn keep_members<'f, T: Facet<'f>>(
    target: &T,
    json: &Value,
    kept: &mut Table,
) -> (Vec<(JsonPath, Value)>, Vec<Misread>) {
    let walked = Walk::of(target, json).misreads(true).run();
    let unknowns = keep(walked.unknowns, Peek::new(target), false, kept);
    (members(unknowns), walked.misread)
}

/// [`keep_members`], and among the unknown members those of a [`Range`](crate::shared::Range)
/// given as an object that the range does not read (`{"a": 1}`, a `min` beside a
/// `range_min`): reported, not kept, since a range has no `extra`.
pub fn keep_members_and_range_members<'f, T: Facet<'f>>(
    target: &T,
    json: &Value,
    kept: &mut Table,
) -> (Vec<(JsonPath, Value)>, Vec<Misread>) {
    let walked = Walk::of(target, json).ranges(true).misreads(true).run();
    let unknowns = keep(walked.unknowns, Peek::new(target), false, kept);
    (members(unknowns), walked.misread)
}

/// One unknown member as the walk returns it, its disposition beside it.
type Told = ((JsonPath, Value), UnknownRule);

/// The members of `told`, their dispositions left out.
fn members(told: Vec<Told>) -> Vec<(JsonPath, Value)> {
    told.into_iter().map(|(member, _)| member).collect()
}

/// The keep step over the unknown members a walk found in `target`'s value: each of a struct
/// that [`keeps`] (`keeps_unknown` the reading's rule) and that its route reaches in `target`
/// is put in `kept`, by the path of its object, and told [`UnknownRule::Keep`]; every other
/// one is told its struct's word (`Report` for none, and for a struct that keeps behind a step
/// the table cannot follow).
fn keep(
    unknowns: Vec<Unknown>,
    target: Peek<'_, '_>,
    keeps_unknown: bool,
    kept: &mut Table,
) -> Vec<Told> {
    unknowns
        .into_iter()
        .map(|unknown| {
            let keeps = keeps(unknown.has_extra, unknown.rule, keeps_unknown);
            let mut told = if keeps {
                UnknownRule::Keep
            } else {
                unknown.rule.unwrap_or(UnknownRule::Report)
            };
            if let (true, Some(route), Some(key)) = (keeps, &unknown.route, &unknown.key)
                && reachable(target, route)
            {
                let mut owner = unknown.path.clone();
                owner.pop();
                kept.put(&owner, key, unknown.value.clone());
            } else if told == UnknownRule::Keep {
                // A struct that keeps, behind a step the table cannot follow: told as one
                // that says nothing.
                told = UnknownRule::Report;
            }
            ((unknown.path, unknown.value), told)
        })
        .collect()
}

/// Whether a struct keeps its unknown members in the document's table: it is marked with an
/// `extra` field (`has_extra`), or its word (`rule`, the field's that holds it before its own)
/// is `keep`, or it says nothing and the reading keeps them (`reading_keeps`,
/// `Reading::keeps_unknown`). `report`, `reject` and `ignore` keep nothing but under the mark.
pub(super) fn keeps(has_extra: bool, rule: Option<UnknownRule>, reading_keeps: bool) -> bool {
    has_extra
        || match rule {
            Some(UnknownRule::Keep) => true,
            Some(_) => false,
            None => reading_keeps,
        }
}

/// Whether `route` leads from `peek` to a value: every struct field, active variant field,
/// `Some` and list element on it is there. The member is kept only then, as when the walk
/// stored it in the struct itself.
fn reachable(peek: Peek<'_, '_>, route: &[Step]) -> bool {
    let Some((step, rest)) = route.split_first() else {
        return true;
    };
    let next = match *step {
        Step::Field(index) => peek.into_struct().ok().and_then(|st| st.field(index).ok()),
        Step::VariantField(variant, index) => peek
            .into_enum()
            .ok()
            .filter(|en| en.variant_index() == Ok(variant))
            .and_then(|en| en.field(index).ok().flatten()),
        Step::Some => peek.into_option().ok().and_then(facet::PeekOption::value),
        Step::Index(index) => peek.into_list().ok().and_then(|list| list.get(index)),
    };
    next.is_some_and(|next| reachable(next, rest))
}

#[cfg(test)]
mod tests {
    use facet::Facet;

    use super::*;
    use crate::json::Value;

    /// The keep decision, row by row: (mark, word, reading keeps) -> kept.
    #[test]
    fn a_struct_keeps_by_its_mark_its_word_or_else_the_reading() {
        use UnknownRule::{Ignore, Keep, Reject, Report};
        let rows = [
            ((true, None, false), true),
            ((true, Some(Report), false), true),
            ((false, Some(Keep), false), true),
            ((false, None, true), true),
            ((false, Some(Report), true), false),
            ((false, Some(Reject), true), false),
            ((false, Some(Ignore), true), false),
            ((false, None, false), false),
        ];
        for ((mark, word, reading), kept) in rows {
            assert_eq!(
                keeps(mark, word, reading),
                kept,
                "{mark} {word:?} {reading}"
            );
        }
    }

    /// A struct that says nothing keeps its unknown members under a reading that keeps them,
    /// and one that says `report` does not.
    #[test]
    fn a_hand_reader_keeps_by_its_reading() {
        use crate::schema;
        #[derive(Debug, Facet)]
        struct Silent {
            #[facet(default)]
            a: i32,
            #[facet(default)]
            told: Option<Told>,
        }
        #[derive(Debug, Facet)]
        #[facet(schema::unknown = "report")]
        struct Told {
            #[facet(default)]
            b: i32,
        }
        let json: Value =
            crate::json::from_str_jsonc(r#"{"a": 1, "x": 2, "told": {"b": 3, "y": 4}}"#).unwrap();
        let value: Silent = crate::json::from_str_jsonc(r#"{"a": 1, "told": {"b": 3}}"#).unwrap();
        let root = JsonPath::new();
        let mut kept = Table::new();
        let found = keep_unknown_members_with(&value, &json, true, &mut kept);
        assert_eq!(found.len(), 2);
        assert_eq!(kept.get(&root, "x"), Some(&Value::from(2)));
        assert_eq!(kept.get(&root.child("told"), "y"), None);
        let mut kept = Table::new();
        assert_eq!(
            keep_unknown_members_with(&value, &json, false, &mut kept).len(),
            2
        );
        assert!(kept.is_empty());
    }
}
