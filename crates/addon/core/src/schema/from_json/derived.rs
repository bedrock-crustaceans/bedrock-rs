//! What the `FromJson` that `#[derive(Schema)]` emits calls: the object of a struct and its
//! unknown members ([`Object`]), the slot of a field ([`Slot`]), the fields of a struct read
//! from an object it may share ([`Fields`]), and the arms of the enums ([`unit_enum`],
//! [`untagged_arm`], [`tagged`], [`external`]).
//!
//! The emitted code stays small: per field a slot, a match arm by its JSON name and one call
//! that says what absence and failure make of it; everything else is here.

use facet::{Facet, Type, UserType};

use super::FromJson;
use crate::diagnostic::codes as core_codes;
use crate::diagnostic::{Code, CodeDef, CodeMeta, Diagnostic, DiagnosticDetail};
use crate::document::UnknownMembers;
use crate::json::NULL_MEMBER_READ_AS_ABSENT;
use crate::json::Token;
use crate::json::arms::{ScalarArm, choose_scalar_arm, untagged_arm as object_arm};
use crate::json::kit::Mark;
use crate::json::kit::{Elements, Expected, Members, Names, Node, Objects, Pass, Read, UnknownAt};
use crate::schema::UnknownRule;
use crate::version::FormatVersion;
use crate::versioning::NullHandling;

/// The fields of a struct read from an object: the struct's own, or the object of the struct
/// that flattens it (`#[facet(flatten)]`), whose unknown members that struct tells.
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no fields the derived read can share an object with",
    label = "flattened here, or held by an internally tagged newtype variant",
    note = "only a struct that derives `Schema` reads its fields from another's object"
)]
pub trait Fields: Sized {
    /// The member of an internally tagged enum that names the variant, which a struct that
    /// flattens the enum reads at the place it declares the enum.
    const TAG: Option<&'static str> = None;

    /// The JSON names of the variants of an internally tagged enum, which a function that names
    /// the variant of an object (`schema::tag_with`) looks its name up in; of a flattened
    /// externally tagged enum, the members that hold a variant, in declaration order.
    const VARIANTS: &'static [&'static str] = &[];

    /// The member a struct that flattens the type asks for in a read in declared order, where it
    /// reads it: an internally tagged enum's tag, the first variant of an externally tagged one;
    /// `None` for a struct, which is read after the members.
    const AT: Option<&'static str> = Self::TAG;

    /// Whether a read in file order reads the type where the file has the member `key`: at the
    /// tag of an internally tagged enum, at any member that holds a variant of an externally
    /// tagged one.
    fn reads_at(key: &str) -> bool {
        Some(key) == Self::TAG
    }

    /// Whether a member named `key` is one of the fields (a flattened struct's included).
    fn knows(key: &str) -> bool;

    /// [`knows`](Self::knows) for the fields of `object`: a flattened enum's are the members of
    /// the variant read from it, once there is one.
    fn knows_given(object: &Object<'_>, key: &str) -> bool {
        let _ = object;
        Self::knows(key)
    }

    /// Reads the fields from `object`, telling the members no field takes when `unknown` (the
    /// struct read it is the object of; a flattened struct's are its owner's to tell). `tag`
    /// is the member of an internally tagged enum that names the variant: no unknown member.
    fn read_fields(
        object: &Object<'_>,
        read: &mut Read<'_>,
        unknown: bool,
        tag: Option<&'static str>,
    ) -> Option<Self>;
}

/// What the object's unknown members are, from its struct's word and the reading.
#[derive(Clone, Copy, Debug)]
pub struct Disposal {
    /// Whether they are kept (in the document's table, or the struct's `UnknownMembers` field).
    pub keep: bool,
    /// The word they are told by.
    pub told: UnknownRule,
}

impl Disposal {
    /// The same members kept, and told nothing (`schema::quiet`).
    #[must_use]
    pub fn silenced(self) -> Self {
        Self {
            told: UnknownRule::Silent,
            ..self
        }
    }
}

/// What a variant of a flattened enum knows of an object's members.
pub type Knows = fn(&str) -> bool;

/// The object a struct reads its fields from: its members, and the `schema::unknown` of the
/// field that holds it, which comes before the struct's own.
#[derive(Debug)]
pub struct Object<'v> {
    members: Members<'v>,
    field: Option<UnknownRule>,
    /// What the read had told when the object was opened.
    mark: Mark,
    /// Under a tree ([`Objects::Tree`]), whether an error was raised while the members were
    /// read ([`settle`](Self::settle)).
    failed: std::cell::Cell<bool>,
    /// What the variant of a flattened internally tagged enum, once read from the object, knows
    /// ([`choose`](Self::choose)).
    chosen: std::cell::Cell<Option<Knows>>,
    /// The members written `null` read as absent and kept as written
    /// ([`NullHandling::AbsentKept`]), for [`keep_nulls`](Self::keep_nulls).
    nulls: std::cell::RefCell<Vec<String>>,
}

impl<'v> Object<'v> {
    /// The object `node` holds; a wrong type when it is none. The field's word (`schema::unknown`
    /// on the field that holds the struct) is taken for this object and not passed below it,
    /// until [`close`](Self::close).
    pub fn open(node: &Node<'v>, read: &mut Read<'_>) -> Option<Self> {
        if read.stopped() {
            return None;
        }
        let members = read.object(node)?;
        let field = read.set_field_unknown(None);
        Some(Self {
            members,
            field,
            mark: read.mark(),
            failed: std::cell::Cell::new(false),
            chosen: std::cell::Cell::new(None),
            nulls: std::cell::RefCell::new(Vec::new()),
        })
    }

    /// Ends the read of the object's owner there: a flattened enum's variant failed, and the
    /// members still to come are no more told missing than the unknown ones are told.
    pub fn abandon(&self) {
        self.failed.set(true);
    }

    /// Notes the variant of a flattened internally tagged enum the object's members are read as:
    /// `knows` says which of them it takes.
    pub fn choose(&self, knows: Knows) {
        self.chosen.set(Some(knows));
    }

    /// Whether the variant chosen ([`choose`](Self::choose)) takes the member `key`; `None` while
    /// none is.
    pub fn chosen_knows(&self, key: &str) -> Option<bool> {
        self.chosen.get().map(|knows| knows(key))
    }

    /// Ends the read of the members: whether the struct may still be built. Under the game's
    /// schema tree ([`Objects::Tree`]) an error raised while they were read fails it, and its
    /// missing members are then not told.
    pub fn settle(&self, read: &Read<'_>) -> bool {
        let failed = read.telling().objects == Objects::Tree && read.raised_since(self.mark);
        self.failed.set(failed);
        !failed
    }

    /// Ends the read of the object: the field's word is back for a sibling the field also
    /// holds (the next element of a list).
    pub fn close(self, read: &mut Read<'_>) {
        read.set_field_unknown(self.field);
    }

    /// The members, in file order.
    pub fn members(&self) -> impl Iterator<Item = (&'v str, Node<'v>)> + '_ {
        self.members.iter()
    }

    /// The object's node.
    pub fn node(&self) -> &Node<'v> {
        self.members.node()
    }

    /// The member `key`, as written (a `null` too): one of those a field is fed by
    /// (`schema::members`).
    pub fn member(&self, key: &str) -> Option<Node<'v>> {
        self.members.get(key)
    }

    /// What the object's unknown members are, for a struct that says `rule` (its
    /// `schema::unknown`) and is `marked` (an `extra: Extra` or `UnknownMembers` field keeps
    /// them): kept when marked, when the word (the field's before the struct's) is `keep`, or
    /// when there is none and the reading keeps them; told by the word, `report` for none.
    pub fn disposal(&self, read: &Read<'_>, rule: Option<UnknownRule>, marked: bool) -> Disposal {
        let word = self.field.or(rule);
        let keep = marked
            || match word {
                Some(UnknownRule::Keep | UnknownRule::Silent) => true,
                Some(_) => false,
                None => read.reading().keeps_unknown,
            };
        let told = match word {
            Some(UnknownRule::Silent) => UnknownRule::Silent,
            _ if keep => UnknownRule::Keep,
            _ => word.unwrap_or(UnknownRule::Report),
        };
        Disposal { keep, told }
    }

    /// Whether the member `node` is read as absent for its `null` ([`NullHandling::Absent`],
    /// which is then told ([`Pass::Absent`]); [`NullHandling::AbsentSilently`], which is not;
    /// [`NullHandling::AbsentKept`], which keeps it as written).
    pub fn absent(&self, node: &Node<'_>, read: &mut Read<'_>) -> bool {
        if !node.value().is_null() {
            return false;
        }
        match read.reading().null {
            NullHandling::AbsentSilently => return true,
            NullHandling::AbsentKept => {
                let key = node.path().last_key().unwrap_or_default().to_owned();
                self.nulls.borrow_mut().push(key);
                return true;
            }
            NullHandling::Absent => {}
            _ => return false,
        }
        let mut finding = Diagnostic::of(&core_codes::VALUE_REWRITTEN)
            .detail(DiagnosticDetail::Note(NULL_MEMBER_READ_AS_ABSENT));
        finding.location.json_path = node.path().clone();
        finding.location.span = node.span();
        read.defer(Pass::Absent, finding);
        true
    }

    /// Keeps `value`, the member `key` a field could not read (`schema::or_kept`), beside the
    /// typed value: in `extra` (the struct's `UnknownMembers` field) when it has one, else in
    /// the document's table at the object's path, where the read can keep it.
    pub fn keep(
        &self,
        key: &str,
        value: crate::json::Value,
        read: &mut Read<'_>,
        extra: Option<&mut UnknownMembers>,
    ) {
        match extra {
            Some(extra) => {
                extra.insert(key.to_owned(), value);
            }
            None if read.keepable() => {
                let owner = self.members.node().path().clone();
                read.findings_mut().unknown_mut().put(&owner, key, value);
            }
            None => {}
        }
    }

    /// Keeps each member written `null` that a field read as absent under
    /// [`NullHandling::AbsentKept`], as written, told nothing.
    pub fn keep_nulls(&self, read: &mut Read<'_>, mut extra: Option<&mut UnknownMembers>) {
        for key in self.nulls.take() {
            self.keep(&key, crate::json::Value::NULL, read, extra.as_deref_mut());
        }
    }

    /// Tells the member `key` (at `node`) that no field takes, by `disposal`, and keeps it: in
    /// `extra` (the struct's `UnknownMembers` field) when it has one, else in the document's
    /// table at the object's path, where the read can keep it.
    pub fn unknown(
        &self,
        key: &str,
        node: &Node<'_>,
        read: &mut Read<'_>,
        disposal: Disposal,
        extra: Option<&mut UnknownMembers>,
    ) {
        let told = if disposal.keep
            && disposal.told != UnknownRule::Silent
            && extra.is_none()
            && !read.keepable()
        {
            UnknownRule::Report
        } else {
            disposal.told
        };
        read.unknown_by_rule(node, told);
        if !disposal.keep {
            return;
        }
        match extra {
            Some(extra) => {
                let value = kept(node, read);
                extra.insert(key.to_owned(), value);
            }
            None if read.keepable() => {
                let owner = self.members.node().path().clone();
                let value = kept(node, read);
                read.findings_mut().unknown_mut().put(&owner, key, value);
            }
            None => {}
        }
    }
}

impl Object<'_> {
    /// Puts what the struct kept in `extra` (its unknown members, what a field could not read, a
    /// `null` kept as written) in the order the object has them, where the read keeps them so
    /// ([`Read::keep_in_file_order`]); else what the fields kept stays after the unknown members.
    pub fn in_file_order(&self, read: &Read<'_>, extra: &mut UnknownMembers) {
        if !read.keeps_in_file_order() || extra.len() < 2 {
            return;
        }
        let places: std::collections::HashMap<&str, usize> = read
            .ordered(&self.members)
            .iter()
            .enumerate()
            .map(|(place, (key, _))| (*key, place))
            .collect();
        extra.sort_by_place(|key| places.get(key).copied().unwrap_or(usize::MAX));
    }
}

/// What a struct declares of its members, for its read: its JSON names in declaration order
/// (aliases included), the ones it reads from a format version on (`schema::since_format`),
/// whether a key is one of its members or of a struct it flattens (`knows`), and its tag.
#[derive(Clone, Copy, Debug)]
pub struct Declared<'a, K> {
    /// The JSON names, in declaration order.
    pub names: &'a [&'a str],
    /// The names read from a format version on, with it.
    pub gates: &'a [(&'a str, FormatVersion)],
    /// Whether a key names a member of the struct (or of one it flattens).
    pub knows: K,
    /// The tag member of an internally tagged enum's variant.
    pub tag: Option<&'static str>,
}

impl<'a, K: Fn(&str) -> bool> Declared<'a, K> {
    /// Whether the member `name` is read from a later format version than the document's: no
    /// member of the struct in that document.
    fn later(&self, read: &Read<'_>, name: &str) -> bool {
        read.known_format().is_some_and(|format| {
            self.gates
                .iter()
                .any(|(gated, since)| *gated == name && format < *since)
        })
    }

    /// The declared name a member written as `key` is read as when the profile ignores the case
    /// of names ([`Names::IgnoreAsciiCase`]): `key` itself is no declared name, another spelling
    /// of one is.
    fn respelled_as(&self, spelling: Names, key: &str) -> Option<&'a str> {
        if spelling != Names::IgnoreAsciiCase || (self.knows)(key) {
            return None;
        }
        self.names
            .iter()
            .copied()
            .find(|name| name.eq_ignore_ascii_case(key))
    }

    /// Whether `key` is no member of the struct: one it does not know (or knows from a later
    /// format version than the document's) and is not its tag. A spelling of a declared name in
    /// another case is that member where the profile ignores the case.
    fn is_unknown(&self, read: &Read<'_>, key: &str) -> bool {
        let respelled = self.respelled_as(read.telling().names, key);
        let name = respelled.unwrap_or(key);
        (!((self.knows)(key) || respelled.is_some()) || self.later(read, name))
            && Some(key) != self.tag
    }
}

/// The unknown member `node` as it is kept: as written, its numbers no double holds clamped and
/// told where the read says so ([`Read::clamp_kept`]).
fn kept(node: &Node<'_>, read: &mut Read<'_>) -> crate::json::Value {
    <crate::json::Value as FromJson>::from_json(node, read).unwrap_or_else(|| node.value().clone())
}

/// Where the profile tells a struct's unknown members: before its members (`.0`), among them in
/// file order (`.1`), or after them (neither).
fn unknown_placement(read: &Read<'_>) -> (bool, bool) {
    let profile = read.telling();
    match profile.unknown {
        UnknownAt::First => (true, false),
        UnknownAt::WhereMet => (false, !profile.members.declared()),
        UnknownAt::Last => (false, false),
    }
}

/// Reads the members of `object` for a struct, by the read's [`Profile`](crate::json::kit::Profile):
/// `field` reads one (`true` when it is one of the struct's, a flattened struct's or the tag; the
/// member is `None` where a read in declaration order asks for one the object lacks),
/// `unknown_member` tells and keeps one no field takes (when `unknown`: a flattened struct's
/// are its owner's to tell). In file order ([`MemberOrder::File`]) each member is read where
/// the file has it; in declaration order ([`MemberOrder::Declaration`], [`MemberOrder::Named`])
/// the struct's names ([`Declared`]) are looked up one by one. The unknown members are told, each in file order, before
/// ([`UnknownAt::First`]) or among ([`UnknownAt::WhereMet`], file order) the members; the ones
/// told after them are [`read_unknown_last`]'s, so that the struct can keep what it could not
/// read first. Nothing more is read once the read has stopped.
///
/// [`MemberOrder::File`]: crate::json::kit::MemberOrder::File
/// [`MemberOrder::Declaration`]: crate::json::kit::MemberOrder::Declaration
/// [`MemberOrder::Named`]: crate::json::kit::MemberOrder::Named
///
/// A member read from a later format version than the document's (`schema::since_format`) is
/// no member of the struct in that document: an unknown one.
pub fn read_members<K: Fn(&str) -> bool>(
    object: &Object<'_>,
    read: &mut Read<'_>,
    unknown: bool,
    declared: &Declared<'_, K>,
    mut field: impl FnMut(&str, Option<&Node<'_>>, &mut Read<'_>) -> bool,
    mut unknown_member: impl FnMut(&str, &Node<'_>, &mut Read<'_>),
) {
    let profile = read.telling();
    let in_file_order = !profile.members.declared();
    let (first, among) = unknown_placement(read);
    let members = read.ordered(&object.members);
    if unknown && first {
        for (key, member) in &members {
            if read.stopped() {
                return;
            }
            if declared.is_unknown(read, key) {
                unknown_member(key, member, read);
            }
        }
    }
    if in_file_order {
        // The last member read under each name a member was respelled to, for a later one of
        // the same name ([`Names::IgnoreAsciiCase`]).
        let mut spelled: Vec<(&str, Node<'_>)> = Vec::new();
        for (key, member) in &members {
            if read.stopped() {
                return;
            }
            let key = *key;
            let respelled = declared.respelled_as(profile.names, key);
            let name = respelled.unwrap_or(key);
            if respelled.is_some() {
                read.respelled(member);
            }
            if profile.names == Names::IgnoreAsciiCase
                && ((declared.knows)(key) || respelled.is_some())
                && !declared.later(read, name)
            {
                one_spelling(&mut spelled, name, member, read);
            }
            let read_it = !declared.later(read, name) && field(name, Some(member), read);
            if !read_it && unknown && among {
                unknown_member(key, member, read);
            }
        }
    } else {
        // A flattened struct is asked for by its place (`flat_name`), where it is declared.
        for name in declared.names {
            if read.stopped() {
                return;
            }
            let member = object
                .members
                .get(name)
                .filter(|_| !declared.later(read, name));
            field(name, member.as_ref(), read);
        }
    }
}

/// The unknown members of `object` a profile tells after the members ([`UnknownAt::Last`], or
/// in declaration order), each in file order: what [`read_members`] leaves to the struct.
pub fn read_unknown_last<K: Fn(&str) -> bool>(
    object: &Object<'_>,
    read: &mut Read<'_>,
    unknown: bool,
    declared: &Declared<'_, K>,
    mut unknown_member: impl FnMut(&str, &Node<'_>, &mut Read<'_>),
) {
    let (first, among) = unknown_placement(read);
    if !unknown || first || among {
        return;
    }
    for (key, member) in &read.ordered(&object.members) {
        if read.stopped() {
            return;
        }
        if declared.is_unknown(read, key) {
            unknown_member(key, member, read);
        }
    }
}

/// Of two members read under one name in two cases ([`Names::IgnoreAsciiCase`]), the earlier is
/// told dropped (a `DROPPED_FIELD` information at it), unless both hold one value: the later is
/// the one the field keeps. `spelled` holds the last member read under each name.
fn one_spelling<'n, 'v>(
    spelled: &mut Vec<(&'n str, Node<'v>)>,
    name: &'n str,
    member: &Node<'v>,
    read: &mut Read<'_>,
) {
    let Some((_, slot)) = spelled.iter_mut().find(|(seen, _)| *seen == name) else {
        spelled.push((name, member.clone()));
        return;
    };
    let earlier = std::mem::replace(slot, member.clone());
    if earlier.value() != member.value() {
        let mut finding = Diagnostic::of(&core_codes::DROPPED_FIELD)
            .severity(crate::diagnostic::Severity::Info)
            .at(earlier.path().clone());
        finding.location.span = earlier.span();
        read.push(finding);
    }
}

/// What a field read: nothing yet, its value, a `null` read as `None`, a failure (told), or a
/// member read as absent for a scalar its profile refuses ([`Read::take_defaulted`]).
#[derive(Debug, Default)]
enum State<T> {
    #[default]
    Absent,
    Null,
    Read(T),
    Failed,
    Defaulted,
    /// A value that failed to read, kept as written (`schema::or_kept`).
    Kept(crate::json::Value),
}

/// The code a field states for one of its member's findings (`schema::wrong_type`, `missing`,
/// `invalid`, `negative`): a `CodeDef<0>`, told as it is, or a `CodeDef<1>`, fed the read's
/// subject ([`Read::subject`]: the block an entry is read for, the file a document is in).
#[derive(Clone, Copy, Debug)]
pub struct MemberCode {
    meta: &'static CodeMeta,
    /// The code's one argument is the read's subject.
    fed: bool,
}

impl MemberCode {
    /// The code of `def`, of no argument or of one, the read's subject.
    ///
    /// # Panics
    ///
    /// When `def` takes more than one argument: at compile time, in the `static` the derive
    /// emits for the field.
    #[must_use]
    pub const fn of<const N: usize>(def: &'static CodeDef<N>) -> Self {
        assert!(
            N <= 1,
            "a member's code takes no argument or one, the read's subject (`Read::subject`)"
        );
        Self {
            meta: def.meta(),
            fed: N == 1,
        }
    }

    /// The code told, fed `subject` when it takes one; `None` for a code of one argument where
    /// the read names no subject: the read's own code is told then.
    pub(crate) fn code(self, subject: Option<&str>) -> Option<Code> {
        match (self.fed, subject) {
            (false, _) => Some(Code::of_meta(self.meta, Box::new([]))),
            (true, Some(subject)) => Some(Code::of_meta(self.meta, Box::new([subject.to_owned()]))),
            (true, None) => None,
        }
    }
}

/// What a field says of its member's findings and numbers (`schema::wrong_type`, `missing`,
/// `invalid`, `finite`): the codes in place of the read's, at the member itself (not at a
/// value inside it), and whether a float beyond its width reads as its largest finite value.
#[derive(Clone, Copy, Debug, Default)]
pub struct MemberRules {
    /// The code of a value of the wrong JSON type at the member.
    pub wrong_type: Option<MemberCode>,
    /// The code of the member missing.
    pub missing: Option<MemberCode>,
    /// The code of a value the member's type refuses.
    pub invalid: Option<MemberCode>,
    /// The code of a negative whole number given to an unsigned integer member, in place of the
    /// wrong type the strict profiles tell and the invalid value the converting one does.
    pub negative: Option<MemberCode>,
    /// A number beyond the member's float reads as its largest finite value of that sign.
    pub finite: bool,
    /// The codes are one of the game's readers' lines (`schema::era`): each is told at its own
    /// severity, as [`Diagnostic::of`](crate::diagnostic::Diagnostic::of) grades it (a kept
    /// error kept), where the others are graded by the reading; the member fails or resets as
    /// the reading says all the same.
    pub own_severity: bool,
    /// The Molang spec of the member in the era the document is in (`schema::era(.., molang =
    /// SPEC)`), for a leaf that reads by it ([`Read::member_molang`]).
    pub molang: Option<crate::molang::MolangFieldSpec>,
}

impl MemberRules {
    /// No rule: the read's own codes.
    pub const NONE: Self = Self {
        wrong_type: None,
        missing: None,
        invalid: None,
        negative: None,
        finite: false,
        own_severity: false,
        molang: None,
    };
}

/// The slot of one field while its struct reads its members; `T` is the field's type, or the
/// type an `Option` field holds.
#[derive(Debug)]
pub struct Slot<T> {
    state: State<T>,
    /// The field's rules for its member (`schema::wrong_type` ..), when it states any.
    rules: Option<&'static MemberRules>,
    /// How many steps the path of the member read has.
    depth: usize,
    /// Whether the member failed without an error under the game's schema tree, which leaves
    /// it out ([`Objects::Tree`], [`Read::left_out_since`]).
    left_out: bool,
    /// Whether the member's absence was told where it was asked for ([`missing_now`](Self::missing_now)).
    missing_told: bool,
    /// Whether the field's `schema::absent_with` was asked ([`absent_with`](Self::absent_with)).
    absent_asked: bool,
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            state: State::Absent,
            rules: None,
            depth: 0,
            left_out: false,
            missing_told: false,
            absent_asked: false,
        }
    }
}

impl<T> Slot<T> {
    /// An empty slot.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty slot of a field that states `rules` for its member.
    #[must_use]
    pub fn ruled(rules: &'static MemberRules) -> Self {
        Self {
            state: State::Absent,
            rules: Some(rules),
            depth: 0,
            left_out: false,
            missing_told: false,
            absent_asked: false,
        }
    }

    /// Reads the member `node` with `with` (`schema::read_with`, or `T`'s `FromJson`), under
    /// the field's `schema::unknown` for the object it holds and its rules for the member.
    pub fn read_with(
        &mut self,
        node: &Node<'_>,
        read: &mut Read<'_>,
        unknown: Option<UnknownRule>,
        with: impl FnOnce(&Node<'_>, &mut Read<'_>) -> Option<T>,
    ) {
        let saved = read.set_field_unknown(unknown);
        let ruled = self.rules.map(|rules| read.set_member_rules(node, rules));
        let mark = read.mark();
        let value = read.in_map_value(false, |read| with(node, read));
        if let Some(saved) = ruled {
            read.restore_member_rules(saved);
        }
        read.set_field_unknown(saved);
        self.depth = node.path().len();
        self.state = match value {
            Some(value) => State::Read(value),
            None if read.take_defaulted() => State::Defaulted,
            None => State::Failed,
        };
        self.left_out = matches!(self.state, State::Failed) && read.left_out_since(mark);
        if !matches!(self.state, State::Read(_)) {
            // A value that is not read keeps none of the unknown members it held: the owner
            // fails, or the member is reset, and the default carries none.
            read.findings_mut().unknown_mut().forget(node.path());
        }
    }

    /// The value read, while the slot holds one: what a `schema::scope` field puts in scope
    /// for the members after it.
    pub fn value(&self) -> Option<&T> {
        match &self.state {
            State::Read(value) => Some(value),
            _ => None,
        }
    }

    /// Reads the member `key` the object lacks, as a field that `reads_absent` reads it: `with`
    /// at the `null` jsoncpp gives for it ([`Node::absent`]), telling what the game tells of the
    /// absence. A value is the field's; `None` leaves the member absent (the field's default or
    /// `None`, or missing where it is required), whatever was told.
    pub fn read_absent(
        &mut self,
        object: &Object<'_>,
        key: &str,
        read: &mut Read<'_>,
        with: impl FnOnce(&Node<'_>, &mut Read<'_>) -> Option<T>,
    ) {
        if read.stopped() {
            return;
        }
        let node = Node::absent(object.node().path().child(key));
        self.depth = node.path().len();
        self.missing_told = true;
        if let Some(value) = with(&node, read) {
            self.state = State::Read(value);
        }
    }

    /// [`read_absent`](Self::read_absent) after the members of a read in file order, which asks
    /// for the members the object has alone: the field's member was not among them.
    pub fn read_absent_after(
        &mut self,
        object: &Object<'_>,
        key: &str,
        read: &mut Read<'_>,
        with: impl FnOnce(&Node<'_>, &mut Read<'_>) -> Option<T>,
    ) {
        if matches!(self.state, State::Absent) && !self.missing_told {
            self.read_absent(object, key, read, with);
        }
    }

    /// An empty slot of a field whose rules for its member are `rules`, `None` for none: the
    /// rules of the era the document is in (`schema::era`), else the field's own.
    #[must_use]
    pub fn with_rules(rules: Option<&'static MemberRules>) -> Self {
        Self {
            rules,
            ..Self::default()
        }
    }

    /// Where the member `key` is absent and the field takes its default: tells its absence
    /// under the era's `missing` code, at the code's own severity, once (a field with an era's
    /// `missing` code, `schema::era`, where the era does not require it: the game's line before
    /// it goes on with the default). Nothing without an era's code.
    pub fn absent_told(&mut self, object: &Object<'_>, key: &str, read: &mut Read<'_>) {
        if !matches!(self.state, State::Absent) || self.missing_told {
            return;
        }
        self.missing_told = true;
        if let Some(code) = self
            .rules
            .filter(|rules| rules.own_severity)
            .and_then(|rules| rules.missing)
            .and_then(|code| read.member_code(code))
        {
            read.absent_as(&object.members, key, code);
        }
    }

    /// A required field: the value, or the owner fails (a missing member told, unless the
    /// object of a tree failed: [`Object::settle`]).
    pub fn required(
        self,
        object: &Object<'_>,
        key: &str,
        read: &mut Read<'_>,
        ok: &mut bool,
    ) -> Option<T> {
        let left_out = self.left_out;
        match self.state {
            State::Read(value) => Some(value),
            State::Failed if !left_out => {
                *ok = false;
                None
            }
            // A member `schema::or_kept` kept as written is there: not missing, and not read.
            State::Kept(_) => None,
            _ if object.failed.get() => {
                *ok = false;
                None
            }
            _ => {
                if !self.missing_told {
                    self.tell_missing(object, key, read);
                }
                *ok = false;
                None
            }
        }
    }

    /// Tells the member `key` missing, under the field's own code when it has one.
    fn tell_missing(&self, object: &Object<'_>, key: &str, read: &mut Read<'_>) {
        let own = self.rules.is_some_and(|rules| rules.own_severity);
        match self
            .rules
            .and_then(|rules| rules.missing)
            .and_then(|code| read.member_code(code))
        {
            Some(code) if own => read.absent_as(&object.members, key, code),
            Some(code) => read.missing_member_as(&object.members, key, code),
            None => read.missing_member(&object.members, key),
        }
    }

    /// Where a read in declared order asks for the member `key` and the object does not have it:
    /// a required field tells its absence here, in its place among the members, and
    /// [`required`](Self::required) then does not tell it again. A tree tells it at the end
    /// ([`Object::settle`]).
    pub fn missing_now(&mut self, object: &Object<'_>, key: &str, read: &mut Read<'_>) {
        if matches!(self.state, State::Absent)
            && !self.missing_told
            && read.telling().objects != Objects::Tree
        {
            self.tell_missing(object, key, read);
            self.missing_told = true;
        }
    }

    /// A plain field of a tuple struct, whose element the array has: the value, or the owner
    /// fails.
    pub fn element(self, ok: &mut bool) -> Option<T> {
        if let State::Read(value) = self.state {
            return Some(value);
        }
        *ok = false;
        None
    }

    /// Whether the member of a field that `ends` its struct's read ends it: it failed, or it is
    /// missing and was told so where a read in declared order asked for it.
    pub fn ended(&self) -> bool {
        match self.state {
            State::Failed | State::Defaulted => true,
            State::Absent => self.missing_told,
            State::Null | State::Read(_) | State::Kept(_) => false,
        }
    }

    /// Asks `with` (`schema::absent_with`), once, for the value of the member `object` lacks
    /// (or holds as a `null` the reading takes for absence): the field's value when it gives
    /// one; it tells what it must. Nothing for a member that is written, or a read that stopped.
    pub fn absent_with(
        &mut self,
        object: &Object<'_>,
        read: &mut Read<'_>,
        with: impl FnOnce(&Node<'_>, &mut Read<'_>) -> Option<T>,
    ) {
        if self.absent_asked || !matches!(self.state, State::Absent) || read.stopped() {
            return;
        }
        self.absent_asked = true;
        if let Some(value) = with(object.node(), read) {
            self.state = State::Read(value);
        }
    }

    /// An `Option` field: the value, `None` when absent; a failed member is `None` where the
    /// reading resets it, else the owner fails.
    pub fn optional(self, read: &Read<'_>, ok: &mut bool) -> Option<T> {
        let left_out = self.left_out;
        match self.state {
            State::Read(value) => Some(value),
            State::Absent | State::Null | State::Defaulted | State::Kept(_) => None,
            State::Failed => {
                *ok &= left_out || read.resets_at(self.depth);
                None
            }
        }
    }

    /// The value a `schema::or_kept` field failed to read, kept as written; the slot is then
    /// absent, but for `required`, which finds the member there.
    pub fn take_kept(&mut self) -> Option<crate::json::Value> {
        match &mut self.state {
            // The slot stays kept: the member is there, `required` does not tell it missing.
            State::Kept(value) => Some(std::mem::replace(value, crate::json::Value::NULL)),
            _ => None,
        }
    }

    /// Reads the member `node` of an `Option` field that keeps a value it cannot read
    /// (`schema::or_kept`): an attempt ([`Read::fork`]) that reads a `null` by the null rule
    /// (for a type of `kind`) and anything else by `with`. A value read is the field's, its
    /// findings told; a value that fails leaves the field absent and is kept as written
    /// ([`take_kept`](Self::take_kept)), its findings told when `told`, failing nothing. A
    /// field that `reads_null` has a `null` read by `with` too, and kept when it fails. A field
    /// that `ends` its struct's read keeps only a value of another JSON kind than its type
    /// takes: one of that kind that fails is a failure, told.
    #[allow(clippy::too_many_arguments)] // the derive's one call per field
    pub fn read_or_kept(
        &mut self,
        node: &Node<'_>,
        read: &mut Read<'_>,
        unknown: Option<UnknownRule>,
        kind: Kind,
        told: bool,
        reads_null: bool,
        ends: bool,
        with: impl FnOnce(&Node<'_>, &mut Read<'_>) -> Option<T>,
    ) {
        if read.stopped() {
            return;
        }
        let mut attempt = read.fork();
        let mut inner = Self {
            state: State::Absent,
            rules: self.rules,
            depth: node.path().len(),
            left_out: false,
            missing_told: false,
            absent_asked: false,
        };
        if reads_null {
            inner.read_with(node, &mut attempt, unknown, with);
        } else {
            inner.read_option_with(node, &mut attempt, unknown, kind, with);
        }
        self.state = match inner.state {
            State::Failed | State::Defaulted
                if ends && kind.expected.is_some_and(|kind| kind.takes(node.value())) =>
            {
                read.commit(attempt);
                State::Failed
            }
            State::Failed | State::Defaulted => {
                if told {
                    read.commit_findings(attempt);
                }
                State::Kept(node.value().clone())
            }
            state => {
                read.commit(attempt);
                state
            }
        };
    }

    /// A field with facet's `default`: the value, `default` when absent; a failed member takes
    /// it where the reading resets it, else the owner fails.
    pub fn or_default(
        self,
        read: &Read<'_>,
        ok: &mut bool,
        default: impl FnOnce() -> T,
    ) -> Option<T> {
        let left_out = self.left_out;
        match self.state {
            State::Read(value) => Some(value),
            State::Absent | State::Null | State::Defaulted | State::Kept(_) => Some(default()),
            State::Failed if left_out || read.resets_at(self.depth) => Some(default()),
            State::Failed => {
                *ok = false;
                None
            }
        }
    }
}

// The bounds are the methods' own, so a field type without a read is the trait's error (and
// its message) at the field.
impl<T> Slot<T> {
    /// Reads the member `node` as `T`.
    pub fn read(&mut self, node: &Node<'_>, read: &mut Read<'_>, unknown: Option<UnknownRule>)
    where
        T: FromJson,
    {
        self.read_with(node, read, unknown, T::from_json);
    }

    /// Reads the member `node` of an `Option` field: a `null` by the reading's null rule (the
    /// `Option` impl's), anything else as `T`.
    pub fn read_option(
        &mut self,
        node: &Node<'_>,
        read: &mut Read<'_>,
        unknown: Option<UnknownRule>,
    ) where
        T: FromJson,
    {
        let kind = Kind {
            expected: T::EXPECTED,
            holds_null: T::HOLDS_NULL,
        };
        self.read_option_with(node, read, unknown, kind, T::from_json);
    }
}

impl<T> Slot<T> {
    /// Reads the member `node` of an `Option` field with `with` (`schema::read_with`): a
    /// `null` by the reading's null rule, as every `Option` field reads it (for a type that
    /// takes `kind`: [`Probe`]), anything else by `with`.
    pub fn read_option_with(
        &mut self,
        node: &Node<'_>,
        read: &mut Read<'_>,
        unknown: Option<UnknownRule>,
        kind: Kind,
        with: impl FnOnce(&Node<'_>, &mut Read<'_>) -> Option<T>,
    ) {
        if !node.value().is_null() || read.reading().null == NullHandling::AsValue {
            return self.read_with(node, read, unknown, with);
        }
        self.depth = node.path().len();
        let ruled = self.rules.map(|rules| read.set_member_rules(node, rules));
        let mark = read.mark();
        self.state = if super::impls::null_of(kind.expected, kind.holds_null, node, read) {
            State::Null
        } else {
            State::Failed
        };
        self.left_out = matches!(self.state, State::Failed) && read.left_out_since(mark);
        if let Some(saved) = ruled {
            read.restore_member_rules(saved);
        }
    }
}

/// Runs `with` with `word` as the unknown-member rule of the field that holds the value read:
/// what a transparent newtype's field says (`schema::unknown`) for the value the newtype is.
pub fn under_word<T>(
    read: &mut Read<'_>,
    word: Option<UnknownRule>,
    with: impl FnOnce(&mut Read<'_>) -> T,
) -> T {
    let before = read.set_field_unknown(word);
    let out = with(read);
    read.set_field_unknown(before);
    out
}

/// What a type's read says of the JSON it takes, for the null rule of a field read by its own
/// function: its `FromJson`'s [`EXPECTED`](FromJson::EXPECTED) and `HOLDS_NULL`, or nothing for
/// a type that has no `FromJson`.
#[derive(Clone, Copy, Debug)]
pub struct Kind {
    /// What JSON the type takes.
    pub expected: Option<crate::json::kit::Expected>,
    /// Whether the type holds a `null` as written.
    pub holds_null: bool,
}

/// The [`Kind`] of `T` where the derive cannot tell whether `T` has a read: the derive's field
/// types are concrete, so `(&Probe::<T>::NEW).kind()` takes [`KindOfRead`] when `T: FromJson`
/// and [`KindOfNone`] otherwise (each trait in scope, as the emitted code puts them).
pub struct Probe<T>(std::marker::PhantomData<T>);

impl<T> Probe<T> {
    /// The probe of `T`.
    pub const NEW: Self = Self(std::marker::PhantomData);
}

/// The [`Kind`] of a type with a read.
pub trait KindOfRead {
    /// Its read's kind.
    fn kind(&self) -> Kind;
}

impl<T: FromJson> KindOfRead for Probe<T> {
    fn kind(&self) -> Kind {
        Kind {
            expected: T::EXPECTED,
            holds_null: T::HOLDS_NULL,
        }
    }
}

/// The [`Kind`] of a type without a read: nothing said.
pub trait KindOfNone {
    /// No kind.
    fn kind(&self) -> Kind;
}

impl<T> KindOfNone for &Probe<T> {
    fn kind(&self) -> Kind {
        Kind {
            expected: None,
            holds_null: false,
        }
    }
}

/// The read of a list field that states how it reads (`schema::iterated`, `schema::drop`):
/// iterated as jsoncpp iterates a value when `iterated` ([`read_iterated`](super::read_iterated)),
/// a failed element left out alone when `drop` ([`ElementFailure::Drop`](super::ElementFailure::Drop)),
/// else as the reading says.
pub fn list<E: FromJson>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    iterated: bool,
    drop: bool,
) -> Option<Vec<E>> {
    let failure = if drop {
        super::ElementFailure::Drop
    } else {
        super::ElementFailure::ByReading
    };
    if iterated {
        super::read_iterated(node, read, failure, E::from_json)
    } else {
        super::read_list(node, read, failure, E::from_json)
    }
}

/// The elements of the array a tuple struct reads, or a wrong type: under the member's own
/// `schema::wrong_type` when it states one, else the struct's (`wrong_type`), else the read's.
pub fn elements<'v>(
    node: &Node<'v>,
    read: &mut Read<'_>,
    wrong_type: Option<&'static CodeDef<0>>,
) -> Option<Elements<'v>> {
    if read.stopped() {
        return None;
    }
    if let Some(code) = wrong_type
        && !node.value().is_array()
        && read
            .member_rules(node)
            .is_none_or(|rules| rules.wrong_type.is_none())
    {
        read.refuse(node, Code::from(code));
        return None;
    }
    read.array(node)
}

/// Tells `code` at `node`, over it, failing nothing: what the game tells of a value it reads on
/// (a tuple struct's array of another length, `schema::arity`).
pub fn tell_at(node: &Node<'_>, code: &'static CodeDef<0>, read: &mut Read<'_>) {
    let mut finding = Diagnostic::of(code).at(node.path().clone());
    finding.location.span = node.span();
    read.push(finding);
}

/// Whether the member `node` is of the JSON kind `T` takes (`FromJson::EXPECTED`; any kind when
/// `T` names none): a field that says `schema::if_kind` reads it only then, and is absent for a
/// member of another kind.
pub fn of_kind<T: FromJson>(node: &Node<'_>) -> bool {
    let value = node.value();
    match T::EXPECTED {
        None => true,
        Some(Expected::Object) => value.is_object(),
        Some(Expected::Array) => value.is_array(),
        Some(Expected::String) => value.as_string().is_some(),
        Some(Expected::Number) => value.as_number().is_some(),
        Some(Expected::Bool) => value.as_bool().is_some(),
    }
}

/// A flattened struct's fields, read from its owner's object: the value, or the owner fails.
pub fn flattened<T>(value: Option<T>, ok: &mut bool) -> Option<T> {
    if value.is_none() {
        *ok = false;
    }
    value
}

/// A string enum read by name with `from_name`: another string, a number, a bool or a `null`
/// (facet read it as a variant that is none) is an invalid enum value, an array a wrong type; an object, which facet reads as an
/// externally tagged variant, an invalid enum value at its first member (a wrong type when it
/// has none). An enum read from a string only (`schema::string_only`) takes any other JSON
/// type as a wrong type.
pub fn unit_enum<T>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    string_only: bool,
    from_name: impl FnOnce(&str) -> Option<T>,
) -> Option<T> {
    if read.stopped() {
        return None;
    }
    let value = node.value();
    if string_only && !value.is_string() {
        read.wrong_kind(node, Expected::String);
        return None;
    }
    if let Some(text) = value.as_string() {
        let found = from_name(text.as_str());
        if found.is_none() {
            read.refuse(node, read.invalid_enum_code());
        }
        return found;
    }
    if value.is_array() {
        read.wrong_type(node);
        return None;
    }
    if value.is_object() {
        let first = read
            .object(node)
            .and_then(|members| members.iter().next().map(|(_, member)| member));
        match first {
            Some(member) => read.refuse(&member, read.invalid_enum_code()),
            None => read.wrong_type(node),
        }
        return None;
    }
    read.refuse(node, read.invalid_enum_code());
    None
}

/// The arm of an untagged enum `T` facet reads `node` as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    /// The variant at this index of the declaration.
    Variant(usize),
    /// The first unit variant (`null`), or the unit variant of the string's name.
    Unit,
}

/// The arm of the untagged enum `T` that reads `node`, as facet's untagged read chose it
/// (mirrored here, under the reading's scalar profile): an
/// object by its keys, an array the first list arm, a scalar the first arm that takes it. A
/// scalar no arm takes, or a value of a kind no arm reads, is a wrong type at the enum. An enum
/// with `schema::invalid` (`invalid`) tells a negative whole number no arm takes (an unsigned
/// arm refuses it) under that code, as a value the enum refuses; a number beyond an arm's width
/// is a wrong type still.
pub fn untagged_arm<T: Facet<'static>>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    invalid: Option<&'static CodeDef<0>>,
) -> Option<Arm> {
    if read.stopped() {
        return None;
    }
    let Type::User(UserType::Enum(en)) = T::SHAPE.ty else {
        read.wrong_type(node);
        return None;
    };
    let variants = en.variants;
    let value = node.value();
    let arm = match Token::of(value) {
        Some(token) => match choose_scalar_arm(variants, token, read.reading().scalar_rules()) {
            ScalarArm::Unit => Some(Arm::Unit),
            ScalarArm::Arm { index, .. } => Some(Arm::Variant(index)),
            // A scalar the profile refuses for every arm: as a refused scalar member is.
            ScalarArm::Refused => {
                match invalid {
                    Some(code) if matches!(token, Token::Int(_)) => {
                        read.refuse_variant(node, code);
                    }
                    _ => read.refused_scalar(node, Expected::Number),
                }
                return None;
            }
            ScalarArm::None => None,
        },
        None => object_arm(variants, value).map(Arm::Variant),
    };
    if arm.is_none() {
        read.wrong_type(node);
    }
    arm
}

/// The index of the unit variant `name` names among `names` (an untagged enum's unit
/// variants), or the first (`null`).
pub fn unit_arm(node: &Node<'_>, names: &[&str]) -> Option<usize> {
    match node.value().as_string() {
        Some(text) => names.iter().position(|name| *name == text.as_str()),
        None => (!names.is_empty()).then_some(0),
    }
}

/// The name of the variant a read names by a function of its own (`schema::tag_with`): it looks
/// at the object, tells what it refuses (a name that is none, with the game's words) and gives
/// the variant's name as the enum's JSON gives it, or `None`.
pub type TagWith = fn(&Node<'_>, &mut Read<'_>) -> Option<&'static str>;

/// The name an internally tagged enum's object gives its variant: the string at the member `tag`
/// (a missing or non-string one is a missing member at the tag, as facet-json told it), or what
/// `with` says of the object.
pub fn variant_of(
    object: &Object<'_>,
    read: &mut Read<'_>,
    tag: &'static str,
    with: Option<TagWith>,
) -> Option<String> {
    if read.stopped() {
        return None;
    }
    if let Some(with) = with {
        return with(object.node(), read).map(str::to_owned);
    }
    let name = object.members.get(tag).and_then(|member| {
        member
            .value()
            .as_string()
            .map(|name| name.as_str().to_owned())
    });
    if name.is_none() {
        read.missing_member(&object.members, tag);
    }
    name
}

/// The member name a struct that flattens a type with this `tag` is read at, among the members of
/// the struct: where its variant is read; for a type with none `place`, which no member of an
/// object has, so that a read in declaration order reads the flattened struct where its owner
/// declares it.
#[must_use]
pub const fn flat_name(tag: Option<&'static str>, place: &'static str) -> &'static str {
    match tag {
        Some(tag) => tag,
        None => place,
    }
}

/// The variant a flattened externally tagged enum reads from its owner's object, and the member
/// that holds it: the one `with` (`schema::tag_with`) names, or else the first of `names` (the
/// variants' members, in declaration order) the object has and is not `null`. Without `with`, every
/// other of `names` the object has is a payload the read does not take ([`ignored_member`]); with
/// none present the first one is missing.
pub fn variant_member<'v>(
    object: &Object<'v>,
    read: &mut Read<'_>,
    names: &'static [&'static str],
    with: Option<TagWith>,
) -> Option<(&'static str, Node<'v>)> {
    if read.stopped() {
        return None;
    }
    let name = if let Some(with) = with {
        with(object.node(), read)?
    } else {
        let Some(chosen) = present_member(object.node(), names) else {
            read.missing_member(&object.members, names.first().copied().unwrap_or_default());
            return None;
        };
        for name in names.iter().filter(|name| **name != chosen) {
            ignored_member(object.node(), name, read);
        }
        chosen
    };
    let Some(member) = object.members.get(name) else {
        read.missing_member(&object.members, name);
        return None;
    };
    Some((name, member))
}

/// The first of `names` the object `node` has as a member that is not `null`: the variant of a
/// flattened externally tagged enum, as the game probes its payload keys.
#[must_use]
pub fn present_member(node: &Node<'_>, names: &'static [&'static str]) -> Option<&'static str> {
    let object = node.value().as_object()?;
    names
        .iter()
        .copied()
        .find(|name| object.get(name).is_some_and(|value| !value.is_null()))
}

/// Tells the member `key` of the object `node`, a variant the read does not take beside the one
/// it reads, if the object has it: `core/ignored_payload` at the read's unknown severity, the
/// member kept as written with the object's unknown members.
pub fn ignored_member(node: &Node<'_>, key: &str, read: &mut Read<'_>) {
    let Some(value) = node.value().as_object().and_then(|object| object.get(key)) else {
        return;
    };
    let span = read
        .object(node)
        .and_then(|members| members.get(key))
        .and_then(|member| member.span());
    let mut finding = Diagnostic::of(&core_codes::IGNORED_PAYLOAD).severity(read.unknown_grade());
    finding.location.json_path = node.path().child(key);
    finding.location.span = span;
    read.push(finding);
    if read.keepable() {
        read.findings_mut()
            .unknown_mut()
            .put(node.path(), key, value.clone());
    }
}

/// Tells that the tag of an internally tagged enum names no variant, as facet-json told it: a
/// wrong type at the enum's object; under the enum's `schema::invalid` (`invalid`), that code, at
/// the member the object belongs to.
pub fn unknown_tag(object: &Object<'_>, read: &mut Read<'_>, invalid: Option<&'static CodeDef<0>>) {
    match invalid {
        Some(code) => read.refuse_variant(object.node(), code),
        None => read.wrong_type(object.node()),
    }
}

/// The variant an externally tagged enum's value names: a string (a unit variant), or an
/// object of one member whose key names the variant and whose value is its payload.
#[derive(Debug)]
pub enum External<'v> {
    /// A string: the unit variant of this name.
    Unit(String),
    /// `{name: payload}`.
    Payload(String, Node<'v>),
}

/// The variant of an externally tagged enum `node` names ([`External`]); a value of another
/// kind, or an object of no or several members, is a wrong type. Under the enum's
/// `schema::invalid` (`invalid`) a `null`, a bool or a number is a value that names no variant,
/// told under that code.
pub fn external<'v>(
    node: &Node<'v>,
    read: &mut Read<'_>,
    invalid: Option<&'static CodeDef<0>>,
) -> Option<External<'v>> {
    if read.stopped() {
        return None;
    }
    if let Some(text) = node.value().as_string() {
        return Some(External::Unit(text.as_str().to_owned()));
    }
    let value = node.value();
    if let Some(code) = invalid
        && (value.is_null() || value.is_bool() || value.is_number())
    {
        read.refuse_variant(node, code);
        return None;
    }
    let members = read.object(node)?;
    let mut iter = members.iter();
    if let (Some((key, payload)), None) = (iter.next(), iter.next()) {
        return Some(External::Payload(key.to_owned(), payload));
    }
    read.wrong_type(node);
    None
}

/// Tells that an externally tagged enum's value names no variant: an invalid enum value at
/// `node` (the string, or the payload of the member whose key names none); under the enum's
/// `schema::invalid` (`invalid`), that code, at the member the value belongs to.
pub fn unknown_variant(node: &Node<'_>, read: &mut Read<'_>, invalid: Option<&'static CodeDef<0>>) {
    match invalid {
        Some(code) => read.refuse_variant(node, code),
        None => read.refuse(node, read.invalid_enum_code()),
    }
}
