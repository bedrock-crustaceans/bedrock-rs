//! The engine's loose entity documents of 1.26.60, copied under
//! `fixtures/entity/entity/engine/`: the mobs the freezing, fall-damage, path-finder and powder-snow server tests spawn, the projectile
//! helper's template and the experimental-field template of the game's beta-load tests. Every
//! spawned mob must load (each test spawns it and asserts on its behaviour), so each loads
//! without a warning or an error and round-trips (written, read back equal, written the same).
//! A runtime identifier the game does not hold is filled in by the crate.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::document::Content;

use super::common::{cx, fixture, outcome, reparse, stable};

/// A loose document by file name, its runtime identifiers filled in.
fn engine(name: &str) -> String {
    fixture(&format!("engine/{name}"))
        .replace(
            "<PowderSnowServerTestInternal::CAN_STAND_ON_POWDER_SNOW_WALKING_MOB_NAME>",
            "server_test:can_stand_on_powder_snow_walking_mob",
        )
        .replace(
            "<PowderSnowServerTestInternal::CAN_STAND_ON_POWDER_SNOW_MOB_NAME>",
            "test:can_stand_on_powder_snow_mob",
        )
        .replace(
            "<PowderSnowServerTestInternal::DEFAULT_MOB_NAME>",
            "test:default_mob",
        )
}

/// `text` with the first placeholder that starts with `start` (up to its matching `>`, nested
/// placeholders included) replaced by `with`.
fn placeholder(text: &str, start: &str, with: &str) -> String {
    let from = text.find(start).expect("the placeholder");
    let mut depth = 0;
    let mut to = from;
    for (i, c) in text[from..].char_indices() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    to = from + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    format!("{}{with}{}", &text[..from], &text[to..])
}

/// Loads without a warning or an error, and round-trips.
#[track_caller]
fn loads_and_round_trips(name: &str, text: &str) {
    let (entity, found) = stable(text, &cx());
    assert!(
        found.iter().all(|d| d.severity < Severity::Warning),
        "{name}: {found:#?}"
    );
    let again = reparse(&entity);
    assert_eq!(again, entity, "{name}");
    assert_eq!(
        again.to_json().unwrap(),
        entity.to_json().unwrap(),
        "{name}"
    );
}

/// The six 1.26.50 mobs of the freezing, fall-damage, path-finder and powder-snow tests.
#[test]
fn the_engine_s_test_mobs_load_and_round_trip() {
    for name in [
        "freezing-1.json",
        "fall_damage-1.json",
        "path_finder-1.json",
        "powder_snow_block-1.json",
        "powder_snow-1.json",
        "powder_snow-2.json",
    ] {
        loads_and_round_trips(name, &engine(name));
    }
}

/// The projectile server tests' custom source actor template,
/// instantiated as one user of it shoots (offsets, no ignite or
/// events). Its `"anchor": 1` is the integer spelling the 1.26.40 → 1.26.50 step converts (at
/// 1.26.50 the typed member is a name), so the helper's format is below 1.26.50: 1.26.40 here.
#[test]
fn the_engine_s_custom_projectile_template_loads() {
    let template = fixture("engine/projectile-1.fmt.txt");
    let text = placeholder(&template, "<template", "")
        .replace("<format_version>", "1.26.40")
        .replace("<identifier>", "test:custom_projectile")
        .replace("<runtime_identifier>", "minecraft:arrow")
        .replace("<isolated_physics>", "false")
        .replace("<reflect_on_hurt>", "false")
        .replace("<destroy_on_hurt>", "false")
        .replace("<power>", "1.6")
        .replace("<gravity>", "0.05")
        .replace("<potion_effect>", "-1")
        .replace("<angle_offset>", "0.0")
        .replace("<offset x>", "0.0")
        .replace("<offset y>", "-0.1")
        .replace("<offset z>", "0.5");
    let text = placeholder(&text, "<on_hit", r#""impact_damage": {"damage": 4}"#);
    let text = placeholder(&text, "<optional:", "");
    let text = placeholder(&text, "<optional:", "");
    assert!(!text.contains('<'), "{text}");
    loads_and_round_trips("projectile-1", &text);
}

/// `puv_load-2`: a `"beta"` actor whose component (named at run time by the test)
/// has an experimental member `flag`. No component of the shipped game has it: the test
/// registers its own. In the game without that registration, as in the crate, the component is
/// unknown to the strict beta slice, an error, and the actor is refused even from a base-game
/// pack; the "required experiment" text needs the test's component.
#[test]
fn the_engine_s_experimental_field_template_names_an_unknown_component() {
    let text = fixture("engine/puv_load-2.fmt.txt")
        .replacen("{}", "test:experimental_mob", 1)
        .replacen("{}", "test:experimental_component", 1);
    let mut base_game = cx();
    base_game.is_base_game_pack = true;
    let refused = outcome(&text, &base_game);
    assert!(refused.value.is_none(), "{:#?}", refused.diagnostics);
    assert!(
        refused.diagnostics.iter().any(|d| matches!(
            d.code,
            DiagnosticCode::UnrecognizedComponent { .. }
        ) && d.severity == Severity::Error
            && d.location.json_path.to_string()
                == "minecraft:entity/components/test:experimental_component"),
        "{:#?}",
        refused.diagnostics
    );
}

/// `pushable_upgrade-asm-1..4.json`: the mob `server_test:test_mob` has the
/// `PushableByBlock` and `PushableByEntity` components at 1.12.1 without a component (the
/// pre-1.13 `pushable` insert, then the 1.26.10 split), none at 1.13.0, both from 1.26.0's
/// `minecraft:pushable {is_pushable, is_pushable_by_piston}` and both from 1.26.10's empty
/// `pushable_by_block` / `pushable_by_entity`. Each loads without a warning and round-trips.
#[test]
fn the_pushable_upgrade_tests_documents() {
    use bedrock_addon_core::json::Value;
    for (n, pushable) in [(1, true), (2, false), (3, true), (4, true)] {
        let name = format!("pushable_upgrade-asm-{n}.json");
        let text = fixture(&format!("engine/{name}"));
        loads_and_round_trips(&name, &text);
        let (entity, _) = stable(&text, &cx());
        let written: Value = facet_json::from_str(&entity.to_json().unwrap()).unwrap();
        let components = written
            .as_object()
            .and_then(|root| root.get("minecraft:entity"))
            .and_then(Value::as_object)
            .and_then(|payload| payload.get("components"))
            .and_then(Value::as_object);
        let has = |key: &str| components.is_some_and(|c| c.get(key).is_some());
        assert_eq!(has("minecraft:pushable_by_block"), pushable, "{name}");
        assert_eq!(has("minecraft:pushable_by_entity"), pushable, "{name}");
        assert!(!has("minecraft:pushable"), "{name}");
    }
}
