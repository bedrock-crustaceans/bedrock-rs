//! Where the members an upgrade step moved came from, so that what reads a step's output
//! names a member by the path the FILE has.

use std::borrow::Cow;
use std::collections::HashMap;

use super::JsonPath;

/// The moves of a chain's steps, in the order they were made, each with the number of
/// findings told before it.
///
/// A step that renames a component or carries a member to another place records it
/// ([`Diagnostics::moved`](super::Diagnostics::moved)). A finding told AFTER a move names the
/// member at its new place;
/// [`Diagnostics::name_as_the_file_has_it`](super::Diagnostics::name_as_the_file_has_it) undoes,
/// for each finding, the moves made before it, so that it is at the member as the file has it. A
/// finding told before a move is about the document as it was then and is left alone.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Origins(Vec<Move>);

/// One move: the member now at `to` was at `from`; `told` findings had been told.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Move {
    told: usize,
    to: JsonPath,
    from: JsonPath,
}

/// The moves of an [`Origins`] by the place they moved a member to.
#[derive(Clone, Debug)]
pub(crate) struct ByPlace<'a>(HashMap<Cow<'a, JsonPath>, Vec<usize>>);

impl ByPlace<'_> {
    /// The index with paths of its own: kept beside the moves it is of.
    pub(crate) fn into_owned(self) -> ByPlace<'static> {
        ByPlace(
            self.0
                .into_iter()
                .map(|(to, moves)| (Cow::Owned(to.into_owned()), moves))
                .collect(),
        )
    }
}

impl Origins {
    /// No move.
    #[cfg(test)]
    pub fn new() -> Self {
        Self::default()
    }

    /// No move was recorded.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Records that the member now at `to` was at `from` before the step, `told` findings
    /// having been told. Of two moves to nested places, record the inner one last: the
    /// latest that matches a path is the one undone.
    pub fn moved(&mut self, from: JsonPath, to: JsonPath, told: usize) {
        if from != to {
            self.0.push(Move { told, to, from });
        }
    }

    /// Appends the moves of `other`, made after these, each with `told` more findings told
    /// before it.
    pub(crate) fn append(&mut self, other: Self, told: usize) {
        self.0.extend(other.0.into_iter().map(|mut made| {
            made.told += told;
            made
        }));
    }

    /// Gives each move the number of findings told before it once findings were taken out:
    /// `kept[n]` of the first `n` findings stay.
    pub(crate) fn retold(&mut self, kept: &[usize]) {
        for made in &mut self.0 {
            made.told = kept
                .get(made.told)
                .or(kept.last())
                .copied()
                .unwrap_or_default();
        }
    }

    /// The moves in the order made: the number of findings told before each, where the
    /// member was and where it is now.
    pub(crate) fn into_moves(self) -> Vec<(usize, JsonPath, JsonPath)> {
        self.0
            .into_iter()
            .map(|made| (made.told, made.from, made.to))
            .collect()
    }

    /// The path the member at `at` (a path of the upgraded document) has in the file: every
    /// move undone, the latest first. A path no move touched is itself.
    pub fn source_of(&self, at: &JsonPath) -> JsonPath {
        self.undo(&self.by_place(), at, usize::MAX)
    }

    /// The moves by the place they moved a member to, each list in the order made.
    pub(crate) fn by_place(&self) -> ByPlace<'_> {
        let mut index: HashMap<Cow<'_, JsonPath>, Vec<usize>> = HashMap::new();
        for (number, made) in self.0.iter().enumerate() {
            index
                .entry(Cow::Borrowed(&made.to))
                .or_default()
                .push(number);
        }
        ByPlace(index)
    }

    /// `at`, the path of the finding of index `finding`, with every move made before that
    /// finding undone: the latest move to `at` or to a member above it, then the latest
    /// before that one to where it came from, and so on. Each round costs the depth of the
    /// path, whatever the number of moves.
    pub(crate) fn undo(&self, index: &ByPlace<'_>, at: &JsonPath, finding: usize) -> JsonPath {
        let mut at = at.clone();
        // The moves are in the order made, so those before the finding are the first ones.
        let mut before = self.0.partition_point(|made| made.told <= finding);
        loop {
            let latest = (1..=at.len())
                .filter_map(|length| {
                    let moves = index.0.get(&at.prefix(length))?;
                    let made = moves.partition_point(|number| *number < before);
                    made.checked_sub(1).map(|last| moves[last])
                })
                .max();
            let Some(number) = latest else {
                return at;
            };
            let made = &self.0[number];
            let mut source = made.from.clone();
            if let Some(rest) = at.strip_prefix(&made.to) {
                source.append(&rest);
            }
            at = source;
            before = number;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Location, Phase, Severity};

    fn path(keys: &[&str]) -> JsonPath {
        let mut at = JsonPath::new();
        for key in keys {
            match key.parse::<usize>() {
                Ok(index) => at.push_index(index),
                Err(_) => at.push(key),
            }
        }
        at
    }

    #[test]
    fn a_moved_member_is_named_as_the_file_has_it() {
        let mut origins = Origins::new();
        origins.moved(path(&["c", "old"]), path(&["c", "new"]), 0);
        assert_eq!(
            origins.source_of(&path(&["c", "new", "size", "1"])),
            path(&["c", "old", "size", "1"])
        );
        assert_eq!(origins.source_of(&path(&["c", "new"])), path(&["c", "old"]));
        // A sibling and a member that only begins alike are not moved.
        assert_eq!(
            origins.source_of(&path(&["c", "other"])),
            path(&["c", "other"])
        );
        assert_eq!(origins.source_of(&path(&["c"])), path(&["c"]));
        assert_eq!(
            origins.source_of(&path(&["c", "newer"])),
            path(&["c", "newer"])
        );
    }

    #[test]
    fn moves_are_undone_latest_first_and_the_inner_one_wins() {
        let mut origins = Origins::new();
        // Step 1: `a` becomes `b`; step 2: `b` becomes `c`, its scalar the member `c/value`.
        origins.moved(path(&["a"]), path(&["b"]), 0);
        origins.moved(path(&["b"]), path(&["c"]), 0);
        origins.moved(path(&["b"]), path(&["c", "value"]), 0);
        assert_eq!(
            origins.source_of(&path(&["c", "value", "0"])),
            path(&["a", "0"])
        );
        assert_eq!(origins.source_of(&path(&["c"])), path(&["a"]));
        // An index is no key of the same spelling.
        let mut indexed = Origins::new();
        indexed.moved(path(&["list"]), path(&["wrapped", "0"]), 0);
        assert_eq!(
            indexed.source_of(&path(&["wrapped", "0", "x"])),
            path(&["list", "x"])
        );
        let mut keyed = JsonPath::new();
        keyed.push("wrapped");
        keyed.push("0");
        assert_eq!(indexed.source_of(&keyed), keyed);
    }

    #[test]
    fn a_finding_told_before_a_move_keeps_its_path() {
        let finding = |at: JsonPath| {
            Diagnostic::new(
                DiagnosticCode::InvalidType { text: None },
                Severity::Warning,
                Phase::Parse,
                Location::new(None, None, at, None),
            )
        };
        let mut d = Diagnostics::default();
        // Told of the file's own `new` (a duplicate the step drops), then `old` is moved
        // onto it, then the typed read tells of what is there now.
        d.push(finding(path(&["new"])));
        d.moved(path(&["old"]), path(&["new"]));
        d.push(finding(path(&["new", "x"])));
        d.push(finding(path(&["other"])));
        d.name_as_the_file_has_it();
        let paths: Vec<String> = d.iter().map(|f| f.location.json_path.to_string()).collect();
        assert_eq!(paths, ["new", "old/x", "other"]);
        // The moves are spent: a second call changes nothing.
        d.push(finding(path(&["new"])));
        d.name_as_the_file_has_it();
        let paths: Vec<String> = d.iter().map(|f| f.location.json_path.to_string()).collect();
        assert_eq!(paths, ["new", "old/x", "other", "new"]);
    }

    #[test]
    fn a_list_gives_the_place_a_member_has_in_the_file_while_its_moves_are_held() {
        let mut d = Diagnostics::default();
        assert_eq!(d.source_of(&path(&["new", "x"])), path(&["new", "x"]));
        d.moved(path(&["old"]), path(&["new"]));
        assert_eq!(d.source_of(&path(&["new", "x"])), path(&["old", "x"]));
        assert_eq!(d.source_of(&path(&["other"])), path(&["other"]));
        d.name_as_the_file_has_it();
        assert_eq!(d.source_of(&path(&["new", "x"])), path(&["new", "x"]));
    }

    #[test]
    fn the_findings_and_moves_of_a_list_come_in_the_order_made() {
        let finding = |at: JsonPath| {
            Diagnostic::new(
                DiagnosticCode::InvalidType { text: None },
                Severity::Warning,
                Phase::Parse,
                Location::new(None, None, at, None),
            )
        };
        let mut d = Diagnostics::default();
        d.moved(path(&["a"]), path(&["b"]));
        d.push(finding(path(&["x"])));
        d.moved(path(&["c"]), path(&["d"]));
        d.moved(path(&["e"]), path(&["f"]));
        d.push(finding(path(&["y"])));
        let shown: Vec<String> = d
            .into_told()
            .into_iter()
            .map(|told| match told {
                crate::diagnostic::Told::Finding(f) => format!("at {}", f.location.json_path),
                crate::diagnostic::Told::Moved { from, to } => format!("{from} -> {to}"),
            })
            .collect();
        assert_eq!(shown, ["a -> b", "at x", "c -> d", "e -> f", "at y"]);
    }

    fn told_at(at: JsonPath) -> Diagnostic {
        Diagnostic::new(
            DiagnosticCode::InvalidType { text: None },
            Severity::Warning,
            Phase::Parse,
            Location::new(None, None, at, None),
        )
    }

    fn paths(d: &Diagnostics) -> Vec<String> {
        d.iter().map(|f| f.location.json_path.to_string()).collect()
    }

    /// Once the findings are named as the file has them, the list still gives the place a
    /// member of the loaded document has in the file: a rule that reads the document later
    /// names it there.
    #[test]
    fn a_list_gives_the_place_of_a_member_after_its_findings_are_named() {
        let mut d = Diagnostics::default();
        // A reader left the element 0 of `list` out, then a step renamed the list.
        d.moved(path(&["list", "1"]), path(&["list", "0"]));
        d.moved(path(&["list"]), path(&["renamed"]));
        assert_eq!(
            d.place_of(&path(&["renamed", "0", "x"])),
            path(&["renamed", "0", "x"])
        );
        d.name_as_the_file_has_it();
        assert_eq!(
            d.place_of(&path(&["renamed", "0", "x"])),
            path(&["list", "1", "x"])
        );
        assert_eq!(d.place_of(&path(&["other"])), path(&["other"]));
        // The moves themselves are spent, as before.
        assert_eq!(d.source_of(&path(&["renamed"])), path(&["renamed"]));
        // The places take no part in what the list is, and go with a copy of it.
        assert_eq!(d, Diagnostics::default());
        assert_eq!(format!("{d:?}"), "Diagnostics([])");
        assert_eq!(d.clone().place_of(&path(&["renamed"])), path(&["list"]));
        // A second naming undoes what was moved since, and the places are those of both.
        d.moved(path(&["renamed"]), path(&["again"]));
        d.push(told_at(path(&["again", "0"])));
        d.name_as_the_file_has_it();
        assert_eq!(paths(&d), ["renamed/0"]);
        assert_eq!(d.place_of(&path(&["again", "0"])), path(&["list", "1"]));
    }

    /// A reader that leaves elements out records where each element it keeps was: what is
    /// told of a kept element afterwards is at the file's index, at every level.
    #[test]
    fn the_elements_a_reader_keeps_are_named_at_the_index_the_file_has() {
        let mut d = Diagnostics::default();
        // The file: `outer: [x, {inner: [x, x, a]}, x, {inner: [b, x, c]}]`.
        d.left_out(&path(&["outer", "1", "inner"]), &[2]);
        d.left_out(&path(&["outer", "3", "inner"]), &[0, 2]);
        d.left_out(&path(&["outer"]), &[1, 3]);
        for (held, file) in [
            ("outer/0/inner/0", "outer/1/inner/2"),
            ("outer/1/inner/0", "outer/3/inner/0"),
            ("outer/1/inner/1", "outer/3/inner/2"),
            ("outer/1/inner/1/name", "outer/3/inner/2/name"),
            ("outer/0", "outer/1"),
            ("outer/1", "outer/3"),
        ] {
            let held: Vec<&str> = held.split('/').collect();
            let file: Vec<&str> = file.split('/').collect();
            assert_eq!(d.source_of(&path(&held)), path(&file));
        }
        // A list kept whole records nothing.
        let mut whole = Diagnostics::default();
        whole.left_out(&path(&["list"]), &[0, 1, 2]);
        assert_eq!(whole.source_of(&path(&["list", "1"])), path(&["list", "1"]));
    }

    /// A list appended to another brings its moves, each still before the findings told
    /// after it, and its places.
    #[test]
    fn a_list_appended_brings_its_moves_and_its_places() {
        let mut first = Diagnostics::default();
        first.push(told_at(path(&["new"])));
        let mut second = Diagnostics::default();
        second.push(told_at(path(&["new", "a"])));
        second.moved(path(&["old"]), path(&["new"]));
        second.push(told_at(path(&["new", "b"])));
        let mut spent = Diagnostics::default();
        spent.moved(path(&["was"]), path(&["is"]));
        spent.name_as_the_file_has_it();
        first.append(second);
        first.append(spent);
        first.append(Diagnostics::default());
        assert_eq!(first.source_of(&path(&["new"])), path(&["old"]));
        first.name_as_the_file_has_it();
        assert_eq!(paths(&first), ["new", "new/a", "old/b"]);
        assert_eq!(first.place_of(&path(&["is"])), path(&["was"]));
        assert_eq!(first.place_of(&path(&["new"])), path(&["old"]));
        // Into a list that holds nothing.
        let mut moved = Diagnostics::default();
        moved.moved(path(&["old"]), path(&["new"]));
        let mut empty = Diagnostics::default();
        empty.append(moved);
        assert_eq!(empty.source_of(&path(&["new"])), path(&["old"]));
    }

    /// Findings changed in place, and findings taken out, leave the moves where they were
    /// among the findings that stay, and the places.
    #[test]
    fn the_moves_and_the_places_outlive_a_change_of_the_findings() {
        let mut d = Diagnostics::default();
        d.moved(path(&["was"]), path(&["is"]));
        d.name_as_the_file_has_it();
        d.push(told_at(path(&["gone"])));
        d.push(told_at(path(&["new", "a"])));
        d.moved(path(&["old"]), path(&["new"]));
        d.push(told_at(path(&["new", "b"])));
        for finding in d.iter_mut() {
            finding.severity = Severity::Info;
        }
        d.retain(|finding| finding.location.json_path != path(&["gone"]));
        assert_eq!(d.place_of(&path(&["is"])), path(&["was"]));
        d.name_as_the_file_has_it();
        assert_eq!(paths(&d), ["new/a", "old/b"]);
        assert!(d.iter().all(|finding| finding.severity == Severity::Info));
    }

    #[test]
    fn the_moves_take_no_part_in_what_a_list_of_findings_is() {
        let mut moved = Diagnostics::default();
        moved.moved(path(&["old"]), path(&["new"]));
        assert_eq!(moved, Diagnostics::default());
        assert_eq!(format!("{moved:?}"), "Diagnostics([])");
    }

    /// A path from its text: a segment of digits is an index.
    fn at(text: &str) -> JsonPath {
        path(&text.split('/').collect::<Vec<_>>())
    }

    #[test]
    fn a_list_no_reader_recorded_is_the_files_own() {
        let d = Diagnostics::default();
        assert_eq!(
            d.place_of(&at("pools/1/entries/0")),
            at("pools/1/entries/0")
        );
        let mut d = Diagnostics::default();
        // A list held whole records nothing.
        d.left_out(&at("pools"), &[0, 1, 2]);
        d.name_as_the_file_has_it();
        assert_eq!(d.place_of(&at("pools/2")), at("pools/2"));
    }

    #[test]
    fn an_index_of_a_recorded_list_is_the_files_at_every_level() {
        let mut d = Diagnostics::default();
        // The file: pools [x, P, Q]; of P the entries [x, x, E]; of Q the entries [E].
        d.left_out(&at("pools/1/entries"), &[2]);
        d.left_out(&at("pools"), &[1, 2]);
        d.name_as_the_file_has_it();
        assert_eq!(
            d.place_of(&at("pools/0/entries/0/name")),
            at("pools/1/entries/2/name")
        );
        assert_eq!(
            d.place_of(&at("pools/1/entries/0/name")),
            at("pools/2/entries/0/name")
        );
        // Each level by the path the document holds.
        assert_eq!(d.place_of(&at("pools/1")), at("pools/2"));
        assert_eq!(
            d.place_of(&at("pools/0/entries/0")),
            at("pools/1/entries/2")
        );
        assert_eq!(
            d.place_of(&at("pools/1/entries/0")),
            at("pools/2/entries/0")
        );
        // An index beyond the record (a list changed since) and a key are themselves.
        assert_eq!(d.place_of(&at("pools/5/rolls")), at("pools/5/rolls"));
        assert_eq!(d.place_of(&at("other/0")), at("other/0"));
    }

    #[test]
    fn a_second_record_counts_what_the_first_left() {
        let mut d = Diagnostics::default();
        // Of five elements the first record leaves 0, 2, 3, 4; the second takes the held 1.
        d.left_out(&at("list"), &[0, 2, 3, 4]);
        d.left_out(&at("list"), &[0, 2, 3]);
        d.name_as_the_file_has_it();
        let places: Vec<JsonPath> = (0..3)
            .map(|index| {
                let mut held = at("list");
                held.push_index(index);
                d.place_of(&held)
            })
            .collect();
        assert_eq!(places, [at("list/0"), at("list/3"), at("list/4")]);
    }

    #[test]
    fn the_places_are_no_part_of_what_a_list_of_findings_is() {
        let mut placed = Diagnostics::default();
        placed.push(told_at(at("a")));
        placed.left_out(&at("list"), &[1]);
        let mut plain = Diagnostics::default();
        plain.push(told_at(at("a")));
        assert_eq!(placed, plain);
        assert_eq!(format!("{placed:?}"), format!("{plain:?}"));
        // They stay with the list while its findings are changed in place or added to.
        for finding in placed.iter_mut() {
            finding.severity = Severity::Info;
        }
        placed.extend([told_at(at("b"))]);
        placed.name_as_the_file_has_it();
        assert_eq!(placed.place_of(&at("list/0")), at("list/1"));
        assert_eq!(placed.clone().place_of(&at("list/0")), at("list/1"));
        placed.retain(|finding| finding.location.json_path != at("a"));
        assert_eq!(placed.len(), 1);
    }

    #[test]
    fn a_renumbered_list_names_what_is_told_later_as_the_file_has_it() {
        let mut d = Diagnostics::default();
        // Told at the file's index while the list is read.
        d.push(told_at(at("list/1")));
        d.left_out(&at("list"), &[0, 2, 4]);
        // Told of the held list.
        d.push(told_at(at("list/1/x")));
        d.push(told_at(at("list/2")));
        d.push(told_at(at("list/0")));
        d.name_as_the_file_has_it();
        assert_eq!(paths(&d), ["list/1", "list/2/x", "list/4", "list/0"]);
        assert_eq!(d.place_of(&at("list/1/x")), at("list/2/x"));
    }

    /// The place of a member is asked by the path the document holds, every level of it:
    /// before the moves are undone (a document read as written, not yet upgraded) the path
    /// comes back as it is.
    #[test]
    fn a_place_is_asked_by_the_held_path_and_known_once_the_moves_are_undone() {
        let mut d = Diagnostics::default();
        d.left_out(&at("outer/2/inner"), &[1]);
        d.left_out(&at("outer"), &[2]);
        assert_eq!(d.place_of(&at("outer/0/inner/0")), at("outer/0/inner/0"));
        d.name_as_the_file_has_it();
        assert_eq!(d.place_of(&at("outer/0/inner/0")), at("outer/2/inner/1"));
    }

    /// A list may be recorded after the list that holds it, by the path its element is held
    /// at then (a step that shortens an inner list after a reader shortened the outer one):
    /// both records are undone, the inner one first.
    #[test]
    fn an_inner_list_recorded_after_its_outer_list_by_its_held_path_is_named_as_the_file_has_it() {
        let mut d = Diagnostics::default();
        // The file: `outer: [x, x, {inner: [x, a, b]}]`.
        d.left_out(&at("outer"), &[2]);
        d.left_out(&at("outer/0/inner"), &[1, 2]);
        d.push(told_at(at("outer/0/inner/1/name")));
        d.name_as_the_file_has_it();
        assert_eq!(paths(&d), ["outer/2/inner/2/name"]);
        assert_eq!(d.place_of(&at("outer/0/inner/0")), at("outer/2/inner/1"));
    }

    /// A key a step moved above a list is no list renumbered: the list is recorded under it.
    #[test]
    fn a_list_below_a_moved_key_or_a_renumbered_sibling_may_be_recorded() {
        let mut d = Diagnostics::default();
        d.moved(at("old"), at("new"));
        d.left_out(&at("outer/1/inner"), &[1]);
        d.left_out(&at("new/list"), &[1]);
        d.left_out(&at("outer/0/inner"), &[1]);
        d.left_out(&at("outer"), &[0, 1, 3]);
        d.name_as_the_file_has_it();
        assert_eq!(d.place_of(&at("new/list/0")), at("old/list/1"));
        assert_eq!(d.place_of(&at("outer/2")), at("outer/3"));
    }
}
