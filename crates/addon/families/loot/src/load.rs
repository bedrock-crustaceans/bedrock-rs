//! Loading loot tables: `Content`, the walk of `loot_tables/` and the combine rule.

use super::LootTable;
use super::parse::parse_table;
use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{
    Content, Document, Files, FilesBuilder, FilesRule, FiniteText, Origin, Rejected, VersionInfo,
};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{
    AppDataPlace, CollectionLoader, ListDepth, LoadContext, StackCx, Walk, WalkFile, WalkOrder,
};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};
use bedrock_addon_core::version::FormatVersion;

impl bedrock_addon_core::__private::Sealed for LootTable {}

impl Content for LootTable {
    const NAME: &'static str = "loot table";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(path: &PackPath, bytes: &[u8], cx: &LoadContext) -> Result<Document<Self>, Rejected> {
        parse_file(PackRef::app_data(), PackSide::Behavior, path, bytes, cx)
    }

    fn write_json(&self, out: &mut Vec<u8>, style: WriteStyle) -> Result<(), EncodeError> {
        self.write_into(out, style)
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.write_into(&mut out, WriteStyle::default())?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }
}

/// The version information of every loot table: vanilla reads no version, so
/// none is written, effective or parsed.
fn unversioned() -> VersionInfo {
    let none = FormatVersion::new(0, 0, 0);
    VersionInfo::new(None, false, none, none, false)
}

/// Loads one loot table file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<LootTable>, Rejected> {
    let origin = Origin::new(pack, side, path.clone(), None, Vec::new());
    // A number no double holds is read as the largest finite one, with a warning.
    let finite = FiniteText::of(bytes);
    let (table, mut diagnostics) = parse_table(path, finite.bytes(), cx);
    finite.settle(&mut diagnostics, None);
    locate(&mut diagnostics, &origin);
    match table {
        Some(table) => Ok(Document::new(table, origin, unversioned(), diagnostics)),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

/// Gives every finding the document's pack and file.
fn locate(diagnostics: &mut Diagnostics, origin: &Origin) {
    // In place: the list keeps the places of its lists for the link pass.
    for finding in diagnostics.iter_mut() {
        finding.location.pack = Some(origin.pack.clone());
        finding.location.path = Some(origin.path.clone());
    }
}

// ---- the collection ----------------------------------------------------------------------

/// A loot table file: a `.json` file below `loot_tables/`.
fn is_json(path: &PackPath) -> bool {
    path.as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "json")
}

/// `loot_tables/**/*.json`, packs bottom-up.
const WALK: Walk = Walk {
    directories: &["loot_tables"],
    depth: ListDepth::Recursive,
    accept: is_json,
    order: WalkOrder::BottomUp,
    directory_major: false,
    app_data: &[],
    app_data_place: AppDataPlace::After,
    in_pack: None,
    shadowed: None,
    read_error: None,
};

/// One file: its key (the path without a subpack folder) and the document or its rejection.
type FileCopy = (PackPath, Result<Document<LootTable>, Rejected>);

fn parse_walk_file(file: &WalkFile, cx: &LoadContext) -> FileCopy {
    let document = parse_file(file.pack.clone(), file.side, &file.path, &file.bytes, cx);
    (file.key.clone(), document)
}

/// The files: the top pack's copy of each path wins, and a copy that fails leaves the path
/// without a table (vanilla loads the one file the pack stack resolves the path to, and caches
/// nothing when it is invalid).
fn combine(parsed: Vec<FileCopy>, stack: &StackCx<'_>) -> Files<LootTable> {
    let mut builder = FilesBuilder::new(FilesRule::TopWins, stack.options);
    for (key, copy) in parsed {
        builder.push(key, copy);
    }
    builder.finish()
}

/// The loader of `bp.loot_tables()`: `loot_tables/**/*.json` of every behaviour pack, keyed by
/// path, the top pack's copy winning.
pub const fn load_files() -> CollectionLoader<Files<LootTable>, FileCopy> {
    CollectionLoader {
        walk: WALK,
        parse: parse_walk_file,
        combine,
        needs: &[],
        then: None,
    }
}
