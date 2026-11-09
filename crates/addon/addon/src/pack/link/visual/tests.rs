//! The unit tests of the visual group: what a test through the public API cannot see (the
//! reading of a render-controller expression, and how the cost of the group grows).

#[cfg(all(
    feature = "render_controller",
    feature = "material",
    any(
        feature = "client_entity",
        feature = "atomic_client_entity",
        feature = "attachable"
    )
))]
#[test]
fn a_plain_reference_is_the_prefix_and_a_name_and_nothing_else() {
    use super::plain_reference;
    // Both sides of the prefix: any case, and no other prefix.
    assert_eq!(
        plain_reference("geometry.default", "geometry."),
        Some("default".to_owned())
    );
    assert_eq!(
        plain_reference(" Geometry.Baby.Body_2 ", "geometry."),
        Some("baby.body_2".to_owned())
    );
    assert_eq!(plain_reference("texture.default", "geometry."), None);
    assert_eq!(plain_reference("geometry", "geometry."), None);
    // A name, and nothing after or before it.
    assert_eq!(plain_reference("geometry.", "geometry."), None);
    assert_eq!(plain_reference("geometry.a ? 1 : 0", "geometry."), None);
    assert_eq!(plain_reference("array.geos[0]", "geometry."), None);
    assert_eq!(
        plain_reference("q.x ? geometry.a : geometry.b", "geometry."),
        None
    );
    assert_eq!(plain_reference("geometry.a[0]", "geometry."), None);
    assert_eq!(plain_reference("", "geometry."), None);
    // A text shorter than the prefix, or cut inside a character.
    assert_eq!(plain_reference("geo", "geometry."), None);
    assert_eq!(plain_reference("geometrü.a", "geometry."), None);
}

/// The cost of the group, on a world that every rule of it reads.
#[cfg(all(
    feature = "entity",
    feature = "attachable",
    feature = "client_entity",
    feature = "atomic_client_entity",
    feature = "item",
    feature = "render_controller",
    feature = "material",
    feature = "geometry",
    feature = "animation",
    feature = "animation_controller",
    feature = "particle",
    feature = "texture_atlas",
    feature = "flipbooks",
    feature = "texture_list",
    feature = "texture_set",
    feature = "texture_file",
    feature = "sound_definitions",
    feature = "sounds_json"
))]
mod cost {
    use bedrock_addon_core::load::LoadOptions;

    use crate::pack::StackSet;
    use crate::pack::link::tests::support::{leak, pack, set_of};
    use crate::pack::link::{Findings, LinkCx, LinkOptions};

    /// What the whole group tells of `set`.
    fn told(set: &StackSet) -> usize {
        let options = LinkOptions {
            lints: true,
            ..Default::default()
        };
        let cx = LinkCx::new(set, options);
        let mut out = Findings::new(options);
        crate::pack::link::visual::run(&cx, &mut out);
        out.finish().len()
    }

    /// A world of `n` client entities, `n` attachables and `n` of each kind of definition they
    /// name. Every description names a material (of one file that holds all `n`; a material hit
    /// is the lookup whose cost is not one probe), a geometry, an animation of its own with two
    /// effects, a particle and a render controller of its own; every other one of the names is
    /// defined. One animation and one render controller more are used by every description:
    /// the animation has `n` effects of one key and the controller an array of `n` elements
    /// of one slot, all of which every user has (the shape in which asking every user about
    /// every member is `n` times `n`).
    #[allow(clippy::too_many_lines, reason = "one world, built in one place")]
    fn world(n: usize) -> StackSet {
        use std::fmt::Write as _;

        let mut files: Vec<(&'static str, Option<&'static str>)> = Vec::new();
        let mut add = |path: String, text: String| {
            let path: &'static str = Box::leak(path.into_boxed_str());
            let text = std::str::from_utf8(leak(&text)).unwrap();
            files.push((path, Some(text)));
        };
        let mut materials = String::from(r#"{"materials": {"version": "1.0.0""#);
        let mut geometries =
            String::from(r#"{"format_version": "1.12.0", "minecraft:geometry": ["#);
        let mut animations = String::from(
            r#"{"format_version": "1.8.0", "animations": {"animation.shared": {"particle_effects": {"0.0": {"effect": "spark"}}, "sound_effects": {"0.0": {"effect": "nowhere"}}}"#,
        );
        let mut controllers = String::from(
            r#"{"format_version": "1.8.0", "render_controllers": {"controller.render.shared": {"geometry": "Geometry.default", "materials": [{"*": "Material.default"}], "textures": ["Texture.default", "Texture.missing"]}"#,
        );
        for i in 0..n {
            // Every other name is one nothing defines.
            let named = if i % 2 == 0 { i } else { i + n };
            write!(
                materials,
                r#", "mat_{i}": {{"vertexShader": "shaders/entity.vertex"}}"#
            )
            .unwrap();
            let comma = if i == 0 { "" } else { ", " };
            write!(
                geometries,
                r#"{comma}{{"description": {{"identifier": "geometry.g_{i}", "texture_width": 16, "texture_height": 16}}, "bones": [{{"name": "root"}}]}}"#
            )
            .unwrap();
            write!(
                animations,
                r#", "animation.a_{i}": {{"particle_effects": {{"0.0": {{"effect": "spark"}}}}, "sound_effects": {{"0.0": [{{"effect": "step"}}, {{"effect": "s_{named}"}}]}}}}"#
            )
            .unwrap();
            write!(
                controllers,
                r#", "controller.render.c_{i}": {{"geometry": "Geometry.default", "materials": [{{"*": "Material.default"}}], "textures": ["Texture.default", "Texture.t_{named}"]}}"#
            )
            .unwrap();
            let members = format!(
                r#""materials": {{"default": "Mat_{named}"}}, "textures": {{"default": "textures/entity/x", "t_{i}": "textures/entity/x"}}, "geometry": {{"default": "Geometry.G_{named}"}}, "animations": {{"own": "animation.a_{named}", "shared": "animation.shared", "many": "animation.many"}}, "particle_effects": {{"spark": "demo:p_{named}"}}, "sound_effects": {{"step": "demo.step", "s_{i}": "demo.step"}}, "render_controllers": ["controller.render.c_{named}", "controller.render.shared", "controller.render.many"]"#
            );
            add(
                format!("entity/e_{i}.entity.json"),
                format!(
                    r#"{{"format_version": "1.10.0", "minecraft:client_entity": {{"description": {{"identifier": "demo:e_{i}", {members}}}}}}}"#
                ),
            );
            add(
                format!("attachables/a_{i}.json"),
                format!(
                    r#"{{"format_version": "1.10.0", "minecraft:attachable": {{"description": {{"identifier": "demo:a_{i}", "item": "demo:item_{i}", {members}}}}}}}"#
                ),
            );
            add(
                format!("particles/p_{i}.json"),
                format!(
                    r#"{{"format_version": "1.10.0", "particle_effect": {{"description": {{"identifier": "demo:p_{i}", "basic_render_parameters": {{"material": "particles_alpha", "texture": "textures/particle/particles"}}}}, "components": {{"minecraft:emitter_rate_instant": {{"num_particles": 1}}, "minecraft:emitter_lifetime_once": {{"active_time": 1}}}}}}}}"#
                ),
            );
        }
        // The shared definitions with n members, each the same name.
        let mut effects = String::new();
        let mut elements = String::new();
        for i in 0..n {
            let comma = if i == 0 { "" } else { ", " };
            write!(effects, r#"{comma}"{i}.0": {{"effect": "spark"}}"#).unwrap();
            write!(elements, r#"{comma}"Texture.default""#).unwrap();
        }
        write!(
            animations,
            r#", "animation.many": {{"particle_effects": {{{effects}}}}}"#
        )
        .unwrap();
        write!(
            controllers,
            r#", "controller.render.many": {{"arrays": {{"textures": {{"Array.all": [{elements}]}}}}, "geometry": "Geometry.default", "textures": ["Array.all[0]"]}}"#
        )
        .unwrap();
        materials.push_str("}}");
        geometries.push_str("]}");
        animations.push_str("}}");
        controllers.push_str("}}");
        add("materials/entity.material".to_owned(), materials);
        add(
            "materials/fancy.json".to_owned(),
            r#"[{"path": "materials/entity.material"}]"#.to_owned(),
        );
        add("models/entity/all.geo.json".to_owned(), geometries);
        add("animations/all.animation.json".to_owned(), animations);
        add(
            "render_controllers/all.render_controllers.json".to_owned(),
            controllers,
        );
        add("textures/entity/x.png".to_owned(), String::new());
        let set = set_of(
            vec![pack(1, "data", &[])],
            vec![pack(2, "resources", &files)],
            &LoadOptions::default(),
        );
        // The collections load on their first use: before the clock.
        let _ = told(&set);
        set
    }

    /// The cost of the group: `n` client entities and `n` attachables that each name one of `n`
    /// materials of one file (the check asks twice per name: the name and its skinning variant),
    /// geometries, animations, particles and render controllers, half of them missing, and that
    /// all use one animation and one render controller more, against four times as many.
    #[test]
    fn the_group_over_n_descriptions_and_n_definitions_costs_n() {
        crate::test_cost::assert_linear_at(
            "link visual: n descriptions among n definitions",
            1_500,
            world,
            |set: &StackSet| {
                let found = told(set);
                // Each description has findings of several rules: the rules ran.
                let descriptions = set.resource.client_entities().len();
                assert!(found > 4 * descriptions, "{found} for {descriptions}");
                found
            },
        );
    }

    /// What the cost test's world is made of loads, and the rules it means to exercise speak.
    #[test]
    fn the_world_of_the_cost_test_loads_and_every_rule_of_it_speaks() {
        use bedrock_addon_core::diagnostic::DiagnosticDetail;
        use std::collections::BTreeMap;

        let set = world(4);
        assert_eq!(set.resource.client_entities().len(), 4);
        assert_eq!(set.resource.attachables().len(), 4);
        assert_eq!(set.resource.geometries().len(), 4);
        assert_eq!(set.resource.animations().len(), 6);
        assert_eq!(set.resource.render_controllers().len(), 6);
        assert_eq!(set.resource.particles().len(), 4);
        let options = LinkOptions {
            lints: true,
            ..Default::default()
        };
        let cx = LinkCx::new(&set, options);
        let mut out = Findings::new(options);
        crate::pack::link::visual::run(&cx, &mut out);
        let mut by_kind: BTreeMap<(&str, &str), usize> = BTreeMap::new();
        for finding in &out.finish() {
            let kind = match finding.detail.as_deref() {
                Some(DiagnosticDetail::Reference { kind, .. }) => *kind,
                _ => "",
            };
            *by_kind.entry((finding.code.id(), kind)).or_default() += 1;
        }
        let expected: BTreeMap<(&str, &str), usize> = [
            // Of 4 client entities and 4 attachables, 2 each name what nothing defines.
            (("core/unresolved_reference_worded", "material"), 4),
            (("core/unresolved_reference", "geometry"), 4),
            (("core/unresolved_reference", "animation"), 4),
            (("core/unresolved_reference_worded", "particle"), 4),
            (("core/unresolved_reference", "render controller"), 4),
            // No behaviour pack defines the entities or the items.
            (("core/unresolved_reference", "entity"), 4),
            (("core/unresolved_reference", "item"), 4),
            // `demo.step` is no sound event (client entities only: 2 keys each).
            (("core/unknown_sound", "sound event"), 8),
            // The shared animation's `nowhere`, and of the two own animations in use nothing:
            // each is played by the one entity that has its key.
            (("core/unresolved_reference", "sound effect"), 1),
            // The shared controller's `Texture.missing`; an own controller's `t_<i>` is its
            // user's.
            (("core/unresolved_reference", "texture slot"), 1),
        ]
        .into_iter()
        .collect();
        assert_eq!(by_kind, expected);
    }

    /// A world of one resource pack with the render-controller file `controllers` and the
    /// client entities `entities` (identifier suffix and description members), linked once
    /// so that every collection is loaded.
    fn controllers_world(controllers: String, entities: &[(usize, String)]) -> StackSet {
        let mut files: Vec<(&'static str, Option<&'static str>)> = Vec::new();
        let mut add = |path: String, text: String| {
            let path: &'static str = Box::leak(path.into_boxed_str());
            let text = std::str::from_utf8(leak(&text)).unwrap();
            files.push((path, Some(text)));
        };
        add(
            "render_controllers/all.render_controllers.json".to_owned(),
            controllers,
        );
        for (i, members) in entities {
            add(
                format!("entity/e_{i}.entity.json"),
                format!(
                    r#"{{"format_version": "1.10.0", "minecraft:client_entity": {{"description": {{"identifier": "demo:e_{i}", {members}}}}}}}"#
                ),
            );
        }
        let set = set_of(
            vec![pack(1, "data", &[])],
            vec![pack(2, "resources", &files)],
            &LoadOptions::default(),
        );
        let _ = told(&set);
        set
    }

    /// The slots every user of the worlds below has.
    const SLOTS: &str = r#""materials": {"default": "m"}, "textures": {"default": "textures/x"}, "geometry": {"default": "geometry.x"}"#;

    /// One file of `n` controllers, each used by one client entity and each with one finding.
    fn many_controllers(n: usize) -> StackSet {
        use std::fmt::Write as _;
        let mut controllers =
            String::from(r#"{"format_version": "1.8.0", "render_controllers": {"#);
        let mut entities = Vec::new();
        for i in 0..n {
            let comma = if i == 0 { "" } else { ", " };
            write!(
                controllers,
                r#"{comma}"Controller.Render.C_{i}": {{"geometry": "Geometry.nope", "textures": ["Texture.default"]}}"#
            )
            .unwrap();
            entities.push((
                i,
                format!(r#"{SLOTS}, "render_controllers": ["controller.render.c_{i}"]"#),
            ));
        }
        controllers.push_str("}}");
        controllers_world(controllers, &entities)
    }

    /// One controller with `n` material entries, each a slot of its own that its user lacks.
    fn many_material_slots(n: usize) -> StackSet {
        use std::fmt::Write as _;
        let mut entries = String::new();
        for i in 0..n {
            let comma = if i == 0 { "" } else { ", " };
            write!(entries, r#"{comma}{{"Bone_{i}": "Material.M_{i}"}}"#).unwrap();
        }
        let controllers = format!(
            r#"{{"format_version": "1.8.0", "render_controllers": {{"controller.render.one": {{"geometry": "Geometry.default", "textures": ["Texture.default"], "materials": [{entries}]}}}}}}"#
        );
        let user = format!(r#"{SLOTS}, "render_controllers": ["controller.render.one"]"#);
        controllers_world(controllers, &[(0, user)])
    }

    /// One controller with `n` arrays, each with one element its user lacks.
    fn many_arrays(n: usize) -> StackSet {
        use std::fmt::Write as _;
        let mut arrays = String::new();
        for i in 0..n {
            let comma = if i == 0 { "" } else { ", " };
            write!(
                arrays,
                r#"{comma}"Array.A_{i}": ["Texture.default", "Texture.nope"]"#
            )
            .unwrap();
        }
        let controllers = format!(
            r#"{{"format_version": "1.8.0", "render_controllers": {{"controller.render.one": {{"arrays": {{"textures": {{{arrays}}}}}, "geometry": "Geometry.default", "textures": ["Texture.default"]}}}}}}"#
        );
        let user = format!(r#"{SLOTS}, "render_controllers": ["controller.render.one"]"#);
        controllers_world(controllers, &[(0, user)])
    }

    /// The step of the three tests below: the group tells at least `n` findings of the
    /// render-controller rule.
    fn told_each(set: &StackSet) -> usize {
        let found = told(set);
        let members = set
            .resource
            .render_controllers()
            .iter()
            .map(|(_, document)| {
                let controller = document.value();
                1 + controller.materials.len()
                    + controller
                        .arrays
                        .as_ref()
                        .map_or(0, |arrays| arrays.textures.len())
            })
            .sum::<usize>();
        assert!(
            found + 2 >= members,
            "{found} findings for {members} members"
        );
        found
    }

    /// A finding costs its own place, not the file: `n` controllers of one file, each with a
    /// finding, against four times as many.
    #[test]
    fn n_controllers_of_one_file_with_a_finding_each_cost_n() {
        crate::test_cost::assert_linear_at(
            "link visual: n controllers of one file, a finding each",
            COUNT,
            many_controllers,
            told_each,
        );
    }

    /// `n` material entries of one controller, each a slot its user lacks.
    #[test]
    fn n_lacking_material_slots_of_one_controller_cost_n() {
        crate::test_cost::assert_linear_at(
            "link visual: n lacking material slots of one controller",
            COUNT,
            many_material_slots,
            told_each,
        );
    }

    /// `n` arrays of one controller, each with an element its user lacks.
    #[test]
    fn n_arrays_of_one_controller_with_a_lacking_element_each_cost_n() {
        crate::test_cost::assert_linear_at(
            "link visual: n arrays of one controller, a lacking element each",
            COUNT,
            many_arrays,
            told_each,
        );
    }

    /// The size of the smaller world of the three tests above.
    const COUNT: usize = 2_000;
}
