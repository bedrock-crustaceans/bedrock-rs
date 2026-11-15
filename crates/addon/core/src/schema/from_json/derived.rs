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
use crate::diagnostic::{Code, CodeDef, Diagnostic, DiagnosticDetail};
use crate::document::UnknownMembers;
use crate::json::NULL_MEMBER_READ_AS_ABSENT;
use crate::json::Token;
use crate::json::kit::Mark;
use crate::json::kit::{
    Expected, MemberOrder, Members, Names, Node, Objects, Pass, Read, UnknownAt,
};
use crate::json::{ScalarArm, choose_scalar_arm, untagged_arm as object_arm};
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
    /// Whether a member named `key` is one of the fields (a flattened struct's included).
    fn knows(key: &str) -> bool;

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
        })
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
    /// which is then told ([`Pass::Absent`]); [`NullHandling::AbsentSilently`], which is not).
    pub fn absent(&self, node: &Node<'_>, read: &mut Read<'_>) -> bool {
        if !node.value().is_null() {
            return false;
        }
        match read.reading().null {
            NullHandling::AbsentSilently => return true,
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
                extra.insert(key.to_owned(), node.value().clone());
            }
            None if read.keepable() => {
                let owner = self.members.node().path().clone();
                read.findings_mut()
                    .unknown_mut()
                    .put(&owner, key, node.value().clone());
            }
            None => {}
        }
    }
}

/// The declared name a member written as `key` is read as when the profile ignores the case of
/// names ([`Names::IgnoreAsciiCase`]): `key` itself is no declared name, another spelling of one is.
fn respelled_as<'n>(
    spelling: Names,
    names: &[&'n str],
    knows: &impl Fn(&str) -> bool,
    key: &str,
) -> Option<&'n str> {
    if spelling != Names::IgnoreAsciiCase || knows(key) {
        return None;
    }
    names
        .iter()
        .copied()
        .find(|name| name.eq_ignore_ascii_case(key))
}

/// Whether `key` is no member of a struct: one it does not know (or knows from a later format
/// version than the document's, `schema::since_format`) and is not its tag. A spelling of a
/// declared name in another case is that member where the profile ignores the case.
fn is_unknown(
    read: &Read<'_>,
    tag: Option<&'static str>,
    gates: &[(&str, FormatVersion)],
    names: &[&str],
    knows: &impl Fn(&str) -> bool,
    key: &str,
) -> bool {
    let respelled = respelled_as(read.telling().names, names, knows, key);
    let name = respelled.unwrap_or(key);
    let format = read.known_versions().format;
    let later = format.is_some_and(|format| {
        gates
            .iter()
            .any(|(gated, since)| *gated == name && format < *since)
    });
    (!(knows(key) || respelled.is_some()) || later) && Some(key) != tag
}

/// Where the profile tells a struct's unknown members: before its members (`.0`), among them in
/// file order (`.1`), or after them (neither).
fn unknown_placement(read: &Read<'_>) -> (bool, bool) {
    let profile = read.telling();
    match profile.unknown {
        UnknownAt::First => (true, false),
        UnknownAt::WhereMet => (false, profile.members != MemberOrder::Declaration),
        UnknownAt::Last => (false, false),
    }
}

/// Reads the members of `object` for a struct, by the read's [`Profile`](crate::json::kit::Profile):
/// `field` reads one (`true` when it is one of the struct's, a flattened struct's or the tag),
/// `unknown_member` tells and keeps one no field takes (when `unknown`: a flattened struct's
/// are its owner's to tell). In [`MemberOrder::File`] each member is read where the file has
/// it; in [`MemberOrder::Declaration`] the struct's `names` (its JSON names, in declaration
/// order) are looked up one by one. The unknown members are told, each in file order, before
/// ([`UnknownAt::First`]) or among ([`UnknownAt::WhereMet`], file order) the members; the ones
/// told after them are [`read_unknown_last`]'s, so that the struct can keep what it could not
/// read first. Nothing more is read once the read has stopped.
///
/// A member of `gates` read from a later format version than the document's
/// (`schema::since_format`) is no member of the struct in that document: an unknown one.
#[allow(clippy::too_many_arguments)] // the derive's one call per struct
pub fn read_members(
    object: &Object<'_>,
    read: &mut Read<'_>,
    unknown: bool,
    tag: Option<&'static str>,
    names: &[&str],
    gates: &[(&str, FormatVersion)],
    knows: impl Fn(&str) -> bool,
    mut field: impl FnMut(&str, &Node<'_>, &mut Read<'_>) -> bool,
    mut unknown_member: impl FnMut(&str, &Node<'_>, &mut Read<'_>),
) {
    let profile = read.telling();
    let in_file_order = profile.members != MemberOrder::Declaration;
    let (first, among) = unknown_placement(read);
    let later = |read: &Read<'_>, key: &str| {
        read.known_versions().format.is_some_and(|format| {
            gates
                .iter()
                .any(|(name, since)| *name == key && format < *since)
        })
    };
    let members = read.ordered(&object.members);
    if unknown && first {
        for (key, member) in &members {
            if read.stopped() {
                return;
            }
            if is_unknown(read, tag, gates, names, &knows, key) {
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
            let respelled = respelled_as(profile.names, names, &knows, key);
            let name = respelled.unwrap_or(key);
            if respelled.is_some() {
                read.respelled(member);
            }
            if profile.names == Names::IgnoreAsciiCase
                && (knows(key) || respelled.is_some())
                && !later(read, name)
            {
                one_spelling(&mut spelled, name, member, read);
            }
            let read_it = !later(read, name) && field(name, member, read);
            if !read_it && unknown && among {
                unknown_member(key, member, read);
            }
        }
    } else {
        for name in names {
            if read.stopped() {
                return;
            }
            if let Some(member) = object.members.get(name).filter(|_| !later(read, name)) {
                field(name, &member, read);
            }
        }
    }
}

/// The unknown members of `object` a profile tells after the members ([`UnknownAt::Last`], or
/// in declaration order), each in file order: what [`read_members`] leaves to the struct.
#[allow(clippy::too_many_arguments)] // the derive's one call per struct
pub fn read_unknown_last(
    object: &Object<'_>,
    read: &mut Read<'_>,
    unknown: bool,
    tag: Option<&'static str>,
    names: &[&str],
    gates: &[(&str, FormatVersion)],
    knows: impl Fn(&str) -> bool,
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
        if is_unknown(read, tag, gates, names, &knows, key) {
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

/// What a field says of its member's findings and numbers (`schema::wrong_type`, `missing`,
/// `invalid`, `finite`): the codes in place of the read's, at the member itself (not at a
/// value inside it), and whether a float beyond its width reads as its largest finite value.
#[derive(Clone, Copy, Debug, Default)]
pub struct MemberRules {
    /// The code of a value of the wrong JSON type at the member.
    pub wrong_type: Option<&'static CodeDef<0>>,
    /// The code of the member missing.
    pub missing: Option<&'static CodeDef<0>>,
    /// The code of a value the member's type refuses.
    pub invalid: Option<&'static CodeDef<0>>,
    /// The code of a negative whole number given to an unsigned integer member, in place of the
    /// wrong type the strict profiles tell and the invalid value the converting one does.
    pub negative: Option<&'static CodeDef<0>>,
    /// A number beyond the member's float reads as its largest finite value of that sign.
    pub finite: bool,
}

impl MemberRules {
    /// No rule: the read's own codes.
    pub const NONE: Self = Self {
        wrong_type: None,
        missing: None,
        invalid: None,
        negative: None,
        finite: false,
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
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            state: State::Absent,
            rules: None,
            depth: 0,
            left_out: false,
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
            _ if object.failed.get() => {
                *ok = false;
                None
            }
            _ => {
                match self.rules.and_then(|rules| rules.missing) {
                    Some(code) => read.missing_member_as(&object.members, key, Code::from(code)),
                    None => read.missing_member(&object.members, key),
                }
                *ok = false;
                None
            }
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
    /// absent.
    pub fn take_kept(&mut self) -> Option<crate::json::Value> {
        match std::mem::take(&mut self.state) {
            State::Kept(value) => Some(value),
            state => {
                self.state = state;
                None
            }
        }
    }

    /// Reads the member `node` of an `Option` field that keeps a value it cannot read
    /// (`schema::or_kept`): an attempt ([`Read::fork`]) that reads a `null` by the null rule
    /// (for a type of `kind`) and anything else by `with`. A value read is the field's, its
    /// findings told; a value that fails leaves the field absent and is kept as written
    /// ([`take_kept`](Self::take_kept)), its findings told when `told`, failing nothing. A
    /// field that `reads_null` has a `null` read by `with` too, and kept when it fails.
    #[allow(clippy::too_many_arguments)] // the derive's one call per field
    pub fn read_or_kept(
        &mut self,
        node: &Node<'_>,
        read: &mut Read<'_>,
        unknown: Option<UnknownRule>,
        kind: Kind,
        told: bool,
        reads_null: bool,
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
        };
        if reads_null {
            inner.read_with(node, &mut attempt, unknown, with);
        } else {
            inner.read_option_with(node, &mut attempt, unknown, kind, with);
        }
        self.state = match inner.state {
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
        if !node.value().is_null() || read.reading().null == NullHandling::AsValue {
            return self.read(node, read, unknown);
        }
        self.depth = node.path().len();
        let ruled = self.rules.map(|rules| read.set_member_rules(node, rules));
        let mark = read.mark();
        self.state = if super::impls::null::<T>(node, read) {
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

/// Tells that the member `node`, read from the engine version `since` on
/// (`schema::since_engine`), is written in a pack of an older engine ([`Read::versions`]).
pub fn since_engine(read: &mut Read<'_>, node: &Node<'_>, since: FormatVersion) {
    read.since_engine(node, since);
}

/// The read of a list field that states how it reads (`schema::iterated`, `schema::elements`):
/// iterated as jsoncpp iterates a value when `iterated` ([`read_iterated`](super::read_iterated)),
/// a failed element doing `failure`. With `skip` (`schema::skip_code`) an element that fails to
/// read is read as an attempt and, when it fails, given up whole: its own findings are not told,
/// the one `skip` makes is, at the element, graded as a failed member is and failing nothing
/// (pair it with [`ElementFailure::Drop`](super::ElementFailure::Drop)).
pub fn list<E: FromJson>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    iterated: bool,
    failure: super::ElementFailure,
    skip: Option<fn() -> Code>,
) -> Option<Vec<E>> {
    let element = move |element: &Node<'_>, read: &mut Read<'_>| match skip {
        None => E::from_json(element, read),
        Some(code) => skipping(element, read, code),
    };
    if iterated {
        super::read_iterated(node, read, failure, element)
    } else {
        super::read_list(node, read, failure, element)
    }
}

/// The element `node` as `E`, or, when it fails to read, the one finding `code` makes in place
/// of its own ([`Read::skipped`]).
fn skipping<E: FromJson>(node: &Node<'_>, read: &mut Read<'_>, code: fn() -> Code) -> Option<E> {
    if read.stopped() {
        return None;
    }
    let mut attempt = read.fork();
    let value = E::from_json(node, &mut attempt);
    if value.is_some() {
        read.commit(attempt);
    } else {
        read.skipped(node, code());
    }
    value
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

/// The arm of the untagged enum `T` that reads `node`, as the door chooses it
/// (`json::walk`'s mirror of facet's untagged read, under the reading's scalar profile): an
/// object by its keys, an array the first list arm, a scalar the first arm that takes it. A
/// scalar no arm takes, or a value of a kind no arm reads, is a wrong type at the enum; with
/// `quiet` (an enum with a `schema::otherwise` arm, which takes it) nothing is told. An enum
/// with `schema::invalid` (`invalid`) tells a negative whole number no arm takes (an unsigned
/// arm refuses it) under that code, as a value the enum refuses; a number beyond an arm's width
/// is a wrong type still.
pub fn untagged_arm<T: Facet<'static>>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    quiet: bool,
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
                if !quiet {
                    match invalid {
                        Some(code) if matches!(token, Token::Int(_)) => {
                            read.refuse_variant(node, code);
                        }
                        _ => read.refused_scalar(node, Expected::Number),
                    }
                }
                return None;
            }
            ScalarArm::None => None,
        },
        None => object_arm(variants, value).map(Arm::Variant),
    };
    if arm.is_none() && !quiet {
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

/// The variant an internally tagged enum's object names by its member `tag`: the object and
/// the name, or the failure told as the door tells it: no object a wrong type, a tag that is
/// missing or no string a missing member at the tag (facet looks for a string tag and finds
/// none).
pub fn tagged<'v>(
    node: &Node<'v>,
    read: &mut Read<'_>,
    tag: &'static str,
) -> Option<(Object<'v>, String)> {
    let object = Object::open(node, read)?;
    let name = object.members.get(tag).and_then(|member| {
        member
            .value()
            .as_string()
            .map(|name| name.as_str().to_owned())
    });
    let Some(name) = name else {
        read.missing_member(&object.members, tag);
        object.close(read);
        return None;
    };
    Some((object, name))
}

/// Tells that the tag of an internally tagged enum names no variant, as the door tells it: a
/// wrong type at the enum's object; under the enum's `schema::invalid` (`invalid`), that code, at
/// the member the object belongs to.
pub fn unknown_tag(
    object: Object<'_>,
    read: &mut Read<'_>,
    tag: &'static str,
    invalid: Option<&'static CodeDef<0>>,
) {
    let _ = tag;
    match invalid {
        Some(code) => read.refuse_variant(object.node(), code),
        None => read.wrong_type(object.node()),
    }
    object.close(read);
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
