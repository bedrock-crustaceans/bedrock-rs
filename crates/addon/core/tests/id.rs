#![allow(clippy::pedantic, missing_docs)]
//! Typed identifiers as a user of `bedrock_addon_core::id` sees them: canonical-form equality
//! and the `as_str` round trip per `Syntax`, strict `new` against lenient `parse`, `from_static`,
//! `cast` inside one identifier space, the namespace helpers and the entity / item suffixes.

use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bedrock_addon_core::id::{
    AnimationRef, BiomeId, BlockId, EntityId, GeometryId, Id, IdError, ItemId, Kind, LootTablePath,
    MaterialId, RenderControllerId, SoundEventName, SpawnRulesId, StructureId, Syntax, TexturePath,
    kind,
};

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Equal values must hash equally; the canonical form decides both.
fn assert_same_key<K: Kind>(a: &Id<K>, b: &Id<K>) {
    assert_eq!(a, b, "{a} and {b} name the same thing");
    assert_eq!(hash_of(a), hash_of(b), "{a} and {b} hash alike");
}

mod namespaced {
    use super::*;

    #[test]
    fn a_bare_block_id_equals_its_minecraft_spelling() {
        let bare = BlockId::new("stone").unwrap();
        let full = BlockId::new("minecraft:stone").unwrap();
        assert_same_key(&bare, &full);
        assert_eq!(bare.as_str(), "stone", "as_str keeps what was written");
        assert_eq!(full.as_str(), "minecraft:stone");
    }

    #[test]
    fn namespace_and_path_split_at_the_colon() {
        let id = BlockId::new("demo:ruby_block").unwrap();
        assert_eq!(id.namespace(), Some("demo"));
        assert_eq!(id.path(), "ruby_block");
        assert!(!id.is_vanilla());
    }

    #[test]
    fn a_bare_id_has_no_namespace_but_is_vanilla_by_default() {
        let id = ItemId::new("apple").unwrap();
        assert_eq!(id.namespace(), None);
        assert_eq!(id.path(), "apple");
        assert!(id.is_vanilla(), "the minecraft namespace is defaulted");
        assert!(ItemId::new("minecraft:apple").unwrap().is_vanilla());
    }

    #[test]
    fn namespaced_ids_do_not_fold_case() {
        let lower = BlockId::new("demo:ruby").unwrap();
        let upper = BlockId::new("demo:Ruby").unwrap();
        assert_ne!(lower, upper);
        assert_ne!(
            BlockId::new("Minecraft:stone").unwrap(),
            BlockId::new("stone").unwrap()
        );
    }

    #[test]
    fn a_different_namespace_is_a_different_id() {
        assert_ne!(
            BiomeId::new("demo:plains").unwrap(),
            BiomeId::new("plains").unwrap()
        );
    }

    #[test]
    fn a_kind_without_the_default_requires_a_namespace() {
        assert!(matches!(
            <StructureId as Ctor>::new("house"),
            Err(IdError::MissingNamespace)
        ));
        let id = StructureId::new("mystructure:house").unwrap();
        assert_eq!(id.namespace(), Some("mystructure"));
        assert!(!id.is_vanilla());
    }

    #[test]
    fn a_lenient_bare_id_without_the_default_keeps_no_namespace() {
        let id = StructureId::parse("house");
        assert!(!id.is_well_formed());
        assert_eq!(id.namespace(), None);
        assert!(!id.is_vanilla());
        assert_ne!(id, StructureId::new("minecraft:house").unwrap());
    }

    #[test]
    fn the_syntax_of_the_seventeen_kinds() {
        assert_eq!(
            kind::Block::SYNTAX,
            Syntax::Namespaced {
                default_vanilla: true
            }
        );
        assert_eq!(
            kind::Structure::SYNTAX,
            Syntax::Namespaced {
                default_vanilla: false
            }
        );
        assert_eq!(
            kind::Geometry::SYNTAX,
            Syntax::Prefixed {
                prefix: "geometry.",
                fold_case: true
            }
        );
        assert_eq!(kind::Texture::SYNTAX, Syntax::Path);
        assert_eq!(kind::LootTable::SYNTAX, Syntax::Path);
        assert_eq!(kind::Material::SYNTAX, Syntax::Free);
        assert_eq!(kind::Animation::SYNTAX, Syntax::Free);
        assert_eq!(kind::Block::NAME, "block");
    }

    /// Lets one test call `new` through a trait object-free path for any alias.
    trait Ctor: Sized {
        fn new(text: &str) -> Result<Self, IdError>;
    }

    impl<K: Kind> Ctor for Id<K> {
        fn new(text: &str) -> Result<Self, IdError> {
            Id::new(text)
        }
    }
}

mod prefixed {
    use super::*;

    #[test]
    fn a_folding_prefixed_id_compares_without_ascii_case() {
        let written = GeometryId::new("geometry.Zombie.V2").unwrap();
        let lower = GeometryId::new("geometry.zombie.v2").unwrap();
        assert_same_key(&written, &lower);
        assert_eq!(
            written.as_str(),
            "geometry.Zombie.V2",
            "as_str keeps the case"
        );
        assert_eq!(written.namespace(), None);
        assert_eq!(written.path(), "geometry.Zombie.V2");
        assert!(!written.is_vanilla());
    }

    #[test]
    fn the_prefix_itself_folds_when_the_kind_folds() {
        let id = GeometryId::new("Geometry.zombie").unwrap();
        assert!(id.is_well_formed());
        assert_same_key(&id, &GeometryId::new("geometry.zombie").unwrap());
    }

    #[test]
    fn a_render_controller_needs_its_prefix() {
        assert!(RenderControllerId::new("controller.render.zombie").is_ok());
        assert!(matches!(
            RenderControllerId::new("controller.animation.zombie"),
            Err(IdError::WrongPrefix {
                expected: "controller.render."
            })
        ));
    }

    #[test]
    fn a_wrong_prefix_is_rejected() {
        assert!(matches!(
            GeometryId::new("animation.zombie"),
            Err(IdError::WrongPrefix {
                expected: "geometry."
            })
        ));
    }

    #[test]
    fn the_prefix_alone_is_empty() {
        assert!(matches!(GeometryId::new("geometry."), Err(IdError::Empty)));
    }

    /// No kind of the crate is prefixed without folding, so a test kind covers that mode.
    enum CaseSensitive {}

    impl Kind for CaseSensitive {
        const NAME: &'static str = "case-sensitive test kind";
        const SYNTAX: Syntax = Syntax::Prefixed {
            prefix: "controller.animation.",
            fold_case: false,
        };
        type Space = CaseSensitive;
    }

    type CaseSensitiveId = Id<CaseSensitive>;

    #[test]
    fn a_prefixed_kind_without_folding_compares_and_checks_its_prefix_exactly() {
        let lower = CaseSensitiveId::new("controller.animation.walk").unwrap();
        let upper = CaseSensitiveId::new("controller.animation.Walk").unwrap();
        assert_ne!(lower, upper);
        assert_same_key(
            &lower,
            &CaseSensitiveId::new("controller.animation.walk").unwrap(),
        );
        assert!(matches!(
            CaseSensitiveId::new("Controller.Animation.walk"),
            Err(IdError::WrongPrefix {
                expected: "controller.animation."
            })
        ));
        assert!(!CaseSensitiveId::parse("Controller.Animation.walk").is_well_formed());
        assert_ne!(
            CaseSensitiveId::parse("Controller.Animation.walk"),
            lower,
            "no folding of the prefix either"
        );
    }
}

mod path {
    use super::*;

    #[test]
    fn a_path_compares_as_written() {
        let a = TexturePath::new("textures/blocks/stone").unwrap();
        assert_eq!(a.as_str(), "textures/blocks/stone");
        assert_eq!(a.path(), "textures/blocks/stone");
        assert_eq!(a.namespace(), None);
        assert!(!a.is_vanilla());
        assert_ne!(a, TexturePath::new("textures/blocks/Stone").unwrap());
        assert_same_key(&a, &TexturePath::new("textures/blocks/stone").unwrap());
    }

    /// A path compares as `PackPath` normalises it (`\` and repeated separators).
    #[test]
    fn a_path_compares_with_its_separators_normalised() {
        let slash = TexturePath::new("textures/blocks/stone").unwrap();
        let backslash = TexturePath::new("textures\\blocks\\stone").unwrap();
        let doubled = TexturePath::new("/textures//blocks/stone/").unwrap();
        assert_same_key(&slash, &backslash);
        assert_same_key(&slash, &doubled);
        assert_eq!(
            backslash.as_str(),
            "textures\\blocks\\stone",
            "as_str keeps what was written"
        );
        assert_ne!(slash, TexturePath::new("textures/blocksstone").unwrap());
        assert_ne!(slash, TexturePath::new("textures/blocks/stone/x").unwrap());
    }

    #[test]
    fn a_loot_table_path_keeps_its_extension() {
        let path = LootTablePath::new("loot_tables/chests/simple_dungeon.json").unwrap();
        assert_eq!(path.as_str(), "loot_tables/chests/simple_dungeon.json");
    }

    #[test]
    fn a_dot_segment_is_rejected() {
        assert!(matches!(
            TexturePath::new("textures/../secret"),
            Err(IdError::InvalidChar { index: 9, ch: '.' })
        ));
        assert!(matches!(
            TexturePath::new("./textures/stone"),
            Err(IdError::InvalidChar { index: 0, ch: '.' })
        ));
    }

    #[test]
    fn a_path_longer_than_the_pack_path_limit_is_too_long() {
        let long = format!("textures/{}", "a".repeat(1100));
        assert!(matches!(TexturePath::new(long), Err(IdError::TooLong)));
    }

    #[test]
    fn a_control_character_in_a_path_is_rejected() {
        assert!(matches!(
            TexturePath::new("textures/a\u{0}b"),
            Err(IdError::InvalidChar {
                index: 10,
                ch: '\0'
            })
        ));
    }
}

mod free {
    use super::*;

    #[test]
    fn a_free_id_takes_any_non_empty_text() {
        let id = SoundEventName::new("Random.Click with space").unwrap();
        assert_eq!(id.as_str(), "Random.Click with space");
        assert_eq!(id.path(), "Random.Click with space");
        assert_eq!(id.namespace(), None);
        assert!(!id.is_vanilla());
        assert_ne!(id, SoundEventName::new("random.click with space").unwrap());
        assert!(MaterialId::new("entity_alphatest").is_ok());
        assert!(AnimationRef::new("controller.animation.zombie.move").is_ok());
    }

    #[test]
    fn a_free_id_with_a_colon_has_no_namespace() {
        let id = MaterialId::new("entity:entity_alphatest").unwrap();
        assert_eq!(id.namespace(), None);
        assert_eq!(id.path(), "entity:entity_alphatest");
    }
}

mod strict_and_lenient {
    use super::*;

    #[test]
    fn empty_text_is_rejected() {
        assert!(matches!(BlockId::new(""), Err(IdError::Empty)));
        assert!(matches!(SoundEventName::new(""), Err(IdError::Empty)));
        assert!(matches!(TexturePath::new(""), Err(IdError::Empty)));
    }

    #[test]
    fn an_empty_namespace_or_path_is_empty() {
        assert!(matches!(BlockId::new(":stone"), Err(IdError::Empty)));
        assert!(matches!(BlockId::new("demo:"), Err(IdError::Empty)));
    }

    #[test]
    fn whitespace_in_a_namespaced_id_is_an_invalid_char() {
        assert!(matches!(
            BlockId::new("demo:ruby block"),
            Err(IdError::InvalidChar { index: 9, ch: ' ' })
        ));
    }

    #[test]
    fn a_second_colon_in_a_block_id_is_an_invalid_char() {
        assert!(matches!(
            BlockId::new("a:b:c"),
            Err(IdError::InvalidChar { index: 3, ch: ':' })
        ));
    }

    #[test]
    fn an_angle_bracket_outside_an_entity_event_is_an_invalid_char() {
        assert!(matches!(
            BlockId::new("demo:a<b>"),
            Err(IdError::InvalidChar { index: 6, ch: '<' })
        ));
    }

    #[test]
    fn a_namespaced_id_longer_than_the_limit_is_too_long() {
        let long = format!("demo:{}", "a".repeat(70_000));
        assert!(matches!(BlockId::new(long), Err(IdError::TooLong)));
    }

    #[test]
    fn lenient_parse_keeps_every_malformed_text_as_written() {
        for text in ["", "demo:ruby block", "a:b:c", ":x", "minecraft:"] {
            let id = BlockId::parse(text);
            assert_eq!(id.as_str(), text);
            assert!(!id.is_well_formed(), "{text:?} is not well formed");
            assert!(BlockId::new(text).is_err());
        }
        let wrong = GeometryId::parse("animation.zombie");
        assert!(!wrong.is_well_formed());
        assert_eq!(wrong.as_str(), "animation.zombie");
    }

    #[test]
    fn lenient_parse_of_a_good_text_is_well_formed_and_equal_to_new() {
        let parsed = BlockId::parse("minecraft:stone");
        assert!(parsed.is_well_formed());
        assert_same_key(&parsed, &BlockId::new("stone").unwrap());
    }

    #[test]
    fn errors_render_their_rule() {
        assert_eq!(IdError::Empty.to_string(), "empty identifier");
        assert_eq!(
            IdError::InvalidChar { index: 3, ch: ' ' }.to_string(),
            "invalid character ' ' at byte 3"
        );
    }
}

mod constants {
    use super::*;

    const RUBY: BlockId = Id::from_static("demo:ruby_block");
    const ZOMBIE: GeometryId = Id::from_static("geometry.zombie");

    #[test]
    fn a_const_id_is_validated_at_compile_time() {
        assert_eq!(RUBY.as_str(), "demo:ruby_block");
        assert_eq!(RUBY.namespace(), Some("demo"));
        assert!(RUBY.is_well_formed());
        assert_same_key(&ZOMBIE, &GeometryId::new("GEOMETRY.ZOMBIE").unwrap());
    }

    #[test]
    fn a_static_id_equals_a_shared_one() {
        let shared = BlockId::new(String::from("demo:ruby_block")).unwrap();
        assert_same_key(&RUBY, &shared);
        let set: HashSet<BlockId> = [RUBY, shared].into_iter().collect();
        assert_eq!(set.len(), 1);
    }

    #[test]
    #[should_panic(expected = "identifier")]
    fn a_malformed_static_id_panics_outside_a_const() {
        let text: &'static str = "demo:bad block";
        let _ = BlockId::from_static(text);
    }
}

mod cast_and_format {
    use super::*;

    #[test]
    fn an_entity_id_casts_to_the_spawn_rules_key_of_the_same_space() {
        let entity = EntityId::new("demo:ghost").unwrap();
        let key: SpawnRulesId = entity.clone().cast();
        assert_eq!(key.as_str(), "demo:ghost");
        let back: EntityId = key.cast();
        assert_same_key(&back, &entity);
    }

    /// Kinds that share a space share the suffix rule, so a cast keeps the key.
    #[test]
    fn a_cast_entity_id_with_an_event_keys_the_spawn_rules_without_it() {
        let pig = EntityId::new("minecraft:pig<minecraft:entity_born>").unwrap();
        let key: SpawnRulesId = pig.cast();
        assert!(key.is_well_formed());
        assert_eq!(key.as_str(), "minecraft:pig<minecraft:entity_born>");
        assert_same_key(&key, &SpawnRulesId::new("minecraft:pig").unwrap());
        assert_eq!(kind::SpawnRules::SUFFIX, kind::Entity::SUFFIX);
    }

    #[test]
    fn display_is_the_text_as_written_and_debug_names_the_kind() {
        let id = BlockId::new("stone").unwrap();
        assert_eq!(id.to_string(), "stone");
        assert_eq!(format!("{id:?}"), "Id<block>(\"stone\")");
    }

    #[test]
    fn clones_are_equal() {
        let id = ItemId::new("demo:gem").unwrap();
        assert_same_key(&id, &id.clone());
    }
}

/// The keyed registries: one kind and alias each, with the syntax the family's folder uses.
mod kinds {
    use bedrock_addon_core::id::*;

    const VANILLA: Syntax = Syntax::Namespaced {
        default_vanilla: true,
    };
    const NAMESPACED: Syntax = Syntax::Namespaced {
        default_vanilla: false,
    };

    fn syntax<K: Kind>(_: Option<Id<K>>) -> (Syntax, &'static str) {
        (K::SYNTAX, K::NAME)
    }

    #[test]
    fn every_kind_has_its_syntax() {
        let table: &[((Syntax, &str), Syntax)] = &[
            (syntax::<kind::Block>(None::<BlockId>), VANILLA),
            (syntax(None::<ItemId>), VANILLA),
            (syntax(None::<EntityId>), VANILLA),
            (syntax(None::<SpawnRulesId>), VANILLA),
            (syntax(None::<BiomeId>), VANILLA),
            (
                syntax(None::<GeometryId>),
                Syntax::Prefixed {
                    prefix: "geometry.",
                    fold_case: true,
                },
            ),
            (syntax(None::<TexturePath>), Syntax::Path),
            (syntax(None::<MaterialId>), Syntax::Free),
            (syntax(None::<LootTablePath>), Syntax::Path),
            (syntax(None::<StructureId>), NAMESPACED),
            (syntax(None::<AnimationRef>), Syntax::Free),
            (syntax(None::<FeatureId>), NAMESPACED),
            (syntax(None::<FeatureRuleId>), NAMESPACED),
            (syntax(None::<TemplatePoolId>), NAMESPACED),
            (syntax(None::<ProcessorListId>), NAMESPACED),
            (syntax(None::<JigsawStructureId>), NAMESPACED),
            (syntax(None::<StructureSetId>), NAMESPACED),
            (syntax(None::<DimensionId>), NAMESPACED),
            (syntax(None::<TradeTablePath>), Syntax::Path),
            (syntax(None::<CameraPresetId>), NAMESPACED),
            (syntax(None::<CameraListId>), NAMESPACED),
            (syntax(None::<AimAssistPresetId>), NAMESPACED),
            (syntax(None::<AimAssistCategoryId>), NAMESPACED),
            (syntax(None::<ParticleId>), Syntax::Free),
            (syntax(None::<SoundEventName>), Syntax::Free),
            (syntax(None::<MusicDefinitionId>), Syntax::Free),
            (syntax(None::<FogId>), Syntax::Free),
            (syntax(None::<LightingSettingsId>), Syntax::Free),
            (syntax(None::<WaterSettingsId>), Syntax::Free),
            (syntax(None::<AtmosphereSettingsId>), Syntax::Free),
            (syntax(None::<ColorGradingSettingsId>), Syntax::Free),
            (syntax(None::<CubemapSettingsId>), Syntax::Free),
            (
                syntax(None::<RenderControllerId>),
                Syntax::Prefixed {
                    prefix: "controller.render.",
                    fold_case: true,
                },
            ),
            (syntax(None::<AtlasKey>), Syntax::Free),
            (syntax(None::<DialogueSceneTag>), Syntax::Free),
            (syntax(None::<FunctionPath>), Syntax::Path),
            (syntax(None::<LangKey>), Syntax::Free),
            (syntax(None::<ScriptModuleName>), Syntax::Free),
            (syntax(None::<ExtensionPointName>), Syntax::Free),
            (syntax(None::<VoxelShapeId>), Syntax::Free),
            (syntax(None::<RecipeId>), Syntax::Free),
            (syntax(None::<BehaviorTreeId>), Syntax::Free),
            (syntax(None::<PoiTypeId>), NAMESPACED),
            (syntax(None::<PoiTagId>), NAMESPACED),
            (syntax(None::<CullingRulesId>), NAMESPACED),
            (syntax(None::<AttachableId>), Syntax::Free),
            (syntax(None::<AtomicClientEntityId>), NAMESPACED),
            (syntax(None::<SpawnGroupId>), Syntax::Free),
            (syntax(None::<UiRootId>), NAMESPACED),
            (syntax(None::<UiCompositionId>), NAMESPACED),
            (syntax(None::<UiStyleId>), NAMESPACED),
            (syntax(None::<TrimPatternId>), Syntax::Free),
            (syntax(None::<TrimMaterialId>), Syntax::Free),
        ];
        let mut names = std::collections::HashSet::new();
        for ((actual, name), expected) in table {
            assert_eq!(actual, expected, "the syntax of {name}");
            assert!(names.insert(*name), "{name} names one kind");
        }
        assert_eq!(names.len(), 53);
    }

    #[test]
    fn only_the_entity_space_and_items_carry_a_suffix() {
        assert_eq!(kind::Entity::SUFFIX, Suffix::Event);
        assert_eq!(kind::SpawnRules::SUFFIX, Suffix::Event);
        assert_eq!(kind::Item::SUFFIX, Suffix::Aux);
        assert_eq!(kind::Block::SUFFIX, Suffix::None);
    }

    #[test]
    fn a_script_module_name_is_byte_exact() {
        let server = ScriptModuleName::new("@minecraft/server").unwrap();
        assert_ne!(server, ScriptModuleName::new("@Minecraft/server").unwrap());
    }

    #[test]
    fn a_function_path_follows_the_pack_path_rules() {
        assert!(FunctionPath::new("folder/name").is_ok());
        assert!(FunctionPath::new("../name").is_err());
    }
}

/// The suffixes: the entity event and the item aux value.
mod suffixes {
    use super::*;

    #[test]
    fn an_entity_event_is_split_off_and_ignored_by_the_comparison() {
        // A trailing `<event>` is split off the identifier and ignored by comparison.
        let pig = EntityId::new("minecraft:pig<minecraft:entity_born>").unwrap();
        assert_eq!(pig.event(), Some("minecraft:entity_born"));
        assert_eq!(pig.as_str(), "minecraft:pig<minecraft:entity_born>");
        assert_eq!(pig.namespace(), Some("minecraft"));
        assert_eq!(pig.path(), "pig");
        assert_same_key(&pig, &EntityId::new("pig").unwrap());
        assert_eq!(EntityId::new("demo:ghost").unwrap().event(), None);
    }

    #[test]
    fn a_bare_entity_with_a_namespaced_event_has_no_namespace() {
        let pig = EntityId::new("pig<minecraft:entity_born>").unwrap();
        assert_eq!(pig.namespace(), None);
        assert_eq!(pig.path(), "pig");
        assert!(pig.is_vanilla());
    }

    #[test]
    fn an_unclosed_or_empty_event_is_malformed() {
        assert!(matches!(
            EntityId::new("demo:pig<born"),
            Err(IdError::InvalidChar { index: 8, ch: '<' })
        ));
        assert!(matches!(
            EntityId::new("demo:pig<>"),
            Err(IdError::InvalidChar { index: 8, ch: '<' })
        ));
        let lenient = EntityId::parse("demo:pig<born");
        assert_eq!(lenient.event(), None);
        assert_same_key(&lenient, &EntityId::new("demo:pig").unwrap());
    }

    #[test]
    fn an_item_aux_value_is_split_off_and_ignored_by_the_comparison() {
        // A legacy `ns:name:aux` registers as `ns:name`
        let wool = ItemId::new("minecraft:wool:3").unwrap();
        assert_eq!(wool.aux(), Some(3));
        assert_eq!(wool.path(), "wool");
        assert_same_key(&wool, &ItemId::new("wool").unwrap());
        assert_eq!(ItemId::new("minecraft:wool").unwrap().aux(), None);
        assert_eq!(ItemId::new("demo:x:-1").unwrap().aux(), Some(-1));
        assert_eq!(ItemId::new("demo:x:-32768").unwrap().aux(), Some(i16::MIN));
    }

    #[test]
    fn a_bad_aux_value_is_malformed_but_still_split_off() {
        // An aux that is not a number.
        assert!(matches!(
            ItemId::new("custom:item3:not_an_aux_value"),
            Err(IdError::InvalidChar { index: 12, ch: ':' })
        ));
        assert!(ItemId::new("demo:x:32768").is_err());
        let bad = ItemId::parse("custom:item3:not_an_aux_value");
        assert_eq!(bad.aux(), None);
        assert_same_key(&bad, &ItemId::new("custom:item3").unwrap());
    }

    #[test]
    fn only_entities_take_an_event_and_only_items_an_aux() {
        assert!(BlockId::new("demo:pig<born>").is_err());
        assert!(EntityId::new("demo:pig:3").is_err());
    }
}

/// An identifier inside a document: read leniently from a JSON string, written as written.
mod in_a_document {
    use super::*;

    #[derive(Debug, facet::Facet)]
    struct Placer {
        block: BlockId,
        entity: Option<EntityId>,
    }

    #[test]
    fn a_json_string_reads_and_writes_back_as_written() {
        let text = r#"{"block":"stone","entity":"minecraft:pig<minecraft:entity_born>"}"#;
        let placer: Placer = facet_json::from_str(text).unwrap();
        assert_eq!(placer.block, BlockId::new("minecraft:stone").unwrap());
        assert_eq!(placer.block.as_str(), "stone");
        assert_eq!(
            placer.entity.as_ref().and_then(EntityId::event),
            Some("minecraft:entity_born")
        );
        assert_eq!(facet_json::to_string(&placer).unwrap(), text);
    }

    #[test]
    fn a_malformed_identifier_is_kept_not_rejected() {
        let placer: Placer = facet_json::from_str(r#"{"block":"demo:bad block"}"#).unwrap();
        assert_eq!(placer.block.as_str(), "demo:bad block");
        assert!(!placer.block.is_well_formed());
        assert!(placer.entity.is_none());
    }

    #[test]
    fn a_json_value_other_than_a_string_is_rejected() {
        for text in [r#"{"block":5}"#, r#"{"block":true}"#, r#"{"block":null}"#] {
            assert!(
                facet_json::from_str::<Placer>(text).is_err(),
                "{text} is no identifier"
            );
        }
    }
}
