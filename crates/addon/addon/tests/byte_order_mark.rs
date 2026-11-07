#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg(feature = "full")]
//! A file that starts with a UTF-8 byte-order mark: vanilla's reader skips the mark,
//! and every span of a finding is an offset into the file as written, the three bytes of the
//! mark counted, so that `&file[span]` is the text the finding is about.
//!
//! Three read paths: a family of the versioned load (a block), a family with a reader of its
//! own (a loot table), and the copy of a file a stack walk reads (the terrain atlas).

#[allow(unused_imports)]
#[path = "terrain_texture/common.rs"]
mod common;

use bedrock_addon::diagnostic::Diagnostic;
use bedrock_addon::document::Content;
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::PackType;
use bedrock_addon::storage::PackPath;

#[macro_use]
#[path = "common/document_types.rs"]
mod document_types;
use document_types::{beta, client, ddui, pack, poi};

const MARK: &[u8] = b"\xEF\xBB\xBF";

fn marked(text: &str) -> Vec<u8> {
    [MARK, text.as_bytes()].concat()
}

/// Each finding as `(code, path, bytes under its span)`; a finding without a span has none.
fn under_spans<'a>(
    file: &[u8],
    found: impl IntoIterator<Item = &'a Diagnostic>,
) -> Vec<(String, String, Option<String>)> {
    found
        .into_iter()
        .map(|finding| {
            let under = finding.location.span.map(|span| {
                let start = span.offset as usize;
                let bytes = file
                    .get(start..start + span.len as usize)
                    .unwrap_or_else(|| panic!("the span {span:?} lies in the file"));
                // A span of the whole file begins with the mark: the same text but for it.
                String::from_utf8_lossy(bytes)
                    .trim_start_matches('\u{feff}')
                    .to_owned()
            });
            (
                finding.code.id().to_owned(),
                finding.location.json_path.to_string(),
                under,
            )
        })
        .collect()
}

/// The findings of one file read through its type's public entry.
fn findings<T: Content>(path: &'static str, file: &[u8]) -> Vec<(String, String, Option<String>)> {
    findings_in::<T>(path, file, &LoadContext::detached())
}

fn findings_in<T: Content>(
    path: &'static str,
    file: &[u8],
    cx: &LoadContext,
) -> Vec<(String, String, Option<String>)> {
    match T::parse(&PackPath::from_static(path), file, cx) {
        Ok(document) => under_spans(file, document.diagnostics().iter()),
        Err(rejected) => under_spans(file, rejected.diagnostics.iter()),
    }
}

/// Requires the marked file to give the findings of the file without the mark, each on the
/// same text, and the finding `(code, path)` to be on `token`.
#[track_caller]
fn assert_same_text_under_every_span<T: Content>(
    path: &'static str,
    text: &str,
    on: &[(&str, &str, &str)],
) {
    let plain = findings::<T>(path, text.as_bytes());
    let with_mark = findings::<T>(path, &marked(text));
    assert_eq!(with_mark, plain, "the mark moves no finding off its text");
    for (code, at, token) in on {
        let expected = (
            (*code).to_owned(),
            (*at).to_owned(),
            Some((*token).to_owned()),
        );
        assert!(
            with_mark.contains(&expected),
            "{expected:?} is not among {with_mark:#?}"
        );
    }
}

#[test]
fn a_block_read_by_the_versioned_load_has_its_spans_in_the_file_as_written() {
    // A block is a `MinecraftDocumentInput`: the game's JSON reader refuses the number no double holds,
    // the first error of the text, at the number's token.
    assert_same_text_under_every_span::<bedrock_addon::families::block::Block>(
        "blocks/x.json",
        r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:friction": {"oops": 1}, "demo:kept": {"n": -1e999}}}} tail"#,
        &[("core/document_json", "", "-1e999")],
    );
}

#[test]
fn a_loot_table_read_by_its_own_reader_has_its_spans_in_the_file_as_written() {
    assert_same_text_under_every_span::<bedrock_addon::families::loot::LootTable>(
        "loot_tables/x.json",
        r#"{"pools": [{"rolls": 1, "entries": [{"type": "item", "name": "minecraft:stone"}], "zz": [0, 1e999]}]} tail"#,
        &[("core/invalid_value", "pools/0/zz/1", "1e999")],
    );
}

#[test]
fn a_copy_read_by_a_stack_walk_has_its_spans_in_the_file_as_written() {
    const FILE: &str = "textures/terrain_texture.json";
    let text = r#"{"texture_name": "atlas.terrain", "texture_data": {"x": {"textures": "textures/a", "zz": -1e999}}} tail"#;
    let of = |file: &[u8]| {
        let files: &[(&str, &[u8])] = &[(FILE, file), ("textures/a.png", b"")];
        let stack = common::stack_of_files(&[files]);
        under_spans(file, stack.texture_atlases().terrain_atlas.diagnostics())
    };
    let plain = of(text.as_bytes());
    let with_mark = of(&marked(text));
    assert_eq!(with_mark, plain, "{with_mark:#?}");
    assert!(
        with_mark.contains(&(
            "core/invalid_value".to_owned(),
            "texture_data/x/zz".to_owned(),
            Some("-1e999".to_owned())
        )),
        "{with_mark:#?}"
    );
}

/// A render controller file is read folded and its findings are given the file's spelling
/// back: with a mark in front the path still spells the key as the file does and the
/// span is the number's, in the file as written.
#[test]
fn a_render_controller_with_a_capital_key_has_its_spans_in_the_file_as_written() {
    assert_same_text_under_every_span::<
        bedrock_addon::families::render_controller::RenderControllerFile,
    >(
        "render_controllers/x.json",
        r#"{"format_version": "1.8.0", "render_controllers": {"controller.render.test": {"geometry": "Geometry.default", "materials": [{"*": "Material.default"}, {"bodyLayer_*": 7}], "textures": ["Texture.default"]}}}"#,
        &[(
            "core/invalid_type",
            "render_controllers/controller.render.test/materials/1/bodyLayer_*",
            "7",
        )],
    );
}

/// A function file is text, not JSON: its lines are counted from the start of the file, the
/// mark included.
#[test]
fn a_function_file_has_the_spans_of_its_lines_in_the_file_as_written() {
    let text = "say hi\n/say slash\n\"unclosed\nnot_a_command 1 2 3\n";
    let path = "functions/x.mcfunction";
    let plain = findings::<bedrock_addon::families::function::Function>(path, text.as_bytes());
    let with_mark = findings::<bedrock_addon::families::function::Function>(path, &marked(text));
    assert_eq!(with_mark, plain);
    let lines: Vec<&str> = plain.iter().filter_map(|f| f.2.as_deref()).collect();
    assert!(!lines.is_empty(), "a finding on a line: {plain:?}");
    assert!(
        lines.iter().all(|line| text.lines().any(|l| l == *line)),
        "{lines:?}"
    );
}

// --- every document type ---------------------------------------------------------------------

/// Texts of a document type that give findings with spans: the document, the document with
/// content after its root, with a number no double holds in an unknown member of its root,
/// and cut short.
fn texts(document: &str) -> Vec<String> {
    let mut out = vec![document.to_owned(), format!("{document} tail")];
    if let Some(body) = document.strip_suffix('}') {
        out.push(format!("{body}, \"zz_probe\": [0, -1e999]}}"));
    }
    out.push(document[..document.len() - 1].to_owned());
    out
}

/// One test per document type read through `Content::parse`: with the mark in front, every
/// text gives the findings it gives without, each span on the same text of the file.
macro_rules! families {
    ($( $test:ident: $type:ty, $path:literal $(in $cx:expr)?, $document:expr $(, [$($more:expr),* $(,)?])?; )*) => {
        mod every_type {
            use super::*;
            $(
                #[test]
                fn $test() {
                    #[allow(unused_variables, reason = "most families read in a detached context")]
                    let cx = LoadContext::detached();
                    $(let cx = $cx;)?
                    let mut spans = 0;
                    for text in texts($document) {
                        let plain = findings_in::<$type>($path, text.as_bytes(), &cx);
                        let with_mark = findings_in::<$type>($path, &marked(&text), &cx);
                        assert_eq!(with_mark, plain, "{text}");
                        spans += plain.iter().filter(|finding| finding.2.is_some()).count();
                    }
                    println!("mark: {} | {spans} spans compared", stringify!($test));
                }
            )*
        }
    };
}

document_types!(families);

/// Whether `file` loads as `T`, and its findings as `code severity path`.
fn outcome<T: Content>(path: &'static str, file: &[u8]) -> (bool, Vec<String>) {
    let said =
        |d: &Diagnostic| format!("{} {:?} {}", d.code.id(), d.severity, d.location.json_path);
    match T::parse(&PackPath::from_static(path), file, &LoadContext::detached()) {
        Ok(document) => (true, document.diagnostics().iter().map(said).collect()),
        Err(rejected) => (false, rejected.diagnostics.iter().map(said).collect()),
    }
}

/// The definitions of a document with no text in it.
const EMPTY_DOCUMENT: &[&str] = &[
    "core/empty_document",
    "core/empty_document_worded",
    "fog/empty_after_parse",
    "fog/empty_resource",
    "loot/invalid_table_empty",
    "server_config/commands_malformed_empty_document",
    "server_config/config_parse_failed_empty_document",
    "texture_set/set_empty",
];

/// A file that is only a byte-order mark is the empty file.
/// Vanilla's reader skips the mark, so what is left is no text at all, and each family says
/// of it what it says of a file of no bytes: one outcome, whatever the read path.
#[test]
fn a_file_that_is_only_a_byte_order_mark_is_the_empty_file() {
    const MARK: &[u8] = b"\xEF\xBB\xBF";
    macro_rules! same {
        ($($type:ty: $path:literal;)*) => {$(
            let (empty, marked) = (outcome::<$type>($path, b""), outcome::<$type>($path, MARK));
            assert_eq!(marked, empty, "{}: the mark alone, then no bytes", $path);
            // One outcome across the families: the file is refused as an empty document
            // (the geometry's is an INFO by its folder's rule 0: vanilla is silent there).
            assert_eq!(marked.1.len(), 1, "{}", $path);
            assert!(
                EMPTY_DOCUMENT.iter().any(|id| marked.1[0].starts_with(&format!("{id} "))),
                "{}: {:?}",
                $path,
                marked.1
            );
            assert!(!empty.0, "{}: an empty file is no document", $path);
        )*};
    }
    same! {
        bedrock_addon::families::block::Block: "blocks/x.json";
        bedrock_addon::manifest::Manifest: "manifest.json";
        bedrock_addon::families::item::Item: "items/x.json";
        bedrock_addon::families::loot::LootTable: "loot_tables/x.json";
        bedrock_addon::families::geometry::GeometryFile: "models/entity/x.json";
        bedrock_addon::families::trade::TradeTable: "trading/x.json";
        bedrock_addon::families::entity::Entity: "entities/x.json";
    }
}
