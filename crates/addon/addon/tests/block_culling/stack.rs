//! `rp.block_culling()` and the walk of `block_culling/`.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::Registry;
use bedrock_addon::families::block_culling::{CullingRules, Facing};
use bedrock_addon::id::CullingRulesId;
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{
    PackOrigin, PackRepository, PackSelection, PackStack, Resource, StackSet, WorldPacks,
};

use super::common::document;
use super::pack_common::{Spec, rp};

fn id(text: &str) -> CullingRulesId {
    CullingRulesId::new(text).unwrap()
}

/// Text that lives as long as the test binary (a `Spec` holds `'static` bytes).
fn leak(text: String) -> &'static [u8] {
    Box::leak(text.into_boxed_str()).as_bytes()
}

/// A 1.21.80 document named `identifier` whose one rule hides the bone `bone` against `up`.
fn rules(identifier: &str, bone: &str) -> &'static [u8] {
    let rule = format!(r#"{{"geometry_part": {{"bone": "{bone}"}}, "direction": "up"}}"#);
    leak(document("1.21.80", identifier, &[&rule]))
}

/// The resource stack of `packs`, top first, none of them the base game's.
fn stack(packs: &[Spec]) -> PackStack<Resource> {
    PackStack::<Resource>::of(packs.iter().map(Spec::open), LoadOptions::default()).unwrap()
}

/// The stacks of a world that lists the resource packs `packs`, top first, each registered
/// with its origin.
fn world(packs: &[(Spec, PackOrigin)]) -> StackSet {
    let mut repository = PackRepository::new();
    let mut resource = Vec::new();
    for (spec, origin) in packs {
        repository.add_pack(spec.open(), *origin);
        let id = spec.id();
        resource.push(PackSelection {
            pack_id: id.uuid,
            version: id.version,
            subpack: None,
        });
    }
    let world = WorldPacks {
        behavior: Vec::new(),
        resource,
    };
    repository.stacks(&world, LoadOptions::default())
}

/// The bone of the one rule of `identifier`'s effective document.
fn bone(registry: &Registry<CullingRules>, identifier: &str) -> String {
    registry.get(&id(identifier)).unwrap().rules[0]
        .geometry_part
        .bone
        .clone()
}

/// `(code name, severity, pack name, file)` of every finding of the registry.
fn findings(registry: &Registry<CullingRules>) -> Vec<(&'static str, Severity, String, String)> {
    registry
        .diagnostics()
        .map(|f| {
            (
                f.code.id(),
                f.severity,
                f.location
                    .pack
                    .as_ref()
                    .map(|pack| pack.name().to_owned())
                    .unwrap_or_default(),
                f.location
                    .path
                    .as_ref()
                    .map(|path| path.as_str().to_owned())
                    .unwrap_or_default(),
            )
        })
        .collect()
}

#[test]
fn the_higher_pack_s_document_wins() {
    let stack = stack(&[
        rp(1).file("block_culling/a.json", rules("demo:hedge", "top")),
        rp(2).file("block_culling/a.json", rules("demo:hedge", "bottom")),
    ]);
    let registry = stack.block_culling();
    assert_eq!(registry.len(), 1);
    assert_eq!(bone(registry, "demo:hedge"), "top");
    assert_eq!(
        registry
            .get(&id("demo:hedge"))
            .unwrap()
            .origin()
            .pack
            .name(),
        "rp 1"
    );
    assert!(registry.rejected().is_empty());
    assert_eq!(
        findings(registry),
        [],
        "vanilla overwrites without a message"
    );
    assert!(std::ptr::eq(registry, stack.block_culling()));
}

#[test]
fn of_three_packs_the_top_one_wins_whatever_the_file_names() {
    let stack = stack(&[
        rp(1).file("block_culling/z.json", rules("demo:hedge", "top")),
        rp(2).file("block_culling/a.json", rules("demo:hedge", "middle")),
        rp(3).file("block_culling/m.json", rules("demo:hedge", "bottom")),
    ]);
    assert_eq!(bone(stack.block_culling(), "demo:hedge"), "top");
    // The identifier is the key, not the file name.
    let stack = stack_of_two_identifiers();
    assert_eq!(stack.block_culling().len(), 2);
}

fn stack_of_two_identifiers() -> PackStack<Resource> {
    stack(&[
        rp(1).file("block_culling/a.json", rules("demo:hedge", "x")),
        rp(2).file("block_culling/a.json", rules("demo:fence", "y")),
    ])
}

#[test]
fn a_failed_override_keeps_the_lower_document() {
    // The map is written only on success, so the lower pack's document
    // stays registered: the folder's rule for this family.
    let broken = leak(document(
        "1.21.80",
        "demo:hedge",
        &[r#"{"direction": "up"}"#],
    ));
    let stack = stack(&[
        rp(1).file("block_culling/a.json", broken),
        rp(2).file("block_culling/a.json", rules("demo:hedge", "bottom")),
    ]);
    let registry = stack.block_culling();
    assert_eq!(bone(registry, "demo:hedge"), "bottom");
    assert_eq!(registry.rejected().len(), 1);
    assert_eq!(registry.rejected()[0].origin.pack.name(), "rp 1");
    let errors: Vec<&str> = registry
        .diagnostics()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| f.code.id())
        .collect();
    assert_eq!(
        errors,
        [
            "core/missing_required_field_worded",
            "block_culling/failed_to_parse"
        ]
    );
}

#[test]
fn of_three_packs_a_failed_top_keeps_the_middle_one() {
    let broken = leak(document("1.21.90", "demo:hedge", &[]));
    let stack = stack(&[
        rp(1).file("block_culling/a.json", broken),
        rp(2).file("block_culling/a.json", rules("demo:hedge", "middle")),
        rp(3).file("block_culling/a.json", rules("demo:hedge", "bottom")),
    ]);
    assert_eq!(bone(stack.block_culling(), "demo:hedge"), "middle");
    // A file that fails its identifier check registers nothing either.
    let stack = stack_with_bad_identifier();
    assert!(stack.block_culling().is_empty());
    assert_eq!(stack.block_culling().rejected().len(), 1);
}

fn stack_with_bad_identifier() -> PackStack<Resource> {
    stack(&[rp(1).file("block_culling/a.json", rules("minecraft:hedge", "x"))])
}

#[test]
fn two_files_of_one_pack_the_later_path_wins_with_a_warning_deviation() {
    // Vanilla: a silent overwrite, in directory order. Ours: the
    // bytewise later path wins, with a `DuplicateIdentifier` warning.
    let stack = stack(&[rp(1)
        .file("block_culling/b.json", rules("demo:hedge", "from_b"))
        .file("block_culling/a.json", rules("demo:hedge", "from_a"))
        .file("block_culling/B.json", rules("demo:hedge", "from_upper_b"))]);
    let registry = stack.block_culling();
    assert_eq!(registry.len(), 1);
    // Bytewise: `B.json` < `a.json` < `b.json`.
    assert_eq!(bone(registry, "demo:hedge"), "from_b");
    let found = findings(registry);
    assert_eq!(found.len(), 2, "{found:#?}");
    for finding in &found {
        assert_eq!(
            (finding.0, finding.1, finding.2.as_str()),
            ("core/duplicate_identifier", Severity::Info, "rp 1")
        );
    }
}

#[test]
fn a_sub_folder_is_read_in_a_user_pack_and_not_in_a_base_game_pack_deviation() {
    // The folder reads the game's directory walk with `false` as "top level only" (inferred). The same call in
    // the dialogue loader behaves so: a pack outside the app package is walked
    // with its sub-folders, a pack inside it flat. So `block_culling/sub/x.json` is
    // loaded from a user pack and skipped in a base-game pack.
    let user = stack(&[rp(1).file("block_culling/sub/deeper/x.json", rules("demo:hedge", "x"))]);
    assert_eq!(user.block_culling().len(), 1);
    assert_eq!(
        user.block_culling()
            .get(&id("demo:hedge"))
            .unwrap()
            .origin()
            .path
            .as_str(),
        "block_culling/sub/deeper/x.json"
    );
    let set = world(&[(
        rp(1)
            .file("block_culling/sub/x.json", rules("minecraft:nested", "x"))
            .file("block_culling/flat.json", rules("minecraft:flat", "y")),
        PackOrigin::Package,
    )]);
    let registry = set.resource.block_culling();
    assert_eq!(registry.len(), 1, "{:#?}", findings(registry));
    assert!(registry.get(&id("minecraft:flat")).is_some());
    assert!(
        registry.rejected().is_empty(),
        "the nested file is not read at all"
    );
}

#[test]
fn the_default_path_of_a_minecraft_document_is_read_back_by_a_base_game_pack() {
    use bedrock_addon::document::{Content, Placed};
    // Only a base-game pack may use the namespace, and its walk is flat.
    let written = CullingRules::new(id("minecraft:culling.leaves"), Vec::new());
    let path: &'static str = Box::leak(written.default_path().as_str().to_owned().into_boxed_str());
    assert_eq!(path, "block_culling/culling.leaves.json");
    let set = world(&[(
        rp(1).file(path, leak(written.to_json().unwrap())),
        PackOrigin::Package,
    )]);
    let registry = set.resource.block_culling();
    assert!(
        registry.get(&id("minecraft:culling.leaves")).is_some(),
        "{:#?}",
        findings(registry)
    );
    // The path a document of another namespace gets is one a base-game pack does not read.
    let nested = CullingRules::new(id("demo:hedge"), Vec::new());
    let path: &'static str = Box::leak(nested.default_path().as_str().to_owned().into_boxed_str());
    let set = world(&[(
        rp(1).file(path, leak(nested.to_json().unwrap())),
        PackOrigin::Package,
    )]);
    assert!(set.resource.block_culling().is_empty());
}

#[test]
fn a_slash_and_an_underscore_in_a_minecraft_name_share_the_default_path() {
    use bedrock_addon::document::{Content, Placed};
    // The folder names no file for an identifier (the client finds a document by what it
    // says, not by its file's name), and a base-game pack's walk is flat, so the `/` of a
    // `minecraft:` name becomes `_`: two identifiers can ask for one path.
    let slash = CullingRules::new(id("minecraft:a/b"), Vec::new());
    let underscore = CullingRules::new(id("minecraft:a_b"), Vec::new());
    assert_eq!(slash.default_path().as_str(), "block_culling/a_b.json");
    assert_eq!(underscore.default_path(), slash.default_path());
    // Either is read back from that path by a base-game pack, as itself.
    for written in [&slash, &underscore] {
        let set = world(&[(
            rp(1).file("block_culling/a_b.json", leak(written.to_json().unwrap())),
            PackOrigin::Package,
        )]);
        let registry = set.resource.block_culling();
        assert_eq!(registry.len(), 1, "{:#?}", findings(registry));
        assert!(registry.get(&written.description.identifier).is_some());
    }
    // One pack holds both under two file names: the default path is a default, and the
    // document's text does not depend on it.
    let set = world(&[(
        rp(1)
            .file("block_culling/a_b.json", leak(slash.to_json().unwrap()))
            .file(
                "block_culling/a_b_2.json",
                leak(underscore.to_json().unwrap()),
            ),
        PackOrigin::Package,
    )]);
    let registry = set.resource.block_culling();
    assert_eq!(registry.len(), 2, "{:#?}", findings(registry));
    assert!(registry.get(&id("minecraft:a/b")).is_some());
    assert!(registry.get(&id("minecraft:a_b")).is_some());
    assert!(findings(registry).is_empty());
    // Outside the `minecraft` namespace the two never meet.
    assert_eq!(
        CullingRules::new(id("demo:a/b"), Vec::new())
            .default_path()
            .as_str(),
        "block_culling/demo/a/b.json"
    );
    assert_eq!(
        CullingRules::new(id("demo:a_b"), Vec::new())
            .default_path()
            .as_str(),
        "block_culling/demo/a_b.json"
    );
}

#[test]
fn the_default_path_of_a_document_is_read_back_by_a_user_pack() {
    use bedrock_addon::document::{Content, Placed};
    let written = CullingRules::new(id("demo:hedge"), Vec::new());
    let path: &'static str = Box::leak(written.default_path().as_str().to_owned().into_boxed_str());
    assert_eq!(path, "block_culling/demo/hedge.json");
    let stack = stack(&[rp(1).file(path, leak(written.to_json().unwrap()))]);
    assert!(stack.block_culling().get(&id("demo:hedge")).is_some());
}

#[test]
fn only_files_ending_in_json_exactly_are_read() {
    let stack = stack(&[rp(1)
        .file("block_culling/x.JSON", rules("demo:upper", "x"))
        .file("block_culling/x.Json", rules("demo:mixed", "x"))
        .file("block_culling/x.json5", rules("demo:five", "x"))
        .file("block_culling/x.txt", rules("demo:text", "x"))
        .file("block_culling/json", rules("demo:bare", "x"))
        .file("block_culling/x.json", rules("demo:lower", "x"))
        .file(
            "block_culling/y.culling.json",
            rules("demo:vanilla_style", "x"),
        )]);
    let registry = stack.block_culling();
    let mut loaded: Vec<&str> = registry.iter().map(|(id, _)| id.as_str()).collect();
    loaded.sort_unstable();
    assert_eq!(loaded, ["demo:lower", "demo:vanilla_style"]);
    assert!(registry.rejected().is_empty());
}

#[test]
fn only_the_block_culling_directory_of_a_resource_pack_is_read() {
    let stack = stack(&[rp(1)
        .file("block_cullings/a.json", rules("demo:a", "x"))
        .file("block_culling.json", rules("demo:b", "x"))
        .file("culling/a.json", rules("demo:c", "x"))
        .file("textures/block_culling/a.json", rules("demo:d", "x"))]);
    assert!(stack.block_culling().is_empty());
    assert!(stack.block_culling().rejected().is_empty());
}

#[test]
fn the_minecraft_namespace_follows_the_pack_s_origin() {
    let set = world(&[
        (
            rp(1).file("block_culling/a.json", rules("minecraft:culling.x", "user")),
            PackOrigin::User,
        ),
        (
            rp(2).file(
                "block_culling/a.json",
                rules("minecraft:culling.x", "vanilla"),
            ),
            PackOrigin::Package,
        ),
    ]);
    let registry = set.resource.block_culling();
    assert_eq!(bone(registry, "minecraft:culling.x"), "vanilla");
    assert_eq!(registry.rejected().len(), 1);
    assert_eq!(registry.rejected()[0].origin.pack.name(), "rp 1");
}

#[test]
fn identifiers_are_looked_up_as_written() {
    let stack = stack(&[rp(1)
        .file("block_culling/a.json", rules("demo:Hedge", "upper"))
        .file("block_culling/b.json", rules("demo:hedge", "lower"))]);
    let registry = stack.block_culling();
    assert_eq!(registry.len(), 2);
    assert_eq!(bone(registry, "demo:Hedge"), "upper");
    assert_eq!(bone(registry, "demo:hedge"), "lower");
    assert_eq!(findings(registry), []);
}

/// A document with `count` rules, each a face of its own cube.
fn many_rules(count: usize) -> String {
    let rules: Vec<String> = (0..count)
        .map(|cube| {
            format!(
                r#"{{"condition": "same_block", "geometry_part": {{"bone": "b{}", "cube": {cube}, "face": "up"}}, "direction": "down"}}"#,
                cube % 7
            )
        })
        .collect();
    let rules: Vec<&str> = rules.iter().map(String::as_str).collect();
    document("1.21.80", "demo:many", &rules)
}

/// A document of many rules loads with every rule, in order. (What the rule checks cost is
/// measured on the checks themselves: `src/block/client/cost.rs`.)
#[test]
fn a_document_of_many_rules_loads_every_rule() {
    let count = 300;
    let stack = stack(&[rp(1).file("block_culling/many.json", leak(many_rules(count)))]);
    let registry = stack.block_culling();
    let document = registry.get(&id("demo:many")).expect("it loads");
    assert_eq!(document.rules.len(), count);
    assert_eq!(document.rules[count - 1].geometry_part.cube, Some(299));
    assert_eq!(document.rules[count - 1].direction, Facing::Down);
    assert_eq!(findings(registry), []);
}

/// A file whose every rule repeats the first one's part is refused with one finding per
/// repeat.
#[test]
fn a_document_of_many_repeated_rules_is_refused_with_a_finding_each() {
    let count = 300;
    let rule = r#"{"geometry_part": {"bone": "b"}, "direction": "up"}"#;
    let file = leak(document("1.21.80", "demo:many", &vec![rule; count]));
    let stack = stack(&[rp(1).file("block_culling/many.json", file)]);
    let registry = stack.block_culling();
    assert!(registry.is_empty());
    let found = registry
        .diagnostics()
        .filter(|f| f.code.id() == "block_culling/duplicate_part")
        .count();
    assert_eq!(found, count - 1);
}
