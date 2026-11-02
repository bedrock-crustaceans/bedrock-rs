//! The fixtures (s01–s06): the walk of `models/`, the order inside a pack,
//! last-write-wins per identifier, legacy inheritance across packs, and the identifier loop.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Registry;
use bedrock_addon::families::geometry::Geometry;
use bedrock_addon::id::GeometryId;

use crate::common::stack;

fn modern(identifier: &str, bone: &str) -> String {
    format!(
        r#"{{"format_version": "1.16.0", "minecraft:geometry": [{{"description": {{"identifier": "{identifier}"}}, "bones": [{{"name": "{bone}"}}]}}]}}"#
    )
}

fn legacy(key: &str, bone: &str) -> String {
    format!(r#"{{"format_version": "1.8.0", "{key}": {{"bones": [{{"name": "{bone}"}}]}}}}"#)
}

fn bones(registry: &Registry<Geometry>, identifier: &str) -> Vec<String> {
    registry
        .get(&GeometryId::parse(identifier))
        .unwrap_or_else(|| panic!("{identifier} is not registered"))
        .value()
        .bones()
        .iter()
        .map(|b| b.name.clone())
        .collect()
}

/// The registry's findings at `severity` or above (those of its rejected documents included).
fn findings(registry: &Registry<Geometry>, severity: Severity) -> Vec<String> {
    registry
        .diagnostics()
        .filter(|d| d.severity >= severity)
        .map(|d| format!("{} @{} {}", d.code.name(), d.location.json_path, d.code))
        .collect()
}

#[test]
fn s01_override_by_priority() {
    let top = modern("geometry.pig", "top");
    let bottom = modern("geometry.pig", "bottom");
    let only_bottom = modern("geometry.cow", "cow");
    let set = stack(&[
        &[("models/entity/pig.json", &top)],
        &[
            ("models/entity/pig.geo.json", &bottom),
            ("models/entity/cow.json", &only_bottom),
        ],
    ]);
    let registry = set.resource.geometries();
    assert_eq!(bones(registry, "geometry.pig"), ["top"]);
    assert_eq!(bones(registry, "geometry.cow"), ["cow"]);
    assert_eq!(registry.len(), 2);
    assert!(findings(registry, Severity::Warning).is_empty());
    // The identifier is compared without case.
    assert_eq!(bones(registry, "Geometry.PIG"), ["top"]);
}

#[test]
fn s02_blocks_over_entity() {
    // Inside a pack: `models/mobs.json`, then `models/entity/`, then `models/blocks/`.
    let entity = modern("geometry.x", "entity");
    let blocks = modern("geometry.x", "blocks");
    let entity_y = modern("geometry.y", "entity");
    let mobs = r#"{"format_version": "1.8.0", "geometry.x": {"bones": [{"name": "mobs"}]}, "geometry.y": {"bones": [{"name": "mobs"}]}, "geometry.z": {"bones": [{"name": "mobs"}]}}"#;
    let set = stack(&[&[
        ("models/blocks/b.json", &blocks),
        ("models/entity/a.json", &entity),
        ("models/entity/y.json", &entity_y),
        ("models/mobs.json", mobs),
    ]]);
    let registry = set.resource.geometries();
    assert_eq!(bones(registry, "geometry.x"), ["blocks"]);
    assert_eq!(bones(registry, "geometry.y"), ["entity"]);
    assert_eq!(bones(registry, "geometry.z"), ["mobs"]);
    assert!(
        findings(registry, Severity::Warning).is_empty(),
        "vanilla replaces without a message"
    );
    // Ours: each replaced definition is noted.
    assert_eq!(
        findings(registry, Severity::Info),
        ["ShadowedDefinition @ definition shadowed by a higher-priority source"; 3]
    );
}

#[test]
fn files_of_one_directory_are_read_in_path_order_and_a_file_s_later_geometry_wins_deviation() {
    let a = modern("geometry.x", "a");
    let b = modern("geometry.x", "b");
    let twice = r#"{"format_version": "1.16.0", "minecraft:geometry": [
        {"description": {"identifier": "geometry.twice"}, "bones": [{"name": "first"}]},
        {"description": {"identifier": "geometry.twice"}, "bones": [{"name": "second"}]}]}"#;
    let set = stack(&[&[
        ("models/entity/b.json", &b),
        ("models/entity/a.json", &a),
        ("models/entity/twice.json", twice),
    ]]);
    let registry = set.resource.geometries();
    assert_eq!(bones(registry, "geometry.x"), ["b"]);
    assert_eq!(bones(registry, "geometry.twice"), ["second"]);
}

#[test]
fn s03_legacy_parent_cross_pack() {
    let parent = legacy("geometry.humanoid", "body");
    let child = legacy("geometry.npc:geometry.humanoid", "hat");
    let grandchild = legacy("geometry.guard:geometry.npc", "spear");
    let set = stack(&[
        &[
            ("models/entity/npc.json", &child),
            ("models/entity/guard.json", &grandchild),
        ],
        &[("models/mobs.json", &parent)],
    ]);
    let registry = set.resource.geometries();
    assert!(findings(registry, Severity::Info).is_empty());
    let npc = registry
        .get(&GeometryId::parse("geometry.npc"))
        .unwrap()
        .value();
    assert_eq!(npc.parent(), Some("geometry.humanoid"));
    assert_eq!(bones(registry, "geometry.npc"), ["hat"], "its own bones");
    assert!(npc.bone("body").is_some(), "the parent's bone is inherited");
    let inherited = npc.inherited.as_ref().unwrap();
    assert_eq!(inherited.identifier(), "geometry.humanoid");
    assert!(inherited.parent().is_none());
    let guard = registry
        .get(&GeometryId::parse("geometry.guard"))
        .unwrap()
        .value();
    let chain: Vec<&str> = guard
        .inherited
        .as_ref()
        .unwrap()
        .chain()
        .map(bedrock_addon::families::geometry::Inherited::identifier)
        .collect();
    assert_eq!(chain, ["geometry.npc", "geometry.humanoid"]);
    assert!(guard.bone("hat").is_some() && guard.bone("body").is_some());
    let humanoid = registry
        .get(&GeometryId::parse("geometry.humanoid"))
        .unwrap()
        .value();
    assert!(humanoid.inherited.is_none());
}

/// R22 make the missing parent an error (the child is not
/// built), where the fixtures s03 says warning.
#[test]
fn s03_a_child_without_its_parent_is_not_loaded() {
    let child = legacy("geometry.npc:geometry.humanoid", "hat");
    let grandchild = legacy("geometry.guard:geometry.npc", "spear");
    let other = modern("geometry.other", "body");
    let set = stack(&[&[
        ("models/entity/npc.json", &child),
        ("models/entity/guard.json", &grandchild),
        ("models/entity/other.json", &other),
    ]]);
    let registry = set.resource.geometries();
    assert!(registry.get(&GeometryId::parse("geometry.npc")).is_none());
    assert!(registry.get(&GeometryId::parse("geometry.guard")).is_none());
    assert_eq!(registry.len(), 1);
    let mut errors = findings(registry, Severity::Error);
    errors.sort();
    assert_eq!(
        errors,
        [
            "UnresolvedReference @ geometry 'geometry.guard' inherits from 'geometry.npc', which is not defined",
            "UnresolvedReference @ geometry 'geometry.npc' inherits from 'geometry.humanoid', which is not defined",
        ]
    );
}

#[test]
fn a_cycle_of_legacy_parents_loads_none_of_its_members() {
    let a = legacy("geometry.a:geometry.b", "a");
    let b = legacy("geometry.b:geometry.c", "b");
    let c = legacy("geometry.c:geometry.a", "c");
    let own = legacy("geometry.self:geometry.self", "s");
    let set = stack(&[&[
        ("models/entity/a.json", &a),
        ("models/entity/b.json", &b),
        ("models/entity/c.json", &c),
        ("models/entity/self.json", &own),
    ]]);
    let registry = set.resource.geometries();
    assert!(registry.is_empty());
    let errors = findings(registry, Severity::Error);
    assert_eq!(errors.len(), 4, "{errors:?}");
    assert!(errors.iter().all(|e| e.starts_with("ReferenceCycle @ ")));
}

/// A chain as long as a pack makes it resolves without recursion; every level sees every
/// ancestor's bones.
#[test]
fn a_legacy_chain_of_three_thousand_parents_resolves() {
    const DEPTH: usize = 3000;
    let mut members = vec![r#""geometry.g0": {"bones": [{"name": "b0"}]}"#.to_owned()];
    for level in 1..DEPTH {
        members.push(format!(
            r#""geometry.g{level}:geometry.g{}": {{"bones": [{{"name": "b{level}"}}]}}"#,
            level - 1
        ));
    }
    let file = format!(r#"{{"format_version": "1.8.0", {}}}"#, members.join(", "));
    let set = stack(&[&[("models/mobs.json", &file)]]);
    let registry = set.resource.geometries();
    assert_eq!(registry.len(), DEPTH);
    assert!(findings(registry, Severity::Info).is_empty());
    let last = registry
        .get(&GeometryId::parse(
            format!("geometry.g{}", DEPTH - 1).as_str(),
        ))
        .unwrap()
        .value();
    assert_eq!(last.inherited.as_ref().unwrap().chain().count(), DEPTH - 1);
    assert!(last.bone("b0").is_some());
    assert!(last.bone("b1500").is_some());
    // `Debug` of a geometry names what it inherits from and how long the chain is.
    let printed = format!("{last:?}");
    assert!(
        printed.contains(
            r#"inherited: Some(Inherited { identifier: "geometry.g2998", bones: 1, chain: 2999 })"#
        ),
        "{printed}"
    );
    assert!(!printed.contains("geometry.g0"), "no link above the parent");
    assert_eq!(last.clone(), *last);
}

/// The `Debug` text of a registry grows with the number of geometries,
/// not with the square of a chain's length (each geometry of a chain of 1000 printed
/// every link above it: 159 MB).
#[test]
fn a_registry_of_a_chain_of_five_thousand_prints_in_bounded_size() {
    const DEPTH: usize = 5000;
    let file = long_chain(DEPTH, None);
    let set = stack(&[&[("models/mobs.json", &file)]]);
    let registry = set.resource.geometries();
    assert_eq!(registry.len(), DEPTH);
    let printed = format!("{registry:?}");
    let per_geometry = printed.len() / DEPTH;
    assert!(
        per_geometry < 4000,
        "{per_geometry} bytes per geometry, {} in all",
        printed.len()
    );
}

#[test]
fn s04_ignored_paths() {
    let x = modern("geometry.x", "x");
    let y = modern("geometry.y", "y");
    let z = modern("geometry.z", "z");
    let set = stack(&[&[
        ("models/items/x.json", &x),
        ("models/x.geo.json", &y),
        ("models/mobs.json.bak", &z),
        ("models/entity_extra/z.json", &z),
        ("geometry/x.json", &x),
    ]]);
    let registry = set.resource.geometries();
    assert!(registry.is_empty());
    assert!(registry.rejected().is_empty());
    assert_eq!(registry.diagnostics().count(), 0);
}

#[test]
fn s06_extension_filter_and_subdirs() {
    let a = modern("geometry.a", "a");
    let b = modern("geometry.b", "b");
    let c = modern("geometry.c", "c");
    let d = modern("geometry.d", "d");
    let set = stack(&[&[
        ("models/entity/sub/a.geo.json", &a),
        ("models/entity/b.JSON", &b),
        ("models/blocks/c.txt", &c),
        ("models/blocks/deep/er/d.json", &d),
    ]]);
    let registry = set.resource.geometries();
    assert_eq!(
        bones(registry, "geometry.a"),
        ["a"],
        "sub-folders are read (deviation: also in app-shipped packs)"
    );
    assert_eq!(bones(registry, "geometry.d"), ["d"]);
    assert_eq!(registry.len(), 2);
}

#[test]
fn g09_the_identifier_loop_keeps_the_geometries_before_the_one_it_stops_at() {
    let file = r#"{"format_version": "1.21.0", "minecraft:geometry": [
        {"description": {"identifier": "geometry.before"}},
        {"description": {"identifier": "demo:geometry.x"}},
        {"description": {"identifier": "geometry.after"}}]}"#;
    let set = stack(&[&[("models/entity/x.json", file)]]);
    let registry = set.resource.geometries();
    assert_eq!(registry.len(), 1);
    assert!(
        registry
            .get(&GeometryId::parse("geometry.before"))
            .is_some()
    );
    assert!(registry.get(&GeometryId::parse("geometry.after")).is_none());
    assert_eq!(
        findings(registry, Severity::Error),
        [
            "InvalidValue @minecraft:geometry/1/description/identifier Error: namespaces are not supported for custom geometry: identifier: demo:geometry.x"
        ]
    );
    assert_eq!(
        registry.rejected().len(),
        2,
        "the offender and the one after it"
    );
}

#[test]
fn g20_an_empty_file_is_skipped_and_a_pack_without_mobs_json_says_nothing() {
    let x = modern("geometry.x", "x");
    let set = stack(&[&[
        ("models/entity/a.json", ""),
        ("models/entity/n.json", "null"),
        ("models/entity/x.json", &x),
    ]]);
    let registry = set.resource.geometries();
    assert_eq!(registry.len(), 1);
    assert!(registry.rejected().is_empty(), "skipped, not failed");
    let notes: Vec<_> = registry.diagnostics().collect();
    assert_eq!(notes.len(), 2);
    assert!(
        notes
            .iter()
            .all(|d| d.code.name() == "EmptyDocument" && d.severity == Severity::Info)
    );
    let set = stack(&[&[("models/entity/x.json", &x)]]);
    assert_eq!(set.resource.geometries().diagnostics().count(), 0);
}

#[test]
fn a_file_that_fails_is_rejected_whole_and_a_geometry_s_findings_travel_with_it() {
    let bad = r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.bad"}, "nope": 1}]}"#;
    let warned = r#"{"format_version": "1.16.0", "minecraft:geometry": [
        {"description": {"identifier": "geometry.clean"}},
        {"description": {"identifier": "geometry.warned"}, "bones": [{"name": "a", "parent": "ghost"}]}]}"#;
    let old = r#"{"format_version": "1.8.0", "geometry.old": {"cape": "c"}, "geometry.plain": {}}"#;
    let set = stack(&[&[
        ("models/entity/bad.json", bad),
        ("models/entity/warned.json", warned),
        ("models/entity/old.json", old),
    ]]);
    let registry = set.resource.geometries();
    assert_eq!(registry.len(), 4);
    assert_eq!(registry.rejected().len(), 1);
    let of = |identifier: &str| {
        registry
            .get(&GeometryId::parse(identifier))
            .unwrap()
            .diagnostics()
            .iter()
            .map(|d| d.code.name())
            .collect::<Vec<_>>()
    };
    assert!(of("geometry.clean").is_empty());
    assert_eq!(of("geometry.warned"), ["UnresolvedReference"]);
    assert_eq!(
        of("geometry.old"),
        ["DroppedField"],
        "a step's finding below the legacy key"
    );
    assert!(of("geometry.plain").is_empty());
    let document = registry.get(&GeometryId::parse("geometry.warned")).unwrap();
    assert_eq!(document.origin().key.as_deref(), Some("geometry.warned"));
    assert_eq!(document.origin().path.as_str(), "models/entity/warned.json");
}

#[test]
fn a_base_game_rule_does_not_apply_to_a_world_pack_s_block_geometry() {
    let block = r#"{"format_version": "1.16.0", "minecraft:geometry": [{"description": {"identifier": "geometry.block"}}]}"#;
    let set = stack(&[&[("models/blocks/block.json", block)]]);
    assert_eq!(set.resource.geometries().len(), 1);
}

/// Rule 21 for a legacy child: its bones may hang on a bone of the geometry it inherits from,
/// which only the stack knows (vanilla's own `witch.geo.json` does).
#[test]
fn a_legacy_child_s_bone_parent_is_looked_up_in_the_bones_it_inherits() {
    let parent = legacy("geometry.villager", "Head");
    let child = r#"{"format_version": "1.8.0", "geometry.witch:geometry.villager": {"bones": [
        {"name": "nose", "parent": "head"}, {"name": "hat", "parent": "nose"}, {"name": "wart", "parent": "chin"}]}}"#;
    // Alone, the file cannot tell: nothing is reported.
    let alone = crate::common::loaded(crate::common::load(child));
    crate::common::clean(alone.diagnostics());
    let set = stack(&[&[
        ("models/entity/villager.json", &parent),
        ("models/entity/witch.json", child),
    ]]);
    let registry = set.resource.geometries();
    let witch = registry.get(&GeometryId::parse("geometry.witch")).unwrap();
    let found: Vec<String> = witch
        .diagnostics()
        .iter()
        .map(|d| format!("{:?} {} {}", d.severity, d.location.json_path, d.code))
        .collect();
    assert_eq!(
        found,
        [
            "Info minecraft:geometry/0/bones/2/parent bone 'wart' has parent 'chin', which is not a bone of geometry 'geometry.witch'"
        ]
    );
    assert_eq!(
        witch
            .diagnostics()
            .iter()
            .next()
            .unwrap()
            .location
            .path
            .as_ref()
            .map(bedrock_addon::storage::PackPath::as_str),
        Some("models/entity/witch.json")
    );
    // A geometry without a legacy parent is judged by its file.
    let orphan = r#"{"format_version": "1.8.0", "geometry.orphan": {"bones": [{"name": "nose", "parent": "head"}]}}"#;
    let alone = crate::common::loaded(crate::common::load(orphan));
    assert_eq!(
        crate::common::count(alone.diagnostics(), "UnresolvedReference", Severity::Info),
        1
    );
}

/// One `models/mobs.json` with a chain of `depth` legacy geometries: the root has the bone
/// `root`; every other level has one bone, with the bone parent `bone_parent` when given.
fn long_chain(depth: usize, bone_parent: Option<&str>) -> String {
    let parent = bone_parent.map_or(String::new(), |p| format!(r#", "parent": "{p}""#));
    let mut members = vec![r#""geometry.g0": {"bones": [{"name": "root"}]}"#.to_owned()];
    for level in 1..depth {
        members.push(format!(
            r#""geometry.g{level}:geometry.g{}": {{"bones": [{{"name": "b{level}"{parent}}}]}}"#,
            level - 1
        ));
    }
    format!(r#"{{"format_version": "1.8.0", {}}}"#, members.join(", "))
}

/// Judging the bone parents of legacy children is linear in the length
/// of the chain. Every level's bone hangs on a bone of the chain's root, 20 000 levels up; and
/// the same with a parent no level has.
///
/// No wall-clock bound. Each variant is loaded at 5 000 and at 20 000
/// levels and held to the crate's one protocol: a walk per bone up the chain takes
/// sixteen times as long for four times the levels, the one walk of the forest four times. The
/// file is built before the clock starts; the load of the one file and its registry are timed.
#[test]
fn the_bone_parents_of_a_chain_of_twenty_thousand_legacy_geometries_are_judged_in_linear_time() {
    const DEPTH: usize = 5_000;
    // The root's bone is inherited at every level; a bone no level has is one finding per level
    // below the root.
    let variants = [
        ("geometry chain, no bone parents", None, false),
        ("geometry chain, a parent at the root", Some("root"), false),
        ("geometry chain, a missing parent", Some("ghost"), true),
    ];
    for (what, bone_parent, missing) in variants {
        let warnings = |depth: usize| if missing { depth - 1 } else { 0 };
        crate::cost::assert_linear_timed_at(what, DEPTH, |depth| {
            let file = long_chain(depth, bone_parent);
            let (set, took) = crate::cost::timed(|| {
                let set = stack(&[&[("models/mobs.json", &file)]]);
                set.resource.geometries();
                set
            });
            let registry = set.resource.geometries();
            assert_eq!(registry.len(), depth);
            let found = registry
                .diagnostics()
                .filter(|d| d.severity == Severity::Info)
                .count();
            assert_eq!(found, warnings(depth), "{bone_parent:?}");
            took
        });
    }
}

/// A bone of a sibling branch is not inherited. Two
/// children of one parent; the second hangs a bone on a bone only the first has.
#[test]
fn a_bone_of_another_branch_of_the_inheritance_tree_is_no_parent() {
    let file = r#"{"format_version": "1.8.0",
        "geometry.root": {"bones": [{"name": "trunk"}]},
        "geometry.a:geometry.root": {"bones": [{"name": "leaf", "parent": "trunk"}, {"name": "moss", "parent": "twig"}]},
        "geometry.a1:geometry.a": {"bones": [{"name": "bud", "parent": "leaf"}]},
        "geometry.b:geometry.root": {"bones": [{"name": "twig", "parent": "leaf"}, {"name": "bark", "parent": "trunk"}]}}"#;
    let set = stack(&[&[("models/mobs.json", file)]]);
    let registry = set.resource.geometries();
    assert_eq!(registry.len(), 4);
    // Whichever branch the walk takes first, its bones are gone when the other one is judged.
    let mut found = findings(registry, Severity::Info);
    found.sort();
    assert_eq!(
        found,
        [
            "UnresolvedReference @minecraft:geometry/1/bones/1/parent bone 'moss' has parent 'twig', which is not a bone of geometry 'geometry.a'",
            "UnresolvedReference @minecraft:geometry/2/bones/0/parent bone 'twig' has parent 'leaf', which is not a bone of geometry 'geometry.b'",
        ]
    );
}

/// The last loaded definition of an identifier is its only
/// candidate. When the stack stage rejects it (its legacy parent is missing), the identifier has
/// no geometry: a lower pack's definition is shadowed as usual and does not come back.
#[test]
fn a_lower_definition_does_not_come_back_when_the_winning_one_is_rejected() {
    let top = r#"{"format_version": "1.8.0", "geometry.a:geometry.nowhere": {}}"#;
    let bottom = r#"{"format_version": "1.8.0",
        "geometry.a:geometry.b": {"bones": [{"name": "limb", "parent": "ghost"}]},
        "geometry.b:geometry.gone": {}}"#;
    let set = stack(&[
        &[("models/mobs.json", top)],
        &[("models/mobs.json", bottom)],
    ]);
    let registry = set.resource.geometries();
    assert!(registry.is_empty(), "something is registered");
    let mut found = findings(registry, Severity::Info);
    found.sort();
    assert_eq!(
        found,
        [
            "ShadowedDefinition @ definition shadowed by a higher-priority source",
            "UnresolvedReference @ geometry 'geometry.a' inherits from 'geometry.nowhere', which is not defined",
            "UnresolvedReference @ geometry 'geometry.b' inherits from 'geometry.gone', which is not defined",
        ]
    );
    // Three packs: the top one rejected, two valid ones below it: none is registered.
    let valid = |bone: &str| {
        format!(
            r#"{{"format_version": "1.8.0", "geometry.a": {{"bones": [{{"name": "{bone}"}}]}}, "geometry.other": {{}}}}"#
        )
    };
    let (middle, lowest) = (valid("middle"), valid("lowest"));
    let set = stack(&[
        &[("models/mobs.json", top)],
        &[("models/mobs.json", &middle)],
        &[("models/mobs.json", &lowest)],
    ]);
    let registry = set.resource.geometries();
    assert!(registry.get(&GeometryId::parse("geometry.a")).is_none());
    assert_eq!(registry.len(), 1, "geometry.other alone");
    assert_eq!(
        findings(registry, Severity::Info)
            .iter()
            .filter(|f| f.starts_with("ShadowedDefinition"))
            .count(),
        3,
        "both lower `geometry.a` and the lowest `geometry.other`"
    );
    // The rule is about the stack stage: a file that fails on its own reserves nothing.
    let broken = r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.a"}, "nope": 1}]}"#;
    let set = stack(&[
        &[("models/entity/a.json", broken)],
        &[("models/mobs.json", &middle)],
    ]);
    assert_eq!(bones(set.resource.geometries(), "geometry.a"), ["middle"]);
}

/// The validation rules, file access: a geometry file whose bytes cannot be read
/// is skipped with the stack driver's `ReadError` error on the file, carrying the loader's own
/// text for it (`Walk::read_error`); the other files load, in every completion
/// order.
#[test]
fn a_geometry_file_that_cannot_be_read_is_skipped_with_the_loader_s_text() {
    use std::io::ErrorKind;

    use bedrock_addon::diagnostic::DiagnosticCode;
    use bedrock_addon::load::{
        LoadOptions, RepositoryLoader, RootId, Source, StackLoader, StackOptions,
    };
    use bedrock_addon::pack::{OpenOptions, PackOrigin, WorldPacks};

    use crate::responder::{Order, Responder, manifest};

    for order in Order::ALL {
        let mut responder = Responder::new(Order::Issue);
        responder
            .file(1, "rp/manifest.json", manifest(2, "resources"))
            .file(1, "rp/models/entity/pig.json", modern("geometry.pig", "a"))
            .file(1, "rp/models/entity/cow.json", modern("geometry.cow", "b"))
            .file(1, "rp/models/blocks/sub/x.json", modern("geometry.x", "c"))
            .file(1, "rp/models/mobs.json", legacy("geometry.old", "d"));
        let mut repository = RepositoryLoader::new(
            vec![Source::Directory {
                root: RootId(1),
                origin: PackOrigin::World,
            }],
            OpenOptions::default(),
        );
        let repository = responder.drive(&mut repository);
        let world = format!(
            r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
            uuid::Uuid::from_u128(2)
        );
        let world = WorldPacks::parse(Some(b"[]"), Some(world.as_bytes())).unwrap();
        let mut responder = responder.reordered(order);
        responder
            .failed(1, "rp/models/entity/pig.json", ErrorKind::PermissionDenied)
            .failed(1, "rp/models/blocks/sub/x.json", ErrorKind::NotFound)
            .failed(1, "rp/models/mobs.json", ErrorKind::Other);
        let mut loader = StackLoader::new(
            repository.stacks(&world, LoadOptions::default()),
            StackOptions::default(),
        );
        let set = responder.drive(&mut loader);
        let registry = set.resource.geometries();
        assert_eq!(bones(registry, "geometry.cow"), ["b"], "{order:?}");
        assert_eq!(registry.len(), 1, "{order:?}");
        let mut unread: Vec<(String, Severity, String, String)> = set
            .resource
            .report()
            .iter()
            .filter_map(|f| match &f.code {
                DiagnosticCode::ReadError { kind, .. } => Some((
                    f.location.path.as_ref()?.as_str().to_owned(),
                    f.severity,
                    format!("{kind:?}"),
                    f.code.to_string(),
                )),
                _ => None,
            })
            .collect();
        unread.sort();
        let row = |path: &str, kind: &str| {
            (
                path.to_owned(),
                Severity::Error,
                kind.to_owned(),
                format!("Cannot load geometry file {path}: pack has no access strategy"),
            )
        };
        assert_eq!(
            unread,
            [
                row("models/blocks/sub/x.json", "NotFound"),
                row("models/entity/pig.json", "PermissionDenied"),
                row("models/mobs.json", "Other"),
            ],
            "{order:?}"
        );
        // The unread files are no rejected documents of the family.
        assert!(findings(registry, Severity::Info).is_empty(), "{order:?}");
    }
}
