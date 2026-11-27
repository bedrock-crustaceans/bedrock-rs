//! Where the derived structs of a particle document meet the hand reader: the two eras as the
//! schema reader's [`schema::Era`]s, the reading of each, particle's own read of Molang (the
//! read's [`MolangLeaf`]), and the bridges a struct read by the derive and a value read by hand
//! cross into each other with.

#![allow(
    clippy::single_match_else,
    reason = "the two arms are the two outcomes of a read: the value, or the line about it"
)]

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, CodeDef, Diagnostic, Diagnostics, Severity};
use bedrock_addon_core::json::kit::{self, Profile, Read, Texts, UnknownAt, WrongAt};
use bedrock_addon_core::json::{EncodeError, Value};
use bedrock_addon_core::molang::{
    JsonScalar, Molang, MolangFailure, MolangFieldSpec, MolangFields, MolangSource, MolangVisitor,
    RawVersion, ReadField, ReaderKind, json_scalar,
};
use bedrock_addon_core::schema::{self, FromJson, MolangLeaf, Unknowns, Writing};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{BeyondWidth, MemberFailure, NullHandling, Reading, Scalars};

use crate::codes;
use crate::read::{CEREAL, Era, KEPT, Node, Opt, Reader, Writer};

/// The documents the legacy parser reads: below 1.20.80.
pub(crate) const LEGACY: schema::Era = schema::Era::below(CEREAL);
/// The documents the 1.20.80 reader reads.
pub(crate) const STRICT: schema::Era = schema::Era::since(CEREAL);

/// The Molang spec of a legacy field whose default is `default`: a value that does not compile
/// becomes that constant (the parser's lines are kept errors and the effect loads).
pub(crate) const fn legacy(default: f32) -> MolangFieldSpec {
    KEPT.reader(ReaderKind::SchemaValidated)
        .compile_empty()
        .on_failure(MolangFailure::ReplaceWith(default))
}

/// A legacy Molang field of default 0.
pub(crate) const L0: MolangFieldSpec = legacy(0.0);
/// A legacy Molang field of default 1.
pub(crate) const L1: MolangFieldSpec = legacy(1.0);

/// The legacy spec of a Molang member of an event: the parser reads it only from a string,
/// says nothing about another value, and keeps an expression that does not compile.
pub(crate) const EVENT: MolangFieldSpec = KEPT.reader(ReaderKind::SchemaValidated).compile_empty();

/// The name the pack's Molang version is in the scope of a read under.
const MOLANG_VERSION: &str = "particle/molang_version";

impl Era {
    /// The format a read of the era knows: the one the era's keys and specs are chosen by.
    pub(crate) fn format(self) -> FormatVersion {
        match self {
            Era::Legacy => known::V1_10_0,
            Era::Cereal => CEREAL,
        }
    }

    /// The era of a read or a walk of `format`; one of no format is the 1.20.80 reader's (the
    /// walk of a value built by hand).
    pub(crate) fn of(format: Option<FormatVersion>) -> Self {
        if LEGACY.holds(format) {
            Era::Legacy
        } else {
            Era::Cereal
        }
    }

    /// How the era's reader reads a derived struct: a `null` member is a value (each leaf says
    /// what it makes of it), scalars strictly, a float beyond its width the largest finite one
    /// with an information; the legacy parser goes on with a member's default after its line,
    /// the 1.20.80 reader refuses the document.
    fn derived_reading(self) -> Reading {
        let reading = match self {
            Era::Legacy => Reading::OWN.with_on_member_failure(MemberFailure::DefaultAndTell),
            // An error, and the member's default: the document is refused, the value read
            // whole as far as it goes.
            Era::Cereal => Reading::DOCUMENT
                .with_unknown(Severity::Warning)
                .with_on_member_failure(MemberFailure::DefaultAndRefuse),
        };
        reading
            .with_null(NullHandling::AsValue)
            .with_scalars(Scalars::Strict)
            .with_beyond_width(BeyondWidth::Finite)
    }
}

/// A value of the wrong type in a legacy document whose field states no line of the game's:
/// the crate's own information.
fn invalid_value(_: &bedrock_addon_core::diagnostic::JsonPath) -> Code {
    Code::from(&core_codes::INVALID_VALUE)
}

/// A read of a derived struct under `era`, telling into `d`: the members in declaration order,
/// then the unknown ones (told at `unknown`, kept nowhere), Molang by [`LEAF`].
pub(crate) fn kit_read(
    era: Era,
    molang_version: RawVersion,
    unknown: Severity,
    d: &mut Diagnostics,
) -> Read<'_> {
    let texts = match era {
        Era::Legacy => Texts {
            wrong_at: Some(WrongAt(invalid_value)),
            ..Texts::CODES
        },
        Era::Cereal => Texts::CODES,
    };
    let mut read = Read::new(era.derived_reading(), d)
        .profile(Profile::DECLARED_ORDER.with_unknown(UnknownAt::Last))
        .texts(texts)
        .format(Some(era.format()))
        .molang_leaf(LEAF)
        .unknown_severity(match era {
            Era::Legacy => Severity::Info,
            Era::Cereal => unknown,
        });
    read.put_in_scope(MOLANG_VERSION, molang_version);
    read
}

/// The pack's Molang version a read is under.
fn molang_version(read: &Read<'_>) -> RawVersion {
    read.scope::<RawVersion>(MOLANG_VERSION)
        .copied()
        .unwrap_or(RawVersion(0))
}

impl<'d> Reader<'d> {
    /// The hand reader of a value below a derived struct, at `node`, telling into `read`'s
    /// findings under its era.
    pub(crate) fn of(node: &kit::Node<'_>, read: &'d mut Read<'_>) -> Self {
        let era = Era::of(read.known_format());
        let version = molang_version(read);
        let unknown = read.unknown_grade();
        Reader::new(era, version, node.path().clone(), read.findings_mut()).grading(unknown)
    }
}

/// The read of a derived struct from `value` at the reader's path: `None`, with the field's
/// line, for a value that is no object (`o.wrong`), and where the 1.20.80 reader refused a
/// member.
pub(crate) fn read_derived<T: FromJson>(value: &Value, r: &mut Reader<'_>, o: Opt) -> Option<T> {
    if !value.is_object() {
        r.wrong_type(value, o.wrong);
        return None;
    }
    let (era, version, unknown, path) = r.state();
    let mut scratch = Diagnostics::default();
    let d = r.findings(&mut scratch);
    let mut read = kit_read(era, version, unknown, d);
    let read_value = T::from_json(&kit::Node::new(value, path), &mut read);
    read.finish();
    read_value
}

/// A member of a derived struct read by hand (`schema::read_with = hand`): the hand reader's
/// read of its value, with no line of the field's own.
pub(crate) fn hand<T: Node>(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<T> {
    hand_with(node, read, Opt::NONE)
}

/// [`hand`] with the field's `o`.
pub(crate) fn hand_with<T: Node>(node: &kit::Node<'_>, read: &mut Read<'_>, o: Opt) -> Option<T> {
    if read.stopped() {
        return None;
    }
    T::read(node.value(), &mut Reader::of(node, read), o)
}

/// The JSON of a derived value at the writer's path, by the generic writer under the era's
/// rules: a legacy document takes a Molang bool and an unknown member not named like a member
/// of its struct, a 1.20.80 document neither.
pub(crate) fn write_derived<'f, T: facet::Facet<'f>>(
    value: &T,
    w: &Writer,
) -> Result<Value, EncodeError> {
    let writing = if w.legacy() {
        Writing::REAL.unknown(Unknowns::NoneNamedKnown)
    } else {
        Writing::REAL
            .unknown(Unknowns::Refused)
            .refusing_molang_bools()
    };
    schema::write_value_with(value, writing).map_err(|error| match error {
        EncodeError::NotRepresentable { at, what } => EncodeError::NotRepresentable {
            at: w.path().join(&at),
            what,
        },
        other => other,
    })
}

/// The Molang walk of a derived struct in `era`.
pub(crate) fn walk_derived<T: MolangFields>(value: &mut T, era: Era, v: &mut MolangVisitor<'_>) {
    v.at_format(Some(era.format()), |v| value.visit_molang(v));
}

/// The Molang walk of a value read by hand below a derived struct: in the era of the walk's
/// format, its default the one of the spec the field walks it under (`ReplaceWith`).
pub(crate) fn walk_hand<T: Node>(value: &mut T, v: &mut MolangVisitor<'_>) {
    let o = Opt::with_default(replaced(Some(*v.default_spec())));
    value.walk(Era::of(v.format()), o, v);
}

/// The value a spec replaces a failed expression with; 0 for one that replaces none.
fn replaced(spec: Option<MolangFieldSpec>) -> f32 {
    match spec.map(|spec| spec.on_failure) {
        Some(MolangFailure::ReplaceWith(default)) => default,
        _ => 0.0,
    }
}

/// Particle's own read of Molang: the legacy parser's (a number, a bool, a string at the pack's
/// Molang version; another value logged and read as the member's default) and the 1.20.80
/// reader's (the versioned object form).
pub(crate) const LEAF: MolangLeaf = MolangLeaf {
    one: molang_member,
    array: molang_array,
};

/// A content-log line of vanilla's at `node`: the document loads, so the finding is a kept
/// error.
fn line(read: &mut Read<'_>, node: &kit::Node<'_>, code: &'static CodeDef<0>) {
    read.push(Diagnostic::of(code).at(node.path().clone()).into_kept());
}

/// A finding of the crate's own at `node`, at the information level.
fn note(read: &mut Read<'_>, node: &kit::Node<'_>, code: &'static CodeDef<0>) {
    read.push(
        Diagnostic::error(Code::from(code))
            .severity(Severity::Info)
            .at(node.path().clone()),
    );
}

/// The `f32` of a JSON number; one whose nearest `f32` is infinite is the largest finite float
/// of its sign, with an information.
fn float(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    let number = node.value().as_number()?.to_f64_lossy();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the nearest f32 is the reading"
    )]
    let narrowed = number as f32;
    if narrowed.is_finite() {
        return Some(narrowed);
    }
    note(read, node, &core_codes::INVALID_VALUE);
    Some(if number < 0.0 { f32::MIN } else { f32::MAX })
}

fn molang_member(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<Molang> {
    let spec = read.member_molang(node);
    molang(node, read, spec, &codes::NOT_MOLANG)
}

/// One Molang value under the member's `spec` (`None` for a value that is no member of a
/// derived struct: a list's element), `code` the legacy parser's line for a value it cannot
/// read.
fn molang(
    node: &kit::Node<'_>,
    read: &mut Read<'_>,
    spec: Option<MolangFieldSpec>,
    code: &'static CodeDef<0>,
) -> Option<Molang> {
    let era = Era::of(read.known_format());
    let version = molang_version(read);
    // A legacy member whose spec replaces nothing is an event's.
    let event = era == Era::Legacy
        && spec.is_some_and(|spec| !matches!(spec.on_failure, MolangFailure::ReplaceWith(_)));
    let value = node.value();
    if value.is_number() && !event {
        return float(node, read).map(Molang::Const);
    }
    match era {
        Era::Legacy if event => match json_scalar(value) {
            JsonScalar::String(text) => Some(Molang::Expr(MolangSource::string(text, version))),
            _ => {
                // Read as the member's default, 0, without a line of the game's: the crate
                // notes it at the information level. A `null` member is an absent one.
                if !value.is_null() {
                    note(read, node, &core_codes::INVALID_TYPE);
                }
                None
            }
        },
        Era::Legacy => Some(match json_scalar(value) {
            JsonScalar::Bool(b) => Molang::Bool(b),
            JsonScalar::String(text) => Molang::Expr(MolangSource::string(text, version)),
            _ => {
                line(read, node, code);
                Molang::Const(replaced(spec))
            }
        }),
        Era::Cereal => {
            match bedrock_addon_core::molang::read(value, ReaderKind::StrictVersioned, version) {
                Ok(ReadField::Value(value)) => Some(value),
                _ => {
                    read.wrong_type(node);
                    None
                }
            }
        }
    }
}

/// A `[Molang; len]` member: the legacy parser takes an array of exactly `len`, or logs "error
/// reading array" and leaves every element at the default.
fn molang_array(node: &kit::Node<'_>, read: &mut Read<'_>, len: usize) -> Option<Vec<Molang>> {
    let spec = read.member_molang(node);
    let default = Molang::Const(replaced(spec));
    let legacy = Era::of(read.known_format()) == Era::Legacy;
    let fits = node
        .value()
        .as_array()
        .is_some_and(|array| array.len() == len);
    if !fits {
        if legacy {
            line(read, node, &codes::ARRAY_ERROR);
            return Some(vec![default; len]);
        }
        read.wrong_type(node);
        return None;
    }
    let elements = read.array(node)?;
    let mut complete = true;
    let mut values = Vec::with_capacity(len);
    for element in elements.iter() {
        if legacy && element.value().is_null() {
            line(read, &element, &codes::ARRAY_ELEMENT_MISSING);
            values.push(default.clone());
            continue;
        }
        let value = molang(&element, read, spec, &codes::ARRAY_ELEMENT_NOT_MOLANG);
        complete &= value.is_some();
        values.push(value.unwrap_or_else(|| default.clone()));
    }
    complete.then_some(values)
}

/// A value read by hand below derived structs: its Molang walk ([`walk_hand`]) and its leaf
/// of the schema check.
macro_rules! hand_leaf {
    ($($ty:ty),* $(,)?) => {
        $(
            impl ::bedrock_addon_core::molang::MolangFields for $ty {
                fn visit_molang(&mut self, v: &mut ::bedrock_addon_core::molang::MolangVisitor<'_>) {
                    $crate::derived::walk_hand(self, v);
                }
            }
        )*
        ::bedrock_addon_core::impl_schema_leaf!($($ty),*);
    };
}
pub(crate) use hand_leaf;
