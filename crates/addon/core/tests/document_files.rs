//! `Files<T>`: the top pack's copy of a path wins; under `TopWinsFallback`
//! (structures) a failing copy falls through to the next pack.

use bedrock_addon_core::__private::Sealed;
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, codes,
};
use bedrock_addon_core::document::{
    Content, Document, Files, FilesBuilder, FilesRule, Notice, Origin, Rejected, VersionInfo,
};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{LoadContext, LoadOptions};
use bedrock_addon_core::pack_identity::{PackIdVersion, PackPath, PackRef, PackSide, PackType};
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use uuid::Uuid;

/// A toy whole-file kind: which pack's copy it is.
#[derive(Clone, Debug, PartialEq)]
struct Table(&'static str);

impl Sealed for Table {}

impl Content for Table {
    const NAME: &'static str = "table";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(_: &PackPath, _: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        unimplemented!("the tests build documents by hand")
    }

    fn write_json(&self, _: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
        Ok(())
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        Ok(String::new())
    }
}

fn pack(index: usize) -> PackRef {
    PackRef::new(
        index,
        PackIdVersion::new(
            Uuid::from_u128(index as u128 + 1),
            SemVersion::new(1, 0, 0),
            PackType::Behavior,
        ),
        format!("pack {index}"),
    )
}

fn origin(pack_index: usize, path: &'static str) -> Origin {
    Origin::new(
        pack(pack_index),
        PackSide::Behavior,
        PackPath::from_static(path),
        None,
        Vec::new(),
    )
}

fn copy(pack_index: usize, path: &'static str, from: &'static str) -> Document<Table> {
    let v = FormatVersion::new(1, 0, 0);
    Document::new(
        Table(from),
        origin(pack_index, path),
        VersionInfo::new(None, false, v, v, false),
        Diagnostics::default(),
    )
}

fn failed(pack_index: usize, path: &'static str) -> Result<Document<Table>, Rejected> {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Diagnostic::new(
        DiagnosticCode::from(&codes::INVALID_TYPE),
        Severity::Error,
        Phase::Parse,
        Location::new(Some(pack(pack_index)), None, JsonPath::new(), None),
    ));
    Err(Rejected::new(origin(pack_index, path), diagnostics))
}

fn path(text: &'static str) -> PackPath {
    PackPath::from_static(text)
}

fn build(
    rule: FilesRule,
    options: &LoadOptions,
    walk: Vec<(&'static str, Result<Document<Table>, Rejected>)>,
) -> Files<Table> {
    let mut builder = FilesBuilder::new(rule, options);
    for (key, copy) in walk {
        builder.push(path(key), copy);
    }
    builder.finish()
}

fn keep_shadowed() -> LoadOptions {
    LoadOptions {
        keep_shadowed: true,
        ..LoadOptions::default()
    }
}

const LOOT: &str = "loot_tables/chest.json";

#[test]
fn the_top_copy_wins_whatever_the_walk_order() {
    for walk in [
        vec![
            (LOOT, Ok(copy(1, LOOT, "lower"))),
            (LOOT, Ok(copy(0, LOOT, "top"))),
        ],
        vec![
            (LOOT, Ok(copy(0, LOOT, "top"))),
            (LOOT, Ok(copy(1, LOOT, "lower"))),
        ],
    ] {
        let files = build(FilesRule::TopWins, &keep_shadowed(), walk);
        assert_eq!(files.get(&path(LOOT)).map(|d| d.0), Some("top"));
        let layers: Vec<&str> = files.layers(&path(LOOT)).map(|d| d.0).collect();
        assert_eq!(layers, ["top", "lower"]);
    }
}

#[test]
fn layers_without_keep_shadowed_is_the_effective_copy_only() {
    let files = build(
        FilesRule::TopWins,
        &LoadOptions::default(),
        vec![
            (LOOT, Ok(copy(1, LOOT, "lower"))),
            (LOOT, Ok(copy(0, LOOT, "top"))),
        ],
    );
    let layers: Vec<&str> = files.layers(&path(LOOT)).map(|d| d.0).collect();
    assert_eq!(layers, ["top"]);
    assert_eq!(files.layers(&path("loot_tables/none.json")).count(), 0);
}

#[test]
fn under_top_wins_a_failing_top_copy_claims_the_path() {
    let files = build(
        FilesRule::TopWins,
        &keep_shadowed(),
        vec![(LOOT, Ok(copy(1, LOOT, "lower"))), (LOOT, failed(0, LOOT))],
    );
    assert!(files.get(&path(LOOT)).is_none());
    assert_eq!(files.iter().count(), 0);
    assert_eq!(files.rejected().len(), 1);
    assert_eq!(files.rejected()[0].origin.pack, pack(0));
    let layers: Vec<&str> = files.layers(&path(LOOT)).map(|d| d.0).collect();
    assert_eq!(layers, ["lower"], "listed, not effective");
}

const STRUCTURE: &str = "structures/house.mcstructure";

#[test]
fn under_top_wins_fallback_a_failing_copy_falls_through() {
    let files = build(
        FilesRule::TopWinsFallback {
            fallback: Some(Notice {
                code: DiagnosticCode::from(&codes::SHADOWED_DEFINITION),
                severity: Severity::Info,
            }),
        },
        &LoadOptions::default(),
        vec![
            (STRUCTURE, Ok(copy(2, STRUCTURE, "bottom"))),
            (STRUCTURE, Ok(copy(1, STRUCTURE, "middle"))),
            (STRUCTURE, failed(0, STRUCTURE)),
        ],
    );
    assert_eq!(files.get(&path(STRUCTURE)).map(|d| d.0), Some("middle"));
    let codes: Vec<(DiagnosticCode, Option<usize>)> = files
        .diagnostics()
        .map(|d| {
            (
                d.code.clone(),
                d.location.pack.as_ref().and_then(PackRef::index),
            )
        })
        .collect();
    assert_eq!(
        codes,
        [
            (DiagnosticCode::from(&codes::INVALID_TYPE), Some(0)),
            (DiagnosticCode::from(&codes::SHADOWED_DEFINITION), Some(0)),
        ]
    );
}

#[test]
fn iteration_is_by_first_copy_of_each_path() {
    let files = build(
        FilesRule::TopWins,
        &LoadOptions::default(),
        vec![
            ("b.json", Ok(copy(1, "b.json", "1b"))),
            ("a.json", Ok(copy(1, "a.json", "1a"))),
            ("c.json", Ok(copy(0, "c.json", "0c"))),
            ("b.json", Ok(copy(0, "b.json", "0b"))),
        ],
    );
    let order: Vec<(&str, &str)> = files.iter().map(|(p, d)| (p.as_str(), d.0)).collect();
    assert_eq!(
        order,
        [("b.json", "0b"), ("a.json", "1a"), ("c.json", "0c")]
    );
    assert_eq!((&files).into_iter().count(), 3);
}

#[test]
fn the_key_is_the_path_the_builder_is_given_not_the_subpack_path() {
    // A subpack copy's origin keeps its folder; it overrides the same content path.
    let files = build(
        FilesRule::TopWins,
        &LoadOptions::default(),
        vec![
            (LOOT, Ok(copy(1, LOOT, "base"))),
            (
                LOOT,
                Ok(copy(0, "subpacks/high/loot_tables/chest.json", "subpack")),
            ),
        ],
    );
    let effective = files.get(&path(LOOT)).unwrap();
    assert_eq!(effective.0, "subpack");
    assert_eq!(
        effective.origin().path.as_str(),
        "subpacks/high/loot_tables/chest.json"
    );
}

// ---- all_layers ----

/// Every path `all_layers` lists, with which pack's copy each layer is.
fn all_layers(files: &Files<Table>) -> Vec<(&str, Vec<&'static str>)> {
    files
        .all_layers()
        .map(|(path, layers)| (path.as_str(), layers.map(|copy| copy.0).collect()))
        .collect()
}

#[test]
fn all_layers_lists_every_path_with_its_copies() {
    let walk = || {
        vec![
            (LOOT, Ok(copy(1, LOOT, "lower"))),
            (STRUCTURE, Ok(copy(1, STRUCTURE, "house"))),
            (LOOT, Ok(copy(0, LOOT, "top"))),
        ]
    };
    let files = build(FilesRule::TopWins, &keep_shadowed(), walk());
    assert_eq!(
        all_layers(&files),
        [(LOOT, vec!["top", "lower"]), (STRUCTURE, vec!["house"])]
    );
    // The layers of a path are a named iterator that knows its length, as a registry's.
    let lengths: Vec<usize> = files
        .all_layers()
        .map(|(_, layers)| layers.clone().len())
        .collect();
    assert_eq!(lengths, [2, 1]);
    let files = build(FilesRule::TopWins, &LoadOptions::default(), walk());
    assert_eq!(
        all_layers(&files),
        [(LOOT, vec!["top"]), (STRUCTURE, vec!["house"])]
    );
}

#[test]
fn all_layers_lists_a_path_a_failed_copy_claimed() {
    let walk = || vec![(LOOT, Ok(copy(1, LOOT, "lower"))), (LOOT, failed(0, LOOT))];
    let files = build(FilesRule::TopWins, &keep_shadowed(), walk());
    assert!(files.get(&path(LOOT)).is_none());
    assert_eq!(files.iter().count(), 0);
    assert_eq!(
        all_layers(&files),
        [(LOOT, vec!["lower"])],
        "listed with its kept copy, though `iter` has no entry for it"
    );
    // Nothing is kept of it without `keep_shadowed`: not listed.
    let files = build(FilesRule::TopWins, &LoadOptions::default(), walk());
    assert_eq!(all_layers(&files), Vec::new());
}

#[test]
fn files_layers_is_a_named_iterator_and_files_counts_like_a_registry() {
    let files = build(
        FilesRule::TopWins,
        &keep_shadowed(),
        vec![
            (LOOT, Ok(copy(1, LOOT, "lower"))),
            (LOOT, Ok(copy(0, LOOT, "top"))),
        ],
    );
    let layers: bedrock_addon_core::document::files::Layers<'_, Table> = files.layers(&path(LOOT));
    assert_eq!(layers.len(), 2);
    assert_eq!(files.len(), 1);
    assert!(!files.is_empty());
    assert!(files.contains(&path(LOOT)));
    assert!(!files.contains(&path("loot_tables/none.json")));
    let empty = build(FilesRule::TopWins, &LoadOptions::default(), Vec::new());
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
}

#[test]
fn files_iterators_and_builder_are_debug_without_a_content_bound() {
    struct Opaque;
    impl Sealed for Opaque {}
    impl Content for Opaque {
        const NAME: &'static str = "opaque";
        const SIDES: &'static [PackSide] = &[PackSide::Behavior];
        fn parse(_: &PackPath, _: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
            unimplemented!()
        }
        fn write_json(&self, _: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
            Ok(())
        }
        fn to_json(&self) -> Result<String, EncodeError> {
            Ok(String::new())
        }
    }
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<FilesBuilder<Opaque>>();
    assert_debug::<bedrock_addon_core::document::files::Iter<'_, Opaque>>();
    assert_debug::<bedrock_addon_core::document::files::Layers<'_, Opaque>>();
}
