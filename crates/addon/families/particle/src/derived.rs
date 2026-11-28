//! The read of a particle document: the two eras as the schema reader's
//! [`schema::VersionRange`]s and the kit's read of each, particle's own read of Molang (the
//! read's [`MolangLeaf`]), and the helpers the leaves the derive has no form for (read with
//! `schema::read_with`, or a [`FromJson`] of their own) tell the game's lines with.

#![allow(
    clippy::single_match_else,
    reason = "the two arms are the two outcomes of a read: the value, or the line about it"
)]

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, CodeDef, Diagnostic, Diagnostics, JsonPath, Severity};
use bedrock_addon_core::json::kit::{self, Profile, Read, Texts, UnknownAt, WrongAt};
use bedrock_addon_core::molang::{
    JsonScalar, Molang, MolangFailure, MolangFieldMut, MolangFieldSpec, MolangFields, MolangSource,
    MolangVisitor, RawVersion, ReadField, ReaderKind, Step, json_scalar,
};
use bedrock_addon_core::schema::{self, FromJson, MolangLeaf};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{BeyondWidth, MemberFailure, NullHandling, Reading, Scalars};
use indexmap::IndexMap;

use crate::codes;
use crate::read::CEREAL;

/// Every particle Molang field: its parser lines are kept errors, the effect loads.
pub(crate) const KEPT: MolangFieldSpec = MolangFieldSpec::KEPT;

/// The documents the legacy parser reads: below 1.20.80.
pub(crate) const LEGACY: schema::VersionRange = schema::VersionRange::below(CEREAL);
/// The documents the 1.20.80 reader reads.
pub(crate) const STRICT: schema::VersionRange = schema::VersionRange::since(CEREAL);

/// The Molang spec of a legacy field whose default is `default`: a value that does not compile
/// becomes that constant (the parser's lines are kept errors and the effect loads).
pub(crate) const fn legacy_spec(default: f32) -> MolangFieldSpec {
    KEPT.reader(ReaderKind::SchemaValidated)
        .compile_empty()
        .on_failure(MolangFailure::ReplaceWith(default))
}

/// A legacy Molang field of default 0.
pub(crate) const L0: MolangFieldSpec = legacy_spec(0.0);
/// A legacy Molang field of default 1.
pub(crate) const L1: MolangFieldSpec = legacy_spec(1.0);

/// The legacy spec of a Molang member of an event: the parser reads it only from a string,
/// says nothing about another value, and keeps an expression that does not compile.
pub(crate) const EVENT: MolangFieldSpec = KEPT.reader(ReaderKind::SchemaValidated).compile_empty();

/// The name the pack's Molang version is in the scope of a read under.
const MOLANG_VERSION: &str = "particle/molang_version";

/// The format a read of the era knows: the one the era's keys and specs are chosen by.
pub(crate) fn format(legacy: bool) -> FormatVersion {
    if legacy { known::V1_10_0 } else { CEREAL }
}

/// How the era's reader reads: a `null` member is a value (each leaf says what it makes of
/// it), scalars strictly, a float beyond its width the largest finite one with an information;
/// the legacy parser goes on with a member's default after its line, the 1.20.80 reader
/// refuses the document.
fn reading(legacy: bool) -> Reading {
    let reading = if legacy {
        Reading::OWN.with_on_member_failure(MemberFailure::DefaultAndTell)
    } else {
        // An error, and the member's default: the document is refused, the value read whole
        // as far as it goes.
        Reading::DOCUMENT
            .with_unknown(Severity::Warning)
            .with_on_member_failure(MemberFailure::DefaultAndRefuse)
    };
    reading
        .with_null(NullHandling::AsValue)
        .with_scalars(Scalars::Strict)
        .with_beyond_width(BeyondWidth::Finite)
}

/// A value of the wrong type in a legacy document whose field states no line of the game's:
/// the crate's own information.
fn invalid_value(_: &bedrock_addon_core::diagnostic::JsonPath) -> Code {
    Code::from(&core_codes::INVALID_VALUE)
}

/// The read of a particle document of the era (`legacy` the legacy parser's), telling into
/// `d`: the members in declaration order, then the unknown ones (told at `unknown` in a
/// 1.20.80 document, as an information in a legacy one, kept nowhere), Molang by [`LEAF`] at
/// the pack's `molang_version`.
pub(crate) fn kit_read(
    legacy: bool,
    molang_version: RawVersion,
    unknown: Severity,
    d: &mut Diagnostics,
) -> Read<'_> {
    let texts = if legacy {
        Texts {
            wrong_at: Some(WrongAt(invalid_value)),
            ..Texts::CODES
        }
    } else {
        Texts::CODES
    };
    let mut read = Read::new(reading(legacy), d)
        .profile(Profile::DECLARED_ORDER.with_unknown(UnknownAt::Last))
        .texts(texts)
        .format(Some(format(legacy)))
        .molang_leaf(LEAF)
        .unknown_severity(if legacy { Severity::Info } else { unknown });
    read.put_in_scope(MOLANG_VERSION, molang_version);
    read
}

/// The pack's Molang version a read is under.
fn molang_version(read: &Read<'_>) -> RawVersion {
    read.scope::<RawVersion>(MOLANG_VERSION)
        .copied()
        .unwrap_or(RawVersion(0))
}

/// Whether `read` reads a document of the legacy parser's.
pub(crate) fn legacy(read: &Read<'_>) -> bool {
    LEGACY.holds(read.known_format())
}

/// A finding of the crate's own at `at`, at the information level.
pub(crate) fn note(read: &mut Read<'_>, at: &JsonPath, code: &'static CodeDef<0>) {
    read.push(
        Diagnostic::error(Code::from(code))
            .severity(Severity::Info)
            .at(at.clone()),
    );
}

/// A content-log line of vanilla's at `at`, written by the legacy parser and by the reader of
/// 1.20.80 documents: the document loads and the value takes its fallback, so the finding is
/// a kept error.
pub(crate) fn line(read: &mut Read<'_>, at: &JsonPath, code: &'static CodeDef<0>) {
    read.push(Diagnostic::of(code).at(at.clone()).into_kept());
}

/// `node` is of the wrong JSON type: the legacy parser logs `code` (the crate notes it, at the
/// information level, when the parser has no line) and the caller takes the default; a
/// 1.20.80 document is refused.
pub(crate) fn wrong(read: &mut Read<'_>, node: &kit::Node<'_>, code: Option<&'static CodeDef<0>>) {
    match code {
        _ if !legacy(read) => read.wrong_type(node),
        Some(code) => line(read, node.path(), code),
        None => note(read, node.path(), &core_codes::INVALID_VALUE),
    }
}

/// `T` read from `node`, which must be an object: `code` the legacy parser's line for one that
/// is none.
pub(crate) fn object<T: FromJson>(
    node: &kit::Node<'_>,
    read: &mut Read<'_>,
    code: Option<&'static CodeDef<0>>,
) -> Option<T> {
    if !node.value().is_object() {
        wrong(read, node, code);
        return None;
    }
    T::from_json(node, read)
}

/// The entries of the object at `node`, each read by `entry`, in file order; one that cannot be
/// used is left out (the legacy parser goes on without it; a 1.20.80 document is refused by
/// its finding). `code` is the legacy parser's line for a value that is no object.
pub(crate) fn entries<T>(
    node: &kit::Node<'_>,
    read: &mut Read<'_>,
    code: Option<&'static CodeDef<0>>,
    mut entry: impl FnMut(&str, &kit::Node<'_>, &mut Read<'_>) -> Option<T>,
) -> Option<IndexMap<String, T>> {
    if !node.value().is_object() {
        wrong(read, node, code);
        return None;
    }
    let members = read.object(node)?;
    let mut out = IndexMap::new();
    for (key, member) in members.iter() {
        if let Some(value) = entry(key, &member, read) {
            out.insert(key.to_owned(), value);
        }
    }
    Some(out)
}

/// The elements of the array at `node`, each read by `element`; one that cannot be used is left
/// out, and what is told of the others from here on is at the index the file has. `code` is
/// the legacy parser's line for a value that is no array.
pub(crate) fn elements<T>(
    node: &kit::Node<'_>,
    read: &mut Read<'_>,
    code: Option<&'static CodeDef<0>>,
    mut element: impl FnMut(&kit::Node<'_>, &mut Read<'_>) -> Option<T>,
) -> Option<Vec<T>> {
    if !node.value().is_array() {
        wrong(read, node, code);
        return None;
    }
    let elements = read.array(node)?;
    let mut out = Vec::with_capacity(elements.len());
    let mut places = Vec::with_capacity(elements.len());
    for (index, at) in elements.iter().enumerate() {
        if let Some(value) = element(&at, read) {
            out.push(value);
            places.push(index);
        }
    }
    read.findings_mut().left_out(node.path(), &places);
    Some(out)
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

/// The `f32` of a JSON number; one whose nearest `f32` is infinite is the largest finite float
/// of its sign, with an information.
pub(crate) fn float(node: &kit::Node<'_>, read: &mut Read<'_>) -> Option<f32> {
    let number = node.value().as_number()?.to_f64_lossy();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the nearest f32 is the reading"
    )]
    let narrowed = number as f32;
    if narrowed.is_finite() {
        return Some(narrowed);
    }
    note(read, node.path(), &core_codes::INVALID_VALUE);
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
    let legacy = legacy(read);
    let version = molang_version(read);
    // A legacy member whose spec replaces nothing is an event's.
    let event = legacy
        && spec.is_some_and(|spec| !matches!(spec.on_failure, MolangFailure::ReplaceWith(_)));
    let value = node.value();
    if value.is_number() && !event {
        return float(node, read).map(Molang::Const);
    }
    if !legacy {
        return match bedrock_addon_core::molang::read(value, ReaderKind::StrictVersioned, version) {
            Ok(ReadField::Value(value)) => Some(value),
            _ => {
                read.wrong_type(node);
                None
            }
        };
    }
    match json_scalar(value) {
        JsonScalar::String(text) => Some(Molang::Expr(MolangSource::string(text, version))),
        JsonScalar::Bool(b) if !event => Some(Molang::Bool(b)),
        // Read as the member's default, 0, without a line of the game's: the crate notes it
        // at the information level. A `null` member is an absent one.
        _ if event => {
            if !value.is_null() {
                note(read, node.path(), &core_codes::INVALID_TYPE);
            }
            None
        }
        _ => {
            line(read, node.path(), code);
            Some(Molang::Const(replaced(spec)))
        }
    }
}

fn molang_array(node: &kit::Node<'_>, read: &mut Read<'_>, len: usize) -> Option<Vec<Molang>> {
    let spec = read.member_molang(node);
    molangs(node, read, len, spec)
}

/// A `[Molang; len]` under `spec`: the legacy parser takes an array of exactly `len`, or logs
/// "error reading array" and leaves every element at the default.
fn molangs(
    node: &kit::Node<'_>,
    read: &mut Read<'_>,
    len: usize,
    spec: Option<MolangFieldSpec>,
) -> Option<Vec<Molang>> {
    let default = Molang::Const(replaced(spec));
    let legacy = legacy(read);
    let fits = node
        .value()
        .as_array()
        .is_some_and(|array| array.len() == len);
    if !fits {
        if legacy {
            line(read, node.path(), &codes::ARRAY_ERROR);
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
            line(read, element.path(), &codes::ARRAY_ELEMENT_MISSING);
            values.push(default.clone());
            continue;
        }
        let value = molang(&element, read, spec, &codes::ARRAY_ELEMENT_NOT_MOLANG);
        complete &= value.is_some();
        values.push(value.unwrap_or_else(|| default.clone()));
    }
    complete.then_some(values)
}

/// One Molang value of a leaf read by hand, whose legacy default is `default`.
pub(crate) fn molang_of(node: &kit::Node<'_>, read: &mut Read<'_>, default: f32) -> Option<Molang> {
    molang(node, read, Some(legacy_spec(default)), &codes::NOT_MOLANG)
}

/// `[Molang; N]` of a leaf read by hand, whose legacy default is `default`.
pub(crate) fn molang_array_of<const N: usize>(
    node: &kit::Node<'_>,
    read: &mut Read<'_>,
    default: f32,
) -> Option<[Molang; N]> {
    let values = molangs(node, read, N, Some(legacy_spec(default)))?;
    values.try_into().ok()
}

/// A string of a leaf read by hand; `code` the legacy parser's line for a value that is none.
pub(crate) fn string(
    node: &kit::Node<'_>,
    read: &mut Read<'_>,
    code: Option<&'static CodeDef<0>>,
) -> Option<String> {
    let text = node
        .value()
        .as_string()
        .map(|text| text.as_str().to_owned());
    if text.is_none() {
        wrong(read, node, code);
    }
    text
}

/// A finding at `at` that refuses the document.
pub(crate) fn refuse(read: &mut Read<'_>, at: &JsonPath, code: Code) {
    read.push(
        Diagnostic::error(code)
            .severity(Severity::Error)
            .at(at.clone()),
    );
}

/// `code` as a kept error at `at`: a content-log line the game writes at the error level
/// before the effect loads.
pub(crate) fn kept(read: &mut Read<'_>, at: &JsonPath, code: Code) {
    read.push(Diagnostic::error(code).at(at.clone()).into_kept());
}

/// Takes back what was told at or below `at`, for a member the legacy parser never reads (it
/// returned before it); then notes that it is left out, a finding of the crate's own: an
/// information.
pub(crate) fn unread(read: &mut Read<'_>, at: &JsonPath) {
    read.findings_mut()
        .retain(|finding| !finding.location.json_path.starts_with(at));
    note(read, at, &core_codes::INVALID_VALUE);
}

/// The spec the Molang of a leaf read by hand walks under: in a legacy walk the one that
/// replaces a failed expression with `default` (the one the field walks the leaf under, when
/// `None`), else every field's.
pub(crate) fn walk_spec(v: &MolangVisitor<'_>, default: Option<f32>) -> MolangFieldSpec {
    if LEGACY.holds(v.format()) {
        legacy_spec(default.unwrap_or_else(|| replaced(Some(*v.default_spec()))))
    } else {
        KEPT
    }
}

/// Hands `values` to the walk, each under its index, under `spec`.
pub(crate) fn walk_each(values: &mut [Molang], spec: &MolangFieldSpec, v: &mut MolangVisitor<'_>) {
    for (i, value) in values.iter_mut().enumerate() {
        v.enter(Step::Index(i));
        v.emit(spec, MolangFieldMut::Value(value));
        v.leave();
    }
}

/// The Molang walk of `value` as a document of the era holds it.
pub(crate) fn walk_in<T: MolangFields + ?Sized>(
    value: &mut T,
    legacy: bool,
    v: &mut MolangVisitor<'_>,
) {
    v.at_format(Some(format(legacy)), |v| value.visit_molang(v));
}
