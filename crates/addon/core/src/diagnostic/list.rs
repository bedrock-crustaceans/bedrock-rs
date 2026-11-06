//! [`Diagnostics`]: the ordered list of findings, with the moves of an upgrade in progress and
//! the unknown members of a read.

use std::fmt;

use super::moves::{self, Moves};
use super::{Diagnostic, JsonPath, Severity};
use crate::document::{Origin, Unknown};

/// An ordered list of findings.
#[derive(Clone, Default)]
pub struct Diagnostics(Option<Box<Held>>);

/// What a [`Diagnostics`] holds, behind one pointer (`None`: nothing yet), so that a list of
/// findings is small wherever it is carried (a `Rejected` is the `Err` of every parse).
#[derive(Clone, Default)]
struct Held {
    findings: Vec<Diagnostic>,
    /// The moves of an upgrade in progress.
    moves: Moves,
    /// The moves already undone for the findings, in the order made: where each member of
    /// the loaded document is in the file.
    undone: Moves,
    /// `undone` by the place a member was moved to, made when a place is first asked for:
    /// a rule asks once per member it names.
    undone_index: std::sync::OnceLock<moves::ByDestination<'static>>,
    /// The unknown members a read keeps, by the paths of their objects, until the document
    /// is built and takes them (`Document::new`): they travel with the findings of the read,
    /// the `UnrecognizedField` of each, through every parse and step that hands findings on.
    unknown: Unknown,
}

impl Diagnostics {
    /// The findings.
    fn list(&self) -> &[Diagnostic] {
        self.0.as_ref().map_or(&[], |held| held.findings.as_slice())
    }

    /// What is held, made when nothing was.
    fn held(&mut self) -> &mut Held {
        self.0.get_or_insert_with(Box::default)
    }
}

/// The findings alone: the moves of an upgrade in progress and the places of the lists held
/// without some of their elements are bookkeeping.
impl PartialEq for Diagnostics {
    fn eq(&self, other: &Self) -> bool {
        self.list() == other.list()
    }
}

impl fmt::Debug for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Diagnostics").field(&self.list()).finish()
    }
}

impl Diagnostics {
    /// The findings in order.
    pub fn iter(&self) -> std::slice::Iter<'_, Diagnostic> {
        self.list().iter()
    }

    /// The findings in order, each open to a change in place.
    #[allow(
        clippy::iter_without_into_iter,
        reason = "a loader's door: `&mut Diagnostics` is no public iterator"
    )]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Diagnostic> {
        match self.0.as_mut() {
            Some(held) => held.findings.iter_mut(),
            None => [].iter_mut(),
        }
    }

    /// How many findings.
    pub fn len(&self) -> usize {
        self.list().len()
    }

    /// No findings.
    pub fn is_empty(&self) -> bool {
        self.list().is_empty()
    }

    /// At least one error-level finding that rejects: a kept error does not
    /// count here, though it is an error everywhere findings are counted
    /// ([`at_least`](Self::at_least), the severity).
    pub fn has_errors(&self) -> bool {
        self.list().iter().any(Diagnostic::rejects)
    }

    /// The most severe finding's severity, if any.
    pub fn max_severity(&self) -> Option<Severity> {
        self.list().iter().map(|d| d.severity).max()
    }

    /// Findings at or above `severity`.
    pub fn at_least(&self, severity: Severity) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.list().iter().filter(move |d| d.severity >= severity)
    }

    /// Appends a finding.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.held().findings.push(diagnostic);
    }

    /// Records `diagnostic` as a kept error: `Severity::Error`, marked
    /// [`kept`](Diagnostic::kept), so the document it is about loads all the same.
    ///
    /// Use it only where the game does both: it writes the line to the content log at error
    /// level AND its loader then continues with the document. A finding of the crate's own,
    /// which the game does not report, never uses it; a game error after which the game drops
    /// the document is a plain [`push`](Self::push) of an error.
    pub fn push_kept_error(&mut self, diagnostic: Diagnostic) {
        self.push(diagnostic.into_kept());
    }

    /// Records that an upgrade step moved the member at `from` to `to` (paths from the
    /// document root): a finding told from here on at or below `to` is about the member the
    /// file has at `from` ([`Diagnostics::name_as_the_file_has_it`]).
    ///
    /// The unknown members kept below `from` move with it ([`Unknown::rebase`]): the rows of
    /// the object at `from` and of the objects below it. An unknown member whose own path is
    /// `from` is in its parent's row and does not move; the step takes and puts it.
    pub fn moved(&mut self, from: JsonPath, to: JsonPath) {
        self.held().unknown.rebase(&from, &to);
        self.record_move(from, to);
    }

    /// Records a move for the findings alone.
    fn record_move(&mut self, from: JsonPath, to: JsonPath) {
        let told = self.list().len();
        self.held().moves.moved(from, to, told);
    }

    /// The unknown members kept so far, by the paths of their objects from the document
    /// root: a reader keeps a member here, a step takes one it reads ([`Unknown::take`]).
    pub fn unknown_mut(&mut self) -> &mut Unknown {
        &mut self.held().unknown
    }

    /// The unknown members kept so far ([`unknown_mut`](Self::unknown_mut)).
    pub fn unknown(&self) -> &Unknown {
        static NONE: Unknown = Unknown::new();
        self.0.as_ref().map_or(&NONE, |held| &held.unknown)
    }

    /// Takes the unknown members kept so far out of the list: the document built from the
    /// read holds them.
    pub fn take_unknown(&mut self) -> Unknown {
        self.0
            .as_mut()
            .map(|held| std::mem::take(&mut held.unknown))
            .unwrap_or_default()
    }

    /// Records that the list at `list` now holds, in order, the elements it had at the
    /// indices `kept` (ascending: a reader, a step or a check left the others out): one
    /// move for each element that is no longer at its index. A finding told from here on
    /// of an element of the list, at the index it is held at, is named at the index the
    /// file has ([`Diagnostics::name_as_the_file_has_it`]), and so is a member a rule names
    /// once the document has loaded ([`Diagnostics::place_of`]). A list kept whole records
    /// nothing.
    ///
    /// Two rules, which the moves cannot check afterwards:
    ///
    /// - Call it after everything the caller tells of the list at the indices it had
    ///   before (a reader: the file's). A finding told at such an index after the record
    ///   would be renumbered a second time.
    /// - `list` is the path the list has at the moment of the call, every index of it as
    ///   the elements above are held then. A reader records an inner list before the outer
    ///   one, by the file's index of the outer element; a step that shortens an inner list
    ///   after the outer one was recorded uses the index the outer element is held at.
    ///   Both orders are named as the file has it. What is wrong is an inner list recorded
    ///   after its outer list under the outer element's FILE index: the outer move is
    ///   undone first, and the inner one is then never reached. Such a path cannot be told
    ///   from a held one (the file's index of one element may be the held index of
    ///   another), so it is not checked.
    ///
    /// The unknown members kept below an element left out are dropped, those below one kept
    /// go with it ([`Unknown::left_out`]).
    pub fn left_out(&mut self, list: &JsonPath, kept: &[usize]) {
        if let Some(held) = self.0.as_mut() {
            held.unknown.left_out(list, kept);
        }
        for (now, was) in kept.iter().enumerate() {
            if *was != now {
                let (mut from, mut to) = (list.clone(), list.clone());
                from.push_index(*was);
                to.push_index(now);
                self.record_move(from, to);
            }
        }
    }

    /// The path the member at `at` (a path of the document as the steps have made it so
    /// far) has in the file: every recorded move undone, the latest first. For a step that
    /// holds where the file has a member (a block's write refusal names it).
    pub fn source_of(&self, at: &JsonPath) -> JsonPath {
        self.0
            .as_ref()
            .map_or_else(|| at.clone(), |held| held.moves.undo_all(at))
    }

    /// The findings and the recorded moves, in the order they were made: for a reader that
    /// gathers the findings of one member with paths relative to it, and hands them on
    /// below the member's own path (a move goes on through [`Diagnostics::moved`]). The
    /// unknown members the list holds are not among them: [`take_unknown`](Self::take_unknown)
    /// first.
    pub fn into_told(self) -> Vec<Told> {
        let held = self.0.map(|held| *held).unwrap_or_default();
        // The unknown members are not told: a reader hands them on with `take_unknown`.
        debug_assert!(
            held.unknown.is_empty(),
            "the unknown members of a list are taken before its findings are told: {:?}",
            held.unknown
        );
        let mut moves = held.moves.into_moves().into_iter().peekable();
        let mut out = Vec::with_capacity(held.findings.len());
        for (index, finding) in held.findings.into_iter().enumerate() {
            while let Some((_, from, to)) = moves.next_if(|(told, _, _)| *told <= index) {
                out.push(Told::Moved { from, to });
            }
            out.push(Told::Finding(Box::new(finding)));
        }
        out.extend(moves.map(|(_, from, to)| Told::Moved { from, to }));
        out
    }

    /// Gives every finding told after a recorded move the path its member has in the file
    /// (`Moves`), and forgets the moves. Called once, when the chain and the
    /// checks on its result are done.
    pub fn name_as_the_file_has_it(&mut self) {
        let Some(held) = self.0.as_mut() else {
            return;
        };
        let moves = std::mem::take(&mut held.moves);
        if moves.is_empty() {
            return;
        }
        let by_destination = moves.by_destination();
        for (index, finding) in held.findings.iter_mut().enumerate() {
            finding.location.json_path =
                moves.undo(&by_destination, &finding.location.json_path, index);
        }
        drop(by_destination);
        held.undone.append(moves, 0);
        held.undone_index = std::sync::OnceLock::new();
    }

    /// The path the member at `at` (a path of the loaded document) has in the file, by the
    /// moves [`Diagnostics::name_as_the_file_has_it`] has undone for the findings: for what
    /// names a member of the document once it has loaded.
    ///
    /// Every index and key of `at` is the one the document HOLDS, at every level: a path
    /// whose outer indices are already the file's may be moved a second time. A walk builds
    /// the held path of each member and asks once per member. Before the moves are undone
    /// (a document from `parse_as_written`, whose upgrade has not run) nothing is known yet
    /// and `at` comes back unchanged; so does a path no move touches.
    pub fn place_of(&self, at: &JsonPath) -> JsonPath {
        let Some(held) = self.0.as_ref().filter(|held| !held.undone.is_empty()) else {
            return at.clone();
        };
        let index = held
            .undone_index
            .get_or_init(|| held.undone.by_destination().into_owned());
        held.undone.undo(index, at, usize::MAX)
    }

    /// Appends the findings of `other`, with the moves recorded among them (each still
    /// before the findings told after it) and the places of the moves it has undone.
    pub fn append(&mut self, other: Diagnostics) {
        let Some(other) = other.0 else {
            return;
        };
        let held = self.held();
        let told = held.findings.len();
        held.findings.extend(other.findings);
        held.moves.append(other.moves, told);
        held.undone.append(other.undone, 0);
        held.undone_index = std::sync::OnceLock::new();
        held.unknown.absorb(&JsonPath::new(), other.unknown);
    }

    /// Appends the findings of `other` and keeps its unknown members, without its moves and
    /// places: `extend` with the findings of a list that a parse filled, whose unknown
    /// members (paths from the document root) the document keeps.
    pub fn extend_with_unknown(&mut self, mut other: Diagnostics) {
        let unknown = other.take_unknown();
        self.extend(other);
        if !unknown.is_empty() {
            self.unknown_mut().absorb(&JsonPath::new(), unknown);
        }
    }

    /// Puts every finding in the pack and file of `origin`, in place: the moves and places
    /// the list holds stay, so the link pass can still ask where a member of the document is
    /// in the file ([`place_of`](Self::place_of)). Rebuilding the list from its findings (a
    /// `for` over the list, pushing each) would drop them.
    pub fn locate(&mut self, origin: &Origin) {
        for finding in self.iter_mut() {
            finding.location.pack = Some(origin.pack.clone());
            finding.location.path = Some(origin.path.clone());
        }
    }

    /// [`locate`](Self::locate), by value.
    #[must_use]
    pub fn located(mut self, origin: &Origin) -> Self {
        self.locate(origin);
        self
    }

    /// Keeps the findings `keep` says to keep, in order. A recorded move stays before the
    /// findings told after it, and the places stay.
    pub fn retain(&mut self, mut keep: impl FnMut(&Diagnostic) -> bool) {
        let Some(held) = self.0.as_mut() else {
            return;
        };
        // How many of the first `n` findings stay, for each `n`.
        let mut kept = Vec::with_capacity(held.findings.len() + 1);
        kept.push(0);
        held.findings.retain(|finding| {
            let stays = keep(finding);
            kept.push(kept.last().copied().unwrap_or_default() + usize::from(stays));
            stays
        });
        held.moves.retold(&kept);
    }

    /// The first error-level finding that rejects, if any: a kept error is passed by, as
    /// [`has_errors`](Self::has_errors) passes it, so the answer is `Some` exactly when
    /// `has_errors` is. A kept error is still found by [`at_least`](Self::at_least).
    pub fn first_error(&self) -> Option<&Diagnostic> {
        self.list().iter().find(|d| d.rejects())
    }

    /// For error messages: the first error that rejects, else the first of the most severe
    /// findings (a kept error among them), else "no findings".
    pub fn summary(&self) -> impl fmt::Display + '_ {
        Summary(self)
    }

    /// The finding [`Diagnostics::summary`] names: the first that rejects, else the first of
    /// the most severe.
    fn headline(&self) -> Option<&Diagnostic> {
        if let Some(rejecting) = self.first_error() {
            return Some(rejecting);
        }
        let max = self.max_severity()?;
        self.list().iter().find(|d| d.severity == max)
    }
}

/// One entry of [`Diagnostics::into_told`].
#[derive(Clone, Debug, PartialEq)]
pub enum Told {
    /// A finding.
    Finding(Box<Diagnostic>),
    /// The member at `from` was moved to `to`.
    Moved {
        /// Where the member was.
        from: JsonPath,
        /// Where it is now.
        to: JsonPath,
    },
}

/// [`Diagnostics::summary`].
struct Summary<'a>(&'a Diagnostics);

impl fmt::Display for Summary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.headline() {
            Some(diagnostic) => diagnostic.fmt(f),
            None => f.write_str("no findings"),
        }
    }
}

impl Extend<Diagnostic> for Diagnostics {
    fn extend<I: IntoIterator<Item = Diagnostic>>(&mut self, iter: I) {
        self.held().findings.extend(iter);
    }
}

impl FromIterator<Diagnostic> for Diagnostics {
    fn from_iter<I: IntoIterator<Item = Diagnostic>>(iter: I) -> Self {
        let findings: Vec<Diagnostic> = iter.into_iter().collect();
        if findings.is_empty() {
            return Self(None);
        }
        Self(Some(Box::new(Held {
            findings,
            moves: Moves::default(),
            undone: Moves::default(),
            undone_index: std::sync::OnceLock::new(),
            unknown: Unknown::new(),
        })))
    }
}

/// The findings alone, as [`into_told`](Diagnostics::into_told) tells them: the unknown
/// members a list holds are no finding, so a list that holds some is refused in a debug build
/// (`take_unknown` first; [`extend_with_unknown`](Diagnostics::extend_with_unknown) and
/// [`append`](Diagnostics::append) hand them on). `Extend` takes any iterator of findings and
/// cannot see the list it came from, so `extend(list)` is refused the same way.
impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        let Some(held) = self.0 else {
            return Vec::new().into_iter();
        };
        debug_assert!(
            held.unknown.is_empty(),
            "the unknown members of a list are taken before its findings are iterated: {:?}",
            held.unknown
        );
        held.findings.into_iter()
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.list().iter()
    }
}

#[cfg(test)]
mod tests {
    use super::super::codes;
    use super::*;

    /// A list of findings that holds unknown members is no plain iterator of findings: the
    /// members would be lost without a word.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(
        expected = "the unknown members of a list are taken before its findings are iterated"
    )]
    fn into_iter_of_a_list_with_unknown_members_is_refused() {
        let mut d = Diagnostics::default();
        d.unknown_mut()
            .put(&JsonPath::new(), "zz", crate::json::Value::from(1i64));
        for _ in d {}
    }

    #[test]
    fn into_iter_of_a_list_without_unknown_members_gives_its_findings() {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::of(&codes::UNRECOGNIZED_FIELD).severity(Severity::Error));
        assert_eq!(d.into_iter().count(), 1);
    }
}
