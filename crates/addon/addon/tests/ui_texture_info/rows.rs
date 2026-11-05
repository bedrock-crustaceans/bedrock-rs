//! `textures/ui/*.json`, the side files of UI images. The file's reader is `UiTextureInfo`'s;
//! the stack rules (the highest pack's side file) are `rp.ui_texture_infos()`.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Document};
use bedrock_addon::families::ui_texture_info::{SliceSize, UiTextureInfo};
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;

use super::common::{codes, json, located, pack_number, shown, stack_of, subpack_stacks};

fn fixture(path: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ui/rp_misc");
    std::fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn parse(text: &str) -> Document<UiTextureInfo> {
    UiTextureInfo::parse(
        &PackPath::from_static("textures/ui/x.json"),
        text.as_bytes(),
        &LoadContext::detached(),
    )
    .expect("a side file")
}

fn nineslice(doc: &Document<UiTextureInfo>) -> (Option<SliceSize>, Option<[f32; 2]>) {
    match doc.value() {
        UiTextureInfo::Nineslice(nineslice) => (nineslice.nineslice_size, nineslice.base_size),
        other => panic!("a nine-slice description, not {other:?}"),
    }
}

fn texts(doc: &Document<UiTextureInfo>) -> Vec<(&'static str, Severity, String)> {
    doc.diagnostics().iter().map(shown).collect()
}

#[test]
fn ui01_one_width_for_all_sides() {
    let doc = parse(r#"{ "nineslice_size": 1, "base_size": [4, 4] }"#);
    assert_eq!(
        nineslice(&doc),
        (Some(SliceSize::All(1.0)), Some([4.0, 4.0]))
    );
    assert_eq!(codes(doc.diagnostics()), []);
    assert_eq!(doc.base_size(), [4.0, 4.0]);
}

#[test]
fn ui02_four_widths() {
    let doc = parse(r#"{ "nineslice_size": [1, 2, 1, 2], "base_size": [3, 5] }"#);
    assert_eq!(
        nineslice(&doc),
        (
            Some(SliceSize::Sides([1.0, 2.0, 1.0, 2.0])),
            Some([3.0, 5.0])
        )
    );
    assert_eq!(codes(doc.diagnostics()), []);
}

#[test]
fn ui03_t2_a_slice_size_of_another_shape_is_ignored() {
    for written in ["[1, 2, 3]", "[1, 2, 3, 4, 5]", "[]", "\"4\"", "{}"] {
        let doc = parse(&format!(
            r#"{{ "nineslice_size": {written}, "base_size": [3, 5] }}"#
        ));
        assert_eq!(nineslice(&doc), (None, Some([3.0, 5.0])), "{written}");
        assert_eq!(
            located(doc.diagnostics()),
            [("InvalidValue", Severity::Info, "nineslice_size".to_owned())],
            "{written}"
        );
    }
}

#[test]
fn ui04_t4_a_negative_base_size_is_kept() {
    let doc = parse(r#"{ "base_size": [-1, 4] }"#);
    assert_eq!(nineslice(&doc).1, Some([-1.0, 4.0]));
    assert_eq!(
        texts(&doc),
        [(
            "InvalidValue",
            Severity::Info,
            "UV Base size can't be negative!".to_owned()
        )]
    );
    // Each component counts, and zero is not negative.
    assert_eq!(texts(&parse(r#"{ "base_size": [4, -0.5] }"#)).len(), 1);
    assert_eq!(codes(parse(r#"{ "base_size": [0, 0] }"#).diagnostics()), []);
}

#[test]
fn t3_a_base_size_of_another_shape_is_ignored() {
    for written in ["[4]", "[]", "4", "\"4x4\"", "{}"] {
        let doc = parse(&format!(r#"{{ "base_size": {written} }}"#));
        assert_eq!(nineslice(&doc).1, None, "{written}");
        assert_eq!(
            located(doc.diagnostics()),
            [("InvalidValue", Severity::Info, "base_size".to_owned())],
            "{written}"
        );
    }
    // Two elements are enough; more are not read.
    assert_eq!(
        nineslice(&parse(r#"{ "base_size": [4, 5, 6] }"#)).1,
        Some([4.0, 5.0])
    );
}

#[test]
fn ui05_t7_the_vanilla_tiled_page_edge_deviation() {
    // gdk-retail 1.26.45 resource_packs/vanilla/textures/ui/book_pageedge_left.json, unchanged.
    let text = fixture("ui05_tiled/pack/textures/ui/book_pageedge_left.json");
    let doc = parse(&text);
    assert_eq!(codes(doc.diagnostics()), [], "tiled is no finding");
    let UiTextureInfo::Nineslice(nineslice) = doc.value() else {
        panic!("a nine-slice")
    };
    assert_eq!(nineslice.tiled, Some(true));
    assert_eq!(
        nineslice.nineslice_size,
        Some(SliceSize::Sides([40.0, 40.0, 0.0, 40.0]))
    );
    assert_eq!(json(&doc.to_json().unwrap()), json(&text));
}

#[test]
fn ui06_the_vanilla_jump_animation() {
    // gdk-retail 1.26.45 resource_packs/vanilla/textures/ui/game_tip_animations/jump-mouse.json.
    let text = fixture("ui06_aseprite/pack/textures/ui/game_tip_animations/jump-mouse.json");
    let doc = parse(&text);
    assert_eq!(codes(doc.diagnostics()), []);
    let UiTextureInfo::Aseprite(sheet) = doc.value() else {
        panic!("an Aseprite sheet")
    };
    assert_eq!(sheet.meta.size, [420, 64]);
    assert_eq!(doc.base_size(), [420.0, 64.0], "the base size is meta.size");
    assert_eq!(doc.frames().len(), 5);
    assert_eq!(doc.frames()[1].position, [84, 0]);
    assert!(doc.total_duration() >= 1100);
    assert_eq!(json(&doc.to_json().unwrap()), json(&text));
}

const SHEET: &str = r#"{ "frames": [{ "frame": { "x": 0, "y": 0, "w": 8, "h": 8 }, "duration": 100 }, { "frame": { "x": 8, "y": 0, "w": 8, "h": 8 }, "duration": 50 }], "meta": { "image": "x.png", "size": { "w": 16, "h": 8 } } }"#;

fn rejected_sheet(text: &str) {
    let doc = parse(text);
    assert!(
        matches!(doc.value(), UiTextureInfo::RejectedAseprite(_)),
        "{text}"
    );
    assert_eq!(doc.base_size(), [0.0, 0.0], "{text}");
    assert!(doc.frames().is_empty(), "{text}");
    assert_eq!(
        codes(doc.diagnostics()),
        [("InvalidValue", Severity::Info)],
        "{text}"
    );
    assert_eq!(
        json(&doc.to_json().unwrap()),
        json(text),
        "written back as it was"
    );
}

#[test]
fn ui07_t5_the_hash_export_is_rejected() {
    rejected_sheet(
        r#"{ "frames": { "a.png": { "frame": { "x": 0, "y": 0 }, "duration": 100 } }, "meta": { "image": "x.png", "size": { "w": 16, "h": 8 } } }"#,
    );
    let doc = parse(
        r#"{ "frames": { "a.png": {} }, "meta": { "image": "x.png", "size": { "w": 16, "h": 8 } } }"#,
    );
    assert_eq!(
        located(doc.diagnostics())[0].2,
        "frames",
        "the finding names the failing member"
    );
}

#[test]
fn ui12_t5_the_tests_of_a_sheet() {
    // Accepted: w and h of a frame are not read, and an empty frame list is a sheet.
    let doc = parse(&SHEET.replace(r#", "w": 8, "h": 8"#, ""));
    assert!(matches!(doc.value(), UiTextureInfo::Aseprite(_)));
    assert_eq!(
        (doc.frames().len(), doc.total_duration(), doc.base_size()),
        (2, 150, [16.0, 8.0])
    );
    assert_eq!(codes(doc.diagnostics()), []);
    let doc =
        parse(r#"{ "frames": [], "meta": { "image": "x.png", "size": { "w": 16, "h": 8 } } }"#);
    assert!(matches!(doc.value(), UiTextureInfo::Aseprite(_)));
    assert_eq!(doc.base_size(), [16.0, 8.0]);
    // The boundaries: a duration of 1 and a position of 0 pass.
    let doc = parse(&SHEET.replace("\"duration\": 100", "\"duration\": 1"));
    assert!(matches!(doc.value(), UiTextureInfo::Aseprite(_)));
    // Rejected, one failing test each.
    for (from, to) in [
        ("\"duration\": 100", "\"duration\": 0"),
        ("\"duration\": 100", "\"duration\": -5"),
        ("\"duration\": 100", "\"duration\": \"100\""),
        ("\"duration\": 50", "\"duration\": 1.5"),
        ("\"x\": 0", "\"x\": -1"),
        (
            "\"y\": 0, \"w\": 8, \"h\": 8 }, \"duration\": 50",
            "\"y\": -1 }, \"duration\": 50",
        ),
        ("\"x\": 8", "\"x\": \"8\""),
        (
            "\"frame\": { \"x\": 0, \"y\": 0, \"w\": 8, \"h\": 8 }",
            "\"frame\": 3",
        ),
        ("\"image\": \"x.png\"", "\"image\": 3"),
        ("\"image\": \"x.png\", ", ""),
        ("\"size\": { \"w\": 16, \"h\": 8 }", "\"size\": [16, 8]"),
        ("\"w\": 16", "\"w\": 16.5"),
        ("\"h\": 8 } }", "\"h\": \"8\" } }"),
        (
            "\"meta\": { \"image\": \"x.png\", \"size\": { \"w\": 16, \"h\": 8 } }",
            "\"meta\": 4",
        ),
    ] {
        assert!(SHEET.contains(from), "{from}");
        rejected_sheet(&SHEET.replacen(from, to, 1));
    }
    // A null `meta` is no sheet at all: a nine-slice description.
    let doc = parse(r#"{ "meta": null, "nineslice_size": 2 }"#);
    assert_eq!(nineslice(&doc).0, Some(SliceSize::All(2.0)));
}

#[test]
fn ui10_t2c_a_negative_slice_size_is_kept() {
    let doc = parse(r#"{ "nineslice_size": [-1, 2, 1, 2] }"#);
    assert_eq!(
        nineslice(&doc).0,
        Some(SliceSize::Sides([-1.0, 2.0, 1.0, 2.0]))
    );
    assert_eq!(
        texts(&doc),
        [(
            "InvalidValue",
            Severity::Info,
            "NineSlice cannot be negative".to_owned()
        )]
    );
    let doc = parse(r#"{ "nineslice_size": -0.5 }"#);
    assert_eq!(nineslice(&doc).0, Some(SliceSize::All(-0.5)));
    assert_eq!(texts(&doc).len(), 1);
    assert_eq!(
        codes(parse(r#"{ "nineslice_size": [0, 0, 0, 0] }"#).diagnostics()),
        []
    );
    // The last side counts as the first does.
    assert_eq!(
        texts(&parse(r#"{ "nineslice_size": [1, 2, 1, -2] }"#)).len(),
        1
    );
}

#[test]
fn ui11_t2b_a_boolean_slice_size_is_a_number_deviation() {
    for (written, read) in [("true", 1.0), ("false", 0.0)] {
        let doc = parse(&format!(r#"{{ "nineslice_size": {written} }}"#));
        assert_eq!(nineslice(&doc).0, Some(SliceSize::All(read)), "{written}");
        assert_eq!(
            located(doc.diagnostics()),
            [("InvalidValue", Severity::Info, "nineslice_size".to_owned())],
            "{written}"
        );
    }
}

#[test]
fn t1_a_side_file_that_is_not_json() {
    let rejected = UiTextureInfo::parse(
        &PackPath::from_static("textures/ui/x.json"),
        b"{ nope",
        &LoadContext::detached(),
    )
    .unwrap_err();
    assert_eq!(codes(&rejected.diagnostics)[0].0, "MalformedInput");
    // Through a stack: the image is left without a side file; a lower pack's copy does not
    // stand in (two and three packs).
    let good: &[(&str, &str)] = &[
        ("textures/ui/x.json", r#"{ "nineslice_size": 2 }"#),
        ("textures/ui/x.png", ""),
    ];
    for lower in [1, 2] {
        let mut packs = vec![good; lower];
        packs.push(&[("textures/ui/x.json", "{ nope")]);
        let stack = stack_of(&packs);
        let infos = stack.ui_texture_infos();
        assert!(
            infos
                .get(&PackPath::from_static("textures/ui/x.json"))
                .is_none()
        );
        let found: Vec<_> = infos.diagnostics().collect();
        assert_eq!(
            codes(found.iter().copied()),
            [("MalformedInput", Severity::Error)]
        );
        assert_eq!(pack_number(found[0], lower + 1), Some(lower));
        assert_eq!(
            found[0].location.path.as_ref().unwrap().as_str(),
            "textures/ui/x.json"
        );
    }
}

#[test]
fn ui08_the_highest_pack_s_side_file_applies_whichever_pack_has_the_image() {
    let stack = stack_of(&[
        &[
            ("textures/ui/x.png", ""),
            ("textures/ui/x.json", r#"{ "nineslice_size": 1 }"#),
        ],
        &[("textures/ui/other.png", "")],
        &[(
            "textures/ui/x.json",
            r#"{ "nineslice_size": 3, "base_size": [-1, 1] }"#,
        )],
    ]);
    let infos = stack.ui_texture_infos();
    let side = infos
        .get(&PackPath::from_static("textures/ui/x.json"))
        .expect("a side file");
    assert_eq!(nineslice(side).0, Some(SliceSize::All(3.0)));
    assert_eq!(side.origin().pack.index(), Some(0));
    let found: Vec<_> = infos.diagnostics().collect();
    assert_eq!(
        codes(found.iter().copied()),
        [("InvalidValue", Severity::Info)],
        "the side file's own finding only"
    );
    assert_eq!(pack_number(found[0], 3), Some(2));
    assert_eq!(infos.iter().count(), 1);
}

#[test]
fn ui09_t6_a_side_file_without_an_image_in_any_pack() {
    let stack = stack_of(&[
        &[
            ("textures/ui/x.json", "{}"),
            ("textures/ui/sub/y.json", "{}"),
            ("textures/ui/y.png", ""),
        ],
        &[
            ("textures/ui/x.json", "{}"),
            ("textures/ui/z.json", "{}"),
            ("textures/ui/z.json.png", ""),
            ("textures/x.png", ""),
        ],
    ]);
    let infos = stack.ui_texture_infos();
    let found: Vec<(usize, String)> = infos
        .diagnostics()
        .map(|d| {
            assert_eq!((d.code.name(), d.severity), ("IgnoredFile", Severity::Info));
            (
                pack_number(d, 2).unwrap(),
                d.location.path.as_ref().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            (0, "textures/ui/sub/y.json".to_owned()),
            (1, "textures/ui/x.json".to_owned()),
            (1, "textures/ui/z.json".to_owned()),
        ],
        "once per path, at the highest copy; an image of another folder or name is none"
    );
    assert!(
        infos
            .get(&PackPath::from_static("textures/ui/x.json"))
            .is_some(),
        "the file is still read"
    );
}

#[test]
fn t6_every_image_extension_and_the_bare_name_count() {
    for image in ["x.tga", "x.png", "x.jpg", "x.jpeg", "x.hdr", "x.ktx", "x"] {
        let path = format!("textures/ui/{image}");
        let stack = stack_of(&[&[(path.as_str(), "")], &[("textures/ui/x.json", "{}")]]);
        assert_eq!(codes(stack.ui_texture_infos().diagnostics()), [], "{image}");
    }
    for other in ["x.PNG", "x.gif", "x.png.bak", "xx.png", "x.json.png"] {
        let path = format!("textures/ui/{other}");
        let stack = stack_of(&[&[(path.as_str(), ""), ("textures/ui/x.json", "{}")]]);
        assert_eq!(
            codes(stack.ui_texture_infos().diagnostics()),
            [("IgnoredFile", Severity::Info)],
            "{other}"
        );
    }
}

#[test]
fn su01_the_active_subpack_s_copies_are_used() {
    let set = subpack_stacks(&[
        ("splashes.json", br#"{ "splashes": ["root"] }"#),
        ("subpacks/hi/splashes.json", br#"{ "splashes": ["sub"] }"#),
        ("textures/ui/Banners.json", br#"{ "nineslice_size": 1 }"#),
        (
            "subpacks/hi/textures/ui/Banners.json",
            br#"{ "nineslice_size": 2 }"#,
        ),
        ("textures/ui/Banners.png", b""),
    ]);
    assert_eq!(set.resource.splashes().texts(), ["sub"]);
    let infos = set.resource.ui_texture_infos();
    let side = infos
        .get(&PackPath::from_static("textures/ui/Banners.json"))
        .expect("keyed without the subpack folder");
    assert_eq!(nineslice(side).0, Some(SliceSize::All(2.0)));
    assert_eq!(
        side.origin().path.as_str(),
        "subpacks/hi/textures/ui/Banners.json"
    );
    assert_eq!(codes(infos.diagnostics()), []);
    assert_eq!(infos.iter().count(), 1);
}

#[test]
fn every_finding_names_its_pack_and_file() {
    // A slice size of another shape is the crate's own information; the file loads.
    let stack = stack_of(&[&[
        ("textures/ui/a.png", ""),
        ("textures/ui/a.json", r#"{ "nineslice_size": "x" }"#),
        ("textures/ui/b.png", ""),
        ("textures/ui/b.json", "{"),
    ]]);
    let name = stack
        .ui_texture_infos()
        .diagnostics()
        .next()
        .map(|finding| {
            finding
                .location
                .pack
                .as_ref()
                .map(|p| p.name().to_owned())
                .unwrap_or_default()
        });
    assert_in_files(
        stack.ui_texture_infos().diagnostics(),
        name.as_deref().unwrap_or("a pack"),
        &["textures/ui/a.json", "textures/ui/b.json"],
    );
}

/// Every finding of `findings` names the pack `pack` and one of `files`, and each of `files`
/// has one: the loader gives each finding its document's pack and file (phase 1 step 4 pin).
fn assert_in_files<'a>(
    findings: impl IntoIterator<Item = &'a bedrock_addon::diagnostic::Diagnostic>,
    pack: &str,
    files: &[&str],
) {
    let mut named = Vec::new();
    for finding in findings {
        let in_pack = finding
            .location
            .pack
            .as_ref()
            .map(bedrock_addon::pack::PackRef::name);
        assert_eq!(in_pack, Some(pack), "{finding:?}");
        let path = finding
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str);
        assert!(path.is_some_and(|p| files.contains(&p)), "{finding:?}");
        named.extend(path.map(str::to_owned));
    }
    for file in files {
        assert!(named.iter().any(|p| p == file), "no finding names {file}");
    }
}
