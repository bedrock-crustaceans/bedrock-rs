// The documents of the legacy probe that no test or fixture has: one per member a
// step or legacy reader reads with no op that carries a value, and one per starting version
// of a chain that no test or fixture has a document of. Included by
// `writable_legacy.rs`.

/// More legacy documents of a family, each with its version.
#[expect(
    clippy::too_many_lines,
    reason = "one match over the table of documents of each family: no seam to split at"
)]
fn more(family: &str) -> &'static [(&'static str, &'static str)] {
    match family {
        "item" => &[
            // A legacy member beside the member a step writes it to.
            (
                "1.20.40",
                r#"{"format_version": "1.20.40", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:use_duration": 3, "minecraft:use_modifiers": {"use_duration": 9, "movement_modifier": 0.5}}}}"#,
            ),
            (
                "1.20.40",
                r#"{"format_version": "1.20.40", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:use_duration": 3}}}"#,
            ),
            (
                "1.21.0",
                r#"{"format_version": "1.21.0", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:use_duration": {"value": 3}}}}"#,
            ),
            (
                "1.20.40",
                r#"{"format_version": "1.20.40", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"tag:a": {}, "minecraft:tags": {"tags": ["b"]}}}}"#,
            ),
            (
                "1.20.40",
                r#"{"format_version": "1.20.40", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:tags": {"tags": ["b"]}, "tag:a": {}}}}"#,
            ),
            (
                "1.16.100",
                r#"{"format_version": "1.16.100", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:creative_category": {"parent": "itemGroup.name.sword"}, "minecraft:icon": {"texture": "a"}}}}"#,
            ),
            (
                "1.20.30",
                r#"{"format_version": "1.20.30", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:on_use": {"on_use": {"event": "e"}}, "minecraft:weapon": {"on_hurt_entity": {"event": "e"}}, "minecraft:icon": {"texture": "a"}}}}"#,
            ),
            (
                "1.20.0",
                r#"{"format_version": "1.20.0", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:food": {"nutrition": 4, "saturation_modifier": 0.6}}}}"#,
            ),
            (
                "1.16.100",
                r#"{"format_version": "1.16.100", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:durability": {"max_durability": 10, "damage_chance": {"min": 10, "max": 50}}}}}"#,
            ),
            (
                "1.10.0",
                r#"{"format_version": "1.10.0", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:block": "minecraft:stone", "minecraft:frame_count": 3, "minecraft:mining_speed": 2, "minecraft:foil": true, "minecraft:max_damage": 10}}}"#,
            ),
            // One document per starting version of the chain that no test or fixture has
            // (the tests of those versions build their documents with a helper, not a literal).
            (
                "1.20.80",
                r#"{"format_version": "1.20.80", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 16}}}"#,
            ),
            (
                "1.21.10",
                r#"{"format_version": "1.21.10", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 16}}}"#,
            ),
            (
                "1.21.30",
                r#"{"format_version": "1.21.30", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 16}}}"#,
            ),
            (
                "1.21.40",
                r#"{"format_version": "1.21.40", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 16}}}"#,
            ),
            (
                "1.21.80",
                r#"{"format_version": "1.21.80", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 16}}}"#,
            ),
        ],
        "entity" => &[
            // A legacy member beside the member a step writes it to.
            (
                "1.16.0",
                r#"{"format_version": "1.16.0", "minecraft:entity": {"description": {"identifier": "demo:x", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:behavior.drop_item_for": {"priority": 1, "max_dist": 3, "target_range": [9, 9, 9]}}}}"#,
            ),
            (
                "1.16.0",
                r#"{"format_version": "1.16.0", "minecraft:entity": {"description": {"identifier": "demo:x", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:behavior.explore_outskirts": {"priority": 1, "wait_time": 600, "max_wait_time": 99}}}}"#,
            ),
            (
                "1.12.0",
                r#"{"format_version": "1.12.0", "minecraft:entity": {"description": {"identifier": "demo:x", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:damage_sensor": {"triggers": [{"cause": "fire", "filters": {"test": "has_tag", "value": "a"}, "event": "x:e", "on_damage": {"event": "x:mine"}}]}}}}"#,
            ),
            (
                "1.12.0",
                r#"{"format_version": "1.12.0", "minecraft:entity": {"description": {"identifier": "demo:x", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:damage_sensor": {"cause": "fire", "filters": {"test": "has_tag", "value": "a"}, "event": "x:e", "on_damage": {"event": "x:mine"}}}}}"#,
            ),
            (
                "1.21.120",
                r#"{"format_version": "1.21.120", "minecraft:entity": {"description": {"identifier": "demo:x", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:breedable": {"breeds_with": [{"mate_type": "demo:a", "baby_type": "demo:b", "breed_event": {"event": "x:first"}}, {"mate_type": "demo:a", "baby_type": "demo:c"}]}}}}"#,
            ),
            (
                "1.21.120",
                r#"{"format_version": "1.21.120", "minecraft:entity": {"description": {"identifier": "demo:x", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:breedable": {"breeds_with": {"mate_type": "demo:a", "baby_type": "demo:b"}}, "minecraft:offspring": {"offspring_pairs": {"demo:z": "demo:y"}, "blend_attributes": false}}}}"#,
            ),
            (
                "1.12.0",
                r#"{"format_version": "1.12.0", "minecraft:entity": {"description": {"identifier": "minecraft:turtle", "is_spawnable": true, "is_summonable": true}, "components": {"minecraft:damage_sensor": [{"cause": "lightning", "deals_damage": true, "damage_multiplier": 3}]}}}"#,
            ),
            (
                "1.12.0",
                r#"{"format_version": "1.12.0", "minecraft:entity": {"description": {"identifier": "minecraft:pufferfish", "is_spawnable": true, "is_summonable": true}, "component_groups": {"minecraft:pufferfish_puffed": {"minecraft:scale": {"value": 7.5}}}, "components": {"minecraft:health": {"value": 3}}, "events": {"minecraft:on_puff": {"add": {"component_groups": ["minecraft:pufferfish_puffed"]}}}}}"#,
            ),
            (
                "1.21.120",
                r#"{"format_version": "1.21.120", "minecraft:entity": {"description": {"identifier": "demo:x"}, "components": {"minecraft:breedable": {"breeds_with": [{"mate_type": "demo:x", "baby_type": "demo:x", "breed_event": {"event": "e"}}]}}}}"#,
            ),
            (
                "1.21.120",
                r#"{"format_version": "1.21.120", "minecraft:entity": {"description": {"identifier": "demo:x"}, "components": {"minecraft:ageable": {"feed_items": ["wheat"], "transform_to_item": "bowl"}}}}"#,
            ),
            (
                "1.12.0",
                r#"{"format_version": "1.12.0", "minecraft:entity": {"description": {"identifier": "demo:x"}, "components": {"minecraft:fall_damage": {"value": 2}}}}"#,
            ),
            (
                "1.16.0",
                r#"{"format_version": "1.16.0", "minecraft:entity": {"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.explore_outskirts": {"wait_time": 600}}}}"#,
            ),
            (
                "1.13.0",
                r#"{"format_version": "1.13.0", "minecraft:entity": {"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.circle_around_anchor": {"radius_change_chance": 5, "height_change_chance": 4}}}}"#,
            ),
            (
                "1.21.70",
                r#"{"format_version": "1.21.70", "minecraft:entity": {"description": {"identifier": "demo:x"}, "components": {"minecraft:leashable": {"presets": [{"soft_distance": 2}]}}}}"#,
            ),
            (
                "1.19.0",
                r#"{"format_version": "1.19.0", "minecraft:entity": {"description": {"identifier": "demo:x"}, "components": {"minecraft:behavior.take_flower": {"filters": {"all_of": [{"test": "is_family", "value": "x"}]}}}}}"#,
            ),
            (
                "1.12.0",
                r#"{"format_version": "1.12.0", "minecraft:entity": {"description": {"identifier": "minecraft:iron_golem"}, "components": {"minecraft:behavior.target_when_pushed": {"priority": 1, "percent_chance": 5.0, "entity_types": [{"filters": {"test": "is_family", "value": "monster"}}]}}}}"#,
            ),
        ],
        "trade_table" => &[
            (
                "none",
                r#"{"tiers": [{"trades": [{"wants": [{"item": "minecraft:emerald", "biomes": ["plains", "mushroom"], "filters": {"test": "is_daytime"}}], "gives": [{"item": "minecraft:bread"}]}]}]}"#,
            ),
            (
                "none",
                r#"{"tiers": [{"trades": [{"wants": [{"item": "minecraft:emerald", "biomes": ["plains", "mushroom"]}], "gives": [{"item": "minecraft:bread"}]}]}]}"#,
            ),
            (
                "none",
                r#"{"tiers": [{"trades": [{"wants": [{"item": "minecraft:emerald"}], "gives": [{"item": "minecraft:boat", "filters": {"test": "is_daytime"}, "functions": [{"function": "trader_material_type"}]}]}]}]}"#,
            ),
            (
                "none",
                r#"{"tiers": [{"trades": [{"wants": [{"item": "minecraft:emerald"}], "gives": [{"item": "minecraft:boat", "functions": [{"function": "trader_material_type"}]}]}]}]}"#,
            ),
            (
                "1.20.30",
                r#"{"format_version": "1.20.30", "tiers": [{"trades": [{"wants": [{"item": "minecraft:emerald", "biomes": ["plains"]}], "gives": [{"item": "minecraft:bread"}]}]}]}"#,
            ),
            (
                "none",
                r#"{"tiers": [{"groups": [{"num_to_select": 1, "trades": [{"wants": [{"item": "a:b"}, {"choice": [{"item": "a:c"}]}], "gives": [{"item": "minecraft:bread"}]}]}]}]}"#,
            ),
        ],
        "block" => &[
            // Each legacy member beside its successor.
            (
                "1.19.10",
                r#"{"format_version": "1.19.10", "minecraft:block": {"description": {"identifier": "demo:x", "menu_category": {"category": "nature", "group": "itemGroup.name.wood"}}, "components": {"minecraft:creative_category": {"category": "items"}}}}"#,
            ),
            (
                "1.16.100",
                r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:flammable": {"flame_odds": 5, "burn_odds": 7, "catch_chance_modifier": 1, "destroy_chance_modifier": 2}}}}"#,
            ),
            (
                "1.19.40",
                r#"{"format_version": "1.19.40", "minecraft:block": {"description": {"identifier": "demo:x", "properties": {"demo:open": [true, false]}}, "components": {"minecraft:friction": 0.4}, "permutations": [{"condition": "q.block_property('demo:open')", "components": {"minecraft:rotation": [0, 90, 0], "minecraft:friction": 0.3}}]}}"#,
            ),
            (
                "1.16.100",
                r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:block_light_absorption": 3, "minecraft:block_light_filter": 9}}}"#,
            ),
            (
                "1.16.100",
                r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:destroy_time": 3, "minecraft:destructible_by_mining": {"seconds_to_destroy": 9}}}}"#,
            ),
            (
                "1.16.100",
                r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:explosion_resistance": 3, "minecraft:destructible_by_explosion": {"explosion_resistance": 9}}}}"#,
            ),
            (
                "1.16.100",
                r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:block_light_emission": 0.2, "minecraft:light_emission": 9}}}"#,
            ),
            (
                "1.19.40",
                r#"{"format_version": "1.19.40", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:aim_collision": {"origin": [-8, 0, -8], "size": [16, 4, 16]}, "minecraft:selection_box": {"origin": [-1, 0, -1], "size": [2, 2, 2]}, "minecraft:block_collision": false, "minecraft:collision_box": true}}}"#,
            ),
            (
                "1.19.40",
                r#"{"format_version": "1.19.40", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"tag:wood": {}, "minecraft:tags": ["stone"], "minecraft:friction": 0.1}}}"#,
            ),
            (
                "1.19.40",
                r#"{"format_version": "1.19.40", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:crafting_table": {"custom_description": "A", "table_name": "B", "crafting_tags": ["x"]}}}}"#,
            ),
            (
                "1.20.10",
                r#"{"format_version": "1.20.10", "minecraft:block": {"description": {"identifier": "demo:x", "properties": {"demo:open": [true, false], "demo:level": [0, 1, 2]}}, "components": {"minecraft:friction": 0.4}, "permutations": [{"condition": "q.block_property('demo:open')", "components": {"minecraft:friction": 0.6}}]}}"#,
            ),
            (
                "1.16.100",
                r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x", "use_beta_features": false}, "components": {"minecraft:block_light_absorption": 3, "minecraft:creative_category": {"category": "construction"}, "minecraft:unit_cube": {}, "minecraft:ticking": {"on_tick": {"event": "e"}, "range": [1, 5]}}, "events": {"e": {"set_block_property": {"demo:p": 1}}}}}"#,
            ),
            (
                "1.16.100",
                r#"{"format_version": "1.16.100", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:block_light_absorption": 3}}}"#,
            ),
            // A component under a name of the v1 list.
            (
                "1.19.20",
                r#"{"format_version": "1.19.20", "minecraft:block": {"description": {"identifier": "demo:x", "menu_category": {"category": "construction", "is_hidden_in_commands": true}}, "components": {"minecraft:friction": 0.4}}}"#,
            ),
            (
                "1.19.10",
                r#"{"format_version": "1.19.10", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:creative_category": {"category": "nature", "is_hidden_in_commands": true}}}}"#,
            ),
            (
                "1.19.40",
                r#"{"format_version": "1.19.40", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"minecraft:queued_ticking": {"interval_range": [5, 2], "on_tick": {"event": "e"}}, "minecraft:random_ticking": {"on_tick": {"event": "e"}}}, "events": {"e": {}}}}"#,
            ),
            (
                "1.21.60",
                r#"{"format_version": "1.21.60", "minecraft:block": {"description": {"identifier": "demo:x"}, "components": {"demo:a": {"p": 1}, "minecraft:custom_components": ["demo:a", "demo:b"]}}}"#,
            ),
        ],
        "fog" => &[
            (
                "1.21.0",
                r##"{"format_version": "1.21.0", "minecraft:fog_settings": {"description": {"identifier": "demo:fog"}, "distance": {"air": {"fog_color": "#ABD2FF", "fog_start": 0.5, "fog_end": 1.5, "render_distance_type": "render"}}, "volumetric": {"density": {"weather": {"max_density": 0.5, "max_density_height": 50.5, "zero_density_height": 100.5}}, "media_coefficients": {"air": {"scattering": [0.5, 0.25, 0.125], "absorption": "#aabbcc"}}}}}"##,
            ),
            (
                "1.16.100",
                r##"{"format_version": "1.16.100", "minecraft:fog_settings": {"description": {"identifier": "demo:fog"}, "distance": {"air": {"fog_color": "#ABD2FF", "fog_start": 0.5, "fog_end": 1.5, "render_distance_type": "render"}}, "volumetric": {"density": {"air": {"max_density": 0.5, "max_density_height": 50.5, "zero_density_height": 100.5, "uniform": false}}, "media_coefficients": {"air": {"scattering": [0.5, 0.25, 0.125], "absorption": "#aabbcc"}}}}}"##,
            ),
            (
                "1.21.90",
                r##"{"format_version": "1.21.90", "minecraft:fog_settings": {"description": {"identifier": "demo:fog"}, "distance": {"air": {"fog_color": "#ABD2FF", "fog_start": 0.5, "fog_end": 1.5, "render_distance_type": "render"}, "water": {"fog_color": "#44AFF5", "fog_start": 0.5, "fog_end": 60.5, "render_distance_type": "fixed"}}}}"##,
            ),
        ],
        "pack_settings" => &[(
            "1.26.40",
            r#"{"format_version": "1.26.40", "minecraft:pack_settings": {"settings": [{"pack_id": "d3349771-6782-4420-875a-fcd6179a8915", "last_used": 5120331, "values": {"banana:slider": 5.5, "banana:toggle": true, "banana:choice": "a"}}]}}"#,
        )],
        "geometry" => &[
            (
                "1.14.0",
                r#"{"format_version": "1.14.0", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [{"name": "body", "cubes": [{"origin": [0, 0, 0], "size": [1, 1, 1]}]}]}]}"#,
            ),
            (
                "1.19.30",
                r#"{"format_version": "1.19.30", "minecraft:geometry": [{"description": {"identifier": "geometry.x"}, "bones": [{"name": "body"}]}]}"#,
            ),
        ],
        "client_biome" => &[
            (
                "1.21.70",
                r#"{"format_version": "1.21.70", "minecraft:client_biome": {"description": {"identifier": "minecraft:cherry_grove"}, "components": {}}}"#,
            ),
            (
                "1.21.90",
                r#"{"format_version": "1.21.90", "minecraft:client_biome": {"description": {"identifier": "test:moved"}, "components": {}}}"#,
            ),
            (
                "1.21.100",
                r#"{"format_version": "1.21.100", "minecraft:client_biome": {"description": {"identifier": "minecraft:warped_forest"}, "components": {}}}"#,
            ),
            (
                "1.21.110",
                r#"{"format_version": "1.21.110", "minecraft:client_biome": {"description": {"identifier": "minecraft:river"}, "components": {}}}"#,
            ),
            (
                "1.21.120",
                r#"{"format_version": "1.21.120", "minecraft:client_biome": {"description": {"identifier": "test:moved"}, "components": {}}}"#,
            ),
        ],
        _ => &[],
    }
}
