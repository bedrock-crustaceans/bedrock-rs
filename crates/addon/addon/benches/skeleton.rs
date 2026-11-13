//! What the family skeleton's loader kinds cost, one group per kind of
//! `bedrock_addon_core::load` (`registry`, `files`, `merged`, and the hand-written
//! `CollectionLoader` literal), each over one real-shaped document set built from inline JSON.
//! Nothing is read from disk, the network or a corpus; a bench is not a test and asserts
//! nothing about time.
//!
//! - `registry` is `fog`: 2 resource packs of 200 `fogs/*.json` each, 100 identifiers in both,
//!   so the definitions' combine rule runs. Read through `PackStack::fogs`.
//! - `files` is `loot`: 2 behaviour packs of 200 `loot_tables/**/*.json` each, 100 paths in
//!   both, the top pack winning. Read through `PackStack::loot_tables`.
//! - `merged` is `splashes`: 3 resource packs of one `splashes.json` of 2 000 entries each,
//!   merged into one pool. Read through `PackStack::splashes`.
//! - `hand` is `lang`, a hand-written loader (it carries pack keywords beside each table,
//!   which `load::merged`'s copy cannot): 2 resource packs of one `texts/en_US.lang` of 2 500
//!   lines each. Read through `PackStack::translations`.
//!
//! Every `read` function opens a fresh `PackStack` over the packs built before the timer (a
//! `Pack` is an `Arc`, so cloning one is cheap) and asks for the one collection: the walk, the
//! parse of each file and the combine are what is timed, the stack's own cache is not reused.
//! Where the family writes back, the `write` function puts the loaded documents into a
//! `PackWriter` and lists its files; the stack is loaded before the timer. The lang write goes
//! through `LangFile`, because the stack keeps the merged table and not the file.
//!
//! Run one group at a time, with only the families it needs (the default features build every
//! family):
//!
//! ```text
//! cargo bench -p bedrock_addon --bench skeleton --no-default-features \
//!     --features fog,loot,splashes,lang -- registry
//! ```

use std::fmt::Write as _;
use std::hint::black_box;

use bedrock_addon::families::lang::{LangFile, Language};
use bedrock_addon::json::WriteStyle;
use bedrock_addon::load::{LoadContext, LoadOptions};
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackStack, PackWriter, Resource};
use bedrock_addon::storage::PackPath;
use bedrock_addon::{document::Content, manifest::Manifest};
use bytes::Bytes;
use criterion::{Criterion, criterion_group, criterion_main};

const FOGS: usize = 200;
const LOOT_TABLES: usize = 200;
const SPLASHES: usize = 2_000;
const LANG_LINES: usize = 2_500;

/// The `manifest.json` of pack `n`, with one module of `kind` (`resources` or `data`).
fn manifest(n: usize, kind: &str) -> String {
    format!(
        r#"{{"format_version": 2, "header": {{"name": "bench pack {n}", "description": "", "uuid": "00000000-0000-4000-8000-{n:012}", "version": [1, 0, 0], "min_engine_version": [1, 21, 100]}}, "modules": [{{"type": "{kind}", "uuid": "00000000-0000-4000-9000-{n:012}", "version": [1, 0, 0]}}]}}"#
    )
}

/// Pack `n` of module `kind` holding `files`, opened from memory.
fn pack_of(n: usize, kind: &str, files: Vec<(String, String)>) -> Pack {
    let mut all = vec![(
        PackPath::from_static("manifest.json"),
        Bytes::from(manifest(n, kind)),
    )];
    all.extend(files.into_iter().map(|(path, text)| {
        (
            PackPath::new(&path).expect("a pack path"),
            Bytes::from(text),
        )
    }));
    Pack::from_files(all, &OpenOptions::default()).expect("the pack opens")
}

/// A fog document of 1.26.20 named `bench:fog_<i>`, with the air distance a fog needs.
fn fog_document(i: usize) -> String {
    format!(
        r##"{{"format_version": "1.26.20", "minecraft:fog_settings": {{"description": {{"identifier": "bench:fog_{i}"}}, "distance": {{"air": {{"fog_start": 0.9, "fog_end": 1.0, "fog_color": "#ABD2FF", "render_distance_type": "render"}}, "water": {{"fog_start": 0.0, "fog_end": 0.5, "fog_color": "#334455", "render_distance_type": "fixed"}}}}, "volumetric": {{"density": {{"air": {{"max_density": 0.25, "max_density_height": 64, "zero_density_height": 128, "uniform": false}}}}}}}}}}"##
    )
}

/// `count` fog files starting at fog `first`, so two packs can share a range.
fn fog_files(first: usize, count: usize) -> Vec<(String, String)> {
    (first..first + count)
        .map(|i| (format!("fogs/fog_{i}.json"), fog_document(i)))
        .collect()
}

/// A loot table of two pools.
fn loot_document(i: usize) -> String {
    format!(
        r#"{{"pools": [{{"rolls": {{"min": 1, "max": 3}}, "entries": [{{"type": "item", "name": "minecraft:apple", "weight": {w}, "functions": [{{"function": "set_count", "count": {{"min": 1, "max": 4}}}}]}}, {{"type": "item", "name": "minecraft:bread", "weight": 2}}]}}, {{"rolls": 1, "entries": [{{"type": "empty", "weight": {w}}}]}}]}}"#,
        w = i % 7 + 1
    )
}

/// `count` loot tables starting at table `first`.
fn loot_files(first: usize, count: usize) -> Vec<(String, String)> {
    (first..first + count)
        .map(|i| {
            (
                format!("loot_tables/chests/table_{i}.json"),
                loot_document(i),
            )
        })
        .collect()
}

/// A `splashes.json` of `count` plain entries that keeps the lower packs' (`canMerge`).
fn splashes_document(pack: usize, count: usize) -> String {
    let mut text = String::from(r#"{"canMerge": true, "splashes": ["#);
    for i in 0..count {
        if i > 0 {
            text.push_str(", ");
        }
        write!(text, r#""Splash {pack}.{i}!""#).expect("a string");
    }
    text.push_str("]}");
    text
}

/// A `.lang` file of `count` `key=value` lines, a comment every tenth.
fn lang_document(pack: usize, count: usize) -> String {
    let mut text = String::new();
    for i in 0..count {
        if i % 10 == 0 {
            writeln!(text, "## section {i}").expect("a string");
        }
        writeln!(
            text,
            "bench.key.{i}=Text {pack} of key {i} with %s and %1$d"
        )
        .expect("a string");
    }
    text
}

fn fog_packs() -> Vec<Pack> {
    vec![
        pack_of(1, "resources", fog_files(0, FOGS)),
        pack_of(2, "resources", fog_files(FOGS / 2, FOGS)),
    ]
}

fn loot_packs() -> Vec<Pack> {
    vec![
        pack_of(1, "data", loot_files(0, LOOT_TABLES)),
        pack_of(2, "data", loot_files(LOOT_TABLES / 2, LOOT_TABLES)),
    ]
}

fn splashes_packs() -> Vec<Pack> {
    (1..=3)
        .map(|n| {
            pack_of(
                n,
                "resources",
                vec![("splashes.json".to_owned(), splashes_document(n, SPLASHES))],
            )
        })
        .collect()
}

fn lang_packs() -> Vec<Pack> {
    (1..=2)
        .map(|n| {
            pack_of(
                n,
                "resources",
                vec![
                    ("texts/languages.json".to_owned(), r#"["en_US"]"#.to_owned()),
                    ("texts/en_US.lang".to_owned(), lang_document(n, LANG_LINES)),
                ],
            )
        })
        .collect()
}

/// A fresh resource stack over `packs`; nothing is loaded until an accessor is called.
fn resource_stack(packs: &[Pack]) -> PackStack<Resource> {
    PackStack::<Resource>::of(packs.iter().cloned(), LoadOptions::default()).expect("a stack")
}

/// The `registry` kind: `fog`.
fn registry(c: &mut Criterion) {
    let packs = fog_packs();
    let mut group = c.benchmark_group("registry");
    let stack = resource_stack(&packs);
    let fogs = stack.fogs();
    assert_eq!(fogs.len(), FOGS + FOGS / 2, "the fixture loads");
    group.bench_function("fog read", |b| {
        b.iter(|| {
            let stack = resource_stack(&packs);
            black_box(stack.fogs().len())
        });
    });
    group.bench_function("fog write", |b| {
        b.iter(|| {
            let mut writer = PackWriter::new(Manifest::new(
                "bench",
                bedrock_addon::pack::PackType::Resource,
            ));
            for (_, document) in fogs {
                writer.add(document).expect("a fog writes");
            }
            black_box(writer.files().count())
        });
    });
    group.finish();
}

/// The `files` kind: `loot`.
fn files(c: &mut Criterion) {
    let packs = loot_packs();
    let mut group = c.benchmark_group("files");
    let stack =
        PackStack::<Behavior>::of(packs.iter().cloned(), LoadOptions::default()).expect("a stack");
    let tables = stack.loot_tables();
    assert_eq!(
        tables.len(),
        LOOT_TABLES + LOOT_TABLES / 2,
        "the fixture loads"
    );
    group.bench_function("loot read", |b| {
        b.iter(|| {
            let stack = PackStack::<Behavior>::of(packs.iter().cloned(), LoadOptions::default())
                .expect("a stack");
            black_box(stack.loot_tables().len())
        });
    });
    group.bench_function("loot write", |b| {
        b.iter(|| {
            let mut writer = PackWriter::new(Manifest::new(
                "bench",
                bedrock_addon::pack::PackType::Behavior,
            ));
            for (path, document) in tables {
                writer.put(path.clone(), document).expect("a table writes");
            }
            black_box(writer.files().count())
        });
    });
    group.finish();
}

/// The `merged` kind: `splashes`.
fn merged(c: &mut Criterion) {
    let packs = splashes_packs();
    let mut group = c.benchmark_group("merged");
    let stack = resource_stack(&packs);
    let splashes = stack.splashes();
    assert_eq!(splashes.texts().len(), 3 * SPLASHES, "the fixture loads");
    group.bench_function("splashes read", |b| {
        b.iter(|| {
            let stack = resource_stack(&packs);
            black_box(stack.splashes().texts().len())
        });
    });
    group.bench_function("splashes write", |b| {
        b.iter(|| {
            let mut writer = PackWriter::new(Manifest::new(
                "bench",
                bedrock_addon::pack::PackType::Resource,
            ));
            for (n, layer) in splashes.layers().iter().enumerate() {
                writer
                    .put(
                        PackPath::new(&format!("splashes_{n}.json")).expect("a path"),
                        layer,
                    )
                    .expect("a splashes file writes");
            }
            black_box(writer.files().count())
        });
    });
    group.finish();
}

/// The hand-written `CollectionLoader` literal: `lang`.
fn hand(c: &mut Criterion) {
    let packs = lang_packs();
    let en_us = Language::new("en_US");
    let mut group = c.benchmark_group("hand");
    let stack = resource_stack(&packs);
    let table = stack.translations(&en_us).expect("en_US is listed");
    assert_eq!(table.value().strings.len(), LANG_LINES, "the fixture loads");
    group.bench_function("lang read", |b| {
        b.iter(|| {
            let stack = resource_stack(&packs);
            black_box(stack.translations(&en_us).map(|t| t.value().strings.len()))
        });
    });
    // The stack keeps the merged table; the file a pack holds is read on its own.
    let text = lang_document(1, LANG_LINES);
    let file = LangFile::parse(
        &PackPath::from_static("texts/en_US.lang"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("a lang file");
    group.bench_function("lang write", |b| {
        b.iter(|| {
            let mut out = Vec::with_capacity(text.len());
            file.write_json(&mut out, WriteStyle::default())
                .expect("a lang file writes");
            black_box(out.len())
        });
    });
    group.finish();
}

criterion_group!(benches, registry, files, merged, hand);
criterion_main!(benches);
