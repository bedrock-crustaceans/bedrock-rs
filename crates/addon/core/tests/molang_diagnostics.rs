#![allow(clippy::pedantic, missing_docs)]
//! The diagnostic mapping of the compile pass: one input per
//! row of molangx's `LanguageMessage` table, `tests/fixtures/molang/language_messages.json`,
//! taken from molangx `tests/parse_vectors.json` where it has one. Each message becomes one
//! `InvalidMolang` with the compiler's diagnostic as its detail: the vanilla text, the `E..` id,
//! the severity (error; kept when the expression survives it) and vanilla's scope
//! chain `<member> | <expression> | <message>`.
#![cfg(feature = "molang")]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon_core::diagnostic::{
    DiagnosticCode, DiagnosticDetail, Diagnostics, JsonPath, Location, Severity,
};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::molang::{
    EmptyRule, LanguageMessage, MolangCheck, MolangContext, MolangFieldSpec, MolangOutcome,
    MolangSource, OpsRule, RawVersion, Side,
};

fn language_messages() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/molang/language_messages.json"
    );
    let text = std::fs::read_to_string(path).expect("the language message fixture");
    facet_json::from_str::<Value>(&text).expect("the fixture is JSON")
}

fn text(value: &Value) -> &str {
    value.as_string().expect("a string").as_str()
}

/// The fixture's molangx severity as the finding's severity and `kept`: a message the
/// expression survives (molangx's warning) is the game's level-3 line the parse goes past, a
/// kept error: the expression is still accepted.
fn severity(name: &str) -> (Severity, bool) {
    match name {
        "error" => (Severity::Error, false),
        "warning" => (Severity::Error, true),
        other => panic!("not a severity: {other}"),
    }
}

fn location() -> Location {
    let mut path = JsonPath::new();
    path.push("minecraft:toy");
    path.push("field");
    Location::new(None, None, path, None)
}

#[test]
fn every_language_message_maps_to_invalid_molang() {
    let fixture = language_messages();
    let rows = fixture.as_object().unwrap().get("rows").unwrap();
    let rows = rows.as_array().unwrap();
    let covered: Vec<u8> = rows
        .iter()
        .map(|row| {
            let row = row.as_object().unwrap().get("row").unwrap();
            u8::try_from(row.as_number().unwrap().to_u64().unwrap()).unwrap()
        })
        .collect();
    let all: Vec<u8> = LanguageMessage::ALL.iter().map(|m| m.row()).collect();
    let mut all_rows = all.clone();
    all_rows.dedup();
    assert_eq!(covered, all_rows, "one input per row");

    let check = MolangCheck::new();
    let mut failures = Vec::new();
    for row in rows {
        let row = row.as_object().unwrap();
        let number = row
            .get("row")
            .unwrap()
            .as_number()
            .unwrap()
            .to_u64()
            .unwrap();
        let expr = text(row.get("expr").unwrap());
        let version = row
            .get("version")
            .unwrap()
            .as_number()
            .unwrap()
            .to_i64()
            .unwrap();
        let version = RawVersion(i16::try_from(version).unwrap());
        let ops = match text(row.get("ops").unwrap()) {
            "all" => OpsRule::All,
            "no_assignments" => OpsRule::NoAssignments,
            other => panic!("not an ops rule: {other}"),
        };
        let rejected = row.get("rejected").unwrap().as_bool().unwrap();
        // Compiled as a string loader compiles it, so an empty text reaches the parser (#1).
        let spec = MolangFieldSpec {
            ops,
            empty: EmptyRule::Compile,
            ..MolangFieldSpec::DEFAULT
        };
        let cx = MolangContext {
            pack: version,
            document: version,
            ..MolangContext::detached()
        };
        let mut diagnostics = Diagnostics::default();
        let outcome = check.check_source(
            &MolangSource::string(expr, version),
            &spec,
            Side::Client,
            &cx,
            &location(),
            &mut diagnostics,
        );
        if matches!(outcome, MolangOutcome::Failed(_)) != rejected {
            failures.push(format!("#{number} {expr:?}: outcome {outcome:?}"));
        }
        let want: Vec<(String, String, (Severity, bool))> = row
            .get("messages")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|m| {
                let m = m.as_object().unwrap();
                (
                    text(m.get("id").unwrap()).to_owned(),
                    text(m.get("text").unwrap()).to_owned(),
                    severity(text(m.get("severity").unwrap())),
                )
            })
            .collect();
        let mut got = Vec::new();
        for finding in diagnostics.iter() {
            let Some(DiagnosticDetail::Molang(detail)) = finding.detail.as_deref() else {
                failures.push(format!("#{number} {expr:?}: a finding without detail"));
                continue;
            };
            let Some(message) = detail.language_message() else {
                // A lint: not a parser message of the table.
                continue;
            };
            if finding.code
                != (DiagnosticCode::InvalidMolang {
                    expression: expr.to_owned(),
                })
            {
                failures.push(format!("#{number} {expr:?}: code {:?}", finding.code));
            }
            let message_text = detail.to_string();
            let line = format!(
                "[{}] minecraft:toy/field | {expr} | {message_text}",
                finding.severity
            );
            if finding.to_string() != line {
                failures.push(format!("#{number} {expr:?}: line {finding}"));
            }
            got.push((
                message.id().to_owned(),
                message_text,
                (finding.severity, finding.kept),
            ));
        }
        if got != want {
            failures.push(format!(
                "#{number} {expr:?}:\n  got  {got:?}\n  want {want:?}"
            ));
        }
        let id = format!("E{number:02}");
        if !want.iter().any(|(w, _, _)| *w == id) {
            failures.push(format!(
                "#{number} {expr:?}: the row's message is not in the row"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The kept message (#6): the expression compiles, and the game's line is a kept error, an
/// error-level finding after which the game keeps the expression.
#[test]
fn a_kept_message_is_a_kept_error() {
    let check = MolangCheck::new();
    let cx = MolangContext::detached();
    let mut diagnostics = Diagnostics::default();
    let outcome = check.check_source(
        &MolangSource::string("1e", 13),
        &MolangFieldSpec::DEFAULT,
        Side::Client,
        &cx,
        &location(),
        &mut diagnostics,
    );
    assert_eq!(outcome, MolangOutcome::Ok);
    let found: Vec<(&DiagnosticCode, Severity, bool)> = diagnostics
        .iter()
        .map(|d| (&d.code, d.severity, d.kept))
        .collect();
    assert_eq!(
        found,
        [(
            &DiagnosticCode::InvalidMolang {
                expression: "1e".into()
            },
            Severity::Error,
            true
        )]
    );
    assert!(!diagnostics.has_errors());
}

/// What `text` (a version-13 string in a cereal field, client side) gives: each finding's code
/// name, severity and `kept`.
fn mapped(text: &str) -> Vec<(&'static str, Severity, bool)> {
    let check = MolangCheck::new();
    let cx = MolangContext {
        pack: RawVersion(13),
        document: RawVersion(13),
        ..MolangContext::detached()
    };
    let mut diagnostics = Diagnostics::default();
    check.check_source(
        &MolangSource::string(text, 13),
        &MolangFieldSpec::DEFAULT,
        Side::Client,
        &cx,
        &location(),
        &mut diagnostics,
    );
    diagnostics
        .iter()
        .map(|d| (d.code.name(), d.severity, d.kept))
        .collect()
}

/// The findings that do not reject the expression: a parser message the expression survives (#6, a
/// validator message below the root, #39) is the game's content-log error and the parse goes on, a
/// kept error; a lint of molangx's own (the argument count, "this crate's check") has no line of
/// the game's, so the crate reports it as an information.
#[test]
fn the_messages_that_keep_the_expression_and_molangx_s_own_lints_map_as_the_game_logs() {
    assert_eq!(mapped("1e"), [("InvalidMolang", Severity::Error, true)]);
    assert_eq!(
        mapped("t.x.y = 1;"),
        [("InvalidMolang", Severity::Error, true)]
    );
    assert_eq!(
        mapped("query.is_item_equipped('main_hand', 'x')"),
        [("MolangLint", Severity::Info, false)]
    );
}
