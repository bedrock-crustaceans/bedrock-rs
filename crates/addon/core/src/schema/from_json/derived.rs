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
use crate::diagnostic::{Code, Diagnostic, DiagnosticDetail};
use crate::document::UnknownMembers;
use crate::json::NULL_MEMBER_READ_AS_ABSENT;
use crate::json::Token;
use crate::json::kit::{Expected, Members, Node, Order, Pass, Read};
use crate::json::{ScalarArm, choose_scalar_arm, untagged_arm as object_arm};
use crate::schema::UnknownRule;
use crate::versioning::NullHandling;

/// The fields of a struct read from an object: the struct's own, or the object of the struct
/// that flattens it (`#[facet(flatten)]`), whose unknown members that struct tells.
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

/// The object a struct reads its fields from: its members, and the `schema::unknown` of the
/// field that holds it, which comes before the struct's own.
#[derive(Debug)]
pub struct Object<'v> {
    members: Members<'v>,
    field: Option<UnknownRule>,
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
        Some(Self { members, field })
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
                Some(UnknownRule::Keep) => true,
                Some(_) => false,
                None => read.reading().keeps_unknown,
            };
        let told = if keep {
            UnknownRule::Keep
        } else {
            word.unwrap_or(UnknownRule::Report)
        };
        Disposal { keep, told }
    }

    /// Whether the member `node` is read as absent for its `null` ([`NullHandling::Absent`]),
    /// which is then told ([`Pass::Absent`]).
    pub fn absent(&self, node: &Node<'_>, read: &mut Read<'_>) -> bool {
        if !node.value().is_null() || read.reading().null != NullHandling::Absent {
            return false;
        }
        let mut finding = Diagnostic::of(&core_codes::VALUE_REWRITTEN)
            .detail(DiagnosticDetail::Note(NULL_MEMBER_READ_AS_ABSENT));
        finding.location.json_path = node.path().clone();
        finding.location.span = node.span();
        read.defer(Pass::Absent, finding);
        true
    }

    /// Under [`Order::UnknownFirst`], tells the members that `knows` does not name (nor `tag`),
    /// in file order, before the struct reads its members ([`unknown`](Self::unknown) for each);
    /// whether it did, so the read of the members passes over them.
    pub fn unknown_first(
        &self,
        read: &mut Read<'_>,
        disposal: Disposal,
        tag: Option<&'static str>,
        knows: impl Fn(&str) -> bool,
        mut extra: Option<&mut UnknownMembers>,
    ) -> bool {
        if read.telling() != Order::UnknownFirst {
            return false;
        }
        for (key, member) in self.members() {
            if !knows(key) && Some(key) != tag {
                self.unknown(key, &member, read, disposal, extra.as_deref_mut());
            }
        }
        true
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
        let told = if disposal.keep && extra.is_none() && !read.keepable() {
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
}

/// The slot of one field while its struct reads its members; `T` is the field's type, or the
/// type an `Option` field holds.
#[derive(Debug)]
pub struct Slot<T> {
    state: State<T>,
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            state: State::Absent,
        }
    }
}

impl<T> Slot<T> {
    /// An empty slot.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads the member `node` with `with` (`schema::read_with`, or `T`'s `FromJson`), under
    /// the field's `schema::unknown` for the object it holds.
    pub fn read_with(
        &mut self,
        node: &Node<'_>,
        read: &mut Read<'_>,
        unknown: Option<UnknownRule>,
        with: impl FnOnce(&Node<'_>, &mut Read<'_>) -> Option<T>,
    ) {
        let saved = read.set_field_unknown(unknown);
        let value = with(node, read);
        read.set_field_unknown(saved);
        self.state = match value {
            Some(value) => State::Read(value),
            None if read.take_defaulted() => State::Defaulted,
            None => State::Failed,
        };
    }

    /// A required field: the value, or the owner fails (a missing member told).
    pub fn required(
        self,
        object: &Object<'_>,
        key: &str,
        read: &mut Read<'_>,
        ok: &mut bool,
    ) -> Option<T> {
        match self.state {
            State::Read(value) => Some(value),
            State::Absent | State::Null | State::Defaulted => {
                read.missing_member(&object.members, key);
                *ok = false;
                None
            }
            State::Failed => {
                *ok = false;
                None
            }
        }
    }

    /// An `Option` field: the value, `None` when absent; a failed member is `None` where the
    /// reading resets it, else the owner fails.
    pub fn optional(self, read: &Read<'_>, ok: &mut bool) -> Option<T> {
        match self.state {
            State::Read(value) => Some(value),
            State::Absent | State::Null | State::Defaulted => None,
            State::Failed => {
                *ok &= read.member_failure().resets;
                None
            }
        }
    }

    /// A field with facet's `default`: the value, `default` when absent; a failed member takes
    /// it where the reading resets it, else the owner fails.
    pub fn or_default(
        self,
        read: &Read<'_>,
        ok: &mut bool,
        default: impl FnOnce() -> T,
    ) -> Option<T> {
        match self.state {
            State::Read(value) => Some(value),
            State::Absent | State::Null | State::Defaulted => Some(default()),
            State::Failed if read.member_failure().resets => Some(default()),
            State::Failed => {
                *ok = false;
                None
            }
        }
    }
}

impl<T: FromJson> Slot<T> {
    /// Reads the member `node` as `T`.
    pub fn read(&mut self, node: &Node<'_>, read: &mut Read<'_>, unknown: Option<UnknownRule>) {
        self.read_with(node, read, unknown, T::from_json);
    }

    /// Reads the member `node` of an `Option` field: a `null` by the reading's null rule (the
    /// `Option` impl's), anything else as `T`.
    pub fn read_option(
        &mut self,
        node: &Node<'_>,
        read: &mut Read<'_>,
        unknown: Option<UnknownRule>,
    ) {
        if !node.value().is_null() {
            return self.read(node, read, unknown);
        }
        self.state = if super::impls::null::<T>(node, read) {
            State::Null
        } else {
            State::Failed
        };
    }
}

/// A flattened struct's fields, read from its owner's object: the value, or the owner fails.
pub fn flattened<T>(value: Option<T>, ok: &mut bool) -> Option<T> {
    if value.is_none() {
        *ok = false;
    }
    value
}

/// A string enum read by name with `from_name`: another string, a number or a bool is an
/// invalid enum value, `null` and an array a wrong type; an object, which facet reads as an
/// externally tagged variant, an invalid enum value at its first member (a wrong type when it
/// has none).
pub fn unit_enum<T>(
    node: &Node<'_>,
    read: &mut Read<'_>,
    from_name: impl FnOnce(&str) -> Option<T>,
) -> Option<T> {
    if read.stopped() {
        return None;
    }
    let value = node.value();
    if let Some(text) = value.as_string() {
        let found = from_name(text.as_str());
        if found.is_none() {
            read.refuse(node, Code::from(&core_codes::INVALID_ENUM_VALUE));
        }
        return found;
    }
    if value.is_null() || value.is_array() {
        read.wrong_type(node);
        return None;
    }
    if let Some(object) = value.as_object() {
        match object.iter().next() {
            Some((key, member)) => {
                let child = Node::new(member, node.path().child(key.as_str()));
                read.refuse(&child, Code::from(&core_codes::INVALID_ENUM_VALUE));
            }
            None => read.wrong_type(node),
        }
        return None;
    }
    read.refuse(node, Code::from(&core_codes::INVALID_ENUM_VALUE));
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
/// scalar no arm takes, or a value of a kind no arm reads, is a wrong type at the enum.
pub fn untagged_arm<T: Facet<'static>>(node: &Node<'_>, read: &mut Read<'_>) -> Option<Arm> {
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
                read.refused_scalar(node, Expected::Number);
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

/// The variant an internally tagged enum's object names by its member `tag`: the object and
/// the name, or the failure told (no object, the tag missing or no string).
pub fn tagged<'v>(
    node: &Node<'v>,
    read: &mut Read<'_>,
    tag: &'static str,
) -> Option<(Object<'v>, String)> {
    let object = Object::open(node, read)?;
    let Some(member) = object.members.get(tag) else {
        read.missing_member(&object.members, tag);
        object.close(read);
        return None;
    };
    let Some(name) = member.value().as_string() else {
        read.wrong_type(&member);
        object.close(read);
        return None;
    };
    Some((object, name.as_str().to_owned()))
}

/// Tells that the tag of an internally tagged enum names no variant: an invalid enum value at
/// the tag.
pub fn unknown_tag(object: Object<'_>, read: &mut Read<'_>, tag: &'static str) {
    if let Some(member) = object.members.get(tag) {
        read.refuse(&member, Code::from(&core_codes::INVALID_ENUM_VALUE));
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
/// kind, or an object of no or several members, is a wrong type.
pub fn external<'v>(node: &Node<'v>, read: &mut Read<'_>) -> Option<External<'v>> {
    if read.stopped() {
        return None;
    }
    if let Some(text) = node.value().as_string() {
        return Some(External::Unit(text.as_str().to_owned()));
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
/// `node` (the string, or the payload of the member whose key names none).
pub fn unknown_variant(node: &Node<'_>, read: &mut Read<'_>) {
    read.refuse(node, Code::from(&core_codes::INVALID_ENUM_VALUE));
}
