//! The lookups: defined, the game's own and missing; both sides of each normalisation; the
//! host; the near miss; the bytes; the order and the cost.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{DiagnosticCode, Phase, ReferenceHint, Severity};
use bedrock_addon_core::id::{Id, Kind, kind};
use bedrock_addon_core::load::LoadOptions;
use bedrock_addon_core::pack_identity::{PackPath, PackSide};

use super::super::{Findings, LinkCx, LinkOptions, Names, Target, canonical, link_with_rules};
#[allow(unused_imports, reason = "each feature set uses part of the helpers")]
use super::support::{block, empty_set, entity, hosted, item, origin, pack, set, set_of, set_with};

const ALL: LinkOptions = LinkOptions {
    lints: true,
    client: true,
};

fn id<K: Kind>(text: &str) -> Id<K> {
    Id::parse(text)
}

/// The file a target is defined in.
#[cfg(any(
    feature = "block",
    feature = "entity",
    feature = "item",
    feature = "loot"
))]
fn file(target: &Target<'_>) -> Option<String> {
    target.origin().map(|origin| origin.path.to_string())
}

#[cfg(any(feature = "block", feature = "entity"))]
fn case(name: &str) -> Target<'static> {
    Target::Missing(Some(ReferenceHint::CaseMismatch(name.to_owned())))
}

#[cfg(feature = "block")]
const NO_NAMESPACE: Target<'static> = Target::Missing(Some(ReferenceHint::MissingNamespace));
const MISSING: Target<'static> = Target::Missing(None);

#[test]
fn the_canonical_form_is_the_target_kinds() {
    let block = |text| canonical(&id::<kind::Block>(text)).into_owned();
    assert_eq!(block("stone"), "minecraft:stone");
    assert_eq!(block("demo:Lamp"), "demo:Lamp");
    let item = |text| canonical(&id::<kind::Item>(text)).into_owned();
    assert_eq!(item("minecraft:wool:3"), "minecraft:wool");
    assert_eq!(item("wool"), "minecraft:wool");
    let entity = |text| canonical(&id::<kind::Entity>(text)).into_owned();
    assert_eq!(entity("demo:pet<minecraft:entity_born>"), "demo:pet");
    // A kind whose namespace is required gets none; one that folds case is lower-cased; a
    // path and a free name are as written.
    assert_eq!(canonical(&id::<kind::Feature>("Tree")), "Tree");
    assert_eq!(
        canonical(&id::<kind::Geometry>("geometry.Lamp")),
        "geometry.lamp"
    );
    assert_eq!(
        canonical(&id::<kind::LootTable>("Loot/A.json")),
        "Loot/A.json"
    );
    assert_eq!(canonical(&id::<kind::Material>("Entity")), "Entity");
}

#[test]
fn the_names_of_a_kind_keep_the_first_definition_and_the_load_order() {
    let (first, second, third) = (origin("a.json"), origin("b.json"), origin("c.json"));
    let names: Names<'_, kind::Block> = Names::new([
        (id("demo:b"), &first),
        (id("demo:a"), &second),
        (id("demo:b"), &third),
    ]);
    assert_eq!(names.len(), 2);
    assert!(!names.is_empty());
    let listed: Vec<(&str, &str)> = names
        .iter()
        .map(|(id, origin)| (id.as_str(), origin.path.as_str()))
        .collect();
    assert_eq!(listed, [("demo:b", "a.json"), ("demo:a", "b.json")]);
    assert_eq!(
        names.get(&id("demo:b")).map(|o| o.path.as_str()),
        Some("a.json")
    );
    assert_eq!(names.get(&id("demo:c")), None);
    assert!(
        !names.is_vanilla("minecraft:stone"),
        "the tables were not asked for"
    );
}

#[test]
fn names_with_the_games_tables_know_the_games_names() {
    let place = origin("a.json");
    let names: Names<'_, kind::Block> = Names::new([(id("demo:a"), &place)]).with_vanilla();
    assert!(names.is_vanilla("minecraft:stone"));
    assert!(!names.is_vanilla("minecraft:no_such_block"));
    assert!(!names.is_vanilla("demo:a"), "defined is not the game's");
}

#[test]
fn a_near_miss_is_a_name_that_differs_in_case_or_lacks_its_namespace() {
    let place = origin("a.json");
    let names: Names<'_, kind::Feature> = Names::new([
        (id("demo:Tree"), &place),
        (id("demo:tree"), &place),
        (id("bush"), &place),
    ]);
    let near = |text| names.near_miss(&id(text));
    // The first definition that differs only in case is named.
    assert_eq!(
        near("DEMO:TREE"),
        Some(ReferenceHint::CaseMismatch("demo:Tree".to_owned()))
    );
    assert_eq!(
        near("Bush"),
        Some(ReferenceHint::CaseMismatch("bush".to_owned()))
    );
    // A bare name that a namespaced definition ends in.
    assert_eq!(near("tree"), Some(ReferenceHint::MissingNamespace));
    assert_eq!(near("Tree"), Some(ReferenceHint::MissingNamespace));
    // Neither: another name, another namespace, a path that only starts alike.
    assert_eq!(near("demo:trees"), None);
    assert_eq!(near("other:tree"), None);
    assert_eq!(near("tre"), None);
    // A kind without namespaces has no namespace to miss.
    let free: Names<'_, kind::Particle> = Names::new([(id("demo:spark"), &place)]);
    assert_eq!(free.near_miss(&id("spark")), None);
    assert_eq!(
        free.near_miss(&id("Demo:Spark")),
        Some(ReferenceHint::CaseMismatch("demo:spark".to_owned()))
    );
}

#[cfg(feature = "block")]
#[test]
fn a_block_is_a_packs_the_games_or_missing() {
    let lamp = block("demo:lamp", "{}");
    let set = set(&[("blocks/lamp.json", &lamp)], &[]);
    let cx = LinkCx::new(&set, ALL);
    let found = cx.block(&id("demo:lamp"));
    assert_eq!(file(&found).as_deref(), Some("blocks/lamp.json"));
    assert!(matches!(found, Target::Defined(origin) if origin.side == PackSide::Behavior));
    // Without a host the crate's own list answers, a bare name being `minecraft:`'s.
    assert_eq!(cx.block(&id("minecraft:stone")), Target::BuiltIn);
    assert_eq!(cx.block(&id("stone")), Target::BuiltIn);
    assert_eq!(cx.block(&id("minecraft:no_such_block")), MISSING);
    assert_eq!(cx.block(&id("demo:other")), MISSING);
    assert!(cx.block(&id("demo:other")).is_missing());
    assert!(
        !found.is_missing() && !Target::BuiltIn.is_missing() && !Target::NotInView.is_missing()
    );
    // The names are built once for the pass.
    assert!(std::ptr::eq(cx.block_names(), cx.block_names()));
    assert_eq!(cx.block_names().len(), 1);
}

#[cfg(feature = "block")]
#[test]
fn a_block_that_differs_in_case_or_lacks_its_namespace_is_missing_with_the_hint() {
    let lamp = block("demo:lamp", "{}");
    let set = set(&[("blocks/lamp.json", &lamp)], &[]);
    let cx = LinkCx::new(&set, ALL);
    // No kind but geometry and render controller folds case in the lookup.
    assert_eq!(cx.block(&id("demo:Lamp")), case("demo:lamp"));
    assert_eq!(cx.block(&id("DEMO:LAMP")), case("demo:lamp"));
    assert_eq!(cx.block(&id("Minecraft:Stone")), MISSING);
    assert_eq!(cx.block(&id("lamp")), NO_NAMESPACE);
    assert_eq!(cx.block(&id("demo:lamp")).hint(), None);
    assert_eq!(
        cx.block(&id("lamp")).hint(),
        Some(&ReferenceHint::MissingNamespace)
    );
}

#[cfg(feature = "block")]
#[test]
fn a_host_is_asked_first_and_believed_yes_or_no() {
    let lamp = block("demo:lamp", "{}");
    let options = hosted(&[
        ("block", "minecraft:moon_rock"),
        ("voxel shape", "minecraft:unit_cube"),
    ]);
    let set = set_with(&[("blocks/lamp.json", &lamp)], &[], &options);
    let cx = LinkCx::new(&set, ALL);
    assert!(cx.host_built_ins().is_some());
    // Yes: a name only the host has, bare or not, by the kind's name.
    assert_eq!(cx.block(&id("minecraft:moon_rock")), Target::BuiltIn);
    assert_eq!(cx.block(&id("moon_rock")), Target::BuiltIn);
    #[cfg(feature = "voxel_shape")]
    assert_eq!(cx.voxel_shape(&id("minecraft:unit_cube")), Target::BuiltIn);
    // No: the crate's own list is not asked behind the host's back.
    assert_eq!(cx.block(&id("minecraft:stone")), MISSING);
    // The other kind's name is not this kind's.
    assert_eq!(cx.block(&id("minecraft:unit_cube")), MISSING);
    // A pack's definition needs no host.
    assert_eq!(
        file(&cx.block(&id("demo:lamp"))).as_deref(),
        Some("blocks/lamp.json")
    );
}

#[cfg(feature = "block")]
#[test]
fn without_a_host_a_kind_without_a_list_misses() {
    let set = empty_set();
    let cx = LinkCx::new(&set, ALL);
    assert!(cx.host_built_ins().is_none());
    #[cfg(feature = "voxel_shape")]
    assert_eq!(cx.voxel_shape(&id("minecraft:unit_cube")), MISSING);
    #[cfg(feature = "block_culling")]
    assert_eq!(cx.culling_rules(&id("minecraft:leaves")), MISSING);
    assert_eq!(cx.block(&id("minecraft:stone")), Target::BuiltIn);
}

#[cfg(feature = "block")]
#[test]
fn the_states_of_a_block_of_the_stack_are_its_own() {
    let lamp = block("demo:lamp", r#"{"demo:lit": [true, false]}"#);
    let set = set(&[("blocks/lamp.json", &lamp)], &[]);
    let cx = LinkCx::new(&set, ALL);
    let states = cx
        .block_states(&id("demo:lamp"))
        .expect("a block of the stack");
    assert_eq!(states.keys().collect::<Vec<_>>(), ["demo:lit"]);
    assert_eq!(states["demo:lit"].len(), 2);
    assert!(cx.block_states(&id("demo:other")).is_none());
    assert!(cx.block_states(&id("minecraft:stone")).is_none());
}

#[cfg(feature = "entity")]
#[test]
fn an_entity_resolves_without_its_event() {
    let pet = entity("demo:pet");
    let set = set(&[("entities/pet.json", &pet)], &[]);
    let cx = LinkCx::new(&set, ALL);
    for name in ["demo:pet", "demo:pet<minecraft:entity_born>", "demo:pet<>"] {
        let found = cx.entity(&id(name));
        assert_eq!(file(&found).as_deref(), Some("entities/pet.json"), "{name}");
    }
    assert_eq!(
        cx.entity(&id("minecraft:pig<minecraft:entity_born>")),
        Target::BuiltIn
    );
    assert_eq!(cx.entity(&id("pig")), Target::BuiltIn);
    assert_eq!(cx.entity(&id("demo:pat<minecraft:entity_born>")), MISSING);
    assert_eq!(
        cx.entity(&id("demo:Pet<minecraft:entity_born>")),
        case("demo:pet")
    );
}

#[cfg(feature = "item")]
#[test]
fn an_item_is_a_packs_the_games_a_legacy_name_or_missing() {
    let ruby = item("demo:ruby", "");
    let set = set(&[("items/ruby.json", &ruby)], &[]);
    let cx = LinkCx::new(&set, ALL);
    for name in ["demo:ruby", "demo:ruby:3"] {
        assert_eq!(
            file(&cx.item(&id(name))).as_deref(),
            Some("items/ruby.json"),
            "{name}"
        );
    }
    assert_eq!(cx.item(&id("minecraft:apple")), Target::BuiltIn);
    assert_eq!(cx.item(&id("apple")), Target::BuiltIn);
    assert_eq!(cx.item(&id("minecraft:wool:3")), Target::BuiltIn);
    // The legacy names of the game's own tables, in whatever case a file spells them.
    assert_eq!(cx.item(&id("minecraft:horsearmorgold")), Target::BuiltIn);
    assert_eq!(cx.item(&id("minecraft:appleEnchanted")), Target::BuiltIn);
    assert_eq!(cx.item(&id("minecraft:record_13")), Target::BuiltIn);
    assert_eq!(cx.item(&id("minecraft:appleenchanted")), Target::BuiltIn);
    assert_eq!(cx.item(&id("minecraft:not_an_item")), MISSING);
    // A name outside `minecraft` that is no item: missing with its near miss when the blocks
    // are in view, not judged when they are not (it may be a pack's block).
    #[cfg(feature = "block")]
    assert_eq!(cx.item(&id("demo:Ruby")), case("demo:ruby"));
    #[cfg(not(feature = "block"))]
    assert_eq!(cx.item(&id("demo:Ruby")), Target::NotInView);
    // The definitions alone know neither a legacy name nor a block's item.
    assert_eq!(cx.item_definition(&id("minecraft:horsearmorgold")), MISSING);
    assert_eq!(cx.item_definition(&id("minecraft:apple")), Target::BuiltIn);
}

#[cfg(feature = "item")]
#[test]
fn a_host_that_says_no_leaves_the_legacy_item_names_standing() {
    let set = set_with(&[], &[], &hosted(&[("item", "minecraft:moon_dust")]));
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(cx.item(&id("minecraft:moon_dust")), Target::BuiltIn);
    assert_eq!(cx.item(&id("minecraft:apple")), MISSING);
    assert_eq!(cx.item(&id("minecraft:horsearmorgold")), Target::BuiltIn);
}

/// The frame has one list of legacy item names for every rule group: the names the
/// game's loot tables, recipes, trade tables, items and entity files write. A legacy name is
/// compared without ASCII case (the game's files write `appleEnchanted` and
/// `appleenchanted`, `muttonCooked` and `muttoncooked`); a current name is compared as
/// written, like every other name of the kind.
#[cfg(feature = "item")]
#[test]
fn a_legacy_item_name_of_any_of_the_games_files_is_an_item_in_any_case_with_any_host() {
    // (`minecraft:boat` is a name of the generated list, no legacy name.)
    const LEGACY: [&str; 8] = [
        // A recipe's, a trade table's, an item's descriptor's.
        "minecraft:emptyMap",
        "minecraft:emptymap",
        "minecraft:shulker_box",
        // An entity file's.
        "minecraft:carrotOnAStick",
        "minecraft:muttonCooked",
        "minecraft:muttoncooked",
        // A loot table's, in another case than the table's and with an aux value.
        "minecraft:APPLEENCHANTED",
        "minecraft:muttonraw:0",
    ];
    let plain = set(&[], &[]);
    let hosted = set_with(&[], &[], &hosted(&[("item", "minecraft:moon_dust")]));
    for set in [&plain, &hosted] {
        let cx = LinkCx::new(set, ALL);
        for name in LEGACY {
            assert_eq!(cx.item(&id(name)), Target::BuiltIn, "{name}");
        }
        // The fold is the legacy names' alone, and a legacy name has the `minecraft`
        // namespace.
        assert_eq!(cx.item(&id("minecraft:Moon_Dust")), MISSING);
        assert_ne!(cx.item(&id("demo:boat")), Target::BuiltIn);
        // The definitions alone know no legacy name.
        assert_eq!(cx.item_definition(&id("minecraft:appleenchanted")), MISSING);
        // Written by the game's files and no name of the game: the
        // rows whose files write them let them through, the lookup does not.
        assert_eq!(cx.item(&id("minecraft:nautilusarmor")), MISSING);
        assert_eq!(cx.item(&id("minecraft:iron_pick")), MISSING);
    }
    assert_eq!(
        LinkCx::new(&plain, ALL).item(&id("minecraft:Apple")),
        MISSING
    );
}

/// The block names of older versions the game's own features, entity files,
/// spawn rules and `blocks.json` files write are blocks of the game, with any host.
#[cfg(feature = "block")]
#[test]
fn a_legacy_block_name_is_a_block_of_the_game_with_any_host() {
    const LEGACY: [&str; 6] = [
        "minecraft:grass",
        "minecraft:planks",
        "minecraft:tallgrass",
        "tallgrass",
        "minecraft:sealantern",
        "minecraft:item.soul_campfire",
    ];
    let plain = set(&[], &[]);
    let hosted = set_with(&[], &[], &hosted(&[("block", "minecraft:moon_rock")]));
    for set in [&plain, &hosted] {
        let cx = LinkCx::new(set, ALL);
        for name in LEGACY {
            assert_eq!(cx.block(&id(name)), Target::BuiltIn, "{name}");
        }
        // As written, like every block name; and only in the `minecraft` namespace.
        assert_eq!(cx.block(&id("minecraft:Tallgrass")), MISSING);
        assert_eq!(cx.block(&id("demo:tallgrass")), MISSING);
        // The `blocks.json` entries are another lookup: no legacy name there.
        #[cfg(feature = "blocks_json")]
        assert_eq!(cx.client_block(&id("minecraft:tallgrass")), MISSING);
    }
    // A host is still believed about every other name: `stone`, and the stonecutter, which
    // is a block of the generated list and no legacy name.
    for name in ["minecraft:stone", "minecraft:stonecutter"] {
        assert_eq!(LinkCx::new(&plain, ALL).block(&id(name)), Target::BuiltIn);
        assert_eq!(
            LinkCx::new(&hosted, ALL).block(&id(name)),
            MISSING,
            "{name}"
        );
    }
}

/// The entities of the game that no file defines are entities, with any host.
#[cfg(feature = "entity")]
#[test]
fn an_entity_of_the_game_without_a_file_is_an_entity_with_any_host() {
    let plain = set(&[], &[]);
    let hosted = set_with(&[], &[], &hosted(&[("entity", "minecraft:moon_cow")]));
    for set in [&plain, &hosted] {
        let cx = LinkCx::new(set, ALL);
        for name in [
            "minecraft:evocation_fang",
            "minecraft:pig_zombie",
            "minecraft:skeleton.wither",
            "minecraft:evocation_fang<minecraft:entity_born>",
        ] {
            assert_eq!(cx.entity(&id(name)), Target::BuiltIn, "{name}");
        }
        assert_eq!(cx.entity(&id("minecraft:Evocation_Fang")), MISSING);
        // The identifier of the game's patrol spawn rules and of their delays is no actor:
        // the spawn-rule rows alone let it through.
        assert_eq!(cx.entity(&id("minecraft:pillager_patrol")), MISSING);
        assert_eq!(cx.entity(&id("minecraft:pillager_patrol_hard")), MISSING);
        assert_eq!(cx.entity(&id("minecraft:evocation_fangs")), MISSING);
    }
    assert_eq!(
        LinkCx::new(&hosted, ALL).entity(&id("minecraft:pig")),
        MISSING
    );
}

/// The tags the game's own recipes take an ingredient by are tags of the game, with
/// any host; any other tag of the game's items is a host's to know.
#[cfg(feature = "item")]
#[test]
fn an_item_tag_of_the_games_recipes_is_a_tag_of_the_game_with_any_host() {
    let plain = set(&[], &[]);
    let hosted = set_with(&[], &[], &hosted(&[("item tag", "minecraft:is_food")]));
    for set in [&plain, &hosted] {
        let cx = LinkCx::new(set, ALL);
        for tag in ["minecraft:planks", "minecraft:logs", "minecraft:wool"] {
            assert_eq!(cx.item_tag(tag), Target::BuiltIn, "{tag}");
        }
        assert_eq!(cx.item_tag("minecraft:Planks"), MISSING);
        assert_eq!(cx.item_tag("minecraft:is_tool"), MISSING);
    }
    assert_eq!(
        LinkCx::new(&plain, ALL).item_tag("minecraft:is_food"),
        MISSING
    );
}

/// A particle type of the game (`ParticleType`), by its name in lower case.
#[test]
fn a_particle_type_is_one_of_the_games_by_its_lower_case_name() {
    let set = empty_set();
    let cx = LinkCx::new(&set, ALL);
    for name in ["largeexplode", "wind_burst", "breeze_wind_burst", "heart"] {
        assert_eq!(cx.particle_type(name), Target::BuiltIn, "{name}");
    }
    for name in ["LargeExplode", "demo:spark", "minecraft:heart_particle", ""] {
        assert_eq!(cx.particle_type(name), MISSING, "{name}");
    }
}

#[cfg(all(feature = "item", feature = "block"))]
#[test]
fn every_block_of_the_stack_has_an_item_of_its_name() {
    let lamp = block("demo:lamp", "{}");
    let set = set(&[("blocks/lamp.json", &lamp)], &[]);
    let cx = LinkCx::new(&set, ALL);
    for name in ["demo:lamp", "demo:lamp:1"] {
        assert_eq!(
            file(&cx.item(&id(name))).as_deref(),
            Some("blocks/lamp.json"),
            "{name}"
        );
    }
    assert_eq!(cx.item_definition(&id("demo:lamp")), MISSING);
    assert_eq!(cx.item(&id("demo:lamps")), MISSING);
}

// Without the block family a name that may be a pack's block is not judged.
#[cfg(all(feature = "item", not(feature = "block")))]
#[test]
fn without_the_block_feature_a_name_that_may_be_a_packs_block_is_not_in_view() {
    let set = empty_set();
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(cx.item(&id("demo:lamp")), Target::NotInView);
    assert!(!cx.item(&id("demo:lamp")).is_missing());
    // A `minecraft:` name cannot be a pack's block.
    assert_eq!(cx.item(&id("minecraft:not_an_item")), MISSING);
}

#[cfg(feature = "item")]
#[test]
fn an_item_tag_is_one_an_item_of_the_stack_carries_or_the_hosts() {
    let ruby = item(
        "demo:ruby",
        r#""minecraft:tags": {"tags": ["demo:gem", "demo:red"]}"#,
    );
    let files = [("items/ruby.json", ruby.as_str())];
    let set = set(&files, &[]);
    let cx = LinkCx::new(&set, ALL);
    for tag in ["demo:gem", "demo:red"] {
        assert_eq!(
            file(&cx.item_tag(tag)).as_deref(),
            Some("items/ruby.json"),
            "{tag}"
        );
    }
    // As written; and without a host no tag of the game's items is known.
    assert_eq!(cx.item_tag("demo:Gem"), MISSING);
    assert_eq!(cx.item_tag("minecraft:is_food"), MISSING);
    let set = set_with(&files, &[], &hosted(&[("item tag", "minecraft:is_food")]));
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(cx.item_tag("minecraft:is_food"), Target::BuiltIn);
    assert_eq!(cx.item_tag("minecraft:is_tool"), MISSING);
    assert_eq!(
        file(&cx.item_tag("demo:gem")).as_deref(),
        Some("items/ruby.json")
    );
}

// The four things the lookup does to a loot table reference, each with the
// reference it lets through and the one it does not.
#[cfg(feature = "loot")]
#[test]
fn a_loot_table_reference_is_stripped_lower_cased_and_matched_with_or_without_json() {
    const TABLE: &str = r#"{"pools": []}"#;
    let set = set(
        &[
            ("loot_tables/Chests/Simple.json", TABLE),
            ("loot_tables/gameplay/fish.json", TABLE),
        ],
        &[],
    );
    let cx = LinkCx::new(&set, ALL);
    let found = |reference: &str| file(&cx.loot_table(reference));
    let simple = Some("loot_tables/Chests/Simple.json".to_owned());
    // As written.
    assert_eq!(found("loot_tables/Chests/Simple.json"), simple);
    // Lower-cased on both sides: the key keeps its case, the reference may have any.
    assert_eq!(found("loot_tables/chests/simple.json"), simple);
    assert_eq!(found("LOOT_TABLES/CHESTS/SIMPLE.JSON"), simple);
    assert_eq!(found("loot_tables/chests/simpel.json"), None);
    // Without the extension, and with it.
    assert_eq!(found("loot_tables/chests/simple"), simple);
    assert_eq!(
        found("loot_tables/gameplay/fish"),
        Some("loot_tables/gameplay/fish.json".to_owned())
    );
    assert_eq!(
        found("loot_tables/gameplay/fish.json"),
        Some("loot_tables/gameplay/fish.json".to_owned())
    );
    assert_eq!(found("loot_tables/gameplay/fis"), None);
    assert_eq!(found("loot_tables/gameplay/fish.json.json"), None);
    assert_eq!(found("loot_tables/gameplay/fish.jso"), None);
    // Everything up to the last `:` is dropped, and only that.
    assert_eq!(found("minecraft:loot_tables/chests/simple.json"), simple);
    assert_eq!(found("a:b:loot_tables/chests/simple"), simple);
    assert_eq!(found("minecraft/loot_tables/chests/simple.json"), None);
    assert_eq!(found("loot_tables/chests/simple.json:"), None);
    assert_eq!(cx.loot_table("loot_tables/none.json"), MISSING);
    // A host is asked for the stripped, lower-cased path.
    let set = set_with(
        &[],
        &[],
        &hosted(&[("loot table", "loot_tables/entities/pig.json")]),
    );
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(
        cx.loot_table("minecraft:Loot_Tables/entities/pig.json"),
        Target::BuiltIn
    );
    assert_eq!(cx.loot_table("loot_tables/entities/cow.json"), MISSING);
}

#[test]
fn a_command_is_one_of_the_games_by_name_or_alias() {
    let set = empty_set();
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(cx.command("teleport"), Target::BuiltIn);
    assert_eq!(cx.command("tp"), Target::BuiltIn);
    assert_eq!(cx.command("Teleport"), MISSING);
    assert_eq!(cx.command("no_such_command"), MISSING);
}

#[test]
fn a_pack_is_found_by_its_header_uuid_on_either_stack() {
    let set = set(&[], &[]);
    let cx = LinkCx::new(&set, ALL);
    let uuid = |n: u128| uuid::Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0000 + n);
    assert_eq!(
        cx.pack(uuid(1))
            .map(|pack| pack.name().to_owned())
            .as_deref(),
        Some("pack 1")
    );
    assert_eq!(
        cx.pack(uuid(2))
            .map(|pack| pack.name().to_owned())
            .as_deref(),
        Some("pack 2")
    );
    // A module's UUID is not the pack's.
    assert!(
        cx.pack(uuid::Uuid::from_u128(
            0x0000_0000_0000_4000_9000_0000_0000_0001
        ))
        .is_none()
    );
    assert!(cx.pack(uuid(3)).is_none());
    assert!(std::ptr::eq(cx.behavior(), &raw const set.behavior));
    assert!(std::ptr::eq(cx.resource(), &raw const set.resource));
    assert_eq!(cx.options(), ALL);
}

/// Only the lookups that name the game's tables ask them without a host: the `blocks.json`
/// entry of a block and an aim-assist preset document have no list of the game's, so the
/// game's block and preset are missing there (the rules that know better say so themselves).
#[cfg(any(feature = "blocks_json", feature = "aim_assist"))]
#[test]
fn a_lookup_without_a_list_of_the_games_does_not_ask_the_tables() {
    let set = empty_set();
    let cx = LinkCx::new(&set, ALL);
    #[cfg(feature = "blocks_json")]
    assert_eq!(cx.client_block(&id("minecraft:stone")), MISSING);
    #[cfg(feature = "aim_assist")]
    assert_eq!(
        cx.aim_assist_preset(&id("minecraft:aim_assist_default")),
        MISSING
    );
    #[cfg(feature = "block")]
    assert_eq!(cx.block(&id("minecraft:stone")), Target::BuiltIn);
}

#[cfg(any(feature = "sounds_json", feature = "entity"))]
#[test]
fn a_level_sound_event_is_the_hosts_or_one_the_crate_knows() {
    let set = empty_set();
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(cx.level_sound_event("ambient"), Target::BuiltIn);
    assert_eq!(cx.level_sound_event("Ambient"), MISSING);
    assert_eq!(cx.level_sound_event("custom.boom"), MISSING);
    let set = set_with(&[], &[], &hosted(&[("level sound event", "custom.boom")]));
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(cx.level_sound_event("custom.boom"), Target::BuiltIn);
    assert_eq!(cx.level_sound_event("ambient"), MISSING);
}

#[cfg(feature = "script")]
#[test]
fn a_script_module_is_one_the_crate_has_metadata_of_byte_for_byte() {
    let set = empty_set();
    let cx = LinkCx::new(&set, ALL);
    assert_eq!(cx.script_module("@minecraft/server"), Target::BuiltIn);
    assert_eq!(cx.script_module("@minecraft/Server"), MISSING);
    assert_eq!(cx.script_module("@demo/none"), MISSING);
}

/// The bytes of a file a rule asks for: resident or not, and the one finding of a file that
/// is not.
#[test]
fn the_bytes_of_a_file_are_the_packs_resident_ones_and_a_file_not_loaded_is_told_once() {
    let files = [
        ("textures/a.png", Some("resident")),
        ("textures/b.png", None),
    ];
    let (bytes_of, told) = {
        let set = set_of(
            Vec::new(),
            vec![pack(2, "resources", &files)],
            &LoadOptions::default(),
        );
        let asked = std::cell::RefCell::new(Vec::new());
        // The rule is a plain function: what it reads goes out through the findings' count.
        let cx = LinkCx::new(&set, ALL);
        let mut out = Findings::new(ALL);
        let the_pack = set.resource.packs().next().expect("one pack");
        let path = |text| PackPath::from_static(text);
        let resource = PackSide::Resource;
        // The pack's entry lists the file that is not resident too, by its path.
        let entry = cx.stack_pack(resource, &the_pack).expect("its entry");
        let listed: Vec<&str> = entry.files().iter().map(|f| f.path.as_str()).collect();
        assert!(listed.contains(&"textures/a.png") && listed.contains(&"textures/b.png"));
        assert!(cx.stack_pack(PackSide::Behavior, &the_pack).is_none());
        asked
            .borrow_mut()
            .push(cx.bytes(resource, &the_pack, &path("textures/a.png"), &mut out));
        asked
            .borrow_mut()
            .push(cx.bytes(resource, &the_pack, &path("textures/b.png"), &mut out));
        asked
            .borrow_mut()
            .push(cx.bytes(resource, &the_pack, &path("textures/b.png"), &mut out));
        // A file the pack does not have, and the pack asked for on the stack it is not on.
        asked
            .borrow_mut()
            .push(cx.bytes(resource, &the_pack, &path("textures/c.png"), &mut out));
        asked.borrow_mut().push(cx.bytes(
            PackSide::Behavior,
            &the_pack,
            &path("textures/a.png"),
            &mut out,
        ));
        (asked.into_inner(), out.finish())
    };
    let texts: Vec<Option<&[u8]>> = bytes_of.iter().map(|bytes| bytes.as_deref()).collect();
    assert_eq!(texts, [Some(&b"resident"[..]), None, None, None, None]);
    let told: Vec<_> = told.iter().collect();
    assert_eq!(told.len(), 1, "{told:?}");
    let finding = told[0];
    assert_eq!(
        finding.code,
        DiagnosticCode::from(&core_codes::FILE_NOT_LOADED)
    );
    assert_eq!(finding.severity, Severity::Info);
    assert_eq!(finding.phase, Phase::Link);
    assert_eq!(
        finding.location.path.as_ref().map(PackPath::as_str),
        Some("textures/b.png")
    );
    assert_eq!(
        finding
            .location
            .pack
            .as_ref()
            .map(bedrock_addon_core::pack_identity::PackRef::name),
        Some("pack 2")
    );
}

/// A file that is not loaded is an information: without the lints it is skipped in silence.
#[test]
fn without_the_lints_a_file_not_loaded_is_skipped_in_silence() {
    fn rule(cx: &LinkCx<'_>, out: &mut Findings) {
        let pack = cx.resource().packs().next().expect("one pack");
        let path = PackPath::from_static("textures/b.png");
        assert!(cx.bytes(PackSide::Resource, &pack, &path, out).is_none());
    }
    let set = set_of(
        Vec::new(),
        vec![pack(2, "resources", &[("textures/b.png", None)])],
        &LoadOptions::default(),
    );
    assert_eq!(link_with_rules(&set, ALL, &[rule]).iter().count(), 1);
    assert!(
        link_with_rules(
            &set,
            LinkOptions {
                lints: false,
                ..Default::default()
            },
            &[rule]
        )
        .is_empty()
    );
}

/// A made-up rule that reports every block and entity of the stack and one missing name of
/// each, so that the order of the findings shows the order of the lookups' names.
#[cfg(all(feature = "block", feature = "entity"))]
fn every_name(cx: &LinkCx<'_>, out: &mut Findings) {
    use bedrock_addon_core::diagnostic::JsonPath;

    use super::super::{At, kinds, unresolved};
    for (name, origin) in cx.block_names().iter() {
        let at = At::new(origin, JsonPath::new(), Severity::Info);
        out.push(unresolved(at, kinds::BLOCK, name.as_str(), None));
    }
    for (name, origin) in cx.entity_names().iter() {
        let at = At::new(origin, JsonPath::new(), Severity::Warning);
        let hint = cx
            .entity(&id(&name.as_str().to_uppercase()))
            .hint()
            .cloned();
        out.push(unresolved(at, kinds::ENTITY, name.as_str(), hint));
    }
}

/// The findings of a pass are the same text whenever it runs and in whatever order the packs
/// were registered.
#[cfg(all(feature = "block", feature = "entity"))]
#[test]
fn a_pass_tells_the_same_findings_twice_and_for_either_registration_order() {
    use crate::pack::{PackOrigin, PackRepository, PackSelection, WorldPacks};

    let packs = || {
        let files = |n: u8| -> Vec<(&'static str, Option<String>)> {
            vec![
                ("blocks/a.json", Some(block(&format!("demo:a{n}"), "{}"))),
                ("blocks/b.json", Some(block("demo:shared", "{}"))),
                ("entities/a.json", Some(entity(&format!("demo:pet{n}")))),
                ("entities/b.json", Some(entity("demo:shared"))),
            ]
        };
        [3u8, 4, 5].map(|n| {
            let files = files(n);
            let borrowed: Vec<_> = files.iter().map(|(p, t)| (*p, t.as_deref())).collect();
            pack(n, "data", &borrowed)
        })
    };
    let stacks = |reversed: bool| {
        let mut registered = Vec::from(packs());
        let world = WorldPacks {
            behavior: registered
                .iter()
                .map(|pack| PackSelection {
                    pack_id: pack.id().uuid,
                    version: pack.id().version.clone(),
                    subpack: None,
                })
                .collect(),
            resource: Vec::new(),
        };
        if reversed {
            registered.reverse();
        }
        let mut repository = PackRepository::new();
        for pack in registered {
            repository.add_pack(pack, PackOrigin::World);
        }
        repository.stacks(&world, LoadOptions::default())
    };
    let text =
        |set: &crate::pack::StackSet| format!("{:?}", link_with_rules(set, ALL, &[every_name]));
    let (forward, backward) = (stacks(false), stacks(true));
    let first = text(&forward);
    assert_eq!(first, text(&forward), "the same stacks, twice");
    assert_eq!(first, text(&backward), "the other registration order");
    // Four blocks and four entities: the shared name once, by the pack that wins it.
    assert_eq!(
        link_with_rules(&forward, ALL, &[every_name]).iter().count(),
        8
    );
    assert!(first.contains("CaseMismatch"), "{first}");
}

/// The cost of the lookups: `n` definitions and `n` references, half of them missing with a
/// near miss to find, against four times as many.
#[test]
fn n_lookups_among_n_definitions_cost_n() {
    use bedrock_addon_core::document::Origin;

    type Input = (Vec<(Id<kind::Block>, Origin)>, Vec<Id<kind::Block>>);
    fn input(n: usize) -> Input {
        let defined = (0..n)
            .map(|i| (id(&format!("demo:block_{i}")), origin("blocks/a.json")))
            .collect();
        let references = (0..n)
            .map(|i| match i % 4 {
                0 => id(&format!("demo:block_{i}")),
                1 => id(&format!("Demo:Block_{i}")),
                2 => id(&format!("block_{i}")),
                _ => id(&format!("demo:none_{i}")),
            })
            .collect();
        (defined, references)
    }
    let set = empty_set();
    let cx = LinkCx::new(&set, ALL);
    crate::test_cost::assert_linear_at(
        "link: n lookups among n definitions",
        20_000,
        input,
        |(defined, references): &Input| {
            let names =
                Names::new(defined.iter().map(|(id, origin)| (id.clone(), origin))).with_vanilla();
            let mut missing = 0usize;
            for reference in references {
                missing += usize::from(cx.resolve(&names, reference).is_missing());
            }
            assert_eq!(missing, references.len() - references.len().div_ceil(4));
            missing
        },
    );
}

/// The legacy item names are exactly what the game's own loot tables need: every item name of
/// the loot tables of the game's behaviour packs is an item to the lookup.
#[cfg(feature = "item")]
#[test]
#[allow(clippy::items_after_statements, reason = "the walkers follow the skip")]
fn every_item_name_of_the_vanilla_loot_tables_is_an_item() {
    use std::collections::BTreeSet;
    use std::path::Path;

    use bedrock_addon_core::json::{self, Value};

    let Some(game_files) = std::env::var_os("BEDROCK_ADDON_CORPUS") else {
        eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set");
        return;
    };
    fn tables(dir: &Path, below_loot: bool, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let loot = below_loot || path.file_name().is_some_and(|name| name == "loot_tables");
                tables(&path, loot, out);
            } else if below_loot && path.extension().is_some_and(|ext| ext == "json") {
                out.push(path);
            }
        }
    }
    fn item_names(value: &Value, out: &mut BTreeSet<String>) {
        if let Some(object) = value.as_object() {
            let is_item = object
                .get("type")
                .and_then(Value::as_string)
                .is_some_and(|kind| kind.as_str() == "item");
            if let (true, Some(name)) = (is_item, object.get("name").and_then(Value::as_string)) {
                out.insert(name.as_str().to_owned());
            }
            for (_, member) in object {
                item_names(member, out);
            }
        } else if let Some(array) = value.as_array() {
            for element in array {
                item_names(element, out);
            }
        }
    }
    let game_files = Path::new(&game_files);
    let mut files = Vec::new();
    tables(
        &game_files.join("bds-1.26.36.1").join("behavior_packs"),
        false,
        &mut files,
    );
    tables(
        &game_files.join("bedrock-samples").join("behavior_pack"),
        false,
        &mut files,
    );
    assert!(files.len() > 500, "{} loot table files", files.len());
    let mut names = BTreeSet::new();
    for file in &files {
        let text = std::fs::read_to_string(file).expect("a loot table");
        let text = text.trim_start_matches('\u{feff}');
        let Ok(value) = json::from_str_jsonc::<Value>(text) else {
            continue;
        };
        item_names(&value, &mut names);
    }
    assert!(names.len() > 400, "{} item names", names.len());
    let set = empty_set();
    let cx = LinkCx::new(&set, ALL);
    // A name the tables write that is no name of the game is the entry rule's to let
    // through, not the lookup's.
    let unknown: Vec<&String> = names
        .iter()
        .filter(|name| cx.item(&id(name)).is_missing())
        .filter(|name| !crate::vanilla::names::is_loot_entry_only_item(name))
        .collect();
    assert!(unknown.is_empty(), "no item to the lookup: {unknown:?}");
}

/// The one list of legacy item names is no more than the game's own files need: each
/// name is written, in some case, in a loot table, a recipe, a trade table, an item or an
/// entity file of the game, and none is a name of the generated list. (That the list is
/// enough is the test above for the loot tables, and the tests over the game's files in
/// `tests/` for the recipes, trade tables, items and entity files.)
#[cfg(feature = "item")]
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[test]
fn every_legacy_item_name_is_one_a_vanilla_file_writes() {
    let Some(game_files) = std::env::var_os("BEDROCK_ADDON_CORPUS").filter(|v| !v.is_empty())
    else {
        eprintln!("skipped: BEDROCK_ADDON_CORPUS is not set");
        return;
    };
    let text = super::support::vanilla_behavior_text(
        std::path::Path::new(&game_files),
        &["loot_tables", "recipes", "trading", "items", "entities"],
    );
    let items = bedrock_addon_core::vanilla::ids().items;
    for legacy in crate::vanilla::names::LEGACY_ITEM_NAMES {
        assert!(
            super::support::written_in(&text, legacy),
            "{legacy} is in no file"
        );
        assert!(!items.contains(&legacy), "{legacy} is a generated name");
    }
}
