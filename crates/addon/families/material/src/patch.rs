//! The deep merge the client runs over every pack's copy of a `.material` file before it reads
//! the result: members merge by their name before the first `:`.

use std::collections::{HashMap, HashSet};

use facet_value::VObject;

use super::file::base_name;
use bedrock_addon_core::diagnostic::{JsonPath, JsonPathSegment};
use bedrock_addon_core::json::Value;

/// `patch` laid over `base`, as the client merges a higher pack's copy over the lower ones.
///
/// - Two objects: the patch's members, then each member of the base. A base key is compared
///   with the keys of the result on the text before the first `:`; on a match the result's key
///   (the patch's spelling, with its own `:parent` or none) receives the merge of the two
///   values, and without one the base member is added under its own key.
/// - Two arrays: the patch's elements, then the base's elements past the patch's length. An
///   override by index that keeps the lower tail, not a concatenation.
/// - A `null` patch: the base. Anything else: the patch.
#[cfg(test)]
pub(super) fn patch_json(base: Value, patch: Value) -> Value {
    patch_with(base, patch, &mut |_, _| {})
}

/// [`patch_json`], telling `rekeyed(lower key, upper key)` of each member of the two root
/// objects that the patch takes over under a key spelt differently.
#[cfg(test)]
pub(super) fn patch_with(base: Value, patch: Value, rekeyed: &mut dyn FnMut(&str, &str)) -> Value {
    let mut writers = Writers::default();
    let (lower, upper) = (writers.add(0), writers.add(1));
    let (below, above) = (Trace::Whole(lower), Trace::Whole(upper));
    patch_traced(base, below, patch, above, rekeyed, &mut writers).0
}

/// The writers a [`Trace`] names: a copy, or a part of a copy that the merge keeps apart. Two
/// keys of one name in one copy are merged into one member; the members of each still have
/// their own key above them, so the second key's part of the copy is a writer of its own.
#[derive(Debug, Default)]
pub(super) struct Writers(Vec<u32>);

impl Writers {
    /// A new writer that stands for the copy `copy`.
    pub(super) fn add(&mut self, copy: u32) -> u32 {
        let writer = u32::try_from(self.0.len()).unwrap_or(u32::MAX);
        self.0.push(copy);
        writer
    }

    /// The copy the writer `writer` stands for.
    pub(super) fn copy_of(&self, writer: u32) -> Option<u32> {
        self.0.get(usize::try_from(writer).ok()?).copied()
    }
}

/// Which writer (see [`Writers`]) wrote each part of a merged value, and under which keys: the
/// shape of the merged value as far down as two copies were merged into each other.
#[derive(Debug)]
pub(super) enum Trace {
    /// One copy wrote the value and everything in it, under the keys the merged value has.
    Whole(u32),
    /// An object two copies or more have: its members each with their own trace.
    Object {
        /// The highest copy that has the object.
        top: u32,
        /// The members by their key in the merged value.
        members: HashMap<Box<str>, Member>,
    },
    /// An array two copies or more have. Its elements are not merged with each other: each
    /// is one copy's.
    Array {
        /// The highest copy that has the array.
        top: u32,
        /// (the index the range ends before, the copy that wrote the elements up to it),
        /// from the first element on.
        ranges: Vec<(usize, u32)>,
    },
}

/// One member of a merged object.
#[derive(Debug)]
pub(super) struct Member {
    /// Who wrote its value.
    trace: Trace,
    /// The key each writer that has the member writes it under, in no order; a writer is
    /// here once.
    spelt: Vec<(u32, Box<str>)>,
}

impl Trace {
    /// The highest copy that has the value.
    fn top(&self) -> u32 {
        match self {
            Self::Whole(copy) => *copy,
            Self::Object { top, .. } | Self::Array { top, .. } => *top,
        }
    }

    /// The members of the object `object` this is the trace of.
    fn into_members(self, object: &VObject) -> HashMap<Box<str>, Member> {
        if let Self::Object { members, .. } = self {
            return members;
        }
        let copy = self.top();
        let member = |key: &facet_value::VString| {
            let member = Member {
                trace: Self::Whole(copy),
                spelt: vec![(copy, key.as_str().into())],
            };
            (Box::from(key.as_str()), member)
        };
        object.keys().map(member).collect()
    }

    /// Gives every writer of this trace a new writer for the same copy; `renamed` holds the
    /// ones already given.
    fn renamed(&mut self, renamed: &mut HashMap<u32, u32>, writers: &mut Writers) {
        let mut rename = |writer: &mut u32| {
            let copy = writers.copy_of(*writer).unwrap_or(*writer);
            *writer = *renamed.entry(*writer).or_insert_with(|| writers.add(copy));
        };
        match self {
            Self::Whole(writer) => rename(writer),
            Self::Array { top, ranges } => {
                rename(top);
                for (_, writer) in ranges {
                    rename(writer);
                }
            }
            Self::Object { top, members } => {
                rename(top);
                for member in members.values_mut() {
                    for (writer, _) in &mut member.spelt {
                        rename(writer);
                    }
                }
                for member in members.values_mut() {
                    member.trace.renamed(renamed, writers);
                }
            }
        }
    }

    /// The ranges of the array of `len` elements this is the trace of.
    fn into_ranges(self, len: usize) -> Vec<(usize, u32)> {
        match self {
            Self::Array { ranges, .. } => ranges,
            other => vec![(len, other.top())],
        }
    }

    /// The writer that wrote the value at `path` of the merged value, and the path the value
    /// has in its copy: the same steps, each key as that writer spells it. A path the merged
    /// value does not have belongs to the writer of the deepest part of it that is there.
    pub(super) fn written(&self, path: &JsonPath) -> (u32, JsonPath) {
        let mut at = Some(self);
        let mut owner = self.top();
        let mut steps = Vec::with_capacity(path.len());
        for segment in path.segments() {
            let mut member = None;
            at = match (at, segment) {
                (Some(Self::Object { members, .. }), JsonPathSegment::Key(key)) => {
                    member = members.get(key);
                    member.map(|member| {
                        owner = member.trace.top();
                        &member.trace
                    })
                }
                (Some(Self::Array { ranges, .. }), JsonPathSegment::Index(index)) => {
                    if let Some((_, copy)) = ranges.iter().find(|(upto, _)| index < *upto) {
                        owner = *copy;
                    }
                    None
                }
                // Inside what one copy wrote: the path below is that copy's.
                _ => None,
            };
            steps.push((segment, member));
        }
        let mut placed = JsonPath::new();
        for (segment, member) in steps {
            match segment {
                JsonPathSegment::Index(index) => placed.push_index(index),
                JsonPathSegment::Key(key) => {
                    let spelt =
                        member.and_then(|member| member.spelt.iter().find(|(by, _)| *by == owner));
                    placed.push(spelt.map_or(key, |(_, spelt)| spelt));
                }
            }
        }
        (owner, placed)
    }
}

/// [`patch_with`] over two values whose writers `base_trace` and `patch_trace` name: the
/// merged value with the trace of who wrote what of it.
pub(super) fn patch_traced(
    base: Value,
    base_trace: Trace,
    patch: Value,
    patch_trace: Trace,
    rekeyed: &mut dyn FnMut(&str, &str),
    writers: &mut Writers,
) -> (Value, Trace) {
    if patch.is_null() {
        return (base, base_trace);
    }
    if let (Some(lower), Some(upper)) = (base.as_object(), patch.as_object()) {
        let top = patch_trace.top();
        let below = base_trace.top();
        let mut lower_members = base_trace.into_members(lower);
        let mut members = patch_trace.into_members(upper);
        let mut merged = upper.clone();
        // The first key of each name, as the client's sorted map meets them: the bytewise
        // smallest.
        let mut first: HashMap<&str, &str> = HashMap::with_capacity(upper.len());
        for (key, _) in upper {
            let key = key.as_str();
            let slot = first.entry(base_name(key)).or_insert(key);
            if key < *slot {
                *slot = key;
            }
        }
        let mut lower_keys: Vec<&str> = lower.keys().map(facet_value::VString::as_str).collect();
        lower_keys.sort_unstable();
        // The keys of the result that hold a member of the base already.
        let mut holds_lower: HashSet<&str> = HashSet::new();
        for key in lower_keys {
            let Some(value) = lower.get(key) else {
                continue;
            };
            let mut lower_member = lower_members.remove(key).unwrap_or_else(|| Member {
                trace: Trace::Whole(below),
                spelt: vec![(below, key.into())],
            });
            if let Some(taken) = first.get(base_name(key)).copied() {
                if !holds_lower.insert(taken) {
                    // A second key of the base with this name: what it wrote keeps its own
                    // key above it, as a writer apart from the first key's.
                    let mut renamed = HashMap::new();
                    for (writer, _) in &mut lower_member.spelt {
                        let copy = writers.copy_of(*writer).unwrap_or(*writer);
                        *writer = *renamed.entry(*writer).or_insert_with(|| writers.add(copy));
                    }
                    lower_member.trace.renamed(&mut renamed, writers);
                }
                if let Some(slot) = merged.get_mut(taken) {
                    let over = std::mem::replace(slot, Value::NULL);
                    let (over_trace, over_spelt) = match members.remove(taken) {
                        Some(member) => (member.trace, member.spelt),
                        None => (Trace::Whole(top), Vec::new()),
                    };
                    let (value, trace) = patch_traced(
                        value.clone(),
                        lower_member.trace,
                        over,
                        over_trace,
                        &mut |_, _| {},
                        writers,
                    );
                    *slot = value;
                    // The shorter list onto the longer: a name a copy writes many times
                    // over gathers its keys without copying them once per key.
                    let (mut spelt, more) = if lower_member.spelt.len() < over_spelt.len() {
                        (over_spelt, lower_member.spelt)
                    } else {
                        (lower_member.spelt, over_spelt)
                    };
                    spelt.extend(more);
                    members.insert(taken.into(), Member { trace, spelt });
                }
                // Told only of a key of the patch: two lower keys of one name merge into each
                // other without the patch having a say.
                if taken != key && upper.contains_key(taken) {
                    rekeyed(key, taken);
                }
            } else {
                merged.insert(key, value.clone());
                members.insert(key.into(), lower_member);
                first.insert(base_name(key), key);
                holds_lower.insert(key);
            }
        }
        return (merged.into_value(), Trace::Object { top, members });
    }
    if let (Some(lower), Some(upper)) = (base.as_array(), patch.as_array()) {
        let top = patch_trace.top();
        let mut merged = upper.clone();
        for value in lower.as_slice().iter().skip(upper.len()) {
            merged.push(value.clone());
        }
        let mut ranges = patch_trace.into_ranges(upper.len());
        let tail = base_trace.into_ranges(lower.len());
        ranges.extend(tail.into_iter().filter(|(upto, _)| *upto > upper.len()));
        return (merged.into_value(), Trace::Array { top, ranges });
    }
    (patch, patch_trace)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(text: &str) -> Value {
        bedrock_addon_core::json::from_str::<Value>(text).unwrap()
    }

    fn patched(base: &str, patch: &str) -> Value {
        patch_json(value(base), value(patch))
    }

    #[test]
    fn objects_merge_member_by_member_and_keep_the_lower_members() {
        assert_eq!(
            patched(
                r#"{"a": 1, "b": {"x": 1, "y": 2}}"#,
                r#"{"b": {"x": 9}, "c": 3}"#
            ),
            value(r#"{"b": {"x": 9, "y": 2}, "c": 3, "a": 1}"#)
        );
    }

    #[test]
    fn keys_match_on_the_text_before_the_first_colon_and_the_patch_key_is_kept() {
        // The pack's key wins: its own parent, or none.
        assert_eq!(
            patched(
                r#"{"dolphin": {"a": 1}}"#,
                r#"{"dolphin:entity": {"b": 2}}"#
            ),
            value(r#"{"dolphin:entity": {"b": 2, "a": 1}}"#)
        );
        assert_eq!(
            patched(r#"{"x:p": {"a": 1}}"#, r#"{"x": {"b": 2}}"#),
            value(r#"{"x": {"b": 2, "a": 1}}"#)
        );
        // The first colon, not the last.
        assert_eq!(
            patched(r#"{"a:b:c": {"m": 1}}"#, r#"{"a:z": {"n": 2}}"#),
            value(r#"{"a:z": {"n": 2, "m": 1}}"#)
        );
        // A different name before the colon is another member.
        assert_eq!(
            patched(r#"{"ab:p": 1}"#, r#"{"a:p": 2}"#),
            value(r#"{"a:p": 2, "ab:p": 1}"#)
        );
    }

    #[test]
    fn the_colon_rule_holds_at_every_depth() {
        assert_eq!(
            patched(
                r#"{"m": {"k:one": {"a": 1}}}"#,
                r#"{"m": {"k:two": {"b": 2}}}"#
            ),
            value(r#"{"m": {"k:two": {"b": 2, "a": 1}}}"#)
        );
    }

    #[test]
    fn only_the_first_matching_key_of_the_patch_is_merged() {
        // Of two patch keys with one name, the bytewise first takes the lower member.
        assert_eq!(
            patched(r#"{"a": {"m": 1}}"#, r#"{"a:c": {}, "a:b": {}}"#),
            value(r#"{"a:c": {}, "a:b": {"m": 1}}"#)
        );
    }

    #[test]
    fn two_lower_keys_with_one_name_merge_into_each_other() {
        // The second lower key finds the first, which is by then a key of the result.
        assert_eq!(
            patched(r#"{"a:b": {"m": 1}, "a:c": {"n": 2}}"#, "{}"),
            value(r#"{"a:b": {"m": 1, "n": 2}}"#)
        );
    }

    #[test]
    fn arrays_keep_the_patch_and_the_lower_tail() {
        assert_eq!(
            patched(r#"["a","b","c"]"#, r#"["m"]"#),
            value(r#"["m","b","c"]"#)
        );
        assert_eq!(patched(r#"["a"]"#, r#"["m","n"]"#), value(r#"["m","n"]"#));
        assert_eq!(
            patched(r#"["a","b"]"#, r#"["m","n"]"#),
            value(r#"["m","n"]"#)
        );
        // Elements are not merged with each other.
        assert_eq!(
            patched(r#"[{"p": 1, "q": 2}]"#, r#"[{"p": 3}]"#),
            value(r#"[{"p": 3}]"#)
        );
    }

    #[test]
    fn a_null_patch_keeps_the_base_and_anything_else_replaces_it() {
        assert_eq!(patched(r#"{"a": 1}"#, "null"), value(r#"{"a": 1}"#));
        assert_eq!(
            patched(r#"{"a": 1}"#, r#"{"a": null}"#),
            value(r#"{"a": 1}"#)
        );
        assert_eq!(patched(r#"["a","b"]"#, r#""m""#), value(r#""m""#));
        assert_eq!(patched(r#"{"a": 1}"#, "[]"), value("[]"));
        assert_eq!(patched("null", r#"{"a": 1}"#), value(r#"{"a": 1}"#));
    }

    #[test]
    fn a_root_member_taken_over_under_another_key_is_told() {
        let mut told = Vec::new();
        let merged = patch_with(
            value(r#"{"x:p": {"a": 1}, "y:p": {}, "z": {"k:one": 1}}"#),
            value(r#"{"x": {}, "y:p": {}, "z": {"k:two": 2}}"#),
            &mut |lower, upper| told.push((lower.to_owned(), upper.to_owned())),
        );
        // Two lower keys of one name merge into the first without being told: no key of the
        // patch took them over.
        let mut told_lower = Vec::new();
        patch_with(
            value(r#"{"x:p": {}, "x:q": {}}"#),
            value(r#"{"y": {}}"#),
            &mut |lower, upper| told_lower.push((lower.to_owned(), upper.to_owned())),
        );
        assert_eq!(told_lower, []);
        // Only the root's members, and only where the spelling differs.
        assert_eq!(told, [("x:p".to_owned(), "x".to_owned())]);
        assert_eq!(
            merged,
            value(r#"{"x": {"a": 1}, "y:p": {}, "z": {"k:two": 2}}"#)
        );
    }

    /// The copies merged lowest first, with the trace of the result and its writers.
    fn traced(copies: &[&str]) -> (Value, Trace, Writers) {
        let mut writers = Writers::default();
        let mut merged = (Value::NULL, Trace::Whole(0));
        for (index, copy) in copies.iter().enumerate() {
            let by = Trace::Whole(writers.add(u32::try_from(index).unwrap()));
            let (value, trace) = (merged.0, merged.1);
            merged = patch_traced(
                value,
                trace,
                self::value(copy),
                by,
                &mut |_, _| {},
                &mut writers,
            );
        }
        (merged.0, merged.1, writers)
    }

    fn path(steps: &[&str]) -> JsonPath {
        let mut path = JsonPath::new();
        for step in steps {
            match step.parse::<usize>() {
                Ok(index) => path.push_index(index),
                Err(_) => path.push(step),
            }
        }
        path
    }

    #[track_caller]
    fn assert_written(trace: &(Trace, Writers), at: &[&str], by: u32, there: &str) {
        let (writer, placed) = trace.0.written(&path(at));
        let copy = trace.1.copy_of(writer).unwrap();
        assert_eq!((copy, placed.to_string().as_str()), (by, there), "{at:?}");
    }

    #[test]
    fn the_trace_names_the_copy_that_wrote_each_member_under_its_own_key() {
        let (merged, trace, writers) = traced(&[
            r#"{"x:p": {"a": 1, "f:one": {"m": 1, "n": 1}, "l": [1, 2, 3]}, "low": {"k": 1}}"#,
            r#"{"x:q": {"b": 2, "f:two": {"n": 2}, "l": [9]}}"#,
            r#"{"x": {"c": 3, "f": {"o": 3}}, "top": 1}"#,
        ]);
        assert_eq!(
            merged,
            value(
                r#"{"x": {"c": 3, "f": {"o": 3, "n": 2, "m": 1}, "b": 2, "l": [9, 2, 3], "a": 1},
                    "top": 1, "low": {"k": 1}}"#
            )
        );
        let trace = (trace, writers);
        // Each member by the copy that wrote it, at that copy's spelling of every key above.
        assert_written(&trace, &["x", "c"], 2, "x/c");
        assert_written(&trace, &["x", "b"], 1, "x:q/b");
        assert_written(&trace, &["x", "a"], 0, "x:p/a");
        assert_written(&trace, &["x", "f", "o"], 2, "x/f/o");
        assert_written(&trace, &["x", "f", "n"], 1, "x:q/f:two/n");
        assert_written(&trace, &["x", "f", "m"], 0, "x:p/f:one/m");
        // An object is the highest copy's that has it; an array too.
        assert_written(&trace, &["x"], 2, "x");
        assert_written(&trace, &["x", "f"], 2, "x/f");
        assert_written(&trace, &["x", "l"], 1, "x:q/l");
        // An array's elements: the patch's, then the lower tail.
        assert_written(&trace, &["x", "l", "0"], 1, "x:q/l/0");
        assert_written(&trace, &["x", "l", "2"], 0, "x:p/l/2");
        // What one copy wrote alone, at any depth below it.
        assert_written(&trace, &["low", "k"], 0, "low/k");
        assert_written(&trace, &["top"], 2, "top");
        // A member that is not there belongs to the object it would be in.
        assert_written(&trace, &["x", "f", "gone", "deeper"], 2, "x/f/gone/deeper");
        assert_written(&trace, &["nowhere"], 2, "nowhere");
        assert_written(&trace, &[], 2, "");
    }

    #[test]
    fn a_null_member_of_the_patch_leaves_the_member_to_the_copy_below() {
        let (_, trace, writers) = traced(&[r#"{"x:p": {"a": 1}}"#, r#"{"x": null}"#]);
        let trace = (trace, writers);
        assert_written(&trace, &["x"], 0, "x:p");
        assert_written(&trace, &["x", "a"], 0, "x:p/a");
        // One copy alone: everything is its own.
        let (_, trace, writers) = traced(&[r#"{"x": {"a": [1]}}"#]);
        assert_written(&(trace, writers), &["x", "a", "0"], 0, "x/a/0");
    }

    #[test]
    fn two_keys_of_one_name_in_one_copy_keep_their_own_key_above_what_each_wrote() {
        // The merge folds the copy's second key into its first once a copy is laid over it:
        // each member is still named under the key it was written under.
        let (merged, trace, writers) = traced(&[
            r#"{"x:p": {"a": 1, "f": {"m": 1}}, "x:q": {"b": 2, "f": {"n": 2}}, "x:r": {"c": 3}}"#,
            r#"{"y": {}}"#,
        ]);
        assert_eq!(
            merged,
            value(r#"{"y": {}, "x:p": {"a": 1, "f": {"m": 1, "n": 2}, "b": 2, "c": 3}}"#)
        );
        let trace = (trace, writers);
        assert_written(&trace, &["x:p", "a"], 0, "x:p/a");
        assert_written(&trace, &["x:p", "b"], 0, "x:q/b");
        assert_written(&trace, &["x:p", "c"], 0, "x:r/c");
        assert_written(&trace, &["x:p", "f", "m"], 0, "x:p/f/m");
        assert_written(&trace, &["x:p", "f", "n"], 0, "x:q/f/n");
        assert_written(&trace, &["x:p"], 0, "x:p");
        // A copy over them takes the folded member under its own key.
        let (_, trace, writers) = traced(&[
            r#"{"x:p": {"a": 1}, "x:q": {"b": 2}}"#,
            r#"{"x": {"c": 3}}"#,
        ]);
        let trace = (trace, writers);
        assert_written(&trace, &["x", "a"], 0, "x:p/a");
        assert_written(&trace, &["x", "b"], 0, "x:q/b");
        assert_written(&trace, &["x", "c"], 1, "x/c");
    }
}
