//! Unknown members: the keys of a document no field of its typed struct takes.
//!
//! facet skips an unknown key silently. This walk compares the struct shapes with a `Value`
//! read from the same text: field names come from `effective_name()` (and `alias`), fields
//! with `should_skip_deserializing()` take no key, a flattened struct's fields are the
//! struct's own, and every other key is reported with its JSON path (`UnrecognizedField`). `keep_unknown_members` also keeps each one of a struct that keeps
//! its unknown members in a document's `Unknown` table, by the path of its object.
//!
//! Which struct keeps them ([`keeps`]): one marked with `#[facet(skip)] extra: Extra`, or whose
//! `schema::unknown` is `keep`; and, under a reading that keeps unknown members
//! (`Reading::keeps_unknown`), every struct whose `schema::unknown` is not `report`, `reject` or
//! `ignore`. The word is the struct's own (`#[facet(schema::unknown = ..)]` on the struct, or
//! on a variant for its fields), or the field's that holds it, which comes first.

use std::borrow::Cow;
use std::ptr;

use facet::{Def, Facet, Field, Peek, Shape, Type, UserType};

use super::Value;
use super::extra::Extra;
use super::path::MemberPath;
use super::scalar::{Declared, Refusal, ScalarRules, Scalars, Token, Verdict, judge};
use super::slice::element_of;
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, JsonPath, Location, Phase, Severity, VanillaText,
};
use crate::document::Unknown as Table;
use crate::schema::UnknownRule;

/// Every member of `json` that no field of `shape` takes, at any depth, with its path, in
/// document order. Proxied types (strict wrappers, Molang fields) and untyped `Value`s take
/// anything and are not walked; a member of the wrong JSON type is the typed parse's to reject
/// and is not walked either. An untagged enum is walked through the arm facet would read the
/// value as: for an object, the first object arm left alone by a key in document order, else
/// the first object arm; for an array, the first list arm. With the typed value at hand, prefer
/// [`keep_unknown_members`], which takes the arm the value actually has.
///
/// The walk needs no stack of the caller that grows with `json`: a megabyte on the calling
/// thread is enough at any depth to `MAX_JSON_DEPTH` (`MAX_JSON_DEPTH` names what is outside
/// that rule). A value nested deeper is no text a reader of the crate reads: nothing of it is
/// walked, here and in the functions below.
pub fn unknown_members(shape: &'static Shape, json: &Value) -> Vec<(JsonPath, Value)> {
    walk(shape, json, None)
        .into_iter()
        .map(|unknown| (unknown.path.to_json_path(), unknown.value))
        .collect()
}

/// Whether `json` nests past `MAX_JSON_DEPTH`, which only a value built by hand can: the walks
/// of this module report nothing of such a value, so an empty answer of theirs says "no
/// unknown member" only where this is `false`.
#[must_use]
pub fn too_deep_to_walk(json: &Value) -> bool {
    super::write::levels_of(json).is_err()
}

/// [`unknown_members`] for `target`, which was deserialised from the same text as `json`, and
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
    let (unknowns, _) = keep_walked(target, json, false, false, keeps_unknown, kept);
    unknowns.into_iter().map(|(path, _)| path).collect()
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
    let (unknowns, _) = keep_walked(target, json, false, false, keeps_unknown, kept);
    unknowns
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
    let (unknowns, misread) = keep_walked(target, json, false, true, false, kept);
    (
        unknowns.into_iter().map(|(path, _)| path).collect(),
        misread,
    )
}

/// [`keep_members`], and among the unknown members those of a [`Range`](crate::shared::Range)
/// given as an object that the range does not read (`{"a": 1}`, a `min` beside a
/// `range_min`): reported, not kept, since a range has no `extra`.
pub fn keep_members_and_range_members<'f, T: Facet<'f>>(
    target: &T,
    json: &Value,
    kept: &mut Table,
) -> (Vec<(JsonPath, Value)>, Vec<Misread>) {
    let (unknowns, misread) = keep_walked(target, json, true, true, false, kept);
    (
        unknowns.into_iter().map(|(path, _)| path).collect(),
        misread,
    )
}

/// One unknown member as the walk returns it, its disposition beside it.
type Told = ((JsonPath, Value), UnknownRule);

/// The unknown members of `target`, those it keeps put in `kept` (each with
/// [`UnknownRule::Keep`] beside it, every other one with what its struct says), and the misread
/// scalars when `tell_misread`. `keeps_unknown` is the reading's [`keeps`] rule.
fn keep_walked<'f, T: Facet<'f>>(
    target: &T,
    json: &Value,
    ranges: bool,
    tell_misread: bool,
    keeps_unknown: bool,
    kept: &mut Table,
) -> (Vec<Told>, Vec<Misread>) {
    let walked = walk_with(
        T::SHAPE,
        json,
        Some(Peek::new(target)),
        ranges,
        None,
        tell_misread,
        keeps_unknown,
    );
    let (unknowns, misread) = (walked.unknowns, walked.misread);
    let unknowns = unknowns
        .into_iter()
        .map(|unknown| {
            let mut told = unknown.told;
            if let (Some(route), Some(key)) = (&unknown.route, &unknown.key)
                && reachable(Peek::new(target), route)
            {
                let mut owner = unknown.path.to_json_path();
                owner.pop();
                kept.put(&owner, key, unknown.value.clone());
            } else if told == UnknownRule::Keep {
                // A struct that keeps, behind a step the table cannot follow: told as one
                // that says nothing.
                told = UnknownRule::Report;
            }
            ((unknown.path.to_json_path(), unknown.value), told)
        })
        .collect();
    (unknowns, misread)
}

/// A scalar of a document that the typed member it belongs to read as another value than the
/// document writes: facet coerces scalars and reads a float beyond its width as an infinity
/// without a word (an integer that does not fit its member it
/// refuses). Members behind a strict wrapper or another proxy judge their value
/// themselves and are not looked at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Misread {
    /// The scalar's path, from the root of the value walked.
    pub path: JsonPath,
    /// What was misread.
    pub kind: Misreading,
}

/// The text of the finding of a float member that read a number beyond its width
/// ([`Misread::finding`]): by it the versioned load knows the numbers to clamp.
pub const BEYOND_FLOAT_MEMBER: &str =
    "the number is beyond the float that reads it; the member holds an infinity";

impl Misread {
    /// The finding that tells the misreading, at `at` followed by the scalar's path. For a
    /// number beyond a float an `InvalidValue` information with the text
    /// [`BEYOND_FLOAT_MEMBER`]: the versioned load then clamps the number in the text and
    /// reads again, and its finding of the clamped number stands in the place of this one
    /// (the crate's own: cereal's `JsonCpp`
    /// reader and its non-strict `RapidJSON` reader narrow the double to an infinity without a
    /// line, while its strict `RapidJSON` reader refuses it; which slice reads strictly is not
    /// established per family).
    /// For another JSON type an `InvalidType` error that rejects the document: both cereal
    /// readers fail a value of another type with "invalid numeric value" at level 3 in both
    /// log masks, and the load fails.
    #[must_use]
    pub fn finding(&self, at: &JsonPath) -> Diagnostic {
        let mut path = at.clone();
        path.append(&self.path);
        let code = match self.kind {
            Misreading::BeyondFloat => DiagnosticCode::InvalidValue {
                text: Some(VanillaText::Fixed(BEYOND_FLOAT_MEMBER)),
            },
            Misreading::WrongType => DiagnosticCode::InvalidType { text: None },
            Misreading::Defaulted => DiagnosticCode::InvalidValue { text: None },
        };
        let severity = match self.kind {
            Misreading::BeyondFloat | Misreading::Defaulted => Severity::Info,
            Misreading::WrongType => Severity::Error,
        };
        Diagnostic::new(
            code,
            severity,
            Phase::Parse,
            Location::new(None, None, path, None),
        )
    }
}

/// How a scalar was misread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Misreading {
    /// A number whose nearest float of the member's width is an infinity: the member holds
    /// the infinity, which no writer spells.
    BeyondFloat,
    /// A scalar of another JSON type than the member's: a string where a number or a boolean
    /// belongs, a number or a boolean where a string belongs, `null` where a value belongs.
    /// The member holds what facet made of it.
    WrongType,
    /// A scalar its member refuses, read as the member left out under a reading that defaults
    /// a failed member and tells it (`MemberFailure::DefaultAndTell`): an information at the
    /// scalar, the crate's own.
    Defaulted,
}

/// One unknown member.
#[derive(Debug)]
pub(crate) struct Unknown {
    /// The member's path, its key last.
    pub(crate) path: MemberPath,
    /// Its key.
    key: Option<String>,
    /// Its value.
    pub(crate) value: Value,
    /// How to reach the struct that owns it from the root value, when every step is one facet's
    /// reflection can take mutably and that struct keeps its unknown members ([`keeps`]).
    route: Option<Vec<Step>>,
    /// What its struct does with it: [`UnknownRule::Keep`] where the struct keeps it, else the
    /// struct's word (`Report` for none).
    told: UnknownRule,
}

/// One mutable step from a value to a value inside it.
#[derive(Clone, Copy, Debug)]
enum Step {
    /// A struct field, by index.
    Field(usize),
    /// A field of an enum variant: the variant's index, then the field's. Taken only when that
    /// variant is the active one.
    VariantField(usize, usize),
    /// The value inside `Some`.
    Some,
    /// A list element, by index.
    Index(usize),
}

/// The walk's position: the JSON path and the mutable route (`None` once a step cannot be
/// taken mutably).
struct Cursor {
    path: MemberPath,
    route: Option<Vec<Step>>,
    /// The scalars read as another value, so far.
    misread: Vec<Misread>,
    /// Whether the members a range in its object form does not read are unknown members.
    ranges: bool,
    /// The places, so far, where the shape reads a value and the JSON has `null` (`json::null`,
    /// rule N2).
    nulls: Vec<JsonPath>,
    /// The struct members, so far, written as `null` (`json::null`, rule N1).
    null_members: Vec<MemberPath>,
    /// The slice's scalar rules, when the walk is the scalar pre-pass: each scalar is judged
    /// by them ([`ScalarCall`]) instead of being looked at for a misreading.
    rules: Option<ScalarRules>,
    /// The scalars judged, so far, that are not read as written.
    scalars: Vec<ScalarCall>,
    /// Whether the walk without rules looks for misread scalars ([`Cursor::misread`]).
    tell_misread: bool,
    /// Whether the reading keeps the unknown members of a struct that says nothing ([`keeps`]).
    keeps_unknown: bool,
    /// The `schema::unknown` of the field being walked, for the object it holds (taken by the
    /// first object below it, through `Option`, pointers, list elements and map values).
    field_rule: Option<UnknownRule>,
}

/// What the scalar pre-pass decided for one scalar of the text: every verdict but
/// [`Verdict::Keep`], in document order.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScalarCall {
    /// The scalar's path from the root of the value walked.
    pub(crate) path: MemberPath,
    /// What its member reads it as.
    pub(crate) verdict: Verdict,
    /// The scalar's declared type; `None` for a scalar no arm of its untagged enum takes
    /// (`arm`), or a `null` given for a struct.
    pub(crate) declared: Option<Declared>,
    /// Whether the scalar is a value of an untagged enum none of whose scalar arms takes it:
    /// refused there, before any arm is read.
    pub(crate) arm: bool,
    /// The length of the array the text has where a scalar belongs (under every profile but
    /// [`Scalars::ConvertAndTell`]): facet refuses it at its `[`, a strict wrapper at its last
    /// element (`json::scalar::parse_with_rules` says it there).
    pub(crate) list: Option<usize>,
}

impl Cursor {
    fn step(&mut self, step: Option<Step>) {
        match (&mut self.route, step) {
            (Some(route), Some(step)) => route.push(step),
            (route, _) => *route = None,
        }
    }
}

/// The typed value at the walk's position, when the walk has one.
type Typed<'m, 'f> = Option<Peek<'m, 'f>>;

/// The unknown members of `json` read as `shape`; `typed` is the value deserialised from it,
/// whose enum variants then decide the arms.
pub(crate) fn walk(shape: &'static Shape, json: &Value, typed: Typed<'_, '_>) -> Vec<Unknown> {
    walk_whole(shape, json, typed, false).unknowns
}

/// What one walk of a value found.
#[derive(Default)]
pub(crate) struct Walked {
    /// The unknown members.
    pub(crate) unknowns: Vec<Unknown>,
    /// The scalars read as another value, for a walk that looks for them (`keep_members`).
    pub(crate) misread: Vec<Misread>,
    /// Every place where the shape reads a value (an `Option` among them) and the JSON has
    /// `null`; opaque places (`Value`, `RawJson`, proxies) and a unit are left out.
    pub(crate) nulls: Vec<JsonPath>,
    /// Every struct member written as `null`, whatever its type.
    pub(crate) null_members: Vec<MemberPath>,
    /// The scalars the pre-pass does not read as written ([`walk_scalars`]); empty for a walk
    /// without rules.
    pub(crate) scalars: Vec<ScalarCall>,
}

/// The whole walk of `json` as `shape`, the misread scalars aside (no caller of it asks).
pub(crate) fn walk_whole(
    shape: &'static Shape,
    json: &Value,
    typed: Typed<'_, '_>,
    ranges: bool,
) -> Walked {
    walk_with(shape, json, typed, ranges, None, false, false)
}

/// The scalar pre-pass's walk of `json`, a value of the text to be read as `shape`: every
/// scalar judged under `rules` ([`Walked::scalars`]), an untagged enum's scalar arm chosen as
/// the rules choose it. Untyped: it runs before the typed read.
pub(crate) fn walk_scalars(shape: &'static Shape, json: &Value, rules: ScalarRules) -> Walked {
    walk_with(shape, json, None, false, Some(rules), false, false)
}

/// [`walk_whole`], judging the scalars under `rules` when there are some, and without them
/// looking for the misread ones when `tell_misread`; `keeps_unknown` is the reading's
/// [`keeps`] rule.
fn walk_with(
    shape: &'static Shape,
    json: &Value,
    typed: Typed<'_, '_>,
    ranges: bool,
    rules: Option<ScalarRules>,
    tell_misread: bool,
    keeps_unknown: bool,
) -> Walked {
    // The walk calls itself for each level of `json`: it runs with the stack that depth
    // can take, as facet's read of the typed value did, and not on what its caller
    // happens to have left. A value nested past the limit is no text a reader of
    // the crate reads and is not walked.
    let Ok(levels) = super::write::levels_of(json) else {
        return Walked::default();
    };
    super::stack::with_stack_for(levels, || {
        walk_on_this_stack(
            shape,
            json,
            typed,
            ranges,
            rules,
            tell_misread,
            keeps_unknown,
        )
    })
}

/// [`walk_whole`] on the stack of its caller: the walk calls itself for each level of `json`.
fn walk_on_this_stack(
    shape: &'static Shape,
    json: &Value,
    typed: Typed<'_, '_>,
    ranges: bool,
    rules: Option<ScalarRules>,
    tell_misread: bool,
    keeps_unknown: bool,
) -> Walked {
    let mut out = Vec::new();
    let mut cursor = Cursor {
        path: MemberPath::default(),
        route: Some(Vec::new()),
        misread: Vec::new(),
        ranges,
        nulls: Vec::new(),
        null_members: Vec::new(),
        rules,
        scalars: Vec::new(),
        tell_misread,
        keeps_unknown,
        field_rule: None,
    };
    walk_value(shape, json, typed, &mut cursor, &mut out);
    Walked {
        unknowns: out,
        misread: cursor.misread,
        nulls: cursor.nulls,
        null_members: cursor.null_members,
        scalars: cursor.scalars,
    }
}

/// How a member of the scalar type `ty` misread the scalar `json`, if it did: the typed
/// parse took the value, so what facet refuses is not asked about.
fn misreading(ty: facet::ScalarType, json: &Value) -> Option<Misreading> {
    use facet::ScalarType as T;
    let is_text = matches!(ty, T::String | T::Str | T::CowStr | T::Char);
    let Some(number) = json.as_number() else {
        let same = if json.is_string() {
            is_text
        } else if json.is_bool() {
            matches!(ty, T::Bool)
        } else {
            // `null` is the default of the type; an object or array is facet's to refuse.
            !json.is_null() || matches!(ty, T::Unit)
        };
        return (!same).then_some(Misreading::WrongType);
    };
    let number = number.to_f64_lossy();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the nearest f32 is the question"
    )]
    match ty {
        T::F32 => (number as f32)
            .is_infinite()
            .then_some(Misreading::BeyondFloat),
        T::F64 => number.is_infinite().then_some(Misreading::BeyondFloat),
        T::Bool => Some(Misreading::WrongType),
        _ => is_text.then_some(Misreading::WrongType),
    }
}

/// The pre-pass's judgment of the scalar `json` for a member of the scalar type `ty`, pushed
/// when it is not read as written.
fn judge_scalar(ty: facet::ScalarType, json: &Value, rules: ScalarRules, cursor: &mut Cursor) {
    if let (Some(array), false) = (json.as_array(), rules.scalars == Scalars::ConvertAndTell) {
        cursor.scalars.push(ScalarCall {
            path: cursor.path.clone(),
            verdict: Verdict::Refused(Refusal::WrongType),
            declared: Declared::of(ty),
            arm: false,
            list: Some(array.len()),
        });
        return;
    }
    let (Some(declared), Some(token)) = (Declared::of(ty), Token::of(json)) else {
        return;
    };
    let verdict = judge(declared, token, rules);
    if verdict != Verdict::Keep {
        cursor.scalars.push(ScalarCall {
            path: cursor.path.clone(),
            verdict,
            declared: Some(declared),
            arm: false,
            list: None,
        });
    }
}

/// The arm of an untagged enum that reads the scalar `token`, as the pre-pass chooses it under
/// `rules`, or `None` when no arm with a declared scalar type reads it (then the enum's own
/// read, or the refusal pushed here, says what it is).
///
/// facet-format (rc.7) reads a scalar into an untagged enum by its variants in order: `null`
/// into the first unit variant, a string into the unit variant of its name, else the first
/// newtype variant around a scalar (or an enum of unit variants) whose type takes the token
/// (`scalar_matches_shape`: a string that parses into the type is taken), else the first such
/// variant whatever the token. Under [`Scalars::ConvertAndTell`] that is the arm. Under the
/// other profiles an arm takes the token when the strict rule does (the `JsonCpp` one under
/// [`Scalars::JsonCpp`]): the first such arm in order, and when none does and every scalar arm
/// has a declared scalar type, the token is refused at the enum (the `StrictShape` check the
/// block family made before the typed read).
fn scalar_arm(
    variants: &'static [facet::Variant],
    token: Token<'_>,
    rules: ScalarRules,
    cursor: &mut Cursor,
) -> Option<usize> {
    if matches!(token, Token::Null) && variants.iter().any(|v| v.data.fields.is_empty()) {
        return None;
    }
    if let Token::Str(text) = token
        && variants
            .iter()
            .any(|v| v.data.fields.is_empty() && v.effective_name() == text)
    {
        return None;
    }
    // The newtype variants facet reads a scalar as (`VariantFormat::NewtypeScalar`), with the
    // declared type of each that has one.
    let arms: Vec<(usize, &'static Shape, Option<Declared>)> = variants
        .iter()
        .enumerate()
        .filter_map(|(index, variant)| {
            let payload = see_through_pointer(payload_shape(variant)?);
            let scalar = payload.scalar_type();
            (scalar.is_some() || is_unit_enum(payload))
                .then(|| (index, payload, scalar.and_then(Declared::of)))
        })
        .collect();
    if rules.scalars == Scalars::ConvertAndTell {
        let json = token_value(token);
        let scalar = Scalar::of(&json)?;
        let chosen = arms
            .iter()
            .find(|(_, payload, _)| scalar.match_quality(payload).is_some())
            .or_else(|| arms.first())?;
        return chosen.2.map(|_| chosen.0);
    }
    let arm_rules = match rules.scalars {
        Scalars::JsonCpp => rules,
        _ => rules.with_scalars(Scalars::Strict),
    };
    let takes =
        |declared: Declared| !matches!(judge(declared, token, arm_rules), Verdict::Refused(_));
    if let Some((index, _, _)) = arms
        .iter()
        .find(|(_, _, declared)| declared.is_some_and(takes))
    {
        return Some(*index);
    }
    if !arms.is_empty() && arms.iter().all(|(_, _, declared)| declared.is_some()) {
        cursor.scalars.push(ScalarCall {
            path: cursor.path.clone(),
            verdict: Verdict::Refused(Refusal::WrongType),
            declared: None,
            arm: true,
            list: None,
        });
    }
    None
}

/// The value of a scalar token, for the scalar matching facet's untagged read does.
fn token_value(token: Token<'_>) -> Value {
    match token {
        Token::Null => Value::NULL,
        Token::Bool(b) => Value::from(b),
        Token::UInt(u) => Value::from(u),
        Token::Int(i) => Value::from(i),
        Token::Real(x) => Value::from(x),
        Token::Str(text) => Value::from(text),
    }
}

/// `shape` without pointers (facet-solver `deref_pointer`).
fn see_through_pointer(mut shape: &'static Shape) -> &'static Shape {
    while let Def::Pointer(def) = shape.def {
        match def.pointee() {
            Some(pointee) => shape = pointee,
            None => break,
        }
    }
    shape
}

/// An enum of unit variants only, which facet reads from a scalar (facet-solver
/// `is_unit_enum_shape`).
fn is_unit_enum(shape: &'static Shape) -> bool {
    match shape.ty {
        Type::User(UserType::Enum(en)) => en.variants.iter().all(|v| v.data.fields.is_empty()),
        _ => false,
    }
}

fn walk_value(
    shape: &'static Shape,
    json: &Value,
    typed: Typed<'_, '_>,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    // A `RawJson` is parsed on its own, like a proxy or a `Value`.
    if shape.effective_proxy(None).is_some()
        || shape.id == Value::SHAPE.id
        || shape.id == <facet_json::RawJson<'static> as Facet<'static>>::SHAPE.id
    {
        walk_range(shape, json, cursor, out);
        return;
    }
    if json.is_null() && !takes_null(shape) {
        cursor.nulls.push(cursor.path.to_json_path());
    }
    match shape.def {
        Def::Option(option) => {
            if !json.is_null() {
                let inner = typed
                    .and_then(|t| t.into_option().ok())
                    .and_then(facet::PeekOption::value);
                with_step(cursor, Some(Step::Some), |cursor| {
                    walk_value(option.t(), json, inner, cursor, out);
                });
            }
            return;
        }
        Def::Pointer(pointer) => {
            if let Some(pointee) = pointer.pointee() {
                let inner = typed
                    .and_then(|t| t.into_pointer().ok())
                    .and_then(|p| p.borrow_inner());
                with_step(cursor, None, |cursor| {
                    walk_value(pointee, json, inner, cursor, out);
                });
            }
            return;
        }
        Def::List(_) | Def::Array(_) | Def::Slice(_) | Def::Set(_) => {
            let (Some(array), Some(element)) = (json.as_array(), element_or_set(shape)) else {
                return;
            };
            let list = typed.and_then(|t| t.into_list_like().ok());
            let mutable = matches!(shape.def, Def::List(_));
            for (index, item) in array.iter().enumerate() {
                cursor.path.push_index(index);
                let typed_item = list.and_then(|l| l.get(index));
                with_step(cursor, mutable.then_some(Step::Index(index)), |cursor| {
                    walk_value(element, item, typed_item, cursor, out);
                });
                cursor.path.pop();
            }
            return;
        }
        Def::Map(map) => {
            walk_map(map.v(), json, typed, cursor, out);
            return;
        }
        _ => {}
    }
    if let Some(ty) = shape.scalar_type() {
        if let Some(rules) = cursor.rules {
            judge_scalar(ty, json, rules, cursor);
        } else if cursor.tell_misread
            && let Some(kind) = misreading(ty, json)
        {
            cursor.misread.push(Misread {
                path: cursor.path.to_json_path(),
                kind,
            });
        }
        return;
    }
    match shape.ty {
        // A `null` the typed parse took for a struct is a struct of defaults: another value
        // than the document writes (an `Option` above reads it as none).
        Type::User(UserType::Struct(_)) if json.is_null() => {
            if cursor.rules.is_some() {
                cursor.scalars.push(ScalarCall {
                    path: cursor.path.clone(),
                    verdict: Verdict::Converted,
                    declared: None,
                    arm: false,
                    list: None,
                });
            } else if cursor.tell_misread {
                cursor.misread.push(Misread {
                    path: cursor.path.to_json_path(),
                    kind: Misreading::WrongType,
                });
            }
        }
        Type::User(UserType::Struct(st)) => walk_struct(shape, st.fields, json, typed, cursor, out),
        Type::User(UserType::Enum(en)) => walk_enum(shape, en.variants, json, typed, cursor, out),
        _ => {}
    }
}

/// A struct of the named `fields` (or a transparent newtype, walked as its one field), whose
/// own `schema::unknown` is in `shape`'s attributes.
fn walk_struct(
    shape: &'static Shape,
    fields: &'static [Field],
    json: &Value,
    typed: Typed<'_, '_>,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    let typed_fields = typed.and_then(|t| t.into_struct().ok());
    let typed_field = |index: usize| typed_fields.and_then(|f| f.field(index).ok());
    if shape.is_transparent() && fields.len() == 1 {
        with_step(cursor, Some(Step::Field(0)), |cursor| {
            walk_value(fields[0].shape(), json, typed_field(0), cursor, out);
        });
    } else {
        let own = rule_of(shape.attributes.iter());
        walk_object(fields, own, &Step::Field, &typed_field, json, cursor, out);
    }
}

/// Whether a value of `shape` holds a JSON `null` as what it is: a unit, or an `Option` of an
/// untyped value (vanilla's dynamic value holds a `null`).
fn takes_null(shape: &'static Shape) -> bool {
    match shape.def {
        Def::Option(option) => option.t().id == Value::SHAPE.id,
        _ => shape.scalar_type() == Some(facet::ScalarType::Unit),
    }
}

/// The entries of a JSON object read as a map whose values have the shape `value`.
///
/// A function of its own, so that what it holds is not in the frame of [`walk_value`],
/// which calls itself for each level of a value and runs on the stack of its caller.
// The typed value of each key is found once for the map: looked for from the first entry
// at every key, a map of n entries cost n².
#[inline(never)]
fn walk_map(
    value: &'static Shape,
    json: &Value,
    typed: Typed<'_, '_>,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    let Some(object) = json.as_object() else {
        return;
    };
    let mut typed_items = std::collections::HashMap::new();
    if let Some(typed_map) = typed.and_then(|t| t.into_map().ok()) {
        for (typed_key, typed_item) in typed_map.iter() {
            if let Some(typed_key) = typed_key.as_str() {
                typed_items.entry(typed_key).or_insert(typed_item);
            }
        }
    }
    for (key, item) in object {
        cursor.path.push_key(key.as_str());
        let typed_item = typed_items.get(key.as_str()).copied();
        with_step(cursor, None, |cursor| {
            walk_value(value, item, typed_item, cursor, out);
        });
        cursor.path.pop();
    }
}

/// The members of a range given as an object that the range does not read, and the elements
/// of one given as a list behind its two ends, as unknown members without a route (reported,
/// not kept), when the walk was asked for them.
fn walk_range(shape: &'static Shape, json: &Value, cursor: &mut Cursor, out: &mut Vec<Unknown>) {
    if !cursor.ranges || !crate::shared::is_range(shape) {
        return;
    }
    // The elements of a list behind the two ends (an integer range reads the first two).
    if let Some(list) = json.as_array() {
        for (index, element) in list.iter().enumerate().skip(2) {
            cursor.path.push_index(index);
            out.push(Unknown {
                path: cursor.path.clone(),
                key: None,
                value: element.clone(),
                route: None,
                told: UnknownRule::Report,
            });
            cursor.path.pop();
        }
        return;
    }
    let Some(object) = json.as_object() else {
        return;
    };
    for key in crate::shared::unread_range_members(object) {
        cursor.path.push_key(key);
        out.push(Unknown {
            path: cursor.path.clone(),
            key: None,
            value: object.get(key).cloned().unwrap_or(Value::NULL),
            route: None,
            told: UnknownRule::Report,
        });
        cursor.path.pop();
    }
}

/// Runs `f` one route step deeper.
fn with_step(cursor: &mut Cursor, step: Option<Step>, f: impl FnOnce(&mut Cursor)) {
    let saved = cursor.route.clone();
    cursor.step(step);
    f(cursor);
    cursor.route = saved;
}

/// The element shape of a list, array, slice or set.
fn element_or_set(shape: &'static Shape) -> Option<&'static Shape> {
    match shape.def {
        Def::Set(set) => Some(set.t()),
        _ => element_of(shape),
    }
}

/// The members of a JSON object read as the named `fields` (of a struct, or of a struct
/// variant, whose `schema::unknown` is `own`): known keys are walked, the others reported, and
/// given a route when the struct [`keeps`] them. `to_step` and `typed_field` give the route
/// step and the typed value of the field at an index.
fn walk_object<'m, 'f>(
    fields: &'static [Field],
    own: Option<UnknownRule>,
    to_step: &dyn Fn(usize) -> Step,
    typed_field: &dyn Fn(usize) -> Typed<'m, 'f>,
    json: &Value,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    let Some(object) = json.as_object() else {
        return;
    };
    let held = cursor.field_rule.take();
    let rule = held.or(own);
    let keeps = keeps(fields.iter().any(is_extra), rule, cursor.keeps_unknown);
    let told = if keeps {
        UnknownRule::Keep
    } else {
        rule.unwrap_or(UnknownRule::Report)
    };
    for (key, item) in object {
        let key = key.as_str();
        cursor.path.push_key(key);
        walk_member(
            fields,
            to_step,
            typed_field,
            (key, item),
            (keeps, told),
            cursor,
            out,
        );
        cursor.path.pop();
    }
    // A list or a map below the field holds more objects the field's word is for.
    cursor.field_rule = held;
}

/// The member `key` (whose value is `item`, at the cursor's path) of an object read as the
/// named `fields`: [`walk_object`]'s step for one key. A key a flattened struct field takes is
/// walked as that struct's member, one route step deeper and at the same JSON path: a
/// flattened field adds no JSON level. `keeps` and `told` are the object's, the flattened
/// struct's members being the object's own.
fn walk_member<'m, 'f>(
    fields: &'static [Field],
    to_step: &dyn Fn(usize) -> Step,
    typed_field: &dyn Fn(usize) -> Typed<'m, 'f>,
    (key, item): (&str, &Value),
    (keeps, told): (bool, UnknownRule),
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    match find_field(fields, key) {
        Lookup::Field(index, field) => {
            if item.is_null() {
                cursor.null_members.push(cursor.path.clone());
            }
            if field.proxy().is_none() {
                with_step(cursor, Some(to_step(index)), |cursor| {
                    cursor.field_rule = rule_of(field.attributes.iter());
                    walk_value(field.shape(), item, typed_field(index), cursor, out);
                    cursor.field_rule = None;
                });
            }
        }
        Lookup::Flattened(index, flattened) => {
            // A flattened map takes every key as an entry it holds as written.
            let Some(inner) = struct_fields(flattened.shape()) else {
                return;
            };
            let typed_inner = typed_field(index).and_then(|t| t.into_struct().ok());
            let typed_inner = |inner: usize| typed_inner.and_then(|f| f.field(inner).ok());
            with_step(cursor, Some(to_step(index)), |cursor| {
                walk_member(
                    inner,
                    &Step::Field,
                    &typed_inner,
                    (key, item),
                    (keeps, told),
                    cursor,
                    out,
                );
            });
        }
        Lookup::Unknown => out.push(Unknown {
            path: cursor.path.clone(),
            key: Some(key.to_owned()),
            value: item.clone(),
            route: cursor.route.clone().filter(|_| keeps),
            told,
        }),
    }
}

/// Whether a struct keeps its unknown members in the document's table: it is marked with an
/// `extra` field (`has_extra`), or its word (`rule`, the field's that holds it before its own)
/// is `keep`, or it says nothing and the reading keeps them (`reading_keeps`,
/// `Reading::keeps_unknown`). `report`, `reject` and `ignore` keep nothing but under the mark.
fn keeps(has_extra: bool, rule: Option<UnknownRule>, reading_keeps: bool) -> bool {
    has_extra
        || match rule {
            Some(UnknownRule::Keep) => true,
            Some(_) => false,
            None => reading_keeps,
        }
}

/// The `schema::unknown` word among `attributes` (a shape's, a variant's or a field's).
fn rule_of<'a>(mut attributes: impl Iterator<Item = &'a facet::Attr>) -> Option<UnknownRule> {
    attributes
        .find(|attr| attr.ns() == Some("schema") && attr.key() == "unknown")
        .and_then(|attr| attr.get_as::<&'static str>())
        .and_then(|word| UnknownRule::from_word(word))
}

/// Where a JSON key goes among a struct's fields.
enum Lookup {
    /// To the field at this index.
    Field(usize, &'static Field),
    /// Into the flattened field at this index: a struct one of whose fields takes the key, or
    /// a map.
    Flattened(usize, &'static Field),
    /// Nowhere.
    Unknown,
}

fn find_field(fields: &'static [Field], key: &str) -> Lookup {
    for (index, field) in fields.iter().enumerate() {
        if field.should_skip_deserializing() {
            continue;
        }
        if field.is_flattened() {
            let takes = match field.shape().def {
                Def::Map(_) => true,
                _ => struct_fields(field.shape())
                    .is_some_and(|inner| !matches!(find_field(inner, key), Lookup::Unknown)),
            };
            if takes {
                return Lookup::Flattened(index, field);
            }
            continue;
        }
        if field.effective_name() == key || field.alias == Some(key) {
            return Lookup::Field(index, field);
        }
    }
    Lookup::Unknown
}

/// The named fields of a struct shape.
fn struct_fields(shape: &'static Shape) -> Option<&'static [Field]> {
    match shape.ty {
        Type::User(UserType::Struct(st)) => Some(st.fields),
        _ => None,
    }
}

/// `#[facet(skip)] extra: Extra`.
fn is_extra(field: &Field) -> bool {
    field.should_skip_deserializing() && field.shape().id == Extra::SHAPE.id
}

fn walk_enum(
    shape: &'static Shape,
    variants: &'static [facet::Variant],
    json: &Value,
    typed: Typed<'_, '_>,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    let typed_enum = typed.and_then(|t| t.into_enum().ok());
    let active = typed_enum.and_then(|e| e.variant_index().ok());
    // The typed value's fields when `index` is its variant.
    let typed_payload = |index: usize| typed_enum.filter(|_| active == Some(index));
    if shape.is_untagged() {
        if active.is_none()
            && let Some(rules) = cursor.rules
            && let Some(token) = Token::of(json)
        {
            if let Some(index) = scalar_arm(variants, token, rules, cursor) {
                walk_variant(index, &variants[index], json, None, cursor, out);
            }
            return;
        }
        if let Some(index) = active.or_else(|| untagged_arm(variants, json)) {
            walk_variant(
                index,
                &variants[index],
                json,
                typed_payload(index),
                cursor,
                out,
            );
        }
        return;
    }
    let Some(object) = json.as_object() else {
        return; // a unit variant written as a string
    };
    if let Some(tag) = shape.get_tag_attr() {
        // Internally tagged (`{"type": "a", …}`): the payload's fields sit beside the tag.
        let Some(name) = object.get(tag).and_then(|t| t.as_string()) else {
            return;
        };
        if let Some(index) = variants
            .iter()
            .position(|v| v.effective_name() == name.as_str())
        {
            let mut rest = object.clone();
            rest.remove(tag);
            let payload = Value::from(rest);
            walk_variant(
                index,
                &variants[index],
                &payload,
                typed_payload(index),
                cursor,
                out,
            );
        }
        return;
    }
    // Externally tagged: `{"Variant": payload}`.
    let mut members = object.iter();
    let (Some((name, payload)), None) = (members.next(), members.next()) else {
        return;
    };
    if let Some(index) = variants
        .iter()
        .position(|v| v.effective_name() == name.as_str())
    {
        cursor.path.push_key(name.as_str());
        walk_variant(
            index,
            &variants[index],
            payload,
            typed_payload(index),
            cursor,
            out,
        );
        cursor.path.pop();
    }
}

/// A variant's payload: a struct variant's fields, or a newtype variant's one field. `typed` is
/// the enum value when this variant is its active one.
fn walk_variant(
    index: usize,
    variant: &'static facet::Variant,
    json: &Value,
    typed: Option<facet::PeekEnum<'_, '_>>,
    cursor: &mut Cursor,
    out: &mut Vec<Unknown>,
) {
    let fields = variant.data.fields;
    let typed_field = |field: usize| typed.and_then(|e| e.field(field).ok().flatten());
    match variant.data.kind {
        facet::StructKind::Struct => walk_object(
            fields,
            rule_of(variant.attributes.iter()),
            &|field| Step::VariantField(index, field),
            &typed_field,
            json,
            cursor,
            out,
        ),
        facet::StructKind::TupleStruct | facet::StructKind::Tuple if fields.len() == 1 => {
            with_step(cursor, Some(Step::VariantField(index, 0)), |cursor| {
                walk_value(fields[0].shape(), json, typed_field(0), cursor, out);
            });
        }
        _ => {}
    }
}

/// The arm of an untagged enum facet reads a JSON value as, mirroring facet-format rc.7's
/// untagged deserialiser without the typed value:
/// - an object: the arms that read an object (struct variants, and newtype variants around a
///   named struct or a map) are the candidates; each key, in document order, that some
///   remaining candidate has narrows the candidates to those having it, and the first time one
///   candidate is left it is the arm (facet-solver `see_key`); a key no remaining candidate has
///   is ignored. When the remaining candidates give the key fields of different types and its
///   value is a scalar, they are narrowed further to those whose field type takes the value best
///   (`scalar_narrowing`); when none takes it, facet stops and reads the first object arm. When
///   no key singles one out, facet falls back to the first object arm;
/// - an array: the first list arm;
/// - a scalar: no members to walk.
///
/// Residual uncertainty: a map arm is taken to have no keys, flattened fields and
/// `alias` are matched as `find_field` matches them, and a newtype around an `Option` or a
/// pointer is not an object arm, as facet classifies it; these follow facet's source and are not
/// confirmed against every input.
fn untagged_arm(variants: &'static [facet::Variant], json: &Value) -> Option<usize> {
    if json.is_array() {
        return variants.iter().position(|v| {
            payload_shape(v)
                .is_some_and(|s| matches!(see_through(s).def, Def::List(_) | Def::Array(_)))
        });
    }
    let object = json.as_object()?;
    let arms: Vec<(usize, ObjectArm)> = variants
        .iter()
        .enumerate()
        .filter_map(|(index, variant)| match object_fields(variant) {
            ObjectArm::None => None,
            arm => Some((index, arm)),
        })
        .collect();
    let first = arms.first()?.0;
    let mut candidates: Vec<&(usize, ObjectArm)> = arms.iter().collect();
    for (key, member) in object {
        let with_key: Vec<&(usize, ObjectArm)> = candidates
            .iter()
            .copied()
            .filter(|(_, arm)| match arm {
                ObjectArm::Fields(fields) => {
                    !matches!(find_field(fields, key.as_str()), Lookup::Unknown)
                }
                ObjectArm::Map | ObjectArm::None => false,
            })
            .collect();
        if with_key.is_empty() {
            continue;
        }
        candidates = with_key;
        if let [(index, _)] = candidates.as_slice() {
            return Some(*index);
        }
        if let Some(taking) = scalar_narrowing(&candidates, key.as_str(), member) {
            candidates.retain(|(_, arm)| {
                key_shape(arm, key.as_str()).is_some_and(|s| taking.iter().any(|t| ptr::eq(*t, s)))
            });
            match candidates.as_slice() {
                [] => return Some(first), // facet's `SatisfyResult::NoMatch`
                [(index, _)] => return Some(*index),
                _ => {}
            }
        }
    }
    Some(first)
}

/// facet-solver `KeyResult::Ambiguous` and facet-format rc.7 `select_best_ambiguous_scalar_shapes`:
/// when the candidates' fields for `key` have more than one type (by shape) and the member is a
/// scalar, the field types that take it best: the best match quality (0 exact, 1 a string parsed
/// into a number or bool), then the lowest specificity score (`u8` before `u32` before `u64`).
/// `None` when the key is not ambiguous or the member is an object or array (facet then keeps
/// every candidate). An empty list means no candidate takes the value.
fn scalar_narrowing(
    candidates: &[&(usize, ObjectArm)],
    key: &str,
    member: &Value,
) -> Option<Vec<&'static Shape>> {
    let mut shapes: Vec<&'static Shape> = Vec::new();
    for (_, arm) in candidates {
        if let Some(shape) = key_shape(arm, key)
            && !shapes.iter().any(|s| ptr::eq(*s, shape))
        {
            shapes.push(shape);
        }
    }
    if shapes.len() < 2 {
        return None;
    }
    let scalar = Scalar::of(member)?;
    let rated: Vec<(&'static Shape, u8, u64)> = shapes
        .into_iter()
        .filter_map(|shape| {
            scalar
                .match_quality(shape)
                .map(|quality| (shape, quality, specificity_score(shape)))
        })
        .collect();
    let Some(quality) = rated.iter().map(|(_, q, _)| *q).min() else {
        return Some(Vec::new());
    };
    let score = rated
        .iter()
        .filter(|(_, q, _)| *q == quality)
        .map(|(_, _, s)| *s)
        .min();
    Some(
        rated
            .into_iter()
            .filter(|(_, q, s)| *q == quality && Some(*s) == score)
            .map(|(shape, _, _)| shape)
            .collect(),
    )
}

/// The type of the field an object arm reads `key` into (a flattened struct's field included).
fn key_shape(arm: &ObjectArm, key: &str) -> Option<&'static Shape> {
    fn in_fields(fields: &'static [Field], key: &str) -> Option<&'static Shape> {
        match find_field(fields, key) {
            Lookup::Field(_, field) => Some(field.shape()),
            Lookup::Flattened(..) => fields
                .iter()
                .filter(|f| f.is_flattened() && !f.should_skip_deserializing())
                .find_map(|f| in_fields(struct_fields(f.shape())?, key)),
            Lookup::Unknown => None,
        }
    }
    match arm {
        ObjectArm::Fields(fields) => in_fields(fields, key),
        ObjectArm::Map | ObjectArm::None => None,
    }
}

/// A JSON scalar as facet-json's parser hands it to the solver.
enum Scalar<'v> {
    Null,
    Bool,
    /// A non-negative integer token.
    U64(u64),
    /// A negative integer token.
    I64,
    /// A real token, or an integer token past 64 bits: the door reads that as the double it
    /// spells (`json/read.rs`).
    F64,
    /// A string.
    Str(Cow<'v, str>),
}

impl<'v> Scalar<'v> {
    /// The scalar `value` is, or `None` for an object or array.
    fn of(value: &'v Value) -> Option<Self> {
        if value.is_null() {
            return Some(Self::Null);
        }
        if value.as_bool().is_some() {
            return Some(Self::Bool);
        }
        if let Some(text) = value.as_string() {
            return Some(Self::Str(Cow::Borrowed(text.as_str())));
        }
        let number = value.as_number()?;
        Some(if number.is_float() {
            Self::F64
        } else if let Some(u) = number.to_u64() {
            Self::U64(u)
        } else if number.to_i64().is_some() {
            Self::I64
        } else {
            // An integer past 64 bits: no text read through the door gives one.
            Self::F64
        })
    }

    /// facet-format rc.7 `scalar_match_quality`: 0 for a direct match, 1 for a string that
    /// parses into the type, `None` when the type does not take the value. Only primitive scalar
    /// types take a scalar (an `Option` takes only `null`; a struct or proxied type none).
    fn match_quality(&self, shape: &'static Shape) -> Option<u8> {
        use facet::ScalarType as T;
        let Some(ty) = shape.scalar_type() else {
            return (matches!(self, Self::Null) && matches!(shape.def, Def::Option(_)))
                .then_some(0);
        };
        let takes = match self {
            Self::Null => matches!(ty, T::Unit),
            Self::Bool => matches!(ty, T::Bool),
            Self::F64 => matches!(ty, T::F32 | T::F64),
            // An unsigned token: any unsigned type, a signed one it fits.
            Self::U64(u) => match ty {
                T::U8 | T::U16 | T::U32 | T::U64 | T::U128 | T::USize => true,
                T::I8 => i8::try_from(*u).is_ok(),
                T::I16 => i16::try_from(*u).is_ok(),
                T::I32 => i32::try_from(*u).is_ok(),
                T::I64 | T::I128 | T::ISize => i64::try_from(*u).is_ok(),
                _ => false,
            },
            // A negative token: any signed type (facet checks no range), no unsigned one.
            Self::I64 => matches!(ty, T::I8 | T::I16 | T::I32 | T::I64 | T::I128 | T::ISize),
            Self::Str(text) => {
                if matches!(ty, T::String | T::Str | T::CowStr | T::Char) {
                    return Some(0);
                }
                return parses(ty, text).then_some(1);
            }
        };
        takes.then_some(0)
    }
}

/// Whether the type's `FromStr` (facet's parse function for these scalars) takes `text`.
fn parses(ty: facet::ScalarType, text: &str) -> bool {
    use facet::ScalarType as T;
    match ty {
        T::Bool => text.parse::<bool>().is_ok(),
        T::F32 => text.parse::<f32>().is_ok(),
        T::F64 => text.parse::<f64>().is_ok(),
        T::U8 => text.parse::<u8>().is_ok(),
        T::U16 => text.parse::<u16>().is_ok(),
        T::U32 => text.parse::<u32>().is_ok(),
        T::U64 => text.parse::<u64>().is_ok(),
        T::U128 => text.parse::<u128>().is_ok(),
        T::USize => text.parse::<usize>().is_ok(),
        T::I8 => text.parse::<i8>().is_ok(),
        T::I16 => text.parse::<i16>().is_ok(),
        T::I32 => text.parse::<i32>().is_ok(),
        T::I64 => text.parse::<i64>().is_ok(),
        T::I128 => text.parse::<i128>().is_ok(),
        T::ISize => text.parse::<isize>().is_ok(),
        // `()`, `ConstTypeId` and the network types: no field of a pack document has them.
        _ => false,
    }
}

/// facet-solver `specificity_score`: lower is a narrower type.
fn specificity_score(shape: &'static Shape) -> u64 {
    match shape.type_identifier {
        "u8" | "i8" => 8,
        "u16" | "i16" => 16,
        "u32" | "i32" | "f32" => 32,
        "u64" | "i64" | "f64" | "usize" | "isize" => 64,
        "u128" | "i128" => 128,
        _ => 1000,
    }
}

/// What an arm reads a JSON object as.
enum ObjectArm {
    /// A struct (a struct variant, or a newtype variant around a named struct).
    Fields(&'static [Field]),
    /// A map.
    Map,
    /// Not an object.
    None,
}

/// How facet classifies a variant for object input (facet-solver `VariantFormat::from_variant`:
/// `Struct`, or `NewtypeStruct` for a newtype around a named struct or a map).
fn object_fields(variant: &'static facet::Variant) -> ObjectArm {
    if variant.data.kind == facet::StructKind::Struct {
        return ObjectArm::Fields(variant.data.fields);
    }
    let Some(shape) = payload_shape(variant) else {
        return ObjectArm::None;
    };
    if shape.effective_proxy(None).is_some() {
        return ObjectArm::None;
    }
    if matches!(shape.def, Def::Map(_)) {
        return ObjectArm::Map;
    }
    struct_fields(shape).map_or(ObjectArm::None, ObjectArm::Fields)
}

/// The payload shape of a newtype variant.
fn payload_shape(variant: &'static facet::Variant) -> Option<&'static Shape> {
    match variant.data.fields {
        [field] if variant.data.kind != facet::StructKind::Struct => Some(field.shape()),
        _ => None,
    }
}

/// `shape` without `Option` and pointers.
fn see_through(mut shape: &'static Shape) -> &'static Shape {
    loop {
        shape = match shape.def {
            Def::Option(option) => option.t(),
            Def::Pointer(def) => match def.pointee() {
                Some(pointee) => pointee,
                None => return shape,
            },
            _ => return shape,
        };
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
    use indexmap::IndexMap;

    use super::*;

    #[derive(Debug, Facet)]
    struct Inner {
        #[facet(default)]
        v: u8,
        #[facet(skip)]
        extra: Extra,
    }

    #[derive(Debug, Facet)]
    struct Outer {
        #[facet(default)]
        list: Vec<Inner>,
        #[facet(default)]
        map: IndexMap<String, Inner>,
        #[facet(default)]
        boxed: Option<Box<Inner>>,
        #[facet(default)]
        tagged: Option<Tagged>,
    }

    #[derive(Debug, Facet)]
    #[repr(u8)]
    #[allow(dead_code)]
    enum Tagged {
        First { v: u8 },
        Second(Inner),
    }

    /// The paths of the unknown members of `text` read as `Outer`, joined by `/`.
    fn paths(text: &str) -> Vec<String> {
        let json: Value = crate::json::from_str(text).unwrap();
        walk(Outer::SHAPE, &json, None)
            .iter()
            .map(|u| u.path.to_string())
            .collect()
    }

    /// The typed value of a map entry is found once for the map, not looked for from the
    /// map's first entry at every entry.
    #[test]
    fn the_walk_of_a_typed_map_of_n_entries_costs_n() {
        crate::test_cost::assert_linear_at(
            "core unknown members: a typed map of n entries",
            15_000,
            |n| {
                let entries: Vec<String> = (0..n)
                    .map(|i| format!(r#""k_{i}": {{"v": 1, "zz": 2}}"#))
                    .collect();
                let text = format!(r#"{{"map": {{{}}}}}"#, entries.join(","));
                let typed: Outer = crate::json::from_str(&text).unwrap();
                let json: Value = crate::json::from_str(&text).unwrap();
                (typed, json, n)
            },
            |(typed, json, n)| {
                let unknown = walk(Outer::SHAPE, json, Some(Peek::new(typed)));
                assert_eq!(unknown.len(), *n);
                unknown
            },
        );
    }

    #[test]
    fn paths_name_list_indices_and_map_keys() {
        assert_eq!(
            paths(r#"{ "list": [{}, { "a": 1 }], "map": { "k": { "b": 2 } }, "c": 3 }"#),
            ["list/1/a", "map/k/b", "c"]
        );
    }

    #[test]
    fn paths_go_through_boxes_and_enum_variants() {
        assert_eq!(
            paths(r#"{ "boxed": { "a": 1 }, "tagged": { "Second": { "b": 2 } } }"#),
            ["boxed/a", "tagged/Second/b"]
        );
        assert_eq!(
            paths(r#"{ "tagged": { "First": { "c": 3 } } }"#),
            ["tagged/First/c"]
        );
    }

    #[test]
    fn only_structs_with_an_extra_field_reachable_mutably_get_a_route() {
        let json: Value = crate::json::from_str(
            r#"{ "list": [{ "a": 1 }], "map": { "k": { "b": 2 } }, "boxed": { "c": 3 }, "d": 4 }"#,
        )
        .unwrap();
        let routes: Vec<bool> = walk(Outer::SHAPE, &json, None)
            .iter()
            .map(|u| u.route.is_some())
            .collect();
        // list element: kept; map value, box: unreachable; `Outer` has no `extra`.
        assert_eq!(routes, [true, false, false, false]);
    }

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

    #[derive(Debug, Facet)]
    struct Scalars {
        #[facet(default)]
        float: f32,
        #[facet(default)]
        double: f64,
        #[facet(default)]
        whole: i32,
        #[facet(default)]
        small: u8,
        #[facet(default)]
        name: String,
        #[facet(default)]
        flag: bool,
        #[facet(default)]
        maybe: Option<f32>,
        #[facet(default)]
        list: Vec<f32>,
        #[facet(default)]
        map: IndexMap<String, Box<Scalars>>,
        #[facet(default)]
        strict: Option<crate::json::StrictInt<i32>>,
    }

    /// The scalars of `text` that `Scalars` reads as another value, as `path: kind`.
    fn misread(text: &str) -> Vec<String> {
        let json: Value = crate::json::from_str(text).unwrap();
        let typed: Scalars = crate::json::from_str(text).unwrap();
        let (_, misread) = keep_members(&typed, &json, &mut Table::new());
        misread
            .iter()
            .map(|m| format!("{}: {:?}", m.path, m.kind))
            .collect()
    }

    #[test]
    fn a_scalar_read_as_the_value_it_spells_is_no_misreading() {
        let none: [&str; 0] = [];
        assert_eq!(
            misread(
                r#"{"float": 1, "double": 1e39, "whole": -7, "small": 255, "name": "7",
                    "flag": true, "maybe": null, "list": [0.5, 3.4028235e38],
                    "map": {"a": {"whole": 2.0, "float": 4294967296}}, "strict": 1}"#
            ),
            none
        );
    }

    #[test]
    fn a_number_beyond_a_float_member_is_a_misreading_at_its_path() {
        assert_eq!(
            misread(
                r#"{"float": 1e39, "double": 1e39, "maybe": -1e39, "list": [0, 1e300],
                    "map": {"a": {"float": 3.5e38}}}"#
            ),
            [
                "float: BeyondFloat",
                "maybe: BeyondFloat",
                "list/1: BeyondFloat",
                "map/a/float: BeyondFloat"
            ]
        );
    }

    #[test]
    fn a_number_an_integer_member_does_not_hold_is_refused_by_the_typed_parse() {
        // Characterization: facet neither saturates nor truncates an integer member, so
        // there is no such misreading to tell.
        for text in [
            r#"{"whole": 0.5}"#,
            r#"{"whole": 1e39}"#,
            r#"{"whole": 4294967296}"#,
            r#"{"whole": -2147483649}"#,
            r#"{"small": 256}"#,
            r#"{"small": -1}"#,
            r#"{"small": 1.5}"#,
            r#"{"map": {"a": {"whole": 4294967296.0}}}"#,
        ] {
            assert!(crate::json::from_str::<Scalars>(text).is_err(), "{text}");
        }
    }

    #[test]
    fn a_scalar_of_another_json_type_is_a_misreading() {
        assert_eq!(
            misread(r#"{"float": "7", "whole": "7", "name": 7, "flag": "true"}"#),
            [
                "float: WrongType",
                "whole: WrongType",
                "name: WrongType",
                "flag: WrongType"
            ]
        );
    }

    #[derive(Debug, Default, Facet)]
    struct Leaf {
        #[facet(default)]
        whole: i32,
    }

    #[derive(Debug, Facet)]
    struct Nulls {
        #[facet(default)]
        list: Vec<Leaf>,
        #[facet(default)]
        one: Leaf,
        #[facet(default)]
        maybe: Option<Leaf>,
    }

    #[test]
    fn a_null_read_as_an_empty_struct_is_a_misreading() {
        // facet reads a `null` as a struct of defaults; an `Option` reads it as none, which
        // is what it spells.
        let misread = |text: &str| -> Vec<String> {
            let json: Value = crate::json::from_str(text).unwrap();
            let typed: Nulls = crate::json::from_str(text).unwrap();
            let (_, misread) = keep_members(&typed, &json, &mut Table::new());
            misread
                .iter()
                .map(|m| format!("{}: {:?}", m.path, m.kind))
                .collect()
        };
        let none: [&str; 0] = [];
        assert_eq!(misread(r#"{"maybe": null, "one": {}, "list": [{}]}"#), none);
        assert_eq!(
            misread(r#"{"list": [{"whole": 1}, null], "one": null}"#),
            ["list/1: WrongType", "one: WrongType"]
        );
    }

    #[derive(Debug, Default, Facet)]
    struct Ranged {
        #[facet(default)]
        interval: Option<crate::shared::FloatRange>,
        #[facet(default)]
        counts: Vec<crate::shared::IntRange>,
        #[facet(skip)]
        extra: Extra,
    }

    #[test]
    fn a_member_a_range_does_not_read_is_unknown_when_asked() {
        let text = r#"{"interval": {"a": 1, "min": 2}, "counts": [{"range_min": 1, "max": 3}, 4, [1, 2]]}"#;
        let json: Value = crate::json::from_str(text).unwrap();
        let typed: Ranged = crate::json::from_str(text).unwrap();
        assert!(
            keep_members(&typed, &json, &mut Table::new()).0.is_empty(),
            "not asked"
        );
        let mut kept = Table::new();
        let paths: Vec<String> = keep_members_and_range_members(&typed, &json, &mut kept)
            .0
            .iter()
            .map(|(path, _)| path.to_string())
            .collect();
        assert_eq!(paths, ["interval/a", "counts/0/max"]);
        assert!(kept.is_empty(), "a range has no extra: not kept");
    }

    #[test]
    fn an_element_a_range_given_as_a_list_does_not_read_is_unknown_when_asked() {
        // An integer range takes the first two numbers of a list; a
        // third went without a word.
        let text = r#"{"interval": [1, 2], "counts": [[1, 2], [3, 4, 5, 6]]}"#;
        let json: Value = crate::json::from_str(text).unwrap();
        let typed: Ranged = crate::json::from_str(text).unwrap();
        assert!(
            keep_members(&typed, &json, &mut Table::new()).0.is_empty(),
            "not asked"
        );
        let paths: Vec<String> = keep_members_and_range_members(&typed, &json, &mut Table::new())
            .0
            .iter()
            .map(|(path, _)| path.to_string())
            .collect();
        assert_eq!(paths, ["counts/1/2", "counts/1/3"]);
    }

    #[test]
    fn a_strict_member_is_its_own_to_judge() {
        // The proxy refuses `"1"`; what it takes is not looked at here.
        let none: [&str; 0] = [];
        assert_eq!(misread(r#"{"strict": 5}"#), none);
    }

    /// The stack of the walk: each case in a child process (a stack that overflows ends the
    /// process), on a thread of a stack the case chooses.
    #[allow(
        clippy::disallowed_methods,
        clippy::disallowed_types,
        reason = "the tests run each case in a child process, on a thread of a stack they choose"
    )]
    mod stack {
        use std::process::{Command, Stdio};

        use facet::Facet;
        use indexmap::IndexMap;

        use super::super::{Peek, Table, Value, keep_unknown_members, walk_on_this_stack};
        use crate::json::MAX_JSON_DEPTH;
        use crate::json::stack::to_stay;

        /// An untagged enum that holds itself in a box and in a list: the costliest shape a
        /// level of the walk.
        #[derive(Facet)]
        #[facet(untagged)]
        #[repr(u8)]
        #[allow(dead_code, reason = "read, never looked at")]
        enum Either {
            Many(Vec<Either>),
            One {
                #[facet(default)]
                a: Option<Box<Either>>,
            },
        }

        /// A struct that holds itself in a box.
        #[derive(Facet)]
        struct Chain {
            #[facet(default)]
            a: Option<Box<Chain>>,
        }

        /// A struct that holds itself in a list.
        #[derive(Facet)]
        struct Tree {
            #[facet(default)]
            a: Option<Vec<Tree>>,
        }

        /// A struct that holds itself in a map of options.
        #[derive(Facet)]
        struct Named {
            #[facet(default)]
            a: Option<IndexMap<String, Option<Named>>>,
        }

        /// A struct that holds itself in a map of maps.
        #[derive(Facet)]
        struct Mapped {
            #[facet(default)]
            a: Option<IndexMap<String, IndexMap<String, Option<Mapped>>>>,
        }

        /// `open` `levels` times around `bottom`, closed.
        fn nest(levels: usize, open: fn(usize) -> &'static str, bottom: &str) -> String {
            let mut text = String::new();
            for level in 0..levels {
                text.push_str(open(level));
            }
            text.push_str(bottom);
            for level in (0..levels).rev() {
                text.push(if open(level).starts_with('[') {
                    ']'
                } else {
                    '}'
                });
            }
            text
        }

        fn objects(_: usize) -> &'static str {
            "{\"a\":"
        }

        fn mixed(level: usize) -> &'static str {
            if level.is_multiple_of(2) {
                "{\"a\":"
            } else {
                "["
            }
        }

        /// The bottom of a text of [`mixed`] levels: an object ends in `null`, a list in
        /// nothing.
        fn mixed_bottom(levels: usize) -> &'static str {
            if levels.is_multiple_of(2) { "" } else { "null" }
        }

        /// How a case walks: through the entry point, which gives the walk its stack, or
        /// the walk itself on the thread's own.
        #[derive(Clone, Copy)]
        enum How {
            Entry,
            OnThisStack,
        }

        /// `T` and a `Value` read from `text` through the door, and walked; whether the
        /// walk found no unknown member, which the texts have none of.
        fn walks<T: for<'f> Facet<'f>>(text: &str, how: How) -> bool {
            let typed: T = crate::json::from_str(text).expect("the typed value");
            let json: Value = crate::json::from_str(text).expect("the value");
            let found = match how {
                How::Entry => keep_unknown_members(&typed, &json, &mut Table::new()).len(),
                How::OnThisStack => walk_on_this_stack(
                    T::SHAPE,
                    &json,
                    Some(Peek::new(&typed)),
                    true,
                    None,
                    true,
                    false,
                )
                .unknowns
                .len(),
            };
            // What nests is dropped by the process ending, not by a call for each level.
            std::mem::forget((typed, json));
            found == 0
        }

        /// A shape by name, and its walk of a text `levels` deep.
        type Shaped = (&'static str, fn(usize, How) -> bool);

        const SHAPES: &[Shaped] = &[
            (
                "an untagged enum that holds itself in a box",
                |levels, how| walks::<Either>(&nest(levels, objects, "null"), how),
            ),
            (
                "an untagged enum that holds itself in a list",
                |levels, how| walks::<Either>(&nest(levels, mixed, mixed_bottom(levels)), how),
            ),
            ("a struct that holds itself in a box", |levels, how| {
                walks::<Chain>(&nest(levels, objects, "null"), how)
            }),
            ("a struct that holds itself in a list", |levels, how| {
                walks::<Tree>(&nest(levels, mixed, mixed_bottom(levels)), how)
            }),
            (
                "a struct that holds itself in a map of options",
                |levels, how| walks::<Named>(&nest(levels, objects, "null"), how),
            ),
            (
                "a struct that holds itself in a map of maps",
                |levels, how| {
                    let open = |level: usize| {
                        if level.is_multiple_of(3) {
                            "{\"a\":"
                        } else {
                            "{\"k\":"
                        }
                    };
                    // An empty map ends the text where a `null` would be a map's value.
                    let text = if levels % 3 == 2 {
                        nest(levels - 1, open, "{}")
                    } else {
                        nest(levels, open, "null")
                    };
                    walks::<Mapped>(&text, how)
                },
            ),
        ];

        /// The child of the tests below: `<shape> <levels> <stack in bytes> <0: through the
        /// entry point, 1: on the thread's own stack>`. Without the variable it is no child.
        #[test]
        #[ignore = "no test of its own: the child process of the tests below, which run it with --ignored"]
        fn child() {
            let Ok(case) = std::env::var("JSON_WALK_STACK") else {
                return;
            };
            let parts: Vec<usize> = case.split(' ').map(|part| part.parse().unwrap()).collect();
            let (shape, levels, stack) = (parts[0], parts[1], parts[2]);
            let how = if parts[3] == 0 {
                How::Entry
            } else {
                How::OnThisStack
            };
            let (_, run) = SHAPES[shape];
            let ended = std::thread::Builder::new()
                .stack_size(stack)
                .spawn(move || run(levels, how))
                .unwrap()
                .join()
                .unwrap();
            assert!(
                ended,
                "JSON_WALK_STACK: the walk found a member the text has not"
            );
        }

        /// How the child of one case ended: well, or with what it said.
        fn in_a_child(shape: usize, levels: usize, stack: usize, how: How) -> Result<(), String> {
            let output = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "json::unknown::tests::stack::child",
                    "--ignored",
                    "--test-threads=1",
                    "--nocapture",
                ])
                .env(
                    "JSON_WALK_STACK",
                    format!("{shape} {levels} {stack} {}", how as usize),
                )
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .output()
                .unwrap();
            if output.status.success() {
                return Ok(());
            }
            let said = String::from_utf8_lossy(&output.stderr);
            let why = said
                .lines()
                .find(|line| line.contains("overflow") || line.contains("JSON_WALK_STACK"))
                .unwrap_or("no message");
            Err(format!("{} ({why})", output.status))
        }

        /// Every case of `stack_of` levels, through `how`; the ones that did not end well.
        fn failures(how: How, stack_of: fn(usize) -> usize) -> Vec<String> {
            let mut failures = Vec::new();
            for (shape, (name, _)) in SHAPES.iter().enumerate() {
                for levels in [1, 16, MAX_JSON_DEPTH] {
                    if let Err(ended) = in_a_child(shape, levels, stack_of(levels), how) {
                        failures.push(format!("{name}, {levels} levels: {ended}"));
                    }
                }
            }
            failures
        }

        /// The walk does not run on what its caller happens to have left: through an entry
        /// point it ends well on a thread of a quarter of a megabyte, at one level, at the
        /// depth of an ordinary file and at the limit, whatever the shape.
        #[test]
        fn the_walk_of_a_value_nested_to_the_limit_ends_well_on_a_small_stack() {
            let failures = failures(How::Entry, |_| 256 * 1024);
            assert!(failures.is_empty(), "{}", failures.join("\n"));
        }

        /// What a thread must have left for the walk to stay on it is more than twice what
        /// the walk takes, in the build at hand: each shape ends well on half of it, the
        /// walk run on the thread's own stack.
        #[test]
        fn a_walk_that_stays_on_the_thread_s_stack_has_twice_what_it_takes() {
            let failures = failures(How::OnThisStack, |levels| to_stay(levels) / 2);
            assert!(failures.is_empty(), "{}", failures.join("\n"));
        }
    }
}
