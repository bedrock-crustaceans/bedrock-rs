//! One module per game document family, each behind the feature of its name: the behaviour
//! pack's definitions, the resource pack's, and the few families the game reads from elsewhere.
//! A family imports the core vocabulary and the shared crates, never another family; what one
//! family needs to know of another's loaded definitions it asks the stack.

#[cfg(feature = "aim_assist")]
#[cfg_attr(docsrs, doc(cfg(feature = "aim_assist")))]
#[doc(inline)]
pub use bedrock_addon_family_aim_assist as aim_assist;
#[cfg(feature = "animation")]
#[cfg_attr(docsrs, doc(cfg(feature = "animation")))]
#[doc(inline)]
pub use bedrock_addon_family_animation as animation;
#[cfg(feature = "animation_controller")]
#[cfg_attr(docsrs, doc(cfg(feature = "animation_controller")))]
#[doc(inline)]
pub use bedrock_addon_family_animation_controller as animation_controller;
#[cfg(feature = "atomic_client_entity")]
#[cfg_attr(docsrs, doc(cfg(feature = "atomic_client_entity")))]
#[doc(inline)]
pub use bedrock_addon_family_atomic_client_entity as atomic_client_entity;
#[cfg(feature = "attachable")]
#[cfg_attr(docsrs, doc(cfg(feature = "attachable")))]
#[doc(inline)]
pub use bedrock_addon_family_attachable as attachable;
#[cfg(feature = "behavior_tree")]
#[cfg_attr(docsrs, doc(cfg(feature = "behavior_tree")))]
#[doc(inline)]
pub use bedrock_addon_family_behavior_tree as behavior_tree;
#[cfg(feature = "biome")]
#[cfg_attr(docsrs, doc(cfg(feature = "biome")))]
#[doc(inline)]
pub use bedrock_addon_family_biome as biome;
#[cfg(feature = "biomes_client")]
#[cfg_attr(docsrs, doc(cfg(feature = "biomes_client")))]
#[doc(inline)]
pub use bedrock_addon_family_biomes_client as biomes_client;
#[cfg(feature = "block")]
#[cfg_attr(docsrs, doc(cfg(feature = "block")))]
#[doc(inline)]
pub use bedrock_addon_family_block as block;
#[cfg(feature = "block_culling")]
#[cfg_attr(docsrs, doc(cfg(feature = "block_culling")))]
#[doc(inline)]
pub use bedrock_addon_family_block_culling as block_culling;
#[cfg(feature = "blocks_json")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocks_json")))]
#[doc(inline)]
pub use bedrock_addon_family_blocks_json as blocks_json;
#[cfg(feature = "camera")]
#[cfg_attr(docsrs, doc(cfg(feature = "camera")))]
#[doc(inline)]
pub use bedrock_addon_family_camera as camera;
#[cfg(feature = "client_biome")]
#[cfg_attr(docsrs, doc(cfg(feature = "client_biome")))]
#[doc(inline)]
pub use bedrock_addon_family_client_biome as client_biome;
#[cfg(feature = "client_entity")]
#[cfg_attr(docsrs, doc(cfg(feature = "client_entity")))]
#[doc(inline)]
pub use bedrock_addon_family_client_entity as client_entity;
#[cfg(feature = "crafting_catalog")]
#[cfg_attr(docsrs, doc(cfg(feature = "crafting_catalog")))]
#[doc(inline)]
pub use bedrock_addon_family_crafting_catalog as crafting_catalog;
#[cfg(feature = "credits")]
#[cfg_attr(docsrs, doc(cfg(feature = "credits")))]
#[doc(inline)]
pub use bedrock_addon_family_credits as credits;
#[cfg(feature = "ddui")]
#[cfg_attr(docsrs, doc(cfg(feature = "ddui")))]
#[doc(inline)]
pub use bedrock_addon_family_ddui as ddui;
#[cfg(feature = "deferred_settings")]
#[cfg_attr(docsrs, doc(cfg(feature = "deferred_settings")))]
#[doc(inline)]
pub use bedrock_addon_family_deferred_settings as deferred_settings;
#[cfg(feature = "dialogue")]
#[cfg_attr(docsrs, doc(cfg(feature = "dialogue")))]
#[doc(inline)]
pub use bedrock_addon_family_dialogue as dialogue;
#[cfg(feature = "dimension")]
#[cfg_attr(docsrs, doc(cfg(feature = "dimension")))]
#[doc(inline)]
pub use bedrock_addon_family_dimension as dimension;
#[cfg(feature = "entity")]
#[cfg_attr(docsrs, doc(cfg(feature = "entity")))]
#[doc(inline)]
pub use bedrock_addon_family_entity as entity;
#[cfg(feature = "feature")]
#[cfg_attr(docsrs, doc(cfg(feature = "feature")))]
#[doc(inline)]
pub use bedrock_addon_family_feature as feature;
#[cfg(feature = "feature_rules")]
#[cfg_attr(docsrs, doc(cfg(feature = "feature_rules")))]
#[doc(inline)]
pub use bedrock_addon_family_feature_rules as feature_rules;
#[cfg(feature = "flipbooks")]
#[cfg_attr(docsrs, doc(cfg(feature = "flipbooks")))]
#[doc(inline)]
pub use bedrock_addon_family_flipbooks as flipbooks;
#[cfg(feature = "fog")]
#[cfg_attr(docsrs, doc(cfg(feature = "fog")))]
#[doc(inline)]
pub use bedrock_addon_family_fog as fog;
#[cfg(feature = "font")]
#[cfg_attr(docsrs, doc(cfg(feature = "font")))]
#[doc(inline)]
pub use bedrock_addon_family_font as font;
#[cfg(feature = "function")]
#[cfg_attr(docsrs, doc(cfg(feature = "function")))]
#[doc(inline)]
pub use bedrock_addon_family_function as function;
#[cfg(feature = "geometry")]
#[cfg_attr(docsrs, doc(cfg(feature = "geometry")))]
#[doc(inline)]
pub use bedrock_addon_family_geometry as geometry;
#[cfg(feature = "item")]
#[cfg_attr(docsrs, doc(cfg(feature = "item")))]
#[doc(inline)]
pub use bedrock_addon_family_item as item;
#[cfg(feature = "jigsaw")]
#[cfg_attr(docsrs, doc(cfg(feature = "jigsaw")))]
#[doc(inline)]
pub use bedrock_addon_family_jigsaw as jigsaw;
#[cfg(feature = "json_ui")]
#[cfg_attr(docsrs, doc(cfg(feature = "json_ui")))]
#[doc(inline)]
pub use bedrock_addon_family_json_ui as json_ui;
#[cfg(feature = "lang")]
#[cfg_attr(docsrs, doc(cfg(feature = "lang")))]
#[doc(inline)]
pub use bedrock_addon_family_lang as lang;
#[cfg(feature = "languages")]
#[cfg_attr(docsrs, doc(cfg(feature = "languages")))]
#[doc(inline)]
pub use bedrock_addon_family_lang::languages;
#[cfg(feature = "legacy_path_mapping")]
#[cfg_attr(docsrs, doc(cfg(feature = "legacy_path_mapping")))]
#[doc(inline)]
pub use bedrock_addon_family_legacy_path_mapping as legacy_path_mapping;
#[cfg(feature = "loading_messages")]
#[cfg_attr(docsrs, doc(cfg(feature = "loading_messages")))]
#[doc(inline)]
pub use bedrock_addon_family_loading_messages as loading_messages;
#[cfg(feature = "loot")]
#[cfg_attr(docsrs, doc(cfg(feature = "loot")))]
#[doc(inline)]
pub use bedrock_addon_family_loot as loot;
#[cfg(feature = "material")]
#[cfg_attr(docsrs, doc(cfg(feature = "material")))]
#[doc(inline)]
pub use bedrock_addon_family_material as material;
#[cfg(feature = "music")]
#[cfg_attr(docsrs, doc(cfg(feature = "music")))]
#[doc(inline)]
pub use bedrock_addon_family_music as music;
#[cfg(feature = "particle")]
#[cfg_attr(docsrs, doc(cfg(feature = "particle")))]
#[doc(inline)]
pub use bedrock_addon_family_particle as particle;
#[cfg(feature = "persona")]
#[cfg_attr(docsrs, doc(cfg(feature = "persona")))]
#[doc(inline)]
pub use bedrock_addon_family_persona as persona;
#[cfg(feature = "poi")]
#[cfg_attr(docsrs, doc(cfg(feature = "poi")))]
#[doc(inline)]
pub use bedrock_addon_family_poi as poi;
#[cfg(feature = "recipe")]
#[cfg_attr(docsrs, doc(cfg(feature = "recipe")))]
#[doc(inline)]
pub use bedrock_addon_family_recipe as recipe;
#[cfg(feature = "render_controller")]
#[cfg_attr(docsrs, doc(cfg(feature = "render_controller")))]
#[doc(inline)]
pub use bedrock_addon_family_render_controller as render_controller;
#[cfg(feature = "script")]
#[cfg_attr(docsrs, doc(cfg(feature = "script")))]
#[doc(inline)]
pub use bedrock_addon_family_script as script;
#[cfg(feature = "server_sound_definitions")]
#[cfg_attr(docsrs, doc(cfg(feature = "server_sound_definitions")))]
#[doc(inline)]
pub use bedrock_addon_family_server_sound_definitions as server_sound_definitions;
#[cfg(feature = "skin")]
#[cfg_attr(docsrs, doc(cfg(feature = "skin")))]
#[doc(inline)]
pub use bedrock_addon_family_skin as skin;
#[cfg(feature = "sound_definitions")]
#[cfg_attr(docsrs, doc(cfg(feature = "sound_definitions")))]
#[doc(inline)]
pub use bedrock_addon_family_sound_definitions as sound_definitions;
#[cfg(feature = "sound_file")]
#[cfg_attr(docsrs, doc(cfg(feature = "sound_file")))]
#[doc(inline)]
pub use bedrock_addon_family_sound_file as sound_file;
#[cfg(feature = "sounds_json")]
#[cfg_attr(docsrs, doc(cfg(feature = "sounds_json")))]
#[doc(inline)]
pub use bedrock_addon_family_sounds_json as sounds_json;
#[cfg(feature = "spawn_groups")]
#[cfg_attr(docsrs, doc(cfg(feature = "spawn_groups")))]
#[doc(inline)]
pub use bedrock_addon_family_spawn_groups as spawn_groups;
#[cfg(feature = "spawn_rules")]
#[cfg_attr(docsrs, doc(cfg(feature = "spawn_rules")))]
#[doc(inline)]
pub use bedrock_addon_family_spawn_rules as spawn_rules;
#[cfg(feature = "splashes")]
#[cfg_attr(docsrs, doc(cfg(feature = "splashes")))]
#[doc(inline)]
pub use bedrock_addon_family_splashes as splashes;
#[cfg(feature = "structure")]
#[cfg_attr(docsrs, doc(cfg(feature = "structure")))]
#[doc(inline)]
pub use bedrock_addon_family_structure as structure;
#[cfg(feature = "texture_atlas")]
#[cfg_attr(docsrs, doc(cfg(feature = "texture_atlas")))]
#[doc(inline)]
pub use bedrock_addon_family_texture_atlas as texture_atlas;
#[cfg(feature = "texture_file")]
#[cfg_attr(docsrs, doc(cfg(feature = "texture_file")))]
#[doc(inline)]
pub use bedrock_addon_family_texture_file as texture_file;
#[cfg(feature = "texture_list")]
#[cfg_attr(docsrs, doc(cfg(feature = "texture_list")))]
#[doc(inline)]
pub use bedrock_addon_family_texture_list as texture_list;
#[cfg(feature = "texture_set")]
#[cfg_attr(docsrs, doc(cfg(feature = "texture_set")))]
#[doc(inline)]
pub use bedrock_addon_family_texture_set as texture_set;
#[cfg(feature = "trade")]
#[cfg_attr(docsrs, doc(cfg(feature = "trade")))]
#[doc(inline)]
pub use bedrock_addon_family_trade as trade;
#[cfg(feature = "ui_texture_info")]
#[cfg_attr(docsrs, doc(cfg(feature = "ui_texture_info")))]
#[doc(inline)]
pub use bedrock_addon_family_ui_texture_info as ui_texture_info;
#[cfg(feature = "voxel_shape")]
#[cfg_attr(docsrs, doc(cfg(feature = "voxel_shape")))]
#[doc(inline)]
pub use bedrock_addon_family_voxel_shape as voxel_shape;
