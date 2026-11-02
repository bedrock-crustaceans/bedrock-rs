//! `bp.spawn_rules()`: the registry of a behaviour stack through the accessor list: every file
//! below `spawn_rules/`, the higher pack's rules winning, routed by the world's experiment, and
//! the rules vanilla does not register (rule 8) left out.

use bedrock_addon::diagnostic::DiagnosticCode;
use bedrock_addon::families::spawn_rules::SpawnRules;
use bedrock_addon::id::{EntityId, SpawnRulesId, kind};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Behavior, OpenOptions, Pack, PackOrigin, PackStack};
use bedrock_addon::storage::PackPath;

use super::common::simplified_options;

/// A behaviour pack `n` with `files` besides its manifest.
fn pack(n: u128, files: Vec<(&str, String)>) -> Pack {
    let manifest = format!(
        r#"{{"format_version": 2, "header": {{"name": "bp{n}", "uuid": "{}", "version": [1, 0, 0], "min_engine_version": [1, 26, 20]}}, "modules": [{{"type": "data", "uuid": "{}", "version": [1, 0, 0]}}]}}"#,
        uuid::Uuid::from_u128(n),
        uuid::Uuid::from_u128(n + 1000),
    );
    let files = std::iter::once(("manifest.json", manifest))
        .chain(files)
        .map(|(path, text)| (PackPath::new(path).expect("a pack path"), text.into_bytes()));
    let options = OpenOptions {
        origin: PackOrigin::User,
        ..OpenOptions::default()
    };
    Pack::from_files(files, &options).expect("the pack opens")
}

/// Standard rules for `id` in `pool` with one condition of weight `weight`.
fn rules(id: &str, pool: &str, weight: i32) -> String {
    format!(
        r#"{{"format_version": "1.8.0", "minecraft:spawn_rules": {{"description": {{"identifier": "{id}", "population_control": "{pool}"}}, "conditions": [{{"minecraft:weight": {{"default": {weight}}}}}]}}}}"#
    )
}

/// The weight of the first condition of the standard rules of `id`.
fn weight(stack: &PackStack<Behavior>, id: &str) -> i32 {
    let document = stack
        .spawn_rules()
        .get(&SpawnRulesId::parse(id))
        .unwrap_or_else(|| panic!("{id} is not registered"));
    let SpawnRules::Standard(rules) = document.value() else {
        panic!("simplified");
    };
    rules.conditions.as_ref().unwrap()[0].spawn_weight()
}

#[test]
fn the_higher_pack_wins_and_every_file_below_spawn_rules_is_read() {
    let top = pack(
        1,
        vec![("spawn_rules/pig.json", rules("x:pig", "animal", 9))],
    );
    let low = pack(
        2,
        vec![
            ("spawn_rules/pig.json", rules("x:pig", "animal", 1)),
            ("spawn_rules/deep/cow.json", rules("x:cow", "animal", 3)),
            ("spawn_rules/cow.txt", rules("x:txt", "animal", 3)),
        ],
    );
    let stack = PackStack::<Behavior>::of([top, low], LoadOptions::default()).unwrap();
    assert_eq!(weight(&stack, "x:pig"), 9);
    assert_eq!(weight(&stack, "x:cow"), 3);
    assert!(!stack.spawn_rules().contains(&SpawnRulesId::parse("x:txt")));
    // The key is the entity's: an `EntityId` casts to it.
    let entity = EntityId::parse("x:cow").cast::<kind::SpawnRules>();
    assert!(stack.spawn_rules().contains(&entity));
}

#[test]
fn s09h_rules_without_a_known_pool_are_not_registered() {
    let bp = pack(
        1,
        vec![
            ("spawn_rules/a.json", rules("x:boss", "boss", 1)),
            ("spawn_rules/b.json", rules("x:none", "", 1)),
            ("spawn_rules/c.json", rules("x:monster", "Monster", 1)),
        ],
    );
    let stack = PackStack::<Behavior>::of([bp], LoadOptions::default()).unwrap();
    let registry = stack.spawn_rules();
    assert!(registry.contains(&SpawnRulesId::parse("x:monster")));
    assert!(!registry.contains(&SpawnRulesId::parse("x:boss")));
    assert!(!registry.contains(&SpawnRulesId::parse("x:none")));
    assert!(registry.rejected().is_empty());
    // Their warnings are the registry's.
    assert!(
        registry
            .diagnostics()
            .any(|d| matches!(d.code, DiagnosticCode::InvalidEnumValue { .. }))
    );
}

#[test]
fn the_world_experiment_routes_the_files() {
    let modern = r#"{"format_version": "1.21.60", "minecraft:spawn_rules": {"description": {"identifier": "x:new"}, "conditions": [{"weight": 4}]}}"#;
    let bp = || pack(1, vec![("spawn_rules/new.json", modern.to_owned())]);
    let on = PackStack::<Behavior>::of([bp()], simplified_options()).unwrap();
    let document = on.spawn_rules().get(&SpawnRulesId::parse("x:new")).unwrap();
    assert!(matches!(document.value(), SpawnRules::Simplified(_)));
    // Without the experiment the file is read by `spawn`, has no pool and is not registered.
    let off = PackStack::<Behavior>::of([bp()], LoadOptions::default()).unwrap();
    assert!(!off.spawn_rules().contains(&SpawnRulesId::parse("x:new")));
}
