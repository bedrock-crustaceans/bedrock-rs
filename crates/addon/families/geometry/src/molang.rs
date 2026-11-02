//! The one Molang member of a geometry file, bone `binding` (from 1.16.0): pass 1 (the pack's
//! version) and pass 2 (the compile check on the client catalogue). Vanilla's schema node
//! compiles the string at load and a rejected expression fails the whole file.
//!
//! The game compiles a binding a second time, lazily, when a model is set up; that compile is
//! part of rendering, not of loading, and is not reproduced here.

use super::{Geometry, GeometryFile};
use bedrock_addon_core::diagnostic::{Diagnostics, Location, Severity};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::molang::{
    Admission, EmptyRule, MolangContext, MolangFailure, MolangFieldSpec, MolangFields,
    MolangVersionSource, MolangVisitor, OpsRule, ReaderKind, Step, not_checked, pass_one,
};
use bedrock_addon_core::version::FormatVersionInput;

/// Bone `binding`: a number or bool is kept uncompiled, a string is compiled at the pack's
/// version with the default query set; a rejection drops the file.
pub(crate) const BINDING: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::SchemaValidated,
    version: MolangVersionSource::Pack,
    admission: Admission::Default,
    ops: OpsRule::All,
    empty: EmptyRule::Compile,
    on_failure: MolangFailure::RejectDocument,
    error_severity: Severity::Error,
    error_kept: false,
};

/// Visits `minecraft:geometry[*].bones[*].binding`.
pub(crate) fn visit(geometries: &mut [Geometry], visitor: &mut MolangVisitor<'_>) {
    visitor.enter(Step::Key("minecraft:geometry"));
    for (index, geometry) in geometries.iter_mut().enumerate() {
        let Some(bones) = &mut geometry.bones else {
            continue;
        };
        visitor.enter(Step::Index(index));
        visitor.enter(Step::Key("bones"));
        for (at, bone) in bones.iter_mut().enumerate() {
            visitor.enter(Step::Index(at));
            visitor.field("binding", &BINDING, &mut bone.binding);
            visitor.leave();
        }
        visitor.leave();
        visitor.leave();
    }
    visitor.leave();
}

impl MolangFields for GeometryFile {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visit(&mut self.geometries, visitor);
    }
}

/// Pass 1 and pass 2 over a file; `false` when a `binding` was rejected, which fails the file.
pub(crate) fn run(
    file: &mut GeometryFile,
    written: FormatVersionInput,
    load: &LoadContext,
    diagnostics: &mut Diagnostics,
) -> bool {
    let cx = MolangContext::new(
        load.min_engine_version.as_ref(),
        load.molang_experiments,
        &written,
        Some(load.molang_version),
    );
    let base = Location::default();
    // A reader failure of a `MolangFailure::RejectDocument` member fails the file. This cannot
    // happen for a file the walk read, which used the same reader, but a caller-built value
    // can reach it.
    if pass_one(file, &cx, &base, diagnostics).failed > 0 {
        return false;
    }
    #[cfg(feature = "molang")]
    if let Some(check) = &load.molang {
        use bedrock_addon_core::molang::{MolangOutcome, Side, check_fields};
        // The geometry loader is client code: the client catalogue whatever the pack.
        let outcomes = check_fields(file, check, Side::Client, &cx, &base, diagnostics);
        return !outcomes
            .iter()
            .any(|field| matches!(field.outcome, MolangOutcome::Failed(_)));
    }
    not_checked(file, &base, diagnostics);
    true
}
