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

// ---- write_envelope ------------------------------------------------------------------------

/// The toy struct on the raw-text path below 2.0.0: the family reads its root struct from the
/// whole document there.
mod raw_below_2 {
    use super::*;
    use bedrock_addon_core::versioning::{Envelope, EnvelopeCheck};

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
        const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
        const ENVELOPE: Envelope = Envelope {
            check: EnvelopeCheck::From(FormatVersion::new(2, 0, 0)),
            ..Envelope::DEFAULT
        };
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
fn write_envelope_puts_the_version_first_and_the_payload_under_the_family_key() {
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::versioning::{__macro::document_json, write_envelope};
    let payload = write_payload(&spark()).unwrap();
    let v = FormatVersion::new(1, 0, 0);
    let written =
        write_envelope::<by_macro::GlintChain>(v, payload.clone(), &UnknownMembers::default());
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
fn write_envelope_writes_the_beta_version_as_beta() {
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::versioning::{BETA_VERSION, write_envelope};
    let written = write_envelope::<by_macro::GlintChain>(
        BETA_VERSION,
        write_payload(&spark()).unwrap(),
        &UnknownMembers::default(),
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
fn write_envelope_writes_the_root_extras_after_the_payload_in_their_order() {
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::versioning::write_envelope;
    let extras: UnknownMembers = [
        ("zeta".to_owned(), Value::from(1)),
        ("alpha".to_owned(), Value::from(true)),
    ]
    .into_iter()
    .collect();
    let written = write_envelope::<by_macro::GlintChain>(
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
fn write_envelope_on_the_raw_text_path_is_the_payload_alone() {
    use bedrock_addon_core::document::UnknownMembers;
    use bedrock_addon_core::versioning::{__macro::document_json, write_envelope};
    let payload = write_payload(&spark()).unwrap();
    let extras: UnknownMembers = [("zeta".to_owned(), Value::from(1))].into_iter().collect();
    let below = FormatVersion::new(1, 0, 0);
    let written = write_envelope::<raw_below_2::RawGlintChain>(below, payload.clone(), &extras);
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
    let enveloped = write_envelope::<raw_below_2::RawGlintChain>(at, payload.clone(), &extras);
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
    CombineRule, Content, Definition, Document, JsonContent, JsonDefinition, JsonDocument,
    JsonPlaced, Notice, Origin, Placed, Rejected, default_path,
};
use bedrock_addon_core::id::Id;
use bedrock_addon_core::json::WriteStyle;
use bedrock_addon_core::load::StackCx;
use bedrock_addon_core::pack_identity::{PackRef, PackSide};

/// The whole of what the toy family states for its file kind.
impl JsonDocument for Glint {
    type Family = single::GlintFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Resource];
}

impl JsonPlaced for Glint {
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
/// root member after the payload, refusing a level of 0, named apart from its family. Shaped
/// like trade's tables, it has no path of its own (no `JsonPlaced`, so no `Placed`).
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

// No Molang field: the empty walk a `Single` family's struct needs.
bedrock_addon_core::impl_no_molang!(Ember);

pub struct EmberFamily;

impl Family for EmberFamily {
    const NAME: &'static str = "ember";
    const PAYLOAD_KEY: &'static str = "demo:ember";
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
    fn write_version(&self) -> FormatVersion {
        self.read_at.unwrap_or(<Self as Versioned>::VERSION)
    }
    fn write_root_members(&self, root: &mut bedrock_addon_core::document::UnknownMembers) {
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

/// `C` is a file kind, whether placed or not.
fn content<C: Content>() {}

/// Whether `T` is `Placed`, by autoref specialisation: the inherent-like method on `&Probe<T>`
/// for a placed `T` is found before the fallback on `Probe<T>`.
struct Probe<T>(std::marker::PhantomData<T>);

trait IsPlaced {
    fn placed(&self) -> bool;
}

impl<T: Placed> IsPlaced for &Probe<T> {
    fn placed(&self) -> bool {
        true
    }
}

trait NotPlaced {
    fn placed(&self) -> bool;
}

impl<T> NotPlaced for Probe<T> {
    fn placed(&self) -> bool {
        false
    }
}

#[test]
#[allow(
    clippy::needless_borrow,
    reason = "the probe needs both borrows to try the placed impl first"
)]
fn a_json_document_is_placed_only_when_it_is_json_placed() {
    content::<Ember>();
    assert!((&&Probe::<Glint>(std::marker::PhantomData)).placed());
    assert!(!(&&Probe::<Ember>(std::marker::PhantomData)).placed());
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
}

/// A file kind that picks its format on write, as block and item do: a hot cinder is
/// written as the older document `{"format_version": "0.9.0", "demo:cinder": {"glow": n}}`,
/// whose payload is not the struct's; any other through the family's envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct Cinder {
    pub heat: u8,
}

impl Versioned for Cinder {
    const VERSION: FormatVersion = FormatVersion::new(1, 0, 0);
}

impl VersionStruct for Cinder {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        let json: EmberJson = parse_payload(payload, cx, d)?;
        Some(Cinder { heat: json.heat })
    }
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(&EmberJson { heat: self.heat })
    }
}

bedrock_addon_core::impl_no_molang!(Cinder);

pub struct CinderFamily;

impl Family for CinderFamily {
    const NAME: &'static str = "cinder";
    const PAYLOAD_KEY: &'static str = "demo:cinder";
    const FLOOR: FormatVersion = FormatVersion::new(0, 9, 0);
    type Latest = Cinder;
    type Beta = NoBeta<kind::Block>;
    type Component = NoComponents;
    type Any = bedrock_addon_core::versioning::Single<Self>;
}

impl JsonDocument for Cinder {
    type Family = CinderFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];
    fn write_root(&self) -> Result<Value, EncodeError> {
        use bedrock_addon_core::versioning::write_envelope;
        let extras = bedrock_addon_core::document::UnknownMembers::default();
        if self.heat > 9 {
            let older: Value =
                bedrock_addon_core::json::from_str_jsonc(&format!(r#"{{"glow": {}}}"#, self.heat))
                    .unwrap();
            return Ok(write_envelope::<CinderFamily>(
                FormatVersion::new(0, 9, 0),
                older,
                &extras,
            ));
        }
        Ok(write_envelope::<CinderFamily>(
            <Self as Versioned>::VERSION,
            self.write()?,
            &extras,
        ))
    }
}

#[test]
fn a_json_document_that_picks_its_format_writes_its_own_root() {
    let compact = |cinder: Cinder| -> String {
        let mut out = Vec::new();
        cinder.write_json(&mut out, WriteStyle::default()).unwrap();
        String::from_utf8(out).unwrap().split_whitespace().collect()
    };
    assert_eq!(
        compact(Cinder { heat: 4 }),
        r#"{"format_version":"1.0.0","demo:cinder":{"heat":4}}"#
    );
    assert_eq!(
        compact(Cinder { heat: 12 }),
        r#"{"format_version":"0.9.0","demo:cinder":{"glow":12}}"#
    );
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

/// The finding of an unreadable file, as voxel shape spells it.
fn malformed() -> Diagnostic {
    Diagnostic::error(DiagnosticCode::MalformedInput {
        message: "not UTF-8 JSON within the nesting limit".to_owned(),
        text: None,
    })
}

/// The hand loader's file step voxel shape had before `load::registry` (its `parse_file`),
/// kept as it was but for the family and the struct: the oracle the generic loader is checked
/// against.
fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<Glint>, Rejected> {
    use bedrock_addon_core::json::Prepared;
    use bedrock_addon_core::versioning::load_file;
    let origin = Origin::file(pack, side, path.clone());
    // The BOM is blanked, not cut, so every span is an offset into the file as written.
    let Ok(prepared) = Prepared::new(bytes) else {
        let mut diagnostics = Diagnostics::default();
        diagnostics.push(malformed());
        <single::GlintFamily as Family>::close(&mut diagnostics, None, path, true);
        return Err(Rejected::located(origin, diagnostics));
    };
    let document = prepared.slice();
    let outcome = load_file::<single::GlintFamily>(document, path, cx);
    let mut diagnostics = outcome.diagnostics;
    diagnostics.locate(&origin);
    match outcome.value {
        Some(Loaded::Stable(shape)) => {
            Ok(Document::new(shape, origin, outcome.version, diagnostics))
        }
        _ => Err(Rejected::new(origin, diagnostics)),
    }
}

/// The walk step of that hand loader (voxel shape's former `parse_walk_file`), as it was but
/// for the struct.
fn hand_parse(file: &WalkFile, cx: &LoadContext) -> Contribution<Glint> {
    match parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx) {
        Ok(document) => Contribution::Stable(document.description.identifier.clone(), document),
        Err(rejected) => Contribution::Rejected(None, rejected),
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

/// Files of pack 1 that fail early: no text, a malformed version, no payload.
fn early_failures() -> Vec<WalkFile> {
    let file = |path: &'static str, bytes: &'static [u8]| WalkFile {
        pack: pack(1),
        side: PackSide::Resource,
        path: PackPath::from_static(path),
        key: PackPath::from_static(path),
        bytes: bytes::Bytes::from_static(bytes),
    };
    vec![
        file("glints/demo/binary.json", &[0xff, 0xfe, 0x00]),
        file(
            "glints/demo/version.json",
            br#"{"format_version": "one", "demo:glint": {"description": {"identifier": "demo:a"}, "level": 1}}"#,
        ),
        file("glints/demo/empty.json", br#"{"format_version": "1.0.0"}"#),
    ]
}

/// Each file's contribution through the generic loader equals the hand loader's, but for one
/// difference: a rejected file names the identifier its payload gave (`Some`, when it got as
/// far as the payload) where voxel shape's loader names none. Under voxel shape's rule
/// (`LastLoadedWins`) a rejection's identifier is never read, so the registries are equal.
#[test]
fn registry_contributions_equal_the_hand_loader_but_for_a_rejected_identifier() {
    let load = LoadContext::detached();
    let mut files = walked();
    files.extend(early_failures());
    let mut named = Vec::new();
    for file in &files {
        match ((GLINTS.parse)(file, &load), hand_parse(file, &load)) {
            (Contribution::Stable(id, generic), Contribution::Stable(hand_id, hand)) => {
                assert_eq!(id, hand_id);
                assert_eq!(generic, hand, "{}", file.path);
            }
            (Contribution::Rejected(id, generic), Contribution::Rejected(None, hand)) => {
                assert_eq!(generic.origin, hand.origin);
                assert_eq!(generic.diagnostics, hand.diagnostics, "{}", file.path);
                named.push((file.path.as_str().to_owned(), id.map(|id| id.to_string())));
            }
            (generic, hand) => panic!("{}: {generic:?} against {hand:?}", file.path),
        }
    }
    assert_eq!(
        named,
        [
            (
                "glints/demo/spark.json".to_owned(),
                Some("demo:spark".to_owned())
            ),
            ("glints/demo/binary.json".to_owned(), None),
            ("glints/demo/version.json".to_owned(), None),
            ("glints/demo/empty.json".to_owned(), None),
        ],
        "the payload's identifier of a file that got as far as its payload, else none"
    );
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
    // And with the files that fail before their payload is read.
    let mut failing = files.clone();
    failing.extend(early_failures());
    assert_eq!(
        summary(&(GLINTS.combine)(
            failing.iter().map(|f| (GLINTS.parse)(f, &load)).collect(),
            &cx
        )),
        summary(&hand_combine(
            failing.iter().map(|f| hand_parse(f, &load)).collect(),
            &cx
        ))
    );
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
fn a_failed_definition_names_no_identifier_before_its_version_is_read() {
    let load = LoadContext::detached();
    let mut file = walked()[1].clone();
    file.bytes = bytes::Bytes::from_static(
        br#"{"format_version": "one", "demo:glint": {"description": {"identifier": "demo:spark"}, "level": 1}}"#,
    );
    let Contribution::Rejected(id, _) = (GLINTS.parse)(&file, &load) else {
        panic!("the version is malformed")
    };
    assert_eq!(id, None, "the identity is read after the version");
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

// ---- beta documents ------------------------------------------------------------------------

use bedrock_addon_core::versioning::{BETA_VERSION, BetaPermission};

/// The stable struct of a toy family that also has a beta struct.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Flare {
    pub description: GlintDescription,
    pub level: u8,
}

impl Versioned for Flare {
    const VERSION: FormatVersion = FormatVersion::new(1, 0, 0);
}

impl VersionStruct for Flare {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_payload(payload, cx, d)
    }
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

/// The beta struct (`"format_version": "beta"`): another shape than the stable one.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct FlareBeta {
    pub description: GlintDescription,
    pub glow: bool,
}

impl Versioned for FlareBeta {
    const VERSION: FormatVersion = BETA_VERSION;
}

impl VersionStruct for FlareBeta {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_payload(payload, cx, d)
    }
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl bedrock_addon_core::__private::Sealed for FlareBeta {}

/// The beta definition, read from the same files as the stable one.
impl Definition for FlareBeta {
    type Kind = kind::Block;
    type File = Flare;
    type Beta = NoBeta<kind::Block>;
}

pub struct FlareFamily;

bedrock_addon_core::version_chain! {
    family = FlareFamily;
    #[derive(Clone, Debug, PartialEq)]
    pub enum FlareAny {
        V1_0_0(Flare),
    }
    beta = Beta(FlareBeta);
    molang = none;
}

impl Family for FlareFamily {
    const NAME: &'static str = "flare";
    const PAYLOAD_KEY: &'static str = "demo:flare";
    const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
    // Beta documents are allowed in every pack, so the toy needs no beta-enabled context.
    const BETA_PERMISSION: BetaPermission = BetaPermission::Always;
    type Latest = Flare;
    type Beta = FlareBeta;
    type Component = NoComponents;
    type Any = FlareAny;

    /// Records each close and whether the file failed, so a test sees how it was closed.
    fn close(
        diagnostics: &mut Diagnostics,
        _: Option<JsonSlice<'_>>,
        path: &PackPath,
        failed: bool,
    ) {
        diagnostics.push(Diagnostic::info(DiagnosticCode::DocumentRejected {
            text: Some(VanillaText::Formatted {
                template: "flare {} closed, failed: {}",
                args: vec![path.as_str().to_owned(), failed.to_string()],
            }),
        }));
    }
}

impl JsonDocument for Flare {
    type Family = FlareFamily;
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    /// The generic read, then an identifier check of the family's own (as `feature_rules`
    /// has): a stable flare in the `minecraft` namespace outside a base-game pack is rejected.
    fn read(
        origin: Origin,
        bytes: &[u8],
        cx: &LoadContext,
    ) -> bedrock_addon_core::versioning::DocumentRead<FlareFamily> {
        use bedrock_addon_core::versioning::{DocumentRead, parse_document};
        match parse_document::<FlareFamily>(origin, bytes, cx) {
            DocumentRead::Stable(flare)
                if flare
                    .description
                    .identifier
                    .as_str()
                    .starts_with("minecraft:")
                    && !cx.is_base_game_pack =>
            {
                let mut diagnostics = flare.diagnostics().clone();
                diagnostics.push(Diagnostic::error(DiagnosticCode::NamespaceRule {
                    text: None,
                }));
                DocumentRead::Rejected(
                    Some(flare.description.identifier.to_string()),
                    Rejected::located(flare.origin().clone(), diagnostics),
                )
            }
            read => read,
        }
    }
}

impl JsonDefinition for Flare {
    type Kind = kind::Block;
    fn identifier(&self) -> &BlockId {
        &self.description.identifier
    }
}

const BETA_FLARE: &str = r#"{"format_version": "beta", "demo:flare": {"description": {"identifier": "demo:sun"}, "glow": true}}"#;

fn flare_path() -> PackPath {
    PackPath::from_static("flares/demo/sun.json")
}

/// The closes `FlareFamily::close` recorded, each as `failed: <bool>`.
fn closes(diagnostics: &Diagnostics) -> Vec<bool> {
    diagnostics
        .iter()
        .filter_map(|d| match &d.code {
            DiagnosticCode::DocumentRejected {
                text: Some(VanillaText::Formatted { template, args }),
            } if template.starts_with("flare") => Some(args[1] == "true"),
            _ => None,
        })
        .collect()
}

#[test]
fn a_stable_json_document_refuses_a_beta_file_and_closes_it_failed() {
    let rejected = <Flare as Content>::parse(
        &flare_path(),
        BETA_FLARE.as_bytes(),
        &LoadContext::detached(),
    )
    .expect_err("the stable type reads no beta document");
    assert!(
        rejected.diagnostics.iter().any(|d| matches!(
            &d.code,
            DiagnosticCode::UnsupportedVersion {
                text: Some(VanillaText::NoParserForVersion { version })
            } if version == "beta"
        )),
        "the beta refusal entity's own parse pushes: {:?}",
        rejected.diagnostics
    );
    assert_eq!(
        closes(&rejected.diagnostics),
        [true],
        "closed once, as failed"
    );
    for finding in &rejected.diagnostics {
        assert_eq!(finding.location.path, Some(flare_path()));
    }
}

#[test]
fn a_registry_takes_a_beta_file_as_a_beta_contribution_closed_as_loaded() {
    const FLARES: CollectionLoader<Registry<Flare>, Contribution<Flare>> =
        bedrock_addon_core::load::registry::<Flare>(Walk::json_tree_in(&["flares"]));
    let file = WalkFile {
        pack: pack(1),
        side: PackSide::Behavior,
        path: flare_path(),
        key: flare_path(),
        bytes: bytes::Bytes::from_static(BETA_FLARE.as_bytes()),
    };
    let contribution = (FLARES.parse)(&file, &LoadContext::detached());
    let Contribution::Beta(id, document) = contribution else {
        panic!("a beta contribution: {contribution:?}")
    };
    assert_eq!(id, BlockId::parse("demo:sun"));
    assert!(document.glow);
    assert_eq!(
        *document.origin(),
        Origin::file(pack(1), PackSide::Behavior, flare_path())
    );
    assert_eq!(
        closes(document.diagnostics()),
        [false],
        "closed once, as loaded"
    );
}

/// A beta document a family built its own way, opened with `OpenBeta::new`: kept, it is
/// closed once as loaded; refused, once as failed with the beta refusal.
#[test]
fn a_beta_document_a_family_built_opens_and_closes_once() {
    use bedrock_addon_core::document::VersionInfo;
    use bedrock_addon_core::versioning::{BETA_VERSION, OpenBeta};
    let beta = || {
        let document = Document::new(
            FlareBeta {
                description: GlintDescription {
                    identifier: BlockId::parse("demo:sun"),
                },
                glow: true,
            },
            Origin::file(pack(1), PackSide::Behavior, flare_path()),
            VersionInfo::new(None, true, BETA_VERSION, BETA_VERSION, false),
            Diagnostics::default(),
        );
        OpenBeta::<FlareFamily>::new(document)
    };
    let kept = beta().keep();
    assert!(kept.glow);
    assert_eq!(
        closes(kept.diagnostics()),
        [false],
        "closed once, as loaded"
    );
    let refused = beta().refuse();
    assert_eq!(
        closes(&refused.diagnostics),
        [true],
        "closed once, as failed"
    );
    assert!(refused.diagnostics.iter().any(|d| matches!(
        &d.code,
        DiagnosticCode::UnsupportedVersion {
            text: Some(VanillaText::NoParserForVersion { version })
        } if version == "beta"
    )));
    for finding in &refused.diagnostics {
        assert_eq!(finding.location.path, Some(flare_path()));
    }
}

const VANILLA_FLARE: &str = r#"{"format_version": "1.0.0", "demo:flare": {"description": {"identifier": "minecraft:sun"}, "level": 1}}"#;

fn is_namespace_rule(d: &Diagnostic) -> bool {
    matches!(d.code, DiagnosticCode::NamespaceRule { .. })
}

#[test]
fn an_overridden_read_is_what_content_parse_reads() {
    let cx = LoadContext::detached();
    let rejected = <Flare as Content>::parse(&flare_path(), VANILLA_FLARE.as_bytes(), &cx)
        .expect_err("the family's identifier check");
    let found: Vec<_> = rejected
        .diagnostics
        .iter()
        .filter(|d| is_namespace_rule(d))
        .collect();
    assert_eq!(found.len(), 1, "{:?}", rejected.diagnostics);
    assert_eq!(found[0].location.path, Some(flare_path()));
    // A flare of another namespace passes the check.
    let sun = BETA_FLARE
        .replace("beta", "1.0.0")
        .replace(r#""glow": true"#, r#""level": 2"#);
    let flare = <Flare as Content>::parse(&flare_path(), sun.as_bytes(), &cx).expect("loads");
    assert_eq!(flare.level, 2);
}

#[test]
fn an_overridden_read_is_what_a_registry_reads() {
    const FLARES: CollectionLoader<Registry<Flare>, Contribution<Flare>> =
        bedrock_addon_core::load::registry::<Flare>(Walk::json_tree_in(&["flares"]));
    let file = WalkFile {
        pack: pack(2),
        side: PackSide::Behavior,
        path: flare_path(),
        key: flare_path(),
        bytes: bytes::Bytes::from_static(VANILLA_FLARE.as_bytes()),
    };
    let contribution = (FLARES.parse)(&file, &LoadContext::detached());
    let Contribution::Rejected(id, rejected) = contribution else {
        panic!("the family's identifier check rejects it: {contribution:?}")
    };
    assert_eq!(id, Some(BlockId::parse("minecraft:sun")));
    assert_eq!(
        rejected.origin,
        Origin::file(pack(2), PackSide::Behavior, flare_path())
    );
    assert!(
        rejected.diagnostics.iter().any(is_namespace_rule),
        "{:?}",
        rejected.diagnostics
    );
}

// ---- files and merged over a JSON document -------------------------------------------------

const EMBERS: CollectionLoader<Files<Ember>, FileCopy<Ember>> =
    bedrock_addon_core::load::files::<Ember>(Walk::exact_files(&["embers.json"]));

/// The heat of every ember that loaded, and how many copies failed.
fn sum_heat(parsed: Vec<FileCopy<Ember>>, _: &StackCx<'_>) -> (u32, usize) {
    let failed = parsed.iter().filter(|(_, copy)| copy.is_err()).count();
    let heat = parsed
        .into_iter()
        .filter_map(|(_, copy)| copy.ok())
        .map(|document| u32::from(document.heat))
        .sum();
    (heat, failed)
}

const ALL_EMBERS: CollectionLoader<(u32, usize), FileCopy<Ember>> =
    bedrock_addon_core::load::merged::<Ember, _>(Walk::exact_files(&["embers.json"]), sum_heat);

/// Pack 1's ember at the root, and pack 0's in a subpack, on the resource side (`Ember` is
/// read on its own as a behaviour file), whose heat is no `u8`.
fn embers() -> [WalkFile; 2] {
    let file = |index: usize, path: &'static str, text: &'static str| WalkFile {
        pack: pack(index),
        side: PackSide::Resource,
        path: PackPath::from_static(path),
        key: PackPath::from_static("embers.json"),
        bytes: bytes::Bytes::from_static(text.as_bytes()),
    };
    [
        file(
            1,
            "embers.json",
            r#"{"format_version": "1.2.0", "demo:ember": {"heat": 3}}"#,
        ),
        file(
            0,
            "subpacks/hd/embers.json",
            r#"{"format_version": "1.2.0", "demo:ember": {"heat": "hot"}}"#,
        ),
    ]
}

/// The failing copy is rejected in the walked file: its pack, its side and its subpack path,
/// on the rejection and on every finding.
fn assert_rejected_in_its_subpack(copy: &FileCopy<Ember>) {
    let (key, copy) = copy;
    assert_eq!(
        key.as_str(),
        "embers.json",
        "keyed without the subpack folder"
    );
    let rejected = copy.as_ref().expect_err("heat is no u8");
    let walked = Origin::file(
        pack(0),
        PackSide::Resource,
        PackPath::from_static("subpacks/hd/embers.json"),
    );
    assert_eq!(rejected.origin, walked);
    assert!(!rejected.diagnostics.is_empty());
    for finding in &rejected.diagnostics {
        assert_eq!(finding.location.pack, Some(pack(0)), "{finding:?}");
        assert_eq!(
            finding.location.path,
            Some(walked.path.clone()),
            "{finding:?}"
        );
    }
}

#[test]
fn files_puts_a_failed_json_document_in_its_subpack_file() {
    let options = LoadOptions::default();
    let packs = stack();
    let cx = StackCx::new(&options, &packs);
    let load = LoadContext::detached();
    let parsed: Vec<_> = embers().iter().map(|f| (EMBERS.parse)(f, &load)).collect();
    assert_rejected_in_its_subpack(&parsed[1]);
    let loaded = parsed[0].1.as_ref().expect("loads");
    assert_eq!(loaded.origin().pack, pack(1));
    assert_eq!(loaded.origin().side, PackSide::Resource, "the walked side");
    let files = (EMBERS.combine)(parsed, &cx);
    assert!(
        files
            .rejected()
            .iter()
            .any(|r| r.origin.path.as_str() == "subpacks/hd/embers.json"),
        "the rejection is kept in its file"
    );
}

#[test]
fn merged_puts_a_failed_json_document_in_its_subpack_file() {
    let options = LoadOptions::default();
    let packs = stack();
    let cx = StackCx::new(&options, &packs);
    let load = LoadContext::detached();
    let parsed: Vec<_> = embers()
        .iter()
        .map(|f| (ALL_EMBERS.parse)(f, &load))
        .collect();
    assert_rejected_in_its_subpack(&parsed[1]);
    assert_eq!((ALL_EMBERS.combine)(parsed, &cx), (3, 1));
}

// ---- impl_no_molang! -----------------------------------------------------------------------

#[test]
fn impl_no_molang_states_a_struct_without_molang_fields() {
    use bedrock_addon_core::molang::count_sources;
    let mut ember = Ember {
        heat: 1,
        read_at: None,
    };
    assert_eq!(count_sources(&mut ember), 0);
    let read = <Ember as Content>::parse(
        &PackPath::from_static("embers.json"),
        br#"{"format_version": "1.2.0", "demo:ember": {"heat": 1}}"#,
        &LoadContext::detached(),
    )
    .expect("loads");
    assert!(
        !read
            .diagnostics()
            .iter()
            .any(|d| matches!(d.code, DiagnosticCode::MolangNotChecked { .. })),
        "nothing to check"
    );
}

// ---- Single and Molang ---------------------------------------------------------------------

/// A one-version struct with a Molang field.
#[derive(Clone, Debug, PartialEq, Facet)]
pub struct Beam {
    pub condition: bedrock_addon_core::molang::Molang,
}

impl Versioned for Beam {
    const VERSION: FormatVersion = FormatVersion::new(1, 0, 0);
}

impl VersionStruct for Beam {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        parse_payload(payload, cx, d)
    }
    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl bedrock_addon_core::molang::MolangFields for Beam {
    fn visit_molang(&mut self, visitor: &mut bedrock_addon_core::molang::MolangVisitor<'_>) {
        visitor.field(
            "condition",
            &bedrock_addon_core::molang::MolangFieldSpec::DEFAULT,
            &mut self.condition,
        );
    }
}

/// The beam family twice: by `version_chain!` and on `Single`.
mod beam {
    use super::*;

    pub struct ByMacro;

    bedrock_addon_core::version_chain! {
        family = ByMacro;
        pub enum Any {
            V1_0_0(Beam),
        }
    }

    pub struct OnSingle;

    macro_rules! family {
        ($family:ty, $any:ty) => {
            impl Family for $family {
                const NAME: &'static str = "beam";
                const PAYLOAD_KEY: &'static str = "demo:beam";
                const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
                type Latest = Beam;
                type Beta = NoBeta<kind::Block>;
                type Component = NoComponents;
                type Any = $any;
            }
        };
    }

    family!(ByMacro, Any);
    family!(OnSingle, bedrock_addon_core::versioning::Single<Self>);
}

#[test]
fn single_walks_its_struct_for_molang_pass_two_as_a_chain_of_one_does() {
    use bedrock_addon_core::versioning::{Single, VersionChain};
    let text = r#"{"format_version": "1.0.0", "demo:beam": {"condition": "q.is_baby"}}"#;
    let cx = LoadContext::detached();
    let single = load::<beam::OnSingle>(JsonSlice::new(text), &cx);
    let generated = load::<beam::ByMacro>(JsonSlice::new(text), &cx);
    assert!(
        single
            .diagnostics
            .iter()
            .any(|d| matches!(d.code, DiagnosticCode::MolangNotChecked { .. })),
        "pass 2 reached the condition: {:?}",
        single.diagnostics
    );
    assert_eq!(single.diagnostics, generated.diagnostics);
    // The walk itself: the struct, once.
    let Some(mut loaded) = single.value else {
        panic!("loads")
    };
    let mut visits = 0;
    let mut sources = 0;
    <Single<beam::OnSingle> as VersionChain<beam::OnSingle>>::with_molang(&mut loaded, &mut |f| {
        visits += 1;
        sources += bedrock_addon_core::molang::count_sources(f);
    });
    assert_eq!((visits, sources), (1, 1));
}
