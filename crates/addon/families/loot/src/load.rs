//! Loading loot tables: `Content`, the walk of `loot_tables/` and the combine rule.

use super::LootTable;
use super::parse::parse_table;
use bedrock_addon_core::document::unversioned;
use bedrock_addon_core::document::{
    Content, Document, Files, FilesBuilder, FilesRule, FiniteText, JsonContent, Origin, Rejected,
};
use bedrock_addon_core::json::{EncodeError, WriteStyle};
use bedrock_addon_core::load::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

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
}

impl JsonContent for LootTable {}

/// Loads one loot table file of `pack`.
pub(crate) fn parse_file(
    pack: PackRef,
    side: PackSide,
    path: &PackPath,
    bytes: &[u8],
    cx: &LoadContext,
) -> Result<Document<LootTable>, Rejected> {
    let origin = Origin::file(pack, side, path.clone());
    // A number no double holds is read as the largest finite one, with a warning.
    let finite = FiniteText::of(bytes);
    let (table, mut diagnostics) = parse_table(path, finite.bytes(), cx);
    finite.settle(&mut diagnostics, None);
    diagnostics.locate(&origin);
    match table {
        // Vanilla reads no version: none is written, effective or parsed.
        Some(table) => Ok(Document::new(
            table,
            origin,
            unversioned::version(),
            diagnostics,
        )),
        None => Err(Rejected::new(origin, diagnostics)),
    }
}

// ---- the collection ----------------------------------------------------------------------

/// `loot_tables/**/*.json`, packs bottom-up.
const WALK: Walk = Walk::json_tree_in(&["loot_tables"]);

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
    FilesBuilder::combine(parsed, FilesRule::TopWins, stack.options)
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
