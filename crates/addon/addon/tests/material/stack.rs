//! The materials of a stack (`rp.materials()`): the lists, the merge of each file over the
//! packs, and the creation of the materials.

use bedrock_addon::diagnostic::Severity::{Error, Info};
use bedrock_addon::diagnostic::{Diagnostic, DiagnosticDetail};
use bedrock_addon::families::material::{
    KeyRules, MaterialFileForm, MaterialGroupKind, MaterialKey, MaterialListKind, Materials,
};
use bedrock_addon::id::MaterialId;
use bedrock_addon::pack::{PackStack, Resource};
use bedrock_addon::storage::PackPath;

use crate::common::{Files, Rendered, all, finding, fixture_text, flat, names, pack_name, stack};

const COMMON: &str = "materials/common.json";
const FANCY: &str = "materials/fancy.json";
const SAD: &str = "materials/sad.json";
const A: &str = "materials/a.material";
const B: &str = "materials/b.material";
const ENTITY: &str = "materials/entity.material";
const SPRITE: &str = "materials/cameraFacingSprite.material";
const BARRIER: &str = "materials/barrier.material";

/// The vanilla files the fixtures hold (gdk-retail 1.26.45, copied unchanged): the three
/// lists, `entity.material`, `cameraFacingSprite.material` and `sky.material`. The lists name
/// more files than these; a path no pack has loads nothing.
fn vanilla() -> Vec<(&'static str, String)> {
    vec![
        (COMMON, fixture_text("vanilla/common.json")),
        (FANCY, fixture_text("vanilla/fancy.json")),
        (SAD, fixture_text("vanilla/sad.json")),
        (ENTITY, fixture_text("vanilla/entity.material")),
        (SPRITE, fixture_text("vanilla/cameraFacingSprite.material")),
        (
            "materials/sky.material",
            fixture_text("vanilla/sky.material"),
        ),
    ]
}

/// The stack of `vanilla()` below `packs` (lowest first).
fn over_vanilla(packs: &[Files<'_>]) -> PackStack<Resource> {
    let vanilla = vanilla();
    let vanilla: Vec<(&str, &str)> = vanilla.iter().map(|(p, t)| (*p, t.as_str())).collect();
    let mut all: Vec<Files<'_>> = vec![&vanilla];
    all.extend(packs);
    stack(&all)
}

/// A list of the paths `paths`, without defines.
fn list(paths: &[&str]) -> String {
    let entries: Vec<String> = paths
        .iter()
        .map(|path| format!(r#"{{"path": "{path}"}}"#))
        .collect();
    format!("[{}]", entries.join(", "))
}

/// A wrapped `1.0.0` file of `members`.
fn wrapped(members: &str) -> String {
    crate::common::wrapped(members)
}

/// The findings of the group with the pack and file each names.
fn placed(materials: &Materials) -> Vec<(Rendered, Option<String>, Option<String>)> {
    materials
        .diagnostics()
        .map(|finding| {
            let rendered = crate::common::rendered([finding]).remove(0);
            let path = finding.location.path.as_ref().map(ToString::to_string);
            (rendered, pack_name(finding), path)
        })
        .collect()
}

fn id(name: &str) -> MaterialId {
    MaterialId::parse(name)
}

fn path(text: &str) -> PackPath {
    PackPath::new(text).unwrap()
}

/// The reference a finding names.
fn reference(finding: &Diagnostic) -> Option<(&'static str, String)> {
    match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => Some((*kind, id.clone())),
        _ => None,
    }
}

#[test]
fn m01_the_vanilla_entity_file_under_the_vanilla_lists() {
    let stack = over_vanilla(&[]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    let fancy = &materials.fancy;
    for name in [
        "entity_static",
        "entity_static.skinning",
        "entity_static.skinning_color",
        "entity_static.skinning_alphatest",
        "entity",
        "entity.skinning",
        "entity_alphatest",
        "entity_alphatest.skinning_color",
        "camera_facing_sprite",
    ] {
        assert!(fancy.contains(name), "{name}");
        assert!(materials.sad.contains(name), "{name}");
        assert!(materials.contains(&id(name)), "{name}");
    }
    // The entity file is no file of the common list; the sprite file is one of all three.
    assert!(!materials.common.contains("entity_static"));
    assert!(materials.common.contains("camera_facing_sprite"));
    assert!(materials.common.contains("sun_moon"));
    assert!(!fancy.contains("sun_moon"));
    // One material per key of the file, each a link that names its parent.
    let entity_file = &materials.files[&path(ENTITY)];
    assert_eq!(fancy.len(), entity_file.materials.len() + 1);
    assert_eq!(fancy.len(), materials.sad.len());
    let entity = fancy.get("entity").expect("entity");
    assert_eq!(entity.parent.as_deref(), Some("entity_static"));
    assert_eq!(entity.file.as_str(), ENTITY);
    assert_eq!(entity.key_rules, KeyRules::Versioned);
    assert_eq!(
        entity.variants().collect::<Vec<_>>(),
        ["skinning", "skinning_alphatest", "skinning_color"]
    );
    assert_eq!(entity.variant_count(), 3);
    assert!(entity.has_variant("skinning") && !entity.has_variant("skin"));
    assert_eq!(fancy.get("entity_static").unwrap().parent, None);
    // A variant is a name of the group and no material of its own.
    assert!(fancy.get("entity.skinning").is_none());
    assert!(materials.get("entity.skinning").is_none());
    assert!(!fancy.contains("entity.nope") && !fancy.contains("nope.skinning"));
    assert!(!materials.contains(&id("no_such_material")));
    // `get` looks in the common group first, then fancy, then sad.
    assert_eq!(
        materials.get("sun_moon").unwrap().file.as_str(),
        "materials/sky.material"
    );
    assert_eq!(materials.get("entity").unwrap().file.as_str(), ENTITY);
    assert_eq!(
        materials.group(MaterialGroupKind::Sad).kind,
        MaterialGroupKind::Sad
    );
    assert_eq!(
        materials.group(MaterialGroupKind::Common).len(),
        materials.common.len()
    );
    assert!(!fancy.is_empty());
    // The lists of the pack, by path; every walked file is a merged file.
    let kinds: Vec<MaterialListKind> = materials.lists.iter().map(|list| list.kind).collect();
    assert_eq!(
        kinds,
        [
            MaterialListKind::Common,
            MaterialListKind::Fancy,
            MaterialListKind::Sad
        ]
    );
    assert_eq!(materials.files.len(), 3);
}

#[test]
fn s01_a_pack_adds_materials_to_a_vanilla_file_as_chemistry_does() {
    let plain = over_vanilla(&[]);
    let chemistry = fixture_text("vanilla/chemistry/entity.material");
    let stack = over_vanilla(&[&[(ENTITY, &chemistry)]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    let mut expected = names(&plain.materials().fancy);
    // The retail 1.26.45 vanilla file has `dolphin:entity` itself: the pack's key merges with
    // it. The other two are the pack's own.
    assert!(plain.materials().fancy.contains("dolphin"));
    assert_eq!(
        materials.fancy.get("dolphin").unwrap().parent.as_deref(),
        Some("entity")
    );
    for (name, parent) in [
        ("balloon", "entity_change_color"),
        ("ice_bomb", "entity_alphatest"),
    ] {
        assert!(!expected.contains(&name.to_owned()));
        expected.push(name.to_owned());
        let material = materials.fancy.get(name).expect(name);
        assert_eq!(material.parent.as_deref(), Some(parent));
        // Each inherits its parent's variants.
        let inherited: Vec<&str> = materials.fancy.get(parent).unwrap().variants().collect();
        assert_eq!(material.variants().collect::<Vec<_>>(), inherited);
        for variant in inherited {
            expected.push(format!("{name}.{variant}"));
        }
    }
    expected.sort();
    assert_eq!(names(&materials.fancy), expected);
    // The merged file: two layers, the pack's first, and each key from the pack that wrote it.
    let merged = &materials.files[&path(ENTITY)];
    assert_eq!(merged.layers().len(), 2);
    assert_eq!(merged.layers()[0].origin.pack.name(), "pack 2");
    let from = |key: &str| {
        merged
            .origin(&MaterialKey::parse(key))
            .map(|origin| origin.pack.name().to_owned())
    };
    assert_eq!(from("dolphin:entity").as_deref(), Some("pack 2"));
    assert_eq!(from("entity:entity_static").as_deref(), Some("pack 1"));
    assert_eq!(from("nope"), None);
    assert!(matches!(merged.form, MaterialFileForm::Wrapped { .. }));
}

#[test]
fn s02_s03_a_pack_overrides_a_member_and_an_array_by_index() {
    let pack = wrapped(
        r#""entity_alphatest:entity_nocull": {"msaaSupport": "MSAA", "+defines": ["USE_EMISSIVE"]},
           "three": {"+defines": ["X"]}"#,
    );
    let below = wrapped(r#""three": {"+defines": ["A", "B", "C"], "depthFunc": "Less"}"#);
    let stack = over_vanilla(&[&[(A, &below)], &[(ENTITY, &pack), (A, &pack)]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    let merged = &materials.files[&path(ENTITY)];
    let key = MaterialKey::parse("entity_alphatest:entity_nocull");
    let def = &merged.materials[&key];
    assert_eq!(def.msaa_support.as_deref(), Some("MSAA"));
    // Vanilla's `+defines` has one entry, `ALPHA_TEST`: the pack's entry replaces it.
    assert_eq!(def.defines.add, Some(vec!["USE_EMISSIVE".to_owned()]));
    // The lower members stay.
    assert!(def.sampler_states.add.is_some());
    // The game's patching overrides by index and keeps the lower tail: vanilla's first entry is lost.
    let three = &materials.files[&path(A)].materials[&MaterialKey::new("three")];
    assert_eq!(
        three.defines.add,
        Some(vec!["X".to_owned(), "B".to_owned(), "C".to_owned()])
    );
    assert_eq!(three.depth_func.as_deref(), Some("Less"));
}

#[test]
fn s04_a_packs_list_adds_to_the_lists_below() {
    let mine = wrapped(r#""mine": {}"#);
    let stack = over_vanilla(&[&[
        (COMMON, &list(&["materials/my.material"])),
        ("materials/my.material", &mine),
    ]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    assert!(materials.common.contains("mine"));
    assert!(materials.common.contains("sun_moon"));
    assert!(materials.common.contains("camera_facing_sprite"));
    assert!(!materials.fancy.contains("mine"));
    assert_eq!(
        materials.common.get("mine").unwrap().file.as_str(),
        "materials/my.material"
    );
    // The pack's list first.
    let lists: Vec<(String, MaterialListKind)> = materials
        .lists
        .iter()
        .map(|list| (list.origin().pack.name().to_owned(), list.kind))
        .collect();
    assert_eq!(lists[0], ("pack 2".to_owned(), MaterialListKind::Common));
    assert_eq!(lists.len(), 4);
    // Its material comes first in the group: the highest list is processed first.
    assert_eq!(materials.common.iter().next().unwrap().name, "mine");
}

#[test]
fn s05_a_copy_that_is_no_json_is_skipped_and_the_other_copies_merge() {
    let stack = over_vanilla(&[
        &[(ENTITY, "{ not json")],
        &[(ENTITY, &wrapped(r#""top:entity": {}"#))],
    ]);
    let materials = stack.materials();
    let found = placed(materials);
    assert_eq!(found.len(), 1);
    let ((code, severity, at, text), pack, file) = &found[0];
    assert_eq!(
        (*code, *severity, at.as_str()),
        ("MalformedInput", Error, "")
    );
    assert!(
        text.starts_with("Error parsing materials/entity.material :\n "),
        "{text}"
    );
    assert_eq!(pack.as_deref(), Some("pack 2"));
    assert_eq!(file.as_deref(), Some(ENTITY));
    // Vanilla's materials and the top pack's are there.
    assert!(materials.fancy.contains("entity_alphatest"));
    assert!(materials.fancy.contains("top"));
    assert_eq!(materials.files[&path(ENTITY)].layers().len(), 2);
}

/// `experimental_y_2026_drop_4/materials/entity.material` of 1.26.60 is an empty file.
/// The loader reads every copy of the path
/// and gives each to the JSON reader, which refuses an empty
/// text ("value, object or array expected"), so the drop's copy is V1's "Error parsing %s :\n %s"
/// (Error, Json 17) and is skipped, and vanilla's copy below it still loads.
#[test]
fn the_drop_4_empty_entity_material_is_skipped_with_the_parse_error() {
    let empty = fixture_text("drop4/entity.material");
    assert!(empty.is_empty());
    let stack = over_vanilla(&[&[(ENTITY, &empty)]]);
    let materials = stack.materials();
    let found = placed(materials);
    assert_eq!(found.len(), 1, "{found:#?}");
    let ((code, severity, at, text), pack, file) = &found[0];
    assert_eq!(
        (*code, *severity, at.as_str()),
        ("MalformedInput", Error, "")
    );
    assert!(
        text.starts_with("Error parsing materials/entity.material :\n "),
        "{text}"
    );
    assert_eq!(pack.as_deref(), Some("pack 2"));
    assert_eq!(file.as_deref(), Some(ENTITY));
    assert!(materials.fancy.contains("entity_alphatest"));
    assert_eq!(materials.files[&path(ENTITY)].layers().len(), 1);
}

#[test]
fn a_copy_whose_root_is_no_object_is_skipped_with_an_error() {
    let stack = stack(&[
        &[(COMMON, &list(&[A])), (A, &wrapped(r#""x": {}"#))],
        &[(A, "[1, 2]")],
    ]);
    let materials = stack.materials();
    assert_eq!(
        all(materials),
        [finding("InvalidType", Error, "", "wrong JSON type")]
    );
    assert!(materials.common.contains("x"));
}

#[test]
fn s06_v20_a_key_that_drops_the_parent_of_the_key_below_is_reported_deviation() {
    let pack = wrapped(r#""entity_alphatest": {"msaaSupport": "MSAA"}"#);
    let stack = over_vanilla(&[&[(ENTITY, &pack)]]);
    let materials = stack.materials();
    assert_eq!(
        placed(materials),
        [(
            finding(
                "ShadowedDefinition",
                Info,
                "materials/entity_alphatest",
                "definition shadowed by a higher-priority source"
            ),
            Some("pack 2".to_owned()),
            Some(ENTITY.to_owned())
        )]
    );
    // The merged key is the pack's, without a parent; the material is still created, and its
    // children with it.
    let merged = &materials.files[&path(ENTITY)];
    assert!(
        merged
            .materials
            .contains_key(&MaterialKey::new("entity_alphatest"))
    );
    assert!(
        !merged
            .materials
            .contains_key(&MaterialKey::parse("entity_alphatest:entity_nocull"))
    );
    let material = materials.fancy.get("entity_alphatest").expect("indexed");
    assert_eq!(material.parent, None);
    assert_eq!(material.variant_count(), 0);
    assert!(materials.fancy.contains("map"));
}

#[test]
fn v20_a_changed_parent_is_reported_once_by_the_pack_that_changes_it_deviation() {
    let files = |key: &str| wrapped(&format!(r#""p": {{}}, "q": {{}}, "{key}": {{}}"#));
    let (low, mid, top) = (files("x:p"), files("x:q"), files("x:q"));
    let stack = stack(&[
        &[(COMMON, &list(&[A])), (A, &low)],
        &[(A, &mid)],
        &[(A, &top)],
    ]);
    let found = placed(stack.materials());
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0.0, "ShadowedDefinition");
    assert_eq!(found[0].0.2, "materials/x:q");
    assert_eq!(found[0].1.as_deref(), Some("pack 2"));
    assert_eq!(
        stack.materials().common.get("x").unwrap().parent.as_deref(),
        Some("q")
    );
    // The same key over the same key says nothing, in two packs and in three.
    let same = stack_of_same("x:p");
    assert_eq!(all(same.materials()), []);
}

fn stack_of_same(key: &str) -> PackStack<Resource> {
    let file = wrapped(&format!(r#""p": {{}}, "{key}": {{}}"#));
    stack(&[
        &[(COMMON, &list(&[A])), (A, &file)],
        &[(A, &file)],
        &[(A, &file)],
    ])
}

#[test]
fn g07_v17_a_barrier_file_no_list_asks_for_is_not_loaded_deviation() {
    let barrier = wrapped(r#""barrier_only": {}"#);
    let plain = over_vanilla(&[]);
    let stack = over_vanilla(&[&[(BARRIER, &barrier)]]);
    let materials = stack.materials();
    assert_eq!(
        placed(materials),
        [(
            finding("IgnoredFile", Info, "", "file ignored"),
            Some("pack 2".to_owned()),
            Some(BARRIER.to_owned())
        )]
    );
    for kind in MaterialGroupKind::ALL {
        assert_eq!(
            names(materials.group(kind)),
            names(plain.materials().group(kind))
        );
    }
    assert!(!materials.contains(&id("barrier_only")));
    // The file is merged for whoever wants to look at it.
    assert!(materials.files.contains_key(&path(BARRIER)));
    // One warning per pack that ships one.
    let two = over_vanilla(&[&[(BARRIER, &barrier)], &[(BARRIER, &barrier)]]);
    let packs: Vec<Option<String>> = placed(two.materials()).into_iter().map(|f| f.1).collect();
    assert_eq!(
        packs,
        [Some("pack 2".to_owned()), Some("pack 3".to_owned())]
    );
}

#[test]
fn g08_a_request_for_the_barrier_file_falls_back_to_the_sprite_file_of_each_pack_deviation() {
    let stack = over_vanilla(&[&[(COMMON, &list(&[BARRIER]))]]);
    let materials = stack.materials();
    assert_eq!(
        placed(materials),
        [
            (
                finding(
                    "ShadowedDefinition",
                    Info,
                    "",
                    "definition shadowed by a higher-priority source"
                ),
                Some("pack 1".to_owned()),
                Some(SPRITE.to_owned())
            ),
            // Vanilla's own list then asks for the sprite file, whose material the group
            // already has from the barrier request (V18).
            (
                finding(
                    "DuplicateIdentifier",
                    Info,
                    "materials/camera_facing_sprite",
                    "duplicate identifier"
                ),
                Some("pack 1".to_owned()),
                Some(SPRITE.to_owned())
            ),
        ]
    );
    assert!(materials.common.contains("camera_facing_sprite"));
    // The merge is filed under the path that was asked for.
    assert_eq!(
        materials.common.iter().next().unwrap().file.as_str(),
        BARRIER
    );
    let merged = &materials.files[&path(BARRIER)];
    assert_eq!(merged.layers().len(), 1);
    assert_eq!(merged.layers()[0].origin.path.as_str(), SPRITE);
}

#[test]
fn g08_a_pack_with_its_own_barrier_file_contributes_that_one() {
    // Three packs: the sprite file below, a barrier file in the middle, a sprite file on top.
    let low = wrapped(r#""low": {}, "shared": {"depthFunc": "Less"}"#);
    let mid = wrapped(r#""mid": {}, "shared": {"depthFunc": "Always"}"#);
    let top = wrapped(r#""top": {}"#);
    let stack = stack(&[
        &[(COMMON, &list(&[BARRIER])), (SPRITE, &low)],
        &[(BARRIER, &mid), (SPRITE, &wrapped(r#""unused": {}"#))],
        &[(SPRITE, &top)],
    ]);
    let materials = stack.materials();
    let found = placed(materials);
    let fallbacks: Vec<_> = found.iter().map(|f| (f.0.0, f.1.as_deref())).collect();
    assert_eq!(
        fallbacks,
        [
            ("ShadowedDefinition", Some("pack 1")),
            ("ShadowedDefinition", Some("pack 3"))
        ]
    );
    assert_eq!(names(&materials.common), ["low", "mid", "shared", "top"]);
    let merged = &materials.files[&path(BARRIER)];
    let layers: Vec<&str> = merged
        .layers()
        .iter()
        .map(|l| l.origin.path.as_str())
        .collect();
    assert_eq!(layers, [SPRITE, BARRIER, SPRITE]);
    assert_eq!(
        merged.materials[&MaterialKey::new("shared")]
            .depth_func
            .as_deref(),
        Some("Always")
    );
    // The middle pack's sprite file is asked for by nobody.
    assert!(!materials.common.contains("unused"));
    // No V17: a list asks for the barrier file.
    assert!(found.iter().all(|f| f.0.0 != "IgnoredFile"));
}

#[test]
fn g10_a_single_material_file_creates_one_material_named_after_the_file() {
    let foo =
        r#"{"vertexShader": "shaders/entity.vertex", "fragmentShader": "shaders/entity.fragment"}"#;
    let stack = over_vanilla(&[&[
        (COMMON, &list(&["materials/foo.material"])),
        ("materials/foo.material", foo),
    ]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    let material = materials.common.get("foo").expect("foo");
    assert_eq!(material.parent, None);
    assert_eq!(material.key_rules, KeyRules::Legacy);
    assert!(matches!(
        materials.files[&path("materials/foo.material")].form,
        MaterialFileForm::Single { .. }
    ));
}

#[test]
fn a_single_material_replaces_the_material_of_its_name() {
    // "stored in the group map, replacing any entry of that name": no V18.
    let stack = stack(&[&[
        (COMMON, &list(&[A, "materials/x.material"])),
        (A, &wrapped(r#""x": {"variants": [{"v": {}}]}"#)),
        ("materials/x.material", r#"{"vertexShader": "s"}"#),
    ]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    let x = materials.common.get("x").unwrap();
    assert_eq!(x.file.as_str(), "materials/x.material");
    assert_eq!(x.variant_count(), 0);
    assert_eq!(materials.common.len(), 1);
}

#[test]
fn s07_the_highest_packs_listing_of_a_path_is_the_one_that_loads_it() {
    // The pack's list is processed first: vanilla's listing of the same path, with its
    // `FANCY`, finds the file loaded and is skipped without a word.
    let stack = over_vanilla(&[&[(FANCY, &list(&[ENTITY]))]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    // The entity file's materials are the first of the group: the pack's entry loaded it.
    assert_eq!(materials.fancy.iter().next().unwrap().file.as_str(), ENTITY);
    assert!(materials.fancy.contains("entity_static"));
    assert_eq!(materials.lists[0].origin().pack.name(), "pack 2");
}

#[test]
fn s08_v19_a_path_listed_twice_in_one_list_is_used_once_deviation() {
    let listing = format!(
        r#"[{{"path": "{A}", "+defines": ["ONE"]}}, {{"path": "{B}"}}, {{"path": "{A}", "+defines": ["TWO"]}}, {{"path": "{A}"}}]"#
    );
    let stack = stack(&[&[
        (COMMON, &listing),
        (A, &wrapped(r#""a": {}"#)),
        (B, &wrapped(r#""b": {}"#)),
    ]]);
    let materials = stack.materials();
    assert_eq!(
        placed(materials),
        [
            (
                finding("NoEffect", Info, "2", "member has no effect"),
                Some("pack 1".to_owned()),
                Some(COMMON.to_owned())
            ),
            (
                finding("NoEffect", Info, "3", "member has no effect"),
                Some("pack 1".to_owned()),
                Some(COMMON.to_owned())
            ),
        ]
    );
    assert_eq!(names(&materials.common), ["a", "b"]);
}

#[test]
fn v19_a_path_two_packs_list_once_each_says_nothing_and_twice_in_the_lower_one_does_deviation() {
    let file = wrapped(r#""a": {}"#);
    let once = stack(&[
        &[(COMMON, &list(&[A])), (A, &file)],
        &[(COMMON, &list(&[A]))],
    ]);
    assert_eq!(all(once.materials()), []);
    // The lower list repeats a path the higher list loaded: its second entry repeats its own
    // first.
    let twice = stack(&[
        &[(COMMON, &list(&[A, A])), (A, &file)],
        &[(COMMON, &list(&[A]))],
    ]);
    let found = placed(twice.materials());
    assert_eq!(found.len(), 1);
    assert_eq!((found[0].0.0, found[0].0.2.as_str()), ("NoEffect", "1"));
    assert_eq!(found[0].1.as_deref(), Some("pack 1"));
    // The same path in two different lists is two loads.
    let both = stack(&[&[(COMMON, &list(&[A])), (FANCY, &list(&[A])), (A, &file)]]);
    assert_eq!(all(both.materials()), []);
    assert!(both.materials().common.contains("a") && both.materials().fancy.contains("a"));
    assert!(!both.materials().sad.contains("a"));
}

#[test]
fn m03_v13_a_parent_the_file_does_not_define_leaves_the_material_uncreated_deviation() {
    let stack = stack(&[&[
        (COMMON, &list(&[A])),
        (A, &wrapped(r#""x:does_not_exist": {}, "y": {}"#)),
    ]]);
    let materials = stack.materials();
    assert_eq!(
        all(materials),
        [finding(
            "UnresolvedReference",
            Info,
            "materials/x:does_not_exist",
            "unresolved reference"
        )]
    );
    let finding = materials.diagnostics().next().unwrap();
    assert_eq!(
        reference(finding),
        Some(("material", "does_not_exist".to_owned()))
    );
    assert_eq!(pack_name(finding).as_deref(), Some("pack 1"));
    assert_eq!(
        finding
            .location
            .path
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some(A)
    );
    assert_eq!(names(&materials.common), ["y"]);
}

#[test]
fn m04_v13_a_cycle_leaves_every_material_of_it_uncreated_deviation() {
    let stack = stack(&[&[
        (COMMON, &list(&[A])),
        (
            A,
            &wrapped(r#""a:b": {}, "b:a": {}, "c:a": {}, "self:self": {}, "ok": {}"#),
        ),
    ]]);
    let materials = stack.materials();
    let unresolved = |at: &str| finding("UnresolvedReference", Info, at, "unresolved reference");
    assert_eq!(
        all(materials),
        [
            unresolved("materials/a:b"),
            unresolved("materials/b:a"),
            unresolved("materials/c:a"),
            unresolved("materials/self:self")
        ]
    );
    assert_eq!(names(&materials.common), ["ok"]);
}

#[test]
fn m14_v13_a_parent_in_another_file_of_the_group_does_not_resolve_deviation() {
    let mine = wrapped(
        r#""my:entity_alphatest": {}, "+entity_alphatest": {}, "kid:entity_alphatest": {}"#,
    );
    let stack = over_vanilla(&[&[
        (FANCY, &list(&[ENTITY, "materials/my.material"])),
        ("materials/my.material", &mine),
    ]]);
    let materials = stack.materials();
    assert!(materials.fancy.contains("entity_alphatest"));
    // A parent is found through the file's own keys: `+entity_alphatest` is one, so `kid` and
    // `my` hang under it and are created from the group's material of that name.
    assert_eq!(all(materials), []);
    assert!(materials.fancy.contains("my") && materials.fancy.contains("kid"));
    // Without a key of that name in the file, the children are unreachable.
    let alone = wrapped(r#""my:entity_alphatest": {}"#);
    let stack = over_vanilla(&[&[
        (FANCY, &list(&[ENTITY, "materials/my.material"])),
        ("materials/my.material", &alone),
    ]]);
    let materials = stack.materials();
    assert_eq!(
        all(materials),
        [finding(
            "UnresolvedReference",
            Info,
            "materials/my:entity_alphatest",
            "unresolved reference"
        )]
    );
    assert!(!materials.fancy.contains("my"));
    assert!(materials.fancy.contains("entity_alphatest"));
}

#[test]
fn m15_a_plus_key_extends_the_material_an_earlier_file_created() {
    let a = wrapped(r#""x": {"variants": [{"one": {}}]}, "before:x": {}"#);
    let b = wrapped(
        r#""+x": {"+defines": ["USE_EMISSIVE"], "+variants": [{"two": {}}]}, "after:x": {}"#,
    );
    let stack = stack(&[&[(COMMON, &list(&[A, B])), (A, &a), (B, &b)]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    let common = &materials.common;
    assert_eq!(
        names(common),
        [
            "after",
            "after.one",
            "after.two",
            "before",
            "before.one",
            "x",
            "x.one",
            "x.two"
        ]
    );
    // The material is now the later file's; a child made before the extension is untouched.
    assert_eq!(common.get("x").unwrap().file.as_str(), B);
    assert_eq!(common.get("before").unwrap().file.as_str(), A);
    assert_eq!(common.len(), 3);
}

#[test]
fn a_plus_key_for_a_name_the_group_lacks_creates_the_material() {
    let stack = stack(&[&[
        (COMMON, &list(&[A])),
        (
            A,
            &wrapped(r#""p": {"variants": [{"v": {}}]}, "+x:p": {}, "c:x": {}"#),
        ),
    ]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    assert_eq!(
        names(&materials.common),
        ["c", "c.v", "p", "p.v", "x", "x.v"]
    );
    assert_eq!(
        materials.common.get("x").unwrap().parent.as_deref(),
        Some("p")
    );
}

#[test]
fn m16_v18_a_later_definition_of_an_existing_name_is_ignored_deviation() {
    let a = wrapped(r#""x": {"variants": [{"from_a": {}}]}"#);
    let b = wrapped(r#""x": {"variants": [{"from_b": {}}]}, "child:x": {}"#);
    let stack = stack(&[&[(COMMON, &list(&[A, B])), (A, &a), (B, &b)]]);
    let materials = stack.materials();
    assert_eq!(
        placed(materials),
        [(
            finding(
                "DuplicateIdentifier",
                Info,
                "materials/x",
                "duplicate identifier"
            ),
            Some("pack 1".to_owned()),
            Some(B.to_owned())
        )]
    );
    let x = materials.common.get("x").unwrap();
    assert_eq!(x.file.as_str(), A);
    assert!(x.has_variant("from_a") && !x.has_variant("from_b"));
    // The child of the later file is made from the material the group has.
    assert!(materials.common.get("child").unwrap().has_variant("from_a"));
}

#[test]
fn v18_is_reported_by_each_later_file_and_not_within_one_file_deviation() {
    let x = wrapped(r#""x": {}"#);
    let c = "materials/c.material";
    let stack = stack(&[&[(COMMON, &list(&[A, B, c])), (A, &x), (B, &x), (c, &x)]]);
    let found = placed(stack.materials());
    let files: Vec<_> = found.iter().map(|f| (f.0.0, f.2.as_deref())).collect();
    assert_eq!(
        files,
        [
            ("DuplicateIdentifier", Some(B)),
            ("DuplicateIdentifier", Some(c))
        ]
    );
    // Two keys of one name in one file are V2's, once.
    let twice = wrapped(r#""p": {}, "q": {}, "x:p": {}, "x:q": {}"#);
    let stack = crate::common::stack(&[&[(COMMON, &list(&[A])), (A, &twice)]]);
    let codes: Vec<_> = all(stack.materials())
        .into_iter()
        .map(|f| (f.0, f.2))
        .collect();
    assert_eq!(codes, [("DuplicateIdentifier", "materials/x:q".to_owned())]);
    assert_eq!(
        stack.materials().common.get("x").unwrap().parent.as_deref(),
        Some("p")
    );
}

#[test]
fn m17_the_parent_is_the_text_after_the_last_colon() {
    let stack = stack(&[&[
        (COMMON, &list(&[A])),
        (A, &wrapped(r#""c": {}, "a:b:c": {}"#)),
    ]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    let material = materials.common.get("a:b").expect("a:b");
    assert_eq!(material.parent.as_deref(), Some("c"));
    assert!(materials.common.get("a").is_none());
}

#[test]
fn materials_are_created_parent_first_in_key_order() {
    // Breadth first from the parentless keys, the keys of a level bytewise.
    let file = wrapped(r#""z": {}, "b:z": {}, "a:z": {}, "y": {}, "k:a": {}, "c:y": {}"#);
    let stack = stack(&[&[(COMMON, &list(&[A])), (A, &file)]]);
    let order: Vec<&str> = stack
        .materials()
        .common
        .iter()
        .map(|m| m.name.as_str())
        .collect();
    assert_eq!(order, ["y", "z", "c", "a", "b", "k"]);
}

/// The V26 finding at `at` for the variant `name`.
fn replaced(at: &str, name: &str) -> Rendered {
    finding(
        "ShadowedDefinition",
        Info,
        at,
        &format!("Variant: {name} was replaced with a new version"),
    )
}

#[test]
fn v26_a_variant_the_material_already_has_is_replaced_with_the_clients_text_deviation() {
    let file = wrapped(
        r#""p": {"variants": [{"v": {}}, {"w": {}}, {"v": {}}]},
           "adds:p": {"+variants": [{"w": {}}, {"new": {}}]},
           "replaces:p": {"variants": [{"w": {}}]}"#,
    );
    let stack = stack(&[&[(COMMON, &list(&[A])), (A, &file)]]);
    let materials = stack.materials();
    assert_eq!(
        all(materials),
        [
            // Twice in one array.
            replaced("materials/p/variants/2/v", "v"),
            // `+variants` keeps the inherited ones: `w` is replaced, `new` is new.
            replaced("materials/adds:p/+variants/0/w", "w"),
        ]
    );
    let common = &materials.common;
    assert_eq!(
        common.get("p").unwrap().variants().collect::<Vec<_>>(),
        ["v", "w"]
    );
    assert_eq!(
        common.get("adds").unwrap().variants().collect::<Vec<_>>(),
        ["new", "v", "w"]
    );
    // `variants` empties what is inherited first: nothing is replaced.
    assert_eq!(
        common
            .get("replaces")
            .unwrap()
            .variants()
            .collect::<Vec<_>>(),
        ["w"]
    );
}

#[test]
fn variants_beside_plus_variants_is_the_one_that_is_read() {
    let file = wrapped(
        r#""p": {"variants": [{"a": {}}], "+variants": [{"b": {}}]}, "q": {"variants": [], "+variants": [{"b": {}}]}"#,
    );
    let stack = stack(&[&[(COMMON, &list(&[A])), (A, &file)]]);
    let common = &stack.materials().common;
    assert_eq!(
        common.get("p").unwrap().variants().collect::<Vec<_>>(),
        ["a"]
    );
    assert_eq!(common.get("q").unwrap().variant_count(), 0);
}

#[test]
fn v11_a_nested_variant_needs_its_parent_variant_first_deviation() {
    let missing = |at: &str| finding("UnresolvedReference", Info, at, "unresolved reference");
    let file = wrapped(
        r#""p": {"variants": [{"a.b": {}}, {"a": {}}, {"a.b": {}}, {"a.b.c": {}}, {"x.y.z": {}}]},
           "adds:p": {"+variants": [{"a.d": {}}]},
           "replaces:p": {"variants": [{"a.d": {}}]}"#,
    );
    let stack = stack(&[&[(COMMON, &list(&[A])), (A, &file)]]);
    let materials = stack.materials();
    assert_eq!(
        all(materials),
        [
            // `a.b` before `a`; `x.y.z` without `x.y`.
            missing("materials/p/variants/0/a.b"),
            missing("materials/p/variants/4/x.y.z"),
            // The inherited `a` is gone once `variants` replaced the list.
            missing("materials/replaces:p/variants/0/a.d"),
        ]
    );
    let finding = materials.diagnostics().next().unwrap();
    assert_eq!(
        reference(finding),
        Some(("material variant", "a".to_owned()))
    );
    let common = &materials.common;
    assert_eq!(
        common.get("p").unwrap().variants().collect::<Vec<_>>(),
        ["a", "a.b", "a.b.c"]
    );
    assert!(common.get("adds").unwrap().has_variant("a.d"));
    assert_eq!(common.get("replaces").unwrap().variant_count(), 0);
    // `<name>.<variant>` with dots on both sides is still found.
    assert!(common.contains("p.a.b.c") && common.contains("adds.a.d"));
    assert!(!common.contains("p.a.d"));
}

#[test]
fn v4_a_material_with_an_unknown_keyword_is_refused_and_keeps_its_name_taken_deviation() {
    let a = wrapped(r#""bad": {"blendSrc": "Nope"}, "kid:bad": {}, "grandkid:kid": {}, "ok": {}"#);
    let b = wrapped(r#""bad": {}, "+bad": {}, "later:bad": {}"#);
    let stack = stack(&[&[(COMMON, &list(&[A, B])), (A, &a), (B, &b)]]);
    let materials = stack.materials();
    let found: Vec<_> = placed(materials)
        .into_iter()
        .map(|((code, severity, at, _), _, file)| (code, severity, at, file.unwrap()))
        .collect();
    let at =
        |code, severity, at: &str, file: &str| (code, severity, at.to_owned(), file.to_owned());
    assert_eq!(
        found,
        [
            at("InvalidValue", Error, "materials/bad/blendSrc", A),
            at("UnresolvedReference", Info, "materials/kid:bad", A),
            at("UnresolvedReference", Info, "materials/grandkid:kid", A),
            // The later file does not stand in for the refused material: neither its own
            // definition, nor an extension, nor a child.
            at("UnresolvedReference", Info, "materials/+bad", B),
            at("DuplicateIdentifier", Info, "materials/bad", B),
            at("UnresolvedReference", Info, "materials/later:bad", B),
        ]
    );
    assert_eq!(names(&materials.common), ["ok"]);
    assert!(materials.common.get("bad").is_none());
    assert!(!materials.contains(&id("bad")));
}

#[test]
fn v4_a_refused_keyword_in_a_variant_body_or_an_extension_refuses_that_much() {
    let a =
        wrapped(r#""x": {"variants": [{"v": {"depthFunc": "Nope"}}]}, "y": {"defines": ["A"]}"#);
    let b = wrapped(r#""+y": {"msaaSupport": "Nope", "+variants": [{"late": {}}]}"#);
    let stack = stack(&[&[(COMMON, &list(&[A, B])), (A, &a), (B, &b)]]);
    let materials = stack.materials();
    let codes: Vec<_> = all(materials)
        .into_iter()
        .map(|f| (f.0, f.1, f.2))
        .collect();
    assert_eq!(
        codes,
        [
            (
                "InvalidValue",
                Error,
                "materials/x/variants/0/v/depthFunc".to_owned()
            ),
            ("InvalidValue", Error, "materials/+y/msaaSupport".to_owned()),
        ]
    );
    // `x` is refused whole; `y` stays as the first file made it.
    assert_eq!(names(&materials.common), ["y"]);
    assert_eq!(materials.common.get("y").unwrap().file.as_str(), A);
}

#[test]
fn a_refused_winner_does_not_let_the_definition_below_it_stand_in() {
    let good = wrapped(r#""x": {"blendSrc": "One"}"#);
    let bad = wrapped(r#""x": {"blendSrc": "Nope"}"#);
    let silent = wrapped(r#""other": {}"#);
    let base: Files<'_> = &[(COMMON, &list(&[A])), (A, &good)];
    // Two packs: the higher copy's keyword wins the member and refuses the material.
    let two = stack(&[base, &[(A, &bad)]]);
    assert!(two.materials().common.get("x").is_none());
    assert_eq!(all(two.materials()).len(), 1);
    // Three packs, the top one silent about `x`: still refused.
    let three = stack(&[base, &[(A, &bad)], &[(A, &silent)]]);
    assert!(three.materials().common.get("x").is_none());
    assert_eq!(names(&three.materials().common), ["other"]);
    // Three packs, the top one writing a keyword the client knows: that member wins.
    let fixed = stack(&[base, &[(A, &bad)], &[(A, &good)]]);
    assert!(fixed.materials().common.get("x").is_some());
    assert_eq!(all(fixed.materials()), []);
}

#[test]
fn a_winning_key_whose_parent_does_not_resolve_does_not_fall_back_to_the_key_below() {
    let low = wrapped(r#""x": {}"#);
    let high = wrapped(r#""x:nope": {}"#);
    let base: Files<'_> = &[(COMMON, &list(&[A])), (A, &low)];
    let silent = wrapped(r#""other": {}"#);
    for packs in [
        vec![base, &[(A, high.as_str())]],
        vec![base, &[(A, high.as_str())], &[(A, silent.as_str())]],
    ] {
        let stack = stack(&packs);
        let materials = stack.materials();
        assert!(materials.common.get("x").is_none());
        let codes: Vec<_> = all(materials).into_iter().map(|f| (f.0, f.2)).collect();
        assert_eq!(
            codes,
            [
                ("ShadowedDefinition", "materials/x:nope".to_owned()),
                ("UnresolvedReference", "materials/x:nope".to_owned())
            ]
        );
    }
}

#[test]
fn an_uncreated_material_does_not_take_its_name_from_a_later_file() {
    // Vanilla's visitor creates a material when the group has none of that name: one that
    // was never created (its parent is missing) leaves the name free for the next file.
    let a = wrapped(r#""x:nope": {}"#);
    let b = wrapped(r#""x": {}"#);
    let stack = stack(&[&[(COMMON, &list(&[A, B])), (A, &a), (B, &b)]]);
    let materials = stack.materials();
    let codes: Vec<_> = all(materials).into_iter().map(|f| (f.0, f.2)).collect();
    assert_eq!(
        codes,
        [("UnresolvedReference", "materials/x:nope".to_owned())]
    );
    assert_eq!(materials.common.get("x").unwrap().file.as_str(), B);
}

// The rejected-winner table, three packs with the refused or uncreated definition on
// top of two good ones: V4 (ours) keeps the name taken; V13 does not create the
// material of the merged key, and leaves the name free for another file of any pack.
#[test]
fn three_packs_a_refused_or_uncreated_top_definition_over_two_good_ones() {
    let good = wrapped(r#""x": {"blendSrc": "One"}"#);
    let other = wrapped(r#""x": {"blendSrc": "Zero"}"#);
    let base: Files<'_> = &[(COMMON, &list(&[A])), (A, &good)];
    // V4 on top: neither the middle nor the lowest definition stands in.
    let bad = wrapped(r#""x": {"blendSrc": "Nope"}"#);
    let refused = stack(&[base, &[(A, &other)], &[(A, &bad)]]);
    assert!(refused.materials().common.get("x").is_none());
    let codes: Vec<_> = all(refused.materials()).into_iter().map(|f| f.0).collect();
    assert_eq!(codes, ["InvalidValue"]);
    // V13 on top: the merged key is the top pack's, and its parent does not resolve.
    let orphan = wrapped(r#""x:nope": {}"#);
    let uncreated = stack(&[base, &[(A, &other)], &[(A, &orphan)]]);
    assert!(uncreated.materials().common.get("x").is_none());
    let codes: Vec<_> = all(uncreated.materials())
        .into_iter()
        .map(|f| f.0)
        .collect();
    assert!(codes.contains(&"UnresolvedReference"), "{codes:?}");
}

#[test]
fn an_uncreated_material_of_a_higher_pack_leaves_its_name_free_for_another_file() {
    let orphan = wrapped(r#""x:nope": {}"#);
    let good = wrapped(r#""x": {}"#);
    // Two packs: the higher pack's file A does not create `x`; the lower pack's file B does.
    let two = stack(&[&[(COMMON, &list(&[A, B])), (B, &good)], &[(A, &orphan)]]);
    assert_eq!(two.materials().common.get("x").unwrap().file.as_str(), B);
    // Three packs, a silent one between them: the same.
    let silent = wrapped(r#""other": {}"#);
    let three = stack(&[
        &[(COMMON, &list(&[A, B])), (B, &good)],
        &[(B, &silent)],
        &[(A, &orphan)],
    ]);
    assert_eq!(three.materials().common.get("x").unwrap().file.as_str(), B);
    // A refused definition (V4) in the higher pack's file A keeps the name: file B's is not
    // created.
    let bad = wrapped(r#""x": {"blendSrc": "Nope"}"#);
    let taken = stack(&[&[(COMMON, &list(&[A, B])), (B, &good)], &[(A, &bad)]]);
    assert!(taken.materials().common.get("x").is_none());
}

#[test]
fn the_key_rules_of_a_material_are_those_of_the_highest_copy_that_names_it() {
    // A flat copy over a wrapped file makes the materials it names legacy ones, and the
    // reverse; the others keep the rules of their own copy.
    let low = wrapped(r#""a": {"+states": ["Blending"]}, "b": {"+states": ["Blending"]}, "c": {}"#);
    let mid = r#"{"a:c": {}, "d": {}}"#;
    let top = wrapped(r#""d": {}, "e": {}"#);
    let stack = stack(&[
        &[(COMMON, &list(&[A])), (A, &low)],
        &[(A, mid)],
        &[(A, &top)],
    ]);
    let materials = stack.materials();
    let merged = &materials.files[&path(A)];
    let rules = |key: &str| merged.key_rules(&MaterialKey::parse(key)).unwrap();
    assert_eq!(rules("a:c"), KeyRules::Legacy);
    assert_eq!(rules("b"), KeyRules::Versioned);
    assert_eq!(rules("c"), KeyRules::Versioned);
    assert_eq!(rules("d"), KeyRules::Versioned);
    assert_eq!(rules("e"), KeyRules::Versioned);
    assert_eq!(
        merged.materials[&MaterialKey::parse("a:c")].key_rules,
        Some(KeyRules::Legacy)
    );
    assert_eq!(
        materials.common.get("a").unwrap().key_rules,
        KeyRules::Legacy
    );
    assert_eq!(
        materials.common.get("b").unwrap().key_rules,
        KeyRules::Versioned
    );
    // `a` is legacy now, so its `+states` is unread (V22); `b`'s is read. The key that
    // changed is reported too (V20). The unread member is named where it was written: in
    // pack 1's wrapped file, under pack 1's key, though it is pack 2's copy that makes it
    // unread.
    let found: Vec<_> = placed(materials)
        .into_iter()
        .map(|(f, pack, _)| (f.0, f.2, pack.unwrap()))
        .collect();
    assert_eq!(
        found,
        [
            ("ShadowedDefinition", "a:c".to_owned(), "pack 2".to_owned()),
            (
                "NoEffect",
                "materials/a/+states".to_owned(),
                "pack 1".to_owned()
            ),
        ]
    );
    // The form of the merged file is the top copy's.
    assert!(matches!(merged.form, MaterialFileForm::Wrapped { .. }));
}

/// The findings of the group as (code, path in the file, pack, file).
fn written(materials: &Materials) -> Vec<(&'static str, String, String, String)> {
    placed(materials)
        .into_iter()
        .map(|(f, pack, file)| (f.0, f.2, pack.unwrap(), file.unwrap()))
        .collect()
}

fn at(
    code: &'static str,
    path: &str,
    pack: &str,
    file: &str,
) -> (&'static str, String, String, String) {
    (code, path.to_owned(), pack.to_owned(), file.to_owned())
}

#[test]
fn the_findings_of_a_copy_name_its_pack_and_those_of_the_merge_the_pack_that_wrote_the_member() {
    let low = wrapped(r#""p": {}, "x:p": {"+states": ["Nope"]}, "x:q": {}, "q": {}"#);
    let high = r#"{"materials": {"version": "2.0.0", "lost": {}}, "y": {"varient": []}}"#;
    let stack = stack(&[&[(COMMON, &list(&[A])), (A, &low)], &[(A, high)]]);
    assert_eq!(
        written(stack.materials()),
        [
            // Of the copies, lowest pack first: each at the path it has in its file, the
            // wrapper's member included.
            at("DuplicateIdentifier", "materials/x:q", "pack 1", A),
            at("UnsupportedVersion", "materials/version", "pack 2", A),
            // Of the merged file, in its order: the top copy's members first. The top copy is
            // read flat (its wrapper does not pass the gate), the low one is wrapped.
            at("UnrecognizedField", "y/varient", "pack 2", A),
            at("InvalidValue", "materials/x:p/+states/0", "pack 1", A),
        ]
    );
}

#[test]
fn a_finding_names_the_pack_that_wrote_the_member_not_the_highest_pack_naming_the_key() {
    // Coordinator ruling on B4: pack 2 names `x` and writes none of the three members.
    let low = wrapped(r#""x": {"blendSrc": "Bad", "+vertexShader": "s", "states": ["Nope"]}"#);
    let high = wrapped(r#""x": {"msaaSupport": "MSAA"}"#);
    let packs = stack(&[&[(COMMON, &list(&[A])), (A, &low)], &[(A, &high)]]);
    assert_eq!(
        written(packs.materials()),
        [
            at(
                "UnrecognizedField",
                "materials/x/+vertexShader",
                "pack 1",
                A
            ),
            at("InvalidValue", "materials/x/blendSrc", "pack 1", A),
            at("InvalidValue", "materials/x/states/0", "pack 1", A),
        ]
    );
    // Three packs: the middle one overrides one member with a bad value of its own, the top
    // one writes another.
    let middle = wrapped(r#""x": {"blendSrc": "Worse"}"#);
    let top = wrapped(r#""x": {"depthFunc": "Sometimes"}"#);
    let packs = stack(&[
        &[(COMMON, &list(&[A])), (A, &low)],
        &[(A, &middle)],
        &[(A, &top)],
    ]);
    assert_eq!(
        written(packs.materials()),
        [
            at("InvalidValue", "materials/x/depthFunc", "pack 3", A),
            at(
                "UnrecognizedField",
                "materials/x/+vertexShader",
                "pack 1",
                A
            ),
            at("InvalidValue", "materials/x/blendSrc", "pack 2", A),
            at("InvalidValue", "materials/x/states/0", "pack 1", A),
        ]
    );
}

#[test]
fn a_finding_has_the_path_of_the_file_that_wrote_the_member_wrapped_or_flat() {
    // The low copy is flat and spells the key with a parent; the high one is wrapped and
    // spells it without: each finding has the key and the wrapper of its own file.
    let low = flat(r#""p": {}, "x:p": {"blendSrc": "Bad", "frontFace": {"stencilFunc": "Bad"}}"#);
    let high = wrapped(r#""x": {"depthFunc": "Bad", "frontFace": {"stencilPassOp": "Bad"}}"#);
    let packs = stack(&[&[(COMMON, &list(&[A])), (A, &low)], &[(A, &high)]]);
    assert_eq!(
        written(packs.materials()),
        [
            // V20: the key that drops the parent, where pack 2 writes it.
            at("ShadowedDefinition", "materials/x", "pack 2", A),
            at("InvalidValue", "materials/x/depthFunc", "pack 2", A),
            at(
                "InvalidValue",
                "materials/x/frontFace/stencilPassOp",
                "pack 2",
                A
            ),
            at("InvalidValue", "x:p/frontFace/stencilFunc", "pack 1", A),
            at("InvalidValue", "x:p/blendSrc", "pack 1", A),
        ]
    );
    // The reverse: the wrapped copy below, the flat one on top.
    let low = wrapped(r#""x": {"blendSrc": "Bad"}"#);
    let high = flat(r#""x": {"depthFunc": "Bad"}"#);
    let packs = stack(&[&[(COMMON, &list(&[A])), (A, &low)], &[(A, &high)]]);
    assert_eq!(
        written(packs.materials()),
        [
            at("InvalidValue", "x/depthFunc", "pack 2", A),
            at("InvalidValue", "materials/x/blendSrc", "pack 1", A),
        ]
    );
}

#[test]
fn an_array_element_is_written_by_the_pack_whose_array_reaches_it() {
    // Arrays merge by index: element 0 is pack 2's, element 1 is what pack 1 has past pack
    // 2's length. Pack 1's bad element 0 is not in the merged file and is not reported.
    let low = wrapped(
        r#""x": {"samplerStates": [{"samplerIndex": 0, "textureFilter": "Bad"},
                                   {"samplerIndex": 1, "textureWrap": "Bad"}]}"#,
    );
    let high =
        wrapped(r#""x": {"samplerStates": [{"samplerIndex": 0, "textureFilter": "Worse"}]}"#);
    let packs = stack(&[&[(COMMON, &list(&[A])), (A, &low)], &[(A, &high)]]);
    assert_eq!(
        written(packs.materials()),
        [
            at(
                "InvalidValue",
                "materials/x/samplerStates/0/textureFilter",
                "pack 2",
                A
            ),
            at(
                "InvalidValue",
                "materials/x/samplerStates/1/textureWrap",
                "pack 1",
                A
            ),
        ]
    );
}

#[test]
fn a_finding_of_the_creation_names_the_pack_that_wrote_what_it_is_about() {
    // V11 is about a variant pack 1 writes under a key pack 2 respells; V13 is about the
    // parent pack 2's key names.
    let low = flat(r#""x": {"+variants": [{"a.b": {}}]}"#);
    let high = wrapped(r#""x:nope": {"depthFunc": "Always"}"#);
    let packs = stack(&[&[(COMMON, &list(&[A])), (A, &low)], &[(A, &high)]]);
    assert_eq!(
        written(packs.materials()),
        [
            at("ShadowedDefinition", "materials/x:nope", "pack 2", A),
            at("UnresolvedReference", "materials/x:nope", "pack 2", A),
        ]
    );
    let high = wrapped(r#""p": {}, "x:p": {"depthFunc": "Always"}"#);
    let packs = stack(&[&[(COMMON, &list(&[A])), (A, &low)], &[(A, &high)]]);
    assert_eq!(
        written(packs.materials()),
        [
            at("ShadowedDefinition", "materials/x:p", "pack 2", A),
            at("UnresolvedReference", "x/+variants/0/a.b", "pack 1", A),
        ]
    );
}

#[test]
fn a_bias_of_the_largest_double_is_reported_in_the_pack_that_wrote_it_out() {
    // The lower pack writes a number past the doubles, which is read as the largest double
    // and reported there. The upper pack writes the largest double itself over it, and over
    // nothing in `y`: a number beyond the floats like any other, reported where it is
    // written, once.
    let low = wrapped(r#""x": {"depthBias": 1e999}, "y": {"depthBias": 1e999}"#);
    let top = flat(
        r#""x": {"depthBias": 1.7976931348623157e308}, "y": {"slopeScaledDepthBias": -1.7976931348623157e308}"#,
    );
    let stack = stack(&[&[(COMMON, &list(&[A])), (A, &low)], &[(A, &top)]]);
    let materials = stack.materials();
    let invalid: Vec<_> = placed(materials)
        .into_iter()
        .filter(|found| found.0.0 == "InvalidValue")
        .map(|found| (found.0.2, found.1.unwrap_or_default()))
        .collect();
    let at = |path: &str, pack: &str| (path.to_owned(), pack.to_owned());
    assert_eq!(
        invalid,
        [
            at("materials/x/depthBias", "pack 1"),
            at("materials/y/depthBias", "pack 1"),
            at("x/depthBias", "pack 2"),
            at("y/slopeScaledDepthBias", "pack 2"),
        ]
    );
    let file = &materials.files[&path(A)];
    let y = &file.materials[&MaterialKey::new("y")];
    assert_eq!(y.depth_bias, Some(f32::MAX));
    assert_eq!(y.slope_scaled_depth_bias, Some(f32::MIN));
}

#[test]
fn a_file_read_on_its_own_has_its_findings_at_the_paths_of_the_file() {
    // The same members wrapped and flat: the wrapper's member is part of the path, as it is
    // in the gate's finding and in a `to_json` refusal.
    let members = r#""x": {"blendSrc": "Bad"}, "x:p": {}, "y": {"depthBias": 1e999}"#;
    let paths = |text: &str| -> Vec<(&'static str, String)> {
        crate::common::found(text)
            .into_iter()
            .map(|f| (f.0, f.2))
            .collect()
    };
    assert_eq!(
        paths(&wrapped(members)),
        [
            ("InvalidValue", "materials/y/depthBias".to_owned()),
            ("DuplicateIdentifier", "materials/x:p".to_owned()),
            ("InvalidValue", "materials/x/blendSrc".to_owned()),
        ]
    );
    assert_eq!(
        paths(&flat(members)),
        [
            ("InvalidValue", "y/depthBias".to_owned()),
            ("DuplicateIdentifier", "x:p".to_owned()),
            ("InvalidValue", "x/blendSrc".to_owned()),
        ]
    );
    // A single material inside a wrapper.
    assert_eq!(
        paths(r#"{"materials": {"version": "1.0.0", "vertexShader": "s", "blendSrc": "Bad"}}"#),
        [("InvalidValue", "materials/blendSrc".to_owned())]
    );
    // What `to_json` refuses is named at the same path.
    let mut file = crate::common::file(&wrapped(r#""x": {}"#)).value().clone();
    file.materials[0].blend_src = Some("Bad".to_owned());
    let refused = bedrock_addon::document::Content::to_json(&file).unwrap_err();
    assert!(
        matches!(&refused, bedrock_addon::json::EncodeError::NotRepresentable { at, .. } if at.to_string() == "materials/x/blendSrc"),
        "{refused:?}"
    );
}

#[test]
fn a_list_that_cannot_be_read_is_skipped_and_the_lists_of_the_other_packs_load() {
    let file = wrapped(r#""a": {}"#);
    let stack = stack(&[
        &[(COMMON, &list(&[A])), (A, &file)],
        &[(COMMON, "[{")],
        &[(COMMON, r#"{"path": "materials/a.material"}"#)],
    ]);
    let materials = stack.materials();
    let found: Vec<_> = placed(materials)
        .into_iter()
        .map(|(f, pack, file)| (f.0, f.1, pack.unwrap(), file.unwrap()))
        .collect();
    assert_eq!(
        found,
        [
            (
                "MalformedInput",
                Error,
                "pack 2".to_owned(),
                COMMON.to_owned()
            ),
            ("InvalidType", Error, "pack 3".to_owned(), COMMON.to_owned()),
        ]
    );
    assert_eq!(materials.lists.len(), 1);
    assert!(materials.common.contains("a"));
}

#[test]
fn the_findings_of_a_list_are_reported_with_the_group() {
    let listing = format!(r#"[{{"path": "{A}", "tag": "x"}}]"#);
    let stack = stack(&[&[(SAD, &listing), (A, &wrapped(r#""a": {}"#))]]);
    let materials = stack.materials();
    assert_eq!(
        placed(materials),
        [(
            finding(
                "DeprecatedMember",
                Error,
                "0/tag",
                "key \"tag\" is no longer supported, please remove"
            ),
            Some("pack 1".to_owned()),
            Some(SAD.to_owned())
        )]
    );
    assert!(materials.sad.contains("a"));
    assert!(materials.common.is_empty() && materials.fancy.is_empty());
}

#[test]
fn the_gameface_list_is_read_and_feeds_no_group() {
    let stack = stack(&[&[
        ("materials/gameface.json", &list(&[A])),
        (A, &wrapped(r#""a": {}"#)),
    ]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    assert_eq!(materials.lists.len(), 1);
    assert_eq!(materials.lists[0].kind, MaterialListKind::Gameface);
    assert!(!materials.contains(&id("a")));
    // The file is merged all the same.
    assert!(
        materials.files[&path(A)]
            .materials
            .contains_key(&MaterialKey::new("a"))
    );
}

#[test]
fn a_material_file_no_list_names_is_merged_and_creates_nothing() {
    let stack = stack(&[
        &[
            (B, &wrapped(r#""b": {}"#)),
            ("deep/dir/c.material", r#"{"c": {"blendSrc": "Nope"}}"#),
        ],
        &[(B, &wrapped(r#""b2": {}"#))],
    ]);
    let materials = stack.materials();
    // The findings of a file are reported whether or not a list names it.
    let codes: Vec<_> = all(materials).into_iter().map(|f| (f.0, f.2)).collect();
    assert_eq!(codes, [("InvalidValue", "c/blendSrc".to_owned())]);
    let files: Vec<&str> = materials.files.keys().map(PackPath::as_str).collect();
    assert_eq!(files, ["deep/dir/c.material", B]);
    assert_eq!(materials.files[&path(B)].materials.len(), 2);
    for kind in MaterialGroupKind::ALL {
        assert!(materials.group(kind).is_empty());
    }
}

#[test]
fn a_listed_path_that_is_no_material_file_is_not_read_deviation() {
    // vanilla reads whatever path a list names; ours reads `.material` files only and says so
    // when a pack has the file. A path no pack has loads nothing, in silence, as in vanilla.
    let stack = stack(&[&[
        (
            COMMON,
            &list(&[
                "materials/a.json",
                "materials/missing.material",
                "materials/nope.json",
                "",
            ]),
        ),
        ("materials/a.json", r#"{"a": {}}"#),
    ]]);
    let materials = stack.materials();
    assert_eq!(
        placed(materials),
        [(
            finding("IgnoredFile", Info, "0/path", "file ignored"),
            Some("pack 1".to_owned()),
            Some(COMMON.to_owned())
        )]
    );
    assert!(materials.common.is_empty());
    assert!(materials.files.is_empty());
}

#[test]
fn the_three_groups_are_separate_and_a_name_of_any_of_them_is_defined_deviation() {
    let stack = stack(&[&[
        (COMMON, &list(&[A])),
        (FANCY, &list(&[B])),
        (SAD, &list(&["materials/c.material"])),
        (A, &wrapped(r#""x": {"variants": [{"from_common": {}}]}"#)),
        (
            B,
            &wrapped(r#""x": {"variants": [{"from_fancy": {}}]}, "f": {}"#),
        ),
        (
            "materials/c.material",
            &wrapped(r#""x": {}, "s": {"variants": [{"v": {}}]}"#),
        ),
    ]]);
    let materials = stack.materials();
    // The same name in two groups is no redefinition.
    assert_eq!(all(materials), []);
    assert_eq!(names(&materials.common), ["x", "x.from_common"]);
    assert_eq!(names(&materials.fancy), ["f", "x", "x.from_fancy"]);
    assert_eq!(names(&materials.sad), ["s", "s.v", "x"]);
    for name in ["x", "f", "s", "x.from_common", "x.from_fancy", "s.v"] {
        assert!(materials.contains(&id(name)), "{name}");
    }
    assert!(!materials.contains(&id("f.v")));
    assert_eq!(materials.get("x").unwrap().file.as_str(), A);
    assert_eq!(materials.get("f").unwrap().file.as_str(), B);
    assert_eq!(
        materials.get("s").unwrap().file.as_str(),
        "materials/c.material"
    );
    assert!(materials.get("nope").is_none());
}

#[test]
fn a_finding_of_a_file_two_groups_load_is_reported_once() {
    let file = wrapped(r#""x:nope": {}, "p": {"variants": [{"v": {}}, {"v": {}}]}"#);
    let stack = stack(&[&[
        (COMMON, &list(&[A])),
        (FANCY, &list(&[A])),
        (SAD, &list(&[A])),
        (A, &file),
    ]]);
    let codes: Vec<_> = all(stack.materials())
        .into_iter()
        .map(|f| (f.0, f.2))
        .collect();
    assert_eq!(
        codes,
        [
            ("ShadowedDefinition", "materials/p/variants/1/v".to_owned()),
            ("UnresolvedReference", "materials/x:nope".to_owned()),
        ]
    );
}

#[test]
fn two_loads_of_one_stack_print_the_same() {
    let chemistry = fixture_text("vanilla/chemistry/entity.material");
    let build = || over_vanilla(&[&[(ENTITY, &chemistry)]]);
    let (first, second) = (build(), build());
    assert_eq!(
        format!("{:?}", first.materials()),
        format!("{:?}", second.materials())
    );
    // A material prints its own link, not its ancestors or its variants' names.
    let printed = format!("{:?}", first.materials().fancy.get("dolphin").unwrap());
    assert!(printed.contains("parent: Some(\"entity\")"), "{printed}");
    assert!(printed.contains("3 names"), "{printed}");
    assert!(!printed.contains("entity_static"), "{printed}");
}

#[test]
fn a_single_material_named_like_a_member_is_still_a_legacy_one() {
    // The file's name is also a key of its wrapped copy's map (`vertexShader`): the key rules
    // of that key are not the single material's, which the client stamps with no version.
    let file =
        r#"{"materials": {"version": "1.0.0", "vertexShader": "s", "+states": ["Blending"]}}"#;
    let name = "materials/vertexShader.material";
    let stack = stack(&[&[(COMMON, &list(&[name])), (name, file)]]);
    let materials = stack.materials();
    let material = materials
        .common
        .get("vertexShader")
        .expect("the single material");
    assert_eq!(material.key_rules, KeyRules::Legacy);
    let merged = &materials.files[&path(name)];
    assert_eq!(
        merged.materials[&MaterialKey::new("vertexShader")].key_rules,
        Some(KeyRules::Legacy)
    );
    let resolved = materials
        .resolve(MaterialGroupKind::Common, "vertexShader")
        .unwrap();
    assert!(resolved.states.is_empty());
}

#[test]
fn a_name_with_dots_is_split_at_every_dot_between_material_and_variant() {
    // A material's name may hold dots, and so may a variant's: `a.b.c` is the material `a.b`
    // with the variant `c`, or the material `a` with the variant `b.c`.
    let file = wrapped(
        r#""a.b": {"variants": [{"c": {}}]},
           "a": {"variants": [{"b": {}}, {"b.d": {}}]},
           "p.q.r": {"variants": [{"s": {}}, {"s.t": {}}]},
           "plain": {}"#,
    );
    let stack = stack(&[&[(COMMON, &list(&[A])), (A, &file)]]);
    let common = &stack.materials().common;
    for name in [
        // The materials themselves, dots and all.
        "a.b",
        "a",
        "p.q.r",
        "plain",
        // `a.b` + `c`: the split is at the second dot.
        "a.b.c",
        // `a` + `b` and `a` + `b.d`: the split is at the first dot.
        "a.b.d",
        // Three dots in the material, one in the variant.
        "p.q.r.s",
        "p.q.r.s.t",
    ] {
        assert!(common.contains(name), "{name}");
        assert!(stack.materials().contains(&id(name)), "{name}");
    }
    for name in [
        "a.c",
        "a.b.c.d",
        "a.b.x",
        "p",
        "p.q",
        "p.q.r.t",
        "plain.",
        ".plain",
        "",
        ".",
        "a.",
        "a.b.",
        "p.q.r.s.",
        "plain.plain",
    ] {
        assert!(!common.contains(name), "{name:?}");
    }
    // `get` is for materials only.
    assert!(common.get("a.b").is_some());
    assert!(common.get("a.b.c").is_none());
}

/// The common group of one pack whose list names `a` then `b`.
fn common_of(a: &str, b: &str) -> PackStack<Resource> {
    stack(&[&[(COMMON, &list(&[A, B])), (A, &wrapped(a)), (B, &wrapped(b))]])
}

#[test]
fn two_groups_whose_variants_arrive_at_different_steps_are_not_equal() {
    // In the first load `P` gets its variant from the second file, after `C` was created from
    // it; in the second `P` has the variant when `C` copies it. The materials, their parents,
    // their files and what each added in the end are the same; the names are not.
    let late = common_of(
        r#""P": {}, "C:P": {}"#,
        r#""+P": {"+variants": [{"v": {}}]}"#,
    );
    let early = common_of(
        r#""P": {"+variants": [{"v": {}}]}, "C:P": {}"#,
        r#""+P": {}"#,
    );
    let (late, early) = (&late.materials().common, &early.materials().common);
    assert_eq!(names(late), ["C", "P", "P.v"]);
    assert_eq!(names(early), ["C", "C.v", "P", "P.v"]);
    assert!(!late.contains("C.v"));
    assert!(early.contains("C.v"));
    assert!(late != early, "groups with different names compare equal");
    // The same load twice is equal.
    let again = common_of(
        r#""P": {}, "C:P": {}"#,
        r#""+P": {"+variants": [{"v": {}}]}"#,
    );
    assert_eq!(*late, again.materials().common);
}

#[test]
fn two_groups_whose_step_was_read_under_other_key_rules_are_not_equal() {
    // `P` is created by the first file and extended by the second, which is wrapped in both
    // loads: the material ends with the versioned rules either way. Its first step is read
    // with the rules of the first file's form: versioned rules drop the inherited defines
    // for a plain `defines`, legacy rules add to them.
    let listing = format!(r#"[{{"path": "{A}", "+defines": ["BASE"]}}, {{"path": "{B}"}}]"#);
    let members = r#""P": {"defines": ["X"]}"#;
    let extension = wrapped(r#""+P": {}"#);
    let load = |a: &str| stack(&[&[(COMMON, &listing), (A, a), (B, &extension)]]);
    let (versioned, legacy) = (load(&wrapped(members)), load(&flat(members)));
    let (versioned, legacy) = (versioned.materials(), legacy.materials());
    let defines = |materials: &Materials| {
        let resolved = materials.resolve(MaterialGroupKind::Common, "P");
        resolved.expect("P is a material").defines
    };
    assert_eq!(defines(versioned), ["X"]);
    assert_eq!(defines(legacy), ["BASE", "X"]);
    // The same material as a link: its name, file, final key rules and place.
    assert_eq!(versioned.common.get("P"), legacy.common.get("P"));
    assert!(
        versioned.common != legacy.common,
        "groups whose first step was read under other key rules compare equal"
    );
    // The same load twice is equal.
    assert_eq!(versioned.common, load(&wrapped(members)).materials().common);
}

#[test]
fn v20_a_key_over_two_lower_keys_of_its_name_is_reported_once_deviation() {
    // The lower copy spells `x` twice, alone and under `q`; the upper key `x:p` takes both
    // over. V20 is about the pack's key, which is one key.
    let low = wrapped(r#""p": {}, "q": {}, "x": {}, "x:q": {}"#);
    let top = wrapped(r#""p": {}, "q": {}, "x:p": {}"#);
    let stack = stack(&[&[(COMMON, &list(&[A])), (A, &low)], &[(A, &top)]]);
    let shadowed: Vec<_> = placed(stack.materials())
        .into_iter()
        .filter(|found| found.0.0 == "ShadowedDefinition")
        .collect();
    assert_eq!(
        shadowed,
        [(
            finding(
                "ShadowedDefinition",
                Info,
                "materials/x:p",
                "definition shadowed by a higher-priority source"
            ),
            Some("pack 2".to_owned()),
            Some(A.to_owned())
        )]
    );
    assert_eq!(
        stack.materials().common.get("x").unwrap().parent.as_deref(),
        Some("p")
    );
}

/// A collection keeps `rejected()` beside `diagnostics()`. A list file the
/// stack cannot read is one rejected file of the group, with its origin and its findings;
/// the findings are among the group's as well, once.
#[test]
fn a_list_that_fails_to_load_is_a_rejected_file_of_the_group() {
    let stack = stack(&[
        &[
            (COMMON, &list(&[A])),
            (SAD, "[{"),
            (A, &wrapped(r#""x": {}"#)),
        ],
        &[(FANCY, r#"{"not": "a list"}"#)],
    ]);
    let materials = stack.materials();
    // Top pack first.
    let rejected: Vec<(String, String, Vec<&str>)> = materials
        .rejected()
        .iter()
        .map(|rejected| {
            (
                rejected.origin.pack.name().to_owned(),
                rejected.origin.path.as_str().to_owned(),
                rejected
                    .diagnostics
                    .iter()
                    .filter(|finding| finding.severity == Error)
                    .map(|finding| finding.code.name())
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        rejected,
        [
            ("pack 2".to_owned(), FANCY.to_owned(), vec!["InvalidType"]),
            ("pack 1".to_owned(), SAD.to_owned(), vec!["MalformedInput"]),
        ]
    );
    // The list that loads is the group's only list, and creates its material.
    let loaded: Vec<&str> = materials
        .lists
        .iter()
        .map(|list| list.origin().path.as_str())
        .collect();
    assert_eq!(loaded, [COMMON]);
    assert_eq!(names(&materials.common), ["x"]);
    // Each rejected file's findings are the group's too, once, in the walk's order (lowest
    // pack first).
    let errors: Vec<Option<String>> = materials
        .diagnostics()
        .filter(|finding| finding.severity == Error)
        .map(|finding| {
            finding
                .location
                .path
                .as_ref()
                .map(|path| path.as_str().to_owned())
        })
        .collect();
    assert_eq!(errors, [Some(SAD.to_owned()), Some(FANCY.to_owned())]);
    // A stack whose lists all load has none.
    assert!(over_vanilla(&[]).materials().rejected().is_empty());
}

/// `item_in_hand` of `entity.material` has three
/// `+defines`; a pack that writes one keeps vanilla's second and third and loses its first.
#[test]
fn s03_an_index_override_of_a_vanilla_material_with_three_defines() {
    let vanilla = &over_vanilla(&[]).materials().files[&path(ENTITY)].materials
        [&MaterialKey::parse("item_in_hand:entity")]
        .defines
        .add
        .clone()
        .expect("vanilla's +defines");
    assert_eq!(vanilla, &["COLOR_BASED", "NO_TEXTURE", "ITEM_IN_HAND"]);
    let pack = wrapped(r#""item_in_hand:entity": {"+defines": ["USE_EMISSIVE"]}"#);
    let stack = over_vanilla(&[&[(ENTITY, &pack)]]);
    let materials = stack.materials();
    assert_eq!(all(materials), []);
    let merged =
        &materials.files[&path(ENTITY)].materials[&MaterialKey::parse("item_in_hand:entity")];
    assert_eq!(
        merged.defines.add,
        Some(vec![
            "USE_EMISSIVE".to_owned(),
            vanilla[1].clone(),
            vanilla[2].clone()
        ])
    );
}

/// the names a group has are the same with
/// the `-variants` key as without it (the key is never read; the warning is ours).
#[test]
fn m22_the_index_is_the_same_as_without_the_minus_variants_key_deviation() {
    let members = |minus: &str| {
        wrapped(&format!(
            r#""entity_static": {{"variants": [{{"skinning": {{}}}}]}},
               "c:entity_static": {{{minus}"depthFunc": "Less"}}"#
        ))
    };
    let index = |text: &str| {
        let stack = stack(&[&[(COMMON, &list(&[A])), (A, text)]]);
        let materials = stack.materials();
        let warnings: Vec<Rendered> = all(materials)
            .into_iter()
            .filter(|found| found.1 >= Info)
            .collect();
        let resolved: Vec<(String, Vec<String>)> = materials
            .resolve_all(MaterialGroupKind::Common)
            .into_iter()
            .map(|(name, material)| (name, material.defines.into_iter().collect()))
            .collect();
        (names(&materials.common), resolved, warnings)
    };
    let (with, with_resolved, with_warnings) =
        index(&members(r#""-variants": [{"skinning": {}}], "#));
    let (without, without_resolved, without_warnings) = index(&members(""));
    assert_eq!(
        with_warnings,
        [finding(
            "UnrecognizedField",
            Info,
            "materials/c:entity_static/-variants",
            "unrecognized field"
        )]
    );
    assert_eq!(without_warnings, []);
    assert_eq!(with, without);
    assert_eq!(
        with,
        ["c", "c.skinning", "entity_static", "entity_static.skinning"]
    );
    assert_eq!(with_resolved, without_resolved);
}

// One copy that spells one material name twice (`x:p` and
// `x:q`). V2 tells the second key once. Nothing folds the two keys while no higher copy is
// laid over the file: the first key defines the material and the second defines nothing
// (V18, silent after V2), yet the members of both keys are read and a fault in either is
// told at its own key.
#[test]
fn v2_two_spellings_of_one_name_in_one_copy_tell_the_members_of_both_keys() {
    const TWICE: &str = "Material \"x\" is defined twice in \"materials/a.material\" as \"x:p\" \
                         and \"x:q\". You cannot redefine a material.";
    let told = |packs: &PackStack<Resource>| -> Vec<(&'static str, String, String)> {
        placed(packs.materials())
            .into_iter()
            .map(|(finding, pack, file)| {
                assert_eq!(pack.as_deref(), Some("pack 1"));
                assert_eq!(file.as_deref(), Some(A));
                (finding.0, finding.2, finding.3)
            })
            .collect()
    };
    let say = |code: &'static str, at: &str, text: &str| (code, at.to_owned(), text.to_owned());
    // Both keys well formed: the first is the material.
    let valid = flat(r#""p": {}, "q": {}, "x:p": {"+defines": ["P"]}, "x:q": {"+defines": ["Q"]}"#);
    let alone = stack(&[&[(COMMON, &list(&[A])), (A, &valid)]]);
    assert_eq!(told(&alone), [say("DuplicateIdentifier", "x:q", TWICE)]);
    let group = alone.materials().group(MaterialGroupKind::Common);
    assert_eq!(names(group), ["p", "q", "x"]);
    assert_eq!(group.get("x").unwrap().parent.as_deref(), Some("p"));
    let x = alone
        .materials()
        .resolve(MaterialGroupKind::Common, "x")
        .unwrap();
    assert_eq!(x.defines, ["P"], "the second key adds nothing");
    // A fault under each key: both are told, each at the key it was written under.
    let faulty =
        flat(r#""p": {}, "q": {}, "x:p": {"blendSrc": "Bad"}, "x:q": {"depthFunc": "Sometimes"}"#);
    let alone = stack(&[&[(COMMON, &list(&[A])), (A, &faulty)]]);
    assert_eq!(
        told(&alone),
        [
            say("DuplicateIdentifier", "x:q", TWICE),
            say("InvalidValue", "x:p/blendSrc", "invalid value"),
            say("InvalidValue", "x:q/depthFunc", "invalid value"),
            // The first key's material was refused and its name stays taken, so the
            // second key is also told as the definition of a taken name.
            say("DuplicateIdentifier", "x:q", "duplicate identifier"),
        ]
    );
    assert!(
        alone
            .materials()
            .group(MaterialGroupKind::Common)
            .get("x")
            .is_none()
    );
    // Under a higher copy the merge folds the two keys into one member; each member still
    // has the key it was written under, and the folded material is one definition.
    let under = stack(&[
        &[(COMMON, &list(&[A])), (A, &faulty)],
        &[(A, &flat(r#""y": {}"#))],
    ]);
    assert_eq!(
        told(&under),
        [
            say("DuplicateIdentifier", "x:q", TWICE),
            say("InvalidValue", "x:p/blendSrc", "invalid value"),
            say("InvalidValue", "x:q/depthFunc", "invalid value"),
        ]
    );
}
