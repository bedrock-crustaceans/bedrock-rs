//! The steps of `families::client_biome::history`: the steps key on the identifier, so each test
//! runs its step for other biomes than the step's base.

use bedrock_addon::diagnostic::codes;
use bedrock_addon::families::client_biome::history::{
    v1_21_40, v1_21_70, v1_21_90, v1_21_100, v1_21_110, v1_21_120,
};
use bedrock_addon::version::FormatVersion;
use bedrock_addon::versioning::testing::{Checks, Run, StepOutput, step_output};
use facet_value::Value;

use super::common::{biome, json};

/// The step of `version` run on a document of `identifier` with `components`.
fn run(version: &str, identifier: &str, components: &str) -> StepOutput {
    let parts: Vec<u16> = version.split('.').map(|p| p.parse().unwrap()).collect();
    let settings = Run {
        original_version: Some(FormatVersion::new(parts[0], parts[1], parts[2])),
        skip_section_a: false,
    };
    let input = json(&biome(version, identifier, components));
    let checks = Checks::default();
    match version {
        "1.21.40" => step_output::<v1_21_40::ClientBiome>(&input, settings, &checks),
        "1.21.70" => step_output::<v1_21_70::ClientBiome>(&input, settings, &checks),
        "1.21.90" => step_output::<v1_21_90::ClientBiome>(&input, settings, &checks),
        "1.21.100" => step_output::<v1_21_100::ClientBiome>(&input, settings, &checks),
        "1.21.110" => step_output::<v1_21_110::ClientBiome>(&input, settings, &checks),
        "1.21.120" => step_output::<v1_21_120::ClientBiome>(&input, settings, &checks),
        other => panic!("no step at {other}"),
    }
    .unwrap_or_else(|failure| panic!("the step at {version} runs: {failure:?}"))
}

/// The component `name` of a step's output (a whole document).
fn output_component(output: &StepOutput, name: &str) -> Option<Value> {
    output
        .payload
        .as_object()?
        .get("minecraft:client_biome")?
        .as_object()?
        .get("components")?
        .as_object()?
        .get(name)
        .cloned()
}

/// The identifier of a step's output.
fn output_identifier(output: &StepOutput) -> Option<String> {
    let identifier = output
        .payload
        .as_object()?
        .get("minecraft:client_biome")?
        .as_object()?
        .get("description")?
        .as_object()?
        .get("identifier")?;
    Some(identifier.as_string()?.as_str().to_owned())
}

#[test]
fn the_1_21_40_step_inserts_the_dry_foliage_colors_of_its_table() {
    let name = "minecraft:dry_foliage_color";
    let pale = run("1.21.40", "minecraft:pale_garden", "");
    assert!(
        output_component(&pale, name) == Some(json(r##"{"color": "#a0a69c"}"##)),
        "the pale garden's colour",
    );
    let forest = run("1.21.40", "minecraft:roofed_forest", "");
    assert!(
        output_component(&forest, name) == Some(json(r##"{"color": "#7b5334"}"##)),
        "the dark forest's colour",
    );
    let custom = run("1.21.40", "test:swampland", "");
    assert!(
        output_component(&custom, name).is_none(),
        "a custom biome gets none",
    );
}

#[test]
fn the_1_21_40_step_misses_the_tables_for_a_bare_identifier() {
    let bare = run("1.21.40", "swampland", "");
    assert!(
        output_component(&bare, "minecraft:foliage_appearance").is_none()
            && output_component(&bare, "minecraft:grass_appearance").is_none()
            && output_component(&bare, "minecraft:dry_foliage_color").is_none(),
        "a bare identifier gets none of the three",
    );
    assert!(
        output_identifier(&bare).as_deref() == Some("swampland"),
        "the identifier is not namespaced by this step",
    );
    let missed = bare
        .diagnostics
        .iter()
        .filter(|d| d.code.is(&codes::UPGRADE_TABLE_MISSED_BARE_IDENTIFIER))
        .count();
    assert!(missed == 3, "each missed table is reported");
}

#[test]
fn the_1_21_40_step_inserts_the_foliage_and_grass_tables() {
    let mesa = run("1.21.40", "minecraft:mesa", "");
    assert!(
        output_component(&mesa, "minecraft:foliage_appearance")
            == Some(json(r##"{"color": "#aea42a"}"##))
            && output_component(&mesa, "minecraft:grass_appearance")
                == Some(json(r##"{"color": "#90814d"}"##)),
        "the mesa's colours",
    );
    let plateau = run("1.21.40", "minecraft:mesa_plateau", "");
    assert!(
        output_component(&plateau, "minecraft:foliage_appearance").is_none(),
        "the mesa plateau is not in the table",
    );
}

#[test]
fn the_1_21_70_step_namespaces_a_bare_identifier() {
    let bare = run("1.21.70", "glade", "");
    assert!(
        output_identifier(&bare).as_deref() == Some("minecraft:glade"),
        "a bare identifier gets `minecraft:`",
    );
    assert!(
        bare.diagnostics
            .iter()
            .any(|d| d.code.is(&codes::IDENTIFIER_NAMESPACED)),
        "and the step says so",
    );
    let kept = run("1.21.70", "myns:glade", "");
    assert!(
        output_identifier(&kept).as_deref() == Some("myns:glade")
            && !kept
                .diagnostics
                .iter()
                .any(|d| d.code.is(&codes::IDENTIFIER_NAMESPACED)),
        "a namespaced identifier is kept",
    );
    // The inserts after it see the namespaced identifier.
    let cherry = run("1.21.70", "cherry_grove", "");
    assert!(
        output_component(&cherry, "minecraft:biome_music")
            == Some(json(r#"{"music_definition": "cherry_grove"}"#)),
        "the music table matches the namespaced identifier",
    );
}

#[test]
fn the_1_21_70_step_inserts_the_music_table() {
    let name = "minecraft:biome_music";
    let end = run("1.21.70", "minecraft:the_end", "");
    assert!(
        output_component(&end, name) == Some(json(r#"{"music_definition": "end"}"#)),
        "the End's definition is `end`",
    );
    let plains = run("1.21.70", "minecraft:plains", "");
    assert!(
        output_component(&plains, name).is_none(),
        "plains has no row",
    );
    let kept = run(
        "1.21.70",
        "minecraft:forest",
        r#""minecraft:biome_music": {"music_definition": "mine"}"#,
    );
    assert!(
        output_component(&kept, name) == Some(json(r#"{"music_definition": "mine"}"#)),
        "an existing definition is kept",
    );
}

#[test]
fn the_1_21_90_step_shades_the_grass_of_the_dark_forest() {
    let name = "minecraft:grass_appearance";
    let present = run(
        "1.21.90",
        "minecraft:roofed_forest",
        r##""minecraft:grass_appearance": {"color": "#507a32"}"##,
    );
    assert!(
        output_component(&present, name)
            == Some(json(r##"{"color": "#507a32", "grass_is_shaded": true}"##)),
        "an existing component is shaded",
    );
    let absent = run("1.21.90", "minecraft:roofed_forest", "");
    assert!(
        output_component(&absent, name) == Some(json(r#"{"grass_is_shaded": true}"#)),
        "without the component it is inserted",
    );
    let mutated = run("1.21.90", "minecraft:roofed_forest_mutated", "");
    assert!(
        output_component(&mutated, name).is_none(),
        "the mutated forest is not shaded",
    );
}

#[test]
fn the_1_21_100_step_inserts_the_precipitation_of_the_nether_biomes() {
    let name = "minecraft:precipitation";
    for (identifier, expected) in [
        ("minecraft:basalt_deltas", r#"{"white_ash": 2.0}"#),
        ("minecraft:crimson_forest", r#"{"red_spores": 0.25}"#),
        ("minecraft:soulsand_valley", r#"{"ash": 0.05}"#),
    ] {
        let output = run("1.21.100", identifier, "");
        assert!(
            output_component(&output, name) == Some(json(expected)),
            "a nether biome gets its one particle",
        );
    }
    let other = run("1.21.100", "minecraft:hell", "");
    assert!(
        output_component(&other, name).is_none(),
        "the wastes get none",
    );
}

#[test]
fn the_1_21_100_step_sets_underwater_music_for_the_waters() {
    let name = "minecraft:biome_music";
    let ocean = run("1.21.100", "minecraft:ocean", "");
    assert!(
        output_component(&ocean, name) == Some(json(r#"{"underwater_music": true}"#)),
        "an ocean without music gets the component",
    );
    let river = run(
        "1.21.100",
        "minecraft:frozen_river",
        r#""minecraft:biome_music": {"volume_multiplier": 0.5}"#,
    );
    assert!(
        output_component(&river, name)
            == Some(json(
                r#"{"volume_multiplier": 0.5, "underwater_music": true}"#,
            )),
        "an existing component is set",
    );
    let land = run("1.21.100", "minecraft:plains", "");
    assert!(output_component(&land, name).is_none(), "land gets none");
}

#[test]
fn the_1_21_110_step_converts_the_ambient_sounds() {
    let name = "minecraft:ambient_sounds";
    let custom = run(
        "1.21.110",
        "myns:glade",
        r#""minecraft:ambient_sounds": {"addition": "a", "loop": "l", "mood": "m"}"#,
    );
    assert!(
        output_component(&custom, name)
            == Some(json(
                r#"{"addition": {"asset": "a", "chance": 0.0111}, "loop": "l", "mood": "m",
                    "underwater_addition": {"asset": "a", "chance": 0.0111},
                    "underwater_loop": "l", "underwater_mood": "m"}"#,
            )),
        "a custom biome's sounds also play under water",
    );
    let empty = run(
        "1.21.110",
        "myns:glade",
        r#""minecraft:ambient_sounds": {"addition": "", "mood": "m"}"#,
    );
    assert!(
        output_component(&empty, name) == Some(json(r#"{"mood": "m", "underwater_mood": "m"}"#)),
        "an empty addition stays absent",
    );
}

#[test]
fn the_1_21_110_step_adds_the_underwater_ambience_of_the_vanilla_biomes() {
    let name = "minecraft:ambient_sounds";
    let present = run(
        "1.21.110",
        "minecraft:river",
        r#""minecraft:ambient_sounds": {"addition": "x", "loop": "y"}"#,
    );
    assert!(
        output_component(&present, name)
            == Some(json(
                r#"{"addition": {"asset": "x", "chance": 0.0111}, "loop": "y",
                    "underwater_addition": {"asset": "ambient.underwater.additions", "chance": 0.01},
                    "underwater_loop": "ambient.underwater.loop"}"#,
            )),
        "a vanilla biome's underwater sounds are the game's",
    );
    let custom = run("1.21.110", "myns:glade", "");
    assert!(
        output_component(&custom, name).is_none(),
        "a custom biome without the component gets none",
    );
    // `minecraftx:` is not the vanilla namespace.
    let near = run("1.21.110", "minecraftx:river", "");
    assert!(
        output_component(&near, name).is_none(),
        "only the `minecraft` namespace is vanilla",
    );
}

/// A component a struct holds is never replaced by a step's insertion ("addMissing" means
/// absent only), also where no document of the struct's version can write the component: the
/// steps are run here on structs built by hand.
mod absent_only {
    use bedrock_addon::diagnostic::Diagnostics;
    use bedrock_addon::document::{ComponentMap, ComponentRef};
    use bedrock_addon::families::client_biome::component::AmbientSounds;
    use bedrock_addon::families::client_biome::component::{
        AtmosphereIdentifier, ClientBiomeComponent, ColorGradingIdentifier, DryFoliageColor,
        Precipitation, WaterIdentifier,
    };
    use bedrock_addon::families::client_biome::history::{
        ClientBiomePayload, v1_21_40, v1_21_70, v1_21_100, v1_21_110,
    };
    use bedrock_addon::families::client_biome::{Color255Rgb, Description};
    use bedrock_addon::id::Id;
    use bedrock_addon::load::LoadContext;
    use bedrock_addon::vanilla::ActorType;
    use bedrock_addon::version::FormatVersion;
    use bedrock_addon::versioning::{Legacy, Upgrade, UpgradeCx};

    /// A map holding `components`, as the author's.
    fn map(components: Vec<ClientBiomeComponent>) -> ComponentMap<v1_21_40::Component> {
        let mut map = ComponentMap::new();
        for component in components {
            map.insert_typed(v1_21_40::Component::from(component), false);
        }
        map
    }

    /// Runs `step` for a document of `version`.
    fn run<S: Upgrade>(step: S, version: (u16, u16, u16)) -> (S::Next, Diagnostics) {
        let load = LoadContext::detached();
        let mut diagnostics = Diagnostics::default();
        let next = step.upgrade(&mut UpgradeCx {
            load: &load,
            original_version: FormatVersion::new(version.0, version.1, version.2),
            identifier: "",
            actor_type: ActorType::Mob,
            skip_section_a: false,
            reading: load.reading,
            diagnostics: &mut diagnostics,
        });
        (next, diagnostics)
    }

    /// The typed entries of a map: `(name, inserted by a step)`.
    fn entries(map: &ComponentMap<v1_21_40::Component>) -> Vec<(String, bool)> {
        map.iter()
            .filter_map(|entry| match entry {
                ComponentRef::Typed {
                    name, synthesized, ..
                } => Some((name.to_owned(), synthesized)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_1_21_70_step_keeps_a_dry_foliage_color() {
        let own = DryFoliageColor::new(Color255Rgb::Hex("#010203".to_owned()));
        let step: v1_21_40::ClientBiome = Legacy(ClientBiomePayload {
            format_version: "1.21.60".to_owned(),
            description: Description::new(Id::parse("minecraft:pale_garden")),
            components: map(vec![own.clone().into()]),
        });
        let (next, diagnostics) = run(step, (1, 21, 60));
        assert_eq!(
            entries(&next.components),
            [("minecraft:dry_foliage_color".to_owned(), false)]
        );
        assert!(diagnostics.is_empty());
        assert_eq!(next.components, map(vec![own.into()]));
    }

    #[test]
    fn the_1_21_90_step_keeps_the_render_identifiers() {
        let own = vec![
            AtmosphereIdentifier::new(Id::parse("mine:a")).into(),
            ColorGradingIdentifier::new(Id::parse("mine:c")).into(),
            WaterIdentifier::new(Id::parse("mine:w")).into(),
        ];
        let step: v1_21_70::ClientBiome = Legacy(ClientBiomePayload {
            format_version: "1.21.60".to_owned(),
            description: Description::new(Id::parse("myns:glade")),
            components: map(own.clone()),
        });
        let (next, _) = run(step, (1, 21, 60));
        assert_eq!(
            entries(&next.components),
            [
                ("minecraft:atmosphere_identifier".to_owned(), false),
                ("minecraft:color_grading_identifier".to_owned(), false),
                ("minecraft:water_identifier".to_owned(), false),
                ("minecraft:lighting_identifier".to_owned(), true),
            ]
        );
        // The author's three are unchanged.
        let mut expected = map(own);
        for entry in next.components.iter() {
            if let ComponentRef::Typed {
                value,
                synthesized: true,
                ..
            } = entry
            {
                expected.insert_typed(value.clone(), true);
            }
        }
        assert_eq!(next.components, expected);
    }

    #[test]
    fn the_1_21_110_step_keeps_a_precipitation() {
        let own = Precipitation {
            ash: Some(9.0),
            ..Precipitation::default()
        };
        let step: v1_21_100::ClientBiome = Legacy(ClientBiomePayload {
            format_version: "1.21.100".to_owned(),
            description: Description::new(Id::parse("minecraft:basalt_deltas")),
            components: map(vec![own.clone().into()]),
        });
        let (next, diagnostics) = run(step, (1, 21, 100));
        assert_eq!(next.components, map(vec![own.into()]));
        assert!(diagnostics.is_empty());
    }

    /// A struct below 1.21.120 built by hand can hold `minecraft:ambient_sounds` in the newest
    /// shape: the 1.21.120 step treats it as the component it is (nothing to convert) and
    /// still gives it the underwater sounds.
    #[test]
    fn the_1_21_120_step_handles_ambient_sounds_in_the_newest_shape() {
        let sounds = AmbientSounds {
            loop_sound: Some(Id::parse("l")),
            ..AmbientSounds::default()
        };
        let step = |identifier: &str| -> v1_21_110::ClientBiome {
            Legacy(ClientBiomePayload {
                format_version: "1.21.110".to_owned(),
                description: Description::new(Id::parse(identifier)),
                components: map(vec![sounds.clone().into()]),
            })
        };
        let (vanilla, _) = run(step("minecraft:river"), (1, 21, 110));
        assert_eq!(vanilla.components.len(), 1);
        let got = vanilla
            .components
            .get::<AmbientSounds>()
            .expect("the component");
        assert_eq!(got.loop_sound(), "l");
        assert_eq!(got.underwater_loop(), "ambient.underwater.loop");
        assert_eq!(
            got.underwater_addition().asset.as_str(),
            "ambient.underwater.additions"
        );
        let (custom, diagnostics) = run(step("myns:glade"), (1, 21, 110));
        let got = custom
            .components
            .get::<AmbientSounds>()
            .expect("the component");
        assert_eq!(got.underwater_loop(), "l");
        assert_eq!(got.underwater_addition().asset.as_str(), "");
        assert_eq!(diagnostics.len(), 1);
    }

    /// The gates whose constant is the next struct's version: at that version (which only a
    /// struct built by hand reaches) the step inserts and changes nothing.
    #[test]
    fn a_step_at_its_before_version_does_nothing() {
        use bedrock_addon::families::client_biome::history::v1_21_90;
        let empty = |identifier: &str| (Description::new(Id::parse(identifier)), map(Vec::new()));
        let (description, components) = empty("minecraft:pale_garden");
        let dry: v1_21_40::ClientBiome = Legacy(ClientBiomePayload {
            format_version: "1.21.70".to_owned(),
            description,
            components,
        });
        assert!(run(dry.clone(), (1, 21, 70)).0.components.is_empty());
        assert_eq!(
            entries(&run(dry, (1, 21, 69)).0.components),
            [("minecraft:dry_foliage_color".to_owned(), true)]
        );
        let (description, components) = empty("minecraft:roofed_forest");
        let shade: v1_21_90::ClientBiome = Legacy(ClientBiomePayload {
            format_version: "1.21.100".to_owned(),
            description,
            components,
        });
        assert!(run(shade.clone(), (1, 21, 100)).0.components.is_empty());
        assert_eq!(run(shade, (1, 21, 99)).0.components.len(), 1);
        for identifier in ["minecraft:basalt_deltas", "minecraft:ocean"] {
            let (description, components) = empty(identifier);
            let step: v1_21_100::ClientBiome = Legacy(ClientBiomePayload {
                format_version: "1.21.110".to_owned(),
                description,
                components,
            });
            assert!(
                run(step.clone(), (1, 21, 110)).0.components.is_empty(),
                "{identifier}"
            );
            assert_eq!(
                run(step, (1, 21, 109)).0.components.len(),
                1,
                "{identifier}"
            );
        }
        let (description, _) = empty("minecraft:river");
        let sounds = AmbientSounds {
            loop_sound: Some(Id::parse("l")),
            ..AmbientSounds::default()
        };
        for components in [map(Vec::new()), map(vec![sounds.clone().into()])] {
            let step: v1_21_110::ClientBiome = Legacy(ClientBiomePayload {
                format_version: "1.21.120".to_owned(),
                description: description.clone(),
                components,
            });
            let (at, diagnostics) = run(step.clone(), (1, 21, 120));
            assert!(
                at.components
                    .get::<AmbientSounds>()
                    .is_none_or(|sounds| sounds.underwater_loop().is_empty())
            );
            assert!(diagnostics.is_empty());
            let (below, _) = run(step, (1, 21, 119));
            assert_eq!(
                below
                    .components
                    .get::<AmbientSounds>()
                    .unwrap()
                    .underwater_loop(),
                "ambient.underwater.loop"
            );
        }
    }
}
