//! Pass 1 over a document's Molang fields (the first of the two Molang passes): a toy version
//! struct deserialised by facet from text, walked through `MolangFields`; each field's reader check
//! and version source. Without the `molang` feature, `not_checked` stands in for the compile pass
//! not compiling.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
use bedrock_addon_core::diagnostic::{
    DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::molang::{
    self, ExperimentMask, Molang, MolangContext, MolangFieldMut, MolangFieldSpec, MolangFields,
    MolangList, MolangSource, MolangVersion, MolangVersionSource, MolangVisitor, RawVersion,
    ReaderKind, SourceForm,
};
use bedrock_addon_core::version::{FormatVersionInput, MinEngineVersion, SemVersion};
use indexmap::IndexMap;

const CONDITION: MolangFieldSpec = MolangFieldSpec::DEFAULT;
const SCRIPT: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::SchemaValidated,
    ..MolangFieldSpec::DEFAULT
};
const WEIGHT: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::SchemaValidated,
    version: MolangVersionSource::FormatVersion,
    ..MolangFieldSpec::DEFAULT
};
const DATA: MolangFieldSpec = MolangFieldSpec {
    reader: ReaderKind::ScalarOrArray,
    ..MolangFieldSpec::DEFAULT
};
const TAGS: MolangFieldSpec = MolangFieldSpec {
    version: MolangVersionSource::Constant(MolangVersion::V13),
    ..MolangFieldSpec::DEFAULT
};

#[derive(Debug, facet::Facet)]
struct ToyEvent {
    condition: Molang,
}

impl MolangFields for ToyEvent {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("condition", &CONDITION, &mut self.condition);
    }
}

#[derive(Debug, facet::Facet)]
struct ToyDocument {
    condition: Molang,
    script: MolangSource,
    #[facet(default)]
    weights: Vec<Molang>,
    #[facet(default)]
    data: Option<MolangList>,
    #[facet(default)]
    events: IndexMap<String, ToyEvent>,
    #[facet(default)]
    tags: Option<MolangSource>,
}

impl MolangFields for ToyDocument {
    fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
        visitor.field("condition", &CONDITION, &mut self.condition);
        visitor.field("script", &SCRIPT, &mut self.script);
        visitor.field("weights", &WEIGHT, &mut self.weights);
        visitor.field("data", &DATA, &mut self.data);
        visitor.nested("events", &mut self.events);
        visitor.field("tags", &TAGS, &mut self.tags);
    }
}

fn toy(json: &str) -> ToyDocument {
    facet_json::from_str(json).expect("the toy document parses")
}

/// A pack at 1.18.20 (6) holding a document of `format_version` 1.20.40 (10).
fn context() -> MolangContext {
    MolangContext::new(
        Some(&MinEngineVersion {
            version: SemVersion::new(1, 18, 20),
        }),
        ExperimentMask::empty(),
        &FormatVersionInput::parse("1.20.40").unwrap(),
        None,
    )
}

fn base() -> Location {
    let mut path = JsonPath::new();
    path.push("minecraft:toy");
    Location::new(None, None, path, None)
}

const VALID: &str = r#"{
    "condition": "q.is_baby",
    "script": "v.x = 1;",
    "weights": [0.5, "v.w", true],
    "data": [1, "q.y", ["q.z"]],
    "events": {"a": {"condition": {"expression": "q.x", "version": 3}}, "b": {"condition": 2}},
    "tags": "q.any_tag('stone')"
}"#;

fn raw_versions(document: &mut ToyDocument) -> Vec<(String, Option<i16>)> {
    let mut versions = Vec::new();
    molang::visit_fields(document, &base().json_path, |field| {
        let path = field.path.to_string();
        let mut push = |source: &MolangSource| {
            versions.push((path.clone(), source.raw_version().map(i16::from)));
        };
        match field.value {
            MolangFieldMut::Value(value) => {
                if let Some(source) = value.source() {
                    push(source);
                }
            }
            MolangFieldMut::Source(source) => push(source),
            MolangFieldMut::List(list) => list_sources(list, &mut push),
        }
    });
    versions
}

fn list_sources(list: &MolangList, push: &mut impl FnMut(&MolangSource)) {
    match list {
        MolangList::One(value) => {
            if let Some(source) = value.source() {
                push(source);
            }
        }
        MolangList::Many(elements) => {
            for element in elements {
                list_sources(element, push);
            }
        }
    }
}

/// After the load, no expression is without a version, each from its field's
/// source, and an object form keeps its own.
#[test]
fn pass_one_gives_every_expression_its_fields_version() {
    let mut document = toy(VALID);
    // facet has no load context: every string form arrives without a version.
    assert!(
        raw_versions(&mut document)
            .iter()
            .any(|(_, version)| version.is_none())
    );
    let mut diagnostics = Diagnostics::default();
    let pass = molang::pass_one(&mut document, &context(), &base(), &mut diagnostics);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(pass.failed, 0);
    assert_eq!(pass.sources, 7);
    let versions = raw_versions(&mut document);
    let expected: Vec<(String, Option<i16>)> = [
        ("minecraft:toy/condition", 6),
        ("minecraft:toy/script", 6),
        ("minecraft:toy/weights/1", 10),
        ("minecraft:toy/data", 6),
        ("minecraft:toy/data", 6),
        ("minecraft:toy/events/a/condition", 3),
        ("minecraft:toy/tags", 13),
    ]
    .into_iter()
    .map(|(path, version)| (path.to_owned(), Some(version)))
    .collect();
    assert_eq!(versions, expected);
    assert!(versions.iter().all(|(_, version)| version.is_some()));
    assert_eq!(
        document.events["a"].condition.source().unwrap().form(),
        SourceForm::Object {
            version: RawVersion(3)
        }
    );
}

/// A shape facet accepts but the field's reader refuses is `InvalidType` at the field's path, at
/// error level, in the parse phase: a bool in a strict versioned field, the object form in a
/// string-only field.
#[test]
fn pass_one_reports_a_field_its_reader_refuses() {
    let mut document = toy(r#"{
        "condition": true,
        "script": {"expression": "v.x", "version": 6},
        "events": {"a": {"condition": "q.x"}, "b": {"condition": false}}
    }"#);
    let mut diagnostics = Diagnostics::default();
    let pass = molang::pass_one(&mut document, &context(), &base(), &mut diagnostics);
    assert_eq!(pass.failed, 3);
    let found: Vec<(String, &DiagnosticCode, Severity, Phase)> = diagnostics
        .iter()
        .map(|d| {
            (
                d.location.json_path.to_string(),
                &d.code,
                d.severity,
                d.phase,
            )
        })
        .collect();
    let invalid_type = DiagnosticCode::InvalidType { text: None };
    let expected: Vec<(String, &DiagnosticCode, Severity, Phase)> = [
        "minecraft:toy/condition",
        "minecraft:toy/script",
        "minecraft:toy/events/b/condition",
    ]
    .into_iter()
    .map(|path| {
        (
            path.to_owned(),
            &invalid_type,
            Severity::Error,
            Phase::Parse,
        )
    })
    .collect();
    assert_eq!(found, expected);
    // The fields that passed still got their version.
    assert_eq!(
        document.events["a"]
            .condition
            .source()
            .unwrap()
            .raw_version(),
        Some(RawVersion(6))
    );
}

/// `ReprError::NumberOutOfRange` is `InvalidValue`: facet never produces such a constant, but a
/// value built by hand can hold one.
#[test]
fn a_number_out_of_range_is_invalid_value() {
    let mut document = toy(r#"{"condition": 1, "script": "1"}"#);
    document.condition = Molang::Const(f32::INFINITY);
    let mut diagnostics = Diagnostics::default();
    molang::pass_one(&mut document, &context(), &base(), &mut diagnostics);
    let codes: Vec<&DiagnosticCode> = diagnostics.iter().map(|d| &d.code).collect();
    assert!(matches!(
        codes[..],
        [DiagnosticCode::InvalidValue { text: None }]
    ));
}

/// Without the compile pass a document that holds expressions gets one
/// `MolangNotChecked` info naming how many.
#[test]
fn not_checked_names_the_number_of_expressions() {
    let mut document = toy(VALID);
    let mut diagnostics = Diagnostics::default();
    molang::pass_one(&mut document, &context(), &base(), &mut diagnostics);
    assert_eq!(
        molang::not_checked(&mut document, &base(), &mut diagnostics),
        7
    );
    let [finding] = diagnostics.iter().collect::<Vec<_>>()[..] else {
        panic!("one finding: {diagnostics:?}");
    };
    assert_eq!(finding.code, DiagnosticCode::MolangNotChecked { fields: 7 });
    assert_eq!(finding.severity, Severity::Info);
    assert_eq!(finding.location, base());
}

/// A document whose Molang fields hold only constants needs no check and gets no finding.
#[test]
fn not_checked_is_silent_without_expressions() {
    #[derive(Debug, facet::Facet)]
    struct OnlyConstants {
        values: Vec<Molang>,
    }
    impl MolangFields for OnlyConstants {
        fn visit_molang(&mut self, visitor: &mut MolangVisitor<'_>) {
            visitor.field("values", &CONDITION, &mut self.values);
        }
    }
    let mut document: OnlyConstants =
        facet_json::from_str(r#"{"values": [1, 2.5, false]}"#).unwrap();
    let mut diagnostics = Diagnostics::default();
    assert_eq!(
        molang::not_checked(&mut document, &base(), &mut diagnostics),
        0
    );
    assert!(diagnostics.is_empty());
}

/// Pass 1 never compiles, so an invalid expression loads with no
/// `InvalidMolang`; without the compile pass the document only gets its `MolangNotChecked`.
#[test]
fn an_invalid_expression_loads_unchecked_without_the_compile_pass() {
    let mut document = toy(r#"{"condition": "1 +* ((", "script": "v.x = ;"}"#);
    let mut diagnostics = Diagnostics::default();
    let pass = molang::pass_one(&mut document, &context(), &base(), &mut diagnostics);
    assert_eq!(pass.failed, 0);
    molang::not_checked(&mut document, &base(), &mut diagnostics);
    let codes: Vec<&DiagnosticCode> = diagnostics.iter().map(|d| &d.code).collect();
    assert_eq!(codes, [&DiagnosticCode::MolangNotChecked { fields: 2 }]);
}
