//! The round-trip guarantees, for every family, through `PackWriter`: a stack
//! is loaded, every document of every pack is written back with `PackWriter::put` into a
//! writer started with `PackWriter::from_pack`, the written packs are loaded as a stack again,
//! and the two loads are compared at the level the family is promised.
//!
//! The stack is the one of `determinism.rs`: a behaviour and a resource pack holding every
//! pack fragment a family left under `tests/fixtures/<module>/<family>/determinism/`, plus
//! the fragment of this suite, `tests/fixtures/pack/round_trip/` (documents of older format
//! versions, a subpack, a `.lang` file with CRLF line ends, a function with comments). Two
//! passes, as there: the default options, and every experiment on with the app-data files.
//!
//! One row per stack accessor (`writer_round_trip/rows.rs`); a `Collection` without a row
//! fails `every_collection_has_a_row`. A row names the document types of its files. Per file:
//!
//! * **Value**: the written file reads back as an equal value with equal findings (code,
//!   severity, path);
//! * **Value (raw)**: as Value, and the written text holds the same members in the same
//!   order with the same number tokens, compared through `bedrock_addon::json`;
//! * **Upgraded** (a document of an older format version in a Value row): it is written at
//!   the newest version and reads back as the value the first load gave; and written as it
//!   was read (`history::<Name>History::to_json`) it reads back at its own version, equal;
//! * **Lines**: the file differs from what was read only in the named ways (`.lang`: CRLF
//!   as LF; a function: comments, blank lines and CRLF gone), each counted and required to
//!   occur;
//! * **Exact**: the bytes are equal (`.mcstructure`; every file no document type claims is
//!   copied byte for byte).
//!
//! Per collection: the two loads' collections are equal as loaded (their `Debug`, text
//! positions aside) wherever the collection can be: where every document of the row is
//! written with the tokens it was read with. About half of the collections cannot (26 or 27
//! of 56 in the two passes), and are compared document by document alone: a collection
//! holds the version, the findings and, for a raw layer, the JSON of the FILE, so it differs
//! as soon as one of its documents was upgraded, is of the Lines level, is written with
//! another spelling than it was read with (`1` as `1.0`, a short form as the long one, a
//! member the load told and did not keep), or is of a type that records its file
//! ([`RECORDS_OF_THE_FILE`]). The `round-trip:` lines name them.
//! And the second write of the written packs is byte-identical to the first.

#![allow(dead_code, reason = "each feature set uses part of the helpers")]

#[path = "writer_round_trip/rows.rs"]
mod rows;

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use bedrock_addon::diagnostic::Diagnostics;
use bedrock_addon::document::Content;
use bedrock_addon::load::{
    AppData, CollectionSet, Experiment, Experiments, LoadContext, LoadOptions,
};
use bedrock_addon::manifest::Manifest;
use bedrock_addon::pack::{
    OpenOptions, Pack, PackOrigin, PackRepository, PackType, PackWriter, StackSet, WorldPacks,
};
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::{ENGINE_VERSION, MinEngineVersion};
use bytes::Bytes;

/// Whether every content feature is compiled in: the checks over the whole fixture stack
/// (every file claimed, every level met) need every family's document types.
const EVERY_FAMILY: bool = cfg!(all(
    feature = "block",
    feature = "voxel_shape",
    feature = "blocks_json",
    feature = "block_culling",
    feature = "item",
    feature = "crafting_catalog",
    feature = "attachable",
    feature = "entity",
    feature = "spawn_groups",
    feature = "spawn_rules",
    feature = "biome",
    feature = "dimension",
    feature = "feature",
    feature = "feature_rules",
    feature = "jigsaw",
    feature = "client_entity",
    feature = "atomic_client_entity",
    feature = "client_biome",
    feature = "biomes_client",
    feature = "loot",
    feature = "recipe",
    feature = "trade",
    feature = "dialogue",
    feature = "aim_assist",
    feature = "camera",
    feature = "animation",
    feature = "animation_controller",
    feature = "geometry",
    feature = "render_controller",
    feature = "fog",
    feature = "material",
    feature = "deferred_settings",
    feature = "texture_atlas",
    feature = "flipbooks",
    feature = "texture_list",
    feature = "texture_set",
    feature = "legacy_path_mapping",
    feature = "texture_file",
    feature = "sound_file",
    feature = "particle",
    feature = "sound_definitions",
    feature = "sounds_json",
    feature = "music",
    feature = "server_sound_definitions",
    feature = "lang",
    feature = "languages",
    feature = "function",
    feature = "json_ui",
    feature = "ddui",
    feature = "font",
    feature = "splashes",
    feature = "loading_messages",
    feature = "credits",
    feature = "ui_texture_info",
    feature = "poi",
    feature = "behavior_tree",
    feature = "structure",
    feature = "script"
));

// ---- the rows -------------------------------------------------------------------------------

/// The levels a document type is written at. `Upgraded` is no level of a
/// type: it is what `Value` means for a document read at an older version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Exact,
    Value,
    ValueRaw,
    Lines,
}

/// The stack side a document type is read on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Behavior,
    Resource,
    Both,
}

impl Side {
    fn has(self, pack_type: PackType) -> bool {
        match self {
            Side::Behavior => pack_type == PackType::Behavior,
            Side::Resource => pack_type == PackType::Resource,
            Side::Both => true,
        }
    }
}

/// One file handed to a document type: where it is, its bytes and its pack's context.
pub struct Carry<'a> {
    pub path: &'a PackPath,
    pub bytes: &'a [u8],
    pub cx: &'a LoadContext,
}

/// A document type of a row.
pub struct Kind {
    /// Its name in `common/document_types.rs`.
    pub name: &'static str,
    pub level: Level,
    pub side: Side,
    /// Whether the type claims the file at this key (the path without a subpack folder, or
    /// the app-data path).
    pub claims: fn(&str) -> bool,
    /// Reads the file and puts the document into the writer.
    pub carry: fn(&Carry<'_>, &mut PackWriter) -> Outcome,
    /// Whether two values of the type, as `carry` gave them, are equal.
    pub same: fn(&dyn Any, &dyn Any) -> bool,
    /// For a family with a version chain: the document written as it was read.
    pub as_written: Option<fn(&Carry<'_>) -> AsWritten>,
}

/// The row of one stack accessor.
pub struct Row {
    /// The `Collection` variant's name.
    pub collection: &'static str,
    pub kinds: Vec<Kind>,
}

impl Row {
    pub fn of(collection: &'static str, kinds: Vec<Kind>) -> Self {
        Self { collection, kinds }
    }
}

/// A finding as the levels compare it: code, severity and the path in the file.
type Finding = (String, String, String);

fn findings(diagnostics: &Diagnostics) -> BTreeSet<Finding> {
    diagnostics
        .iter()
        .map(|f| {
            (
                f.code.id().to_owned(),
                format!("{:?}", f.severity),
                f.location.json_path.to_string(),
            )
        })
        .collect()
}

/// What a document type made of a file.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// The file is no document of the type: it stays as it was copied.
    Rejected(BTreeSet<Finding>),
    Loaded {
        /// The value.
        value: Rc<dyn Any>,
        /// The `Debug` of the value, for a message.
        debug: String,
        findings: BTreeSet<Finding>,
        /// At least one upgrade step ran.
        upgraded: bool,
        /// The findings of the upgrade phase.
        of_the_upgrade: usize,
        /// What `PackWriter::put` said.
        put: Result<(), String>,
        /// The unknown members the write left out (`PackWriter::put_reporting`), by path.
        orphans: Vec<String>,
    },
}

/// Reads the file as a `T` and puts the document into the writer at the file's path.
pub fn carry<T: Content + Debug + PartialEq>(file: &Carry<'_>, writer: &mut PackWriter) -> Outcome {
    match T::parse(file.path, file.bytes, file.cx) {
        Err(rejected) => Outcome::Rejected(findings(&rejected.diagnostics)),
        Ok(document) => {
            let put = writer
                .put_reporting(file.path.clone(), &document)
                .map_err(|error| format!("{error:?}"));
            let orphans = put
                .as_ref()
                .map(|told| {
                    told.iter()
                        .map(|finding| finding.location.json_path.to_string())
                        .collect()
                })
                .unwrap_or_default();
            let put = put.map(drop);
            Outcome::Loaded {
                debug: format!("{:?} {:?}", document.value(), document.unknown()),
                findings: findings(document.diagnostics()),
                upgraded: document.version().upgraded,
                of_the_upgrade: document
                    .diagnostics()
                    .iter()
                    .filter(|f| f.phase == bedrock_addon::diagnostic::Phase::Upgrade)
                    .count(),
                put,
                orphans,
                value: Rc::new((document.unknown().clone(), document.into_value())),
            }
        }
    }
}

/// Whether two values `carry::<T>` gave are equal, their documents' unknown members too.
pub fn same<T: PartialEq + 'static>(a: &dyn Any, b: &dyn Any) -> bool {
    type Carried<T> = (bedrock_addon::document::Unknown, T);
    match (
        a.downcast_ref::<Carried<T>>(),
        b.downcast_ref::<Carried<T>>(),
    ) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// Where two texts first differ, with what stands around it in each.
fn first_difference(a: &str, b: &str) -> String {
    let at = a
        .bytes()
        .zip(b.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or(a.len().min(b.len()));
    let cut = |text: &str| {
        let mut from = at.saturating_sub(120);
        while !text.is_char_boundary(from) {
            from -= 1;
        }
        let mut to = (at + 120).min(text.len());
        while !text.is_char_boundary(to) {
            to += 1;
        }
        text[from..to].to_owned()
    };
    format!("\n read    …{}…\n written …{}…", cut(a), cut(b))
}

/// A document written as it was read (`history::<Name>History::to_json`) and read again.
#[derive(Clone, Debug, PartialEq)]
pub enum AsWritten {
    Rejected,
    NotWritten(String),
    Unreadable(String),
    Written {
        same_value: bool,
        read_at: String,
        written_at: String,
        rewritten: bool,
    },
}

// ---- the fixture packs ----------------------------------------------------------------------

/// Every file below `dir`, as `(path relative to dir, bytes)`, sorted.
fn files_below(dir: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((relative, std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// The `determinism/` fragments of every family, sorted by path, then this suite's own.
fn fragments() -> Vec<PathBuf> {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut found = Vec::new();
    for module in std::fs::read_dir(&fixtures).unwrap() {
        let module = module.unwrap().path();
        if !module.is_dir() {
            continue;
        }
        for family in std::fs::read_dir(&module).unwrap() {
            let fragment = family.unwrap().path().join("determinism");
            if fragment.is_dir() {
                found.push(fragment);
            }
        }
    }
    found.sort();
    found.push(fixtures.join("pack/round_trip"));
    found
}

/// A format-2 manifest with UUID `n`, one module of `module` type and `rest` after it.
fn manifest(n: u128, module: &str, rest: &str) -> String {
    format!(
        r#"{{
  "format_version": 2,
  "header": {{ "name": "pack {n}", "description": "", "uuid": "{}",
              "version": [1, 0, 0], "min_engine_version": [1, 21, 0] }},
  "modules": [{{ "type": "{module}", "uuid": "{}", "version": [1, 0, 0] }}]{rest}
}}"#,
        uuid::Uuid::from_u128(n),
        uuid::Uuid::from_u128(n | 0xffff_0000_0000_0000_0000),
    )
}

/// The files of the fixture stack: the two packs and the app data.
struct Fixture {
    behavior: BTreeMap<String, Vec<u8>>,
    resource: BTreeMap<String, Vec<u8>>,
    app_data: BTreeMap<String, Vec<u8>>,
}

/// The subpack of the behaviour pack (`pack/round_trip/behavior_pack/subpacks/alt/`).
const SUBPACK: &str = r#",
  "subpacks": [{ "folder_name": "alt", "name": "alt", "memory_tier": 0 }]"#;

fn fixture() -> Fixture {
    let archive = brarchive::serialize([("note.txt", &b"archived"[..])]).unwrap();
    let mut fixture = Fixture {
        behavior: BTreeMap::new(),
        resource: BTreeMap::new(),
        app_data: BTreeMap::new(),
    };
    let lang = b"pack.name=Round trip\n".to_vec();
    fixture.behavior.insert(
        "manifest.json".to_owned(),
        manifest(1, "data", SUBPACK).into(),
    );
    fixture
        .behavior
        .insert("texts/en_US.lang".to_owned(), lang.clone());
    fixture
        .behavior
        .insert("__brarchive/extra.brarchive".to_owned(), archive);
    fixture.resource.insert(
        "manifest.json".to_owned(),
        manifest(2, "resources", "").into(),
    );
    fixture.resource.insert("texts/en_US.lang".to_owned(), lang);
    for fragment in fragments() {
        for (path, bytes) in files_below(&fragment) {
            if let Some(rest) = path.strip_prefix("app_data/") {
                fixture.app_data.insert(rest.to_owned(), bytes);
            } else if let Some(rest) = path.strip_prefix("behavior_pack/") {
                fixture.behavior.insert(rest.to_owned(), bytes);
            } else if let Some(rest) = path.strip_prefix("resource_pack/") {
                fixture.resource.insert(rest.to_owned(), bytes);
            } else {
                fixture.behavior.insert(path.clone(), bytes.clone());
                fixture.resource.insert(path, bytes);
            }
        }
    }
    fixture
}

fn path(text: &str) -> PackPath {
    PackPath::new(text).unwrap_or_else(|error| panic!("{text}: {error:?}"))
}

fn open(files: &BTreeMap<String, Vec<u8>>) -> Pack {
    Pack::from_files(
        files
            .iter()
            .map(|(at, bytes)| (path(at), Bytes::from(bytes.clone()))),
        &OpenOptions::default(),
    )
    .unwrap()
}

fn app_data(files: &BTreeMap<String, Vec<u8>>) -> AppData {
    AppData::from_files(
        files
            .iter()
            .map(|(at, bytes)| (path(at), Bytes::from(bytes.clone()))),
    )
}

/// The options of the second pass: every experiment a world can turn on and the two toggles
/// (the app data is the pass's own).
fn everything_on() -> LoadOptions {
    let mut experiments = Experiments::new();
    for experiment in Experiment::ALL {
        if experiment.save_name().is_some() {
            experiments.insert(*experiment);
        }
    }
    LoadOptions {
        experiments,
        beta_apis: true,
        upcoming_creator_features: true,
        ..LoadOptions::default()
    }
}

/// The world list naming both packs.
fn world() -> WorldPacks {
    let entry = |n: u128| {
        format!(
            r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
            uuid::Uuid::from_u128(n)
        )
    };
    WorldPacks::parse(Some(entry(1).as_bytes()), Some(entry(2).as_bytes())).unwrap()
}

// ---- a load ---------------------------------------------------------------------------------

/// What a load of the stack gave: each collection as loaded, and every finding.
struct Loaded {
    /// `(side, collection)` to the collection's `Debug`, text positions taken out.
    collections: BTreeMap<(String, String), String>,
    /// Code, severity, file and path of every finding of the stack report.
    findings: BTreeSet<(String, String, String, String)>,
}

/// `text` without the text positions of its findings (`span: Some(Span { … })`): the written
/// file has its members at other offsets.
fn without_spans(text: &str) -> String {
    const START: &str = "span: Some(Span {";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(START) {
        out.push_str(&rest[..at]);
        out.push_str("span: _");
        let after = &rest[at + START.len()..];
        rest = after.find("})").map_or("", |end| &after[end + 2..]);
    }
    out.push_str(rest);
    out
}

/// The collections of a stack set's `Debug`: `(Some(Name), "<the collection's Debug>")`
/// entries, the behaviour stack's before the resource stack's.
fn collections(debug: &str) -> BTreeMap<(String, String), String> {
    let resource = debug
        .find("resource: PackStack")
        .expect("the resource stack");
    let mut out = BTreeMap::new();
    let mut from = 0;
    while let Some(found) = debug[from..].find("(Some(") {
        let start = from + found + "(Some(".len();
        from = start;
        let Some(len) = debug[start..].find("), \"") else {
            break;
        };
        let name = &debug[start..start + len];
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric()) {
            continue;
        }
        let body = start + len + "), \"".len();
        let mut end = body;
        let bytes = debug.as_bytes();
        while end < bytes.len() && bytes[end] != b'"' {
            end += if bytes[end] == b'\\' { 2 } else { 1 };
        }
        let side = if start < resource {
            "behavior"
        } else {
            "resource"
        };
        out.insert(
            (side.to_owned(), name.to_owned()),
            without_spans(&debug[body..end]),
        );
        from = end;
    }
    out
}

fn load(behavior: Pack, resource: Pack, options: &LoadOptions) -> Loaded {
    let mut repository = PackRepository::new();
    repository.add_pack(behavior, PackOrigin::World);
    repository.add_pack(resource, PackOrigin::World);
    let set: StackSet = repository.stacks(&world(), options.clone());
    let findings = set
        .load_all()
        .iter()
        .map(|f| {
            (
                f.code.id().to_owned(),
                format!("{:?}", f.severity),
                f.location
                    .path
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                f.location.json_path.to_string(),
            )
        })
        .collect();
    Loaded {
        collections: collections(&format!("{set:?}")),
        findings,
    }
}

// ---- a write --------------------------------------------------------------------------------

/// What a write of one pack (or of the app data) gave.
struct Written {
    /// Every file the writer held before a document was put: what the pack reads.
    copied: BTreeMap<String, Vec<u8>>,
    /// Every file of the writer but the manifest.
    files: BTreeMap<String, Vec<u8>>,
    /// The manifest as written.
    manifest: Vec<u8>,
    /// Per file a document type claimed: the type's name and what it made of the file.
    carried: BTreeMap<String, (&'static str, Outcome)>,
}

impl Written {
    fn all(&self) -> BTreeMap<String, Vec<u8>> {
        let mut all = self.files.clone();
        all.insert("manifest.json".to_owned(), self.manifest.clone());
        all
    }
}

/// The key a walk gives a file: its path without the folder of a subpack.
fn key(at: &str) -> &str {
    at.strip_prefix("subpacks/")
        .and_then(|rest| rest.split_once('/'))
        .map_or(at, |(_, key)| key)
}

/// The context of a pack of the stack, as the stack gives it to its loaders.
fn context(pack: &Pack, options: &LoadOptions) -> LoadContext {
    let min_engine_version = pack.manifest().header.min_engine_version.clone();
    let mut cx = LoadContext::for_pack(
        options,
        pack.id().uuid,
        pack.pack_type(),
        min_engine_version.clone(),
        false,
    );
    cx.molang_version = bedrock_addon::molang::pack_molang_version(min_engine_version.as_ref());
    cx
}

/// The context app data is read with: the game's own, at the engine version.
fn app_context(pack_type: PackType, options: &LoadOptions) -> LoadContext {
    let engine = MinEngineVersion::new(
        ENGINE_VERSION.major,
        ENGINE_VERSION.minor,
        ENGINE_VERSION.patch,
    );
    let mut cx = LoadContext::for_pack(
        options,
        uuid::Uuid::nil(),
        pack_type,
        Some(engine.clone()),
        true,
    );
    cx.molang_version = bedrock_addon::molang::pack_molang_version(Some(&engine));
    cx
}

/// The one document type that claims `at` on a pack of `pack_type`.
fn claimed<'r>(rows: &'r [Row], pack_type: PackType, at: &str) -> Option<&'r Kind> {
    let claiming: Vec<&Kind> = rows
        .iter()
        .flat_map(|row| &row.kinds)
        .filter(|kind| kind.side.has(pack_type) && (kind.claims)(key(at)))
        .collect();
    assert!(
        claiming.len() <= 1,
        "{at}: claimed by {:?}",
        claiming.iter().map(|kind| kind.name).collect::<Vec<_>>()
    );
    claiming.first().copied()
}

/// Puts every document of `writer`'s files back into it, each read with `cx_of` its type.
fn put_all(
    rows: &[Row],
    mut writer: PackWriter,
    pack_type: Option<PackType>,
    cx_of: &dyn Fn(&Kind) -> (PackType, LoadContext),
) -> Written {
    let files_before: Vec<(PackPath, Vec<u8>)> = writer
        .files()
        .filter(|(at, _)| at.as_str() != "manifest.json")
        .map(|(at, bytes)| (at.clone(), bytes.to_vec()))
        .collect();
    let mut carried = BTreeMap::new();
    for (at, bytes) in &files_before {
        let kind = match pack_type {
            Some(pack_type) => claimed(rows, pack_type, at.as_str()),
            // App data is no pack: a file is its type's on whichever side reads it.
            None => claimed(rows, PackType::Behavior, at.as_str())
                .or_else(|| claimed(rows, PackType::Resource, at.as_str())),
        };
        let Some(kind) = kind else {
            continue;
        };
        let (_, cx) = cx_of(kind);
        let file = Carry {
            path: at,
            bytes,
            cx: &cx,
        };
        let outcome = (kind.carry)(&file, &mut writer);
        carried.insert(at.to_string(), (kind.name, outcome));
    }
    let mut manifest = Vec::new();
    let files = writer
        .files()
        .filter_map(|(at, bytes)| {
            if at.as_str() == "manifest.json" {
                manifest = bytes.to_vec();
                None
            } else {
                Some((at.to_string(), bytes.to_vec()))
            }
        })
        .collect();
    Written {
        copied: files_before
            .into_iter()
            .map(|(at, bytes)| (at.to_string(), bytes))
            .collect(),
        files,
        manifest,
        carried,
    }
}

/// A pack written back: a writer started with `from_pack`, every document put at its path.
fn write_pack(rows: &[Row], files: &BTreeMap<String, Vec<u8>>, options: &LoadOptions) -> Written {
    let pack = open(files);
    let cx = context(&pack, options);
    let pack_type = pack.pack_type();
    let writer = PackWriter::from_pack(&pack).unwrap();
    put_all(rows, writer, Some(pack_type), &|_| (pack_type, cx.clone()))
}

/// The app data written back. It belongs to no pack, and a `PackWriter` writes packs: the
/// documents go through a writer of their own, whose files are the app data again.
fn write_app_data(
    rows: &[Row],
    files: &BTreeMap<String, Vec<u8>>,
    options: &LoadOptions,
) -> Written {
    let mut writer = PackWriter::new(Manifest::with_ids(
        "app data",
        PackType::Behavior,
        uuid::Uuid::from_u128(98),
        uuid::Uuid::from_u128(99),
    ));
    for (at, bytes) in files {
        writer.put_bytes(path(at), bytes.clone()).unwrap();
    }
    put_all(rows, writer, None, &|kind| {
        let pack_type = if kind.side == Side::Resource {
            PackType::Resource
        } else {
            PackType::Behavior
        };
        (pack_type, app_context(pack_type, options))
    })
}

// ---- the levels -----------------------------------------------------------------------------

/// The differences a Lines file may have from what was read, each counted.
#[derive(Debug, Default, PartialEq)]
struct LineLoss {
    /// Lines that ended in CRLF and are written with LF.
    crlf: usize,
    /// Comment lines of a function, not written.
    comments: usize,
    /// Blank lines of a function, not written.
    blank: usize,
    /// A last line without a line end, written with one.
    final_newline: usize,
}

/// The loss of a written Lines file, or what differs beyond the stated loss.
fn line_loss(name: &str, read: &[u8], written: &[u8]) -> Result<LineLoss, String> {
    let read = String::from_utf8_lossy(read);
    let written = String::from_utf8_lossy(written);
    let mut loss = LineLoss::default();
    let mut expected = String::new();
    for line in read.split_inclusive('\n') {
        let (body, end) = match line.strip_suffix("\r\n") {
            Some(body) => {
                loss.crlf += 1;
                (body, true)
            }
            None => match line.strip_suffix('\n') {
                Some(body) => (body, true),
                None => (line, false),
            },
        };
        if name == "function" {
            if body.trim().is_empty() {
                loss.blank += 1;
                continue;
            }
            if body.trim_start().starts_with('#') {
                loss.comments += 1;
                continue;
            }
        }
        expected.push_str(body);
        expected.push('\n');
        if !end {
            loss.final_newline += 1;
        }
    }
    if expected == written {
        Ok(loss)
    } else {
        Err(format!(
            "beyond the stated loss:\nexpected {expected:?}\nwritten  {written:?}"
        ))
    }
}

/// A JSON text as its tokens in order: member names, strings, and number and literal tokens
/// as written. Two texts with equal token lists hold the same members in the same order with
/// the same number tokens.
fn tokens(text: &[u8]) -> Result<Vec<String>, String> {
    use bedrock_addon::json::Value;
    fn walk(value: &Value, out: &mut Vec<String>) {
        if let Some(object) = value.as_object() {
            out.push("{".to_owned());
            for (key, member) in object {
                out.push(format!("k:{}", key.as_str()));
                walk(member, out);
            }
            out.push("}".to_owned());
        } else if let Some(array) = value.as_array() {
            out.push("[".to_owned());
            for element in array.as_slice() {
                walk(element, out);
            }
            out.push("]".to_owned());
        } else if let Some(text) = value.as_string() {
            out.push(format!("s:{}", text.as_str()));
        } else if let Some(number) = value.as_number() {
            // An integer and a real are different tokens (`1` and `1.0`).
            out.push(format!("n:{number:?}"));
        } else if let Some(flag) = value.as_bool() {
            out.push(format!("b:{flag}"));
        } else {
            out.push("null".to_owned());
        }
    }
    let text = std::str::from_utf8(text).map_err(|error| error.to_string())?;
    let value: Value =
        bedrock_addon::json::from_str_jsonc(text).map_err(|error| format!("{error:?}"))?;
    let mut out = Vec::new();
    walk(&value, &mut out);
    Ok(out)
}

/// Whether the JSON text has a value at `at` (a finding's path, `/` between its members).
fn has_member(text: &[u8], at: &str) -> bool {
    let Ok(text) = std::str::from_utf8(text) else {
        return false;
    };
    let Ok(mut value) = bedrock_addon::json::from_str_jsonc::<bedrock_addon::json::Value>(text)
    else {
        return false;
    };
    for segment in at.split('/').filter(|segment| !segment.is_empty()) {
        let next = if let Some(object) = value.as_object() {
            object.get(segment).cloned()
        } else if let Some(array) = value.as_array() {
            segment
                .parse::<usize>()
                .ok()
                .and_then(|index| array.as_slice().get(index).cloned())
        } else {
            None
        };
        match next {
            Some(next) => value = next,
            None => return false,
        }
    }
    true
}

/// Members of a value that record the file as it was read and not what the document says:
/// the document type, the member (as the value's `Debug` names it) and what it is. A written
/// file reads back as a value that differs there, and nowhere else; the written text is a
/// fixed point all the same (the second write is the first). Each row is a decision: a
/// caller relies on the member for what the FILE was, and a written file is another file.
const RECORDS_OF_THE_FILE: &[(&str, &str, &str)] = &[
    // The payload of `AddBehaviorTreePacket` is the file's text, byte for byte: a caller
    // sends what the pack has, and after a write the pack has the written text.
    (
        "behavior_tree",
        "raw",
        "the text of the file, with its line breaks and spaces",
    ),
    (
        "textures_list",
        "skipped_non_strings",
        "the count of the entries the load told and skipped, which a written list does not have",
    ),
];

/// Members that record an OLDER file: masked for an upgraded document alone. A document of
/// the newest format reads back equal in every one of them. The version of the file is what
/// validation and the client's flags compare (a caller reads which rules the file was under),
/// and a written file is of the newest version.
const RECORDS_OF_AN_OLDER_FILE: &[(&str, &str, &str)] = &[
    (
        "block",
        "source_format_version",
        "the version of the file, which a written file has as the newest",
    ),
    (
        "dimension",
        "source_format_version",
        "the version of the file",
    ),
    (
        "geometry",
        "format_version",
        "the version of the file, which a written file has as the newest",
    ),
];

/// `debug` without the value of each `member: …` in it.
fn without_member(debug: &str, member: &str) -> String {
    let start = format!("{member}: ");
    let mut out = String::with_capacity(debug.len());
    let mut rest = debug;
    while let Some(at) = rest.find(&start) {
        // A member, not the end of a longer name.
        let whole = rest[..at]
            .chars()
            .next_back()
            .is_none_or(|before| !before.is_alphanumeric() && before != '_');
        out.push_str(&rest[..at + start.len()]);
        rest = &rest[at + start.len()..];
        if !whole {
            continue;
        }
        // One `Debug` value: up to the `,` or the closing bracket at its own depth.
        let mut depth = 0usize;
        let mut in_text = false;
        let mut escaped = false;
        let mut end = rest.len();
        for (index, c) in rest.char_indices() {
            if in_text {
                match c {
                    _ if escaped => escaped = false,
                    '\\' => escaped = true,
                    '"' => in_text = false,
                    _ => {}
                }
                continue;
            }
            match c {
                '"' => in_text = true,
                '{' | '[' | '(' => depth += 1,
                '}' | ']' | ')' | ',' if depth == 0 => {
                    end = index;
                    break;
                }
                '}' | ']' | ')' => depth -= 1,
                _ => {}
            }
        }
        out.push('_');
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Whether two values that are not equal differ only in what records the file as read: the
/// members of [`RECORDS_OF_THE_FILE`], and, for an upgraded document, those of
/// [`RECORDS_OF_AN_OLDER_FILE`] and the mark of a component the upgrade made (`synthesized`),
/// which a written file has as its own.
fn differ_in_the_record_alone(kind: &str, upgraded: bool, read: &str, written: &str) -> bool {
    let mask = |debug: &str| {
        let mut debug = debug.to_owned();
        for (_, member, _) in RECORDS_OF_THE_FILE.iter().filter(|(of, _, _)| *of == kind) {
            debug = without_member(&debug, member);
        }
        if upgraded {
            for (_, member, _) in RECORDS_OF_AN_OLDER_FILE
                .iter()
                .filter(|(of, _, _)| *of == kind)
            {
                debug = without_member(&debug, member);
            }
            debug = debug.replace("synthesized: true", "synthesized: false");
        }
        debug
    };
    read != written && mask(read) == mask(written)
}

/// What the text an upgrade writes says when it is read, and the first load announced: the
/// document type; the code and the member of the written text the new finding is at; the
/// code of the announcement, an information of the first load, and the members of the FILE
/// it may be at, in the same item.
const ANNOUNCED: &[Announced] = &[
    // A legacy trade table's filters and functions keep their legacy evaluation: the table is
    // written back as a 1.20.30 legacy document (the newest format has them only under
    // deprecated names the game rejects), whose upgrade announces them again.
    (
        "trade_table",
        ("core/value_rewritten", "filters"),
        ("core/value_rewritten", &["filters", "biomes", "functions"]),
    ),
    (
        "trade_table",
        ("core/value_rewritten", "functions"),
        ("core/value_rewritten", &["functions"]),
    ),
];

/// A row of [`ANNOUNCED`].
type Announced = (
    &'static str,
    (&'static str, &'static str),
    (&'static str, &'static [&'static str]),
);

/// The path of the item a member is of, as both texts have it: without the member, and
/// without the `groups/<n>` the upgrade puts flat `trades` under.
fn item_of(path: &str) -> Vec<&str> {
    let mut segments: Vec<&str> = path.split('/').collect();
    segments.pop();
    let mut item = Vec::new();
    let mut skip = false;
    for segment in segments {
        if skip {
            skip = false;
        } else if segment == "groups" {
            skip = true;
        } else {
            item.push(segment);
        }
    }
    item
}

/// Whether `told`, a finding of the written text alone, was announced by the first load: by
/// an information of the row's code at one of the row's members of the SAME item (the item
/// of the file, or the item an expansion made of it, which lies below it).
fn announced(kind: &str, told: &Finding, first: &BTreeSet<Finding>) -> bool {
    let (code, _, path) = told;
    let last = |path: &str| path.rsplit('/').next().unwrap_or_default().to_owned();
    ANNOUNCED
        .iter()
        .any(|(of, (written, written_as), (announcement, from))| {
            *of == kind
                && written == code
                && last(path) == *written_as
                && first.iter().any(|(code, severity, at)| {
                    code == announcement
                        && severity == "Info"
                        && from.contains(&last(at).as_str())
                        && item_of(path).starts_with(&item_of(at))
                })
        })
}

/// What one pass counted.
#[derive(Debug, Default)]
struct Tally {
    /// Per document type: documents carried, of which upgraded.
    carried: BTreeMap<&'static str, (usize, usize)>,
    /// Files no type took (rejected by their type, or no document type claims them).
    copied: usize,
    /// The files no document type claims, and those their type rejected.
    unclaimed: BTreeSet<String>,
    rejected: BTreeSet<String>,
    loss: LineLoss,
    /// Per document type with a version chain: documents written as they were read.
    as_written: BTreeMap<&'static str, usize>,
    /// Documents the load did not keep whole: a finding at a member the written text does
    /// not have (told, not written), by document type.
    not_kept_whole: BTreeMap<&'static str, usize>,
    /// Upgraded documents whose written text has findings the read file did not have, each
    /// announced by the first load ([`ANNOUNCED`]), by file.
    announced_by_the_first_load: BTreeSet<String>,
    /// Those findings themselves: file, code and path.
    announced: BTreeSet<(String, String, String)>,
    /// Documents whose written file reads back as a value that differs only in what records
    /// the file as read ([`RECORDS_OF_THE_FILE`], `synthesized`), by document type.
    differ_in_the_record: BTreeMap<&'static str, usize>,
    /// Collections equal as loaded, and those compared document by document alone.
    collections_equal: usize,
    collections_by_document: BTreeSet<String>,
}

/// The faults of one written pack against what was read, by level.
#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "one check a level"
)]
fn judge(
    rows: &[Row],
    name: &str,
    first: &Written,
    second: &Written,
    cx_of: &dyn Fn(&Kind) -> LoadContext,
    tally: &mut Tally,
    faults: &mut Vec<String>,
) {
    let kind_of = |kind: &str| {
        rows.iter()
            .flat_map(|row| &row.kinds)
            .find(|other| other.name == kind)
            .expect("a kind of the rows")
    };
    // The second write of the written pack is the first, byte for byte.
    if first.all() != second.all() {
        let differing: Vec<&String> = first
            .files
            .keys()
            .filter(|at| first.files.get(*at) != second.files.get(*at))
            .collect();
        faults.push(format!(
            "{name}: the second write differs from the first at {differing:?} (manifest equal: {})",
            first.manifest == second.manifest
        ));
    }
    if first.files.keys().ne(first.copied.keys()) {
        faults.push(format!("{name}: a write added or removed a file"));
    }
    for (at, bytes) in &first.files {
        let Some(was) = first.copied.get(at) else {
            continue;
        };
        let Some((kind, outcome)) = first.carried.get(at) else {
            // No document type claims it: copied.
            tally.copied += 1;
            tally.unclaimed.insert(format!("{name}/{at}"));
            if bytes != was {
                faults.push(format!("{name}/{at}: a copied file has other bytes"));
            }
            continue;
        };
        let Outcome::Loaded {
            value,
            debug,
            findings,
            upgraded,
            put,
            orphans,
            ..
        } = outcome
        else {
            // Not a document of its type: it stays as it was, and reads the same again.
            tally.copied += 1;
            tally.rejected.insert(format!("{name}/{at}"));
            if bytes != was {
                faults.push(format!("{name}/{at}: a rejected file has other bytes"));
            }
            match (outcome, second.carried.get(at)) {
                (Outcome::Rejected(read), Some((_, Outcome::Rejected(again)))) if read == again => {
                }
                _ => faults.push(format!("{name}/{at}: rejected otherwise the second time")),
            }
            continue;
        };
        let counted = tally.carried.entry(*kind).or_default();
        counted.0 += 1;
        counted.1 += usize::from(*upgraded);
        for orphan in orphans {
            // A document written as it was read writes every unknown member it kept.
            faults.push(format!(
                "{name}/{at}: an unknown member is not written: {orphan}"
            ));
        }
        if let Err(error) = put {
            faults.push(format!("{name}/{at}: {kind} is not written: {error}"));
            continue;
        }
        let kind = kind_of(kind);
        let Some((
            _,
            Outcome::Loaded {
                value: value_again,
                debug: debug_again,
                findings: findings_again,
                upgraded: upgraded_again,
                ..
            },
        )) = second.carried.get(at)
        else {
            faults.push(format!(
                "{name}/{at}: the written {} does not load: {:?}",
                kind.name,
                second.carried.get(at)
            ));
            continue;
        };
        match kind.level {
            Level::Exact => {
                if bytes != was {
                    faults.push(format!("{name}/{at}: {} is not written exactly", kind.name));
                }
            }
            Level::Lines => match line_loss(kind.name, was, bytes) {
                Ok(loss) => {
                    tally.loss.crlf += loss.crlf;
                    tally.loss.comments += loss.comments;
                    tally.loss.blank += loss.blank;
                    tally.loss.final_newline += loss.final_newline;
                }
                Err(error) => faults.push(format!("{name}/{at}: {error}")),
            },
            Level::Value | Level::ValueRaw => {
                // The findings of the read file the written one does not have are those at
                // members the written text does not have: what the load told and did not
                // keep. Its value is compared like every other (a member that counts what
                // was dropped is one of `RECORDS_OF_THE_FILE`).
                let gone: Vec<&Finding> = findings.difference(findings_again).collect();
                let told_not_kept =
                    !gone.is_empty() && gone.iter().all(|(_, _, path)| !has_member(bytes, path));
                let new: Vec<&Finding> = findings_again.difference(findings).collect();
                if told_not_kept {
                    *tally.not_kept_whole.entry(kind.name).or_default() += 1;
                } else if !gone.is_empty() && !*upgraded {
                    faults.push(format!(
                        "{name}/{at}: the written {} lacks findings of members it has: {gone:?}",
                        kind.name
                    ));
                }
                // What the written text says and the read file did not: nothing, but what
                // the first load announced of the text an upgrade writes ([`ANNOUNCED`]).
                let unannounced: Vec<&&Finding> = new
                    .iter()
                    .filter(|told| !(*upgraded && announced(kind.name, told, findings)))
                    .collect();
                if !unannounced.is_empty() {
                    faults.push(format!(
                        "{name}/{at}: the written {} has new findings {unannounced:?}",
                        kind.name
                    ));
                } else if !new.is_empty() {
                    tally.announced_by_the_first_load.insert(at.clone());
                    for (code, _, path) in &new {
                        tally
                            .announced
                            .insert((at.clone(), code.clone(), path.clone()));
                    }
                }
                let equal = (kind.same)(&**value, &**value_again);
                let in_the_record =
                    differ_in_the_record_alone(kind.name, *upgraded, debug, debug_again);
                if !equal && in_the_record {
                    *tally.differ_in_the_record.entry(kind.name).or_default() += 1;
                }
                if !equal && !in_the_record {
                    faults.push(format!(
                        "{name}/{at}: the written {} reads back as another value{}",
                        kind.name,
                        first_difference(debug, debug_again)
                    ));
                }
                if *upgraded_again && !*upgraded {
                    faults.push(format!(
                        "{name}/{at}: {} is written at an older version than it was read at",
                        kind.name
                    ));
                }
                if kind.level == Level::ValueRaw && !*upgraded {
                    match (tokens(was), tokens(bytes)) {
                        (Ok(read), Ok(written)) if read == written => {}
                        (Ok(read), Ok(written)) => {
                            let at_token = read
                                .iter()
                                .zip(&written)
                                .position(|(a, b)| a != b)
                                .unwrap_or(read.len().min(written.len()));
                            faults.push(format!(
                                "{name}/{at}: the written {} has other members, another order or other number tokens, from token {at_token}: read {:?}, written {:?}",
                                kind.name,
                                read.get(at_token.saturating_sub(2)..(at_token + 4).min(read.len())),
                                written.get(at_token.saturating_sub(2)..(at_token + 4).min(written.len())),
                            ));
                        }
                        (read, written) => faults.push(format!(
                            "{name}/{at}: no JSON: {:?} {:?}",
                            read.err(),
                            written.err()
                        )),
                    }
                }
            }
        }
        // Upgraded, the other half: written as it was read, it reads back at its version.
        if *upgraded {
            let Some(as_written) = kind.as_written else {
                faults.push(format!(
                    "{name}/{at}: {} was upgraded and its row names no version chain",
                    kind.name
                ));
                continue;
            };
            let cx = cx_of(kind);
            let file = Carry {
                path: &path(at),
                bytes: was,
                cx: &cx,
            };
            match as_written(&file) {
                AsWritten::Written {
                    same_value: true,
                    read_at,
                    written_at,
                    rewritten: true,
                } if read_at == written_at => {
                    *tally.as_written.entry(kind.name).or_default() += 1;
                }
                other => faults.push(format!(
                    "{name}/{at}: {} written as it was read: {other:?}",
                    kind.name
                )),
            }
        }
    }
}

/// The document types of the packs written first that the second load holds as the documents
/// of the first, not as collections: one upgraded, of the Lines level, rewritten or recorded.
fn changed_kinds<'a>(
    rows: &'a [Row],
    behavior_and_resource: [&'a Written; 2],
    app_written: Option<&'a Written>,
) -> BTreeSet<&'a str> {
    [behavior_and_resource[0], behavior_and_resource[1]]
        .into_iter()
        .chain(app_written)
        .flat_map(|written| {
            written
                .carried
                .iter()
                .map(move |(at, carried)| (at, carried, written))
        })
        .filter_map(|(at, (kind, outcome), written)| match outcome {
            Outcome::Loaded {
                upgraded,
                of_the_upgrade,
                ..
            } => {
                let lines = rows
                    .iter()
                    .flat_map(|row| &row.kinds)
                    .any(|other| other.name == *kind && other.level == Level::Lines);
                // A text with other members than the read one (a member told and not kept,
                // an order the writer gives): the collection holds the file as it was read.
                let rewritten = match (written.copied.get(at), written.files.get(at)) {
                    (Some(read), Some(wrote)) => tokens(read).ok() != tokens(wrote).ok(),
                    _ => false,
                };
                let records = RECORDS_OF_THE_FILE.iter().any(|(of, _, _)| of == kind);
                (*upgraded || *of_the_upgrade > 0 || lines || rewritten || records).then_some(*kind)
            }
            Outcome::Rejected(_) => None,
        })
        .collect()
}

/// Compares the collections of the two loads: equal as loaded, or counted by document, or a
/// fault.
fn compare_collections(
    rows: &[Row],
    loaded: &Loaded,
    again: &Loaded,
    changed: &BTreeSet<&str>,
    tally: &mut Tally,
    faults: &mut Vec<String>,
) {
    for ((side, collection), debug) in &loaded.collections {
        let Some(debug_again) = again.collections.get(&(side.clone(), collection.clone())) else {
            faults.push(format!(
                "{side} {collection}: not loaded from the written packs"
            ));
            continue;
        };
        let by_document = rows
            .iter()
            .filter(|row| row.collection == collection)
            .flat_map(|row| &row.kinds)
            .any(|kind| changed.contains(kind.name));
        if by_document {
            tally
                .collections_by_document
                .insert(format!("{side} {collection}"));
        } else if debug == debug_again {
            tally.collections_equal += 1;
        } else {
            let at = debug
                .bytes()
                .zip(debug_again.bytes())
                .position(|(a, b)| a != b)
                .unwrap_or(0);
            let from = at.saturating_sub(200);
            let cut = |text: &str| {
                text.get(from..(at + 200).min(text.len()))
                    .map(str::to_owned)
            };
            faults.push(format!(
                "{side} {collection}: the written packs load as another collection\n read    …{:?}\n written …{:?}",
                cut(debug),
                cut(debug_again)
            ));
        }
    }
}

/// One pass: the stack loaded, written, loaded again, written again; the faults by level.
fn round_trip(options: &LoadOptions, with_app_data: bool) -> (Tally, Vec<String>) {
    let rows = rows::rows();
    let fixture = fixture();
    let app = with_app_data.then_some(&fixture.app_data);
    let options_with = |app: Option<&BTreeMap<String, Vec<u8>>>| LoadOptions {
        app_data: app.map(app_data),
        ..options.clone()
    };

    let loaded = load(
        open(&fixture.behavior),
        open(&fixture.resource),
        &options_with(app),
    );
    // The first write, and the stack of the written packs.
    let behavior = write_pack(&rows, &fixture.behavior, options);
    let resource = write_pack(&rows, &fixture.resource, options);
    let app_written = app.map(|files| write_app_data(&rows, files, options));
    let again = load(
        open(&behavior.all()),
        open(&resource.all()),
        &options_with(app_written.as_ref().map(|written| &written.files)),
    );
    // The second write, of the written packs.
    let behavior_again = write_pack(&rows, &behavior.all(), options);
    let resource_again = write_pack(&rows, &resource.all(), options);
    let app_again = app_written
        .as_ref()
        .map(|written| write_app_data(&rows, &written.files, options));

    let mut tally = Tally::default();
    let mut faults = Vec::new();
    let pack_cx = |files: &BTreeMap<String, Vec<u8>>| context(&open(files), options);
    let (behavior_cx, resource_cx) = (pack_cx(&fixture.behavior), pack_cx(&fixture.resource));
    judge(
        &rows,
        "behavior",
        &behavior,
        &behavior_again,
        &|_| behavior_cx.clone(),
        &mut tally,
        &mut faults,
    );
    judge(
        &rows,
        "resource",
        &resource,
        &resource_again,
        &|_| resource_cx.clone(),
        &mut tally,
        &mut faults,
    );
    if let (Some(first), Some(second)) = (&app_written, &app_again) {
        judge(
            &rows,
            "app data",
            first,
            second,
            &|kind| {
                let pack_type = if kind.side == Side::Resource {
                    PackType::Resource
                } else {
                    PackType::Behavior
                };
                app_context(pack_type, options)
            },
            &mut tally,
            &mut faults,
        );
    }

    // The collections of the two loads. A collection none of whose documents was upgraded
    // or is of the Lines level is equal as loaded; the others hold the versions and the
    // findings of the files that were read, and are compared document by document above.
    let changed = changed_kinds(&rows, [&behavior, &resource], app_written.as_ref());
    compare_collections(&rows, &loaded, &again, &changed, &mut tally, &mut faults);
    // The written packs say nothing the read ones did not, but what the newest format says
    // of the form an upgrade writes (counted per file above).
    let new: Vec<_> = again
        .findings
        .difference(&loaded.findings)
        .filter(|(code, _, file, path)| {
            !tally
                .announced
                .contains(&(file.clone(), code.clone(), path.clone()))
        })
        .collect();
    if !new.is_empty() {
        faults.push(format!("the written packs load with new findings: {new:?}"));
    }
    (tally, faults)
}

fn report(pass: &str, tally: &Tally) {
    let documents: usize = tally.carried.values().map(|(count, _)| count).sum();
    let upgraded: usize = tally.carried.values().map(|(_, upgraded)| upgraded).sum();
    println!(
        "round-trip: {pass} | document types {} | documents {documents} (upgraded {upgraded}, written as read {}) | copied {} | not kept whole {:?} | differ in the record of the file {:?} | announced by the first load {:?} | collections equal as loaded {}, by document {:?} | lines {:?}",
        tally.carried.len(),
        tally.as_written.values().sum::<usize>(),
        tally.copied,
        tally.not_kept_whole,
        tally.differ_in_the_record,
        tally.announced_by_the_first_load,
        tally.collections_equal,
        tally.collections_by_document,
        tally.loss,
    );
}

// ---- the tests ------------------------------------------------------------------------------

/// The names of every `Collection` compiled in, from the `Debug` of the set of all.
fn collection_names() -> Vec<String> {
    format!("{:?}", CollectionSet::ALL)
        .trim_matches(|c| c == '{' || c == '}')
        .split(',')
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .collect()
}

#[test]
fn every_collection_has_a_row() {
    let rows: Vec<&str> = rows::rows().iter().map(|row| row.collection).collect();
    assert_eq!(
        rows,
        collection_names(),
        "the rows are the accessor list, in its order"
    );
}

#[test]
fn every_family_round_trips_at_its_level() {
    let (tally, faults) = round_trip(&LoadOptions::default(), false);
    report("default options", &tally);
    assert!(
        faults.is_empty(),
        "{} faults:\n{}",
        faults.len(),
        faults.join("\n")
    );
}

#[test]
fn every_family_round_trips_with_experiments_and_app_data() {
    let (tally, faults) = round_trip(&everything_on(), true);
    report("experiments and app data", &tally);
    assert!(
        faults.is_empty(),
        "{} faults:\n{}",
        faults.len(),
        faults.join("\n")
    );
}

#[macro_use]
#[path = "common/document_types.rs"]
mod document_types;

/// The names of the document types of `common/document_types.rs`.
macro_rules! names {
    ($($name:ident : $ty:ty, $path:literal $(in $cx:expr)?, $doc:expr $(, [$($extra:tt)*])? ;)*) => {
        &[$(stringify!($name)),*]
    };
}
const DOCUMENT_TYPES: &[&str] = document_types!(names);

/// The document types that no row has, each with the reason: no stack accessor reads their
/// files, or another name of the list is the same type on a row.
const NOT_ON_A_ROW: &[(&str, &str)] = &[
    (
        "block_cereal",
        "the block at its newest format: the type of `block`",
    ),
    (
        "item_beta",
        "a beta document: no fragment of the stack has one",
    ),
    (
        "entity_beta",
        "a beta document: no fragment of the stack has one",
    ),
    (
        "biome_beta",
        "a beta document: no fragment of the stack has one",
    ),
    (
        "client_biome_beta",
        "a beta document: no fragment of the stack has one",
    ),
    (
        "manifest",
        "the writer's own: `PackWriter::manifest_mut`, the suite of `writer.rs`",
    ),
    (
        "camera_entity",
        "no stack accessor reads `cameras/*.json` of a resource pack",
    ),
    (
        "legacy_path_mapping",
        "no stack accessor returns the mapping as a document",
    ),
    ("skin_pack", "a skin pack is no pack of a stack"),
    (
        "persona_piece_meta",
        "a persona piece is no pack of a stack (`read_only`)",
    ),
    (
        "persona_piece_store_info",
        "a persona piece is no pack of a stack (`read_only`)",
    ),
];

/// The files of the fixture stack that are no document: copied as their bytes.
const NO_DOCUMENT: &[&str] = &[
    ".png",
    ".ogg",
    ".ttf",
    ".js",
    ".txt",
    ".dat",
    ".mcfunction",
    ".lang",
    ".tga",
    ".fsb",
];

#[test]
fn every_document_type_is_on_a_row_or_named() {
    let rows = rows::rows();
    let on_a_row: BTreeSet<&str> = rows
        .iter()
        .flat_map(|row| &row.kinds)
        .map(|kind| kind.name)
        .collect();
    if !EVERY_FAMILY {
        return;
    }
    for name in DOCUMENT_TYPES {
        let named = NOT_ON_A_ROW.iter().any(|(other, _)| other == name);
        assert!(
            on_a_row.contains(name) != named,
            "{name}: on a row {}, named as on none {named}",
            on_a_row.contains(name)
        );
    }
}

/// The fixture stack is one in which every level can fail: every document type of a row has
/// a document that is carried, every family with a version chain a document of an older
/// version (written upgraded, and as it was read), the Lines level each of its losses, and
/// no JSON file is left to no type.
#[test]
fn the_fixture_stack_has_a_document_for_every_row_and_level() {
    if !EVERY_FAMILY {
        return;
    }
    let (default, _) = round_trip(&LoadOptions::default(), false);
    let (everything, _) = round_trip(&everything_on(), true);
    let rows = rows::rows();
    let carried = |name: &str| {
        let of = |tally: &Tally| tally.carried.get(name).copied().unwrap_or_default();
        (
            of(&default).0 + of(&everything).0,
            of(&default).1 + of(&everything).1,
        )
    };
    let written_as_read = |name: &str| {
        default.as_written.get(name).copied().unwrap_or_default()
            + everything.as_written.get(name).copied().unwrap_or_default()
    };
    let mut missing = Vec::new();
    for kind in rows.iter().flat_map(|row| &row.kinds) {
        let (documents, upgraded) = carried(kind.name);
        if documents == 0 {
            missing.push(format!("{}: no document of the type is carried", kind.name));
        }
        if kind.as_written.is_some() && (upgraded == 0 || written_as_read(kind.name) == 0) {
            missing.push(format!(
                "{}: no document of an older version ({upgraded} upgraded, {} written as read)",
                kind.name,
                written_as_read(kind.name)
            ));
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
    for tally in [&default, &everything] {
        let loss = &tally.loss;
        assert!(
            loss.crlf > 0 && loss.comments > 0 && loss.blank > 0 && loss.final_newline > 0,
            "every loss of the Lines level occurs: {loss:?}"
        );
        let json: Vec<&String> = tally
            .unclaimed
            .iter()
            .filter(|at| !NO_DOCUMENT.iter().any(|extension| at.ends_with(extension)))
            .collect();
        assert!(json.is_empty(), "files no document type claims: {json:?}");
        println!("round-trip: rejected by their type {:?}", tally.rejected);
    }
}

// ---- Read-only (last row): what the writer does with what has no writer --------

/// A brarchive layer is written as loose files: the copy has every file the layer serves,
/// with the bytes the pack reads, and not the archive.
#[test]
fn read_only_a_brarchive_layer_is_written_as_loose_files() {
    let fixture = fixture();
    assert!(fixture.behavior.contains_key("__brarchive/extra.brarchive"));
    let written = write_pack(&rows::rows(), &fixture.behavior, &LoadOptions::default());
    assert_eq!(
        written.files.get("extra/note.txt").map(Vec::as_slice),
        Some(&b"archived"[..])
    );
    assert!(
        !written.files.keys().any(|at| at.ends_with(".brarchive")),
        "{:?}",
        written.files.keys().collect::<Vec<_>>()
    );
    // The pack of the written files reads the same file, loose.
    let pack = open(&written.all());
    assert_eq!(
        &pack.read(&path("extra/note.txt")).unwrap()[..],
        b"archived"
    );
}

/// An MCB document (the binary JSON of a `.brarchive` entry or of a loose file) whose body
/// the decoder reads is written as JSON text, which its document type reads to the value
/// the stack gave; one of no known layout has no text: the copy keeps its bytes, and the
/// written pack says of it what the read one did (`UnsupportedFileFormat`).
// The baked shape is decoded and written again by the voxel shape family.
#[cfg(all(feature = "block", feature = "voxel_shape"))]
#[test]
fn read_only_an_mcb_document_is_written_as_json_text_where_its_body_is_decoded() {
    let archive = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../storage/tests/fixtures/brarchive/mcb.brarchive"),
    )
    .unwrap();
    // A baked voxel shape, `demo:step` with one box.
    let mut shape = b"\x7FMCB\x01\x00\x15\x00\x6e\x00\x00\x00\x15minecraft:voxel_shape".to_vec();
    shape.push(9);
    shape.extend(b"demo:step");
    shape.push(1);
    for value in [0.0_f32, 0.0, 0.0, 16.0, 8.0, 16.0] {
        shape.extend(value.to_le_bytes());
    }
    let mut files = BTreeMap::new();
    files.insert(
        // The pack the world of `load` names.
        "manifest.json".to_owned(),
        manifest(1, "data", "").into_bytes(),
    );
    files.insert("__brarchive/items.brarchive".to_owned(), archive);
    files.insert("shapes/step.json".to_owned(), shape);
    let pack = open(&files);
    // Voxel shapes are behind an experiment.
    let options = everything_on();
    let written = write_pack(&rows::rows(), &files, &options);
    let binary: Vec<&String> = written
        .files
        .iter()
        .filter(|(_, bytes)| bytes.starts_with(b"\x7FMCB"))
        .map(|(at, _)| at)
        .collect();
    assert_eq!(binary, ["items/ruby.json"], "{:?}", written.files.keys());
    // Its bytes are the entry's, as the pack reads them; no document type took it.
    assert_eq!(
        &pack.read(&path(binary[0])).unwrap()[..],
        &written.files[binary[0]][..]
    );
    assert!(match written.carried.get(binary[0]) {
        None | Some((_, Outcome::Rejected(_))) => true,
        Some((_, Outcome::Loaded { .. })) => false,
    });
    // The shape is text (the writer of its document type wrote it again), and that type
    // reads it.
    let text = String::from_utf8_lossy(&written.files["shapes/step.json"]);
    assert!(
        text.contains(r#""identifier": "demo:step""#) && text.contains("16.0"),
        "{text}"
    );
    assert!(matches!(
        written.carried.get("shapes/step.json"),
        Some(("voxel_shape", Outcome::Loaded { .. }))
    ));
    // The stack of the written pack holds the shape the stack of the read one decoded, and
    // tells of the document without a text what that one told.
    let fixture = fixture();
    let read = load(open(&files), open(&fixture.resource), &options);
    let again = load(open(&written.all()), open(&fixture.resource), &options);
    let key = ("behavior".to_owned(), "VoxelShapes".to_owned());
    let shapes = read.collections.get(&key).expect("the shapes of the stack");
    assert!(shapes.contains("demo:step"), "{shapes}");
    assert_eq!(Some(shapes), again.collections.get(&key));
    for loaded in [&read, &again] {
        let told: Vec<_> = loaded
            .findings
            .iter()
            .filter(|(code, _, file, _)| {
                code == "core/unsupported_file_format" && file == "items/ruby.json"
            })
            .collect();
        // (The item loader is the one that reads `items/`, and so the one that tells.)
        assert_eq!(
            told.len(),
            usize::from(cfg!(feature = "item")),
            "{:?}",
            loaded.findings
        );
    }
}

/// An encrypted pack is not written: content keys never leave the reader.
#[cfg(feature = "encrypted")]
#[test]
fn read_only_an_encrypted_pack_is_not_written() {
    use bedrock_addon::pack::WriteError;
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../storage/tests/fixtures/encrypted/testPack");
    let files: BTreeMap<String, Vec<u8>> = files_below(&dir).into_iter().collect();
    let pack = open(&files);
    assert!(matches!(
        PackWriter::from_pack(&pack),
        Err(WriteError::Encrypted)
    ));
}

/// A persona piece is no pack of a stack and has no accessor; its two documents have
/// writers all the same, at the Value level.
#[cfg(feature = "persona")]
#[test]
fn read_only_a_persona_piece_document_is_written_by_put_and_reads_back() {
    use bedrock_addon::families::persona::PieceStoreInfo;
    let text = r#"{"piece_store_name": "n", "piece_thumbnail": "textures/persona_thumbnails/hat_Thumbnail_0", "kept": {"a": 1}}"#;
    let at = path("pieces/hat/hat.store_info.json");
    let cx = LoadContext::detached();
    let read = PieceStoreInfo::parse(&at, text.as_bytes(), &cx).unwrap();
    let mut writer = PackWriter::new(Manifest::with_ids(
        "piece",
        PackType::Resource,
        uuid::Uuid::from_u128(5),
        uuid::Uuid::from_u128(6),
    ));
    writer.put(at.clone(), &read).unwrap();
    let written: Vec<u8> = writer
        .files()
        .find(|(file, _)| **file == at)
        .map(|(_, bytes)| bytes.to_vec())
        .unwrap();
    let again = PieceStoreInfo::parse(&at, &written, &cx).unwrap();
    assert_eq!(again.value(), read.value());
    assert_eq!(again.unknown(), read.unknown());
    assert_eq!(tokens(text.as_bytes()), tokens(&written));
}

/// Script files have no document type (the bundle is read-only metadata over their bytes):
/// a copy keeps every one byte for byte, and the bundles of the written packs are those of
/// the read ones.
#[cfg(feature = "script")]
#[test]
fn read_only_script_files_are_copied_and_the_bundles_are_the_same() {
    let fixture = fixture();
    let written = write_pack(&rows::rows(), &fixture.behavior, &LoadOptions::default());
    let scripts: Vec<&String> = fixture
        .behavior
        .keys()
        .filter(|at| at.starts_with("scripts/"))
        .collect();
    assert!(scripts.len() >= 3, "{scripts:?}");
    for at in scripts {
        assert_eq!(written.files.get(at), fixture.behavior.get(at), "{at}");
        assert!(!written.carried.contains_key(at), "{at}");
    }
    let options = LoadOptions::default();
    let resource = write_pack(&rows::rows(), &fixture.resource, &options);
    let read = load(open(&fixture.behavior), open(&fixture.resource), &options);
    let again = load(open(&written.all()), open(&resource.all()), &options);
    for side in ["behavior", "resource"] {
        let key = (side.to_owned(), "Scripts".to_owned());
        assert!(read.collections.contains_key(&key), "{side}");
        assert_eq!(
            read.collections.get(&key),
            again.collections.get(&key),
            "{side}"
        );
    }
}
