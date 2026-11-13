//! Packs built in memory for the persona-piece tests, and their pieces as a load reads them.

use bedrock_addon::families::persona::{BUILTIN_PACK, PersonaPiece, PersonaPieces, PieceSource};
use bedrock_addon::pack::{OpenOptions, Pack, PackIdVersion};
use bedrock_addon::storage::PackPath;
use bytes::Bytes;

use crate::common::{ORDINARY_PACK, PIECE_PACK, manifest};

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
