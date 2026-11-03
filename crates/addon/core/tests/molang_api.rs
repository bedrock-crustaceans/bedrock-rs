//! The public surface of `molang`: the molang re-exports at their one
//! path, the `Molang` alias, `MolangList`, the crate itself at `molang::molang`, and, with the
//! `molang` feature, `MolangCheck` and the compile items.

use bedrock_addon_core::molang::{
    EngineVersion, Experiment, ExperimentMask, HashedStr, JsonScalar, JsonType, Molang, MolangList,
    MolangSource, MolangValueRepr, MolangVersion, OpSet, OtherJson, QueryAdmission, QueryAllowList,
    QueryCatalog, QuerySetMask, RawVersion, ReadField, ReaderKind, ReprError, Side, SourceForm,
    VERSION_THRESHOLDS,
};

/// `Molang` is a name for molang's type, not a second type.
#[test]
fn molang_is_the_crates_value_repr() {
    let value: Molang = MolangValueRepr::Const(1.0);
    let same: molang::json::MolangValueRepr = value;
    assert_eq!(same, Molang::Const(1.0));
    // The module re-exports the crate itself.
    let source: bedrock_addon_core::molang::molang::json::MolangSource =
        MolangSource::string("q.x", 13);
    assert_eq!(
        source.form(),
        SourceForm::String {
            context_version: Some(RawVersion(13))
        }
    );
}

/// Every re-export names the molang item.
#[test]
fn the_reexports_are_molangs_items() {
    fn same<T>(_: T, _: T) {}
    same::<Option<Side>>(None, None::<molang::catalog::Side>);
    same::<Option<QueryAdmission>>(None, None::<molang::catalog::QueryAdmission>);
    same::<Option<QueryAllowList>>(None, None::<molang::catalog::QueryAllowList>);
    same::<Option<QueryCatalog>>(None, None::<molang::catalog::QueryCatalog>);
    same::<Option<QuerySetMask>>(None, None::<molang::catalog::QuerySetMask>);
    same::<Option<HashedStr>>(None, None::<molang::hash::HashedStr>);
    same::<Option<JsonScalar>>(None, None::<molang::json::JsonScalar>);
    same::<Option<JsonType>>(None, None::<molang::json::JsonType>);
    same::<Option<OtherJson>>(None, None::<molang::json::OtherJson>);
    same::<Option<ReadField>>(None, None::<molang::json::ReadField>);
    same::<Option<ReaderKind>>(None, None::<molang::json::ReaderKind>);
    same::<Option<ReprError>>(None, None::<molang::json::ReprError>);
    same::<Option<OpSet>>(None, None::<molang::ops::OpSet>);
    same::<Option<EngineVersion>>(None, None::<molang::version::EngineVersion>);
    same::<Option<Experiment>>(None, None::<molang::version::Experiment>);
    same::<Option<ExperimentMask>>(None, None::<molang::version::ExperimentMask>);
    same::<Option<MolangVersion>>(None, None::<molang::version::MolangVersion>);
    same::<Option<RawVersion>>(None, None::<molang::version::RawVersion>);
    assert_eq!(
        VERSION_THRESHOLDS.len(),
        molang::version::VERSION_THRESHOLDS.len()
    );
    let _: Option<MolangList> = None;
}

#[cfg(feature = "molang")]
#[test]
fn the_compile_items_are_molangs() {
    use bedrock_addon_core::molang::{
        CompileCache, CompileFailure, CompileOptions, CompileOutcome, Compiled, Deviations,
        DiagCode, LanguageMessage, MolangCheck, MolangDiagnostic, MolangSeverity, compile_source,
    };
    fn same<T>(_: T, _: T) {}
    same::<Option<CompileCache>>(None, None::<molang::cache::CompileCache>);
    same::<Option<CompileFailure>>(None, None::<molang::compile::CompileFailure>);
    same::<Option<CompileOptions>>(None, None::<molang::compile::CompileOptions>);
    same::<Option<CompileOutcome>>(None, None::<molang::compile::CompileOutcome>);
    same::<Option<Compiled>>(None, None::<molang::compile::Compiled>);
    same::<Option<DiagCode>>(None, None::<molang::diag::DiagCode>);
    same::<Option<Deviations>>(None, None::<molang::compile::Deviations>);
    same::<Option<LanguageMessage>>(None, None::<molang::diag::LanguageMessage>);
    same::<Option<MolangDiagnostic>>(None, None::<molang::diag::Diagnostic>);
    same::<Option<MolangSeverity>>(None, None::<molang::diag::Severity>);
    let compiled = compile_source(
        &MolangSource::string("1 + 1", 13),
        &CompileOptions::server(MolangVersion::LATEST),
    );
    assert!(compiled.is_success());
    // The standard catalogues and an empty cache.
    let check = MolangCheck::default();
    assert!(check.cache().is_empty());
    assert_eq!(check.catalog(Side::Client).side(), Side::Client);
}
