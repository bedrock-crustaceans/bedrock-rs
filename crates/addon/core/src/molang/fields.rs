//! The walk over a document's Molang fields (trait `MolangFields`) and pass 1 over it:
//! each field's reader check and context version, right after facet produced the version
//! struct.

use indexmap::IndexMap;

use super::read::settle_source;
use super::{
    Molang, MolangContext, MolangFieldSpec, MolangList, MolangSource, ReprError, settle,
    settle_list,
};
use crate::diagnostic::{
    Code, Diagnostic, Diagnostics, JsonPath, Location, Phase, Severity, codes,
};

/// A version struct (or part of one) that holds Molang fields. Implemented next to each struct,
/// generated where the struct is macro-built: `visit_molang` hands every field to the visitor
/// with its JSON key and spec, in document order.
///
/// ```ignore
/// impl MolangFields for Permutation {
///     fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
///         visitor.field("condition", &CONDITION, &mut self.condition);
///         visitor.nested("components", &mut self.components);
///     }
/// }
/// ```
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
pub trait MolangFields {
    /// Hands every Molang field to `visitor`.
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>);
}

/// One step into a JSON document: a member key or an array index.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step<'s> {
    /// A member.
    Key(&'s str),
    /// An array element.
    Index(usize),
}

impl<'s> From<&'s str> for Step<'s> {
    fn from(key: &'s str) -> Self {
        Self::Key(key)
    }
}

impl From<usize> for Step<'_> {
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

/// A Molang field's value, as the visitor hands it out.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
#[derive(Debug)]
pub enum MolangFieldMut<'a> {
    /// A number, bool or expression field.
    Value(&'a mut Molang),
    /// A string-only field.
    Source(&'a mut MolangSource),
    /// A value-or-array field.
    List(&'a mut MolangList),
}

/// One Molang field: where it is, its spec and its value.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
#[derive(Debug)]
pub struct MolangField<'a> {
    /// The field's JSON path, from the document root (or the caller's base path).
    pub path: &'a JsonPath,
    /// The field's spec.
    pub spec: &'a MolangFieldSpec,
    /// The value.
    pub value: MolangFieldMut<'a>,
}

/// What [`MolangFields::visit_molang`] reports to: keeps the JSON path while the walk descends.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
pub struct MolangVisitor<'v> {
    path: JsonPath,
    default_spec: &'v MolangFieldSpec,
    visit: &'v mut dyn FnMut(MolangField<'_>),
}

impl std::fmt::Debug for MolangVisitor<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MolangVisitor")
            .field("path", &self.path)
            .field("default_spec", self.default_spec)
            .finish_non_exhaustive()
    }
}

impl MolangVisitor<'_> {
    /// The field `step` with spec `spec`; `slot` is a Molang value, a source, a list, or an
    /// `Option`, `Vec`, `IndexMap` or `Box` of them.
    pub fn field<'s, S: MolangSlot + ?Sized>(
        &mut self,
        step: impl Into<Step<'s>>,
        spec: &MolangFieldSpec,
        slot: &mut S,
    ) {
        self.enter(step.into());
        slot.visit_slot(spec, self);
        self.path.pop();
    }

    /// The field `step` under the walk's default spec ([`visit_fields_under`]): a
    /// `#[derive(Schema)]` field with no `schema::molang` key.
    pub fn field_default<'s, S: MolangSlot + ?Sized>(
        &mut self,
        step: impl Into<Step<'s>>,
        slot: &mut S,
    ) {
        let spec = self.default_spec;
        self.field(step, spec, slot);
    }

    /// The spec a field without its own takes: the family's, or [`MolangFieldSpec::DEFAULT`].
    pub fn default_spec(&self) -> &MolangFieldSpec {
        self.default_spec
    }

    /// The member or element `step`, which holds Molang fields of its own.
    pub fn nested<'s, T: MolangFields + ?Sized>(
        &mut self,
        step: impl Into<Step<'s>>,
        inner: &mut T,
    ) {
        self.enter(step.into());
        inner.visit_molang(self);
        self.path.pop();
    }

    /// The member or element `step`, which holds Molang fields of its own, walked with `spec`
    /// the spec of each field below that has none of its own: a `#[derive(Schema)]` field
    /// whose `schema::molang` names a nested walk's default.
    pub fn nested_under<'s, T: MolangFields + ?Sized>(
        &mut self,
        step: impl Into<Step<'s>>,
        spec: &MolangFieldSpec,
        inner: &mut T,
    ) {
        self.enter(step.into());
        self.under(spec, |visitor| inner.visit_molang(visitor));
        self.path.pop();
    }

    /// Runs `walk` at the current path with `spec` the default spec ([`Self::nested_under`]
    /// without the step: a flattened field).
    pub fn under(&mut self, spec: &MolangFieldSpec, walk: impl FnOnce(&mut MolangVisitor<'_>)) {
        let mut under = MolangVisitor {
            path: std::mem::take(&mut self.path),
            default_spec: spec,
            visit: &mut *self.visit,
        };
        walk(&mut under);
        self.path = under.path;
    }

    /// Hands one value at the current path to the walk's callback; for [`MolangSlot`] impls.
    pub fn emit(&mut self, spec: &MolangFieldSpec, value: MolangFieldMut<'_>) {
        (self.visit)(MolangField {
            path: &self.path,
            spec,
            value,
        });
    }

    /// Steps into `step`; [`MolangVisitor::leave`] steps out. For [`MolangSlot`] impls.
    pub fn enter(&mut self, step: Step<'_>) {
        match step {
            Step::Key(key) => self.path.push(key),
            Step::Index(index) => self.path.push_index(index),
        }
    }

    /// Steps out of the last [`MolangVisitor::enter`].
    pub fn leave(&mut self) {
        self.path.pop();
    }
}

/// A field slot the visitor understands: a Molang value, source or list, or a container of them.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
pub trait MolangSlot {
    /// Emits every value in the slot with `spec`.
    fn visit_slot(&mut self, spec: &MolangFieldSpec, visitor: &mut MolangVisitor<'_>);
}

impl MolangSlot for Molang {
    fn visit_slot(&mut self, spec: &MolangFieldSpec, visitor: &mut MolangVisitor<'_>) {
        visitor.emit(spec, MolangFieldMut::Value(self));
    }
}

impl MolangSlot for MolangSource {
    fn visit_slot(&mut self, spec: &MolangFieldSpec, visitor: &mut MolangVisitor<'_>) {
        visitor.emit(spec, MolangFieldMut::Source(self));
    }
}

/// A value-or-array field is one field, its elements read together.
impl MolangSlot for MolangList {
    fn visit_slot(&mut self, spec: &MolangFieldSpec, visitor: &mut MolangVisitor<'_>) {
        visitor.emit(spec, MolangFieldMut::List(self));
    }
}

impl<T: MolangSlot + ?Sized> MolangSlot for Box<T> {
    fn visit_slot(&mut self, spec: &MolangFieldSpec, visitor: &mut MolangVisitor<'_>) {
        (**self).visit_slot(spec, visitor);
    }
}

impl<T: MolangSlot> MolangSlot for Option<T> {
    fn visit_slot(&mut self, spec: &MolangFieldSpec, visitor: &mut MolangVisitor<'_>) {
        if let Some(slot) = self {
            slot.visit_slot(spec, visitor);
        }
    }
}

/// Each element is a field of its own, at its index.
impl<T: MolangSlot> MolangSlot for Vec<T> {
    fn visit_slot(&mut self, spec: &MolangFieldSpec, visitor: &mut MolangVisitor<'_>) {
        for (index, slot) in self.iter_mut().enumerate() {
            visitor.enter(Step::Index(index));
            slot.visit_slot(spec, visitor);
            visitor.leave();
        }
    }
}

/// Each value is a field of its own, at its key.
impl<T: MolangSlot> MolangSlot for IndexMap<String, T> {
    fn visit_slot(&mut self, spec: &MolangFieldSpec, visitor: &mut MolangVisitor<'_>) {
        for (key, slot) in self.iter_mut() {
            visitor.enter(Step::Key(key));
            slot.visit_slot(spec, visitor);
            visitor.leave();
        }
    }
}

impl<T: MolangFields + ?Sized> MolangFields for Box<T> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        (**self).visit_molang(visitor);
    }
}

impl<T: MolangFields> MolangFields for Option<T> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        if let Some(inner) = self {
            inner.visit_molang(visitor);
        }
    }
}

impl<T: MolangFields> MolangFields for Vec<T> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        for (index, inner) in self.iter_mut().enumerate() {
            visitor.nested(index, inner);
        }
    }
}

impl<T: MolangFields, const N: usize> MolangFields for [T; N] {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        for (index, inner) in self.iter_mut().enumerate() {
            visitor.nested(index, inner);
        }
    }
}

/// An identifier holds no Molang.
impl<K: crate::id::Kind> MolangFields for crate::id::Id<K> {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

// A filter and a range hold no Molang a document's walk visits.
crate::impl_no_molang!(crate::shared::Filter, crate::shared::FilterSubject);

impl<T: crate::shared::RangeEnd> MolangFields for crate::shared::Range<T> {
    fn visit_molang(&mut self, _: &mut MolangVisitor<'_>) {}
}

impl<T: MolangFields> MolangFields for IndexMap<String, T> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        for (key, inner) in self.iter_mut() {
            visitor.nested(key.as_str(), inner);
        }
    }
}

impl<T: MolangFields, S: std::hash::BuildHasher> MolangFields
    for std::collections::HashMap<String, T, S>
{
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        for (key, inner) in self.iter_mut() {
            visitor.nested(key.as_str(), inner);
        }
    }
}

impl<T: MolangFields> MolangFields for std::collections::BTreeMap<String, T> {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        for (key, inner) in self.iter_mut() {
            visitor.nested(key.as_str(), inner);
        }
    }
}

/// Calls `visit` for every Molang field of `document`, with paths below `base`; a field with
/// no spec of its own takes [`MolangFieldSpec::DEFAULT`].
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
pub fn visit_fields<T: MolangFields + ?Sized>(
    document: &mut T,
    base: &JsonPath,
    visit: impl FnMut(MolangField<'_>),
) {
    visit_fields_under(document, base, &MolangFieldSpec::DEFAULT, visit);
}

/// [`visit_fields`] with `default_spec` the spec of every field that has none of its own
/// ([`MolangVisitor::field_default`]): the family's default.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
pub fn visit_fields_under<T: MolangFields + ?Sized>(
    document: &mut T,
    base: &JsonPath,
    default_spec: &MolangFieldSpec,
    mut visit: impl FnMut(MolangField<'_>),
) {
    let mut visitor = MolangVisitor {
        path: base.clone(),
        default_spec,
        visit: &mut visit,
    };
    document.visit_molang(&mut visitor);
}

/// What pass 1 found in a document.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct MolangPass {
    /// How many expressions the document holds (a list counts each element).
    pub sources: u32,
    /// How many fields failed their reader check.
    pub failed: u32,
}

/// Pass 1: each field's reader check (`InvalidType`, or `InvalidValue` for a number
/// out of range, at error level and at the field's path) and the version of its string forms
/// from the field's version source. Afterwards no expression of a field that passed is without
/// a version. Runs in both builds, so reader errors and versions are the same with and without
/// the `molang` feature.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
pub fn pass_one<T: MolangFields + ?Sized>(
    document: &mut T,
    cx: &MolangContext,
    base: &Location,
    diagnostics: &mut Diagnostics,
) -> MolangPass {
    let mut pass = MolangPass::default();
    visit_fields(document, &base.json_path, |field| {
        let version = field.spec.version.resolve(cx);
        let reader = field.spec.reader;
        let (result, sources) = match field.value {
            MolangFieldMut::Value(value) => {
                let result = settle(value, reader, version);
                (result, u32::from(value.source().is_some()))
            }
            MolangFieldMut::Source(source) => (settle_source(source, reader, version), 1),
            MolangFieldMut::List(list) => {
                let result = settle_list(list, version);
                (result, list_sources(list))
            }
        };
        pass.sources += sources;
        if let Err(error) = result {
            pass.failed += 1;
            diagnostics.push(Diagnostic::new(
                reader_error_code(&error),
                Severity::Error,
                Phase::Parse,
                at(base, field.path),
            ));
        }
    });
    pass
}

/// How many expressions `document` holds (a list counts each element).
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
pub fn count_sources<T: MolangFields + ?Sized>(document: &mut T) -> u32 {
    let mut sources = 0;
    visit_fields(document, &JsonPath::new(), |field| {
        sources += match field.value {
            MolangFieldMut::Value(value) => u32::from(value.source().is_some()),
            MolangFieldMut::Source(_) => 1,
            MolangFieldMut::List(list) => list_sources(list),
        };
    });
    sources
}

/// What stands in for the compile pass without the `molang` feature
/// (`MolangNotChecked`): one info at `base` naming how many expressions `document` holds, if
/// it holds any. No failure rule is applied. Returns the count.
// hidden: Molang pass machinery of the crate family, outside SemVer
#[doc(hidden)]
pub fn not_checked<T: MolangFields + ?Sized>(
    document: &mut T,
    base: &Location,
    diagnostics: &mut Diagnostics,
) -> u32 {
    let fields = count_sources(document);
    if fields > 0 {
        diagnostics.push(Diagnostic::new(
            codes::MOLANG_NOT_CHECKED.with([fields.to_string()]),
            Severity::Info,
            Phase::Validate,
            base.clone(),
        ));
    }
    fields
}

fn list_sources(list: &MolangList) -> u32 {
    match list {
        MolangList::One(value) => u32::from(value.source().is_some()),
        MolangList::Many(elements) => elements.iter().map(list_sources).sum(),
    }
}

/// `ReprError::UnsupportedType` (and anything a newer molang adds) is `InvalidType`,
/// `ReprError::NumberOutOfRange` is `InvalidValue`.
fn reader_error_code(error: &ReprError) -> Code {
    match error {
        ReprError::NumberOutOfRange { .. } => Code::from(&codes::INVALID_VALUE),
        _ => Code::from(&codes::INVALID_TYPE),
    }
}

/// `base` with the field's path.
pub(super) fn at(base: &Location, path: &JsonPath) -> Location {
    Location {
        json_path: path.clone(),
        ..base.clone()
    }
}

/// The empty [`MolangFields`] impl of each struct named: a document struct with no Molang
/// field, whose walk visits nothing.
///
/// A family's chain needs the impl for every struct in it, and `version_chain!` writes it for
/// a chain declared `molang = none;`; a one-version family on `versioning::Single` has no
/// macro to write it, so it names its struct here, beside the struct:
///
/// ```
/// pub struct VoxelShape {
///     pub identifier: String,
/// }
///
/// bedrock_addon_core::impl_no_molang!(VoxelShape);
/// ```
// hidden: macro support, named by the expansion of the exported macro
#[doc(hidden)]
#[macro_export]
macro_rules! impl_no_molang {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl $crate::molang::MolangFields for $ty {
                fn visit_molang(&mut self, _: &mut $crate::molang::MolangVisitor<'_>) {}
            }
        )+
    };
}
