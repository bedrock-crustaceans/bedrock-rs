//! `Family::close`, the post-parse hook every family may implement, a family with a beta
//! struct included, and the closed entry points that run it once per file.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Code, Diagnostic, Diagnostics, Kind, Location, Phase, Severity,
};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::versioning::{
    BelowFloor, ComponentTable, Envelope, Family, finish_file, load, load_file, parse_as_written,
    parse_file_as_written,
};

use crate::toy::{Toy120, ToyBeta, ToyComponent, v1_0_0};

/// A family with a real beta struct that closes a failed load with a line of its own.
pub struct Closing;

pub mod closing {
    use super::*;
    bedrock_addon_core::version_chain! {
        family = Closing;
        /// The toy's chain for the closing family.
        pub enum History {
            V1_0_0(v1_0_0::Toy),
            V1_1_0(crate::toy::v1_1_0::Toy),
            V1_2_0(Toy120),
        }
        beta = Beta(ToyBeta);
    }
}

const CLOSING_LINE: &str = "Failed to load toy";

/// The closing line's code.
fn closing_code() -> Code {
    core_codes::DOCUMENT_REJECTED_WORDED.with([CLOSING_LINE])
}

impl Family for Closing {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
    type Latest = Toy120;
    type Beta = ToyBeta;
    type Component = ToyComponent;
    type History = closing::History;
    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }

    fn close(diagnostics: &mut Diagnostics, _: Option<JsonSlice<'_>>, _: &PackPath, failed: bool) {
        if failed {
            diagnostics.push(Diagnostic::new(
                core_codes::DOCUMENT_REJECTED_WORDED.with([CLOSING_LINE]),
                Severity::Error,
                Phase::Parse,
                Location::default(),
            ));
        }
    }
}

fn closed<F: Family>(failed: bool) -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    F::close(
        &mut diagnostics,
        None,
        &PackPath::from_static("toys/a.json"),
        failed,
    );
    diagnostics
}

#[test]
fn a_family_with_a_beta_struct_closes_a_failed_load() {
    let failed = closed::<Closing>(true);
    assert_eq!(failed.len(), 1, "the failed load gets the closing line");
    assert_eq!(failed.iter().next().map(|d| &d.code), Some(&closing_code()));
    assert_eq!(
        closed::<Closing>(false).len(),
        0,
        "a loaded file is left alone"
    );
}

#[test]
fn close_is_a_no_op_by_default() {
    assert_eq!(closed::<crate::toy::Toy>(true).len(), 0);
}

/// A family below whose floor the game's loader has no parser at all.
pub struct Floored;

pub mod floored {
    use super::*;
    bedrock_addon_core::version_chain! {
        family = Floored;
        /// The toy's chain for the floored family.
        pub enum History {
            V1_0_0(v1_0_0::Toy),
            V1_1_0(crate::toy::v1_1_0::Toy),
            V1_2_0(Toy120),
        }
        beta = Beta(ToyBeta);
    }
}

impl Family for Floored {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
    const ENVELOPE: Envelope = Envelope {
        below_floor: BelowFloor::NoParser,
        ..Envelope::DEFAULT
    };
    type Latest = Toy120;
    type Beta = ToyBeta;
    type Component = ToyComponent;
    type History = floored::History;
    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }
}

fn toy_at(version: &str) -> String {
    format!(
        r#"{{"format_version": {version}, "minecraft:toy": {{"description": {{"identifier": "demo:x"}}}}}}"#
    )
}

/// The `UnsupportedVersion` codes of a load.
fn unsupported<F: Family>(text: &str) -> Vec<Code> {
    load::<F>(JsonSlice::new(text), &LoadContext::detached())
        .diagnostics
        .iter()
        // By kind: every definition of an unsupported version.
        .filter(|d| d.code.kind() == Kind::UnsupportedVersion)
        .map(|d| d.code.clone())
        .collect()
}

/// `BelowFloor::NoParser`: the pipeline raises "no parser available" itself, the version
/// rendered as the text above the engine renders it (`no_parser_version`): the full triple,
/// a pre-release as written.
#[test]
fn below_the_floor_of_a_no_parser_family_is_no_parser_with_the_version_as_parsed() {
    for (written, shown) in [
        (r#""0.9.0""#, "0.9.0"),
        (r#""0.9""#, "0.9.0"),
        ("[0, 9, 0]", "0.9.0"),
        (r#""1.0.0-beta.1""#, "1.0.0-beta.1"),
    ] {
        assert_eq!(
            unsupported::<Floored>(&toy_at(written)),
            [core_codes::NO_PARSER_FOR_VERSION.with([shown])],
            "{written}"
        );
    }
}

#[test]
fn below_the_floor_of_a_default_family_is_the_floor_line() {
    let texts = unsupported::<crate::toy::Toy>(&toy_at(r#""0.9.0""#));
    assert!(
        matches!(texts.as_slice(), [code] if code.is(&core_codes::USE_FORMAT_VERSION)),
        "{texts:?}"
    );
}

fn closing_lines(diagnostics: &Diagnostics) -> usize {
    diagnostics
        .iter()
        .filter(|d| d.code == closing_code())
        .count()
}

fn path() -> PackPath {
    PackPath::from_static("toys/a.json")
}

/// A file's load closes once, at its end: the closing line for a failed file, nothing for one
/// that loads.
#[test]
fn a_file_s_load_closes_once_with_whether_it_failed() {
    let cx = LoadContext::detached();
    let failed = load_file::<Closing>(JsonSlice::new(&toy_at(r#""0.9.0""#)), &path(), &cx);
    assert!(failed.value.is_none());
    assert_eq!(
        closing_lines(&failed.diagnostics),
        1,
        "{:?}",
        failed.diagnostics
    );
    let loaded = load_file::<Closing>(JsonSlice::new(&toy_at(r#""1.2.0""#)), &path(), &cx);
    assert!(loaded.value.is_some(), "{:?}", loaded.diagnostics);
    assert_eq!(closing_lines(&loaded.diagnostics), 0);
    // The unclosed stages do not close.
    let unclosed = load::<Closing>(JsonSlice::new(&toy_at(r#""0.9.0""#)), &cx);
    assert_eq!(closing_lines(&unclosed.diagnostics), 0);
}

/// A parse that fails ends the file: its stage closes it; the unclosed stage and the rest of a
/// load each close what they are given once.
#[test]
fn a_file_s_stages_close_once_each() {
    let cx = LoadContext::detached();
    let text = toy_at(r#""0.9.0""#);
    let document = JsonSlice::new(&text);
    let parsed = parse_file_as_written::<Closing>(document, &path(), &cx);
    assert_eq!(
        closing_lines(&parsed.diagnostics),
        1,
        "{:?}",
        parsed.diagnostics
    );
    let unclosed = parse_as_written::<Closing>(document, &cx);
    assert_eq!(closing_lines(&unclosed.diagnostics), 0);
    let finished = finish_file::<Closing>(unclosed, Some(document), &path(), &cx);
    assert_eq!(
        closing_lines(&finished.diagnostics),
        1,
        "{:?}",
        finished.diagnostics
    );
}

/// A family whose close pushes a line on every file, loaded or not.
pub struct Always;

pub mod always {
    use super::*;
    bedrock_addon_core::version_chain! {
        family = Always;
        /// The toy's chain for the always-closing family.
        pub enum History {
            V1_0_0(v1_0_0::Toy),
            V1_1_0(crate::toy::v1_1_0::Toy),
            V1_2_0(Toy120),
        }
        beta = Beta(ToyBeta);
    }
}

impl Family for Always {
    const NAME: &'static str = "toy";
    const PAYLOAD_KEY: &'static str = "minecraft:toy";
    const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
    type Latest = Toy120;
    type Beta = ToyBeta;
    type Component = ToyComponent;
    type History = always::History;
    fn components() -> &'static ComponentTable<Self> {
        &ComponentTable::EMPTY
    }

    fn close(diagnostics: &mut Diagnostics, _: Option<JsonSlice<'_>>, _: &PackPath, _: bool) {
        diagnostics.push(Diagnostic::new(
            core_codes::DOCUMENT_REJECTED_WORDED.with([CLOSING_LINE]),
            Severity::Info,
            Phase::Parse,
            Location::default(),
        ));
    }
}

/// A file loaded in two stages, the parse stage then the rest, is closed once: by the rest when
/// the parse gave a value, by the parse stage when the file ends there.
#[test]
fn a_file_loaded_in_two_stages_closes_once() {
    let cx = LoadContext::detached();
    let loaded = toy_at(r#""1.2.0""#);
    let document = JsonSlice::new(&loaded);
    let parsed = parse_file_as_written::<Always>(document, &path(), &cx);
    assert!(parsed.value.is_some(), "{:?}", parsed.diagnostics);
    assert_eq!(closing_lines(&parsed.diagnostics), 0, "the rest closes");
    let finished = finish_file::<Always>(parsed, Some(document), &path(), &cx);
    assert_eq!(
        closing_lines(&finished.diagnostics),
        1,
        "{:?}",
        finished.diagnostics
    );
    // In one call, the same.
    let whole = load_file::<Always>(document, &path(), &cx);
    assert_eq!(closing_lines(&whole.diagnostics), 1);
    // A parse that gives no value ends the file: its stage closes.
    let refused = toy_at(r#""0.9.0""#);
    let parsed = parse_file_as_written::<Always>(JsonSlice::new(&refused), &path(), &cx);
    assert!(parsed.value.is_none());
    assert_eq!(closing_lines(&parsed.diagnostics), 1);
}
