//! Text smoke over the game's own files: the `texts/` folder of every behaviour and resource
//! pack of the sample packs, the server install and the client installs loads through `Driver`
//! into `languages()` and `translations(&language)` with no error-level diagnostic, and every
//! `.lang` file on disk is written back to text that reads to the same lines and is written
//! again byte for byte. MCB documents (baked documents the loaders do not decode) are counted
//! apart.
//!
//! Run with `BEDROCK_ADDON_CORPUS=<corpus> cargo nextest run -p bedrock_addon_std --test smoke_text`;
//! without the variable every test skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[macro_use]
mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::families::lang::{LangFile, Language};
use bedrock_addon::load::{LoadOptions, StackOptions};
use bedrock_addon::pack::{OpenOptions, PackOrigin, PackStack, Side, StackSet};
use bedrock_addon_std::{Driver, Options};

use common::game_files;

/// Which stack a pack directory goes to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Behavior,
    Resource,
}

/// The stack set of the base-game pack `dir` alone, on the side of `kind`, loaded by the driver
/// that opened it.
fn load(dir: &Path, kind: Kind) -> StackSet {
    let mut driver = Driver::new(Options::default().with_open(OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    }))
    .unwrap();
    let pack = driver
        .pack_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e:?}", dir.display()));
    let options = LoadOptions::default();
    let set = match kind {
        Kind::Behavior => StackSet {
            behavior: PackStack::of([pack], options.clone()).unwrap(),
            resource: PackStack::of([], options).unwrap(),
        },
        Kind::Resource => StackSet {
            behavior: PackStack::of([], options.clone()).unwrap(),
            resource: PackStack::of([pack], options).unwrap(),
        },
    };
    driver.load_stacks(set, StackOptions::default()).unwrap()
}

/// What the packs of one source gave.
#[derive(Default)]
struct Tally {
    packs: usize,
    codes: usize,
    names: usize,
    keys: usize,
    keywords: usize,
    errors: Vec<String>,
    mcb: usize,
    warnings: BTreeMap<&'static str, usize>,
    infos: BTreeMap<&'static str, usize>,
}

impl Tally {
    /// Adds one finding of the pack at `dir`, whose `texts/` holds `on_disk` MCB documents.
    fn add(&mut self, dir: &Path, on_disk: usize, seen: &mut usize, diagnostic: &Diagnostic) {
        if diagnostic.code.id() == "core/unsupported_file_format" && *seen < on_disk {
            *seen += 1;
            self.mcb += 1;
        } else if diagnostic.severity >= Severity::Error {
            self.errors.push(format!("{}: {diagnostic}", dir.display()));
        } else if diagnostic.severity == Severity::Warning {
            *self.warnings.entry(diagnostic.code.id()).or_insert(0) += 1;
        } else if diagnostic.severity == Severity::Info {
            *self.infos.entry(diagnostic.code.id()).or_insert(0) += 1;
        }
    }

    /// Adds what the stack of the one pack at `dir` loaded of its `texts/`.
    fn add_stack<S: Side>(&mut self, dir: &Path, stack: &PackStack<S>) {
        let on_disk = game_files::mcb_on_disk(&dir.join("texts")).total();
        let mut seen = 0;
        let languages = stack.languages();
        self.packs += 1;
        self.codes += languages.codes.len();
        self.names += languages.names.entries.len();
        for diagnostic in languages.diagnostics() {
            self.add(dir, on_disk, &mut seen, diagnostic);
        }
        for code in &languages.codes {
            let table = stack.translations(code).expect("a listed code has a table");
            let keywords = table
                .strings
                .keys()
                .filter(|key| key.ends_with("pack.name") || key.ends_with("pack.description"))
                .count();
            self.keywords += keywords;
            self.keys += table.strings.len() - keywords;
            for diagnostic in table.diagnostics() {
                self.add(dir, on_disk, &mut seen, diagnostic);
            }
        }
        // what the loader itself found about the files below `texts/`
        for diagnostic in stack.report().iter() {
            let in_texts = diagnostic
                .location
                .path
                .as_ref()
                .is_some_and(|path| path.as_str().to_ascii_lowercase().starts_with("texts/"));
            if in_texts && diagnostic.code.id() == "core/unsupported_file_format" {
                self.add(dir, on_disk, &mut seen, diagnostic);
            }
        }
    }
}

/// The pack folders directly below `packs` that have a `texts/` folder.
fn packs_with_texts(packs: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(packs) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.join("manifest.json").is_file() && path.join("texts").is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// Loads every pack in `dirs` alone on the side of `kind` and requires no error, and the
/// warnings by code to be `warnings`, exactly.
fn assert_loads(source: &str, dirs: &[PathBuf], kind: Kind, warnings: &[(&str, usize)]) -> Tally {
    let mut tally = Tally::default();
    for dir in dirs {
        let set = load(dir, kind);
        match kind {
            Kind::Behavior => tally.add_stack(dir, &set.behavior),
            Kind::Resource => tally.add_stack(dir, &set.resource),
        }
    }
    eprintln!(
        "{source}: {} packs, {} listed codes, {} names, {} keys, {} pack keywords, {} errors, {} MCB documents, warnings {:?}, infos {:?}",
        tally.packs,
        tally.codes,
        tally.names,
        tally.keys,
        tally.keywords,
        tally.errors.len(),
        tally.mcb,
        tally.warnings,
        tally.infos,
    );
    for error in tally.errors.iter().take(30) {
        eprintln!("{source}: {error}");
    }
    assert!(
        tally.errors.is_empty(),
        "{source}: {} errors",
        tally.errors.len()
    );
    game_files::assert_warnings(source, &tally.warnings, warnings);
    tally
}

/// Every `.lang` file directly in the `texts/` folder of the packs in `dirs`.
fn lang_files(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = dirs
        .iter()
        .filter_map(|dir| std::fs::read_dir(dir.join("texts")).ok())
        .flat_map(|entries| entries.filter_map(|entry| entry.ok().map(|entry| entry.path())))
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "lang")
        })
        .collect();
    files.sort();
    files
}

/// Every `.lang` file of `dirs` is written back to its own bytes with CRLF as LF (the one
/// loss of the round-trip level Lines: the byte order mark and a missing final line feed are
/// kept), and the written text reads to the same file and is written again byte for byte.
fn assert_round_trips(source: &str, dirs: &[PathBuf]) {
    let files = lang_files(dirs);
    let (mut lines, mut with_bom, mut with_cr, mut unchanged) = (0, 0, 0, 0);
    let mut without_final_lf = 0;
    for path in &files {
        let bytes = std::fs::read(path).unwrap();
        with_bom += usize::from(bytes.starts_with(b"\xEF\xBB\xBF"));
        with_cr += usize::from(bytes.contains(&b'\r'));
        let language = Language::new(path.file_stem().unwrap().to_string_lossy());
        let (file, _) = LangFile::from_bytes(language.clone(), &bytes);
        lines += file.lines.len();
        let written = file
            .to_text()
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        unchanged += usize::from(written.as_bytes() == bytes);
        // the one loss of the Lines level: CRLF is written as LF
        let mut expected = Vec::with_capacity(bytes.len());
        for (index, byte) in bytes.iter().enumerate() {
            if !(*byte == b'\r' && bytes.get(index + 1) == Some(&b'\n')) {
                expected.push(*byte);
            }
        }
        without_final_lf += usize::from(!bytes.ends_with(b"\n"));
        assert!(
            written.as_bytes() == expected,
            "{}: the written text differs from the input beyond CRLF",
            path.display()
        );
        let (again, _) = LangFile::from_bytes(language, written.as_bytes());
        assert_eq!(
            again,
            file,
            "{}: the written text reads differently",
            path.display()
        );
        assert_eq!(
            again.translations(),
            file.translations(),
            "{}",
            path.display()
        );
        assert_eq!(
            again.to_text().unwrap(),
            written,
            "{}: the second write differs",
            path.display()
        );
    }
    eprintln!(
        "{source}: {} lang files round trip ({lines} lines; {with_bom} with a BOM, {with_cr} with CR, {without_final_lf} without a final line feed; all byte-identical after CRLF -> LF, {unchanged} byte-identical outright)",
        files.len()
    );
    assert!(!files.is_empty(), "{source}: no lang file found");
}

#[test]
fn sample_packs_resource_pack_texts_load() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    let dir = samples.join("resource_pack");
    let tally = assert_loads(
        "sample packs resource_pack",
        std::slice::from_ref(&dir),
        Kind::Resource,
        &[],
    );
    // A key that ends with a space is an information of the crate's own (`LangKeyTrailingSpace`):
    // the game stores such a key as written, and vanilla writes the space on purpose.
    assert_eq!(tally.infos.get("core/lang_key_trailing_space"), Some(&783));
    // 29 language codes with `en_US` first, 29 name pairs
    assert_eq!(tally.codes, 29);
    assert_eq!(tally.names, 29);
    let set = load(&dir, Kind::Resource);
    let languages = set.resource.languages();
    assert_eq!(languages.base(), Some(&Language::new("en_US")));
    assert_eq!(
        languages.name(&Language::new("en_US")),
        Some("English (United States)")
    );
    let english = set.resource.translations(&Language::new("en_US")).unwrap();
    assert!(
        english.strings.len() > 10_000,
        "{} keys",
        english.strings.len()
    );
    // stored lower-cased, found whatever the case
    assert_eq!(english.get("item.apple.name"), Some("Apple"));
    assert_eq!(english.get("Item.Apple.Name"), Some("Apple"));
    let german = set.resource.translations(&Language::new("de_DE")).unwrap();
    assert_eq!(german.get("item.apple.name"), Some("Apfel"));
    assert!(set.resource.translations(&Language::new("xx_XX")).is_none());
}

#[test]
fn sample_packs_behavior_pack_texts_load() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    // The sample packs ship no `behavior_pack/texts/`: the stack has no language
    let set = load(&samples.join("behavior_pack"), Kind::Behavior);
    let languages = set.behavior.languages();
    let errors: Vec<String> = languages
        .diagnostics()
        .filter(|d| d.severity >= Severity::Error)
        .map(ToString::to_string)
        .collect();
    eprintln!(
        "sample packs behavior_pack: {} listed codes, {} errors",
        languages.codes.len(),
        errors.len()
    );
    assert!(errors.is_empty(), "{errors:#?}");
    game_files::assert_warnings(
        "sample packs behavior_pack",
        &game_files::warnings_by_code(languages.diagnostics()),
        &[],
    );
    assert!(
        set.behavior
            .translations(&Language::new("en_US"))
            .is_none_or(|table| table.diagnostics().all(|d| d.severity < Severity::Error))
    );
}

#[test]
fn sample_packs_lang_files_round_trip() {
    let samples = skip!(game_files::SAMPLE_PACKS);
    assert_round_trips("sample packs", &[samples.join("resource_pack")]);
}

#[test]
fn server_install_texts_load() {
    let server = skip!(game_files::SERVER_INSTALL);
    let resource = packs_with_texts(&server.join("resource_packs"));
    let behavior = packs_with_texts(&server.join("behavior_packs"));
    let tally = assert_loads(
        "server install resource_packs",
        &resource,
        Kind::Resource,
        // The crate's own language-list findings are informations: the game says nothing
        // about them.
        &[],
    );
    assert!(
        tally.packs > 0 && tally.keys > 10_000,
        "{} keys",
        tally.keys
    );
    assert_loads(
        "server install behavior_packs",
        &behavior,
        Kind::Behavior,
        &[],
    );
}

#[test]
fn server_install_lang_files_round_trip() {
    let server = skip!(game_files::SERVER_INSTALL);
    let mut dirs = packs_with_texts(&server.join("resource_packs"));
    dirs.extend(packs_with_texts(&server.join("behavior_packs")));
    assert_round_trips("server install", &dirs);
}

#[test]
fn client_install_b_texts_load() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let tally = assert_loads(
        "client install B resource_packs",
        &packs_with_texts(&client.join("resource_packs")),
        Kind::Resource,
        &[], // the crate's own language-list findings are informations
    );
    assert!(
        tally.packs > 0 && tally.keys > 10_000,
        "{} keys",
        tally.keys
    );
    assert_loads(
        "client install B behavior_packs",
        &packs_with_texts(&client.join("behavior_packs")),
        Kind::Behavior,
        &[],
    );
}

#[test]
fn client_install_a_texts_load() {
    let client = skip!(game_files::CLIENT_INSTALL_A);
    let tally = assert_loads(
        "client install A resource_packs",
        &packs_with_texts(&client.join("resource_packs")),
        Kind::Resource,
        &[], // the crate's own language-list findings are informations
    );
    assert!(
        tally.packs > 0 && tally.keys > 10_000,
        "{} keys",
        tally.keys
    );
    assert_loads(
        "client install A behavior_packs",
        &packs_with_texts(&client.join("behavior_packs")),
        Kind::Behavior,
        &[],
    );
}

#[test]
fn client_lang_files_round_trip() {
    let install_b = skip!(game_files::CLIENT_INSTALL_B);
    let install_a = skip!(game_files::CLIENT_INSTALL_A);
    for (source, client) in [
        ("client install B", install_b),
        ("client install A", install_a),
    ] {
        let mut dirs = packs_with_texts(&client.join("resource_packs"));
        dirs.extend(packs_with_texts(&client.join("behavior_packs")));
        assert_round_trips(source, &dirs);
    }
}

/// The two base packs of client install B as one resource stack, `vanilla` above
/// `vanilla_base` as the game orders them: the union of the language lists with `en_US` first,
/// and the higher pack's name and values winning.
#[test]
fn client_install_b_vanilla_stack_merges_its_texts() {
    let client = skip!(game_files::CLIENT_INSTALL_B);
    let mut driver = Driver::new(Options::default().with_open(OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    }))
    .unwrap();
    let mut open = |name: &str| {
        let dir = client.join("resource_packs").join(name);
        driver
            .pack_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e:?}", dir.display()))
    };
    let (vanilla, vanilla_base) = (open("vanilla"), open("vanilla_base"));
    let options = LoadOptions::default();
    let set = StackSet {
        behavior: PackStack::of([], options.clone()).unwrap(),
        resource: PackStack::of([vanilla, vanilla_base], options).unwrap(),
    };
    let stacks = driver.load_stacks(set, StackOptions::default()).unwrap();
    let languages = stacks.resource.languages();
    let errors: Vec<String> = languages
        .diagnostics()
        .filter(|d| d.severity >= Severity::Error)
        .map(ToString::to_string)
        .collect();
    let english = stacks
        .resource
        .translations(&Language::new("en_US"))
        .expect("en_US is listed");
    eprintln!(
        "client install B vanilla over vanilla_base: {} codes, {} names, {} language layers, {} en_US keys in {} layers, {} errors",
        languages.codes.len(),
        languages.names.entries.len(),
        languages.layers().len(),
        english.strings.len(),
        english.layers().len(),
        errors.len()
    );
    assert!(errors.is_empty(), "{errors:#?}");
    game_files::assert_warnings(
        "client install B vanilla over vanilla_base",
        &game_files::warnings_by_code(languages.diagnostics().chain(english.diagnostics())),
        // the 27 trailing-space findings are informations
        &[],
    );
    // `vanilla_base` lists `en_US` alone, `vanilla` 29 codes
    assert_eq!(languages.base(), Some(&Language::new("en_US")));
    assert_eq!(languages.codes.len(), 29);
    assert_eq!(languages.layers().len(), 4);
    // the code's origin is the pack that listed it first: the lower one
    assert_eq!(
        languages
            .origin(&Language::new("en_US"))
            .and_then(|origin| origin.pack.index()),
        Some(1)
    );
    // `vanilla_base` names it `English (US)`; the higher `vanilla` pack's name wins
    assert_eq!(
        languages.name(&Language::new("en_US")),
        Some("English (United States)")
    );
    assert!(
        english.strings.len() > 10_000,
        "{} keys",
        english.strings.len()
    );
    assert_eq!(english.layers().len(), 2);
    assert!(english.diagnostics().all(|d| d.severity < Severity::Error));
}
