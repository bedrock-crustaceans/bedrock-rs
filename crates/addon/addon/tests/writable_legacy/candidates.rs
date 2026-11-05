// A list of candidate documents: each is loaded, written and read again, and
// what the written text does not have as the file had it is said at the member of the file,
// or the text is the same value and nothing is owed. One test per candidate.

use bedrock_addon::document::Content;
use bedrock_addon::load::LoadContext;
use bedrock_addon::pack::PackType;
use bedrock_addon::storage::PackPath;
use bedrock_addon::version::MinEngineVersion;

fn pack(pack_type: PackType, version: (u16, u16, u16)) -> LoadContext {
    let mut cx = LoadContext::detached();
    cx.pack_type = pack_type;
    cx.min_engine_version = Some(MinEngineVersion::new(version.0, version.1, version.2));
    cx.molang_version = bedrock_addon::molang::pack_molang_version(cx.min_engine_version.as_ref());
    cx
}

/// What loading `text` tells (`Code Severity at path`, the Molang note aside), and the text
/// the document is written as, on one line.
#[track_caller]
fn told<T: Content>(at: &str, cx: &LoadContext, text: &str) -> (Vec<String>, String) {
    let document = T::parse(&PackPath::new(at).unwrap(), text.as_bytes(), cx)
        .unwrap_or_else(|rejected| panic!("rejected: {:?}", rejected.diagnostics));
    let told = document
        .diagnostics()
        .iter()
        .filter(|f| f.code.name() != "MolangNotChecked")
        .map(|f| {
            format!(
                "{} {:?} at {}",
                f.code.name(),
                f.severity,
                f.location.json_path
            )
        })
        .collect();
    let written = document.to_json().expect("written");
    // The written text loads.
    T::parse(&PackPath::new(at).unwrap(), written.as_bytes(), cx)
        .unwrap_or_else(|rejected| panic!("the written text: {:?}", rejected.diagnostics));
    (
        told,
        written.split_whitespace().collect::<Vec<_>>().join(" "),
    )
}

fn fog(air: &str) -> (Vec<String>, String) {
    told::<bedrock_addon::families::fog::FogSettings>(
        "fogs/x.json",
        &pack(PackType::Resource, (1, 21, 0)),
        &format!(
            r##"{{"format_version":"1.16.100","minecraft:fog_settings":{{"description":{{"identifier":"demo:fog"}},"distance":{{"air":{{{air},"fog_color":"#ABD2FF"}}}}}}}}"##
        ),
    )
}

fn geometry(text: &str) -> (Vec<String>, String) {
    told::<bedrock_addon::families::geometry::GeometryFile>(
        "models/entity/x.json",
        &pack(PackType::Resource, (1, 21, 0)),
        text,
    )
}

fn trade(wants: &str, rest: &str) -> (Vec<String>, String) {
    told::<bedrock_addon::families::trade::TradeTable>(
        "trading/x.json",
        &pack(PackType::Behavior, (1, 21, 0)),
        &format!(
            r#"{{"tiers":[{{"trades":[{{"wants":[{wants}],"gives":[{{"item":"minecraft:bread"}}]{rest}}}]}}]}}"#
        ),
    )
}

fn particle(components: &str, rest: &str) -> (Vec<String>, String) {
    told::<bedrock_addon::families::particle::ParticleEffect>(
        "particles/demo.particle.json",
        &pack(PackType::Resource, (1, 21, 100)),
        &format!(
            r#"{{"format_version":"1.10.0","particle_effect":{{"description":{{"identifier":"demo:spark","basic_render_parameters":{{"material":"particles_alpha","texture":"textures/particle/particles"}}}},"components":{{{components}}}{rest}}}}}"#
        ),
    )
}

/// FOG1: `render_distance_type` is matched in any case and held as the type: `RENDER` is
/// written `render`. The game reads both alike; the spelling of the file is noted.
#[test]
fn fog1_a_distance_type_in_another_case_is_noted_at_the_member() {
    let member = "minecraft:fog_settings/distance/air/render_distance_type";
    let (found, written) = fog(r#""fog_start":0.5,"fog_end":1.0,"render_distance_type":"RENDER""#);
    assert_eq!(found, [format!("ValueRewritten Info at {member}")]);
    assert!(
        written.contains(r#""render_distance_type": "render""#),
        "{written}"
    );
    let (found, _) = fog(r#""fog_start":0.5,"fog_end":1.0,"render_distance_type":"render""#);
    assert_eq!(found, [] as [&str; 0]);
}

/// FOG2: the game holds these members as 32-bit floats. A number a float does not hold
/// exactly as written is written as the float: noted at the member. A number whose shortest
/// float spelling is the file's (`0.9`) says nothing.
#[test]
fn fog2_a_number_a_float_does_not_hold_as_written_is_noted_at_the_member() {
    let air = "minecraft:fog_settings/distance/air";
    let (found, written) =
        fog(r#""fog_start":0.123456789012,"fog_end":16777217,"render_distance_type":"fixed""#);
    assert_eq!(
        found,
        [
            format!("LossyConversion Info at {air}/fog_end"),
            format!("LossyConversion Info at {air}/fog_start"),
        ]
    );
    assert!(
        written.contains(r#""fog_start": 0.12345679"#)
            && written.contains(r#""fog_end": 16777216"#),
        "{written}"
    );
    let (found, _) = fog(r#""fog_start":0.9,"fog_end":16777216,"render_distance_type":"fixed""#);
    assert_eq!(found, [] as [&str; 0]);
}

/// GEO7: the same for a geometry's floats.
#[test]
fn geo7_a_number_a_float_does_not_hold_as_written_is_noted_at_the_member() {
    let file = |pivot: &str, inflate: &str| {
        geometry(&format!(
            r#"{{"format_version":"1.12.0","minecraft:geometry":[{{"description":{{"identifier":"geometry.x","texture_width":64,"texture_height":64}},"bones":[{{"name":"b","pivot":[{pivot},0,0],"inflate":{inflate}}}]}}]}}"#
        ))
    };
    let (found, written) = file("16777217", "0.123456789012");
    assert_eq!(
        found,
        [
            "LossyConversion Info at minecraft:geometry/0/bones/0/pivot/0",
            "LossyConversion Info at minecraft:geometry/0/bones/0/inflate",
        ]
    );
    assert!(
        written.contains("16777216") && written.contains("0.12345679"),
        "{written}"
    );
    let (found, _) = file("16777216", "0.1");
    assert_eq!(found, [] as [&str; 0]);
}

/// TRD5: the same for a trade's `price_multiplier`.
#[test]
fn trd5_a_price_multiplier_a_float_does_not_hold_as_written_is_noted() {
    let (found, written) = trade(
        r#"{"item":"minecraft:emerald","quantity":1,"price_multiplier":0.123456789012}"#,
        "",
    );
    assert_eq!(
        found,
        ["LossyConversion Info at tiers/0/trades/0/wants/0/price_multiplier"]
    );
    assert!(
        written.contains(r#""price_multiplier": 0.12345679"#),
        "{written}"
    );
    let (found, _) = trade(
        r#"{"item":"minecraft:emerald","quantity":1,"price_multiplier":0.05}"#,
        "",
    );
    assert_eq!(found, [] as [&str; 0]);
}

/// CENT4: an element of a 1.8.0 `animation_controllers` list without a member moves nothing
/// and goes with the list: noted at the element of the file.
#[test]
fn cent4_a_controller_element_without_a_member_is_noted_where_it_is() {
    let entity = |controllers: &str| {
        told::<bedrock_addon::families::client_entity::ClientEntity>(
            "entity/x.entity.json",
            &pack(PackType::Resource, (1, 21, 0)),
            &format!(
                r#"{{"format_version":"1.8.0","minecraft:client_entity":{{"description":{{"identifier":"demo:ghost","animations":{{"walk":"animation.x.walk"}},"animation_controllers":[{controllers}]}}}}}}"#
            ),
        )
    };
    let (found, written) = entity(r#"{},{"move":"controller.animation.x.move"}"#);
    assert_eq!(
        found,
        ["DroppedField Info at minecraft:client_entity/description/animation_controllers/0"]
    );
    assert!(written.contains("controller__move"), "{written}");
    let (found, _) = entity(r#"{"move":"controller.animation.x.move"}"#);
    assert_eq!(found, [] as [&str; 0]);
}

/// GEO5: the pre-1.8 step removes a cube without a member: noted at the index of the file.
#[test]
fn geo5_a_cube_without_a_member_the_pre_1_8_step_removes_is_noted_where_it_is() {
    let file = |cubes: &str| {
        let (found, written) = geometry(&format!(
            r#"{{"geometry.a":{{"bones":[{{"name":"b","cubes":[{cubes}]}}]}}}}"#
        ));
        let found: Vec<String> = found
            .into_iter()
            .filter(|f| !f.starts_with("MissingFormatVersion"))
            .collect();
        (found, written)
    };
    let first = r#"{"origin":[0,0,0],"size":[1,1,1],"uv":[0,0]}"#;
    let last = r#"{"origin":[1,2,3],"size":[2,2,2],"uv":[8,8]}"#;
    let (found, _) = file(&format!("{{}},{first},{last}"));
    assert_eq!(found, ["DroppedField Info at geometry.a/bones/0/cubes/0"]);
    let (found, _) = file(&format!("{first},{{}},{last}"));
    assert_eq!(found, ["DroppedField Info at geometry.a/bones/0/cubes/1"]);
    let (found, _) = file(&format!("{first},{last}"));
    assert_eq!(found, [] as [&str; 0]);
}

/// GEO6: the pre-1.8 reader takes `mirror` and `reset` written as the texts `"true"` and
/// `"false"`, and the file is written with booleans: noted at each member.
#[test]
fn geo6_a_boolean_written_as_a_text_is_noted_at_the_member() {
    let file = |mirror: &str, reset: &str, cube: &str| {
        let (found, written) = geometry(&format!(
            r#"{{"geometry.a":{{"bones":[{{"name":"b","mirror":{mirror},"reset":{reset},"cubes":[{{"origin":[0,0,0],"size":[1,1,1],"uv":[0,0],"mirror":{cube}}}]}}]}}}}"#
        ));
        let found: Vec<String> = found
            .into_iter()
            .filter(|f| !f.starts_with("MissingFormatVersion"))
            .collect();
        (found, written)
    };
    let (found, written) = file(r#""false""#, r#""true""#, r#""true""#);
    assert_eq!(
        found,
        [
            "ValueRewritten Info at geometry.a/bones/0/reset",
            "ValueRewritten Info at geometry.a/bones/0/mirror",
            "ValueRewritten Info at geometry.a/bones/0/cubes/0/mirror",
        ]
    );
    assert!(
        written.contains(r#""reset": true"#) && !written.contains(r#""true""#),
        "{written}"
    );
    let (found, _) = file("false", "true", "true");
    assert_eq!(found, [] as [&str; 0]);
}

/// TRD1, TRD2: a member written `null` is the absent member to the legacy parser (jsoncpp's
/// null test), and the table is the one of the file without it: the same value, the same
/// text, nothing owed.
#[test]
fn trd1_trd2_a_null_member_is_the_absent_member_and_nothing_is_owed() {
    let with = trade(
        r#"{"item":"minecraft:emerald","quantity":null,"functions":null}"#,
        r#","max_uses":null,"reward_exp":null"#,
    );
    let without = trade(r#"{"item":"minecraft:emerald"}"#, "");
    assert_eq!(with.0, [] as [&str; 0]);
    assert_eq!(with, without);
    // A `groups` that is null beside flat `trades`.
    let cx = pack(PackType::Behavior, (1, 21, 0));
    let tier = |groups: &str| {
        told::<bedrock_addon::families::trade::TradeTable>(
            "trading/x.json",
            &cx,
            &format!(
                r#"{{"tiers":[{{"trades":[{{"wants":[{{"item":"minecraft:emerald","quantity":1}}],"gives":[{{"item":"minecraft:bread"}}]}}]{groups}}}]}}"#
            ),
        )
    };
    let with = tier(r#","groups":null"#);
    assert_eq!(with.0, [] as [&str; 0]);
    assert_eq!(with, tier(""));
}

/// TRD3: below 1.20.30 `biomes` becomes the item's filter; an empty list makes none and
/// goes: noted at the member. TRD4, its control: a list with names is converted, and the
/// conversion is announced at `biomes` (`tests/trading/round_trip.rs`, `k107_`).
#[test]
fn trd3_trd4_an_empty_biomes_list_is_noted_and_a_list_with_names_is_converted() {
    let at = "tiers/0/trades/0/wants/0/biomes";
    let (found, written) = trade(
        r#"{"item":"minecraft:emerald","quantity":1,"biomes":[]}"#,
        "",
    );
    assert_eq!(found, [format!("DroppedField Info at {at}")]);
    assert!(!written.contains("__DEP_fltrs"), "{written}");
    let (found, written) = trade(
        r#"{"item":"minecraft:emerald","quantity":1,"biomes":["plains","plains"]}"#,
        "",
    );
    assert_eq!(found, [format!("ValueRewritten Info at {at}")]);
    assert_eq!(written.matches("is_mark_variant").count(), 2, "{written}");
}

/// TRD6: a quantity written as a text is one of the forms of the legacy quantity; its value
/// is held and written (`{"min": 2, "max": 5}`), and spaces and a sign in the text are its
/// spelling, as white space is anywhere else: the table of `"2-5"`, nothing owed.
#[test]
fn trd6_a_quantity_text_with_spaces_and_a_sign_is_the_range_it_spells() {
    let spelled = trade(r#"{"item":"minecraft:emerald","quantity":" +2 - 5 "}"#, "");
    let plain = trade(r#"{"item":"minecraft:emerald","quantity":"2-5"}"#, "");
    assert_eq!(spelled.0, [] as [&str; 0]);
    assert_eq!(spelled, plain);
    assert!(
        spelled.1.contains(r#""quantity": { "min": 2, "max": 5 }"#),
        "{}",
        spelled.1
    );
}

/// PRT4: the legacy reader takes a boolean at a Molang member, and the effect is written with
/// the number it stands for: noted at the member.
#[test]
fn prt4_a_boolean_at_a_molang_member_is_noted_at_the_member() {
    use bedrock_addon::document::JsonDocument as _;
    let components = "particle_effect/components";
    let (found, written) = particle(
        r#""minecraft:emitter_rate_instant":{"num_particles":true},"minecraft:emitter_lifetime_once":{"active_time":1},"minecraft:particle_initial_speed":false"#,
        "",
    );
    assert_eq!(
        found,
        [
            format!(
                "ValueRewritten Info at {components}/minecraft:emitter_rate_instant/num_particles"
            ),
            format!("ValueRewritten Info at {components}/minecraft:particle_initial_speed"),
        ]
    );
    assert!(
        written.contains(r#""num_particles": 1"#) && !written.contains("true"),
        "{written}"
    );
    let (found, _) = particle(
        r#""minecraft:emitter_rate_instant":{"num_particles":1},"minecraft:emitter_lifetime_once":{"active_time":1},"minecraft:particle_initial_speed":0"#,
        "",
    );
    assert_eq!(found, [] as [&str; 0]);
    // Read as it was written, the effect holds its booleans: nothing is rewritten and
    // nothing is noted.
    let text = r#"{"format_version":"1.10.0","particle_effect":{"description":{"identifier":"demo:spark","basic_render_parameters":{"material":"particles_alpha","texture":"textures/particle/particles"}},"components":{"minecraft:emitter_rate_instant":{"num_particles":true},"minecraft:emitter_lifetime_once":{"active_time":1}}}}"#;
    let as_read = bedrock_addon::families::particle::ParticleEffect::parse_as_written(
        &PackPath::new("particles/demo.particle.json").unwrap(),
        text.as_bytes(),
        &pack(PackType::Resource, (1, 21, 100)),
    )
    .expect("loads as it was written");
    assert!(
        as_read
            .diagnostics()
            .iter()
            .all(|f| f.code.name() != "ValueRewritten"),
        "{:?}",
        as_read.diagnostics()
    );
    // The 1.10.0 text spells the boolean, and reads back as the document it was.
    let written = as_read.to_json().expect("written as it was read");
    assert!(written.contains(r#""num_particles": true"#), "{written}");
    let again = bedrock_addon::families::particle::ParticleEffect::parse_as_written(
        &PackPath::new("particles/demo.particle.json").unwrap(),
        written.as_bytes(),
        &pack(PackType::Resource, (1, 21, 100)),
    )
    .expect("the written text loads");
    assert_eq!(again.value(), as_read.value(), "{written}");
    assert_eq!(again.to_json().expect("written again"), written);
}

/// PRT10: an element of a legacy `sequence` that is no object is left out, told at the
/// element; an element with an option-less `randomize` is kept.
#[test]
fn prt10_a_sequence_element_that_is_left_out_is_told_at_the_element() {
    let (found, written) = particle(
        r#""minecraft:emitter_rate_instant":{"num_particles":1},"minecraft:emitter_lifetime_once":{"active_time":1}"#,
        r#","events":{"e":{"sequence":[{"expression":"v.a = 1;"},7,{"randomize":[]}]}}"#,
    );
    // The legacy reader's content-log line, a kept error.
    assert_eq!(
        found,
        ["InvalidValue Error at particle_effect/events/e/sequence/1"]
    );
    assert!(written.contains(r#""randomize": []"#), "{written}");
}

/// CB1: an `addition` that is the empty text names no sound; the 1.21.120 shape has no place
/// for it and it goes: noted at the member.
#[test]
fn cb1_an_empty_ambient_addition_the_step_leaves_out_is_noted_at_the_member() {
    let biome = |sounds: &str| {
        let (found, written) = told::<bedrock_addon::families::client_biome::ClientBiome>(
            "biomes/demo.client_biome.json",
            &pack(PackType::Resource, (1, 26, 30)),
            &format!(
                r#"{{"format_version":"1.21.100","minecraft:client_biome":{{"description":{{"identifier":"demo:marsh"}},"components":{{"minecraft:ambient_sounds":{{{sounds}}}}}}}}}"#
            ),
        );
        let found: Vec<String> = found
            .into_iter()
            .filter(|f| !f.starts_with("SynthesizedComponent"))
            .collect();
        (found, written)
    };
    let (found, written) = biome(r#""addition":"","loop":"ambient.cave""#);
    assert_eq!(
        found,
        [
            "DroppedField Info at minecraft:client_biome/components/minecraft:ambient_sounds/addition"
        ]
    );
    assert!(!written.contains("addition"), "{written}");
    let (found, _) = biome(r#""loop":"ambient.cave""#);
    assert_eq!(found, [] as [&str; 0]);
}

/// A float written `-0.0` is held with its sign, and written with it: the text of a written
/// document has the number the file had (the corpus smoke: six development geometries have
/// `-0.0` in `pivot` and `rotation`).
#[test]
fn a_negative_zero_is_written_with_its_sign() {
    let negative = |written: &str, what: &str| {
        assert!(
            written.contains("-0.0") && written.matches("-0.0").count() >= 2,
            "{what}: {written}"
        );
    };
    let (found, written) = geometry(
        r#"{"format_version":"1.26.40","minecraft:geometry":[{"description":{"identifier":"geometry.x","texture_width":64,"texture_height":64},"bones":[{"name":"b","pivot":[-0.0,1,0],"rotation":[0,-0.0,0]}]}]}"#,
    );
    assert_eq!(found, [] as [&str; 0]);
    negative(&written, "geometry");
    assert!(written.contains(r#""pivot": [ -0.0, 1, 0 ]"#), "{written}");
    let (_, written) = fog(r#""fog_start":-0.0,"fog_end":-0.0,"render_distance_type":"fixed""#);
    negative(&written, "fog");
    let (_, written) = told::<bedrock_addon::families::block::Block>(
        "blocks/x.json",
        &pack(PackType::Behavior, (1, 26, 0)),
        r#"{"format_version":"1.26.30","minecraft:block":{"description":{"identifier":"demo:x"},"components":{"minecraft:transformation":{"translation":[-0.0,0.5,-0.0]}}}}"#,
    );
    negative(&written, "block");
    let (_, written) = told::<bedrock_addon::families::entity::Entity>(
        "entities/x.json",
        &pack(PackType::Behavior, (1, 26, 0)),
        r#"{"format_version":"1.26.50","minecraft:entity":{"description":{"identifier":"demo:x","is_spawnable":true,"is_summonable":true},"components":{"minecraft:collision_box":{"width":-0.0,"height":-0.0},"minecraft:rideable":{"seat_count":1,"seats":{"position":[-0.0,1,-0.0]}}}}}"#,
    );
    negative(&written, "entity");
}

/// The same for an entity read at an older version, whose components go through the
/// conversion that types them as the game writes them back (a whole real as an integer): a
/// negative zero keeps its sign there, in a typed component and in one kept as JSON.
#[test]
fn a_negative_zero_of_an_older_entity_is_written_with_its_sign() {
    let (found, written) = told::<bedrock_addon::families::entity::Entity>(
        "entities/x.json",
        &pack(PackType::Behavior, (1, 26, 0)),
        r#"{"format_version":"1.26.30","minecraft:entity":{"description":{"identifier":"demo:x","is_spawnable":true,"is_summonable":true},"components":{"minecraft:collision_box":{"width":-0.0,"height":-0.0},"minecraft:scale":{"value":-0.0},"minecraft:rideable":{"seat_count":1,"seats":{"position":[-0.0,1,-0.0]}}}}}"#,
    );
    assert_eq!(found, [] as [&str; 0]);
    assert_eq!(written.matches("-0.0").count(), 5, "{written}");
    // A whole real that is not a negative zero is still the integer the game writes back,
    // and a negative zero where the member is an integer is read as that integer.
    let (found, written) = told::<bedrock_addon::families::entity::Entity>(
        "entities/x.json",
        &pack(PackType::Behavior, (1, 26, 0)),
        r#"{"format_version":"1.26.30","minecraft:entity":{"description":{"identifier":"demo:x","is_spawnable":true,"is_summonable":true},"components":{"minecraft:boss":{"hud_range":-0.0,"should_darken_sky":true},"minecraft:collision_box":{"width":1.0,"height":2.5}}}}"#,
    );
    assert!(
        !found.iter().any(|f| f.contains("minecraft:boss")),
        "{found:?}"
    );
    assert!(
        written.contains(r#""hud_range": 0"#) && !written.contains("-0.0"),
        "{written}"
    );
}

/// The other conversions that type a JSON value by writing and reading it keep the sign the
/// same way: a goal of an older entity, and a repair entry an item step retypes.
#[test]
fn a_negative_zero_keeps_its_sign_in_a_goal_and_in_a_retyped_item_member() {
    let (_, written) = told::<bedrock_addon::families::entity::Entity>(
        "entities/x.json",
        &pack(PackType::Behavior, (1, 26, 0)),
        r#"{"format_version":"1.16.0","minecraft:entity":{"description":{"identifier":"demo:x","is_spawnable":true,"is_summonable":true},"components":{"minecraft:behavior.random_stroll":{"priority":1,"speed_multiplier":-0.0,"xz_dist":-0.0}}}}"#,
    );
    assert!(written.contains(r#""speed_multiplier": -0.0"#), "{written}");
    let mut cx = pack(PackType::Behavior, (1, 26, 0));
    cx.pack_type = PackType::Behavior;
    let (_, written) = told::<bedrock_addon::families::item::Item>(
        "items/x.json",
        &cx,
        r#"{"format_version":"1.20.50","minecraft:item":{"description":{"identifier":"demo:x"},"components":{"minecraft:max_stack_size":1,"minecraft:repairable":{"repair_items":[{"items":["minecraft:stick"],"repair_amount":-0.0}]}}}}"#,
    );
    assert!(written.contains(r#""repair_amount": -0.0"#), "{written}");
}

/// The integer token `-0` is the integer zero: an integer has no sign, a float member takes
/// it as it takes `1`, and it is written `0`. The real tokens `-0.0` and `-0e0` are the
/// negative zero and are written `-0.0`. One rule for every reader of the crate.
#[test]
fn an_integer_negative_zero_is_the_integer_zero_and_a_real_one_keeps_its_sign() {
    let bones = |pivot: &str, rotation: &str| {
        geometry(&format!(
            r#"{{"format_version":"1.26.40","minecraft:geometry":[{{"description":{{"identifier":"geometry.x","texture_width":64,"texture_height":64}},"bones":[{{"name":"b","pivot":[{pivot},1,0],"rotation":[0,{rotation},0]}}]}}]}}"#
        ))
    };
    let (found, written) = bones("-0", "-0");
    assert_eq!(found, [] as [&str; 0]);
    assert!(
        written.contains(r#""pivot": [ 0, 1, 0 ]"#)
            && written.contains(r#""rotation": [ 0, 0, 0 ]"#),
        "{written}"
    );
    let (found, written) = bones("-0.0", "-0e0");
    assert_eq!(found, [] as [&str; 0]);
    assert!(
        written.contains(r#""pivot": [ -0.0, 1, 0 ]"#)
            && written.contains(r#""rotation": [ 0, -0.0, 0 ]"#),
        "{written}"
    );
    let (found, written) = fog(r#""fog_start":-0,"fog_end":-0,"render_distance_type":"fixed""#);
    assert_eq!(found, [] as [&str; 0]);
    assert!(!written.contains("-0"), "{written}");
    let (found, written) = fog(r#""fog_start":-0.0,"fog_end":-0e0,"render_distance_type":"fixed""#);
    assert_eq!(found, [] as [&str; 0]);
    assert_eq!(written.matches("-0.0").count(), 2, "{written}");
}
