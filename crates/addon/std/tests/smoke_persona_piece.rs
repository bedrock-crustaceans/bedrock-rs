//! Persona piece smoke over the game's own files: the client's built-in persona pack of each
//! client install (`resource_packs/persona/`) opens through `Driver`, its 66 pieces load with no
//! error-level finding, the findings are exactly the pinned ones, every meta and store-info
//! file is written back as it was read, and the load asks for the bytes of no image. The sample
//! packs and the server install ship no persona pack.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test
//! smoke_persona_piece`; without the variable every test skips.

#[macro_use]
mod common;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::families::persona::{
    BUILTIN_PACK, PackKind, PersonaPiece, PersonaPieces, PieceSource, PieceType,
};
use bedrock_addon::json::Value;
use bedrock_addon::pack::{Pack, PackIdVersion, PackType};
use bedrock_addon::storage::PackPath;
use bedrock_addon_std::{Options, load_dir};
use bytes::Bytes;

use common::game_files;

/// The persona pack below a client install's `data/` directory.
const PERSONA: &str = "resource_packs/persona";

/// A pack opened through the driver, whose loose files are read from disk on demand. The
/// `__brarchive/` copies are left alone: only the loose files are read.
struct OnDisk {
    pack: Pack,
    root: PathBuf,
    read: RefCell<Vec<String>>,
}

impl OnDisk {
    fn open(root: &Path) -> OnDisk {
        let pack = load_dir(root, &Options::default())
            .unwrap_or_else(|error| panic!("{}: {error}", root.display()));
        OnDisk {
            pack,
            root: root.to_owned(),
            read: RefCell::new(Vec::new()),
        }
    }
}

impl PieceSource for OnDisk {
    fn id(&self) -> &PackIdVersion {
        self.pack.id()
    }

    fn name(&self) -> &'static str {
        "persona"
    }

    fn paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        self.pack
            .paths()
            .filter(|path| !path.as_str().starts_with("__brarchive/"))
    }

    fn read(&self, path: &PackPath) -> Result<Bytes, std::io::ErrorKind> {
        self.read.borrow_mut().push(path.as_str().to_owned());
        std::fs::read(self.root.join(path.as_str()))
            .map(Bytes::from)
            .map_err(|error| error.kind())
    }
}

/// The variant name of a finding's code.
fn code_name(found: &Diagnostic) -> String {
    found.code.id().to_owned()
}

/// The findings of a load counted by severity and code.
fn by_code(pieces: &PersonaPieces) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for found in pieces.report() {
        let severity = match found.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            _ => "info",
        };
        *counts
            .entry(format!("{severity} {}", code_name(found)))
            .or_insert(0) += 1;
    }
    counts
}

/// Whether two values are the same JSON, members compared by key.
#[expect(
    clippy::float_cmp,
    reason = "two numbers are the same JSON only when they read to the same value"
)]
fn same(a: &Value, b: &Value) -> bool {
    if let (Some(a), Some(b)) = (a.as_object(), b.as_object()) {
        return a.len() == b.len()
            && a.iter()
                .all(|(key, value)| b.get(key.as_str()).is_some_and(|other| same(value, other)));
    }
    if let (Some(a), Some(b)) = (a.as_array(), b.as_array()) {
        return a.len() == b.len()
            && a.as_slice()
                .iter()
                .zip(b.as_slice())
                .all(|(a, b)| same(a, b));
    }
    if let (Some(a), Some(b)) = (a.as_number(), b.as_number()) {
        return a.to_f64_lossy() == b.to_f64_lossy();
    }
    a == b
}

/// A file of the pack as a JSON value.
fn json_of(root: &Path, path: &PackPath) -> Value {
    let bytes = std::fs::read(root.join(path.as_str())).expect("the file");
    let text = String::from_utf8_lossy(&bytes);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    bedrock_addon::json::from_str_jsonc::<Value>(text)
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn counts(entries: &[(&str, usize)]) -> BTreeMap<String, usize> {
    entries
        .iter()
        .map(|(code, count)| ((*code).to_owned(), *count))
        .collect()
}

/// Loads the built-in persona pack of the install `entry` and pins what it holds.
#[expect(
    clippy::too_many_lines,
    reason = "one load of the pack, with the findings pinned after each step"
)]
fn builtin_pack_loads(entry: &str, data: &Path) {
    let root = data.join(PERSONA);
    if !root.is_dir() {
        eprintln!("skipped: {} has no {PERSONA}", data.display());
        return;
    }
    let on_disk: usize = walkdir(&root)
        .iter()
        .filter(|path| {
            path.to_string_lossy().ends_with(".meta.json")
                && !path.to_string_lossy().contains("__brarchive")
        })
        .count();
    let pack = OnDisk::open(&root);
    assert_eq!(pack.pack.id().uuid, BUILTIN_PACK);
    assert_eq!(pack.pack.pack_type(), PackType::Resource);

    // Without the vanilla resource pack's names.
    let alone = PersonaPiece::load_all(&pack);
    assert_eq!(alone.kind, PackKind::BuiltIn);
    assert_eq!(alone.pieces.len(), 66, "{entry}");
    assert_eq!(
        alone.pieces.len(),
        on_disk,
        "{entry}: every meta file on disk"
    );
    assert!(alone.rejected.is_empty(), "{entry}: {:?}", alone.rejected);
    assert_eq!(
        alone.kept().count(),
        66,
        "{entry}: every piece_id is distinct"
    );
    let in_body = alone
        .pieces
        .iter()
        .filter(|piece| piece.directory.as_str().starts_with("pieces/body/"))
        .count();
    assert_eq!((alone.pieces.len() - in_body, in_body), (58, 8), "{entry}");
    let types: BTreeSet<PieceType> = alone.pieces.iter().map(PersonaPiece::piece_type).collect();
    assert_eq!(types.len(), 15, "{entry}: {types:?}");
    assert!(!types.contains(&PieceType::Unknown));
    assert_eq!(
        by_code(&alone),
        counts(&[
            // The four bodies and the four skeletons are named by no text of this pack.
            // (`is_face: "true"`, two sources in each body piece, is the string 1.26.40 types
            // the member as: no finding.)
            ("info core/unresolved_lang_key", 8),
        ]),
        "{entry}"
    );

    // With them: the bodies are named there, the skeletons nowhere.
    let lang = std::fs::read(data.join("resource_packs/vanilla/texts/en_US.lang"))
        .expect("the vanilla resource pack's en_US.lang");
    pack.read.borrow_mut().clear();
    let named = PersonaPiece::load_all_with_texts(&pack, &[&lang]);
    assert_eq!(
        by_code(&named),
        counts(&[("info core/unresolved_lang_key", 4)]),
        "{entry}"
    );
    let unnamed: BTreeSet<String> = named
        .pieces
        .iter()
        .filter(|piece| {
            piece
                .findings()
                .iter()
                .any(|found| code_name(found) == "core/unresolved_lang_key")
        })
        .map(PersonaPiece::name_key)
        .collect();
    let skeletons: BTreeSet<String> = ["smaller", "small", "medium", "tall"]
        .iter()
        .map(|size| format!("persona.{size}_skeleton.title"))
        .collect();
    assert_eq!(unnamed, skeletons, "{entry}");
    assert!(
        named
            .report()
            .all(|found| found.severity < Severity::Warning)
    );

    // The four emotes are named by their own texts, and their animations are kept.
    let emotes: Vec<&PersonaPiece> = named
        .pieces
        .iter()
        .filter(|piece| piece.piece_type() == PieceType::Emote)
        .collect();
    assert_eq!(emotes.len(), 4, "{entry}");
    for emote in &emotes {
        assert!(emote.offer_title.is_some(), "{}", emote.directory);
        assert_eq!(emote.animations.len(), 1, "{}", emote.directory);
        assert_eq!(
            emote.animations[0].stored_name,
            format!("{}.{}", emote.animations[0].name, emote.id)
        );
        assert!(emote.emote.is_some(), "{}", emote.directory);
    }
    // Geometry: the eight body and skeleton pieces, every source's model found.
    let with_geometry = named
        .pieces
        .iter()
        .filter(|piece| piece.geometry.is_some())
        .count();
    assert_eq!(with_geometry, 8, "{entry}");
    let with_store = named
        .pieces
        .iter()
        .filter(|piece| piece.store_info.is_some())
        .count();
    assert_eq!(with_store, 58, "{entry}");

    // Only JSON and `.lang` files were read: an image is known by its path.
    let read = pack.read.borrow();
    assert!(
        read.iter()
            .all(|path| has_extension(path, "json") || has_extension(path, "lang")),
        "{entry}: {:?}",
        read.iter()
            .filter(|path| !has_extension(path, "json") && !has_extension(path, "lang"))
            .collect::<Vec<_>>()
    );
    let metas = read
        .iter()
        .filter(|path| path.ends_with(".meta.json"))
        .count();
    assert_eq!(metas, 66, "{entry}: each meta file once");
    // Each file once: 66 meta files, 58 store infos, 8 geometry files, the 4 emotes'
    // animations, and 33 `.lang` files (the 4 emotes' own `en_US.lang` and the pack's 29).
    let mut once: Vec<&String> = read.iter().collect();
    once.sort_unstable();
    once.dedup();
    assert_eq!(once.len(), read.len(), "{entry}: no file is read twice");
    let ending = |suffix: &str| read.iter().filter(|path| path.ends_with(suffix)).count();
    assert_eq!(
        (
            ending(".meta.json"),
            ending(".store_info.json"),
            ending(".geometry.json"),
            ending(".animation.json"),
            ending(".lang"),
        ),
        (66, 58, 8, 4, 33),
        "{entry}"
    );
    assert_eq!(read.len(), 169, "{entry}: {read:?}");

    // Every meta and store-info file is written back as it was read.
    let mut written = 0;
    for piece in &named.pieces {
        let text = piece.meta.to_json().expect("a meta file");
        let path = &piece.meta.origin().path;
        assert!(
            same(
                &bedrock_addon::json::from_str::<Value>(&text).expect("JSON"),
                &json_of(&root, path)
            ),
            "{entry}: {path}"
        );
        written += 1;
        if let Some(store) = &piece.store_info {
            let text = store.to_json().expect("a store-info file");
            let path = &store.origin().path;
            assert!(
                same(
                    &bedrock_addon::json::from_str::<Value>(&text).expect("JSON"),
                    &json_of(&root, path)
                ),
                "{entry}: {path}"
            );
            written += 1;
        }
    }
    assert_eq!(written, 66 + 58, "{entry}");

    eprintln!(
        "{entry}: {} pieces ({} under pieces/, {in_body} under pieces/body/), {} types, {} with a store info, {with_geometry} with geometry, {} emotes; findings alone {:?}, with the vanilla names {:?}; {written} files written back",
        named.pieces.len(),
        named.pieces.len() - in_body,
        types.len(),
        with_store,
        emotes.len(),
        by_code(&alone),
        by_code(&named),
    );
}

/// Every file below `dir`.
fn walkdir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_owned()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files
}

#[test]
fn the_client_install_b_persona_pack_loads() {
    let data = skip!(game_files::CLIENT_INSTALL_B);
    builtin_pack_loads(game_files::CLIENT_INSTALL_B, &data);
}

#[test]
fn the_client_install_a_persona_pack_loads() {
    let data = skip!(game_files::CLIENT_INSTALL_A);
    builtin_pack_loads(game_files::CLIENT_INSTALL_A, &data);
}

#[test]
fn sample_packs_and_the_server_ship_no_persona_pack() {
    for (entry, folder) in [
        (game_files::SAMPLE_PACKS, "resource_pack/pieces"),
        (game_files::SERVER_INSTALL, "resource_packs/persona"),
    ] {
        let Some(root) = game_files::source(entry) else {
            eprintln!("skipped: {}", game_files::missing(entry));
            continue;
        };
        assert!(!root.join(folder).exists(), "{entry} has {folder}");
        eprintln!("{entry}: no {folder}; nothing to load");
    }
}

/// The eight `<name>.geometry.json` files of an install's persona pack through
/// `geometry::load_keyed_file`, the entry point a piece's model takes (a keyed model that
/// states 1.12.0): every file loads, states 1.12.0 and has the models counted here; what the
/// reader says about them is warnings at most. Returns, per file name, its models and its
/// warnings.
fn persona_geometry_loads(data: &Path) -> BTreeMap<String, (usize, usize)> {
    use bedrock_addon::families::geometry::load_keyed_file;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::version::FormatVersion;

    let root = data.join(PERSONA);
    let mut files: Vec<PathBuf> = walkdir(&root)
        .into_iter()
        .filter(|path| {
            let path = path.to_string_lossy();
            path.ends_with(".geometry.json") && !path.contains("__brarchive")
        })
        .collect();
    files.sort();
    let mut found = BTreeMap::new();
    let kinds: RefCell<Vec<String>> = RefCell::new(Vec::new());
    for file in files {
        let relative = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = std::fs::read(&file).unwrap();
        let path = PackPath::new(&relative).unwrap();
        let document = load_keyed_file(&path, &bytes, &LoadContext::detached())
            .unwrap_or_else(|rejected| panic!("{relative}: {:#?}", rejected.diagnostics));
        assert_eq!(
            document.format_version,
            FormatVersion::new(1, 12, 0),
            "{relative}"
        );
        assert_eq!(
            document.version().written,
            Some(FormatVersion::new(1, 12, 0)),
            "{relative}"
        );
        let above_warning: Vec<&Diagnostic> = document
            .diagnostics()
            .iter()
            .filter(|found| found.severity > Severity::Warning)
            .collect();
        assert!(above_warning.is_empty(), "{relative}: {above_warning:#?}");
        // The geometry family's own lints are informations (the game accepts these values
        // without a word); the pin counts them, and the `InvalidValue` informations beside them
        // are not counted.
        let warnings = document
            .diagnostics()
            .iter()
            .filter(|found| {
                found.severity == Severity::Info && found.code.id() != "geometry/bone_lower_cased"
            })
            .inspect(|found| kinds.borrow_mut().push(code_name(found)))
            .count();
        let name = relative.rsplit('/').next().unwrap().to_owned();
        found.insert(name, (document.geometries.len(), warnings));
    }
    let mut kinds = kinds.into_inner();
    kinds.sort();
    kinds.dedup();
    // What the keyed reader says of vanilla's own piece models, by code; none is above a
    // warning (asserted per file above).
    assert_eq!(
        kinds,
        [
            "core/unknown_child_option",
            "core/unrecognized_field_worded",
            "geometry/bone_parent_missing"
        ]
    );
    found
}

/// The eight persona geometry files, the same in both installs: each of the four body sizes has
/// 16 models and each of the four skeletons 13, with 31 and 22 warnings of the reader (their
/// codes: `persona_geometry_loads`).
const PERSONA_GEOMETRY: [(&str, (usize, usize)); 8] = [
    ("medium.geometry.json", (16, 31)),
    ("medium_skeleton.geometry.json", (13, 22)),
    ("small.geometry.json", (16, 31)),
    ("small_skeleton.geometry.json", (13, 22)),
    ("smaller.geometry.json", (16, 31)),
    ("smaller_skeleton.geometry.json", (13, 22)),
    ("tall.geometry.json", (16, 31)),
    ("tall_skeleton.geometry.json", (13, 22)),
];

#[test]
fn the_persona_geometry_of_both_installs_loads_through_the_keyed_door() {
    for entry in [game_files::CLIENT_INSTALL_B, game_files::CLIENT_INSTALL_A] {
        let Some(data) = game_files::source(entry) else {
            eprintln!("skipped: {}", game_files::missing(entry));
            continue;
        };
        let found = persona_geometry_loads(&data);
        eprintln!("{entry}: persona geometry (models, warnings) {found:?}");
        let found: Vec<(&str, (usize, usize))> = found
            .iter()
            .map(|(name, counts)| (name.as_str(), *counts))
            .collect();
        // 8 of 8 in each install: 16 of 16 where both are there.
        assert_eq!(found, PERSONA_GEOMETRY, "{entry}");
    }
}

/// Whether `name` ends in `.{ext}`, the extension compared as written.
fn has_extension(name: &str, ext: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(_, found)| found == ext)
}
