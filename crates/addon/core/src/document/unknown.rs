//! [`Unknown`]: the unknown members of one document, kept beside its typed value for
//! write-back.

use std::fmt;
use std::ops::Range;

use crate::diagnostic::{JsonPath, JsonPathSegment};
use crate::json::Value;

/// The unknown members of one document: the members no field of its typed value takes, each
/// by the path of the JSON object that holds it, kept so that a written document has them
/// again. Empty for most documents, and then it allocates nothing.
///
/// The read fills it: every member the door's walk finds unknown in an object whose struct
/// keeps them, and every member a hand reader keeps. The writer looks up the object it opens
/// by its path and writes that object's unknown members after its known ones, in document
/// order. An upgrade step that moves an object moves its members with it
/// ([`rebase`](Self::rebase), done for every recorded move by `Diagnostics::moved`); one
/// that consumes a member [`take`](Self::take)s it.
///
/// A typed value holds no unknown member, so its `PartialEq` and `Clone` leave them out; a
/// `Document`'s compare them. Two tables are equal when each object holds the same members in
/// the same order: the order is the document's, which a written document has again.
#[derive(Clone, Default, PartialEq)]
pub struct Unknown {
    /// One row per object that has unknown members, sorted by path for binary search.
    owners: Vec<Owner>,
    /// Every unknown member, grouped by owner in the owners' order; in document order inside
    /// a group.
    members: Vec<(Box<str>, Value)>,
}

/// One object that has unknown members.
#[derive(Clone, Debug, PartialEq)]
struct Owner {
    /// The object's path from the document root.
    path: JsonPath,
    /// Its members in [`Unknown::members`].
    members: Range<u32>,
}

/// One object's path with its members, in document order.
type Group = (JsonPath, Vec<(Box<str>, Value)>);

/// An index into the members as the owners' ranges hold it.
fn index(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl Unknown {
    /// The empty table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            owners: Vec::new(),
            members: Vec::new(),
        }
    }

    /// No unknown member.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// How many unknown members, at every path.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// The owner at `at`: `Ok` with its index, or `Err` with where it would go.
    fn find(&self, at: &JsonPath) -> Result<usize, usize> {
        self.owners.binary_search_by(|owner| owner.path.cmp(at))
    }

    /// The members of the owner at `index`.
    fn group(&self, index: usize) -> &[(Box<str>, Value)] {
        let range = &self.owners[index].members;
        &self.members[range.start as usize..range.end as usize]
    }

    /// The unknown members of the object at `at`, in document order.
    pub fn members_at(&self, at: &JsonPath) -> impl Iterator<Item = (&str, &Value)> + '_ {
        let group = self.find(at).map_or(&[][..], |index| self.group(index));
        group.iter().map(|(key, value)| (&**key, value))
    }

    /// The unknown member `key` of the object at `at`.
    pub fn get(&self, at: &JsonPath, key: &str) -> Option<&Value> {
        self.members_at(at)
            .find_map(|(name, value)| (name == key).then_some(value))
    }

    /// Whether the object at `at` has the unknown member `key`.
    pub fn contains(&self, at: &JsonPath, key: &str) -> bool {
        self.get(at, key).is_some()
    }

    /// The paths of the objects that have unknown members, sorted.
    pub fn owners(&self) -> impl Iterator<Item = &JsonPath> + '_ {
        self.owners.iter().map(|owner| &owner.path)
    }

    /// Every unknown member with the path of its object: the objects sorted by path, the
    /// members of each in document order.
    pub fn iter(&self) -> impl Iterator<Item = (&JsonPath, &str, &Value)> + '_ {
        self.owners.iter().flat_map(move |owner| {
            let members = &self.members[owner.members.start as usize..owner.members.end as usize];
            members
                .iter()
                .map(move |(key, value)| (&owner.path, &**key, value))
        })
    }

    /// Keeps `value` as the unknown member `key` of the object at `at`: in place of a member
    /// of that key, else after the object's other unknown members. Returns the replaced value.
    pub fn put(&mut self, at: &JsonPath, key: &str, value: Value) -> Option<Value> {
        let owner = match self.find(at) {
            Ok(owner) => owner,
            Err(place) => {
                let start = self
                    .owners
                    .get(place)
                    .map_or(index(self.members.len()), |next| next.members.start);
                self.owners.insert(
                    place,
                    Owner {
                        path: at.clone(),
                        members: start..start,
                    },
                );
                place
            }
        };
        let range = self.owners[owner].members.clone();
        let slot =
            (range.start as usize..range.end as usize).find(|&slot| &*self.members[slot].0 == key);
        if let Some(slot) = slot {
            return Some(std::mem::replace(&mut self.members[slot].1, value));
        }
        self.members.insert(range.end as usize, (key.into(), value));
        self.owners[owner].members.end += 1;
        for later in &mut self.owners[owner + 1..] {
            later.members.start += 1;
            later.members.end += 1;
        }
        None
    }

    /// Takes the unknown member `key` of the object at `at` out of the table: a step that
    /// reads it into a typed member, or drops it. An object left with none has no row.
    pub fn take(&mut self, at: &JsonPath, key: &str) -> Option<Value> {
        let owner = self.find(at).ok()?;
        let range = self.owners[owner].members.clone();
        let slot = (range.start as usize..range.end as usize)
            .find(|&slot| &*self.members[slot].0 == key)?;
        let (_, value) = self.members.remove(slot);
        self.owners[owner].members.end -= 1;
        for later in &mut self.owners[owner + 1..] {
            later.members.start -= 1;
            later.members.end -= 1;
        }
        if self.owners[owner].members.is_empty() {
            self.owners.remove(owner);
        }
        Some(value)
    }

    /// Every unknown member of the object at `at`, taken out of the table in document order.
    pub fn take_all(&mut self, at: &JsonPath) -> Vec<(Box<str>, Value)> {
        let Ok(owner) = self.find(at) else {
            return Vec::new();
        };
        let range = self.owners.remove(owner).members;
        let taken: Vec<_> = self
            .members
            .drain(range.start as usize..range.end as usize)
            .collect();
        let gone = index(taken.len());
        for later in &mut self.owners[owner..] {
            later.members.start -= gone;
            later.members.end -= gone;
        }
        taken
    }

    /// How many objects have unknown members (the writer's index space).
    pub(crate) fn owner_count(&self) -> usize {
        self.owners.len()
    }

    /// The path of the owner at `index`.
    pub(crate) fn owner_path(&self, index: usize) -> &JsonPath {
        &self.owners[index].path
    }

    /// The members of the owner at `index`, in document order.
    pub(crate) fn owner_members(&self, index: usize) -> &[(Box<str>, Value)] {
        self.group(index)
    }

    /// The rows, each object's path with its members, in the owners' order.
    fn into_groups(self) -> Vec<Group> {
        let mut members = self.members.into_iter();
        self.owners
            .into_iter()
            .map(|owner| {
                let count = (owner.members.end - owner.members.start) as usize;
                (owner.path, members.by_ref().take(count).collect())
            })
            .collect()
    }

    /// The table of `groups`, put one after the other ([`put`](Self::put)'s rule).
    fn of_groups(groups: impl IntoIterator<Item = Group>) -> Self {
        let mut table = Self::new();
        for (path, members) in groups {
            for (key, value) in members {
                table.put(&path, &key, value);
            }
        }
        table
    }

    /// Rebuilds the table with every row's path given by `to` (`None` drops the row): the rows
    /// whose path stays first, in the owners' order, then the moved ones, each member put by
    /// [`put`](Self::put)'s rule, so a moved member replaces a staying one of its key.
    fn remap(&mut self, mut to: impl FnMut(&JsonPath) -> Option<JsonPath>) {
        if self.is_empty() {
            return;
        }
        let groups = std::mem::take(self).into_groups();
        let (mut stayed, mut moved) = (Vec::new(), Vec::new());
        for (path, members) in groups {
            match to(&path) {
                Some(new) if new == path => stayed.push((path, members)),
                Some(new) => moved.push((new, members)),
                None => {}
            }
        }
        // A moved member replaces one of its key at the place it is moved to.
        *self = Self::of_groups(stayed.into_iter().chain(moved));
    }

    /// Moves the unknown members of the object at `from`, and of every object below it, to
    /// `to`: an upgrade step moved that object. A moved member replaces a member of the same
    /// key at the place it goes to.
    ///
    /// It moves objects, not members: an unknown member whose own path is `from` is a member
    /// of its parent's row and stays there; a step that renames one [`take`](Self::take)s and
    /// [`put`](Self::put)s it.
    pub fn rebase(&mut self, from: &JsonPath, to: &JsonPath) {
        self.remap(|path| {
            Some(
                path.strip_prefix(from)
                    .map_or_else(|| path.clone(), |rest| to.join(&rest)),
            )
        });
    }

    /// Copies the unknown members of the object at `from`, and of every object below it, to
    /// `to`, each by [`put`](Self::put)'s rule; those at `from` stay: a step that copies an
    /// object copies its members.
    #[doc(hidden)]
    pub fn copy(&mut self, from: &JsonPath, to: &JsonPath) {
        let copies: Vec<(JsonPath, Box<str>, Value)> = self
            .iter()
            .filter_map(|(path, key, value)| {
                let rest = path.strip_prefix(from)?;
                Some((to.join(&rest), key.into(), value.clone()))
            })
            .collect();
        for (path, key, value) in copies {
            self.put(&path, &key, value);
        }
    }

    /// Drops the unknown members of the object at `at` and of every object below it: the
    /// object itself is gone (a component dropped after its parse).
    pub fn forget(&mut self, at: &JsonPath) {
        self.remap(|path| (!path.starts_with(at)).then(|| path.clone()));
    }

    /// The list at `list` now holds, in order, the elements it had at the indices `kept`
    /// (ascending): the members of an element left out are dropped, those of an element kept
    /// go to its new index ([`Diagnostics::left_out`](crate::diagnostic::Diagnostics::left_out)).
    #[doc(hidden)]
    pub fn left_out(&mut self, list: &JsonPath, kept: &[usize]) {
        self.remap(|path| {
            let Some(rest) = path.strip_prefix(list) else {
                return Some(path.clone());
            };
            let mut segments = rest.segments();
            let Some(JsonPathSegment::Index(was)) = segments.next() else {
                return Some(path.clone());
            };
            let now = kept.iter().position(|&k| k == was)?;
            let below = rest.strip_prefix(&rest.prefix(1)).unwrap_or_default();
            Some(list.index(now).join(&below))
        });
    }

    /// Takes the unknown members of the object at `at` and of every object below it out of
    /// the table, by paths relative to `at`: a part of the document becomes a document of its
    /// own (a file of several definitions split into one document per definition). The
    /// inverse of [`absorb`](Self::absorb).
    #[doc(hidden)]
    #[must_use]
    pub fn take_below(&mut self, at: &JsonPath) -> Unknown {
        if self.is_empty() {
            return Self::new();
        }
        let (mut below, mut rest) = (Vec::new(), Vec::new());
        for (path, members) in std::mem::take(self).into_groups() {
            match path.strip_prefix(at) {
                Some(relative) => below.push((relative, members)),
                None => rest.push((path, members)),
            }
        }
        *self = Self::of_groups(rest);
        Self::of_groups(below)
    }

    /// Puts every row of `other` below `at` (its paths relative to `at`), each member by
    /// [`put`](Self::put)'s rule: a reader that read a part of the document with paths of
    /// its own hands its unknown members on.
    #[doc(hidden)]
    pub fn absorb(&mut self, at: &JsonPath, other: Unknown) {
        for (path, members) in other.into_groups() {
            let path = at.join(&path);
            for (key, value) in members {
                self.put(&path, &key, value);
            }
        }
    }
}

/// The objects by path, each with its members, as a map.
impl fmt::Debug for Unknown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for (index, owner) in self.owners.iter().enumerate() {
            let members = self.group(index).iter().map(|(key, value)| (&**key, value));
            map.entry(&PathDebug(&owner.path), &MembersDebug(members.collect()));
        }
        map.finish()
    }
}

/// A path as its segments joined by `/`.
struct PathDebug<'a>(&'a JsonPath);

impl fmt::Debug for PathDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, segment) in self.0.segments().enumerate() {
            if index > 0 {
                f.write_str("/")?;
            }
            match segment {
                JsonPathSegment::Key(key) => f.write_str(key)?,
                JsonPathSegment::Index(index) => write!(f, "{index}")?,
            }
        }
        Ok(())
    }
}

/// The members of one object, as a map.
struct MembersDebug<'a>(Vec<(&'a str, &'a Value)>);

impl fmt::Debug for MembersDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.0.iter().copied()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(keys: &[&str]) -> JsonPath {
        JsonPath::of_keys(keys)
    }

    fn rows(table: &Unknown) -> Vec<(String, String)> {
        table
            .iter()
            .map(|(at, key, _)| (at.to_string(), key.to_owned()))
            .collect()
    }

    #[test]
    fn an_empty_table_allocates_nothing() {
        let table = Unknown::new();
        assert!(table.is_empty());
        assert_eq!(table.owners.capacity(), 0);
        assert_eq!(table.members.capacity(), 0);
        assert_eq!(Unknown::default(), table);
    }

    #[test]
    fn members_are_grouped_by_owner_in_document_order_and_owners_sorted() {
        let mut table = Unknown::new();
        table.put(&path(&["b"]), "z", Value::from(1i64));
        table.put(&path(&["a"]), "y", Value::from(2i64));
        table.put(&path(&["b"]), "x", Value::from(3i64));
        table.put(&JsonPath::new(), "w", Value::from(4i64));
        assert_eq!(
            rows(&table),
            [("", "w"), ("a", "y"), ("b", "z"), ("b", "x")]
                .map(|(a, k)| (a.to_owned(), k.to_owned()))
        );
        let at_b: Vec<_> = table.members_at(&path(&["b"])).map(|(k, _)| k).collect();
        assert_eq!(at_b, ["z", "x"]);
        assert_eq!(table.len(), 4);
    }

    #[test]
    fn put_replaces_a_member_of_the_same_key_in_place() {
        let mut table = Unknown::new();
        table.put(&path(&["a"]), "k", Value::from(1i64));
        table.put(&path(&["a"]), "l", Value::from(2i64));
        assert_eq!(
            table.put(&path(&["a"]), "k", Value::from(3i64)),
            Some(Value::from(1i64))
        );
        let at_a: Vec<_> = table.members_at(&path(&["a"])).collect();
        assert_eq!(at_a, [("k", &Value::from(3i64)), ("l", &Value::from(2i64))]);
    }

    #[test]
    fn take_removes_the_member_and_an_owner_left_empty() {
        let mut table = Unknown::new();
        table.put(&path(&["a"]), "k", Value::from(1i64));
        table.put(&path(&["b"]), "l", Value::from(2i64));
        assert_eq!(table.take(&path(&["a"]), "missing"), None);
        assert_eq!(table.take(&path(&["a"]), "k"), Some(Value::from(1i64)));
        assert_eq!(table.owners().count(), 1);
        assert_eq!(table.get(&path(&["b"]), "l"), Some(&Value::from(2i64)));
        assert!(!table.contains(&path(&["a"]), "k"));
        assert_eq!(table.take(&path(&["b"]), "l"), Some(Value::from(2i64)));
        assert!(table.is_empty());
        assert_eq!(table, Unknown::new());
    }

    #[test]
    fn take_all_takes_one_object_in_document_order() {
        let mut table = Unknown::new();
        table.put(&path(&["a"]), "k", Value::from(1i64));
        table.put(&path(&["a"]), "l", Value::from(2i64));
        table.put(&path(&["b"]), "m", Value::from(3i64));
        let taken: Vec<_> = table
            .take_all(&path(&["a"]))
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert_eq!(taken, ["k".into(), "l".into()] as [Box<str>; 2]);
        assert_eq!(rows(&table), [("b".to_owned(), "m".to_owned())]);
    }

    #[test]
    fn rebase_moves_the_object_and_every_object_below_it() {
        let mut table = Unknown::new();
        table.put(&path(&["old"]), "k", Value::from(1i64));
        table.put(&path(&["old", "inner"]), "l", Value::from(2i64));
        table.put(&path(&["older"]), "m", Value::from(3i64));
        table.rebase(&path(&["old"]), &path(&["new", "deep"]));
        assert_eq!(
            rows(&table),
            [("new/deep", "k"), ("new/deep/inner", "l"), ("older", "m")]
                .map(|(a, k)| (a.to_owned(), k.to_owned()))
        );
    }

    #[test]
    fn a_rebased_member_replaces_one_of_its_key_where_it_goes() {
        let mut table = Unknown::new();
        table.put(&path(&["to"]), "k", Value::from(1i64));
        table.put(&path(&["to"]), "stays", Value::from(2i64));
        table.put(&path(&["from"]), "k", Value::from(3i64));
        table.put(&path(&["from"]), "new", Value::from(4i64));
        table.rebase(&path(&["from"]), &path(&["to"]));
        let at_to: Vec<_> = table.members_at(&path(&["to"])).collect();
        assert_eq!(
            at_to,
            [
                ("k", &Value::from(3i64)),
                ("stays", &Value::from(2i64)),
                ("new", &Value::from(4i64))
            ]
        );
        assert_eq!(table.owners().count(), 1);
    }

    #[test]
    fn rebase_of_a_path_no_row_is_under_changes_nothing() {
        let mut table = Unknown::new();
        table.put(&path(&["a"]), "k", Value::from(1i64));
        let before = table.clone();
        table.rebase(&path(&["b"]), &path(&["c"]));
        assert_eq!(table, before);
    }

    #[test]
    fn left_out_drops_the_members_of_elements_left_out_and_renumbers_the_rest() {
        let list = path(&["list"]);
        let mut table = Unknown::new();
        for (index, key) in [(0, "a"), (1, "b"), (2, "c")] {
            table.put(&list.index(index), key, Value::from(1i64));
        }
        table.put(&list.index(2).child("deep"), "d", Value::from(1i64));
        table.left_out(&list, &[0, 2]);
        assert_eq!(
            rows(&table),
            [("list/0", "a"), ("list/1", "c"), ("list/1/deep", "d")]
                .map(|(a, k)| (a.to_owned(), k.to_owned()))
        );
    }

    #[test]
    fn copy_puts_the_object_and_everything_below_it_at_a_second_place() {
        let mut table = Unknown::new();
        table.put(&path(&["c", "d"]), "k", Value::from(1i64));
        table.put(&path(&["c", "d", "e"]), "l", Value::from(2i64));
        table.copy(&path(&["c", "d"]), &path(&["s", "t"]));
        assert_eq!(
            rows(&table),
            [("c/d", "k"), ("c/d/e", "l"), ("s/t", "k"), ("s/t/e", "l")]
                .map(|(a, k)| (a.to_owned(), k.to_owned()))
        );
    }

    #[test]
    fn forget_drops_the_object_and_everything_below_it() {
        let mut table = Unknown::new();
        table.put(&path(&["c", "x"]), "k", Value::from(1i64));
        table.put(&path(&["c", "x", "y"]), "l", Value::from(1i64));
        table.put(&path(&["c", "xy"]), "m", Value::from(1i64));
        table.forget(&path(&["c", "x"]));
        assert_eq!(rows(&table), [("c/xy".to_owned(), "m".to_owned())]);
    }

    #[test]
    fn absorb_puts_a_relative_table_below_its_path() {
        let mut local = Unknown::new();
        local.put(&JsonPath::new(), "k", Value::from(1i64));
        local.put(&path(&["inner"]), "l", Value::from(2i64));
        let mut table = Unknown::new();
        table.put(&path(&["components"]), "z", Value::from(3i64));
        table.absorb(&path(&["components", "minecraft:foo"]), local);
        assert_eq!(
            rows(&table),
            [
                ("components", "z"),
                ("components/minecraft:foo", "k"),
                ("components/minecraft:foo/inner", "l")
            ]
            .map(|(a, k)| (a.to_owned(), k.to_owned()))
        );
    }

    #[test]
    fn take_below_takes_a_part_of_the_document_with_relative_paths() {
        let mut table = Unknown::new();
        table.put(&path(&["list"]), "a", Value::from(1i64));
        table.put(&path(&["list", "x"]), "b", Value::from(2i64));
        table.put(&path(&["list", "x", "y"]), "c", Value::from(3i64));
        table.put(&path(&["list", "xy"]), "d", Value::from(4i64));
        let below = table.take_below(&path(&["list", "x"]));
        assert_eq!(
            rows(&below),
            [("", "b"), ("y", "c")].map(|(a, k)| (a.to_owned(), k.to_owned()))
        );
        assert_eq!(
            rows(&table),
            [("list", "a"), ("list/xy", "d")].map(|(a, k)| (a.to_owned(), k.to_owned()))
        );
        let mut again = table.clone();
        again.absorb(&path(&["list", "x"]), below);
        assert_eq!(again.len(), 4);
    }

    #[test]
    fn debug_lists_each_object_with_its_members() {
        let mut table = Unknown::new();
        table.put(&path(&["a", "b"]), "k", Value::from(1i64));
        let text = format!("{table:?}");
        assert!(text.contains("a/b"), "{text}");
        assert!(text.contains("\"k\""), "{text}");
    }
}
