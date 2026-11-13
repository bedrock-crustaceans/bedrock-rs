//! The locale overlay `texts/<code>/` (README "Locale overlay"): the `overlay/` rows of
//! fixtures.md resolved with `locale_overlay_paths`, and the lints L15 and L16.

use bedrock_addon::diagnostic::{Kind, Severity};
use bedrock_addon::families::lang::{Language, locale_overlay_paths};
use bedrock_addon::pack::{PackStack, Resource};
use bedrock_addon::storage::PackPath;

use super::common::*;
use super::stacks::*;

fn lang_of(code: &str) -> Language {
    Language::new(code)
}

/// The lookup order inside one pack, as text.
fn paths(language: &str, path: &str, subpack: Option<&str>) -> Vec<String> {
    let subpack = subpack.map(|folder| PackPath::new(folder).unwrap());
    locale_overlay_paths(
        &lang_of(language),
        &PackPath::new(path).unwrap(),
        subpack.as_ref(),
    )
    .iter()
    .map(ToString::to_string)
    .collect()
}

/// One pack of an overlay case: its files and its selected subpack folder.
struct OverlayPack {
    files: Vec<(String, Vec<u8>)>,
    subpack: Option<&'static str>,
}

/// The packs of `overlay/<case>`, top first, none with a selected subpack.
fn packs(case: &str) -> Vec<OverlayPack> {
    let mut folders: Vec<String> = std::fs::read_dir(fixtures().join("overlay").join(case))
        .unwrap()
        .filter_map(|entry| entry.unwrap().file_name().into_string().ok())
        .collect();
    folders.sort();
    folders.reverse();
    folders
        .iter()
        .map(|folder| OverlayPack {
            files: files_below(&format!("overlay/{case}/{folder}")),
            subpack: None,
        })
        .collect()
}

/// What a client whose language is `language` reads for `path` from the stack `packs` (top
/// first): the first pack that has any copy gives its first copy in lookup order. Returns the
/// pack's index and the file's bytes.
fn read(packs: &[OverlayPack], language: &str, path: &str) -> Option<(usize, String)> {
    packs.iter().enumerate().find_map(|(index, pack)| {
        paths(language, path, pack.subpack)
            .iter()
            .find_map(|candidate| {
                pack.files
                    .iter()
                    .find(|(file, _)| file == candidate)
                    .map(|(_, bytes)| (index, String::from_utf8(bytes.clone()).unwrap()))
            })
    })
}

const SHEET: &str = "font/glyph_30.png";

// ---- the path rule -------------------------------------------------------------------------

#[test]
fn a_pack_is_asked_for_its_overlay_copy_and_then_its_root_copy() {
    assert_eq!(
        paths("ja_JP", SHEET, None),
        ["texts/ja_JP/font/glyph_30.png", "font/glyph_30.png"]
    );
    // the code as the language list has it, whatever its form
    assert_eq!(
        paths("Klingon", "a.json", None),
        ["texts/Klingon/a.json", "a.json"]
    );
}

#[test]
fn the_selected_subpack_comes_first_with_its_own_overlay() {
    assert_eq!(
        paths("ja_JP", SHEET, Some("subpacks/hd")),
        [
            "subpacks/hd/texts/ja_JP/font/glyph_30.png",
            "subpacks/hd/font/glyph_30.png",
            "texts/ja_JP/font/glyph_30.png",
            "font/glyph_30.png",
        ]
    );
}

#[test]
fn a_code_that_is_no_folder_name_has_no_overlay() {
    for code in ["..", ".", "a:b", "", "ja_JP/", "/ja_JP", "ja/JP", "ja\\JP"] {
        assert_eq!(paths(code, SHEET, None), [SHEET], "{code:?}");
    }
}

// ---- the fixture rows ----------------------------------------------------------------------

#[test]
fn font_sheet_is_the_overlay_copy_for_its_language_only() {
    let packs = packs("font_sheet");
    assert_eq!(read(&packs, "ja_JP", SHEET), Some((0, "B".to_owned())));
    assert_eq!(read(&packs, "en_US", SHEET), Some((0, "A".to_owned())));
}

#[test]
fn fallthrough_the_top_packs_overlay_copy_wins_over_a_lower_root_copy() {
    let packs = packs("fallthrough_overlay_on_top");
    assert_eq!(read(&packs, "ja_JP", SHEET), Some((0, "B".to_owned())));
    // for another language the top pack has no copy at all
    assert_eq!(read(&packs, "en_US", SHEET), Some((1, "A".to_owned())));
}

#[test]
fn fallthrough_the_top_packs_root_copy_wins_over_a_lower_overlay_copy() {
    let packs = packs("fallthrough_root_on_top");
    assert_eq!(read(&packs, "ja_JP", SHEET), Some((0, "A".to_owned())));
}

#[test]
fn subpack_first_the_selected_subpack_comes_before_the_parents_overlay() {
    let mut packs = packs("subpack_first");
    packs[0].subpack = Some("subpacks/hd");
    assert_eq!(read(&packs, "ja_JP", SHEET), Some((0, "C".to_owned())));
    // without a selected subpack the parent's overlay copy is read
    packs[0].subpack = None;
    assert_eq!(read(&packs, "ja_JP", SHEET), Some((0, "B".to_owned())));
}

#[test]
fn any_path_is_overlaid_not_only_font_sheets() {
    let packs = packs("any_path");
    let stone = "textures/blocks/stone.png";
    assert_eq!(read(&packs, "ja_JP", stone), Some((0, "B".to_owned())));
    assert_eq!(read(&packs, "en_US", stone), Some((0, "A".to_owned())));
    assert_eq!(read(&packs, "ja_JP", "textures/blocks/dirt.png"), None);
}

// ---- L15, L16 ------------------------------------------------------------------------------

/// The findings of `languages()` with `code`'s name at info, as `(path, pack index)`.
fn infos(stack: &PackStack<Resource>, code: &str) -> Vec<(String, Option<usize>)> {
    stack
        .languages()
        .diagnostics()
        .filter(|d| d.code.id() == code && d.severity == Severity::Info)
        .map(|d| {
            (
                d.location
                    .path
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                d.location
                    .pack
                    .as_ref()
                    .and_then(bedrock_addon::pack::PackRef::index),
            )
        })
        .collect()
}

/// A stack of one pack listing `en_US` and `ja_JP`, with these other files.
fn overlay_stack(files: &[&str]) -> PackStack<Resource> {
    let mut all = vec![("texts/languages.json", r#"["en_US", "ja_JP"]"#)];
    all.extend(files.iter().map(|path| (*path, "x")));
    resource_stack(vec![pack_with(1, "resources", &all)])
}

#[test]
fn an_overlay_file_that_is_no_font_sheet_is_reported_as_shadowing_deviation() {
    // L15, vanilla = "silent per-file overlay (ResourcePack::getResource)"
    let stack = resource_stack(fixture_packs("overlay/any_path", "resources"));
    assert_eq!(
        infos(&stack, "core/shadowed_definition"),
        [("texts/ja_JP/textures/blocks/stone.png".to_owned(), Some(0))]
    );
    assert!(
        stack
            .languages()
            .diagnostics()
            .all(|d| { d.is(&bedrock_addon::diagnostic::codes::SHADOWED_DEFINITION) })
    );
    assert_eq!(stack.languages().diagnostics().count(), 1);
}

#[test]
fn the_font_sheets_of_an_overlay_are_what_it_is_for() {
    let stack = overlay_stack(&[
        "texts/ja_JP/font/glyph_30.png",
        "texts/ja_JP/font/sub/glyph_31.png",
        "font/glyph_30.png",
    ]);
    assert_eq!(stack.languages().diagnostics().count(), 0);
}

#[test]
fn only_the_folder_named_font_is_exempt() {
    let stack = overlay_stack(&[
        "texts/ja_JP/fonts/glyph_30.png",
        "texts/ja_JP/font.png",
        "texts/ja_JP/texts/en_US.lang",
    ]);
    let found: Vec<String> = infos(&stack, "core/shadowed_definition")
        .into_iter()
        .map(|(path, _)| path)
        .collect();
    assert_eq!(
        found,
        [
            "texts/ja_JP/font.png",
            "texts/ja_JP/fonts/glyph_30.png",
            "texts/ja_JP/texts/en_US.lang",
        ]
    );
}

#[test]
fn an_overlay_folder_of_an_unlisted_code_is_reported_once_deviation() {
    // L16, vanilla = "silent": the overlay can never be active
    let stack = overlay_stack(&[
        "texts/xx_XX/font/glyph_30.png",
        "texts/xx_XX/textures/a.png",
        "texts/zz_ZZ/a.json",
    ]);
    assert_eq!(
        infos(&stack, "core/ignored_file"),
        [
            ("texts/xx_XX".to_owned(), Some(0)),
            ("texts/zz_ZZ".to_owned(), Some(0)),
        ]
    );
    // a folder that can never be active shadows nothing
    assert!(
        !stack
            .languages()
            .diagnostics()
            .any(|d| d.code.kind() == Kind::Shadowed && d.severity == Severity::Info)
    );
    assert_eq!(stack.languages().diagnostics().count(), 2);
}

#[test]
fn each_pack_with_the_folder_is_reported() {
    let stack = resource_stack(vec![
        pack_with(2, "resources", &[("texts/xx_XX/a.json", "x")]),
        pack_with(
            1,
            "resources",
            &[
                ("texts/languages.json", r#"["en_US"]"#),
                ("texts/xx_XX/a.json", "x"),
            ],
        ),
    ]);
    assert_eq!(
        infos(&stack, "core/ignored_file"),
        [
            ("texts/xx_XX".to_owned(), Some(1)),
            ("texts/xx_XX".to_owned(), Some(0)),
        ]
    );
}

#[test]
fn a_file_directly_in_texts_is_no_overlay() {
    let stack = overlay_stack(&["texts/readme.txt", "texts/en_US.txt"]);
    assert_eq!(stack.languages().diagnostics().count(), 0);
}
