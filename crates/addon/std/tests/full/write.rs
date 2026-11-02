//! The writer over a tree of the game's packs: every pack copied through `PackWriter` and
//! loaded again as the same world, and every typed document of every pack written back through
//! `PackWriter::put`, at the levels the writer promises (equal as a value, or as an upgraded
//! value).
//!
//! A binary includes this file with `#[path = "full/write.rs"] mod write;` beside
//! `support`.
//!
//! The packs are read from disk by this file (every file, as a `PackWriter` needs it) and
//! opened with `Pack::from_files`; nothing here goes through a driver but the load of the
//! written world on rayon.

#![allow(dead_code, reason = "each binary uses the part its tree needs")]

#[path = "kinds.rs"]
mod rows;

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bedrock_addon::document::Content;
use bedrock_addon::json::WriteStyle;
use bedrock_addon::load::{LoadContext, LoadOptions, RECOMMENDED_JOB_STACK, StackOptions};
use bedrock_addon::pack::{
    OpenOptions, Pack, PackOrigin, PackRepository, PackStack, PackType, PackWriter, StackSet,
    WriteError,
};
use bedrock_addon::storage::PackPath;
use bytes::Bytes;

use crate::common::world::base_game_world;
use crate::support::{Counts, Facts, Run, Tree, driver, facts, load_options, on_a_thread};

// ---- the packs of a tree, in memory ---------------------------------------------------------

/// Every file below `dir`, as `(path relative to dir, bytes)`.
fn files_below(dir: &Path) -> Vec<(PackPath, Bytes)> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(folder) = pending.pop() {
        let entries = std::fs::read_dir(&folder)
            .unwrap_or_else(|error| panic!("{}: {error}", folder.display()));
        for entry in entries {
            let path = entry.expect("a directory entry").path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let relative = path
                .strip_prefix(dir)
                .expect("below the folder")
                .components()
                .map(|part| part.as_os_str().to_str().expect("a UTF-8 name"))
                .collect::<Vec<_>>()
                .join("/");
            let at = PackPath::new(&relative)
                .unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
            let bytes =
                std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            files.push((at, Bytes::from(bytes)));
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}

/// The pack folders of the tree at `root`, behaviour packs first, each list by name.
fn pack_folders(tree: &Tree, root: &Path) -> Vec<std::path::PathBuf> {
    if tree.samples {
        return vec![root.join("behavior_pack"), root.join("resource_pack")];
    }
    let mut folders = Vec::new();
    for packs in ["behavior_packs", "resource_packs"] {
        let mut children: Vec<_> = std::fs::read_dir(root.join(packs))
            .unwrap_or_else(|error| panic!("{packs}: {error}"))
            .map(|entry| entry.expect("a directory entry").path())
            .filter(|path| path.is_dir())
            .collect();
        children.sort();
        folders.extend(children);
    }
    folders
}

/// Every pack of the tree at `root`, every file resident, opened as the app's own.
pub fn packs(tree: &Tree, root: &Path) -> Vec<Pack> {
    let options = OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    };
    pack_folders(tree, root)
        .iter()
        .map(|folder| {
            Pack::from_files(files_below(folder), &options)
                .unwrap_or_else(|error| panic!("{}: {error:?}", folder.display()))
        })
        .collect()
}

/// The world of `packs` with the options of `run` for the tree, composed as
/// `support::stacks` composes the tree from disk.
pub fn world(tree: &Tree, root: &Path, run: Run, packs: Vec<Pack>) -> StackSet {
    let options = load_options(tree, root, run);
    if tree.samples {
        let mut packs = packs.into_iter();
        let (behavior, resource) = (packs.next().expect("two packs"), packs.next().expect("two"));
        return StackSet {
            behavior: PackStack::of([behavior], options.clone()).expect("a stack"),
            resource: PackStack::of([resource], options).expect("a stack"),
        };
    }
    let mut repository = PackRepository::new();
    for pack in packs {
        repository.add_pack(pack, PackOrigin::Package);
    }
    // The vanilla experimental packs whose experiment the run leaves off are not on the world.
    let world = crate::support::without_inactive_experimental_packs(
        base_game_world(&repository, &options),
        &options,
    );
    repository.stacks(&world, options)
}

/// The facts of the world of `packs`: loaded through a rayon driver when `rayon` (the packs
/// are in memory, so the driver runs the jobs and reads nothing), else by the accessors on
/// a thread of `RECOMMENDED_JOB_STACK`.
pub fn facts_of(tree: Tree, root: &Path, packs: Vec<Pack>, rayon: bool) -> Facts {
    let at = root.to_path_buf();
    on_a_thread(RECOMMENDED_JOB_STACK, move || {
        let started = Instant::now();
        let set = world(&tree, &at, Run::SERVER, packs);
        let set = if rayon {
            driver(true)
                .load_stacks(set, StackOptions::default())
                .expect("the stacks load")
        } else {
            set
        };
        facts(&tree, &set, started)
    })
}

// ---- the copy -------------------------------------------------------------------------------

/// The files of a pack root a writer leaves out: they list and sign the bytes it is there to
/// change.
const NOT_COPIED: [&str; 2] = ["contents.json", "signatures.json"];

/// What the copy of a tree's packs came to.
pub struct Copied {
    /// The counts of the pin file `<tree>.copy`.
    pub counts: Counts,
    /// The packs to load in place of the tree's: each written pack, and the pack itself
    /// where the writer refused it (an encrypted pack).
    pub packs: Vec<Pack>,
}

/// Copies every pack of `packs` (read from `folders`, in their order) with
/// `PackWriter::from_pack` and `into_pack`, and requires of each copy: the same identity and
/// manifest; every file of the pack with its bytes, but `contents.json` and `signatures.json`
/// of the root and the `.brarchive` archives whose entries are copied as loose files; no file
/// the pack does not have. A baked binary document is copied as the JSON text it stands for,
/// where the crate decodes it ([`same_reading`] holds that text against the game's own), or
/// as its bytes.
#[expect(
    clippy::too_many_lines,
    reason = "one pass over each pack whose counters and failure lists are shared by every step"
)]
pub fn copy(tree: &Tree, packs: Vec<Pack>, folders: &[PathBuf], options: &LoadOptions) -> Copied {
    assert_eq!(packs.len(), folders.len(), "a folder for each pack");
    let mut unlike_seen = BTreeSet::new();
    let rows = rows::rows();
    let mut compared = BTreeMap::<(&'static str, &'static str), usize>::new();
    let mut unequal = Vec::new();
    let mut counts = Counts::new();
    let mut count = |key: &str, by: usize| *counts.entry(key.to_owned()).or_insert(0) += by;
    for key in [
        "packs",
        "packs copied",
        "packs refused: encrypted",
        "files copied",
        "bytes copied",
        "baked binary documents copied as JSON text",
        "baked binary documents copied as their bytes",
        "files left out: contents.json, signatures.json",
        "files left out: brarchive archives",
        "manifests written with other bytes than read",
    ] {
        count(key, 0);
    }
    let mut out = Vec::new();
    for (pack, folder) in packs.into_iter().zip(folders) {
        count("packs", 1);
        let name = format!("{:?}", pack.id());
        let writer = match PackWriter::from_pack(&pack) {
            Ok(writer) => writer,
            Err(WriteError::Encrypted) => {
                count("packs refused: encrypted", 1);
                out.push(pack);
                continue;
            }
            Err(error) => panic!("{name}: {error:?}"),
        };
        let written: BTreeMap<PackPath, Vec<u8>> = writer
            .files()
            .map(|(path, bytes)| (path.clone(), bytes.to_vec()))
            .collect();
        let cx = context(&pack, options);
        let read: BTreeSet<&PackPath> = pack.paths().collect();
        for (path, bytes) in &written {
            assert!(read.contains(path), "{name}: {path} is no file of the pack");
            let original = pack.read(path).expect("a resident file");
            if path.as_str() == "manifest.json" {
                if original[..] != bytes[..] {
                    count("manifests written with other bytes than read", 1);
                }
                continue;
            }
            if crate::common::game_files::is_mcb(&original) {
                // A baked binary document: copied as the JSON text it stands for where the
                // crate decodes its layout, else as its bytes.
                if original[..] == bytes[..] {
                    count("baked binary documents copied as their bytes", 1);
                } else {
                    assert!(
                        !crate::common::game_files::is_mcb(bytes)
                            && bytes
                                .first()
                                .is_some_and(|byte| matches!(byte, b'{' | b'[')),
                        "{name}: {path} is a baked document copied as neither its bytes nor a JSON text"
                    );
                    count("baked binary documents copied as JSON text", 1);
                    let baked = Baked {
                        name: &name,
                        pack: &pack,
                        cx: &cx,
                        path,
                        twin: folder.join(path.as_str()),
                    };
                    match same_reading(&rows, &baked, bytes) {
                        Ok(seen) => *compared.entry(seen).or_insert(0) += 1,
                        Err(fault)
                            if READ_UNLIKE_THE_TWIN.contains(&(tree.name, path.as_str())) =>
                        {
                            unlike_seen.insert(path.as_str().to_owned());
                            *compared
                                .entry(("", "reads otherwise than its JSON twin, as expected"))
                                .or_insert(0) += 1;
                            eprintln!("  expected to read otherwise: {fault}");
                        }
                        Err(fault) => unequal.push(fault),
                    }
                }
                continue;
            }
            assert!(original[..] == bytes[..], "{name}: {path} differs");
            count("files copied", 1);
            count("bytes copied", bytes.len());
        }
        for path in &read {
            if written.contains_key(*path) {
                continue;
            }
            if NOT_COPIED.contains(&path.as_str()) {
                count("files left out: contents.json, signatures.json", 1);
            } else if path.as_str().ends_with(".brarchive") {
                count("files left out: brarchive archives", 1);
            } else {
                panic!("{name}: {path} is not copied");
            }
        }
        let copy = writer
            .into_pack(PackOrigin::Package)
            .unwrap_or_else(|error| panic!("{name}: {error:?}"));
        assert_eq!(copy.id(), pack.id(), "{name}");
        assert_eq!(copy.pack_type(), pack.pack_type(), "{name}");
        assert_eq!(copy.manifest(), pack.manifest(), "{name}: the manifest");
        let copied: BTreeSet<&PackPath> = copy.paths().collect();
        assert_eq!(
            copied,
            written.keys().collect::<BTreeSet<_>>(),
            "{name}: the files of the copy"
        );
        for (path, bytes) in &written {
            assert!(
                copy.read(path).expect("a resident file")[..] == bytes[..],
                "{name}: {path} of the copy differs"
            );
        }
        count("packs copied", 1);
        out.push(copy);
    }
    eprintln!("baked documents copied as JSON text, by type and twin: {compared:?}");
    for (name, path) in READ_UNLIKE_THE_TWIN {
        assert!(
            *name != tree.name || unlike_seen.contains(*path),
            "{name}: {path} reads as its JSON twin now: take it out of READ_UNLIKE_THE_TWIN"
        );
    }
    assert!(
        unequal.is_empty(),
        "{} baked documents whose copy does not read as the game's JSON twin:\n{}",
        unequal.len(),
        unequal.join("\n")
    );
    Copied { counts, packs: out }
}

/// The baked documents of a tree whose JSON twin reads as another value than the decoded text,
/// by tree and path: the storage crate's own list
/// (`storage/tests/mcb/vanilla.rs::READ_DIFFERENTLY`). Six particle effects ship a twin in the form their author wrote where the body has the
/// baked form (defaults written out, a gradient's colours as numbers, no `velocity`, the Molang
/// version); two items have their components in another order in the twin. (The storage
/// crate's list also has a camera whose `-0.0` is `-0` in the twin: it compares `Debug` texts,
/// and `PartialEq` takes the two zeros as equal.) Each listed document must be met reading
/// otherwise.
const READ_UNLIKE_THE_TWIN: &[(&str, &str)] = &[
    ("client_install_a", "particles/sulfur_cube_particle.json"),
    // The client half of the split pack tree holds the same file.
    ("split_client", "particles/sulfur_cube_particle.json"),
    ("client_install_b", "items/red_shrub.json"),
    ("client_install_b", "items/shelf_mushroom.json"),
    ("client_install_b", "particles/geyser_base.particle.json"),
    ("client_install_b", "particles/geyser_plume.particle.json"),
    ("client_install_b", "particles/geyser_poof.particle.json"),
    ("client_install_b", "particles/noxious_gas_particle.json"),
    ("client_install_b", "particles/sulfur_bubble_particle.json"),
    ("client_install_b", "particles/sulfur_cube_particle.json"),
];

/// A baked binary document of a pack, and where its JSON twin would be on disk.
struct Baked<'a> {
    name: &'a str,
    pack: &'a Pack,
    cx: &'a LoadContext,
    path: &'a PackPath,
    /// The loose file of the same path, which the pack ships beside the archive whose entry
    /// shadows it: the game's own JSON of the document, the oracle of the decoder's layouts.
    twin: PathBuf,
}

/// A baked document copied as JSON text, against the game's own JSON of it: the document type
/// claiming the path reads the copied text, and, where the pack ships the document's JSON twin
/// (a loose file of the same path that is no baked document), reads the twin as an equal
/// value. `Ok` names the type and whether the twin was there; `Err` says what differs.
fn same_reading(
    rows: &[Row],
    baked: &Baked<'_>,
    copied: &[u8],
) -> Result<(&'static str, &'static str), String> {
    let (name, path) = (baked.name, baked.path);
    let claiming: Vec<&Kind> = rows
        .iter()
        .flat_map(|row| &row.kinds)
        .filter(|kind| kind.side.has(baked.pack.pack_type()) && (kind.claims)(key(path.as_str())))
        .collect();
    // The camera entity has no stack accessor and so no row: its reader stands in for one.
    let camera = (baked.pack.pack_type() == PackType::Resource
        && key(path.as_str()).starts_with("cameras/"))
    .then_some((
        "camera_entity",
        carry::<bedrock_addon::families::camera::CameraEntity> as Carrier,
    ));
    let (kind, carrier) = match (&claiming[..], camera) {
        ([kind], _) => (kind.name, kind.carry),
        ([], Some(camera)) => camera,
        _ => {
            return Err(format!(
                "{name}: {path}: a baked document claimed by {:?}",
                claiming.iter().map(|kind| kind.name).collect::<Vec<_>>()
            ));
        }
    };
    let twin = std::fs::read(&baked.twin)
        .ok()
        .filter(|bytes| !crate::common::game_files::is_mcb(bytes));
    let (bytes, seen) = match &twin {
        Some(twin) => (&twin[..], "against its JSON twin"),
        None => (copied, "no JSON twin: read alone"),
    };
    let mut scratch = PackWriter::new(baked.pack.manifest().clone());
    let outcome = carrier(
        &Carry {
            path,
            bytes,
            cx: baked.cx,
            copied: Some(copied),
        },
        &mut scratch,
    );
    match outcome {
        Outcome::Compared { same: true, .. } => Ok((kind, seen)),
        outcome => Err(format!("{name}: {path} ({kind}, {seen}): {outcome:?}")),
    }
}

// ---- the rewrite ----------------------------------------------------------------------------

/// The levels at which a document type is written back.
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
    /// The text a copy holds for the file (a baked binary document copied as JSON text): when
    /// given, `carry` reads both as the type and compares them, and writes nothing.
    pub copied: Option<&'a [u8]>,
}

/// What a document type does with a file (`carry::<T>`).
type Carrier = fn(&Carry<'_>, &mut PackWriter) -> Outcome;

/// A document type of a row (`kinds.rs`).
pub struct Kind {
    pub name: &'static str,
    pub level: Level,
    pub side: Side,
    /// Whether the type claims the file at this key (the path without a subpack folder).
    pub claims: fn(&str) -> bool,
    /// Reads the file and puts the document into the writer.
    pub carry: Carrier,
    pub same: fn(&dyn Any, &dyn Any) -> bool,
    pub as_written: Option<fn(&Carry<'_>) -> AsWritten>,
}

/// The row of one stack accessor.
pub struct Row {
    pub collection: &'static str,
    pub kinds: Vec<Kind>,
}

impl Row {
    pub fn of(collection: &'static str, kinds: Vec<Kind>) -> Self {
        Self { collection, kinds }
    }
}

/// The rows' own result of a document written as it was read; this file does not ask for it.
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

/// What a document type made of a file and of the text it wrote for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The file is no document of the type: it stays as it was copied.
    Rejected,
    /// The value has no text (`PackWriter::put` refused it): the file stays as copied.
    NotWritten(String),
    /// The text written does not read as a document of the type.
    Unreadable,
    /// The file and the text its copy holds, both read as the type (`Carry::copied`).
    Compared {
        /// The two values are equal.
        same: bool,
        /// Every member the `Debug` of the two values differs in, and the texts around the
        /// first place, when they are not `same`.
        difference: (String, String),
    },
    /// Written, and read again.
    Written {
        /// The file was read at an older version than the one it is written at.
        upgraded: bool,
        /// The value read from the written text is equal to the value that was written.
        same: bool,
        /// The value read from the written text is written as that text again: where it is
        /// not `same`, the two differ in what the type records of the file it was read from
        /// and never writes.
        same_text: bool,
        /// Every member the `Debug` of the two values differs in, as one name (`{a, b}`), and
        /// the texts around the first place, when they are not `same`.
        difference: (String, String),
    },
}

/// Reads the file as a `T`, puts the document into the writer at the file's path, and reads
/// the text the writer holds for it as a `T` again; or, for a file with the text of its copy,
/// [`compare`]s the two.
pub fn carry<T: Content + Debug + PartialEq>(file: &Carry<'_>, writer: &mut PackWriter) -> Outcome {
    let Ok(document) = T::parse(file.path, file.bytes, file.cx) else {
        return Outcome::Rejected;
    };
    if let Some(copied) = file.copied {
        return compare(&document, file, copied);
    }
    if let Err(error) = writer.put(file.path.clone(), document.value()) {
        return Outcome::NotWritten(format!("{error:?}"));
    }
    // The text `put` wrote: the same call with the same style.
    let mut text = Vec::new();
    document
        .value()
        .write_json(&mut text, WriteStyle::default())
        .expect("put wrote it");
    match T::parse(file.path, &text, file.cx) {
        Err(_) => Outcome::Unreadable,
        Ok(again) => {
            let mut second = Vec::new();
            let rewritten = again.value().write_json(&mut second, WriteStyle::default());
            let same = again.value() == document.value();
            Outcome::Written {
                upgraded: document.version().upgraded,
                same,
                same_text: rewritten.is_ok() && second == text,
                difference: if same {
                    (String::new(), String::new())
                } else {
                    let (read, written) = (
                        format!("{:?}", document.value()),
                        format!("{:?}", again.value()),
                    );
                    (
                        format!("{{{}}}", differing_members(&read, &written).join(", ")),
                        first_difference(&read, &written),
                    )
                },
            }
        }
    }
}

/// The value `document` (read from the file) against the value read from the text its copy
/// holds.
fn compare<T: Content + Debug + PartialEq>(
    document: &bedrock_addon::document::Document<T>,
    file: &Carry<'_>,
    copied: &[u8],
) -> Outcome {
    let Ok(again) = T::parse(file.path, copied, file.cx) else {
        return Outcome::Unreadable;
    };
    let same = again.value() == document.value();
    Outcome::Compared {
        same,
        difference: if same {
            (String::new(), String::new())
        } else {
            let (read, copy) = (
                format!("{:?}", document.value()),
                format!("{:?}", again.value()),
            );
            (
                format!("{{{}}}", differing_members(&read, &copy).join(", ")),
                first_difference(&read, &copy),
            )
        },
    }
}

// ---- which members two values differ in -----------------------------------------------------

/// A piece of a value's `Debug` text: plain text (a number, a name, a string with its
/// quotes) or a bracketed group of elements, each with the name before its `: ` if it has
/// one (a field of a struct, a key of a map or of a JSON text).
#[derive(Debug, PartialEq)]
enum Piece {
    Text(String),
    Group(char, Vec<(Option<String>, Vec<Piece>)>),
}

/// Reads the elements of a group up to its closing bracket (or the end of the text).
fn elements(text: &[char], at: &mut usize) -> Vec<(Option<String>, Vec<Piece>)> {
    let mut out = Vec::new();
    loop {
        while *at < text.len() && (text[*at] == ' ' || text[*at] == ',') {
            *at += 1;
        }
        if *at >= text.len() || matches!(text[*at], ')' | ']' | '}') {
            *at += 1;
            return out;
        }
        let name = name_of(text, at);
        out.push((name, value(text, at)));
    }
}

/// A string literal starting at `at` (its opening quote), escapes and all.
fn string(text: &[char], at: &mut usize) -> String {
    let start = *at;
    *at += 1;
    while *at < text.len() && text[*at] != '"' {
        if text[*at] == '\\' {
            *at += 1;
        }
        *at += 1;
    }
    *at = (*at + 1).min(text.len());
    text[start..*at].iter().collect()
}

/// The name of an element, when it has one: a word or a string followed by `: `.
fn name_of(text: &[char], at: &mut usize) -> Option<String> {
    let mut end = *at;
    let name: String = if text[end] == '"' {
        let literal = string(text, &mut end);
        literal.trim_matches('"').to_owned()
    } else {
        while end < text.len() && (text[end].is_alphanumeric() || matches!(text[end], '_' | '.')) {
            end += 1;
        }
        text[*at..end].iter().collect()
    };
    let named = !name.is_empty() && text.get(end) == Some(&':') && text.get(end + 1) == Some(&' ');
    named.then(|| {
        *at = end + 2;
        name
    })
}

/// The pieces of one element's value, up to the `,` or the bracket that ends it.
fn value(text: &[char], at: &mut usize) -> Vec<Piece> {
    let mut pieces = Vec::new();
    let mut plain = String::new();
    while *at < text.len() {
        match text[*at] {
            ',' | ')' | ']' | '}' => break,
            '"' => plain.push_str(&string(text, at)),
            open @ ('(' | '[' | '{') => {
                if !plain.trim().is_empty() {
                    pieces.push(Piece::Text(plain.trim().to_owned()));
                }
                plain = String::new();
                *at += 1;
                pieces.push(Piece::Group(open, elements(text, at)));
            }
            other => {
                plain.push(other);
                *at += 1;
            }
        }
    }
    if !plain.trim().is_empty() {
        pieces.push(Piece::Text(plain.trim().to_owned()));
    }
    pieces
}

/// The name a difference below the names `path` is counted under: the innermost name; for a
/// number of a version (`major`, `minor`, `patch`) the name of the version; and for anything
/// inside `shared` (the components of an item's file as the file wrote them) `shared/` and
/// the component; a `category` below `menu_category` is `menu_category/category`.
fn member(path: &[&str]) -> String {
    if let Some(at) = path.iter().position(|name| *name == "shared") {
        return format!("shared/{}", path.get(at + 1).copied().unwrap_or("?"));
    }
    let mut names = path.iter().rev();
    match names.next() {
        None => String::from("?"),
        Some(name) if matches!(*name, "major" | "minor" | "patch") => {
            (*names.next().unwrap_or(name)).to_owned()
        }
        // An item has two: `menu_category/category` and the legacy reading's `category`.
        Some(name) if *name == "category" => match names.next() {
            Some(parent) if *parent == "menu_category" => String::from("menu_category/category"),
            _ => String::from("category"),
        },
        Some(name) => (*name).to_owned(),
    }
}

/// Collects into `out` the members two values' pieces differ in.
fn differing<'a>(
    a: &'a [Piece],
    b: &'a [Piece],
    path: &mut Vec<&'a str>,
    out: &mut BTreeSet<String>,
) {
    if a.len() != b.len() {
        out.insert(member(path));
        return;
    }
    for (a, b) in a.iter().zip(b) {
        match (a, b) {
            (Piece::Text(a), Piece::Text(b)) if a == b => {}
            (Piece::Group(open_a, a), Piece::Group(open_b, b))
                if open_a == open_b
                    && a.len() == b.len()
                    && a.iter().zip(b).all(|(a, b)| a.0 == b.0) =>
            {
                for ((name, a), (_, b)) in a.iter().zip(b) {
                    if let Some(name) = name {
                        path.push(name);
                    }
                    differing(a, b, path, out);
                    if name.is_some() {
                        path.pop();
                    }
                }
            }
            _ => {
                out.insert(member(path));
            }
        }
    }
}

/// EVERY member the `Debug` texts of two values differ in, sorted: the fields of the type
/// and the keys of the JSON texts it holds, each by the name [`member`] gives it. Two texts
/// that are equal differ in none; two that differ where nothing is named differ in `?`.
pub fn differing_members(a: &str, b: &str) -> Vec<String> {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let (a, b) = (value(&a, &mut 0), value(&b, &mut 0));
    let mut out = BTreeSet::new();
    differing(&a, &b, &mut Vec::new(), &mut out);
    out.into_iter().collect()
}

/// What stands around the first place two texts differ at, in each.
fn first_difference(a: &str, b: &str) -> String {
    let at = a
        .bytes()
        .zip(b.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or(a.len().min(b.len()));
    let cut = |text: &str| {
        let mut from = at.saturating_sub(90);
        while !text.is_char_boundary(from) {
            from -= 1;
        }
        let mut to = (at + 90).min(text.len());
        while !text.is_char_boundary(to) {
            to += 1;
        }
        text[from..to].to_owned()
    };
    format!("read …{}… written …{}…", cut(a), cut(b))
}

/// Whether two values `carry::<T>` gave are equal.
pub fn same<T: PartialEq + 'static>(a: &dyn Any, b: &dyn Any) -> bool {
    match (a.downcast_ref::<T>(), b.downcast_ref::<T>()) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// The key a walk gives a file: its path without the folder of a subpack.
fn key(at: &str) -> &str {
    at.strip_prefix("subpacks/")
        .and_then(|rest| rest.split_once('/'))
        .map_or(at, |(_, key)| key)
}

/// The context of `pack` as a stack gives it to its loaders.
fn context(pack: &Pack, options: &LoadOptions) -> LoadContext {
    let min_engine_version = pack.manifest().header.min_engine_version.clone();
    let mut cx = LoadContext::for_pack(
        options,
        pack.id().uuid,
        pack.pack_type(),
        min_engine_version.clone(),
        true,
    );
    cx.molang_version = bedrock_addon::molang::pack_molang_version(min_engine_version.as_ref());
    cx
}

/// The count of the documents whose written text reads back as a value that is not equal to
/// the one written and is written as the same text again: the two differ in something the
/// writer does not write (what the type records of the file it was read from: its format
/// version, its raw text, a mark of what the load added) or writes the same for both (a
/// default of the version the file was read at). The count is by the set of EVERY member
/// the two differ in ([`differing_members`]), so that no difference stands behind another.
pub const UNEQUAL: &str = "read back unequal and written as the same text";
/// The count of the documents whose written text reads back as a value that is written as
/// another text: the value changed.
pub const CHANGED: &str = "read back as another value";

/// What the rewrite of a tree's packs came to.
pub struct Rewritten {
    /// The counts of the pin file `<tree>.rewrite`, by `<document type> <what>`.
    pub counts: Counts,
    /// The packs to load: each rewritten pack, and the pack itself where the writer refused
    /// it.
    pub packs: Vec<Pack>,
    /// One line for each document that did not come back equal to what was written, the
    /// first two of each type and member.
    pub faults: Vec<String>,
}

/// Writes every typed document of every pack of `packs` back through `PackWriter::put` into a
/// writer started with `from_pack`, each read with its pack's context under `options`.
#[expect(
    clippy::too_many_lines,
    reason = "one pass over each pack whose counters and failure lists are shared by every step"
)]
pub fn rewrite(packs: Vec<Pack>, options: &LoadOptions) -> Rewritten {
    let rows = rows::rows();
    let mut counts = Counts::new();
    let mut faults: Vec<String> = Vec::new();
    let mut shown: BTreeMap<(&'static str, String), usize> = BTreeMap::new();
    let mut out = Vec::new();
    for pack in packs {
        let Ok(mut writer) = PackWriter::from_pack(&pack) else {
            out.push(pack);
            continue;
        };
        let cx = context(&pack, options);
        let pack_type = pack.pack_type();
        let files: Vec<(PackPath, Vec<u8>)> = writer
            .files()
            .filter(|(at, _)| at.as_str() != "manifest.json")
            .map(|(at, bytes)| (at.clone(), bytes.to_vec()))
            .collect();
        for (at, bytes) in &files {
            let claiming: Vec<&Kind> = rows
                .iter()
                .flat_map(|row| &row.kinds)
                .filter(|kind| kind.side.has(pack_type) && (kind.claims)(key(at.as_str())))
                .collect();
            assert!(
                claiming.len() <= 1,
                "{at}: claimed by {:?}",
                claiming.iter().map(|kind| kind.name).collect::<Vec<_>>()
            );
            let Some(kind) = claiming.first() else {
                *counts
                    .entry(String::from("(no document type) files"))
                    .or_insert(0) += 1;
                continue;
            };
            let outcome = (kind.carry)(
                &Carry {
                    path: at,
                    bytes,
                    cx: &cx,
                    copied: None,
                },
                &mut writer,
            );
            let mut count =
                |what: &str| *counts.entry(format!("{} {what}", kind.name)).or_insert(0) += 1;
            let fault = match &outcome {
                Outcome::Rejected => {
                    if crate::common::game_files::is_mcb(bytes) {
                        count("baked binary documents, kept as their bytes");
                    } else {
                        count("rejected, kept as their bytes");
                    }
                    None
                }
                Outcome::NotWritten(why) => {
                    count("not written");
                    Some((String::from("not written"), why.clone()))
                }
                Outcome::Unreadable => {
                    count("written and not read back");
                    Some((String::from("written and not read back"), String::new()))
                }
                Outcome::Compared { .. } => unreachable!("the rewrite hands no copied text"),
                Outcome::Written {
                    upgraded,
                    same,
                    same_text,
                    difference,
                } => {
                    count("written");
                    if *upgraded {
                        count("written at a newer version than read");
                    }
                    if *same {
                        None
                    } else if *same_text {
                        let what = format!("{UNEQUAL}, in {}", difference.0);
                        count(&what);
                        Some((what, difference.1.clone()))
                    } else {
                        let what = format!("{CHANGED}, in {}", difference.0);
                        count(&what);
                        Some((what, difference.1.clone()))
                    }
                }
            };
            if let Some((what, why)) = fault {
                let seen = shown.entry((kind.name, what.clone())).or_insert(0);
                *seen += 1;
                if *seen <= 2 {
                    faults.push(format!(
                        "{} {what}: {} {} {at}: {why}",
                        kind.name,
                        pack.manifest().header.name,
                        pack.id().version
                    ));
                }
            }
        }
        out.push(
            writer
                .into_pack(PackOrigin::Package)
                .unwrap_or_else(|error| panic!("{:?}: {error:?}", pack.id())),
        );
    }
    Rewritten {
        counts,
        packs: out,
        faults,
    }
}

/// The document types of the rows, for the test that the table is the writer suite's.
pub fn kind_names() -> Vec<&'static str> {
    rows::rows()
        .iter()
        .flat_map(|row| row.kinds.iter().map(|kind| kind.name))
        .collect()
}

// ---- the checks of a tree -------------------------------------------------------------------

/// One world at a time in this file too: the packs of a client install are a gigabyte in memory.
static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The copy check: every pack of the tree goes through `PackWriter::from_pack` and
/// `into_pack` with its identity, its files and their bytes ([`copy`]; an encrypted pack is
/// refused and counted), and the world of the copies, loaded on rayon, holds what the tree's
/// own load holds: the same pin files as the load check. Its link pass, with every file
/// resident, is pinned beside.
pub fn check_copy(tree: Tree) {
    let Some(root) = crate::support::root_or_skip(&tree) else {
        return;
    };
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let started = Instant::now();
    let copied = copy(
        &tree,
        packs(&tree, &root),
        &pack_folders(&tree, &root),
        &load_options(&tree, &root, Run::SERVER),
    );
    eprintln!("{}: copied in {:.1?}", tree.name, started.elapsed());
    crate::support::assert_pinned(
        &format!("{}.copy", tree.name),
        &copied.counts,
        crate::support::no_key,
    );
    let world = facts_of(tree, &root, copied.packs, cfg!(feature = "rayon"));
    crate::support::dump(&format!("{}.copied", tree.name), &world);
    eprintln!(
        "{}: the copies loaded in {:.1?}, link() {:.1?}",
        tree.name, world.wall.0, world.wall.1
    );
    crate::support::assert_pinned(
        &format!("{}.errors", tree.name),
        &world.error_counts,
        crate::support::no_key,
    );
    crate::support::assert_pinned(
        &format!("{}.documents", tree.name),
        &world.documents,
        crate::support::no_key,
    );
    crate::support::assert_pinned(
        &format!("{}.load", tree.name),
        &world.load,
        crate::support::no_key,
    );
    assert!(
        world.link_errors.is_empty(),
        "{}: errors of the link pass over the copies:\n{}",
        tree.name,
        world.link_errors.join("\n")
    );
    crate::support::assert_pinned(
        &format!("{}.link_resident", tree.name),
        &world.link,
        crate::support::no_key,
    );
}

/// Whether a key of the rewrite's counts needs a sentence: a document that did not come back
/// as the value written.
fn is_a_fault(key: &str) -> bool {
    !(key.ends_with(" written")
        || key.ends_with(" written at a newer version than read")
        || key.ends_with(" kept as their bytes")
        || key.ends_with(" files"))
}

/// The rewrite check: every typed document of every pack is put back through
/// `PackWriter::put` ([`rewrite`]); per document type the counts are pinned, and a count of
/// documents that did not come back as the value written needs its sentence. The world of
/// the rewritten packs holds the documents of the tree's own load (the pin file of the load
/// check), has the known errors alone, and its findings are pinned.
pub fn check_rewrite(tree: Tree) {
    let Some(root) = crate::support::root_or_skip(&tree) else {
        return;
    };
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let started = Instant::now();
    let rewritten = rewrite(
        packs(&tree, &root),
        &load_options(&tree, &root, Run::SERVER),
    );
    eprintln!("{}: rewritten in {:.1?}", tree.name, started.elapsed());
    for fault in &rewritten.faults {
        eprintln!("  {fault}");
    }
    crate::support::assert_pinned(
        &format!("{}.rewrite", tree.name),
        &rewritten.counts,
        is_a_fault,
    );
    let world = facts_of(tree, &root, rewritten.packs, false);
    eprintln!(
        "{}: the rewritten packs loaded in {:.1?}, link() {:.1?}",
        tree.name, world.wall.0, world.wall.1
    );
    crate::support::assert_pinned(
        &format!("{}.errors", tree.name),
        &world.error_counts,
        crate::support::no_key,
    );
    crate::support::assert_pinned(
        &format!("{}.documents", tree.name),
        &world.documents,
        crate::support::no_key,
    );
    crate::support::assert_pinned(
        &format!("{}.load_rewritten", tree.name),
        &world.load,
        crate::support::no_key,
    );
    assert!(
        world.link_errors.is_empty(),
        "{}: errors of the link pass over the rewritten packs:\n{}",
        tree.name,
        world.link_errors.join("\n")
    );
    crate::support::assert_pinned(
        &format!("{}.link_rewritten", tree.name),
        &world.link,
        crate::support::no_key,
    );
}
