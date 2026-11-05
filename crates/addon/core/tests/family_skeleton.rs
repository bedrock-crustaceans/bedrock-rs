//! The family skeleton: a toy one-version family with no beta struct, its chain, its document
//! traits and its loader, each built from core's generic pieces the way a family writes them.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, VanillaText};
use bedrock_addon_core::document::NoBeta;
use bedrock_addon_core::id::{BlockId, kind};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    Family, Loaded, NoComponents, ParseCx, VersionStruct, Versioned, load, parse_payload,
    write_payload,
};
use facet::Facet;

/// `description` of a glint.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct GlintDescription {
    pub identifier: BlockId,
}

/// The one struct of the toy chain: `{ "description": {..}, "level": n }` at 1.0.0.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Glint {
    pub description: GlintDescription,
    pub level: u8,
}

impl Versioned for Glint {
    const VERSION: FormatVersion = FormatVersion::new(1, 0, 0);
}

impl VersionStruct for Glint {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_payload(payload, cx, d)
    }
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

/// The toy chain as a family writes one today, with `version_chain!`.
mod by_macro {
    use super::*;

    pub struct GlintChain;

    bedrock_addon_core::version_chain! {
        family = GlintChain;
        #[derive(Clone, Debug, PartialEq)]
        pub enum Any {
            V1_0_0(Glint),
        }
        molang = none;
    }

    impl Family for GlintChain {
        const NAME: &'static str = "glint";
        const PAYLOAD_KEY: &'static str = "demo:glint";
        const DIRECTORIES: &'static [&'static str] = &["glints"];
        const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
        type Latest = Glint;
        type Beta = NoBeta<kind::Block>;
        type Component = NoComponents;
        type Any = Any;
    }
}

const GLINT: &str = r#"{"format_version": "1.0.0", "demo:glint": {"description": {"identifier": "demo:spark"}, "level": 3}}"#;

fn spark() -> Glint {
    Glint {
        description: GlintDescription {
            identifier: BlockId::parse("demo:spark"),
        },
        level: 3,
    }
}

/// The newest struct of any family, written through the chain's own writer.
fn written_latest<F: Family>(value: &F::Latest) -> Value {
    value.write().unwrap()
}

#[test]
fn a_family_without_a_beta_struct_names_no_beta_and_its_value_clones() {
    let outcome = load::<by_macro::GlintChain>(JsonSlice::new(GLINT), &LoadContext::detached());
    let Some(loaded) = outcome.value else {
        panic!("rejected: {:?}", outcome.diagnostics)
    };
    assert_eq!(loaded.clone(), Loaded::Stable(spark()));
    let Loaded::Stable(glint) = loaded else {
        unreachable!("no beta struct")
    };
    assert_eq!(
        written_latest::<by_macro::GlintChain>(&glint),
        write_payload(&spark()).unwrap()
    );
}

// ---- write_document ------------------------------------------------------------------------

/// The toy struct on the raw-text path below 2.0.0: the family reads its root struct from the
/// whole document there.
mod raw_below_2 {
    use super::*;
    use bedrock_addon_core::versioning::EnvelopeCheck;

    pub struct RawGlintChain;

    bedrock_addon_core::version_chain! {
        family = RawGlintChain;
        pub enum Any {
            V1_0_0(Glint),
        }
    }

    impl Family for RawGlintChain {
        const NAME: &'static str = "raw glint";
        const PAYLOAD_KEY: &'static str = "demo:glint";
        const DIRECTORIES: &'static [&'static str] = &["glints"];
        const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
        const ENVELOPE: EnvelopeCheck = EnvelopeCheck::From(FormatVersion::new(2, 0, 0));
        type Latest = Glint;
        type Beta = NoBeta<kind::Block>;
        type Component = NoComponents;
        type Any = Any;
    }
}

fn text(value: &Value) -> String {
    bedrock_addon_core::json::to_json(value).unwrap()
}

#[test]
fn write_document_puts_the_version_first_and_the_payload_under_the_family_key() {
    use bedrock_addon_core::document::Extra;
    use bedrock_addon_core::versioning::{__macro::document_json, write_document};
    let payload = write_payload(&spark()).unwrap();
    let v = FormatVersion::new(1, 0, 0);
    let written = write_document::<by_macro::GlintChain>(v, payload.clone(), &Extra::default());
    assert_eq!(
        text(&written),
        document_json::<by_macro::GlintChain>(v, &payload).unwrap(),
        "the envelope the generated `Any::to_json` writes"
    );
    let compact: String = text(&written).split_whitespace().collect();
    assert_eq!(
        compact,
        r#"{"format_version":"1.0.0","demo:glint":{"description":{"identifier":"demo:spark"},"level":3}}"#
    );
}

#[test]
fn write_document_writes_the_beta_version_as_beta() {
    use bedrock_addon_core::document::Extra;
    use bedrock_addon_core::versioning::{BETA_VERSION, write_document};
    let written = write_document::<by_macro::GlintChain>(
        BETA_VERSION,
        write_payload(&spark()).unwrap(),
        &Extra::default(),
    );
    let root = written.as_object().unwrap();
    assert_eq!(
        root.get("format_version")
            .and_then(Value::as_string)
            .map(facet_value::VString::as_str),
        Some("beta")
    );
}

#[test]
fn write_document_writes_the_root_extras_after_the_payload_in_their_order() {
    use bedrock_addon_core::document::Extra;
    use bedrock_addon_core::versioning::write_document;
    let extras: Extra = [
        ("zeta".to_owned(), Value::from(1)),
        ("alpha".to_owned(), Value::from(true)),
    ]
    .into_iter()
    .collect();
    let written = write_document::<by_macro::GlintChain>(
        FormatVersion::new(1, 0, 0),
        Value::from(facet_value::VObject::new()),
        &extras,
    );
    let keys: Vec<&str> = written
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, _)| key.as_str())
        .collect();
    assert_eq!(keys, ["format_version", "demo:glint", "zeta", "alpha"]);
}

#[test]
fn write_document_on_the_raw_text_path_is_the_payload_alone() {
    use bedrock_addon_core::document::Extra;
    use bedrock_addon_core::versioning::{__macro::document_json, write_document};
    let payload = write_payload(&spark()).unwrap();
    let extras: Extra = [("zeta".to_owned(), Value::from(1))].into_iter().collect();
    let below = FormatVersion::new(1, 0, 0);
    let written = write_document::<raw_below_2::RawGlintChain>(below, payload.clone(), &extras);
    assert_eq!(
        written, payload,
        "the payload is the root; nothing is added to it"
    );
    assert_eq!(
        text(&written),
        document_json::<raw_below_2::RawGlintChain>(below, &payload).unwrap()
    );
    // From the envelope's version on, the envelope.
    let at = FormatVersion::new(2, 0, 0);
    let enveloped = write_document::<raw_below_2::RawGlintChain>(at, payload.clone(), &extras);
    assert!(enveloped.as_object().unwrap().get("demo:glint").is_some());
}

// ---- Single --------------------------------------------------------------------------------

/// The toy chain as a one-version family writes it with no `history` module: `Single<Self>`.
/// Everything but the chain is `by_macro::GlintChain`'s.
mod single {
    use super::*;
    use bedrock_addon_core::versioning::Single;

    pub struct GlintFamily;

    impl Family for GlintFamily {
        const NAME: &'static str = "glint";
        const PAYLOAD_KEY: &'static str = "demo:glint";
        const DIRECTORIES: &'static [&'static str] = &["glints"];
        const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
        type Latest = Glint;
        type Beta = NoBeta<kind::Block>;
        type Component = NoComponents;
        type Any = Single<Self>;

        /// The closing line of a failed file, once.
        fn close(
            diagnostics: &mut Diagnostics,
            _: Option<JsonSlice<'_>>,
            path: &PackPath,
            failed: bool,
        ) {
            if failed {
                diagnostics.push(super::closing_line(path));
            }
        }
    }
}

/// The toy family's closing line of a failed file.
fn closing_line(path: &PackPath) -> Diagnostic {
    Diagnostic::error(DiagnosticCode::DocumentRejected {
        text: Some(VanillaText::Formatted {
            template: "glint {} failed",
            args: vec![path.as_str().to_owned()],
        }),
    })
}

/// Documents the one-version chain must read as `version_chain!`'s chain of one does: a good
/// one, one below the floor, one without a version, a beta one, one whose payload fails.
const SINGLE_CASES: &[&str] = &[
    GLINT,
    r#"{"format_version": "0.9.0", "demo:glint": {"description": {"identifier": "demo:spark"}, "level": 3}}"#,
    r#"{"demo:glint": {"description": {"identifier": "demo:spark"}, "level": 3}}"#,
    r#"{"format_version": "beta", "demo:glint": {"description": {"identifier": "demo:spark"}, "level": 3}}"#,
    r#"{"format_version": "1.0.0", "demo:glint": {"description": {"identifier": "demo:spark"}, "level": "high"}}"#,
];

#[test]
fn single_has_one_version_and_no_beta() {
    use bedrock_addon_core::versioning::{Single, VersionChain};
    assert_eq!(
        <Single<single::GlintFamily> as VersionChain<single::GlintFamily>>::VERSIONS,
        <by_macro::Any as VersionChain<by_macro::GlintChain>>::VERSIONS
    );
    const { assert!(!<Single<single::GlintFamily> as VersionChain<single::GlintFamily>>::HAS_BETA) };
}

#[test]
fn single_loads_every_document_as_a_generated_chain_of_one_does() {
    let cx = LoadContext::detached();
    for case in SINGLE_CASES {
        let generated = load::<by_macro::GlintChain>(JsonSlice::new(case), &cx);
        let single = load::<single::GlintFamily>(JsonSlice::new(case), &cx);
        let value = |loaded: Option<Glint>| loaded;
        assert_eq!(
            value(single.value.map(|v| match v {
                Loaded::Stable(glint) => glint,
                Loaded::Beta(_) => unreachable!("no beta struct"),
            })),
            value(generated.value.map(|v| match v {
                Loaded::Stable(glint) => glint,
                Loaded::Beta(_) => unreachable!("no beta struct"),
            })),
            "{case}"
        );
        assert_eq!(single.diagnostics, generated.diagnostics, "{case}");
        assert_eq!(single.version, generated.version, "{case}");
        assert_eq!(
            single.original_version, generated.original_version,
            "{case}"
        );
    }
}

#[test]
fn single_parses_as_written_and_writes_its_document_back() {
    use bedrock_addon_core::versioning::{Single, VersionChain, parse_as_written};
    let cx = LoadContext::detached();
    let parsed = parse_as_written::<single::GlintFamily>(JsonSlice::new(GLINT), &cx);
    let value: Single<single::GlintFamily> = parsed.value.expect("parses");
    assert_eq!(value, Single(spark()));
    assert_eq!(value.clone().0, spark());
    assert_eq!(value.parsed_version(), FormatVersion::new(1, 0, 0));
    let generated = parse_as_written::<by_macro::GlintChain>(JsonSlice::new(GLINT), &cx)
        .value
        .expect("parses");
    assert_eq!(value.to_json().unwrap(), generated.to_json().unwrap());
    // The chain's own parse: by version, under the range model.
    let mut d = Diagnostics::default();
    let payload = r#"{"description": {"identifier": "demo:spark"}, "level": 3}"#;
    let by_version = <Single<single::GlintFamily> as VersionChain<single::GlintFamily>>::parse(
        JsonSlice::new(payload),
        FormatVersion::new(1, 0, 0),
        &cx,
        &mut d,
    );
    assert_eq!(by_version, Some(Single(spark())), "{d:?}");
}

// ---- JsonDocument and JsonDefinition -------------------------------------------------------

use bedrock_addon_core::document::{
    CombineRule, Content, Definition, Document, JsonContent, JsonDefinition, JsonDocument, Notice,
    Origin, Placed, Rejected, default_path,
};
use bedrock_addon_core::id::Id;
use bedrock_addon_core::json::WriteStyle;
use bedrock_addon_core::load::StackCx;
use bedrock_addon_core::pack_identity::{PackRef, PackSide};

/// The whole of what the toy family states for its file kind.
impl JsonDocument for Glint {
    type Family = single::GlintFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];
    fn place(&self) -> PackPath {
        default_path("glints", &self.description.identifier)
    }
}

impl JsonDefinition for Glint {
    type Kind = kind::Block;
    fn identifier(&self) -> &BlockId {
        &self.description.identifier
    }
    /// The glint loaded last wins, each replaced one noted (voxel shape's rule).
    fn combine_rule(_: &StackCx<'_>) -> CombineRule<Self> {
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        }
    }
}

/// A file kind with every default overridden: written at the version it was read at, with a
/// root member after the payload, refusing a level of 0, named apart from its family.
#[derive(Clone, Debug, PartialEq)]
pub struct Ember {
    pub heat: u8,
    /// The version the file was read at (not JSON).
    pub read_at: Option<FormatVersion>,
}

/// What `Ember` is read from and written as.
#[derive(Facet)]
struct EmberJson {
    heat: u8,
}

impl Versioned for Ember {
    const VERSION: FormatVersion = FormatVersion::new(1, 2, 0);
}

impl VersionStruct for Ember {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let json: EmberJson = parse_payload(payload, cx, d)?;
        Some(Ember {
            heat: json.heat,
            read_at: cx.written.version,
        })
    }
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(&EmberJson { heat: self.heat })
    }
}

impl bedrock_addon_core::molang::MolangFields for Ember {
    fn visit_molang(&mut self, _: &mut bedrock_addon_core::molang::MolangVisitor<'_>) {}
}

pub struct EmberFamily;

impl Family for EmberFamily {
    const NAME: &'static str = "ember";
    const PAYLOAD_KEY: &'static str = "demo:ember";
    const DIRECTORIES: &'static [&'static str] = &[];
    const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
    type Latest = Ember;
    type Beta = NoBeta<kind::Block>;
    type Component = NoComponents;
    type Any = bedrock_addon_core::versioning::Single<Self>;
}

impl JsonDocument for Ember {
    type Family = EmberFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior, PackSide::Resource];
    const NAME: &'static str = "ember settings";
    fn place(&self) -> PackPath {
        PackPath::from_static("embers.json")
    }
    fn write_version(&self) -> FormatVersion {
        self.read_at.unwrap_or(<Self as Versioned>::VERSION)
    }
    fn write_extras(&self, root: &mut bedrock_addon_core::document::Extra) {
        root.insert("demo:note".to_owned(), Value::from("kept"));
    }
    fn refuse_write(&self) -> Result<(), EncodeError> {
        if self.heat == 0 {
            return Err(EncodeError::NotRepresentable {
                at: bedrock_addon_core::diagnostic::JsonPath::new(),
                what: bedrock_addon_core::json::Unrepresentable::Unsupported,
            });
        }
        Ok(())
    }
}

/// A non-JSON file kind beside the blanket impls: its own `Content`, as lang has.
#[derive(Clone, Debug, PartialEq)]
pub struct Notes(pub String);

impl bedrock_addon_core::__private::Sealed for Notes {}

impl Content for Notes {
    const NAME: &'static str = "notes";
    const SIDES: &'static [PackSide] = &[PackSide::Resource];
    fn parse(path: &PackPath, bytes: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        let origin = Origin::file(PackRef::app_data(), PackSide::Resource, path.clone());
        Ok(bedrock_addon_core::document::unversioned::document(
            Notes(String::from_utf8_lossy(bytes).into_owned()),
            origin,
            Diagnostics::default(),
        ))
    }
    fn write_json(&self, out: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
        out.extend_from_slice(self.0.as_bytes());
        Ok(())
    }
}

impl Placed for Notes {
    fn default_path(&self) -> PackPath {
        PackPath::from_static("notes.txt")
    }
}

/// An entry of a multi-definition file beside the blanket impls: a hand `Definition` whose file
/// is a `JsonDocument`.
pub struct Entry;

impl bedrock_addon_core::__private::Sealed for Entry {}

impl Definition for Entry {
    type Kind = kind::Block;
    type File = Glint;
    type Beta = NoBeta<kind::Block>;
}

/// `D` is a definition of blocks in its own file, without a beta struct.
fn a_block_definition_of_its_own_file<
    D: Definition<Kind = kind::Block, File = D, Beta = NoBeta<kind::Block>>,
>() {
}

fn glint_path() -> PackPath {
    PackPath::from_static("glints/demo/spark.json")
}

#[test]
fn a_json_definition_is_a_definition_of_its_own_file() {
    a_block_definition_of_its_own_file::<Glint>();
    assert_eq!(<Glint as Content>::NAME, "glint", "the family's name");
    assert_eq!(<Glint as Content>::SIDES, [PackSide::Resource]);
    assert_eq!(<Ember as Content>::NAME, "ember settings");
    // The hand impls beside the blanket ones are unchanged.
    assert_eq!(<Notes as Content>::NAME, "notes");
    assert_eq!(Notes(String::new()).default_path().as_str(), "notes.txt");
    let _: fn() -> PackPath = glint_path;
    assert!(std::any::type_name::<<Entry as Definition>::File>().ends_with("Glint"));
}

#[test]
fn a_json_document_parses_a_file_read_on_its_own_through_its_chain() {
    let document =
        <Glint as Content>::parse(&glint_path(), GLINT.as_bytes(), &LoadContext::detached())
            .expect("loads");
    assert_eq!(*document.value(), spark());
    assert_eq!(
        *document.origin(),
        Origin::file(PackRef::app_data(), PackSide::Resource, glint_path())
    );
    assert_eq!(document.version().effective, FormatVersion::new(1, 0, 0));
    assert!(
        document.diagnostics().is_empty(),
        "{:?}",
        document.diagnostics()
    );
}

#[test]
fn a_json_document_that_fails_is_rejected_in_its_file_and_closed_once() {
    let cx = LoadContext::detached();
    let bad = SINGLE_CASES[4];
    let rejected =
        <Glint as Content>::parse(&glint_path(), bad.as_bytes(), &cx).expect_err("level is no u8");
    assert_eq!(rejected.origin.path, glint_path());
    let closing: Vec<_> = rejected
        .diagnostics
        .iter()
        .filter(|d| **d == closing_line(&glint_path()).in_file(&rejected.origin))
        .collect();
    assert_eq!(closing.len(), 1, "{:?}", rejected.diagnostics);
    for finding in &rejected.diagnostics {
        assert_eq!(finding.location.path, Some(glint_path()));
        assert_eq!(finding.location.pack, Some(PackRef::app_data()));
    }
    // Bytes that are no text: the malformed finding, then the close.
    let unreadable =
        <Glint as Content>::parse(&glint_path(), &[0xff, 0xfe, 0x00], &cx).expect_err("no text");
    let codes: Vec<_> = unreadable.diagnostics.iter().map(|d| &d.code).collect();
    assert!(
        matches!(
            codes.as_slice(),
            [
                DiagnosticCode::MalformedInput { .. },
                DiagnosticCode::DocumentRejected { .. }
            ]
        ),
        "{codes:?}"
    );
}

#[test]
fn a_json_document_writes_the_family_envelope_at_its_newest_version() {
    let written = spark().to_json().unwrap();
    let generated = parse_as_written_glint().value().to_json().unwrap();
    assert_eq!(written, generated);
    // `write_document` of JsonContent is the same text.
    let mut out = Vec::new();
    spark()
        .write_document(&mut out, WriteStyle::default())
        .unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), written);
    assert_eq!(spark().default_path().as_str(), "glints/demo/spark.json");
}

#[test]
fn a_json_document_writes_through_its_overrides() {
    let text = r#"{"format_version": "1.1.0", "demo:ember": {"heat": 4}}"#;
    let ember = <Ember as Content>::parse(
        &PackPath::from_static("embers.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("loads");
    assert_eq!(ember.origin().side, PackSide::Behavior, "the first side");
    let compact: String = ember.to_json().unwrap().split_whitespace().collect();
    assert_eq!(
        compact,
        r#"{"format_version":"1.1.0","demo:ember":{"heat":4},"demo:note":"kept"}"#
    );
    let cold = Ember {
        heat: 0,
        read_at: None,
    };
    assert!(matches!(
        cold.to_json(),
        Err(EncodeError::NotRepresentable { .. })
    ));
    assert_eq!(cold.default_path().as_str(), "embers.json");
}

fn parse_as_written_glint() -> Document<bedrock_addon_core::versioning::Single<single::GlintFamily>>
{
    Glint::parse_as_written(&glint_path(), GLINT.as_bytes(), &LoadContext::detached())
        .expect("parses")
}

#[test]
fn a_json_document_parses_as_written_without_upgrading() {
    let document = parse_as_written_glint();
    assert_eq!(document.value().0, spark());
    assert_eq!(
        *document.origin(),
        Origin::file(PackRef::app_data(), PackSide::Resource, glint_path())
    );
    assert!(document.diagnostics().is_empty());
    let rejected = Glint::parse_as_written(
        &glint_path(),
        SINGLE_CASES[4].as_bytes(),
        &LoadContext::detached(),
    )
    .expect_err("level is no u8");
    let closings = rejected
        .diagnostics
        .iter()
        .filter(|d| matches!(d.code, DiagnosticCode::DocumentRejected { .. }))
        .count();
    assert_eq!(closings, 1, "closed once");
}

#[test]
fn the_identifier_of_a_json_definition_is_its_own() {
    let glint = spark();
    let id: &Id<<Glint as Definition>::Kind> = glint.identifier();
    assert_eq!(id.as_str(), "demo:spark");
}

// ---- the loader constructors ---------------------------------------------------------------

use bedrock_addon_core::document::{
    Contribution, Files, FilesBuilder, FilesRule, Registry, RegistryBuilder,
};
use bedrock_addon_core::load::{
    CollectionLoader, FileCopy, LoadOptions, StackPack, Walk, WalkFile,
};
use bedrock_addon_core::pack_identity::{PackIdVersion, PackOrigin, PackType};
use bedrock_addon_core::version::SemVersion;

/// The whole loader of the toy family: one line.
const GLINTS: CollectionLoader<Registry<Glint>, Contribution<Glint>> =
    bedrock_addon_core::load::registry::<Glint>(Walk::json_tree_in(&["glints"]));

/// A loader that reads every copy of `notes.txt`, the top pack's winning.
const NOTES: CollectionLoader<Files<Notes>, FileCopy<Notes>> =
    bedrock_addon_core::load::files::<Notes>(Walk::exact_files(&["notes.txt"]));

/// The notes of every pack, joined top pack first.
fn join_notes(parsed: Vec<FileCopy<Notes>>, _: &StackCx<'_>) -> String {
    let mut notes: Vec<_> = parsed
        .into_iter()
        .filter_map(|(_, copy)| copy.ok())
        .map(|document| (document.origin().pack.index(), document.into_value().0))
        .collect();
    notes.sort();
    notes
        .into_iter()
        .map(|(_, text)| text)
        .collect::<Vec<_>>()
        .join("|")
}

const JOINED_NOTES: CollectionLoader<String, FileCopy<Notes>> =
    bedrock_addon_core::load::merged::<Notes, String>(
        Walk::exact_files(&["notes.txt"]),
        join_notes,
    );

fn pack(index: usize) -> PackRef {
    PackRef::new(
        index,
        PackIdVersion::new(
            uuid::Uuid::from_u128(index as u128 + 1),
            SemVersion::new(1, 0, 0),
            PackType::Resource,
        ),
        format!("pack {index}"),
    )
}

fn stack() -> Vec<StackPack> {
    (0..3)
        .map(|i| StackPack::new(pack(i), PackOrigin::Dev, LoadContext::detached()))
        .collect()
}

/// The walk's files, bottom pack first: pack 2's spark, pack 1's failing spark and an ember
/// glint, pack 0's spark at another level, in a subpack.
fn walked() -> Vec<WalkFile> {
    let file = |pack_index: usize, path: &'static str, key: &'static str, text: String| WalkFile {
        pack: pack(pack_index),
        side: PackSide::Resource,
        path: PackPath::from_static(path),
        key: PackPath::from_static(key),
        bytes: bytes::Bytes::from(text.into_bytes()),
    };
    let glint = |id: &str, level: &str| {
        format!(
            r#"{{"format_version": "1.0.0", "demo:glint": {{"description": {{"identifier": "{id}"}}, "level": {level}}}}}"#
        )
    };
    vec![
        file(
            2,
            "glints/demo/spark.json",
            "glints/demo/spark.json",
            glint("demo:spark", "1"),
        ),
        file(
            1,
            "glints/demo/spark.json",
            "glints/demo/spark.json",
            glint("demo:spark", "\"high\""),
        ),
        file(
            1,
            "glints/demo/ember.json",
            "glints/demo/ember.json",
            glint("demo:ember", "2"),
        ),
        file(
            0,
            "subpacks/hd/glints/demo/spark.json",
            "glints/demo/spark.json",
            glint("demo:spark", "9"),
        ),
    ]
}

/// The loader a family writes by hand today (voxel shape's): its parse and combine.
fn hand_parse(file: &WalkFile, cx: &LoadContext) -> Contribution<Glint> {
    let origin = Origin::file(file.pack.clone(), file.side, file.path.clone());
    let (identifier, document) = bedrock_addon_core::versioning::parse_document::<
        single::GlintFamily,
    >(origin, &file.bytes, cx);
    match document {
        Ok(document) => Contribution::Stable(document.description.identifier.clone(), document),
        Err(rejected) => Contribution::Rejected(identifier.map(Id::parse), rejected),
    }
}

fn hand_combine(parsed: Vec<Contribution<Glint>>, stack: &StackCx<'_>) -> Registry<Glint> {
    RegistryBuilder::combine(
        parsed,
        CombineRule::LastLoadedWins {
            replaced: Some(Notice::shadowed()),
        },
        CombineRule::top_wins(),
        stack.options,
    )
}

/// Each identifier's effective origin and value, the rejections' origins, every finding.
type Summary = (Vec<(String, Origin, Glint)>, Vec<Origin>, Vec<Diagnostic>);

/// What a registry holds, for comparing two: each identifier's effective origin and value, the
/// rejections' origins and every finding.
fn summary(registry: &Registry<Glint>) -> Summary {
    (
        registry
            .iter()
            .map(|(id, document)| {
                (
                    id.to_string(),
                    document.origin().clone(),
                    document.value().clone(),
                )
            })
            .collect(),
        registry
            .rejected()
            .iter()
            .map(|r| r.origin.clone())
            .collect(),
        registry.diagnostics().cloned().collect(),
    )
}

#[test]
fn registry_loads_a_json_definition_as_the_hand_written_loader_does() {
    let options = LoadOptions::default();
    let packs = stack();
    let cx = StackCx::new(&options, &packs);
    let load = LoadContext::detached();
    let files = walked();
    let generic = (GLINTS.combine)(
        files.iter().map(|f| (GLINTS.parse)(f, &load)).collect(),
        &cx,
    );
    let hand = hand_combine(files.iter().map(|f| hand_parse(f, &load)).collect(), &cx);
    assert_eq!(summary(&generic), summary(&hand));
    // Last loaded wins: pack 0's subpack copy, at level 9, in its own file.
    let spark = generic
        .get(&BlockId::parse("demo:spark"))
        .expect("registered");
    assert_eq!(spark.level, 9);
    assert_eq!(
        spark.origin().path.as_str(),
        "subpacks/hd/glints/demo/spark.json"
    );
    assert_eq!(spark.origin().pack, pack(0));
    assert_eq!(
        generic.get(&BlockId::parse("demo:ember")).map(|d| d.level),
        Some(2)
    );
    assert_eq!(generic.rejected().len(), 1);
    assert!(
        generic
            .diagnostics()
            .any(|d| matches!(d.code, DiagnosticCode::ShadowedDefinition { .. })),
        "the family's rule: each replaced glint noted"
    );
    assert!(GLINTS.needs.is_empty() && GLINTS.then.is_none());
    assert_eq!(GLINTS.walk.directories, ["glints"]);
}

#[test]
fn a_failed_definition_names_the_identifier_its_payload_gave() {
    let load = LoadContext::detached();
    let failing = &walked()[1];
    let Contribution::Rejected(id, rejected) = (GLINTS.parse)(failing, &load) else {
        panic!("level is no u8")
    };
    assert_eq!(id, Some(BlockId::parse("demo:spark")));
    assert_eq!(
        rejected.origin,
        Origin::file(pack(1), PackSide::Resource, failing.path.clone())
    );
}

#[test]
fn files_keys_each_copy_by_its_path_in_its_pack_and_the_top_pack_wins() {
    let options = LoadOptions::default();
    let packs = stack();
    let cx = StackCx::new(&options, &packs);
    let load = LoadContext::detached();
    let note = |index: usize, path: &'static str, text: &'static str| WalkFile {
        pack: pack(index),
        side: PackSide::Resource,
        path: PackPath::from_static(path),
        key: PackPath::from_static("notes.txt"),
        bytes: bytes::Bytes::from_static(text.as_bytes()),
    };
    let files = [
        note(1, "notes.txt", "low"),
        note(0, "subpacks/hd/notes.txt", "top"),
    ];
    let parsed: Vec<FileCopy<Notes>> = files.iter().map(|f| (NOTES.parse)(f, &load)).collect();
    let (key, copy) = &parsed[1];
    assert_eq!(key.as_str(), "notes.txt");
    let document = copy.as_ref().expect("parses");
    assert_eq!(
        *document.origin(),
        Origin::file(
            pack(0),
            PackSide::Resource,
            PackPath::from_static("subpacks/hd/notes.txt")
        ),
        "the file in its own pack, not read on its own"
    );
    let hand = FilesBuilder::combine(parsed.clone(), FilesRule::TopWins, &options);
    let generic = (NOTES.combine)(parsed.clone(), &cx);
    let top = generic
        .get(&PackPath::from_static("notes.txt"))
        .expect("one file");
    assert_eq!(top.value().0, "top");
    assert_eq!(
        hand.get(&PackPath::from_static("notes.txt"))
            .map(|d| d.origin().clone()),
        Some(top.origin().clone())
    );
    assert_eq!((JOINED_NOTES.combine)(parsed, &cx), "top|low");
}
