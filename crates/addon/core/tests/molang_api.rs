//! The public surface of `molang`: the molangx re-exports at their one
//! path, the `Molang` alias, `MolangList`, the crate itself at `molang::molangx`, and, with the
//! `molang` feature, `MolangCheck` and the compile items.

use bedrock_addon_core::molang::{
    EngineVersion, Experiment, ExperimentMask, HashedStr, JsonScalar, JsonType, Molang, MolangList,
    MolangSource, MolangValueRepr, MolangVersion, OpSet, OtherJson, QueryAdmission, QueryAllowList,
    QueryCatalog, QuerySetMask, RawVersion, ReadField, ReaderKind, ReprError, Side, SourceForm,
    VERSION_THRESHOLDS,
};

/// `Molang` is a name for molangx's type, not a second type.
#[test]
fn molang_is_the_crates_value_repr() {
    let value: Molang = MolangValueRepr::Const(1.0);
    let same: molangx::json::MolangValueRepr = value;
    assert_eq!(same, Molang::Const(1.0));
    // The module re-exports the crate itself.
    let source: bedrock_addon_core::molang::molangx::json::MolangSource =
        MolangSource::string("q.x", 13);
    assert_eq!(
        source.form(),
        SourceForm::String {
            context_version: Some(RawVersion(13))
        }
    );
}

/// Every re-export names the molangx item.
#[test]
fn the_reexports_are_molangxs_items() {
    fn same<T>(_: T, _: T) {}
    same::<Option<Side>>(None, None::<molangx::catalog::Side>);
    same::<Option<QueryAdmission>>(None, None::<molangx::catalog::QueryAdmission>);
    same::<Option<QueryAllowList>>(None, None::<molangx::catalog::QueryAllowList>);
    same::<Option<QueryCatalog>>(None, None::<molangx::catalog::QueryCatalog>);
    same::<Option<QuerySetMask>>(None, None::<molangx::catalog::QuerySetMask>);
    same::<Option<HashedStr>>(None, None::<molangx::hash::HashedStr>);
    same::<Option<JsonScalar>>(None, None::<molangx::json::JsonScalar>);
    same::<Option<JsonType>>(None, None::<molangx::json::JsonType>);
    same::<Option<OtherJson>>(None, None::<molangx::json::OtherJson>);
    same::<Option<ReadField>>(None, None::<molangx::json::ReadField>);
    same::<Option<ReaderKind>>(None, None::<molangx::json::ReaderKind>);
    same::<Option<ReprError>>(None, None::<molangx::json::ReprError>);
    same::<Option<OpSet>>(None, None::<molangx::ops::OpSet>);
    same::<Option<EngineVersion>>(None, None::<molangx::version::EngineVersion>);
    same::<Option<Experiment>>(None, None::<molangx::version::Experiment>);
    same::<Option<ExperimentMask>>(None, None::<molangx::version::ExperimentMask>);
    same::<Option<MolangVersion>>(None, None::<molangx::version::MolangVersion>);
    same::<Option<RawVersion>>(None, None::<molangx::version::RawVersion>);
    assert_eq!(
        VERSION_THRESHOLDS.len(),
        molangx::version::VERSION_THRESHOLDS.len()
    );
    let _: Option<MolangList> = None;
}

#[cfg(feature = "molang")]
#[test]
fn the_compile_items_are_molangxs() {
    use bedrock_addon_core::molang::{
        CompileCache, CompileFailure, CompileOptions, CompileOutcome, Compiled, Deviations,
        DiagCode, LanguageMessage, MolangCheck, MolangDiagnostic, MolangSeverity, compile_source,
    };
    fn same<T>(_: T, _: T) {}
    same::<Option<CompileCache>>(None, None::<molangx::cache::CompileCache>);
    same::<Option<CompileFailure>>(None, None::<molangx::compile::CompileFailure>);
    same::<Option<CompileOptions>>(None, None::<molangx::compile::CompileOptions>);
    same::<Option<CompileOutcome>>(None, None::<molangx::compile::CompileOutcome>);
    same::<Option<Compiled>>(None, None::<molangx::compile::Compiled>);
    same::<Option<DiagCode>>(None, None::<molangx::diag::DiagCode>);
    same::<Option<Deviations>>(None, None::<molangx::compile::Deviations>);
    same::<Option<LanguageMessage>>(None, None::<molangx::diag::LanguageMessage>);
    same::<Option<MolangDiagnostic>>(None, None::<molangx::diag::Diagnostic>);
    same::<Option<MolangSeverity>>(None, None::<molangx::diag::Severity>);
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
