//! Packs built in memory for the persona-piece tests (the rows of fixtures.md are built here,
//! not kept on disk), and the reading of a load's findings as short lines.

#![allow(dead_code, reason = "each test file uses part of the helpers")]

use bedrock_addon::diagnostic::{Diagnostic, Severity};
use bedrock_addon::families::persona::{BUILTIN_PACK, PersonaPiece, PersonaPieces, PieceSource};
use bedrock_addon::pack::{OpenOptions, Pack, PackIdVersion};
use bedrock_addon::storage::PackPath;
use bytes::Bytes;
use uuid::Uuid;

/// The header UUID of the `persona_piece` packs the tests build.
pub const PIECE_PACK: Uuid = Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0051);
/// The header UUID of the ordinary resource packs the tests build.
pub const ORDINARY_PACK: Uuid = Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0052);

/// A `Pack` as the load asks for its files.
pub struct OfPack<'a>(pub &'a Pack);

impl PieceSource for OfPack<'_> {
    fn id(&self) -> &PackIdVersion {
        self.0.id()
    }

    fn name(&self) -> &'static str {
        "the pack"
    }

    fn paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        self.0.paths()
    }

    fn read(&self, path: &PackPath) -> Result<Bytes, std::io::ErrorKind> {
        self.0.read(path).map_err(|_| std::io::ErrorKind::NotFound)
    }
}

/// A `Pack` whose files are listed last first.
pub struct Reversed<'a>(pub &'a Pack);

impl PieceSource for Reversed<'_> {
    fn id(&self) -> &PackIdVersion {
        self.0.id()
    }

    fn name(&self) -> &'static str {
        "the pack"
    }

    fn paths(&self) -> impl Iterator<Item = &PackPath> + '_ {
        let mut paths: Vec<&PackPath> = self.0.paths().collect();
        paths.reverse();
        paths.into_iter()
    }

    fn read(&self, path: &PackPath) -> Result<Bytes, std::io::ErrorKind> {
        self.0.read(path).map_err(|_| std::io::ErrorKind::NotFound)
    }
}

/// A format-2 manifest with one module of `module` type.
pub fn manifest(uuid: Uuid, module: &str, min_engine: Option<&str>) -> String {
    let min_engine = min_engine
        .map(|version| format!(r#", "min_engine_version": {version}"#))
        .unwrap_or_default();
    format!(
        r#"{{
  "format_version": 2,
  "header": {{ "name": "pieces", "description": "", "uuid": "{uuid}", "version": [1, 0, 0]{min_engine} }},
  "modules": [{{ "type": "{module}", "uuid": "{}", "version": [1, 0, 0] }}]
}}"#,
        Uuid::from_u128(uuid.as_u128() ^ 0xabcd)
    )
}

/// A pack opened from memory: `manifest_json` and `files`.
pub fn open(manifest_json: &str, files: &[(&str, &str)]) -> Pack {
    let mut all: Vec<(PackPath, Bytes)> = vec![(
        PackPath::from_static("manifest.json"),
        Bytes::from(manifest_json.to_owned()),
    )];
    for (path, text) in files {
        all.push((
            PackPath::new(path).expect("a pack path"),
            Bytes::from((*text).to_owned()),
        ));
    }
    Pack::from_files(all, &OpenOptions::default()).expect("the pack opens")
}

/// A `persona_piece` pack (a Marketplace piece) with `files`.
pub fn piece_pack(files: &[(&str, &str)]) -> Pack {
    open(&manifest(PIECE_PACK, "persona_piece", None), files)
}

/// The client's built-in persona pack (a `resources` pack with its UUID) with `files`.
pub fn builtin_pack(files: &[(&str, &str)]) -> Pack {
    open(
        &manifest(BUILTIN_PACK, "resources", Some("[1, 14, 0]")),
        files,
    )
}

/// An ordinary resource pack with `files`.
pub fn ordinary_pack(files: &[(&str, &str)]) -> Pack {
    open(
        &manifest(ORDINARY_PACK, "resources", Some("[1, 14, 0]")),
        files,
    )
}

/// The pieces of `pack`.
pub fn load(pack: &Pack) -> PersonaPieces {
    PersonaPiece::load_all(&OfPack(pack))
}

/// A `persona_piece` pack with the one piece `pieces/hat/hat.meta.json` = `meta` and `files`
/// beside it (paths relative to `pieces/hat/`), loaded.
pub fn hat(meta: &str, files: &[(&str, &str)]) -> PersonaPieces {
    let mut all = vec![
        ("pieces/hat/hat.meta.json".to_owned(), meta.to_owned()),
        // The pack-wide name of the piece.
        (
            "texts/en_US.lang".to_owned(),
            "persona.hat.title=Hat\n".to_owned(),
        ),
    ];
    for (path, text) in files {
        all.push((format!("pieces/hat/{path}"), (*text).to_owned()));
    }
    let all: Vec<(&str, &str)> = all
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    load(&piece_pack(&all))
}

/// The one piece of a load that has exactly one.
pub fn only(pieces: &PersonaPieces) -> &PersonaPiece {
    assert_eq!(pieces.pieces.len(), 1, "{:#?}", lines(pieces));
    &pieces.pieces[0]
}

/// The variant name of a finding's code.
pub fn code_name(found: &Diagnostic) -> String {
    let debug = format!("{:?}", found.code);
    debug
        .split(|c: char| !c.is_ascii_alphanumeric())
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// One finding as `<severity> <Code> <file> <json path>`.
pub fn line(found: &Diagnostic) -> String {
    let severity = match found.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Debug => "other",
    };
    let file = found
        .location
        .path
        .as_ref()
        .map(|path| path.as_str().to_owned())
        .unwrap_or_default();
    format!(
        "{severity} {} {file} {}",
        code_name(found),
        found.location.json_path
    )
    .trim_end()
    .to_owned()
}

/// Every finding of a load, one line each, in report order.
pub fn lines(pieces: &PersonaPieces) -> Vec<String> {
    pieces.report().map(line).collect()
}

/// The text the game prints for a finding (the code's `Display`).
pub fn message(found: &Diagnostic) -> String {
    found.code.to_string()
}

/// Whether two values are the same JSON, members compared by key and numbers by value.
pub fn same_value(a: &bedrock_addon::json::Value, b: &bedrock_addon::json::Value) -> bool {
    if let (Some(a), Some(b)) = (a.as_object(), b.as_object()) {
        return a.len() == b.len()
            && a.iter().all(|(key, value)| {
                b.get(key.as_str())
                    .is_some_and(|other| same_value(value, other))
            });
    }
    if let (Some(a), Some(b)) = (a.as_array(), b.as_array()) {
        return a.len() == b.len()
            && a.as_slice()
                .iter()
                .zip(b.as_slice())
                .all(|(a, b)| same_value(a, b));
    }
    if let (Some(a), Some(b)) = (a.as_number(), b.as_number()) {
        return a.to_f64_lossy().to_bits() == b.to_f64_lossy().to_bits();
    }
    a == b
}

/// Whether two JSON texts hold the same JSON ([`same_value`]).
pub fn same_json(a: &str, b: &str) -> bool {
    let read = |text: &str| {
        bedrock_addon::json::from_str_jsonc::<bedrock_addon::json::Value>(text)
            .unwrap_or_else(|e| panic!("{text}: {e}"))
    };
    same_value(&read(a), &read(b))
}

/// The error findings of the one meta file `pieces` rejects, each as a [`line`] with the file
/// `file` cut off. The closing M2 line ("Could not parse metafile json file") is checked and
/// left out.
pub fn refused_at(pieces: &PersonaPieces, file: &str) -> Vec<String> {
    assert!(
        pieces.pieces.is_empty(),
        "a piece loaded: {:#?}",
        lines(pieces)
    );
    assert_eq!(pieces.rejected.len(), 1, "{:#?}", lines(pieces));
    let diagnostics: Vec<&Diagnostic> = pieces.rejected[0].diagnostics.iter().collect();
    let last = diagnostics.last().expect("a finding");
    assert_eq!(line(last), format!("error MalformedInput {file}"));
    assert_eq!(
        message(last),
        format!("Could not parse metafile json file {file}")
    );
    diagnostics[..diagnostics.len() - 1]
        .iter()
        .filter(|found| found.severity == Severity::Error)
        .map(|found| line(found).replace(&format!(" {file}"), ""))
        .collect()
}
