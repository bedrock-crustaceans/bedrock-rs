#![allow(clippy::pedantic, missing_docs)]
//! Pass 2, the load-time compile check: field
//! outcomes, allow-lists and operation sets, the cache, the empty-text rule, version sources that
//! act in the compile, and the findings' codes, severities and paths.
#![cfg(feature = "molang")]

use std::sync::Arc;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Severity,
};
use bedrock_addon_core::molang::{
    self, Admission, BLOCK_STATE_QUERIES, CompileCache, DiagCode, EmptyRule, ExperimentMask,
    LanguageMessage, Molang, MolangCheck, MolangContext, MolangFailure, MolangFieldSpec,
    MolangFields, MolangOutcome, MolangSource, MolangVersion, MolangVersionSource, MolangVisitor,
    OpsRule, PROPERTY_DEFAULT_QUERIES, RawVersion, ReaderKind, Side,
};
use bedrock_addon_core::version::{FormatVersionInput, MinEngineVersion};

/// A pack at `major.minor.patch`, its documents at 1.21.0.
fn pack(major: u16, minor: u16, patch: u16) -> MolangContext {
    MolangContext::new(
        Some(&MinEngineVersion::new(major, minor, patch)),
        ExperimentMask::empty(),
        &FormatVersionInput::parse("1.21.0").unwrap(),
        None,
    )
}

fn location(member: &str) -> Location {
    let mut path = JsonPath::new();
    path.push("minecraft:toy");
    path.push(member);
    Location::new(None, None, path, None)
}

/// A string source at the context's pack version, as pass 1 leaves it.
fn source(text: &str, cx: &MolangContext) -> MolangSource {
    MolangSource::string(text, cx.pack)
}

struct Checked {
    outcome: MolangOutcome,
    diagnostics: Diagnostics,
}

fn check_with(
    check: &MolangCheck,
    text: &str,
    spec: &MolangFieldSpec,
    side: Side,
    cx: &MolangContext,
) -> Checked {
    let mut diagnostics = Diagnostics::default();
    let outcome = check.check_source(
        &source(text, cx),
        spec,
        side,
        cx,
        &location("field"),
        &mut diagnostics,
    );
    Checked {
        outcome,
        diagnostics,
    }
}

fn check(text: &str, spec: &MolangFieldSpec, side: Side, cx: &MolangContext) -> Checked {
    check_with(&MolangCheck::new(), text, spec, side, cx)
}

/// The language messages of the findings, in order.
fn messages(diagnostics: &Diagnostics) -> Vec<LanguageMessage> {
    diagnostics
        .iter()
        .filter_map(|d| match d.detail.as_deref() {
            Some(DiagnosticDetail::Molang(m)) => m.language_message(),
            _ => None,
        })
        .collect()
}

fn invalid_molang(diagnostics: &Diagnostics) -> Vec<(&str, Severity)> {
    diagnostics
        .iter()
        .filter_map(|d| match &d.code {
            DiagnosticCode::InvalidMolang { expression } => Some((expression.as_str(), d.severity)),
            _ => None,
        })
        .collect()
}

const CEREAL: MolangFieldSpec = MolangFieldSpec::DEFAULT;

// ---- each failure rule, as the game's own unit tests exercise it ----

/// `StringExpressionNodeJsonWithInvalidMolang_ParseJson_HitContentError`: a cereal field with an
/// invalid expression loads and logs.
#[test]
fn a_cereal_field_with_invalid_molang_is_kept_and_logged() {
    let checked = check(
        "query.test_invalid_molang",
        &CEREAL,
        Side::Server,
        &pack(1, 21, 100),
    );
    assert_eq!(
        checked.outcome,
        MolangOutcome::Failed(MolangFailure::KeepLogged)
    );
    // Vanilla's test asserts the log holds "unrecognized token"; the lexer logs the unresolved
    // query first.
    assert_eq!(
        messages(&checked.diagnostics),
        [
            LanguageMessage::QueryUnresolved,
            LanguageMessage::UnrecognizedToken
        ]
    );
    assert!(
        invalid_molang(&checked.diagnostics)
            .iter()
            .all(|&finding| finding == ("query.test_invalid_molang", Severity::Error))
    );
}

/// Legacy block permutation conditions: below 1.19.70 a condition the restricted options reject
/// is compiled again unrestricted (family code calls the check twice); from 1.19.70 the block is
/// not registered.
#[test]
fn a_legacy_block_condition_is_restricted_then_unrestricted_below_1_19_70() {
    let restricted = MolangFieldSpec {
        admission: Admission::Only(BLOCK_STATE_QUERIES),
        ops: OpsRule::NoAssignmentsOrRandom,
        on_failure: MolangFailure::RejectDocument,
        ..CEREAL
    };
    let unrestricted = MolangFieldSpec {
        on_failure: MolangFailure::RejectDocument,
        ..CEREAL
    };
    let old = pack(1, 19, 60);
    assert!(matches!(
        check("q.is_baby", &restricted, Side::Server, &old).outcome,
        MolangOutcome::Failed(_)
    ));
    assert_eq!(
        check("q.is_baby", &unrestricted, Side::Server, &old).outcome,
        MolangOutcome::Ok
    );
    assert_eq!(
        check("q.is_baby", &restricted, Side::Server, &pack(1, 19, 70)).outcome,
        MolangOutcome::Failed(MolangFailure::RejectDocument)
    );
}

/// A render controller whose expression fails is dropped; one that uses arrays is valid.
#[test]
fn a_render_controller_entry_is_dropped() {
    let spec = MolangFieldSpec {
        reader: ReaderKind::SchemaValidated,
        on_failure: MolangFailure::DropEntry,
        ..CEREAL
    };
    let cx = pack(1, 21, 100);
    assert_eq!(
        check("1 +", &spec, Side::Client, &cx).outcome,
        MolangOutcome::Failed(MolangFailure::DropEntry)
    );
    let arrays = check("array.skins[q.variant]", &spec, Side::Client, &cx);
    assert_eq!(arrays.outcome, MolangOutcome::Ok);
    assert!(arrays.diagnostics.is_empty(), "{:?}", arrays.diagnostics);
}

/// A behaviour-pack expression molangx cannot evaluate parses, and gets one `MolangLint`
/// information (the game logs nothing for it).
#[test]
fn an_unevaluable_expression_in_a_behaviour_pack_is_linted() {
    let checked = check("geometry.default", &CEREAL, Side::Server, &pack(1, 21, 100));
    assert_eq!(checked.outcome, MolangOutcome::Ok);
    let found: Vec<(&DiagnosticCode, Severity)> = checked
        .diagnostics
        .iter()
        .map(|d| (&d.code, d.severity))
        .collect();
    assert_eq!(
        found,
        [(
            &DiagnosticCode::MolangLint {
                expression: "geometry.default".into()
            },
            Severity::Info
        )]
    );
}

/// An animation controller transition that fails is replaced by 0.
#[test]
fn a_controller_transition_is_replaced_by_zero() {
    let spec = MolangFieldSpec {
        reader: ReaderKind::SchemaValidated,
        version: MolangVersionSource::ControllerBackCompat,
        on_failure: MolangFailure::ReplaceWith(0.0),
        ..CEREAL
    };
    assert_eq!(
        check("v.x +", &spec, Side::Client, &pack(1, 21, 100)).outcome,
        MolangOutcome::Failed(MolangFailure::ReplaceWith(0.0))
    );
}

/// Controller `on_entry` / `on_exit` report a rejection as a warning.
#[test]
fn a_field_can_report_rejections_as_warnings() {
    let spec = MolangFieldSpec {
        error_severity: Severity::Warning,
        ..CEREAL
    };
    let checked = check("1 +", &spec, Side::Client, &pack(1, 21, 100));
    assert_eq!(
        invalid_molang(&checked.diagnostics),
        [("1 +", Severity::Warning)]
    );
}

/// A field whose parser lines the game logs at content-log error level and goes on past
/// (the content log at level 3, error class) reports its rejection as a kept error: at `Error`,
/// `kept`, rejecting nothing; the same field without the flag rejects.
#[test]
fn a_kept_field_reports_its_rejection_as_a_kept_error() {
    let kept = MolangFieldSpec {
        on_failure: MolangFailure::ReplaceWith(0.0),
        error_kept: true,
        ..CEREAL
    };
    let checked = check("1 +", &kept, Side::Client, &pack(1, 21, 100));
    assert_eq!(
        checked.outcome,
        MolangOutcome::Failed(MolangFailure::ReplaceWith(0.0))
    );
    let errors: Vec<(Severity, bool)> = checked
        .diagnostics
        .iter()
        .filter(|d| matches!(d.code, DiagnosticCode::InvalidMolang { .. }))
        .map(|d| (d.severity, d.kept))
        .collect();
    assert!(!errors.is_empty());
    assert!(
        errors.iter().all(|&e| e == (Severity::Error, true)),
        "the parser's lines are not kept errors: {errors:?}"
    );
    assert!(!checked.diagnostics.has_errors());
    let rejecting = check("1 +", &CEREAL, Side::Client, &pack(1, 21, 100));
    assert!(rejecting.diagnostics.has_errors());
}

// ---- allow-lists and operation sets ----

const PROPERTY_DEFAULT: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::SchemaValidated,
    admission: Admission::Only(PROPERTY_DEFAULT_QUERIES),
    ops: OpsRule::NoAssignments,
    ..MolangFieldSpec::DEFAULT
};

const BLOCK_CONDITION: MolangFieldSpec = MolangFieldSpec {
    admission: Admission::Only(BLOCK_STATE_QUERIES),
    ops: OpsRule::NoAssignmentsOrRandom,
    ..MolangFieldSpec::DEFAULT
};

#[test]
fn a_query_outside_the_allow_list_does_not_resolve() {
    let cx = pack(1, 21, 100);
    for spec in [PROPERTY_DEFAULT, BLOCK_CONDITION] {
        let checked = check("q.is_baby", &spec, Side::Server, &cx);
        assert!(matches!(checked.outcome, MolangOutcome::Failed(_)));
        assert_eq!(
            messages(&checked.diagnostics)[0],
            LanguageMessage::QueryUnresolved
        );
        assert_eq!(LanguageMessage::QueryUnresolved.id(), "E05");
    }
    assert_eq!(
        check(
            "q.had_component_group('baby')",
            &PROPERTY_DEFAULT,
            Side::Server,
            &cx
        )
        .outcome,
        MolangOutcome::Ok
    );
    assert_eq!(
        check(
            "q.block_state('demo:lit') == 1",
            &BLOCK_CONDITION,
            Side::Server,
            &cx
        )
        .outcome,
        MolangOutcome::Ok
    );
}

#[test]
fn random_is_allowed_in_a_property_default_but_not_in_a_block_condition() {
    let cx = pack(1, 21, 100);
    assert_eq!(
        check("math.random(0, 1)", &PROPERTY_DEFAULT, Side::Server, &cx).outcome,
        MolangOutcome::Ok
    );
    let checked = check("math.random(0, 1)", &BLOCK_CONDITION, Side::Server, &cx);
    assert!(matches!(checked.outcome, MolangOutcome::Failed(_)));
    assert_eq!(
        messages(&checked.diagnostics),
        [LanguageMessage::OperationNotAllowed]
    );
    assert_eq!(LanguageMessage::OperationNotAllowed.id(), "E21");
}

#[test]
fn an_assignment_is_allowed_in_neither() {
    let cx = pack(1, 21, 100);
    for spec in [PROPERTY_DEFAULT, BLOCK_CONDITION] {
        let checked = check("v.x = 1;", &spec, Side::Server, &cx);
        assert!(matches!(checked.outcome, MolangOutcome::Failed(_)));
        assert_eq!(
            messages(&checked.diagnostics),
            [LanguageMessage::OperationNotAllowed]
        );
    }
}

/// The two allow-lists name queries the standard catalogues declare.
#[test]
fn the_allow_lists_are_declared() {
    let check = MolangCheck::new();
    for side in [Side::Server, Side::Client] {
        for names in [BLOCK_STATE_QUERIES, PROPERTY_DEFAULT_QUERIES] {
            molang::QueryAllowList::new(check.catalog(side), names).expect("declared");
        }
    }
    let empty = molang::QueryCatalog::new(Side::Server, []).unwrap();
    assert!(
        MolangCheck::with_catalogs(empty.clone(), empty, Arc::new(CompileCache::new())).is_err()
    );
}

/// A client-only query compiled for the server resolves, with an info lint.
#[test]
fn a_client_only_query_on_the_server_is_an_info_lint() {
    let checked = check("q.anim_time", &CEREAL, Side::Server, &pack(1, 21, 100));
    assert_eq!(checked.outcome, MolangOutcome::Ok);
    let lints: Vec<(DiagCode, Severity)> = checked
        .diagnostics
        .iter()
        .filter(|d| matches!(d.code, DiagnosticCode::MolangLint { .. }))
        .filter_map(|d| match d.detail.as_deref() {
            Some(DiagnosticDetail::Molang(m)) => Some((m.code(), d.severity)),
            _ => None,
        })
        .collect();
    assert_eq!(lints, [(DiagCode::QueryClientOnly, Severity::Info)]);
}

/// The lints of `text` compiled for `side` in a 1.21.130 pack: molangx's code and the crate's
/// severity.
fn lints_of(text: &str, side: Side) -> Vec<(DiagCode, Severity)> {
    let checked = check(text, &CEREAL, side, &pack(1, 21, 130));
    assert_eq!(checked.outcome, MolangOutcome::Ok, "{text}");
    checked
        .diagnostics
        .iter()
        .filter(|d| matches!(d.code, DiagnosticCode::MolangLint { .. }))
        .filter_map(|d| match d.detail.as_deref() {
            Some(DiagnosticDetail::Molang(m)) => Some((m.code(), d.severity)),
            _ => None,
        })
        .collect()
}

// The three pack-setting queries do not work in a
// behaviour pack at all, so on the server each expression that uses one gets one warning.
#[test]
fn a_pack_setting_query_on_the_server_is_one_information_per_expression() {
    // The crate reports an information: the game's line comes from the query itself, at run time.
    let warning = [(DiagCode::QueryClientOnly, Severity::Info)];
    for text in [
        "query.get_pack_setting('ns:slider')",
        "q.is_pack_setting_enabled('ns:toggle')",
        "query.is_pack_setting_selected('ns:dropdown', 'banana')",
        "query.get_pack_setting('ns:a') + query.get_pack_setting('ns:b')",
        "q.is_pack_setting_enabled('ns:a') ? query.get_pack_setting('ns:b') : 0",
    ] {
        assert_eq!(lints_of(text, Side::Server), warning, "{text}");
        assert_eq!(lints_of(text, Side::Client), [], "{text}");
    }
    // By name: a query that merely starts like one, and every other client-only query, keeps
    // molangx's info, one per use.
    assert_eq!(
        lints_of(
            "q.anim_time + q.get_pack_setting('ns:a') + q.anim_time",
            Side::Server
        ),
        [
            (DiagCode::QueryClientOnly, Severity::Info),
            (DiagCode::QueryClientOnly, Severity::Info),
            (DiagCode::QueryClientOnly, Severity::Info)
        ]
    );
}

#[test]
fn the_pack_setting_information_is_reported_at_every_field_that_uses_the_text() {
    // The cache holds the compiler's diagnostics; the override is applied per report.
    let check = MolangCheck::new();
    let cx = pack(1, 21, 130);
    for _ in 0..2 {
        let checked = check_with(
            &check,
            "query.get_pack_setting('ns:a') + query.get_pack_setting('ns:b')",
            &CEREAL,
            Side::Server,
            &cx,
        );
        let severities: Vec<Severity> = checked.diagnostics.iter().map(|d| d.severity).collect();
        assert_eq!(severities, [Severity::Info]);
    }
}

// ---- the cache ----

#[test]
fn a_text_used_by_two_fields_is_reported_at_both() {
    let check = MolangCheck::new();
    let cx = pack(1, 21, 100);
    let mut diagnostics = Diagnostics::default();
    for member in ["first", "second"] {
        check.check_source(
            &source("1 +", &cx),
            &CEREAL,
            Side::Server,
            &cx,
            &location(member),
            &mut diagnostics,
        );
    }
    let paths: Vec<String> = diagnostics
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(paths, ["minecraft:toy/first", "minecraft:toy/second"]);
    assert_eq!(
        diagnostics.iter().next().unwrap().code,
        diagnostics.iter().nth(1).unwrap().code
    );
    assert_eq!((check.cache().misses(), check.cache().hits()), (1, 1));
}

#[test]
fn two_packs_at_different_versions_share_no_entry() {
    let check = MolangCheck::new();
    let mut diagnostics = Diagnostics::default();
    for cx in [pack(1, 18, 20), pack(1, 21, 100)] {
        check.check_source(
            &source("v.a || v.b && v.c", &cx),
            &CEREAL,
            Side::Server,
            &cx,
            &location("field"),
            &mut diagnostics,
        );
    }
    assert_eq!((check.cache().len(), check.cache().hits()), (2, 0));
}

// ---- the empty-text rule ----

/// Every cereal field (the default spec): an empty text is cleared without a compile or a log;
/// an object form at version −1 still compiles, at −1 (the game's Molang loading rules).
#[test]
fn a_cereal_field_clears_an_empty_text_and_compiles_minus_one() {
    assert_eq!(MolangFieldSpec::DEFAULT.empty, EmptyRule::NoExpression);
    let check = MolangCheck::new();
    let cx = pack(1, 21, 100);
    let mut diagnostics = Diagnostics::default();
    let outcome = check.check_source(
        &MolangSource::string("", 13),
        &CEREAL,
        Side::Server,
        &cx,
        &location("on_death"),
        &mut diagnostics,
    );
    assert_eq!(outcome, MolangOutcome::NoExpression);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(check.cache().is_empty());
    // At −1 no query resolves: the expression compiles and is rejected.
    let outcome = check.check_source(
        &MolangSource::object("q.is_baby", -1),
        &CEREAL,
        Side::Server,
        &cx,
        &location("on_death"),
        &mut diagnostics,
    );
    assert!(matches!(outcome, MolangOutcome::Failed(_)));
    assert_eq!(messages(&diagnostics)[0], LanguageMessage::QueryUnresolved);
    assert_eq!(check.cache().len(), 1);
}

/// Block files: an empty text and an object form at version −1 are both no expression.
#[test]
fn a_block_field_clears_an_empty_text_and_minus_one() {
    let check = MolangCheck::new();
    let spec = MolangFieldSpec {
        version: MolangVersionSource::Constant(MolangVersion::V13),
        empty: EmptyRule::BlockNoExpression,
        ..CEREAL
    };
    let cx = pack(1, 21, 100);
    for source in [
        MolangSource::string("", 13),
        MolangSource::object("q.x", -1),
    ] {
        let mut diagnostics = Diagnostics::default();
        let outcome = check.check_source(
            &source,
            &spec,
            Side::Server,
            &cx,
            &location("condition"),
            &mut diagnostics,
        );
        assert_eq!(outcome, MolangOutcome::NoExpression, "{source:?}");
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }
    assert!(check.cache().is_empty());
}

/// A loader that parses the string itself compiles an empty text: rejected with #1 from version 4.
#[test]
fn an_empty_text_is_rejected_when_compiled() {
    let compiled = MolangFieldSpec {
        empty: EmptyRule::Compile,
        ..CEREAL
    };
    let cx = pack(1, 21, 100);
    let checked = check("", &compiled, Side::Server, &cx);
    assert!(matches!(checked.outcome, MolangOutcome::Failed(_)));
    assert_eq!(messages(&checked.diagnostics), [LanguageMessage::NoTokens]);
    // Below version 4 the rejection carries only an info lint (molangx's own).
    let old = pack(1, 16, 0);
    let checked = check("", &compiled, Side::Server, &old);
    assert!(matches!(checked.outcome, MolangOutcome::Failed(_)));
    assert!(invalid_molang(&checked.diagnostics).is_empty());
    assert!(
        checked
            .diagnostics
            .iter()
            .all(|d| matches!(d.code, DiagnosticCode::MolangLint { .. })
                && d.severity == Severity::Info)
    );
}

// ---- version sources that act in the compile ----

/// Scatter parameters compile at the pack's version even in the object form; other fields
/// compile an object form at its own.
#[test]
fn scatter_parameters_ignore_the_object_version() {
    let object = MolangSource::object("q.block_property('a')", 3);
    let cx = pack(1, 21, 100);
    let check = MolangCheck::new();
    let mut diagnostics = Diagnostics::default();
    let own = check.check_source(
        &object,
        &CEREAL,
        Side::Server,
        &cx,
        &location("x"),
        &mut diagnostics,
    );
    assert_eq!(own, MolangOutcome::Ok, "{diagnostics:?}");
    let scatter = MolangFieldSpec {
        version: MolangVersionSource::PackIgnoringObject,
        ..CEREAL
    };
    let ignoring = check.check_source(
        &object,
        &scatter,
        Side::Server,
        &cx,
        &location("x"),
        &mut diagnostics,
    );
    assert!(matches!(ignoring, MolangOutcome::Failed(_)));
    assert_eq!(messages(&diagnostics)[0], LanguageMessage::QueryUnresolved);
}

/// Block files: the constant 13 for strings, and an object form keeps its own version.
#[test]
fn a_block_object_form_keeps_its_own_version() {
    let spec = MolangFieldSpec {
        version: MolangVersionSource::Constant(MolangVersion::V13),
        ..CEREAL
    };
    let cx = pack(1, 16, 0);
    let mut string = Molang::Expr(MolangSource::string_without_context(
        "q.block_property('a')",
    ));
    molang::settle(&mut string, spec.reader, spec.version.resolve(&cx)).unwrap();
    let object = Molang::Expr(MolangSource::object("q.block_property('a')", 3));
    let check = MolangCheck::new();
    let mut diagnostics = Diagnostics::default();
    assert!(matches!(
        check.check(
            &string,
            &spec,
            Side::Server,
            &cx,
            &location("x"),
            &mut diagnostics
        ),
        MolangOutcome::Failed(_)
    ));
    let mut diagnostics = Diagnostics::default();
    assert_eq!(
        check.check(
            &object,
            &spec,
            Side::Server,
            &cx,
            &location("x"),
            &mut diagnostics
        ),
        MolangOutcome::Ok
    );
}

/// Biome `height_range` compiles at version 1: `&&` / `||` with the precedence of 0.
#[test]
fn a_constant_version_reaches_the_compile() {
    let spec = MolangFieldSpec {
        reader: ReaderKind::BiomeHeightRange,
        version: MolangVersionSource::Constant(MolangVersion::V1),
        admission: Admission::WorldGen,
        ..CEREAL
    };
    let cx = pack(1, 21, 100);
    let mut value = Molang::Expr(MolangSource::string_without_context("1"));
    molang::settle(&mut value, spec.reader, spec.version.resolve(&cx)).unwrap();
    assert_eq!(
        value.source().unwrap().raw_version(),
        Some(RawVersion::from(MolangVersion::V1))
    );
    let check = MolangCheck::new();
    let mut diagnostics = Diagnostics::default();
    assert_eq!(
        check.check(
            &value,
            &spec,
            Side::Server,
            &cx,
            &location("height_range"),
            &mut diagnostics
        ),
        MolangOutcome::Ok
    );
    // No "made without its load context" lint: every source carries its version.
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

// ---- the findings ----

/// The content-log line is vanilla's scope chain `<member> | <expression> | <message>`, and
/// the detail carries the compiler's diagnostic.
#[test]
fn a_finding_is_at_the_fields_path_with_the_compilers_detail() {
    let checked = check("1 +", &CEREAL, Side::Server, &pack(1, 21, 100));
    let [finding] = checked.diagnostics.iter().collect::<Vec<&Diagnostic>>()[..] else {
        panic!("{:?}", checked.diagnostics);
    };
    let Some(DiagnosticDetail::Molang(detail)) = finding.detail.as_deref() else {
        panic!("a Molang detail");
    };
    assert_eq!(
        detail.language_message(),
        Some(LanguageMessage::BinaryAtEnd)
    );
    assert_eq!(
        finding.to_string(),
        format!("[error] minecraft:toy/field | 1 + | {detail}")
    );
}

/// A constant or a bool is not compiled.
#[test]
fn constants_need_no_compile() {
    let check = MolangCheck::new();
    let cx = pack(1, 21, 100);
    let mut diagnostics = Diagnostics::default();
    for value in [Molang::Const(1.0), Molang::Bool(true)] {
        assert_eq!(
            check.check(
                &value,
                &CEREAL,
                Side::Server,
                &cx,
                &location("x"),
                &mut diagnostics
            ),
            MolangOutcome::Ok
        );
    }
    assert!(check.cache().is_empty());
}

// ---- the document walk ----

#[derive(Debug, facet::Facet)]
struct ToyDocument {
    condition: Molang,
    scripts: Vec<MolangSource>,
    #[facet(default)]
    data: Option<molang::MolangList>,
}

const SCRIPT: MolangFieldSpec = MolangFieldSpec {
    on_failure: MolangFailure::DropEntry,
    ..CEREAL
};

const DATA: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::ScalarOrArray,
    ..CEREAL
};

impl MolangFields for ToyDocument {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("condition", &CEREAL, &mut self.condition);
        visitor.field("scripts", &SCRIPT, &mut self.scripts);
        visitor.field("data", &DATA, &mut self.data);
    }
}

#[test]
fn check_fields_returns_the_fields_that_did_not_pass() {
    let mut document: ToyDocument = facet_json::from_str(
        r#"{"condition": "q.is_baby", "scripts": ["v.x = 1;", "v.y = ;"], "data": [1, "1 +"]}"#,
    )
    .unwrap();
    let cx = pack(1, 21, 100);
    let base = location("root");
    let mut diagnostics = Diagnostics::default();
    let pass = molang::pass_one(&mut document, &cx, &base, &mut diagnostics);
    assert_eq!(pass.failed, 0);
    let outcomes = molang::check_fields(
        &mut document,
        &MolangCheck::new(),
        Side::Server,
        &cx,
        &base,
        &mut diagnostics,
    );
    let found: Vec<(String, MolangOutcome)> = outcomes
        .into_iter()
        .map(|o| (o.path.to_string(), o.outcome))
        .collect();
    assert_eq!(
        found,
        [
            (
                "minecraft:toy/root/scripts/1".to_owned(),
                MolangOutcome::Failed(MolangFailure::DropEntry)
            ),
            (
                "minecraft:toy/root/data".to_owned(),
                MolangOutcome::Failed(MolangFailure::KeepLogged)
            ),
        ]
    );
    // A list element's findings are at its index (vanilla logs a failing element by itself).
    let mut paths: Vec<String> = diagnostics
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    paths.dedup();
    assert_eq!(
        paths,
        ["minecraft:toy/root/scripts/1", "minecraft:toy/root/data/1"]
    );
}

/// Nested list elements are reported at their full index path.
#[test]
fn a_nested_list_element_is_reported_at_its_indices() {
    let cx = pack(1, 21, 100);
    let list = molang::MolangList::Many(vec![
        molang::MolangList::One(Molang::Const(1.0)),
        molang::MolangList::Many(vec![molang::MolangList::One(Molang::Expr(source(
            "1 +", &cx,
        )))]),
    ]);
    let mut diagnostics = Diagnostics::default();
    let outcome = MolangCheck::new().check_list(
        &list,
        &DATA,
        Side::Server,
        &cx,
        &location("data"),
        &mut diagnostics,
    );
    assert_eq!(outcome, MolangOutcome::Failed(MolangFailure::KeepLogged));
    let paths: Vec<String> = diagnostics
        .iter()
        .map(|d| d.location.json_path.to_string())
        .collect();
    assert_eq!(paths, ["minecraft:toy/data/1/0"]);
}

// Vanilla's own files call the two equipped-item tag queries
// with a slot and a tag; molangx's table registers them with at most one argument.
#[test]
fn the_equipped_item_tag_queries_take_a_slot_and_a_tag() {
    for side in [Side::Server, Side::Client] {
        for text in [
            "query.equipped_item_any_tag('slot.weapon.mainhand', 'minecraft:is_tool')",
            "q.equipped_item_all_tags('slot.weapon.offhand', 'a')",
            "query.equipped_item_any_tag('slot.weapon.mainhand')",
            "query.equipped_item_any_tag('slot.weapon.mainhand', 'a') && \
             query.equipped_item_all_tags('slot.weapon.mainhand', 'b')",
        ] {
            let arity: Vec<_> = lints_of(text, side)
                .into_iter()
                .filter(|(code, _)| *code == DiagCode::QueryArity)
                .collect();
            assert!(arity.is_empty(), "{text}: {arity:?}");
        }
        // The count molangx's table has stays its lint for every other query, and for these
        // two beyond a slot and a tag.
        for text in [
            "query.is_item_equipped('main_hand', 'x')",
            "query.equipped_item_any_tag('slot.weapon.mainhand', 'a', 'b')",
            "query.equipped_item_any_tag('slot.weapon.mainhand', 'a') && \
             query.is_item_equipped('main_hand', 'x')",
        ] {
            let arity = lints_of(text, side)
                .into_iter()
                .filter(|(code, _)| *code == DiagCode::QueryArity)
                .count();
            assert_eq!(arity, 1, "{text}");
        }
    }
}

// `query.anger_level` is registered with no argument, and the game's implementation never reads the
// argument list, so vanilla's warden (`query.anger_level(this) >= 5`) runs with no log. The
// one-argument call raises no lint; every other count stays molangx's lint.
#[test]
fn query_anger_level_takes_the_warden_s_one_argument() {
    for side in [Side::Server, Side::Client] {
        for text in ["query.anger_level(this) >= 5", "q.anger_level(this) >= 2"] {
            let arity: Vec<_> = lints_of(text, side)
                .into_iter()
                .filter(|(code, _)| *code == DiagCode::QueryArity)
                .collect();
            assert!(arity.is_empty(), "{text}: {arity:?}");
        }
        let arity = lints_of("query.anger_level(this, 1)", side)
            .into_iter()
            .filter(|(code, _)| *code == DiagCode::QueryArity)
            .count();
        assert_eq!(arity, 1);
    }
}
