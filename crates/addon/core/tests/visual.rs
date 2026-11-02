//! The shared description members and the resolver as a user of `bedrock_addon_core::visual`
//! sees them: the name-or-condition entries, `scripts`, `spawn_egg`, the `Visuals` view, and
//! `EngineVersionMerge` over the client-entity and attachable fixture rows.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests read fixtures and call facet-json directly"
)]
use bedrock_addon_core::id::{
    AnimationRef, GeometryId, MaterialId, ParticleId, RenderControllerId, SoundEventName,
    TexturePath,
};
use bedrock_addon_core::json::{StrictBool, StrictString, Value};
use bedrock_addon_core::molang::Molang;
use bedrock_addon_core::version::{FormatVersion, SemVersion};
use bedrock_addon_core::visual::{
    AnimateEntry, Color, ControllerUse, DescriptionSlots, EngineVersionMerge, HasVisuals,
    HeldItemScale, MergeStep, Scripts, ShadowReason, SharedDescription, SpawnEgg, Visuals,
    description_min_engine_version, merge_step,
};
use indexmap::IndexMap;

fn parse<T: facet::Facet<'static>>(json: &str) -> T {
    facet_json::from_str(json).unwrap_or_else(|error| panic!("{json} parses: {error}"))
}

fn write<T: facet::Facet<'static>>(value: &T) -> String {
    facet_json::to_string(value).unwrap()
}

fn sem(text: &str) -> SemVersion {
    SemVersion::parse(text).unwrap()
}

/// The texts of a `Vec<StrictString>` member.
fn strings(list: Option<&Vec<StrictString>>) -> Vec<&str> {
    list.map(|list| list.iter().map(|text| text.0.as_str()).collect())
        .unwrap_or_default()
}

fn expression(value: Option<&Molang>) -> Option<&str> {
    match value {
        Some(Molang::Expr(source)) => Some(source.as_str()),
        _ => None,
    }
}

/// A client entity's `description`, as a family would declare it over the shared member types.
#[derive(Clone, Debug, Default, PartialEq, facet::Facet)]
struct Description {
    identifier: String,
    #[facet(default)]
    min_engine_version: Option<Value>,
    #[facet(default)]
    materials: IndexMap<String, MaterialId>,
    #[facet(default)]
    textures: IndexMap<String, TexturePath>,
    #[facet(default)]
    geometry: IndexMap<String, GeometryId>,
    #[facet(default)]
    animations: Option<IndexMap<String, AnimationRef>>,
    #[facet(default)]
    sound_effects: Option<IndexMap<String, SoundEventName>>,
    #[facet(default)]
    particle_effects: Option<IndexMap<String, ParticleId>>,
    #[facet(default)]
    queryable_geometry: Option<StrictString>,
    #[facet(default)]
    render_controllers: Option<Vec<ControllerUse>>,
    #[facet(default)]
    spawn_egg: Option<SpawnEgg>,
    #[facet(default)]
    scripts: Option<Scripts>,
    #[facet(default)]
    enable_attachables: Option<StrictBool>,
    #[facet(default)]
    held_item_ignores_lighting: Option<StrictBool>,
    #[facet(default)]
    held_item_scale: Option<HeldItemScale>,
    #[facet(default)]
    hide_armor: Option<StrictBool>,
}

impl SharedDescription for Description {
    fn slots(&mut self) -> DescriptionSlots<'_> {
        DescriptionSlots {
            materials: &mut self.materials,
            textures: &mut self.textures,
            geometry: &mut self.geometry,
            animations: &mut self.animations,
            sound_effects: &mut self.sound_effects,
            particle_effects: &mut self.particle_effects,
            queryable_geometry: &mut self.queryable_geometry,
            render_controllers: &mut self.render_controllers,
            spawn_egg: &mut self.spawn_egg,
            scripts: &mut self.scripts,
            enable_attachables: &mut self.enable_attachables,
            held_item_ignores_lighting: &mut self.held_item_ignores_lighting,
            held_item_scale: &mut self.held_item_scale,
            hide_armor: &mut self.hide_armor,
        }
    }
}

impl HasVisuals for Description {
    fn visuals(&self) -> Visuals<'_> {
        Visuals::new(
            Some(&self.materials),
            Some(&self.textures),
            Some(&self.geometry),
            self.animations.as_ref(),
            self.render_controllers.as_deref().unwrap_or_default(),
        )
    }
}

fn description(json: &str) -> Description {
    parse(json)
}

mod controller_use {
    use super::*;

    #[test]
    fn the_string_form_is_always_drawn() {
        let entry: ControllerUse = parse(r#""controller.render.pig""#);
        assert_eq!(
            entry,
            ControllerUse::Always(RenderControllerId::parse("controller.render.pig"))
        );
        assert_eq!(
            entry.controller(),
            Some(RenderControllerId::parse("controller.render.pig"))
        );
        assert!(entry.condition().is_none());
        assert_eq!(entry.ignored().count(), 0);
        assert_eq!(write(&entry), r#""controller.render.pig""#);
    }

    #[test]
    fn l08_the_object_form_reads_one_member() {
        let json = r#"{"controller.render.a":"q.is_baby","controller.render.b":"1"}"#;
        let entry: ControllerUse = parse(json);
        assert_eq!(
            entry.controller(),
            Some(RenderControllerId::parse("controller.render.a"))
        );
        assert_eq!(expression(entry.condition()), Some("q.is_baby"));
        assert_eq!(entry.ignored().collect::<Vec<_>>(), ["controller.render.b"]);
        assert_eq!(write(&entry), json, "every member is written back");
    }

    #[test]
    fn l14_the_member_read_is_the_smallest_key_not_the_first_written() {
        let entry: ControllerUse =
            parse(r#"{ "controller.render.b": "1", "controller.render.a": "q.is_baby" }"#);
        assert_eq!(
            entry.controller(),
            Some(RenderControllerId::parse("controller.render.a"))
        );
        assert_eq!(expression(entry.condition()), Some("q.is_baby"));
        assert_eq!(entry.ignored().collect::<Vec<_>>(), ["controller.render.b"]);
    }

    #[test]
    fn l21_an_empty_object_names_no_controller() {
        let entry: ControllerUse = parse("{}");
        assert_eq!(entry, ControllerUse::Conditional(IndexMap::new()));
        assert!(entry.controller().is_none());
        assert!(entry.condition().is_none());
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "the reader gives exact values")]
    fn m05_a_number_or_boolean_condition_is_kept_as_written() {
        let entry: ControllerUse = parse(r#"{ "controller.render.x": false }"#);
        assert!(matches!(entry.condition(), Some(Molang::Bool(false))));
        let entry: ControllerUse = parse(r#"{ "controller.render.x": 1 }"#);
        assert!(matches!(entry.condition(), Some(Molang::Const(c)) if *c == 1.0));
    }

    #[test]
    fn only_a_string_or_an_object_is_an_entry() {
        for json in [
            "5",
            "-1",
            "1.5",
            "true",
            "null",
            r#"["controller.render.x"]"#,
        ] {
            assert!(
                facet_json::from_str::<ControllerUse>(json).is_err(),
                "{json} is not an entry"
            );
        }
    }

    #[test]
    fn a_list_mixes_both_forms() {
        let list: Vec<ControllerUse> =
            parse(r#"["controller.render.a", { "controller.render.b": "q.is_baby" }]"#);
        let controllers: Vec<_> = list.iter().filter_map(ControllerUse::controller).collect();
        assert_eq!(
            controllers,
            [
                RenderControllerId::parse("controller.render.a"),
                RenderControllerId::parse("controller.render.b")
            ]
        );
    }
}

mod animate_entry {
    use super::*;

    #[test]
    fn the_string_form_is_always_played() {
        let entry: AnimateEntry = parse(r#""walk""#);
        assert_eq!(entry, AnimateEntry::Always("walk".to_owned()));
        assert_eq!(entry.name(), Some("walk"));
        assert!(entry.condition().is_none());
    }

    #[test]
    fn the_object_form_keeps_its_key_as_written() {
        // Object-form `animate` keys are not lower-cased by the game.
        let entry: AnimateEntry = parse(r#"{ "Walk": "query.is_moving", "Idle": "1.0" }"#);
        assert_eq!(entry.name(), Some("Idle"), "`I` sorts before `W`");
        assert_eq!(expression(entry.condition()), Some("1.0"));
        assert_eq!(entry.ignored().collect::<Vec<_>>(), ["Walk"]);
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "the reader gives exact values")]
    fn m05_numeric_and_boolean_conditions() {
        let list: Vec<AnimateEntry> = parse(r#"[{ "walk": 1 }, { "idle": true }]"#);
        assert!(matches!(list[0].condition(), Some(Molang::Const(c)) if *c == 1.0));
        assert!(matches!(list[1].condition(), Some(Molang::Bool(true))));
    }

    #[test]
    fn only_a_string_or_an_object_is_an_entry() {
        for json in ["5", "true", "null", "[]"] {
            assert!(
                facet_json::from_str::<AnimateEntry>(json).is_err(),
                "{json}"
            );
        }
    }
}

mod scripts {
    use bedrock_addon_core::visual::history;

    use super::*;

    #[test]
    fn the_newest_shape_reads_every_member() {
        let scripts: Scripts = parse(
            r#"{
                "variables": { "variable.attack_time": "public" },
                "initialize": ["variable.a = 1;", "variable.b = 2;"],
                "pre_animation": ["variable.c = query.is_baby;"],
                "scale": "0.9375",
                "parent_setup": "variable.helmet_layer_visible = 0.0;",
                "should_update_bones_and_effects_offscreen": "1.0",
                "should_update_effects_offscreen": "!query.is_on_screen",
                "hide_held_items": "variable.has_absorbed_tnt",
                "animate": ["walk", { "look_at_target": "query.is_baby" }]
            }"#,
        );
        assert_eq!(
            scripts.variables.as_ref().unwrap()["variable.attack_time"].0,
            "public"
        );
        assert_eq!(
            strings(scripts.initialize.as_ref()),
            ["variable.a = 1;", "variable.b = 2;"]
        );
        assert_eq!(expression(scripts.scale.as_ref()), Some("0.9375"));
        assert_eq!(
            expression(scripts.hide_held_items.as_ref()),
            Some("variable.has_absorbed_tnt")
        );
        let animate = scripts.animate.as_ref().unwrap();
        assert_eq!(animate[0].name(), Some("walk"));
        assert_eq!(animate[1].name(), Some("look_at_target"));
    }

    #[test]
    fn absent_members_stay_absent() {
        let scripts: Scripts = parse("{}");
        assert_eq!(scripts, Scripts::default());
    }

    #[test]
    fn l10_scale_x_and_scalex_are_one_script_and_scalex_wins() {
        for json in [
            r#"{ "scalex": "2.0", "scaleX": "3.0" }"#,
            r#"{ "scaleX": "3.0", "scalex": "2.0" }"#,
        ] {
            let scripts: Scripts = parse(json);
            assert_eq!(expression(scripts.scale_x.as_ref()), Some("3.0"));
            assert_eq!(expression(scripts.scalex.as_ref()), Some("2.0"));
            assert_eq!(
                scripts
                    .effective_scale_x()
                    .map(bedrock_addon_core::molang::MolangSource::as_str),
                Some("2.0")
            );
        }
        let scripts: Scripts = parse(r#"{ "scaleY": "3.0" }"#);
        assert_eq!(
            scripts
                .effective_scale_y()
                .map(bedrock_addon_core::molang::MolangSource::as_str),
            Some("3.0")
        );
        assert!(scripts.effective_scale_z().is_none());
    }

    #[test]
    fn m06_a_number_builds_no_named_script() {
        let scripts: Scripts = parse(r#"{ "scalex": 0.9375, "scaleX": "2.0" }"#);
        assert!(matches!(scripts.scalex, Some(Molang::Const(_))));
        assert_eq!(
            scripts
                .effective_scale_x()
                .map(bedrock_addon_core::molang::MolangSource::as_str),
            Some("2.0"),
            "the number is skipped, so `scaleX` is the script"
        );
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "the reader gives exact values")]
    fn version_1_10_elements_are_molang_values() {
        let scripts: history::v1_10_0::Scripts =
            parse(r#"{ "initialize": ["variable.a = 1;", 2, true], "animate": ["walk"] }"#);
        let initialize = scripts.initialize.unwrap();
        assert_eq!(expression(Some(&initialize[0])), Some("variable.a = 1;"));
        assert!(matches!(initialize[1], Molang::Const(c) if c == 2.0));
        assert!(matches!(initialize[2], Molang::Bool(true)));
    }

    #[test]
    fn version_1_8_has_the_scale_scripts_and_parent_setup() {
        let scripts: history::v1_8_0::Scripts = parse(
            r#"{ "pre_animation": ["variable.x = 1;"], "scaleX": "1.5", "parent_setup": "variable.y = 0;" }"#,
        );
        assert_eq!(expression(scripts.scale_x.as_ref()), Some("1.5"));
        assert_eq!(scripts.pre_animation.unwrap().len(), 1);
        assert!(scripts.parent_setup.is_some());
    }
}

mod spawn_egg {
    use super::*;

    #[test]
    fn l23_an_empty_spawn_egg_has_vanillas_defaults() {
        let egg: SpawnEgg = parse("{}");
        let info = egg.resolved();
        assert_eq!(info.texture, "");
        assert_eq!(info.texture_index, 0);
        assert_eq!(info.base_color, Color::NIL);
        assert_eq!(info.overlay_color, Color::NIL);
        assert_eq!(
            Color::NIL,
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0
            }
        );
    }

    #[test]
    fn l15_a_colour_needs_its_hash_and_rrggbb_has_alpha_zero() {
        let egg: SpawnEgg = parse(r##"{ "base_color": "#ff0000", "overlay_color": "red" }"##);
        let info = egg.resolved();
        assert_eq!(
            info.base_color,
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 0.0
            }
        );
        assert_eq!(
            info.overlay_color,
            Color::NIL,
            "no `#`: the default, silently"
        );
        assert_eq!(
            egg.overlay_color.as_ref().map(|color| color.0.as_str()),
            Some("red"),
            "kept as written"
        );
    }

    #[test]
    fn aarrggbb_carries_alpha() {
        let color = Color::from_hex("#80ff8000").unwrap();
        assert_eq!(color.a, 128.0 / 255.0);
        assert_eq!((color.r, color.g, color.b), (1.0, 128.0 / 255.0, 0.0));
    }

    #[test]
    fn unreadable_colours_keep_the_default() {
        for text in ["", "#", "#xyz", "#123456789", "ff0000"] {
            assert_eq!(Color::from_hex(text), None, "{text:?}");
        }
    }

    #[test]
    fn the_texture_and_its_frame() {
        let egg: SpawnEgg = parse(r#"{ "texture": "spawn_egg", "texture_index": 3 }"#);
        let info = egg.resolved();
        assert_eq!((info.texture, info.texture_index), ("spawn_egg", 3));
    }

    /// The game's result for a negative index (`asUInt(0)` of a negative int) is not known, so the
    /// crate reads it as zero.
    #[test]
    fn deviation_a_negative_texture_index_reads_as_zero() {
        let egg: SpawnEgg = parse(r#"{ "texture_index": -2 }"#);
        assert_eq!(egg.resolved().texture_index, 0);
    }

    /// Whether the game's hex parse stops at a non-hex digit or fails outright is not known, so the
    /// crate keeps the default.
    #[test]
    fn deviation_a_colour_with_a_non_hex_digit_keeps_the_default() {
        assert_eq!(Color::from_hex("#ff00zz"), None);
        let egg: SpawnEgg = parse(r##"{ "base_color": "#ff00zz" }"##);
        assert_eq!(egg.resolved().base_color, Color::NIL);
    }
}

mod visuals {
    use super::*;

    #[test]
    fn the_view_borrows_the_documents_slots() {
        let description = description(
            r#"{
                "identifier": "demo:ghost",
                "materials": { "default": "entity_alphatest" },
                "textures": { "default": "textures/entity/ghost" },
                "geometry": { "default": "geometry.ghost" },
                "animations": { "walk": "animation.ghost.walk", "move": "controller.animation.ghost.move" },
                "render_controllers": ["controller.render.ghost"]
            }"#,
        );
        let visuals = description.visuals();
        let geometry: Vec<_> = visuals
            .geometry
            .unwrap()
            .iter()
            .map(|(slot, id)| (slot.as_str(), id.as_str()))
            .collect();
        assert_eq!(geometry, [("default", "geometry.ghost")]);
        assert_eq!(
            visuals.texture("default"),
            Some(&TexturePath::parse("textures/entity/ghost"))
        );
        assert_eq!(
            visuals.material("default").unwrap().as_str(),
            "entity_alphatest"
        );
        assert_eq!(
            visuals.geometry("default").unwrap().as_str(),
            "geometry.ghost"
        );
        assert_eq!(visuals.animations.unwrap().len(), 2);
        assert!(visuals.animation("walk").is_some());
        assert!(visuals.animation("fly").is_none());
        assert_eq!(visuals.render_controllers.len(), 1);
    }

    #[test]
    fn a_document_without_render_controllers_has_none() {
        let description = description(r#"{ "identifier": "demo:ghost" }"#);
        assert!(description.visuals().render_controllers.is_empty());
        // Nor the `animations` it did not write; a lookup finds nothing.
        assert!(description.visuals().animations.is_none());
        assert!(description.visuals().animation("walk").is_none());
    }

    /// A member written as an empty object is not a member that was not written.
    #[test]
    fn an_authored_empty_map_is_present() {
        let description = description(r#"{ "identifier": "demo:ghost", "animations": {} }"#);
        assert!(
            description
                .visuals()
                .animations
                .is_some_and(IndexMap::is_empty)
        );
        assert_eq!(description.animations, Some(IndexMap::new()));
        assert_eq!(description.sound_effects, None);
    }
}

mod resolver {
    use super::*;

    const FULL_STACK: &str = "1.21.0";

    fn stack() -> EngineVersionMerge<Description> {
        EngineVersionMerge::new(sem(FULL_STACK))
    }

    /// Offers a description at its own `min_engine_version` (format version 1.10.0).
    fn offer(map: &mut EngineVersionMerge<Description>, json: &str) -> MergeStep {
        let description = description(json);
        let version = description_min_engine_version(
            FormatVersion::new(1, 10, 0),
            description.min_engine_version.as_ref(),
        );
        let identifier = description.identifier.clone();
        map.offer(&identifier, version, description)
    }

    #[test]
    fn the_step_table() {
        let stack = sem("1.21.0");
        let (v16, v20, v21, v22) = (sem("1.16.0"), sem("1.20.0"), sem("1.21.0"), sem("1.22.0"));
        assert_eq!(merge_step(&stack, &v16, None), MergeStep::Create);
        assert_eq!(
            merge_step(&stack, &v21, None),
            MergeStep::Create,
            "equal to the stack"
        );
        assert_eq!(
            merge_step(&stack, &v22, None),
            MergeStep::Ignore(ShadowReason::MinEngineVersionAboveStack)
        );
        assert_eq!(
            merge_step(&stack, &v16, Some(&v20)),
            MergeStep::Ignore(ShadowReason::OlderMinEngineVersion)
        );
        assert_eq!(merge_step(&stack, &v20, Some(&v20)), MergeStep::Merge);
        assert_eq!(merge_step(&stack, &v20, Some(&v16)), MergeStep::Replace);
        assert_eq!(
            merge_step(&stack, &v22, Some(&v16)),
            MergeStep::Ignore(ShadowReason::MinEngineVersionAboveStack),
            "the stack check comes first"
        );
    }

    #[test]
    fn equal_means_equal_numbers_and_pre_release() {
        let stack = sem("1.21.0");
        assert_eq!(
            merge_step(&stack, &sem("1.20.0+build"), Some(&sem("1.20.0"))),
            MergeStep::Merge
        );
        assert_eq!(
            merge_step(&stack, &sem("1.20.0-beta"), Some(&sem("1.20.0"))),
            MergeStep::Ignore(ShadowReason::OlderMinEngineVersion)
        );
    }

    #[test]
    fn the_competing_version() {
        let json = |text: &str| -> Value { parse(text) };
        let v1_8 = FormatVersion::new(1, 8, 0);
        let zero = sem("0.0.0");
        assert_eq!(description_min_engine_version(v1_8, None), zero);
        // p06: a 1.8.0 document competes at its 1.2.6, not at 0.0.0.
        assert_eq!(
            description_min_engine_version(v1_8, Some(&json(r#""1.2.6""#))),
            sem("1.2.6")
        );
        assert_eq!(
            description_min_engine_version(v1_8, Some(&json("[1, 16, 0]"))),
            sem("1.16.0")
        );
        assert_eq!(
            description_min_engine_version(v1_8, Some(&json(r#""1.13""#))),
            sem("1.13.0")
        );
        for malformed in [r#""one""#, r#""*""#, "1.16", "true", "[1, 16]"] {
            assert_eq!(
                description_min_engine_version(v1_8, Some(&json(malformed))),
                zero,
                "{malformed} reads as 0.0.0"
            );
        }
        assert_eq!(
            description_min_engine_version(FormatVersion::new(1, 7, 0), Some(&json(r#""1.2.6""#))),
            zero,
            "below 1.8.0 the member is not read"
        );
    }

    #[test]
    fn p01_an_equal_version_merges_key_by_key() {
        let mut map = stack();
        let low = r#"{ "identifier": "demo:ghost",
            "geometry": { "default": "geometry.ghost" },
            "textures": { "default": "textures/low" } }"#;
        let high = r#"{ "identifier": "demo:ghost", "textures": { "default": "textures/high" } }"#;
        assert_eq!(offer(&mut map, low), MergeStep::Create);
        assert_eq!(offer(&mut map, high), MergeStep::Merge);
        let ghost = map.get("demo:ghost").unwrap();
        assert_eq!(ghost.geometry["default"].as_str(), "geometry.ghost");
        assert_eq!(ghost.textures["default"].as_str(), "textures/high");
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn p02_a_lower_version_is_ignored_and_a_higher_one_replaces() {
        let low = r#"{ "identifier": "demo:ghost", "min_engine_version": "1.16.0",
            "geometry": { "default": "geometry.low" }, "textures": { "default": "textures/low" } }"#;
        let high = r#"{ "identifier": "demo:ghost", "min_engine_version": "1.8.0",
            "geometry": { "default": "geometry.high" } }"#;
        let mut map = stack();
        offer(&mut map, low);
        assert_eq!(
            offer(&mut map, high),
            MergeStep::Ignore(ShadowReason::OlderMinEngineVersion)
        );
        assert_eq!(
            map.get("demo:ghost").unwrap().geometry["default"].as_str(),
            "geometry.low"
        );

        let low = low.replace("1.16.0", "1.7.0");
        let high = high.replace("1.8.0", "1.16.0");
        let mut map = stack();
        offer(&mut map, &low);
        assert_eq!(offer(&mut map, &high), MergeStep::Replace);
        let ghost = map.get("demo:ghost").unwrap();
        assert_eq!(ghost.geometry["default"].as_str(), "geometry.high");
        assert!(ghost.textures.is_empty(), "no low-pack key survives");
        assert_eq!(map.min_engine_version("demo:ghost"), Some(&sem("1.16.0")));
    }

    #[test]
    fn p03_a_version_above_the_stack_is_ignored() {
        let mut map = EngineVersionMerge::<Description>::new(sem("1.16.0"));
        let step = offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "min_engine_version": "1.20.0" }"#,
        );
        assert_eq!(
            step,
            MergeStep::Ignore(ShadowReason::MinEngineVersionAboveStack)
        );
        assert!(map.is_empty());
    }

    // The attachable resolver rows, the shared-map part only;
    // item bindings and `AttachableShadowed` are the attachable family's.

    #[test]
    fn a20_two_packs_equal_mev_merge() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "example:wand", "textures": { "default": "a" } }"#,
        );
        let step = offer(
            &mut map,
            r#"{ "identifier": "example:wand", "textures": { "default": "b" }, "geometry": { "default": "g" } }"#,
        );
        assert_eq!(step, MergeStep::Merge);
        let wand = map.get("example:wand").unwrap();
        assert_eq!(wand.textures["default"].as_str(), "b");
        assert_eq!(wand.geometry["default"].as_str(), "g");
    }

    #[test]
    fn a21_higher_mev_replaces() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "example:wand", "min_engine_version": "1.16.0", "textures": { "default": "a" } }"#,
        );
        let step = offer(
            &mut map,
            r#"{ "identifier": "example:wand", "min_engine_version": "1.20.0", "geometry": { "default": "g" } }"#,
        );
        assert_eq!(step, MergeStep::Replace);
        assert!(map.get("example:wand").unwrap().textures.is_empty());
    }

    #[test]
    fn a22_lower_mev_ignored() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "example:wand", "min_engine_version": "1.20.0", "textures": { "default": "a" } }"#,
        );
        let step = offer(
            &mut map,
            r#"{ "identifier": "example:wand", "min_engine_version": "1.16.0", "textures": { "default": "b" } }"#,
        );
        assert_eq!(step, MergeStep::Ignore(ShadowReason::OlderMinEngineVersion));
        assert_eq!(
            map.get("example:wand").unwrap().textures["default"].as_str(),
            "a"
        );
    }

    #[test]
    fn a23_mev_above_stack() {
        let mut map = EngineVersionMerge::<Description>::new(sem("1.20.0"));
        let step = offer(
            &mut map,
            r#"{ "identifier": "example:wand", "min_engine_version": "1.21.0" }"#,
        );
        assert_eq!(
            step,
            MergeStep::Ignore(ShadowReason::MinEngineVersionAboveStack)
        );
        assert!(map.is_empty());
    }

    #[test]
    fn p04_the_versioned_file_of_a_vanilla_pair_wins_in_either_order() {
        let versioned = r#"{ "identifier": "minecraft:guardian", "min_engine_version": "1.8.0",
            "geometry": { "default": "geometry.guardian.v1.8" } }"#;
        let legacy = r#"{ "identifier": "minecraft:guardian",
            "geometry": { "default": "geometry.guardian" } }"#;
        for order in [[versioned, legacy], [legacy, versioned]] {
            let mut map = stack();
            for json in order {
                offer(&mut map, json);
            }
            assert_eq!(
                map.get("minecraft:guardian").unwrap().geometry["default"].as_str(),
                "geometry.guardian.v1.8"
            );
        }
    }

    #[test]
    fn p05_render_controllers_are_replaced_not_appended() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "render_controllers": ["controller.render.a", "controller.render.b"] }"#,
        );
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "render_controllers": [{ "controller.render.c": "q.is_baby" }] }"#,
        );
        offer(&mut map, r#"{ "identifier": "demo:ghost" }"#);
        let ghost = map.get("demo:ghost").unwrap();
        let controllers: Vec<_> = ghost
            .visuals()
            .render_controllers
            .iter()
            .filter_map(ControllerUse::controller)
            .collect();
        assert_eq!(
            controllers,
            [RenderControllerId::parse("controller.render.c")]
        );
    }

    #[test]
    fn p07_animate_is_replaced_not_appended() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "scripts": { "animate": ["a", "b"], "scale": "2.0" } }"#,
        );
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "scripts": { "animate": ["c"] } }"#,
        );
        let scripts = map.get("demo:ghost").unwrap().scripts.as_ref().unwrap();
        assert_eq!(
            scripts.animate,
            Some(vec![AnimateEntry::Always("c".to_owned())])
        );
        assert_eq!(expression(scripts.scale.as_ref()), Some("2.0"), "untouched");
    }

    #[test]
    fn p08_a_later_spawn_egg_replaces_the_earlier_one_whole() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "spawn_egg": { "texture": "x", "texture_index": 3 } }"#,
        );
        offer(
            &mut map,
            r##"{ "identifier": "demo:ghost", "spawn_egg": { "base_color": "#00ff00" } }"##,
        );
        let egg = map
            .get("demo:ghost")
            .unwrap()
            .spawn_egg
            .as_ref()
            .unwrap()
            .resolved();
        assert_eq!((egg.texture, egg.texture_index), ("", 0));
        assert_eq!(
            egg.base_color,
            Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 0.0
            }
        );
    }

    /// The merged definition has `animations` / `sound_effects` when one of the two
    /// documents has the member, an authored empty object included; no merge invents one.
    #[test]
    fn a_merged_definition_has_a_slot_map_when_one_of_its_documents_has() {
        let ghost = |earlier: &str, later: &str| {
            let mut map = stack();
            offer(
                &mut map,
                &format!(r#"{{ "identifier": "demo:ghost"{earlier} }}"#),
            );
            assert_eq!(
                offer(
                    &mut map,
                    &format!(r#"{{ "identifier": "demo:ghost"{later} }}"#)
                ),
                MergeStep::Merge
            );
            map.get("demo:ghost").unwrap().clone()
        };
        let neither = ghost("", "");
        assert_eq!(neither.animations, None);
        assert_eq!(neither.sound_effects, None);
        let empty_later = ghost("", r#", "animations": {}"#);
        assert_eq!(empty_later.animations, Some(IndexMap::new()));
        assert_eq!(empty_later.sound_effects, None);
        let empty_earlier = ghost(r#", "sound_effects": {}"#, "");
        assert_eq!(empty_earlier.sound_effects, Some(IndexMap::new()));
        assert_eq!(empty_earlier.animations, None);
        let one = ghost(
            r#", "animations": {"walk": "animation.walk"}"#,
            r#", "animations": {}"#,
        );
        assert_eq!(one.animations.as_ref().unwrap().len(), 1);
        let later = ghost("", r#", "animations": {"walk": "animation.walk"}"#);
        assert_eq!(later.animations.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn particle_effects_are_replaced_and_sound_effects_merged() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost",
                "particle_effects": { "smoke": "minecraft:basic_smoke_particle", "flame": "minecraft:basic_flame_particle" },
                "sound_effects": { "step": "mob.ghost.step" } }"#,
        );
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost",
                "particle_effects": { "smoke": "demo:smoke" },
                "sound_effects": { "hurt": "mob.ghost.hurt" } }"#,
        );
        let ghost = map.get("demo:ghost").unwrap();
        let particles = ghost.particle_effects.as_ref().unwrap();
        assert_eq!(particles.len(), 1);
        assert_eq!(particles["smoke"].as_str(), "demo:smoke");
        assert_eq!(ghost.sound_effects.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn map_keys_merge_without_ascii_case() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "textures": { "Default": "textures/low", "eyes": "textures/eyes" } }"#,
        );
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "textures": { "default": "textures/high" } }"#,
        );
        let textures = &map.get("demo:ghost").unwrap().textures;
        let entries: Vec<_> = textures
            .iter()
            .map(|(slot, path)| (slot.as_str(), path.as_str()))
            .collect();
        assert_eq!(
            entries,
            [("default", "textures/high"), ("eyes", "textures/eyes")]
        );
    }

    /// The key a later document spells differently is found by its new spelling afterwards:
    /// the map is one a lookup works on.
    #[test]
    fn a_key_respelt_by_a_later_document_is_found_by_its_new_spelling() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "textures": { "Default": "textures/low", "eyes": "textures/eyes" }, "animations": { "Walk": "animation.walk" } }"#,
        );
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "textures": { "default": "textures/high", "EYES": "textures/eyes2", "new": "textures/new" }, "animations": { "walk": "animation.walk2" } }"#,
        );
        let ghost = map.get("demo:ghost").unwrap();
        let path = |slot: &str| ghost.textures.get(slot).map(TexturePath::as_str);
        assert_eq!(path("default"), Some("textures/high"));
        assert_eq!(path("EYES"), Some("textures/eyes2"));
        assert_eq!(path("new"), Some("textures/new"));
        assert_eq!(path("Default"), None);
        assert_eq!(path("eyes"), None);
        let slots: Vec<&str> = ghost.textures.keys().map(String::as_str).collect();
        assert_eq!(slots, ["default", "EYES", "new"]);
        let animations = ghost.animations.as_ref().unwrap();
        assert!(animations.contains_key("walk"));
        assert_eq!(animations.len(), 1);
    }

    /// Two documents of a hundred thousand slot names each, every later name a respelling of
    /// an earlier one: each later key is looked up, not searched for. A
    /// search per key is ten thousand million comparisons here.
    #[test]
    fn merging_many_slot_names_looks_each_one_up() {
        const SLOTS: usize = 100_000;
        let document = |upper: bool| {
            let mut description = Description {
                identifier: "demo:ghost".to_owned(),
                ..Description::default()
            };
            for index in 0..SLOTS {
                let slot = if upper {
                    format!("SLOT_{index}")
                } else {
                    format!("slot_{index}")
                };
                description
                    .textures
                    .insert(slot, parse(&format!("\"textures/t{index}_{upper}\"")));
            }
            description
        };
        let (mut earlier, mut later) = (document(false), document(true));
        bedrock_addon_core::visual::merge_slots(&mut earlier.slots(), &mut later.slots());
        assert_eq!(earlier.textures.len(), SLOTS);
        assert!(later.textures.is_empty());
        let (first, path) = earlier.textures.get_index(0).unwrap();
        assert_eq!(
            (first.as_str(), path.as_str()),
            ("SLOT_0", "textures/t0_true")
        );
        let last = earlier
            .textures
            .get(&format!("SLOT_{}", SLOTS - 1))
            .unwrap();
        assert_eq!(last.as_str(), format!("textures/t{}_true", SLOTS - 1));
    }

    #[test]
    fn identifiers_compare_in_lower_case() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "Demo:Ghost", "hide_armor": true }"#,
        );
        assert_eq!(
            offer(
                &mut map,
                r#"{ "identifier": "demo:ghost", "enable_attachables": true }"#
            ),
            MergeStep::Merge
        );
        let ghost = map.get("DEMO:GHOST").unwrap();
        assert_eq!(
            (ghost.hide_armor, ghost.enable_attachables),
            (Some(StrictBool(true)), Some(StrictBool(true)))
        );
        assert_eq!(
            map.iter().map(|(key, _)| key).collect::<Vec<_>>(),
            ["demo:ghost"]
        );
    }

    #[test]
    fn a_later_false_overrides_an_earlier_true() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "enable_attachables": true, "hide_armor": true,
                "held_item_ignores_lighting": true, "queryable_geometry": "geometry.a" }"#,
        );
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "enable_attachables": false, "queryable_geometry": "geometry.b" }"#,
        );
        let ghost = map.get("demo:ghost").unwrap();
        assert_eq!(ghost.enable_attachables, Some(StrictBool(false)));
        assert_eq!(
            ghost.hide_armor,
            Some(StrictBool(true)),
            "absent in the later document"
        );
        assert_eq!(ghost.held_item_ignores_lighting, Some(StrictBool(true)));
        assert_eq!(
            ghost
                .queryable_geometry
                .as_ref()
                .map(|text| text.0.as_str()),
            Some("geometry.b")
        );
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "the reader gives exact values")]
    fn l04_a_held_item_scale_at_or_below_zero_is_not_stored() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "held_item_scale": 2.0 }"#,
        );
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "held_item_scale": 0 }"#,
        );
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "held_item_scale": -1.5 }"#,
        );
        let ghost = map.get("demo:ghost").unwrap();
        assert!(matches!(ghost.held_item_scale, Some(HeldItemScale::Number(s)) if s == 2.0));
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "held_item_scale": 0.5 }"#,
        );
        let ghost = map.get("demo:ghost").unwrap();
        assert!(matches!(ghost.held_item_scale, Some(HeldItemScale::Number(s)) if s == 0.5));
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "held_item_scale": "query.is_on_screen ? 1.5 : 1.0" }"#,
        );
        let ghost = map.get("demo:ghost").unwrap();
        assert_eq!(
            expression(match &ghost.held_item_scale {
                Some(HeldItemScale::Molang(molang)) => Some(molang),
                _ => None,
            }),
            Some("query.is_on_screen ? 1.5 : 1.0"),
            "from engine 1.26.30 a Molang value is stored"
        );
    }

    #[test]
    fn named_scripts_merge_one_by_one() {
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "scripts": {
                "variables": { "variable.a": "public" },
                "initialize": ["variable.x = 1;"],
                "pre_animation": ["variable.y = 1;"],
                "scale": "1.5", "scaleX": "2.0", "parent_setup": "variable.p = 1;" } }"#,
        );
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "scripts": {
                "variables": { "variable.b": "public" },
                "pre_animation": ["variable.y = 2;"],
                "scale": 0.5, "scalex": "3.0", "parent_setup": "" } }"#,
        );
        let scripts = map.get("demo:ghost").unwrap().scripts.as_ref().unwrap();
        let variables: Vec<_> = scripts.variables.as_ref().unwrap().keys().collect();
        assert_eq!(variables, ["variable.a", "variable.b"]);
        assert_eq!(strings(scripts.initialize.as_ref()), ["variable.x = 1;"]);
        assert_eq!(strings(scripts.pre_animation.as_ref()), ["variable.y = 2;"]);
        assert_eq!(
            expression(scripts.scale.as_ref()),
            Some("1.5"),
            "a number builds no script, so the earlier one stays"
        );
        assert_eq!(
            scripts
                .effective_scale_x()
                .map(bedrock_addon_core::molang::MolangSource::as_str),
            Some("3.0"),
            "`scalex` replaces the earlier `scaleX`: one script"
        );
        assert_eq!(
            expression(scripts.parent_setup.as_ref()),
            Some("variable.p = 1;"),
            "an empty source builds no script"
        );
    }

    #[test]
    fn scripts_merge_into_a_definition_that_had_none() {
        let mut map = stack();
        offer(&mut map, r#"{ "identifier": "demo:ghost" }"#);
        offer(
            &mut map,
            r#"{ "identifier": "demo:ghost", "scripts": { "animate": ["walk"] } }"#,
        );
        let scripts = map.get("demo:ghost").unwrap().scripts.as_ref().unwrap();
        assert_eq!(scripts.animate.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn a29_documents_without_a_version_merge_in_load_order() {
        // The app-bundled armour set below a versioned vanilla pack: neither sets
        // `min_engine_version`, so the later layer merges key by key.
        let mut map = stack();
        offer(
            &mut map,
            r#"{ "identifier": "minecraft:chainmail_boots.player",
                "materials": { "default": "armor", "enchanted": "armor_enchanted" },
                "textures": { "default": "textures/models/armor/chain_1" } }"#,
        );
        assert_eq!(
            offer(
                &mut map,
                r#"{ "identifier": "minecraft:chainmail_boots.player",
                    "textures": { "default": "textures/models/armor/chain_1", "enchanted": "textures/misc/enchanted_actor_glint" } }"#,
            ),
            MergeStep::Merge
        );
        let boots = map.get("minecraft:chainmail_boots.player").unwrap();
        assert_eq!(boots.materials.len(), 2);
        assert_eq!(boots.textures.len(), 2);
    }

    #[test]
    fn a_replaced_definition_keeps_its_place() {
        let mut map = stack();
        offer(&mut map, r#"{ "identifier": "demo:a" }"#);
        offer(&mut map, r#"{ "identifier": "demo:b" }"#);
        offer(
            &mut map,
            r#"{ "identifier": "demo:a", "min_engine_version": "1.16.0" }"#,
        );
        let order: Vec<_> = map.iter().map(|(key, _)| key).collect();
        assert_eq!(order, ["demo:a", "demo:b"]);
        let definitions: Vec<_> = map.into_definitions().map(|(key, _)| key).collect();
        assert_eq!(definitions, ["demo:a", "demo:b"]);
    }

    /// A family member outside the shared slots (an attachable's `item`).
    #[derive(Debug)]
    struct WithOwn {
        description: Description,
        own: Vec<&'static str>,
    }

    impl SharedDescription for WithOwn {
        fn slots(&mut self) -> DescriptionSlots<'_> {
            self.description.slots()
        }

        fn merge_own(&mut self, later: &mut Self) {
            self.own.append(&mut later.own);
        }
    }

    #[test]
    fn a_family_merges_its_own_members_after_the_shared_ones() {
        let mut map = EngineVersionMerge::new(sem(FULL_STACK));
        for own in ["first", "second"] {
            let description = description(r#"{ "identifier": "demo:wand" }"#);
            map.offer(
                "demo:wand",
                sem("0.0.0"),
                WithOwn {
                    description,
                    own: vec![own],
                },
            );
        }
        assert_eq!(map.get("demo:wand").unwrap().own, ["first", "second"]);
    }
}

mod round_trip {
    use bedrock_addon_core::visual::history;

    use super::*;

    /// Parses `json` (compact, members in declaration order) and requires it written back
    /// byte for byte.
    fn round_trip<T: facet::Facet<'static>>(json: &str) {
        let value: T = parse(json);
        assert_eq!(write(&value), json);
    }

    #[test]
    fn scripts_are_written_back_as_read() {
        round_trip::<Scripts>(
            r#"{"variables":{"variable.a":"public"},"initialize":["variable.x = 1;"],"scaleX":"2.0","scalex":"3.0","animate":["walk",{"idle":"query.is_baby"}]}"#,
        );
        round_trip::<Scripts>("{}");
        round_trip::<history::v1_10_0::Scripts>(
            r#"{"pre_animation":["variable.x = 1;",2,true],"should_update_effects_offscreen":"1.0"}"#,
        );
        round_trip::<history::v1_8_0::Scripts>(r#"{"scale":0.5,"parent_setup":"variable.p = 1;"}"#);
    }

    #[test]
    fn a_spawn_egg_is_written_back_as_read() {
        round_trip::<SpawnEgg>(
            r##"{"texture":"spawn_egg","texture_index":3,"base_color":"#ff0000","overlay_color":"red"}"##,
        );
        round_trip::<SpawnEgg>("{}");
    }
}

mod strict_types {
    use bedrock_addon_core::json::{Indent, WriteStyle, write_json};

    use super::*;

    /// Vanilla's strict schema fails a member of the wrong JSON type ("unknown child schema
    /// option type"); facet coerces `"3"` to 3 and `5` to `"5"`, so the members need
    /// `core::json`'s strict wrappers.
    #[test]
    fn wrong_json_types_are_rejected() {
        for json in [
            r#"{ "texture_index": "3" }"#,
            r#"{ "texture_index": 3.5 }"#,
            r#"{ "base_color": 5 }"#,
            r#"{ "texture": 5 }"#,
        ] {
            assert!(facet_json::from_str::<SpawnEgg>(json).is_err(), "{json}");
        }
        for json in [
            r#"{ "initialize": [5] }"#,
            r#"{ "pre_animation": [true] }"#,
            r#"{ "variables": { "variable.a": 1 } }"#,
        ] {
            assert!(facet_json::from_str::<Scripts>(json).is_err(), "{json}");
        }
        // The description members the resolver borrows (`DescriptionSlots`).
        for json in [
            r#"{ "identifier": "demo:ghost", "queryable_geometry": 5 }"#,
            r#"{ "identifier": "demo:ghost", "enable_attachables": "true" }"#,
            r#"{ "identifier": "demo:ghost", "held_item_ignores_lighting": 1 }"#,
            r#"{ "identifier": "demo:ghost", "hide_armor": "false" }"#,
            r#"{ "identifier": "demo:ghost", "held_item_scale": [1.0] }"#,
            r#"{ "identifier": "demo:ghost", "held_item_scale": { "expression": "1.0", "version": 13 } }"#,
        ] {
            assert!(facet_json::from_str::<Description>(json).is_err(), "{json}");
        }
    }

    #[test]
    fn held_item_scale_is_a_number_or_molang() {
        let read = |json: &str| facet_json::from_str::<HeldItemScale>(json);
        assert_eq!(read("2").unwrap(), HeldItemScale::Number(2.0));
        assert_eq!(read("0.5").unwrap(), HeldItemScale::Number(0.5));
        assert!(matches!(
            read(r#""query.is_on_screen ? 1.5 : 1.0""#).unwrap(),
            HeldItemScale::Molang(Molang::Expr(source)) if source.as_str() == "query.is_on_screen ? 1.5 : 1.0"
        ));
        // "Any other non-null value is compiled as Molang", as the game does.
        assert_eq!(
            read("true").unwrap(),
            HeldItemScale::Molang(Molang::Bool(true))
        );
        assert!(read("null").is_err());
        for json in ["2.0", "0.5", r#""1 +""#, "false"] {
            let scale: HeldItemScale = parse(json);
            assert_eq!(write_back(&scale), json, "written back as read");
        }
    }

    /// A number is written as the shortest decimal that reads back to its `f32`, not as the
    /// `f32` widened to a double: vanilla's sulfur cube has 2.678.
    #[test]
    fn a_held_item_scale_is_written_as_the_shortest_number_of_its_f32() {
        for (json, written) in [
            ("2.678", "2.678"),
            ("0.1", "0.1"),
            ("1.1", "1.1"),
            ("3", "3.0"),
            ("1e-3", "0.001"),
            ("16777217", "16777216.0"),
            ("0.30000001192092896", "0.3"),
        ] {
            let scale: HeldItemScale = parse(json);
            assert_eq!(write_back(&scale), written, "{json}");
            // The written number reads back to the same value.
            assert_eq!(parse::<HeldItemScale>(&write_back(&scale)), scale, "{json}");
        }
        // A constant built as Molang is the same number.
        let constant = HeldItemScale::Molang(Molang::Const(2.678));
        assert_eq!(write_back(&constant), "2.678");
        // What is no number is not written.
        for bad in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
            let mut out = Vec::new();
            let style = WriteStyle {
                indent: Indent::Compact,
                trailing_newline: false,
            };
            assert!(
                bedrock_addon_core::json::write_json(&HeldItemScale::Number(bad), &mut out, style)
                    .is_err(),
                "{bad}"
            );
        }
    }

    /// The crate's write-back (`json::write_json`), compact and without a newline.
    fn write_back(scale: &HeldItemScale) -> String {
        let style = WriteStyle {
            indent: Indent::Compact,
            trailing_newline: false,
        };
        let mut out = Vec::new();
        write_json(scale, &mut out, style).unwrap();
        String::from_utf8(out).unwrap()
    }

    /// `HeldItemScale::Number(f32)` has no room for the token kind: an integer token is
    /// written back as the shortest real text of the same value, which vanilla reads as the same
    /// scale (a number takes the "> 0" path whatever its token kind).
    #[test]
    fn held_item_scale_writes_an_integer_token_as_a_real_deviation() {
        let scale: HeldItemScale = parse("2");
        assert_eq!(write_back(&scale), "2.0");
        assert_eq!(parse::<HeldItemScale>(&write_back(&scale)), scale);
    }
}
