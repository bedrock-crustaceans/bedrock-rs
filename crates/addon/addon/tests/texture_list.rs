#![allow(clippy::pedantic, missing_docs)]
//! Texture lists (`bedrock_addon::families::texture_list`): every fixture
//! row. Each fixture is a pack tree `<name>/pack/` (`pack0`, `pack1` for a stack, the
//! first on top), loaded through `rp.texture_lists()`; the subpack rows read the two lists and ask
//! `TextureList::mode`.

#![cfg(all(feature = "texture_list", feature = "texture_file"))]
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

// The crate's one protocol for cost tests, and this binary's one lock.
#[path = "../src/test_cost.rs"]
mod cost;

#[path = "terrain_texture/common.rs"]
mod common;

#[path = "common/responder.rs"]
mod responder;

#[path = "texture_list/subpack.rs"]
mod subpack;

use bedrock_addon::diagnostic::Severity::{self, Info};
use bedrock_addon::diagnostic::{Diagnostic, Diagnostics};
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::texture_file::TextureSource;
use bedrock_addon::families::texture_list::{
    TextureList, TextureListMode, TextureListRoot, scan_claims,
};
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::{PackOrigin, PackStack, Resource};
use bedrock_addon::storage::PackPath;
use common::{files_below, fixture_dir, fixture_file, pack_in, stack_of};

const FAMILY: &str = "textures_list";
const FILE: &str = "textures/textures_list.json";
const A: &str = "textures/blocks/a";

/// The one-pack stack of fixture `name`, the pack opened as coming from `origin`.
fn stack_from(name: &str, origin: PackOrigin) -> PackStack<Resource> {
    stack_of(vec![pack_in(
        &fixture_dir(FAMILY, name).join("pack"),
        1,
        origin,
    )])
}

fn stack(name: &str) -> PackStack<Resource> {
    stack_from(name, PackOrigin::User)
}

/// The two-pack stack of fixture `name`: `pack0` on top of `pack1`.
fn stack2(name: &str) -> PackStack<Resource> {
    let dir = fixture_dir(FAMILY, name);
    stack_of(vec![
        pack_in(&dir.join("pack0"), 1, PackOrigin::User),
        pack_in(&dir.join("pack1"), 2, PackOrigin::User),
    ])
}

/// What the client does with the top pack's list.
fn mode(stack: &PackStack<Resource>) -> TextureListMode {
    stack.texture_lists().list_mode(0).unwrap().clone()
}

fn list(entries: &[&str]) -> TextureListMode {
    TextureListMode::List {
        entries: entries.iter().map(|entry| (*entry).to_owned()).collect(),
    }
}

/// The top pack's list document.
fn document(stack: &PackStack<Resource>) -> &TextureList {
    stack.texture_lists().lists.first().expect("a list").value()
}

type Found = (&'static str, Severity, String);

fn shown<'a>(findings: impl IntoIterator<Item = &'a Diagnostic>) -> Vec<Found> {
    findings
        .into_iter()
        .map(|f| {
            let file = f.location.path.as_ref().map_or("", |path| path.as_str());
            let place = f.location.json_path.to_string();
            let at = if file == FILE {
                place
            } else {
                format!("{file}:{place}")
            };
            (f.code.name(), f.severity, at)
        })
        .collect()
}

/// Every finding of the group, each as code, severity and place (`<file>:` first when it is
/// not the list).
fn found(stack: &PackStack<Resource>) -> Vec<Found> {
    shown(stack.texture_lists().diagnostics())
}

fn at(code: &'static str, severity: Severity, place: &str) -> Found {
    (code, severity, place.to_owned())
}

// Every lint of the list but V0 is ours (the client logs nothing about its list): an
// information; V0 is the game's developer-log line, an information with its text.

/// V1: the file is no strict JSON.
fn v1() -> Found {
    at("MalformedInput", Info, "")
}
/// V2 (a scalar root), V3 (an object root) and V9 (images the list omits) are about the list
/// as a whole.
fn whole() -> Found {
    at("InvalidValue", Info, "")
}
/// V4 (not a string), V5 (an extension), V6 (not normalised): about one entry.
fn entry(index: usize) -> Found {
    at("InvalidValue", Info, &index.to_string())
}
/// V8: no file of the pack matches the entry.
fn stale(index: usize) -> Found {
    at("TextureNotFound", Info, &index.to_string())
}

fn claims(stack: &PackStack<Resource>) -> Vec<(String, usize)> {
    stack
        .texture_lists()
        .claims()
        .into_iter()
        .map(|(location, claim)| (location.clone(), claim.stack_index))
        .collect()
}

fn claim(location: &str, stack_index: usize) -> (String, usize) {
    (location.to_owned(), stack_index)
}

fn image(stack_index: usize, file: &str) -> Option<TextureSource> {
    Some(TextureSource::Image {
        stack_index,
        file: file.to_owned(),
    })
}

// `textures/texture_list.json` of the vanilla resource pack, copied unchanged, with a 1x1
// image for each of its 20 entries.
#[test]
fn tl01_the_misnamed_sample_list_is_never_read() {
    let stack = stack("tl01_sample_texture_list");
    assert_eq!(mode(&stack), TextureListMode::Scan);
    assert!(stack.texture_lists().lists.is_empty());
    assert_eq!(
        found(&stack),
        [at("IgnoredFile", Info, "textures/texture_list.json:")]
    );
    let claims = claims(&stack);
    assert_eq!(claims.len(), 20);
    assert!(claims.contains(&claim("textures/blocks/sulfur", 0)));
}

#[test]
fn tl02_the_same_list_under_the_right_name_is_a_list() {
    let stack = stack("tl02_same_list_correct_name");
    let TextureListMode::List { entries } = mode(&stack) else {
        panic!("not list mode");
    };
    assert_eq!(entries.len(), 20);
    assert_eq!(document(&stack).root, TextureListRoot::Array);
    assert_eq!(found(&stack), []);
    assert_eq!(claims(&stack).len(), 20);
}

#[test]
fn tl03_a_pack_without_a_list_is_scanned() {
    let stack = stack("tl03_missing");
    assert_eq!(mode(&stack), TextureListMode::Scan);
    assert_eq!(found(&stack), []);
    assert_eq!(claims(&stack), [claim(A, 0)]);
}

#[test]
fn tl04_a_base_game_pack_without_a_list_gets_vanilla_s_developer_log_line() {
    let stack = stack_from("tl04_missing_base_pack", PackOrigin::Package);
    assert_eq!(mode(&stack), TextureListMode::Scan);
    let findings: Vec<_> = stack.texture_lists().diagnostics().collect();
    assert_eq!(
        shown(findings.iter().copied()),
        [at("MissingRequiredField", Info, "")]
    );
    assert_eq!(
        findings[0].code.to_string(),
        "Failed to find textures/textures_list.json in pack 1 for a base pack with a texture directory, falling back to pack iteration"
    );
}

#[test]
fn a_base_game_pack_without_a_texture_directory_gets_no_warning() {
    let pack = common::pack_of(1, &[("sounds.json", b"{}")], PackOrigin::Package);
    assert_eq!(found(&stack_of(vec![pack])), []);
    // Nor one with a list.
    let pack = common::pack_of(
        1,
        &[(FILE, br#"["textures/a"]"#), ("textures/a.png", b"")],
        PackOrigin::Package,
    );
    assert_eq!(found(&stack_of(vec![pack])), []);
}

#[test]
fn tl05_empty_roots_are_scanned_without_a_finding() {
    for (name, root) in [
        ("tl05_empty_array", TextureListRoot::Array),
        ("tl05b_empty_object", TextureListRoot::Object),
        ("tl05c_null", TextureListRoot::Null),
    ] {
        let stack = stack(name);
        assert_eq!(mode(&stack), TextureListMode::Scan, "{name}");
        assert_eq!(document(&stack).root, root, "{name}");
        assert_eq!(found(&stack), [], "{name}");
    }
}

#[test]
fn tl06_a_scalar_root_is_a_list_without_entries_deviation() {
    let stack = stack("tl06_scalar_root");
    assert_eq!(mode(&stack), list(&[]));
    assert_eq!(document(&stack).root, TextureListRoot::Scalar);
    assert_eq!(found(&stack), [whole()]);
    assert_eq!(claims(&stack), []);
}

#[test]
fn tl07_an_object_root_lists_its_values_deviation() {
    let stack = stack("tl07_object_root");
    assert_eq!(mode(&stack), list(&[A]));
    assert_eq!(found(&stack), [whole()]);
}

#[test]
fn tl07b_an_object_root_is_read_in_key_order_and_the_last_duplicate_wins_deviation() {
    let stack = stack("tl07b_object_key_order");
    assert_eq!(mode(&stack), list(&[A, "textures/blocks/c"]));
    assert_eq!(found(&stack), [whole()]);
}

#[test]
fn tl07c_an_object_that_breaks_off_keeps_its_members_deviation() {
    let stack = stack("tl07c_object_partial");
    assert_eq!(mode(&stack), list(&[A]));
    assert_eq!(document(&stack).skipped_non_strings, 0);
    assert_eq!(found(&stack), [v1(), whole()]);
}

#[test]
fn tl08_elements_that_are_no_strings_are_skipped_deviation() {
    let stack = stack("tl08_non_strings");
    assert_eq!(mode(&stack), list(&[A]));
    assert_eq!(document(&stack).skipped_non_strings, 3);
    assert_eq!(found(&stack), [entry(1), entry(2), entry(3)]);
}

#[test]
fn a_list_of_non_strings_only_is_not_empty() {
    // `[3]` is no empty value: list mode, nothing claimed, nothing scanned.
    let pack: &[(&str, &[u8])] = &[(FILE, b"[3]"), ("textures/a.png", b"")];
    let stack = common::stack_of_files(&[pack]);
    assert_eq!(mode(&stack), list(&[]));
    assert_eq!(claims(&stack), []);
}

#[test]
fn tl09_an_entry_with_an_extension_is_wasted_and_not_stale_deviation() {
    let stack = stack("tl09_extension_in_entry");
    // V5 for the entry, V9 for the unlisted `textures/blocks/a`; no V8: the client probes the
    // path as written first.
    assert_eq!(found(&stack), [entry(0), whole()]);
    assert_eq!(
        stack.texture_files().source("textures/blocks/a.png"),
        image(0, "textures/blocks/a.png")
    );
}

#[test]
fn tl10_unnormalised_entries_are_kept_verbatim_deviation() {
    let stack = stack("tl10_unnormalised");
    assert_eq!(
        mode(&stack),
        list(&[
            "textures\\blocks\\a",
            "./textures/blocks/a",
            "/textures/blocks/a",
            " textures/blocks/a"
        ])
    );
    assert_eq!(
        found(&stack),
        [
            entry(0),
            entry(1),
            entry(2),
            entry(3),
            stale(0),
            stale(1),
            stale(2),
            stale(3)
        ]
    );
}

#[test]
fn tl11_an_entry_outside_textures_is_an_info_deviation() {
    let stack = stack("tl11_outside_textures");
    assert_eq!(found(&stack), [at("InvalidValue", Info, "0")]);
    assert_eq!(
        stack.texture_files().source("ui/foo"),
        image(0, "ui/foo.png")
    );
}

#[test]
fn tl12_a_stale_entry_is_an_info_deviation() {
    let stack = stack("tl12_stale_entry");
    assert_eq!(found(&stack), [stale(0)]);
    assert_eq!(stack.texture_files().source("textures/blocks/gone"), None);
}

#[test]
fn tl13_a_list_that_omits_images_is_incomplete_deviation() {
    let stack = stack("tl13_incomplete");
    assert_eq!(found(&stack), [whole()]);
    // The list is what is claimed.
    assert_eq!(claims(&stack), [claim(A, 0)]);
}

#[test]
fn a_complete_list_is_no_finding() {
    let pack: &[(&str, &[u8])] = &[
        (FILE, br#"["textures/a", "textures/c"]"#),
        ("textures/a.png", b""),
        ("textures/c.tga", b""),
    ];
    assert_eq!(found(&common::stack_of_files(&[pack])), []);
}

#[test]
fn tl14_a_duplicate_entry_is_an_info_and_one_claim_deviation() {
    let stack = stack("tl14_duplicates");
    assert_eq!(found(&stack), [at("DuplicateIdentifier", Info, "0")]);
    assert_eq!(claims(&stack), [claim(A, 0)]);
}

#[test]
fn tl15_a_list_that_breaks_off_keeps_its_prefix_deviation() {
    let stack = stack("tl15_invalid_json");
    assert_eq!(mode(&stack), list(&[A]));
    assert_eq!(document(&stack).skipped_non_strings, 0);
    assert!(document(&stack).partial);
    assert_eq!(found(&stack), [v1()]);
}

#[test]
fn tl15b_an_open_bracket_alone_is_a_list_without_entries_deviation() {
    let stack = stack("tl15b_open_bracket_only");
    assert_eq!(mode(&stack), list(&[]));
    assert_eq!(found(&stack), [v1()]);
}

#[test]
fn tl15c_an_unreadable_root_is_scanned_deviation() {
    let stack = stack("tl15c_root_garbage");
    assert_eq!(mode(&stack), TextureListMode::Scan);
    assert_eq!(document(&stack).root, TextureListRoot::Invalid);
    assert_eq!(found(&stack), [v1()]);
}

#[test]
fn tl15d_a_trailing_comma_is_no_strict_json_deviation() {
    let stack = stack("tl15d_trailing_comma");
    assert_eq!(mode(&stack), list(&[A]));
    assert_eq!(found(&stack), [v1()]);
}

#[test]
fn tl15e_content_after_the_root_is_no_strict_json_deviation() {
    let stack = stack("tl15e_trailing_content");
    assert_eq!(mode(&stack), list(&[A]));
    assert!(!document(&stack).partial);
    assert_eq!(found(&stack), [v1()]);
}

#[test]
fn tl16_a_bom_and_comments_are_no_finding() {
    let stack = stack("tl16_bom_and_comments");
    assert_eq!(mode(&stack), list(&[A]));
    assert_eq!(found(&stack), []);
}

/// The base list and the list of the subpack `hi` of fixture `name`, each with its findings.
fn base_and_subpack(name: &str) -> ((TextureList, Diagnostics), (TextureList, Diagnostics)) {
    let read = |file: &str| {
        let path = fixture_dir(FAMILY, name).join("pack").join(file);
        let bytes = std::fs::read(path).ok();
        let mut findings = Diagnostics::default();
        let list = TextureList::read(bytes.as_deref(), &mut findings);
        (list, findings)
    };
    (read(FILE), read(&format!("subpacks/hi/{FILE}")))
}

#[test]
fn tl17_a_subpack_s_entries_are_appended() {
    let ((base, _), (subpack, _)) = base_and_subpack("tl17_subpack_append");
    assert_eq!(
        base.mode(Some(&subpack)),
        list(&["textures/a", "textures/b"])
    );
    // A subpack that contributes nothing leaves the base as it is.
    assert_eq!(
        base.mode(Some(&TextureList::default())),
        list(&["textures/a"])
    );
    assert_eq!(base.mode(None), list(&["textures/a"]));
}

#[test]
fn tl17b_a_subpack_replaces_a_scalar_base() {
    let ((base, findings), (subpack, _)) = base_and_subpack("tl17b_subpack_scalar_base");
    assert_eq!(base.mode(Some(&subpack)), list(&["textures/b"]));
    // V2 is still reported on the base file.
    assert_eq!(shown(findings.iter()), [at("InvalidValue", Info, ":")]);
}

#[test]
fn tl17c_a_subpack_drops_the_entries_of_an_object_base() {
    let ((base, findings), (subpack, _)) = base_and_subpack("tl17c_subpack_object_base");
    assert_eq!(base.entries, ["textures/a", "textures/c"]);
    assert_eq!(base.mode(Some(&subpack)), list(&["textures/b"]));
    assert_eq!(shown(findings.iter()), [at("InvalidValue", Info, ":")]);
}

#[test]
fn tl18_a_subpack_s_list_alone_is_a_list() {
    let ((base, _), (subpack, _)) = base_and_subpack("tl18_subpack_only");
    assert_eq!(base.root, TextureListRoot::Missing);
    assert_eq!(base.mode(Some(&subpack)), list(&["textures/b"]));
    assert_eq!(base.mode(None), TextureListMode::Scan);
}

#[test]
fn tl19_the_first_claim_wins_and_the_texture_loads_through_the_stack() {
    let stack = stack2("tl19_stack_first_claim_wins");
    // The top pack lists `textures/x` without having it; the lower pack has the image.
    assert_eq!(claims(&stack), [claim("textures/x", 0)]);
    assert_eq!(
        stack.texture_files().source("textures/x"),
        image(1, "textures/x.png")
    );
    assert_eq!(found(&stack), [stale(0)]);
}

#[test]
fn tl20_a_higher_pack_s_override_is_what_is_loaded() {
    let stack = stack2("tl20_stack_incomplete_override");
    // The top pack has `textures/x.png` and a list without it; the lower pack lists it.
    let claims = claims(&stack);
    assert!(claims.contains(&claim("textures/x", 1)), "{claims:?}");
    assert_eq!(
        stack.texture_files().source("textures/x"),
        image(0, "textures/x.png")
    );
    // V9 on the top pack, V8 on the lower one (it lists what it does not have).
    let findings: Vec<_> = stack.texture_lists().diagnostics().collect();
    assert_eq!(shown(findings.iter().copied()), [whole(), stale(0)]);
    assert_eq!(common::pack_index(findings[0]), Some(0));
    assert_eq!(common::pack_index(findings[1]), Some(1));
}

#[test]
fn tl21_an_upper_case_extension_is_not_found_by_the_scan_deviation() {
    let stack = stack("tl21_upper_case_ext");
    assert_eq!(claims(&stack), []);
    assert_eq!(
        found(&stack),
        [at("IgnoredFile", Info, "textures/blocks/a.PNG:")]
    );
}

#[test]
fn tl22_generate_lists_the_images_sorted_and_once_deviation() {
    // `vanilla = "directory order; textures/blocks/a twice; StyledWriter"`.
    let pack = pack_in(
        &fixture_dir(FAMILY, "tl22_generate").join("pack"),
        1,
        PackOrigin::User,
    );
    assert_eq!(
        TextureList::generate(pack.paths().map(PackPath::as_str)).unwrap(),
        "[\n  \"textures/blocks/a\",\n  \"textures/blocks/d\",\n  \"textures/blocks/f.b\",\n  \"textures/blocks/sub/e\"\n]\n"
    );
}

#[test]
fn tl23_generate_writes_nothing_without_an_image() {
    let dir = fixture_dir(FAMILY, "tl23_generate_nothing").join("pack");
    let pack = pack_in(&dir, 1, PackOrigin::User);
    assert_eq!(
        TextureList::generate(pack.paths().map(PackPath::as_str)),
        None
    );
}

#[test]
fn tl24_a_texture_set_and_its_image_are_one_claim() {
    let stack = stack("tl24_scan_texture_set");
    assert_eq!(claims(&stack), [claim("textures/blocks/c", 0)]);
}

#[test]
fn tl25_the_layers_of_a_texture_set_are_not_claimed() {
    let stack = stack("tl25_scan_layer_filter");
    assert_eq!(
        claims(&stack),
        [claim("textures/blocks/c", 0), claim("textures/blocks/d", 0)]
    );
    // In list mode the layer images are not counted as omitted either.
    let dir = fixture_dir(FAMILY, "tl25_scan_layer_filter").join("pack");
    let mut files = files_below(&dir);
    files.push((
        FILE.to_owned(),
        br#"["textures/blocks/c", "textures/blocks/d"]"#.to_vec(),
    ));
    let files: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(p, b)| (p.as_str(), b.as_slice()))
        .collect();
    assert_eq!(found(&common::stack_of_files(&[&files])), []);
}

#[test]
fn tl25b_a_layer_with_an_extension_filters_nothing() {
    let stack = stack("tl25b_scan_layer_with_extension");
    assert_eq!(
        claims(&stack),
        [
            claim("textures/blocks/c", 0),
            claim("textures/blocks/c_color", 0)
        ]
    );
}

#[test]
fn tl25c_the_texture_set_suffix_needs_no_dot() {
    let stack = stack("tl25c_scan_suffix_without_dot");
    assert_eq!(claims(&stack), [claim("textures/blocks/mytexture_set", 0)]);
}

#[test]
fn tl26_the_subpack_s_directory_is_scanned_first() {
    let ((base, _), (subpack, _)) = base_and_subpack("tl26_scan_subpack_first");
    assert_eq!(base.mode(Some(&subpack)), TextureListMode::Scan);
    // The scan walks the subpack's files, then the pack's.
    let files = files_below(&fixture_dir(FAMILY, "tl26_scan_subpack_first").join("pack"));
    let of = |subpack: bool| {
        files
            .iter()
            .filter_map(move |(path, _)| match path.strip_prefix("subpacks/hi/") {
                Some(key) => subpack.then_some(key),
                None => (!subpack).then_some(path.as_str()),
            })
    };
    let walk: Vec<&str> = of(true).chain(of(false)).collect();
    assert_eq!(scan_claims(walk, &[]), ["textures/b", "textures/a"]);
}

// `resource_packs/vanilla_base/textures/textures_list.json` of the 1.26.45 retail build,
// copied unchanged (CRLF, three-space indent).
#[test]
fn tl27_the_shipped_vanilla_base_list() {
    let stack = stack("tl27_vanilla_base_sample");
    assert_eq!(
        mode(&stack),
        list(&["textures/ui/White", "textures/ui/cross_hair"])
    );
    // V9: `textures/ui/render_dragon` is not listed.
    assert_eq!(found(&stack), [whole()]);
}

#[test]
fn tl28_shipped_entries_with_an_extension() {
    let stack = stack("tl28_vanilla_png_entries");
    // V5 twice, V8 on the second (no such file), V9 for the unlisted `textures/ui/RTX_Label`.
    assert_eq!(found(&stack), [entry(0), entry(1), stale(1), whole()]);
    assert_eq!(
        stack.texture_files().source("textures/ui/RTX_Label.png"),
        image(0, "textures/ui/RTX_Label.png")
    );
}

#[test]
fn tl29_a_texture_set_wins_a_tie_with_its_image() {
    let stack = stack("tl29_texture_set_entry");
    assert_eq!(
        stack.texture_files().source("textures/blocks/stone"),
        Some(TextureSource::TextureSet { stack_index: 0 })
    );
    assert_eq!(found(&stack), []);
}

#[test]
fn a_higher_image_wins_over_a_lower_texture_set() {
    let top: &[(&str, &[u8])] = &[("textures/s.png", b"")];
    let bottom: &[(&str, &[u8])] = &[("textures/s.texture_set.json", b"{}")];
    let stack = common::stack_of_files(&[top, bottom]);
    assert_eq!(
        stack.texture_files().source("textures/s"),
        image(0, "textures/s.png")
    );
    let stack = common::stack_of_files(&[bottom, top]);
    assert_eq!(
        stack.texture_files().source("textures/s"),
        Some(TextureSource::TextureSet { stack_index: 0 })
    );
}

// `resource_packs/vanilla_1.26.10/textures/texture_list.json` of the 1.26.45 retail build,
// copied unchanged (163 entries).
#[test]
fn tl30_the_shipped_misnamed_list_of_a_base_game_pack() {
    let stack = stack_from("tl30_misnamed_shipped", PackOrigin::Package);
    assert_eq!(mode(&stack), TextureListMode::Scan);
    assert_eq!(
        found(&stack),
        [
            at("MissingRequiredField", Info, ""),
            at("IgnoredFile", Info, "textures/texture_list.json:"),
        ]
    );
}

#[test]
fn a_list_is_written_as_an_array_of_its_entries() {
    let bytes = fixture_file(
        FAMILY,
        "tl02_same_list_correct_name",
        &format!("pack/{FILE}"),
    );
    let path = PackPath::new(FILE).unwrap();
    let parse = |bytes: &[u8]| {
        TextureList::parse(&path, bytes, &LoadContext::detached())
            .unwrap()
            .into_value()
    };
    let first = parse(&bytes);
    let written = first.to_json().unwrap();
    // The sample ends without a newline; the rest is byte for byte.
    assert_eq!(written.trim_end().as_bytes(), bytes.trim_ascii_end());
    assert_eq!(parse(written.as_bytes()), first);
    assert_eq!(first.default_path().as_str(), FILE);
}

#[test]
fn the_misnamed_file_beside_the_list_is_no_finding() {
    let pack: &[(&str, &[u8])] = &[
        (FILE, br#"["textures/a"]"#),
        ("textures/texture_list.json", br#"["textures/a"]"#),
        ("textures/a.png", b""),
    ];
    assert_eq!(found(&common::stack_of_files(&[pack])), []);
}

/// `count` images below `textures/blocks/`, as pack files.
fn many_images(count: usize) -> Vec<(String, Vec<u8>)> {
    (0..count)
        .map(|n| (format!("textures/blocks/i{n:05}.png"), Vec::new()))
        .collect()
}

// The list lints look entries and claims up in sets: 20 000 entries over 20 000 images took
// 8 s when each lookup was a scan. Four times the entries may take
// about four times the time (5 000 against 20 000).
#[test]
fn a_list_of_twenty_thousand_entries_is_linted_in_linear_time() {
    let load = |count: usize| {
        let mut files = many_images(count);
        let entries: Vec<String> = (0..count)
            .map(|n| format!("\"textures/blocks/i{n:05}\""))
            .collect();
        files.push((
            FILE.to_owned(),
            format!("[{}]", entries.join(",")).into_bytes(),
        ));
        let files: Vec<(&str, &[u8])> = files
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
            .collect();
        let stack = common::stack_of_files(&[&files]);
        let (textures, elapsed) = cost::timed(|| stack.texture_lists());
        assert_eq!(found(&stack), [], "a complete list");
        assert_eq!(textures.claims().len(), count);
        elapsed
    };
    cost::assert_linear_timed_at("a listed pack", 5_000, load);
}

// The claims of a stack are computed once, at load: a scan of 20 000 images took 2.35 s per
// call The scan itself is linear, and a call hands out the same map.
#[test]
fn the_claims_of_a_scanned_pack_are_computed_once() {
    let load = |count: usize| {
        let files = many_images(count);
        let files: Vec<(&str, &[u8])> = files
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
            .collect();
        let stack = common::stack_of_files(&[&files]);
        let (textures, elapsed) = cost::timed(|| stack.texture_lists());
        let claims = textures.claims();
        assert_eq!(claims.len(), count);
        assert!(
            std::ptr::eq(claims, textures.claims()),
            "one map, not one per call"
        );
        let first = claims.get_index(0).unwrap().1;
        assert_eq!(first.location, "textures/blocks/i00000");
        assert_eq!(first.stack_index, 0);
        elapsed
    };
    cost::assert_linear_timed_at("a scanned pack", 5_000, load);
}
