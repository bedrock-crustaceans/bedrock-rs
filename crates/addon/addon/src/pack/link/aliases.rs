//! The names of the game's own content that no definition of a stack and no generated list
//! has: one list per kind, used by every rule group through the lookups of [`LinkCx`], so that
//! every group answers the same for one name. Each list says what it holds and where each
//! name comes from; the tests beside the lookups hold the lists against the game's files
//! when those are available.
//!
//! All of them stand with any host: the tables are the crate's own, and a host that answers
//! for a kind is not asked about these names (a host can add to them, never take one away).
//!
//! [`LinkCx`]: super::LinkCx

#![allow(
    dead_code,
    reason = "the frame of the rule packages: which lists are used depends on the enabled \
              features"
)]

/// Item names of older versions that the game still answers to and that the generated list of
/// the game's items (`bedrock_addon_core::vanilla::ids().items`, the names of
/// `mojang-items.json`) does not have: the game resolves each to an item, so the item lookup
/// counts each as one of the game's. In lower case, sorted bytewise; which current item each
/// stands for is not recorded, since the lookup only needs to know that it is one.
///
/// A name is compared without ASCII case ([`is_legacy_item_name`]): the game's item registry
/// holds these names in lower case (`minecraft:appleenchanted`, `minecraft:muttoncooked`,
/// `minecraft:carrotonastick`, `minecraft:emptymap`, `minecraft:netherstar` are registered in
/// lower case) while its own files write them in mixed case too
/// (`minecraft:appleEnchanted` in loot tables, `minecraft:muttonCooked` beside
/// `minecraft:muttoncooked` in entity files, `minecraft:emptyMap` in trade tables,
/// `carrotOnAStick` in entity files), and every one of those files loads.
///
/// The list is exactly the item names that the game's own files write in a member a link
/// rule resolves as an item and that neither a definition of the game's packs nor the
/// generated list has, lower-cased:
///
/// - the `name` of every `"type": "item"` entry of the loot tables of the game's behaviour
///   packs (`behavior_packs/*/loot_tables`);
/// - the ingredients, results and unlock items of every recipe, the trade items of every
///   trade table and the item descriptors of every item of the same packs, each pack read
///   alone so that the files of the older slices count too;
/// - the item members of the entity files (`entities/`) of the game's behaviour packs.
///
/// A name the game's files write that is no name of the game (`minecraft:iron_pick`,
/// `minecraft:nautilusarmor`) is not here: see [`LOOT_ENTRY_ONLY_ITEMS`] and
/// [`ENTITY_FILE_ONLY_ITEMS`].
// `tests::lookups::every_item_name_of_the_vanilla_loot_tables_is_an_item`,
// `tests::lookups::every_legacy_item_name_is_one_a_vanilla_file_writes`,
// `tests/link/content/vanilla.rs` and `tests/link/entity/audit.rs` hold the list against the
// game's files.
pub(crate) const LEGACY_ITEM_NAMES: [&str; 71] = [
    "minecraft:appleenchanted",
    "minecraft:carpet",
    "minecraft:carrotonastick",
    "minecraft:chain",
    "minecraft:chest_boat",
    "minecraft:chorus_fruit_popped",
    "minecraft:clownfish",
    "minecraft:concrete_powder",
    "minecraft:cooked_fish",
    "minecraft:coral_block",
    "minecraft:darkoak_sign",
    "minecraft:double_plant",
    "minecraft:double_stone_slab",
    "minecraft:double_stone_slab2",
    "minecraft:double_stone_slab3",
    "minecraft:double_stone_slab4",
    "minecraft:emptymap",
    "minecraft:fence",
    "minecraft:fireball",
    "minecraft:fish",
    "minecraft:grass",
    "minecraft:horsearmordiamond",
    "minecraft:horsearmorgold",
    "minecraft:horsearmoriron",
    "minecraft:horsearmorleather",
    "minecraft:log",
    "minecraft:log2",
    "minecraft:map",
    "minecraft:melon",
    "minecraft:muttoncooked",
    "minecraft:muttonraw",
    "minecraft:nametag",
    "minecraft:netherstar",
    "minecraft:planks",
    "minecraft:record_11",
    "minecraft:record_13",
    "minecraft:record_5",
    "minecraft:record_blocks",
    "minecraft:record_cat",
    "minecraft:record_chirp",
    "minecraft:record_far",
    "minecraft:record_mall",
    "minecraft:record_mellohi",
    "minecraft:record_otherside",
    "minecraft:record_pigstep",
    "minecraft:record_stal",
    "minecraft:record_strad",
    "minecraft:record_wait",
    "minecraft:record_ward",
    "minecraft:red_flower",
    "minecraft:sapling",
    "minecraft:sealantern",
    "minecraft:shulker_box",
    "minecraft:sign",
    "minecraft:skull",
    "minecraft:speckled_melon",
    "minecraft:stained_glass",
    "minecraft:stained_glass_pane",
    "minecraft:stained_hardened_clay",
    "minecraft:stone_slab",
    "minecraft:stone_slab2",
    "minecraft:stone_slab3",
    "minecraft:stone_slab4",
    "minecraft:stonebrick",
    "minecraft:tallgrass",
    "minecraft:totem",
    "minecraft:turtle_shell_piece",
    "minecraft:wood",
    "minecraft:wooden_slab",
    "minecraft:wool",
    "minecraft:yellow_flower",
];

/// Whether `name` (an item name with its namespace and without an aux value) is one of
/// [`LEGACY_ITEM_NAMES`], compared without ASCII case.
pub(crate) fn is_legacy_item_name(name: &str) -> bool {
    if name.bytes().any(|byte| byte.is_ascii_uppercase()) {
        listed(&LEGACY_ITEM_NAMES, &name.to_ascii_lowercase())
    } else {
        listed(&LEGACY_ITEM_NAMES, name)
    }
}

/// Block names of older versions that the game still answers to and that the generated list
/// of the game's blocks (`bedrock_addon_core::vanilla::ids().blocks`, the names of
/// `mojang-blocks.json`) does not have: the game resolves each to a block, so the block
/// lookup counts each as one of the game's. In lower case, sorted bytewise; which current
/// block each stands for is not recorded.
///
/// A name is compared in the canonical form of its kind, as written, like every other block
/// name: the game's registry looks a name up as given, then lower-cased with `minecraft:`
/// in front of a bare name, then among its aliases; what a source member does to its text
/// (a key of `blocks.json` is lower-cased: `seaLantern` is `minecraft:sealantern`) is the
/// rule's.
///
/// The list is exactly the block names that the game's own files write in a member a link
/// rule resolves as a block and that neither a definition of the game's packs nor the
/// generated list has:
///
/// - the block members of the entity files (`entities/`), the spawn rules (`spawn_rules/`)
///   and the spawn groups of the game's behaviour packs: 24 names, among them the three the
///   game's features write (`minecraft:grass`, `minecraft:planks`, `minecraft:tallgrass`);
/// - the identifiers the `blocks.json` files of the game's own resource packs give a block:
///   every `resource_packs/*/blocks.json` of the game, each read as the client reads it (a
///   legacy name such as `wool` split onto the blocks it stood for): 28 names. Seven are keys the files write
///   in mixed case (`concretePowder`, `invisibleBedrock`, `movingBlock`,
///   `pistonArmCollision`, `seaLantern`, `stickyPistonArmCollision`, `tripWire`), three are
///   blocks a split legacy name still lists (`deprecated_anvil`, the two
///   `deprecated_purpur_block`s), and one, `minecraft:end_gateway`, is no older name at all:
///   it is a block of the game that the generated list lacks (the game's own
///   `local_lighting.json` and language files name it), listed here until that list has it.
///
/// 30 names in all. Two of them, `minecraft:item.soul_campfire` (an entity file) and
/// `minecraft:tripwire` (`blocks.json`, lower-cased), are written by the game's files alone.
/// A block name is looked up as given, then lower-cased, so the lower-cased aliases count.
// `resource::block::tests::the_legacy_block_names_are_exactly_what_the_vanilla_files_need`
// holds the list against the game's files.
pub(crate) const LEGACY_BLOCK_NAMES: [&str; 30] = [
    "minecraft:concretepowder",
    "minecraft:double_plant",
    "minecraft:double_stone_slab",
    "minecraft:double_stone_slab2",
    "minecraft:double_stone_slab3",
    "minecraft:double_stone_slab4",
    "minecraft:double_wooden_slab",
    "minecraft:grass",
    "minecraft:invisiblebedrock",
    "minecraft:item.soul_campfire",
    "minecraft:leaves",
    "minecraft:leaves2",
    "minecraft:movingblock",
    "minecraft:pistonarmcollision",
    "minecraft:planks",
    "minecraft:red_flower",
    "minecraft:sapling",
    "minecraft:sealantern",
    "minecraft:shulker_box",
    "minecraft:stained_glass",
    "minecraft:stickypistonarmcollision",
    "minecraft:stone_slab",
    "minecraft:stone_slab2",
    "minecraft:stone_slab3",
    "minecraft:stone_slab4",
    "minecraft:stonebrick",
    "minecraft:tallgrass",
    "minecraft:tripwire",
    "minecraft:wooden_slab",
    "minecraft:yellow_flower",
];

/// Whether `name` (a block name in the canonical form of its kind) is one of
/// [`LEGACY_BLOCK_NAMES`].
pub(crate) fn is_legacy_block_name(name: &str) -> bool {
    listed(&LEGACY_BLOCK_NAMES, name)
}

/// Entities of the game that no file defines and the generated list of the game's entities
/// lacks: the fangs an evoker summons and the old name of the zombified piglin and of the
/// two skeleton variants. In the canonical form of the kind, sorted bytewise, compared as
/// written.
///
/// The list is exactly the entity names that the entity files of the game's behaviour packs
/// write as an entity and that resolve to nothing else. The identifiers of the game's patrol spawn rules
/// are no entities and are not here: see [`SPAWN_RULE_IDENTIFIERS`].
pub(crate) const FILELESS_ENTITIES: [&str; 4] = [
    "minecraft:evocation_fang",
    "minecraft:pig_zombie",
    "minecraft:skeleton.stray",
    "minecraft:skeleton.wither",
];

/// Whether `name` (an entity name in the canonical form of its kind, without an `<event>`
/// suffix) is one of [`FILELESS_ENTITIES`].
pub(crate) fn is_fileless_entity(name: &str) -> bool {
    listed(&FILELESS_ENTITIES, name)
}

/// Item names that the game's own loot tables write as an entry's `name` and that are no
/// names of the game: the generated list of the game's items does not have them, so the game
/// logs "is not a valid item name" for its own file. They are let through in that one
/// member, so that the game's files link as the game ships them, and
/// are told like any unknown name everywhere else. In the canonical form, compared as
/// written.
///
/// `minecraft:iron_pick`: `loot_tables/entities/raider_drops.json` of the `vanilla`
/// behaviour pack (the game's items have `iron_pickaxe` and `iron_pick_diggable` alone).
pub(crate) const LOOT_ENTRY_ONLY_ITEMS: [&str; 1] = ["minecraft:iron_pick"];

/// Whether `name` (canonical form) is one of [`LOOT_ENTRY_ONLY_ITEMS`].
pub(crate) fn is_loot_entry_only_item(name: &str) -> bool {
    listed(&LOOT_ENTRY_ONLY_ITEMS, name)
}

/// Item names that the game's own entity files write in an item member of a component and
/// that are no names of the game. They are let through in the entity rows alone and told
/// everywhere else. In the canonical form, compared as written.
///
/// `minecraft:nautilusarmor`: `equippable.slots[].item` of `entities/nautilus.json` and
/// `entities/zombie_nautilus.json` (the game's items are the `minecraft:*_nautilus_armor`
/// ones alone).
pub(crate) const ENTITY_FILE_ONLY_ITEMS: [&str; 1] = ["minecraft:nautilusarmor"];

/// Whether `name` (canonical form) is one of [`ENTITY_FILE_ONLY_ITEMS`].
pub(crate) fn is_entity_file_only_item(name: &str) -> bool {
    listed(&ENTITY_FILE_ONLY_ITEMS, name)
}

/// The identifiers the game's patrol spawn rules are for and are delayed by: names of no
/// actor, which the game's `spawn_rules/pillager_patrol.json` writes as the rules'
/// `description.identifier` and as the three `minecraft:delay_filter.identifier`s. They are
/// let through in those two members alone and told as an entity everywhere else, a rule's `permute_type` included. Sorted
/// bytewise, compared as written.
pub(crate) const SPAWN_RULE_IDENTIFIERS: [&str; 4] = [
    "minecraft:pillager_patrol",
    "minecraft:pillager_patrol_easy",
    "minecraft:pillager_patrol_hard",
    "minecraft:pillager_patrol_normal",
];

/// Whether `name` (canonical form) is one of [`SPAWN_RULE_IDENTIFIERS`].
pub(crate) fn is_spawn_rule_identifier(name: &str) -> bool {
    listed(&SPAWN_RULE_IDENTIFIERS, name)
}

/// The names of the game's particle types (`ParticleType`): what a legacy particle member
/// takes in place of the identifier of a particle effect. Lower-case, sorted bytewise.
///
/// The 98 names of the game's particle type enumeration
/// (`xtask/src/script_meta/game/particle_types.txt`), and the two names the game's own
/// entity files give `minecraft:explode.particle_effect` that the enumeration lacks
/// (`wind_burst`, `breeze_wind_burst`: `wind_charge_projectile.json` and
/// `breeze_wind_charge_projectile.json`; the explosion's own names).
pub(crate) const PARTICLE_TYPES: [&str; 100] = [
    "balloongas",
    "bleach",
    "blockforcefield",
    "blueflame",
    "breeze_wind_burst",
    "breezewindexplosion",
    "bubble",
    "bubblecolumndown",
    "bubblecolumnup",
    "bubblemanual",
    "campfiresmoke",
    "campfiresmoketall",
    "candleflame",
    "carrotboost",
    "coloredflame",
    "conduit",
    "creakingcrumble",
    "crit",
    "dragonbreath",
    "dragonbreathfire",
    "dragonbreathtrail",
    "dragondestroyblock",
    "driphoney",
    "driplava",
    "dripwater",
    "dustplume",
    "electricspark",
    "enchantingtable",
    "endrod",
    "evaporation",
    "explode",
    "eyeblossomclose",
    "eyeblossomopen",
    "fallingborderdust",
    "fallingdust",
    "fireworks",
    "fireworksoverlay",
    "fireworksstarter",
    "flame",
    "food",
    "greenflame",
    "heart",
    "hugeexplosion",
    "iceballbreak",
    "iconcrack",
    "ink",
    "largeexplode",
    "largesmoke",
    "lava",
    "mobappearance",
    "mobflame",
    "mobspell",
    "mobspellambient",
    "mobspellinstantaneous",
    "myceliumdust",
    "none",
    "note",
    "obsidiantear",
    "orangepoplarleaves",
    "paleoakleaves",
    "pausemobgrowth",
    "portal",
    "portalreverse",
    "rainsplash",
    "reddust",
    "redpoplarleaves",
    "resetmobgrowth",
    "risingborderdust",
    "sculksoul",
    "shriek",
    "shulkerbullet",
    "slime",
    "smoke",
    "sneeze",
    "snowballpoof",
    "snowflake",
    "sonicexplosion",
    "soul",
    "sparkler",
    "spit",
    "stalactitedriplava",
    "stalactitedripwater",
    "sulfurcube",
    "terrain",
    "totem",
    "townaura",
    "trackingemitter",
    "vaultconnection",
    "villagerangry",
    "villagerhappy",
    "watersplash",
    "watersplashmanual",
    "waterwake",
    "wax",
    "whitesmoke",
    "wind_burst",
    "windexplosion",
    "witchspell",
    "wolfarmorcrack",
    "yellowpoplarleaves",
];

/// Whether `name`, already lower-cased, is one of [`PARTICLE_TYPES`].
pub(crate) fn is_particle_type(name: &str) -> bool {
    listed(&PARTICLE_TYPES, name)
}

/// The item tags the game's own recipes take an ingredient by: tags the game's items carry,
/// which a world without the game's packs cannot see. Sorted bytewise, compared as written.
///
/// The list is exactly the `tag` of every ingredient of the recipes of the game's behaviour
/// packs (`behavior_packs/*/recipes`). It is not the tags of the game's items: a tag of the
/// game beyond these (`minecraft:is_food`) is known to a host alone.
// `content::tests::every_tag_of_the_vanilla_recipes_is_a_tag_of_the_game` holds
// the list against the game's files.
pub(crate) const VANILLA_ITEM_TAGS: [&str; 16] = [
    "minecraft:coals",
    "minecraft:egg",
    "minecraft:is_pickaxe",
    "minecraft:logs",
    "minecraft:logs_that_burn",
    "minecraft:metal_nuggets",
    "minecraft:mushrooms_for_stew",
    "minecraft:planks",
    "minecraft:soul_fire_base_blocks",
    "minecraft:stone_crafting_materials",
    "minecraft:stone_tool_materials",
    "minecraft:trim_materials",
    "minecraft:trim_templates",
    "minecraft:trimmable_armors",
    "minecraft:wooden_slabs",
    "minecraft:wool",
];

/// Whether `tag` (as written) is one of [`VANILLA_ITEM_TAGS`].
pub(crate) fn is_vanilla_item_tag(tag: &str) -> bool {
    listed(&VANILLA_ITEM_TAGS, tag)
}

/// The crafting tags of the game: those of its own stations, and `deprecated`, by which the
/// game's own packs retire a recipe (no station offers it). Sorted bytewise, compared as
/// written. They are the eleven values the game's own recipes write, and `material_reducer`.
pub(crate) const CRAFTING_STATION_TAGS: [&str; 12] = [
    "blast_furnace",
    "brewing_stand",
    "campfire",
    "cartography_table",
    "crafting_table",
    "deprecated",
    "furnace",
    "material_reducer",
    "smithing_table",
    "smoker",
    "soul_campfire",
    "stonecutter",
];

/// Whether `tag` (as written) is one of [`CRAFTING_STATION_TAGS`].
pub(crate) fn is_crafting_station_tag(tag: &str) -> bool {
    listed(&CRAFTING_STATION_TAGS, tag)
}

/// The world-generation features the game registers in code: no file of the game defines
/// them, and its own feature rules and features place them. Unlike the lists above this is
/// a list of the game's current names, as `bedrock_addon_core::vanilla::ids()` has for other
/// kinds: it answers without a host, and a host that answers for the kind `feature` is
/// believed instead. Sorted bytewise, compared as written.
///
/// It is **not** the features of the game: the ones its behaviour packs define in
/// `features/` and its app data in `definitions/features/` are known when those are on the
/// stack, or to a host.
///
/// The list is exactly the feature names that the game's own files reference and nothing
/// defines, each read as the pass reads it: every behaviour pack and the app data
/// (`definitions/features`, `definitions/feature_rules`) of the game linked as one world; the
/// names the feature rules and the feature references then miss (`places_feature` of a
/// feature rule, the features a feature or a template pool places). One name those files
/// reference is defined nowhere (`minecraft:sulfur_spring_trail_to_surface_system_feature`)
/// and is not listed: the game's own file names a feature that does not exist. Core's
/// `features` list is the game's structure features, not these.
pub(crate) const CODE_FEATURES: [&str; 87] = [
    "minecraft:_basalt_pillar_feature",
    "minecraft:_huge_crimson_fungus_feature",
    "minecraft:_huge_warped_fungus_feature",
    "minecraft:_twisting_vines_cluster_feature",
    "minecraft:_weeping_vines_cluster_feature",
    "minecraft:azalea_root_system_feature",
    "minecraft:bamboo_feature",
    "minecraft:basalt_columns_feature",
    "minecraft:brown_mushroom_feature",
    "minecraft:cactus_feature",
    "minecraft:clay_feature",
    "minecraft:coral_crust_feature",
    "minecraft:coral_feature",
    "minecraft:coral_hang_feature",
    "minecraft:dead_bush_feature",
    "minecraft:delta_feature",
    "minecraft:desert_well_feature",
    "minecraft:dripleaf_feature",
    "minecraft:dripstone_cluster_feature",
    "minecraft:eyeblossom_feature",
    "minecraft:firefly_bush_cluster_feature",
    "minecraft:firefly_bush_water_cluster_feature",
    "minecraft:grass_disc_feature",
    "minecraft:gravel_feature",
    "minecraft:huge_mushroom_feature",
    "minecraft:ice_patch_feature",
    "minecraft:ice_spike_feature",
    "minecraft:kelp_feature",
    "minecraft:large_dripstone_feature",
    "minecraft:leaf_litter_feature",
    "minecraft:legacy:bamboo_jungle_tree_feature",
    "minecraft:legacy:birch_forest_mutated_tree_feature",
    "minecraft:legacy:birch_forest_tree_feature",
    "minecraft:legacy:emerald_ore_feature",
    "minecraft:legacy:extreme_hills_plus_trees_tree_feature",
    "minecraft:legacy:flower_forest_flower_feature",
    "minecraft:legacy:flower_forest_foliage_feature",
    "minecraft:legacy:flower_forest_tree_feature",
    "minecraft:legacy:forest_flower_feature",
    "minecraft:legacy:forest_foliage_feature",
    "minecraft:legacy:forest_rock_feature",
    "minecraft:legacy:forest_tree_feature",
    "minecraft:legacy:ice_feature",
    "minecraft:legacy:ice_tree_feature",
    "minecraft:legacy:jungle_edge_tree_feature",
    "minecraft:legacy:jungle_tree_feature",
    "minecraft:legacy:meadow_trees_feature",
    "minecraft:legacy:mega_taiga_tree_feature",
    "minecraft:legacy:mesa_plateau_stone_tree_feature",
    "minecraft:legacy:overworld_flower_feature",
    "minecraft:legacy:plains_flower_feature",
    "minecraft:legacy:plains_tree_feature",
    "minecraft:legacy:redwood_taiga_mutated_tree_feature",
    "minecraft:legacy:savanna_mutated_tree_feature",
    "minecraft:legacy:savanna_tree_feature",
    "minecraft:legacy:small_mushrooms_feature",
    "minecraft:legacy:springs_feature",
    "minecraft:legacy:swamp_flower_feature",
    "minecraft:legacy:swamp_foliage_feature",
    "minecraft:legacy:taiga_tree_feature",
    "minecraft:meadow_flower_feature",
    "minecraft:melon_feature",
    "minecraft:moss_vegetation_feature",
    "minecraft:nether_fossil_feature",
    "minecraft:pale_moss_vegetation_feature",
    "minecraft:pickle_feature",
    "minecraft:pink_petals_feature",
    "minecraft:podzol_area_feature",
    "minecraft:pointed_dripstone_feature",
    "minecraft:pumpkin_feature",
    "minecraft:red_mushroom_feature",
    "minecraft:reeds_feature",
    "minecraft:sand_feature",
    "minecraft:sea_anemone_feature",
    "minecraft:seagrass_feature",
    "minecraft:shorter_seagrass_feature",
    "minecraft:shortest_seagrass_feature",
    "minecraft:sulfur_pool",
    "minecraft:sulfur_spike_cluster_feature",
    "minecraft:sulfur_spike_feature",
    "minecraft:sulfur_spring_trail_to_surface_feature",
    "minecraft:sweet_berry_bush_feature",
    "minecraft:taller_seagrass_feature",
    "minecraft:vines_feature",
    "minecraft:vines_single_face_feature",
    "minecraft:waterlily_feature",
    "minecraft:wildflowers_feature",
];

/// The languages of the game: the codes its own resource pack lists in
/// `texts/languages.json`, in the file's order (`en_US` first, the base language). The game's
/// packs are on every stack the game builds, so a `.lang` file of one of these codes is
/// loaded from every pack of a stack, whether or not a pack of the stack lists the code.
///
/// The list is exactly `resource_packs/vanilla/texts/languages.json` of the game's resource
/// packs.
// `tests::wiring::the_languages_of_the_game_are_the_vanilla_resource_packs_list` holds the
// list against the game's files.
pub(crate) const VANILLA_LANGUAGES: [&str; 29] = [
    "en_US", "en_GB", "de_DE", "es_ES", "es_MX", "fr_FR", "fr_CA", "it_IT", "ja_JP", "ko_KR",
    "pt_BR", "pt_PT", "ru_RU", "zh_CN", "zh_TW", "nl_NL", "bg_BG", "cs_CZ", "da_DK", "el_GR",
    "fi_FI", "hu_HU", "id_ID", "nb_NO", "pl_PL", "sk_SK", "sv_SE", "tr_TR", "uk_UA",
];

/// Whether the sorted `list` has `name`.
fn listed(list: &[&str], name: &str) -> bool {
    list.binary_search(&name).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each list is found by bisection: sorted bytewise, no name twice.
    #[test]
    fn every_list_is_sorted_and_unique() {
        let lists: [(&str, &[&str]); 8] = [
            ("code features", &CODE_FEATURES),
            ("items", &LEGACY_ITEM_NAMES),
            ("blocks", &LEGACY_BLOCK_NAMES),
            ("entities", &FILELESS_ENTITIES),
            ("spawn rule identifiers", &SPAWN_RULE_IDENTIFIERS),
            ("particle types", &PARTICLE_TYPES),
            ("item tags", &VANILLA_ITEM_TAGS),
            ("crafting tags", &CRAFTING_STATION_TAGS),
        ];
        for (name, list) in lists {
            assert!(list.windows(2).all(|pair| pair[0] < pair[1]), "{name}");
        }
    }

    /// The particle types are the generated `particle_types.txt` and the two explosion names,
    /// nothing else.
    #[test]
    fn the_particle_types_are_the_generated_list_and_the_two_explosion_names() {
        let mut expected: Vec<&str> =
            include_str!("../../../../../../xtask/src/script_meta/game/particle_types.txt")
                .lines()
                .chain(["breeze_wind_burst", "wind_burst"])
                .collect();
        expected.sort_unstable();
        assert_eq!(PARTICLE_TYPES.as_slice(), expected.as_slice());
    }

    /// The lists that are compared lower-cased hold lower-case names alone.
    #[test]
    fn the_folded_lists_are_in_lower_case() {
        for name in LEGACY_ITEM_NAMES
            .iter()
            .chain(&LEGACY_BLOCK_NAMES)
            .chain(&PARTICLE_TYPES)
        {
            assert_eq!(name.to_ascii_lowercase(), *name);
        }
    }

    #[test]
    fn a_legacy_item_name_is_compared_without_case_and_the_others_as_written() {
        assert!(is_legacy_item_name("minecraft:appleenchanted"));
        assert!(is_legacy_item_name("minecraft:appleEnchanted"));
        assert!(is_legacy_item_name("minecraft:carrotOnAStick"));
        assert!(!is_legacy_item_name("minecraft:apple"));
        assert!(!is_legacy_item_name("demo:appleenchanted"));
        assert!(is_legacy_block_name("minecraft:tallgrass"));
        assert!(!is_legacy_block_name("minecraft:Tallgrass"));
        assert!(!is_legacy_block_name("minecraft:stone"));
        assert!(!is_legacy_block_name("demo:sealantern"));
        assert!(is_fileless_entity("minecraft:evocation_fang"));
        assert!(!is_fileless_entity("minecraft:Evocation_Fang"));
        assert!(is_particle_type("largeexplode"));
        assert!(is_particle_type("wind_burst"));
        assert!(!is_particle_type("LargeExplode"));
        assert!(!is_particle_type("demo:spark"));
        assert!(is_vanilla_item_tag("minecraft:planks"));
        assert!(!is_vanilla_item_tag("minecraft:Planks"));
        assert!(is_crafting_station_tag("stonecutter"));
        assert!(!is_crafting_station_tag("minecraft:stonecutter"));
    }
}
