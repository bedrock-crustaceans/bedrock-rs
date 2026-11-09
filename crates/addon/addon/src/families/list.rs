//! The one list of the crate's families, handed to a consumer macro (`with_families!(m)` expands
//! to `m! { … }`), which each consumer invokes in its own module so the list's names resolve
//! where it stands: the re-exports in `families`, the stack accessors in
//! `pack::stack::accessors`, the identifier lookups in `pack::link::lookup`.
//!
//! A family is one group: `"feature" module = crate_path { … } lookups { … }`, the feature
//! literal that gates it, the module name it has under `families`, and the path of the crate (or
//! module) that module is, three spellings because `macro_rules!` derives none of them from
//! another. The braces hold the family's accessor lines, in the grammar of `accessors!` without
//! the `feature = "…"` key, which the group's feature supplies; a family with no accessor has
//! empty braces. The optional `lookups` block holds the family's entries of the link pass's
//! lookup table, in the grammar of `lookups!` without the `#[cfg(…)]`, which the group's feature
//! supplies; an entry names its identifier kind, so a family can have several.
//!
//! The groups go in accessor order, not alphabetically: the order of the accessor lines is the
//! order of `ENTRIES`, which is the order a stack loads and reports in
//! (`entries_keep_the_accessor_order` pins it). The families with no accessor stand at the end.
//!
//! Adding a family, in this order: create its crate under `crates/addon/families/` (the
//! workspace glob picks it up); run `cargo xtask addon-features`, which writes its dependency
//! line and feature line into the generated blocks of this crate's `Cargo.toml` together (a
//! feature line written by hand before that names a `dep:` cargo cannot find and locks every
//! cargo command out, `cargo xtask` included); if the family's feature must pull in more than
//! its own crate, move its line into the hand block above the markers and rerun the generator;
//! then write its group here. Removing one is the mirror: delete its lines from the generated
//! blocks by hand first, then the crate, then rerun. The `accessors!` muncher behind the
//! accessor lines takes about two recursion levels per line and the 58 lines stand near the
//! compiler's default limit of 128 (`pack/stack/accessors.rs` says how near): a few more lines
//! bring back `#![recursion_limit]` on the crate.
//!
//! The glue names a few families by hand rather than through the list, and no `Family` hook
//! fits any of them: the trait is about reading one document of a version chain, and these are
//! facts of a pack, of the game's data or of two families together. Each stays where it is:
//!
//! - `script::starts_a_plugin` (`pack/stack/mod.rs`) sets `LoadContext::runs_scripts`, a fact
//!   of the pack known before any collection loads.
//! - `APP_ONLY_FILES` (`pack/stack/mod.rs`) names `credits/credits.json` among the game's own
//!   files a pack carries in vain: game knowledge that holds whatever families are compiled in.
//! - `pack/stack/texts.rs` joins `lang` and `languages`, and `pack/stack/textures.rs` joins
//!   `texture_file` and `texture_set`: glue of two families, which no family may write.
//! - The `persona::PieceSource` impl for `Pack` (`pack/open.rs`) stands with `Pack` by the
//!   orphan rule.
//! - `listed_languages` (`pack/stack/accessors.rs`), the `keys` of `translations`, reads the
//!   `languages` collection for the `lang` accessor.
//! - The `Special` caches of the link pass (`pack/link/lookup.rs`: loot tables, item tags,
//!   geometry parts, materials, unlisted languages) are lookups that are not one collection by
//!   identifier.
//! - The `structure` lookup's entry calls `structure::Structure::id_of` to name a file by the
//!   identifier `/structure load` gives it.
//! - `providers!` (`pack/stack/accessors.rs`) names the families whose definitions other
//!   collections ask about by kind (block, item, function): a second small hand list, until an
//!   accessor line can carry that flag.

/// Hands the family list to `$consumer`: `with_families!(m)` expands to
/// `m! { $( "feature" module = crate_path { … } $(lookups { … })? )* }`, one group per family in
/// accessor order.
macro_rules! with_families {
    ($consumer:ident) => {
        $consumer! {
            "block" block = bedrock_addon_family_block {
                /// The custom blocks of the behaviour packs (`blocks/**/*.json`), the higher pack's
                /// definition winning.
                blocks: Registry<block::Block> = block::LOADER, collection = Blocks, side = Behavior;
            } lookups {
                /// A block: a custom block of the behaviour stack, one of the game's, or a name of
                /// an older version the game still answers to
                /// ([`LEGACY_BLOCK_NAMES`](crate::vanilla::names::LEGACY_BLOCK_NAMES), with any
                /// host).
                block, block_names: Block [game] + is_legacy_block_name = |set| registered(set.behavior.blocks());
            }
            "item" item = bedrock_addon_family_item {
                /// The items of the behaviour packs (`items/`), by identifier, the higher pack's
                /// definition winning.
                items: Registry<item::Item> = item::LOADER, collection = Items, side = Behavior;
            } lookups {
                /// An item a behaviour pack defines or one of the game's, by the definitions alone:
                /// no block item and no legacy name (the rules ask [`LinkCx::item`]).
                item_definition, item_names: Item [game] = |set| registered(set.behavior.items());
            }
            "crafting_catalog" crafting_catalog = bedrock_addon_family_crafting_catalog {
                /// The crafting item catalogue of the behaviour stack
                /// (`item_catalog/crafting_item_catalog.json`): the topmost vanilla behaviour
                /// pack's file and every file above it, merged bottom-up into the four creative
                /// categories.
                crafting_catalog: Merged<crafting_catalog::CraftingCatalog> = crafting_catalog::LOADER, collection = CraftingCatalog, side = Behavior;
            }
            "entity" entity = bedrock_addon_family_entity {
                /// The entity definitions of the behaviour packs (`entities/`), by identifier, the
                /// higher pack's definition winning.
                entities: Registry<entity::Entity> = entity::LOADER, collection = Entities, side = Behavior;
            } lookups {
                /// An entity of the behaviour stack, one of the game's, or one of the game's that
                /// no file defines
                /// ([`FILELESS_ENTITIES`](crate::vanilla::names::FILELESS_ENTITIES), with any
                /// host); an `<event>` suffix is not part of the name.
                entity, entity_names: Entity [game] + is_fileless_entity = |set| registered(set.behavior.entities());
            }
            "biome" biome = bedrock_addon_family_biome {
                /// The server biomes of the behaviour packs (`biomes/`), by identifier, the higher
                /// pack's definition winning; beta documents are in its `beta()` registry.
                biomes: Registry<biome::Biome> = biome::LOADER, collection = Biomes, side = Behavior;
            } lookups {
                /// A biome of the behaviour stack or one of the game's.
                biome, biome_names: Biome [game] = |set| registered(set.behavior.biomes());
            }
            "client_biome" client_biome = bedrock_addon_family_client_biome {
                /// The client biomes of the resource packs (`biomes/**/*.json`), by biome: the
                /// highest pack's document claims its identifier; beta documents are in its
                /// `beta()` registry.
                client_biomes: Registry<client_biome::ClientBiome> = client_biome::LOADER, collection = ClientBiomes, side = Resource;
            }
            "biomes_client" biomes_client = bedrock_addon_family_biomes_client {
                /// The legacy client-biome settings (`biomes_client.json` of every resource pack
                /// that takes part), merged lowest pack first; base-game packs take no part from
                /// world base-game version 1.21.40.
                legacy_client_biomes: Merged<biomes_client::LegacyClientBiomes, RawLayer> = biomes_client::LOADER, collection = LegacyClientBiomes, side = Resource;
            }
            "spawn_rules" spawn_rules = bedrock_addon_family_spawn_rules {
                /// The spawn rules of the behaviour packs (`spawn_rules/**/*.json`), by entity, the
                /// higher pack's definition winning.
                spawn_rules: Registry<spawn_rules::SpawnRules> = spawn_rules::LOADER, collection = SpawnRules, side = Behavior;
            } lookups {
                /// The spawn rules of an entity.
                spawn_rules, spawn_rules_names: SpawnRules = |set| registered(set.behavior.spawn_rules());
            }
            "spawn_groups" spawn_groups = bedrock_addon_family_spawn_groups {
                /// The raid spawn groups of the app data (`definitions/spawn_groups/*.json`; never
                /// from a pack), by identifier.
                spawn_groups: Registry<spawn_groups::SpawnGroup> = spawn_groups::LOADER, collection = SpawnGroups, side = Behavior;
            }
            "feature" feature = bedrock_addon_family_feature {
                /// The features of the behaviour stack (`features/**/*.json` and the app's
                /// `definitions/features`), by identifier, the highest-priority definition winning.
                features: Registry<feature::Feature> = feature::LOADER, collection = Features, side = Behavior;
            } lookups {
                /// A world-generation feature: one of the behaviour stack (its packs' and the app
                /// data's), or one the game registers in code
                /// ([`CODE_FEATURES`](crate::vanilla::names::CODE_FEATURES)).
                feature, feature_names: Feature [game] = |set| registered(set.behavior.features());
            }
            "jigsaw" jigsaw = bedrock_addon_family_jigsaw {
                /// The jigsaw documents of the behaviour packs (`worldgen/{structures,
                /// template_pools, structure_sets, processors}/**/*.json` and
                /// `worldgen/metadata/*.dat`), by identifier (metadata by path), the higher pack's
                /// definition winning.
                jigsaw: jigsaw::Jigsaw = jigsaw::LOADER, collection = Jigsaw, side = Behavior;
            } lookups {
                /// A jigsaw template pool.
                template_pool, template_pool_names: TemplatePool = |set| registered(&set.behavior.jigsaw().pools);

                /// A jigsaw processor list.
                processor_list, processor_list_names: ProcessorList = |set| registered(&set.behavior.jigsaw().processors);

                /// A jigsaw structure of the stack or a structure feature of the game.
                jigsaw_structure, jigsaw_structure_names: JigsawStructure [game] = |set| registered(&set.behavior.jigsaw().structures);

                /// A jigsaw structure set.
                structure_set, structure_set_names: StructureSet = |set| registered(&set.behavior.jigsaw().structure_sets);
            }
            "recipe" recipe = bedrock_addon_family_recipe {
                /// The recipes of the behaviour packs (`recipes/**/*.json`): one registry per
                /// recipe namespace, the higher pack's valid recipe winning.
                recipes: recipe::Recipes = recipe::LOADER, collection = Recipes, side = Behavior;
            }
            "animation" animation = bedrock_addon_family_animation {
                /// The animations of the stack's packs (`animations/**/*.json`), by identifier
                /// (`animation.<name>`, lower-cased), the higher pack's animation winning.
                animations: Registry<animation::Animation> = animation::LOADER, collection = Animations, side = Both;
            } lookups {
                /// A behaviour-pack animation (`animation.…`).
                behavior_animation, behavior_animation_names: Animation = |set| registered(set.behavior.animations());

                /// A resource-pack animation (`animation.…`).
                client_animation, client_animation_names: Animation = |set| registered(set.resource.animations());
            }
            "animation_controller" animation_controller = bedrock_addon_family_animation_controller {
                /// The animation controllers of the stack's packs (`animation_controllers/*.json`),
                /// by identifier (`controller.animation.<name>`, lower-cased), the higher pack's
                /// controller winning.
                animation_controllers: Registry<animation_controller::AnimationController> = animation_controller::LOADER, collection = AnimationControllers, side = Both;
            } lookups {
                /// A behaviour-pack animation controller (`controller.animation.…`).
                behavior_animation_controller, behavior_animation_controller_names: Animation = |set| registered(set.behavior.animation_controllers());

                /// A resource-pack animation controller (`controller.animation.…`).
                client_animation_controller, client_animation_controller_names: Animation = |set| registered(set.resource.animation_controllers());
            }
            "feature_rules" feature_rules = bedrock_addon_family_feature_rules {
                /// The feature rules of the behaviour packs (`feature_rules/`) and of the app data,
                /// one registry per placement pass, the higher pack's rule winning per pass and
                /// identifier.
                feature_rules: feature_rules::FeatureRules = feature_rules::LOADER, collection = FeatureRules, side = Behavior;
            }
            "dimension" dimension = bedrock_addon_family_dimension {
                /// The dimension documents of the behaviour packs (`dimensions/*.json`), by
                /// identifier, the lowest pack's definition winning (vanilla keeps the first file
                /// it walks).
                dimensions: Registry<dimension::Dimension> = dimension::LOADER, collection = Dimensions, side = Behavior;
            } lookups {
                /// A dimension of the stack or one of the game's.
                dimension, dimension_names: Dimension [game] = |set| registered(set.behavior.dimensions());
            }
            "poi" poi = bedrock_addon_family_poi {
                /// The points of interest of the behaviour packs (`poi/**/*.json`, with the `poi`
                /// experiment): the POI types, the definition loaded last winning, and the tag
                /// documents, unioned per identifier.
                poi: poi::Poi = poi::LOADER, collection = Poi, side = Behavior;
            } lookups {
                /// A point-of-interest type.
                poi_type, poi_type_names: PoiType = |set| registered(&set.behavior.poi().types);

                /// A point-of-interest tag that has a tag document.
                poi_tag, poi_tag_names: PoiTag = |set| registered(&set.behavior.poi().tags);
            }
            "dialogue" dialogue = bedrock_addon_family_dialogue {
                /// The NPC dialogue scenes of the behaviour packs (`dialogue/**/*.json`), by scene
                /// tag, the lowest-priority pack's scene winning.
                dialogue: Registry<dialogue::DialogueScene> = dialogue::LOADER, collection = Dialogue, side = Behavior;
            }
            "trade" trade = bedrock_addon_family_trade {
                /// The trade tables of the behaviour packs (`trading/**/*.json`), by path, the top
                /// pack's copy winning.
                trade_tables: Files<trade::TradeTable> = trade::LOADER, collection = TradeTables, side = Behavior;
            } lookups {
                /// A trade table, by its path in the pack
                /// (`trading/economy_trades/farmer_trades.json`).
                trade_table, trade_table_names: TradeTable = |set| set.behavior.trade_tables().iter().map(|(path, document)| {
                    (Id::parse(path.as_str()), document.origin())
                });
            }
            "structure" structure = bedrock_addon_family_structure {
                /// The structure files of the behaviour packs (`structures/**/*.mcstructure`), by
                /// path (`Structure::find` looks one up by identifier); a copy that fails to load
                /// falls through to the next pack's.
                structures: Files<structure::Structure> = structure::LOADER, collection = Structures, side = Behavior;
            } lookups {
                /// A structure, as `/structure load` names it:
                /// `structures/<namespace>/<name>.mcstructure`, a file directly in `structures/`
                /// being `mystructure:<name>`.
                structure, structure_names: Structure = |set| set.behavior.structures().iter().filter_map(|(path, document)| {
                    Some((crate::families::structure::Structure::id_of(path)?, document.origin()))
                });
            }
            "loot" loot = bedrock_addon_family_loot {
                /// The loot tables of the behaviour packs (`loot_tables/**/*.json`), by path, the
                /// top pack's copy winning.
                loot_tables: Files<loot::LootTable> = loot::LOADER, collection = LootTables, side = Behavior;
            }
            "function" function = bedrock_addon_family_function {
                /// The functions of the behaviour packs (`functions/**/*.mcfunction`), by name, the
                /// higher pack's function winning.
                functions: Registry<function::Function> = function::LOADER, collection = Functions, side = Behavior;

                /// The functions that run every tick (`functions/tick.json` of every behaviour
                /// pack), in run order: the lowest-priority pack's entries first.
                tick_functions: Merged<function::TickList> = function::TICK_LOADER, collection = TickFunctions, side = Behavior;
            } lookups {
                /// A function, by its path under `functions/` without the extension.
                function, function_names: Function = |set| registered(set.behavior.functions());
            }
            "languages" languages = bedrock_addon_family_lang::languages {
                /// The languages of the stack's packs (`texts/languages.json`,
                /// `texts/language_names.json`): the union of the listed codes, lowest pack first,
                /// and the highest pack's name per code.
                languages: Merged<languages::Languages> = languages::LOADER, collection = Languages, side = Both;
            }
            "lang" lang = bedrock_addon_family_lang {
                /// The translations of one listed language (`texts/<code>.lang` of every pack), the
                /// highest pack's value winning per key; `None` for a code no `languages.json`
                /// lists.
                translations[language: lang::Language]: Merged<lang::Translations> = lang::LOADER, keys = listed_languages, key_of = lang::language_of, collection = Translations, side = Both;
            }
            "script" script = bedrock_addon_family_script {
                /// The script files of the stack's packs (`scripts/**/*.js`), one bundle per pack
                /// that has a script module (`script` in a behaviour pack, `client_script` in a
                /// resource pack) or any file below `scripts/`, the highest pack first. A bundle is
                /// its pack's alone: no import crosses packs.
                scripts: [script::ScriptBundle] = script::LOADER, collection = Scripts, side = Both;
            }
            "behavior_tree" behavior_tree = bedrock_addon_family_behavior_tree {
                /// The automation behavior trees of the behaviour packs
                /// (`behavior_trees/**/*.json`) and the nine trees built into the game, by
                /// `minecraft:name`: the higher pack's tree wins, and a built-in tree replaces a
                /// pack tree of its name.
                behavior_trees: Registry<behavior_tree::BehaviorTree> = behavior_tree::LOADER, collection = BehaviorTrees, side = Behavior;
            } lookups {
                /// A behaviour tree.
                behavior_tree, behavior_tree_names: BehaviorTree = |set| registered(set.behavior.behavior_trees());
            }
            "voxel_shape" voxel_shape = bedrock_addon_family_voxel_shape {
                /// The voxel shapes of the behaviour packs (`shapes/**/*.json`), by identifier, the
                /// higher pack's shape winning (vanilla binds an identifier to the shape it loads
                /// last).
                voxel_shapes: Registry<voxel_shape::VoxelShape> = voxel_shape::LOADER, collection = VoxelShapes, side = Behavior;
            } lookups {
                /// A voxel shape of the behaviour stack.
                voxel_shape, voxel_shape_names: VoxelShape = |set| registered(set.behavior.voxel_shapes());
            }
            "geometry" geometry = bedrock_addon_family_geometry {
                /// The geometries of the resource packs (`models/mobs.json`,
                /// `models/entity/**/*.json`, `models/blocks/**/*.json`), by identifier
                /// (`geometry.<name>`, compared without case), the higher pack's geometry winning.
                geometries: Registry<geometry::Geometry> = geometry::LOADER, collection = Geometries, side = Resource;
            } lookups {
                /// A geometry of the resource stack. Compared without ASCII case, as the identifier
                /// kind compares: a reference that differs from the definition only in case
                /// resolves.
                geometry, geometry_names: Geometry = |set| registered(set.resource.geometries());
            }
            "camera" camera = bedrock_addon_family_camera {
                /// The camera documents of the behaviour packs (`cameras/{presets, lists,
                /// splines}/`): presets and lists by identifier, the higher pack's winning, and
                /// every spline file.
                cameras: camera::Cameras = camera::LOADER, collection = Cameras, side = Behavior;
            } lookups {
                /// A camera preset of the stack or one of the game's.
                camera_preset, camera_preset_names: CameraPreset [game] = |set| registered(&set.behavior.cameras().presets);
            }
            "aim_assist" aim_assist = bedrock_addon_family_aim_assist {
                /// The aim-assist documents of the behaviour packs (`aim_assist/{presets,
                /// categories}/`): presets by identifier and categories by name, the higher pack's
                /// winning.
                aim_assist: aim_assist::AimAssist = aim_assist::LOADER, collection = AimAssist, side = Behavior;
            } lookups {
                /// An aim-assist preset.
                aim_assist_preset, aim_assist_preset_names: AimAssistPreset = |set| registered(&set.behavior.aim_assist().presets);

                /// An aim-assist category.
                aim_assist_category, aim_assist_category_names: AimAssistCategory = |set| registered(&set.behavior.aim_assist().categories);
            }
            "sound_definitions" sound_definitions = bedrock_addon_family_sound_definitions {
                /// The sound definitions of the resource packs (`sounds/sound_definitions.json`):
                /// the sound events and the audio each plays, merged across the stack as JSON.
                sound_definitions: Merged<sound_definitions::SoundDefinitions, RawLayer> = sound_definitions::LOADER, collection = SoundDefinitions, side = Resource;
            } lookups {
                /// A sound event of the merged `sound_definitions.json`, compared as written.
                sound_definition, sound_definition_names: SoundEvent = |set| {
                    let definitions = set.resource.sound_definitions();
                    definitions.value().sound_definitions.keys().filter_map(move |name| Some((name.clone(), definitions.origin(name)?)))
                };
            }
            "sounds_json" sounds_json = bedrock_addon_family_sounds_json {
                /// The event bindings of the resource packs (`sounds.json`): which sound event each
                /// game event plays, merged across the stack as JSON.
                sounds_json: Merged<sounds_json::SoundBindings, RawLayer> = sounds_json::LOADER, collection = SoundsJson, side = Resource;
            }
            "music" music = bedrock_addon_family_music {
                /// The music of the resource packs (`sounds/music_definitions.json`): the music of
                /// each situation, merged across the stack as JSON.
                music: Merged<music::MusicDefinitions, RawLayer> = music::LOADER, collection = Music, side = Resource;
            } lookups {
                /// A music definition of the merged `music_definitions.json`.
                music_definition, music_definition_names: MusicDefinition = |set| {
                    let music = set.resource.music();
                    music.value().definitions.keys().filter_map(move |name| Some((name.clone(), music.origin(name)?)))
                };
            }
            "sound_file" sound_file = bedrock_addon_family_sound_file {
                /// The audio files under `sounds/` of the resource packs, as sound names resolve to
                /// them.
                sound_files: sound_file::SoundFiles = sound_file::LOADER, collection = SoundFiles, side = Resource;
            }
            "server_sound_definitions" server_sound_definitions = bedrock_addon_family_server_sound_definitions {
                /// The server's definitions of the sound events the behaviour packs declare
                /// (`sounds/sound_definitions.json`, beta), by sound event name; the lowest pack's
                /// definition of a name is the one kept.
                server_sound_definitions: Registry<server_sound_definitions::SoundMetadata> = server_sound_definitions::LOADER, collection = ServerSoundDefinitions, side = Behavior;
            }
            "deferred_settings" deferred_settings = bedrock_addon_family_deferred_settings {
                /// The deferred renderer's settings of the resource packs (`lighting/`,
                /// `atmospherics/`, `water/`, `color_grading/`, `cubemaps/`, `shadows/global.json`,
                /// `pbr/global.json`, `local_lighting/local_lighting.json`): the identifier maps,
                /// the two single files and the merged local lights, the higher pack's winning.
                deferred_settings: deferred_settings::DeferredSettings = deferred_settings::LOADER, collection = DeferredSettings, side = Resource;
            } lookups {
                /// Deferred lighting settings.
                lighting_settings, lighting_settings_names: LightingSettings = |set| registered(&set.resource.deferred_settings().lighting);

                /// Deferred water settings.
                water_settings, water_settings_names: WaterSettings = |set| registered(&set.resource.deferred_settings().water);

                /// Deferred atmosphere settings.
                atmosphere_settings, atmosphere_settings_names: AtmosphereSettings = |set| registered(&set.resource.deferred_settings().atmosphere);

                /// Deferred colour-grading settings.
                color_grading_settings, color_grading_settings_names: ColorGradingSettings = |set| registered(&set.resource.deferred_settings().color_grading);

                /// Deferred cubemap settings.
                cubemap_settings, cubemap_settings_names: CubemapSettings = |set| registered(&set.resource.deferred_settings().cubemaps);
            }
            "json_ui" json_ui = bedrock_addon_family_json_ui {
                /// The JSON UI of the resource packs: the files every pack's `ui/_ui_defs.json`
                /// lists, each merged from the lowest pack up, the definitions they resolve to, and
                /// `ui/_global_variables.json`.
                ui: Merged<json_ui::UiDefinitions, RawLayer> = json_ui::LOADER, collection = Ui, side = Resource;
            }
            "blocks_json" blocks_json = bedrock_addon_family_blocks_json {
                /// What the client draws and plays for each block (`blocks.json` of every resource
                /// pack), by block: the packs' entries merged field by field, a higher pack's field
                /// replacing a lower pack's. A bare key is a vanilla block's name (`stone` is
                /// `minecraft:stone`).
                client_blocks: Registry<blocks_json::ClientBlock> = blocks_json::LOADER, collection = ClientBlocks, side = Resource;
            } lookups {
                /// The `blocks.json` entry of a block, on the resource stack.
                client_block, client_block_names: Block = |set| registered(set.resource.client_blocks());
            }
            "block_culling" block_culling = bedrock_addon_family_block_culling {
                /// The culling rules of the resource packs (`block_culling/**/*.json`), by
                /// identifier, the higher pack's document winning.
                block_culling: Registry<block_culling::CullingRules> = block_culling::LOADER, collection = BlockCulling, side = Resource;
            } lookups {
                /// The culling rules of a block geometry, on the resource stack: compared as
                /// written.
                culling_rules, culling_rules_names: CullingRules = |set| registered(set.resource.block_culling());
            }
            "ddui" ddui = bedrock_addon_family_ddui {
                /// The data-driven UI of the resource packs (`ddui/{root, composition}/`): roots
                /// and compositions by identifier, the lowest pack's definition of an identifier
                /// winning.
                data_driven_ui: ddui::DataDrivenUi = ddui::LOADER, collection = DataDrivenUi, side = Resource;
            }
            "texture_atlas" texture_atlas = bedrock_addon_family_texture_atlas {
                /// The terrain and item atlases of the resource packs
                /// (`textures/terrain_texture.json` and `textures/item_texture.json`), each merged
                /// across the stack.
                texture_atlases: texture_atlas::TextureAtlases = texture_atlas::LOADER, collection = TextureAtlases, side = Resource;
            } lookups {
                /// A tile of the terrain atlas (`textures/terrain_texture.json`), compared as
                /// written; the near miss of a missing key is the key of the same atlas that
                /// differs only in case.
                terrain_atlas_key, terrain_atlas_names: AtlasTexture = |set| {
                    let atlas = &set.resource.texture_atlases().terrain_atlas;
                    atlas.value().texture_data.iter().flatten().filter_map(move |(key, _)| Some((key.clone(), atlas.origin(key)?)))
                };

                /// A tile of the item atlas (`textures/item_texture.json`), compared as written;
                /// the near miss of a missing key is the key of the same atlas that differs only in
                /// case.
                item_atlas_key, item_atlas_names: AtlasTexture = |set| {
                    let atlas = &set.resource.texture_atlases().item_atlas;
                    atlas.value().texture_data.iter().flatten().filter_map(move |(key, _)| Some((key.clone(), atlas.origin(key)?)))
                };
            }
            "flipbooks" flipbooks = bedrock_addon_family_flipbooks {
                /// The flipbooks of the resource packs (`textures/flipbook_textures.json`), merged
                /// entry by entry from the lowest pack up.
                flipbooks: Merged<flipbooks::Flipbooks, RawLayer> = flipbooks::LOADER, collection = Flipbooks, side = Resource;
            }
            "texture_list" texture_list = bedrock_addon_family_texture_list {
                /// The texture lists of the resource packs (`textures/textures_list.json`): each
                /// pack's list, what the client does with it and the paths the packs claim for
                /// preloading.
                texture_lists: texture_list::TextureLists = texture_list::LOADER, collection = TextureLists, side = Resource;
            }
            "texture_set" texture_set = bedrock_addon_family_texture_set {
                /// The texture sets of the resource packs (`*.texture_set.json`), by the texture
                /// each sits beside.
                texture_sets: texture_set::TextureSets = texture_set::LOADER, collection = TextureSets, side = Resource;
            }
            "texture_file" texture_file = bedrock_addon_family_texture_file {
                /// The texture files of the resource packs: which pack has which image or texture
                /// set, as the packs list them.
                texture_files: texture_file::TextureFiles = texture_file::LOADER, collection = TextureFiles, side = Resource;
            }
            "splashes" splashes = bedrock_addon_family_splashes {
                /// The splash texts of the resource packs (`splashes.json` of every pack, lowest
                /// first): the pool the full game picks from, a copy without `canMerge: true`
                /// replacing the lower packs' splashes.
                splashes: Merged<splashes::Splashes> = splashes::LOADER, collection = Splashes, side = Resource;
            }
            "loading_messages" loading_messages = bedrock_addon_family_loading_messages {
                /// The loading-screen tips of the resource packs (`loading_messages.json`): the
                /// highest pack's file, whole.
                loading_messages: Merged<loading_messages::LoadingMessages> = loading_messages::LOADER, collection = LoadingMessages, side = Resource;
            }
            "credits" credits = bedrock_addon_family_credits {
                /// The credits: the game's own `credits/credits.json` when the app data holds it,
                /// else none. No pack's copy is read; a pack that carries one, another file only
                /// the game's own data supplies, or a `persona/` folder is named in the stack's
                /// report.
                credits: Merged<credits::Credits> = credits::LOADER, collection = Credits, side = Resource;
            }
            "font" font = bedrock_addon_family_font {
                /// The fonts of the resource packs: `font/font_metadata.json` and
                /// `font/emoticons.json`, each the highest pack's file, whole, and the pack that
                /// supplies each font file.
                fonts: font::Fonts = font::LOADER, collection = Fonts, side = Resource;
            }
            "ui_texture_info" ui_texture_info = bedrock_addon_family_ui_texture_info {
                /// The side files of the UI images of the resource packs (`textures/ui/**/*.json`:
                /// a nine-slice description or an Aseprite sheet), by path: the highest pack's
                /// copy, whichever pack supplies the image.
                ui_texture_infos: Files<ui_texture_info::UiTextureInfo> = ui_texture_info::LOADER, collection = UiTextureInfos, side = Resource;
            }
            "render_controller" render_controller = bedrock_addon_family_render_controller {
                /// The render controllers of the resource packs (`render_controllers/**/*.json`) by
                /// name, the highest pack's winning; a controller that failed leaves the lower one
                /// in place.
                render_controllers: Registry<render_controller::RenderController> = render_controller::LOADER, collection = RenderControllers, side = Resource;
            } lookups {
                /// A render controller of the resource stack. Compared without ASCII case, as the
                /// identifier kind compares; a controller the game registered under a name its
                /// lower-cased references never reach (`CaseFoldingDisabled`) resolves here all the
                /// same.
                render_controller, render_controller_names: RenderController = |set| registered(set.resource.render_controllers());
            }
            "fog" fog = bedrock_addon_family_fog {
                /// The fog settings of the resource packs (`fogs/*.json`) by identifier, the
                /// highest pack's winning.
                fogs: Registry<fog::FogSettings> = fog::LOADER, collection = Fogs, side = Resource;
            } lookups {
                /// Fog settings.
                fog, fog_names: Fog = |set| registered(set.resource.fogs());
            }
            "client_entity" client_entity = bedrock_addon_family_client_entity {
                /// The client entities of the resource packs (`entity/**/*.json`), by identifier in
                /// lower case: across packs the definition with the highest
                /// `description.min_engine_version` wins, and definitions with the same one are
                /// merged, the higher pack's members last.
                client_entities: Registry<client_entity::ClientEntity> = client_entity::LOADER, collection = ClientEntities, side = Resource;
            } lookups {
                /// The client entity of an entity, on the resource stack.
                client_entity, client_entity_names: Entity [game] = |set| registered(set.resource.client_entities());
            }
            "atomic_client_entity" atomic_client_entity = bedrock_addon_family_atomic_client_entity {
                /// The atomic client entities of the resource packs
                /// (`atomic_client_entities/**/*.json`), by their own identifiers, the higher
                /// pack's definition replacing the lower one whole.
                atomic_client_entities: Registry<atomic_client_entity::AtomicClientEntity> = atomic_client_entity::LOADER, collection = AtomicClientEntities, side = Resource;
            }
            "attachable" attachable = bedrock_addon_family_attachable {
                /// The attachables of the resource packs (`attachables/**/*.json`), by identifier
                /// in lower case: across packs the definition with the highest
                /// `description.min_engine_version` wins, and definitions with the same one are
                /// merged, the higher pack's members last.
                attachables: Registry<attachable::Attachable> = attachable::LOADER, collection = Attachables, side = Resource;
            }
            "material" material = bedrock_addon_family_material {
                /// The materials of the resource packs: the list files (`materials/common.json`,
                /// `fancy.json`, `sad.json`), every `.material` file merged over the stack, and the
                /// names each list creates.
                materials: material::Materials = material::LOADER, collection = Materials, side = Resource;
            }
            "particle" particle = bedrock_addon_family_particle {
                /// The particle effects of the resource packs (`particles/**/*.json`), by
                /// identifier, the higher pack's effect winning (vanilla binds an identifier to the
                /// effect it loads last); a file that fails to load leaves a lower pack's effect in
                /// place.
                particles: Registry<particle::ParticleEffect> = particle::LOADER, collection = Particles, side = Resource;
            } lookups {
                /// A particle effect of the resource stack: read from `rp.particles()` alone (the
                /// pass needs no provider line for it).
                particle, particle_names: Particle = |set| registered(set.resource.particles());
            }
            "legacy_path_mapping" legacy_path_mapping = bedrock_addon_family_legacy_path_mapping {}
            "persona" persona = bedrock_addon_family_persona {}
            "server_config" server_config = bedrock_addon_family_server_config {}
            "skin" skin = bedrock_addon_family_skin {}
        }
    };
}
pub(crate) use with_families;
