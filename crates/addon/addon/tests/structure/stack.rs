//! `bp.structures()` and the lookups by identifier: the fixtures, the stack
//! and naming rows `n01`–`n08`.

use bedrock_addon::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon::families::structure::Structure;
use bedrock_addon::id::StructureId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;
use bytes::Bytes;

use super::common::fixture;
use super::pack_common::{Spec, bp};

/// A fixture's bytes, leaked: `Spec::file` takes `'static` bytes.
fn leaked(name: &str) -> &'static [u8] {
    Box::leak(fixture(name).into_boxed_slice())
}

/// A valid structure (s01).
fn valid() -> &'static [u8] {
    leaked("s01_vanilla_1x1x1")
}

/// A valid structure whose world origin tells the copies apart.
fn marked(x: i32) -> &'static [u8] {
    let text = format!(
        r#"{{format_version: 1, size: [1, 1, 1], structure_world_origin: [{x}, 0, 0], structure: {{block_indices: [[0], [-1]], entities: [], palette: {{default: {{block_palette: [{{name: "minecraft:stone"}}], block_position_data: {{}}}}}}}}}}"#
    );
    Box::leak(super::common::bytes(&text).into_boxed_slice())
}

/// A stack of `packs`, top first.
fn stack_of(packs: impl IntoIterator<Item = Pack>) -> PackStack<Behavior> {
    PackStack::<Behavior>::of(packs, LoadOptions::default()).expect("a behaviour stack")
}

/// `spec` opened as a base-game pack.
fn base_game(spec: &Spec) -> Pack {
    let mut files = vec![(
        PackPath::from_static("manifest.json"),
        Bytes::from(spec.manifest()),
    )];
    files.extend(
        spec.files
            .iter()
            .map(|(path, bytes)| (PackPath::from_static(path), Bytes::from_static(bytes))),
    );
    let options = OpenOptions {
        origin: PackOrigin::Package,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("the pack opens")
}

fn id(text: &str) -> StructureId {
    StructureId::parse(text)
}

/// The world origin's x of the structure `find` gives for `text`, with its file.
fn found(stack: &PackStack<Behavior>, text: &str) -> Option<(i32, String)> {
    Structure::find(stack.structures(), &id(text))
        .expect("namespaced")
        .map(|d| (d.structure_world_origin[0], d.origin().path.to_string()))
}

#[test]
fn n01_a_file_at_the_structures_root_is_mystructure() {
    let stack = stack_of([bp(1).file("structures/house.mcstructure", valid()).open()]);
    let structures = stack.structures();
    let listed: Vec<_> = structures
        .iter()
        .filter_map(|(path, _)| Structure::id_of(path))
        .map(|id| id.as_str().to_owned())
        .collect();
    assert_eq!(listed, ["mystructure:house"]);
    let document = Structure::find(structures, &id("mystructure:house"))
        .unwrap()
        .expect("found");
    assert_eq!(
        document.origin().path.as_str(),
        "structures/house.mcstructure"
    );
}

#[test]
fn n02_a_nested_file_takes_its_first_folder_as_namespace() {
    let stack = stack_of([bp(1)
        .file("structures/stuff/towers/diamond.mcstructure", valid())
        .open()]);
    let path = PackPath::from_static("structures/stuff/towers/diamond.mcstructure");
    assert_eq!(
        Structure::id_of(&path).unwrap().as_str(),
        "stuff:towers/diamond"
    );
    assert_eq!(
        Structure::path_of(&id("stuff:towers/diamond")),
        Some(path.clone())
    );
    assert!(found(&stack, "stuff:towers/diamond").is_some());
}

#[test]
fn n03_the_mystructure_folder_beats_the_root_in_one_pack() {
    let stack = stack_of([bp(1)
        .file("structures/house.mcstructure", marked(1))
        .file("structures/mystructure/house.mcstructure", marked(2))
        .open()]);
    assert_eq!(
        found(&stack, "mystructure:house"),
        Some((2, "structures/mystructure/house.mcstructure".to_owned()))
    );
    assert_eq!(stack.structures().diagnostics().count(), 0);
}

#[test]
fn n04_an_identifier_without_a_namespace_is_rejected() {
    let stack = stack_of([bp(1).file("structures/house.mcstructure", valid()).open()]);
    let finding = Structure::find(stack.structures(), &id("house")).unwrap_err();
    assert!(matches!(
        finding.code,
        DiagnosticCode::NamespaceRule { text: Some(_) }
    ));
    assert_eq!(finding.severity, Severity::Error);
    assert_eq!(
        finding.code.to_string(),
        "Structure 'house' must be prefixed by a namespace"
    );
    assert!(Structure::path_of(&id("house")).is_none());
}

#[test]
fn n05_a_broken_top_copy_falls_through_to_the_lower_pack_deviation() {
    // Vanilla logs the top copy's error inside its `Loading structure …` scope and tries the
    // next pack; ours: the copy's diagnostic names its pack (a deviation).
    let top = bp(1)
        .file("structures/demo/x.mcstructure", leaked("s04_format_2"))
        .open();
    let bottom = bp(2)
        .file("structures/demo/x.mcstructure", marked(7))
        .open();
    let stack = stack_of([top, bottom]);
    assert_eq!(
        found(&stack, "demo:x"),
        Some((7, "structures/demo/x.mcstructure".to_owned()))
    );
    let structures = stack.structures();
    let findings: Vec<_> = structures.diagnostics().collect();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].severity, Severity::Error);
    assert!(matches!(
        findings[0].code,
        DiagnosticCode::UnsupportedVersion { .. }
    ));
    assert_eq!(findings[0].location.pack.as_ref().unwrap().name(), "bp 1");
    assert_eq!(structures.rejected().len(), 1);
    assert_eq!(
        structures
            .get(&PackPath::from_static("structures/demo/x.mcstructure"))
            .unwrap()
            .origin()
            .pack
            .name(),
        "bp 2"
    );
}

#[test]
fn n06_lookups_and_listing_are_case_sensitive_deviation() {
    // The listing compares the extension exactly; the lookup builds the path exactly. Vanilla
    // on a case-insensitive storage would find `Foo:Bar` (); ours finds it on none.
    let stack = stack_of([bp(1).file("structures/Foo/Bar.MCSTRUCTURE", valid()).open()]);
    assert_eq!(stack.structures().iter().count(), 0);
    assert!(found(&stack, "Foo:Bar").is_none());
    assert!(Structure::id_of(&PackPath::from_static("structures/Foo/Bar.MCSTRUCTURE")).is_none());
    assert_eq!(
        Structure::path_of(&id("Foo:Bar")).unwrap().as_str(),
        "structures/Foo/Bar.mcstructure"
    );
}

#[test]
fn n07_a_jigsaw_location_drops_its_namespace() {
    let stack = stack_of([bp(1)
        .file("structures/ruins/tower.mcstructure", valid())
        .open()]);
    let structures = stack.structures();
    let template = Structure::find_template(structures, &id("demo:ruins/tower")).expect("found");
    assert_eq!(
        template.origin().path.as_str(),
        "structures/ruins/tower.mcstructure"
    );
    // A location without a namespace, as vanilla's pools write them.
    assert!(Structure::find_template(structures, &id("ruins/tower")).is_some());
    // `/structure load demo:ruins/tower` reads `structures/demo/ruins/tower.mcstructure`.
    assert!(found(&stack, "demo:ruins/tower").is_none());
}

#[test]
fn n08_an_add_on_nbt_file_is_ignored_with_an_info_deviation() {
    // Vanilla reads `structures/**/*.nbt` from base-game packs only and says nothing; ours: one
    // `IgnoredFile` info naming the file ().
    let stack = stack_of([bp(1)
        .file("structures/ruins/tower.nbt", b"\x1f\x8b not read")
        .open()]);
    let structures = stack.structures();
    assert_eq!(structures.iter().count(), 0);
    let findings: Vec<_> = structures.diagnostics().collect();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(matches!(
        findings[0].code,
        DiagnosticCode::IgnoredFile { text: None }
    ));
    assert_eq!(findings[0].severity, Severity::Info);
    assert_eq!(
        findings[0].location.path.as_ref().unwrap().as_str(),
        "structures/ruins/tower.nbt"
    );
    // A base-game pack's `.nbt` templates are vanilla's own jigsaw data: no finding.
    let stack = stack_of([base_game(
        &bp(1).file("structures/ruins/tower.nbt", b"\x1f\x8b not read"),
    )]);
    assert_eq!(stack.structures().diagnostics().count(), 0);
}

#[test]
fn a_pack_whose_primary_copy_fails_does_not_read_its_root_copy() {
    // `_findResource` reads `structures/<name>.mcstructure` only when the pack has no
    // `structures/mystructure/<name>.mcstructure`: the top pack's failed primary copy ends
    // that pack's turn, and the lower pack's primary copy is used.
    let top = bp(1)
        .file("structures/house.mcstructure", marked(1))
        .file(
            "structures/mystructure/house.mcstructure",
            leaked("s04_format_2"),
        )
        .open();
    let bottom = bp(2)
        .file("structures/mystructure/house.mcstructure", marked(3))
        .open();
    let stack = stack_of([top, bottom]);
    assert_eq!(
        found(&stack, "mystructure:house"),
        Some((3, "structures/mystructure/house.mcstructure".to_owned()))
    );
}

#[test]
fn a_higher_packs_root_copy_beats_a_lower_packs_mystructure_copy() {
    let top = bp(1).file("structures/house.mcstructure", marked(1)).open();
    let bottom = bp(2)
        .file("structures/mystructure/house.mcstructure", marked(2))
        .open();
    let stack = stack_of([top, bottom]);
    assert_eq!(
        found(&stack, "mystructure:house"),
        Some((1, "structures/house.mcstructure".to_owned()))
    );
    // Another namespace never reads the root.
    assert!(found(&stack, "demo:house").is_none());
}

#[test]
fn the_top_valid_copy_of_a_path_wins() {
    let top = bp(1)
        .file("structures/demo/x.mcstructure", marked(1))
        .open();
    let bottom = bp(2)
        .file("structures/demo/x.mcstructure", marked(2))
        .open();
    let stack = stack_of([top, bottom]);
    assert_eq!(
        found(&stack, "demo:x"),
        Some((1, "structures/demo/x.mcstructure".to_owned()))
    );
    assert_eq!(stack.structures().diagnostics().count(), 0);
}
