//! `projectile`: the 1.26.50 struct, its `on_hit` subcomponents, the value messages
//! and the lenient conversion of the legacy reader's JSON.

use bedrock_addon_core::diagnostic::{DiagnosticCode, Severity};
use bedrock_addon_core::json::{StrictFloat, StrictInt, StrictString};
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::{
    DifficultyRandomization, EffectDuration, Projectile, ProjectileAnchor, ProjectileOffset,
    ShouldBounce,
};

use super::{fixture, has, json, lenient, read_as, round_trip, strict, write, write_with};

#[test]
fn the_vanilla_1_26_50_arrow_round_trips() {
    let arrow = round_trip::<Projectile>(&fixture("projectile_arrow_1_26_50.json"));
    assert_eq!(arrow.anchor, ProjectileAnchor::EyeHeight);
    assert_eq!(arrow.power.0, 1.1);
    assert_eq!(arrow.should_bounce, ShouldBounce::IfNoDamageDealt);
    assert!(!arrow.isolated_physics.0);
    let on_hit = arrow.on_hit.as_ref().expect("on_hit");
    let damage = on_hit.impact_damage.as_ref().expect("impact_damage");
    assert!(damage.apply_knockback_to_blocking_targets.0 && damage.ceil_pre_critical_damage.0);
    assert_eq!(damage.power_multiplier.0, 2.0);
    assert_eq!(
        on_hit.stick_in_ground.as_ref().map(|s| s.shake_time.0),
        Some(0.35)
    );
    assert_eq!(
        arrow.offset,
        ProjectileOffset::Array(vec![StrictFloat(0.0), StrictFloat(-0.1), StrictFloat(0.0)])
    );
}

#[test]
fn projectile_reads_its_schema_defaults() {
    let (value, d) = strict::<Projectile>("{}");
    assert!(d.is_empty());
    assert_eq!(value, Projectile::default());
    assert_eq!(value.gravity.0, 0.05);
    assert_eq!(value.inertia.0, 0.99);
    assert_eq!(value.liquid_inertia.0, 0.6);
    assert_eq!(value.on_fire_time.0, 5.0);
    assert_eq!(value.particle.0, "iconcrack");
    assert_eq!(value.potion_effect.0, -1);
    assert_eq!(value.power.0, 1.3);
    assert_eq!(value.splash_range.0, 4.0);
    assert!(value.shoot_target.0 && value.isolated_physics.0);
    let (damage, _) = strict::<Projectile>(r#"{"on_hit": {"impact_damage": {}}}"#);
    let damage = damage
        .on_hit
        .and_then(|o| o.impact_damage)
        .expect("impact_damage");
    assert_eq!(damage.max_critical_damage.0, i32::MAX);
    assert_eq!(damage.power_multiplier.0, 0.0);
    assert!(damage.destroy_on_hit_requires_damage.0 && damage.set_last_hurt_requires_damage.0);
    assert_eq!(
        damage.difficulty_randomization,
        DifficultyRandomization::None
    );
}

#[test]
fn on_hit_subcomponents_read() {
    let value = round_trip::<Projectile>(
        r#"{"on_hit": {
            "grant_xp": {"minXP": 3, "maxXP": 11},
            "mob_effect": {"effects": [{"effect": "poison", "durationeasy": 0, "durationnormal": 200, "durationhard": "infinite", "amplifier": 0}]},
            "spawn_chance": {"first_spawn_chance": 0.125, "first_spawn_count": 1, "spawn_definition": "minecraft:chicken", "spawn_baby": true},
            "definition_event": {"affect_projectile": true, "event_trigger": {"event": "minecraft:explode", "target": "self"}},
            "freeze_on_hit": {"size": 2, "shape": "sphere", "snap_to_block": true},
            "remove_on_hit": {}, "douse_fire": {}, "teleport_owner": {}, "wind_burst_on_hit": {}, "ignite": {},
            "particle_on_hit": {"particle_type": "snowballpoof", "num_particles": 6, "on_entity_hit": true, "on_other_hit": true},
            "spawn_aoe_cloud": {"radius": 3, "potion": 5, "duration": 30, "particle": "mobspell"},
            "thrown_potion_effect": {"effect": 4},
            "hurt_owner": {"owner_damage": 5, "knockback": true}
        }}"#,
    );
    let on_hit = value.on_hit.expect("on_hit");
    assert_eq!(
        on_hit.grant_xp.as_ref().map(|g| (g.min_xp.0, g.max_xp.0)),
        Some((3, 11))
    );
    let effects = &on_hit.mob_effect.as_ref().expect("mob_effect").effects;
    assert_eq!(
        effects[0].durationnormal,
        Some(EffectDuration::Ticks(StrictInt(200)))
    );
    assert_eq!(
        effects[0].durationhard,
        Some(EffectDuration::Named(StrictString("infinite".to_owned())))
    );
    assert!(on_hit.remove_on_hit.is_some() && on_hit.teleport_owner.is_some());
    assert!(on_hit.spawn_chance.as_ref().is_some_and(|s| s.spawn_baby.0));
}

#[test]
fn spawn_chance_above_one_is_an_error() {
    let (value, d) = read_as::<Projectile>(
        r#"{"on_hit": {"spawn_chance": {"first_spawn_chance": 2}}}"#,
        ConvertMode::Strict,
    );
    assert!(value.is_none());
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::InvalidValue { .. }),
        Severity::Error,
        "on_hit/spawn_chance/first_spawn_chance"
    ));
}

#[test]
fn v12_projectile_value_messages_are_warnings_with_the_retail_texts() {
    // v12;: warnings, `no` / `none` used.
    let (value, d) = strict::<Projectile>(&fixture("v12_projectile_value_messages.json"));
    assert_eq!(value.should_bounce, ShouldBounce::No);
    let damage = value
        .on_hit
        .and_then(|o| o.impact_damage)
        .expect("impact_damage");
    assert_eq!(
        damage.difficulty_randomization,
        DifficultyRandomization::None
    );
    let texts: Vec<(Severity, String, String)> = d
        .iter()
        .map(|f| {
            (
                f.severity,
                f.location.json_path.to_string(),
                f.code.to_string(),
            )
        })
        .collect();
    assert_eq!(
        texts,
        vec![
            (
                Severity::Error,
                "should_bounce".to_owned(),
                "'sometimes' is not a valid 'should_bounce' value.".to_owned()
            ),
            (
                Severity::Error,
                "on_hit/impact_damage/difficulty_randomization".to_owned(),
                "'double' is not a valid 'difficulty_randomization' value.".to_owned()
            ),
        ]
    );
}

#[test]
fn e34_the_lenient_conversion_maps_integer_anchors() {
    // e34 "projectile `anchor` 0/1/2/7";
    for (anchor, expected) in [
        (0, ProjectileAnchor::Origin),
        (1, ProjectileAnchor::EyeHeight),
        (2, ProjectileAnchor::Middle),
    ] {
        let (value, d) = lenient::<Projectile>(&format!(r#"{{"anchor": {anchor}}}"#));
        assert!(d.is_empty(), "{anchor}: {d:?}");
        assert_eq!(value.anchor, expected);
    }
    let (value, d) = lenient::<Projectile>(r#"{"anchor": 7}"#);
    assert_eq!(value.anchor, ProjectileAnchor::Origin);
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::InvalidValue { .. }),
        Severity::Info,
        "anchor"
    ));
}

#[test]
fn an_integer_anchor_is_an_error_in_a_1_26_50_document() {
    let (value, d) = read_as::<Projectile>(r#"{"anchor": 1}"#, ConvertMode::Strict);
    assert!(value.is_none() && d.has_errors());
}

#[test]
fn v14_legacy_aliases_are_read_by_the_lenient_conversion_only() {
    // v14: a 1.26.40 `projectile` with `reflectOnHurt` read by the legacy path; the
    // same member in a 1.26.50 document is `UnrecognizedField`.
    let text = fixture("v14_projectile_reflect_on_hurt.json");
    let (old, d) = lenient::<Projectile>(&text);
    assert!(d.is_empty(), "{d:?}");
    assert!(old.reflect_on_hurt.0);
    assert!(
        d.unknown()
            .get(
                &bedrock_addon_core::diagnostic::JsonPath::new(),
                "reflectOnHurt"
            )
            .is_none()
    );
    let (new, d) = strict::<Projectile>(&text);
    assert!(!new.reflect_on_hurt.0);
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::UnrecognizedField { .. }),
        Severity::Warning,
        "reflectOnHurt"
    ));
}

#[test]
fn the_lenient_conversion_maps_impact_damage_aliases_when_the_member_is_absent() {
    // `projectile.on_hit.impact_damage`: each alias onto its canonical member
    // when the canonical one is absent (ours).
    let (value, d) = lenient::<Projectile>(
        r#"{"onHit": {"impact_damage": {"catchFire": true, "destroyonhit": true, "maxcriticaldamage": 9,
                                         "powermultiplier": 1.5, "power_multiplier": 3}}}"#,
    );
    let damage = value
        .on_hit
        .and_then(|o| o.impact_damage)
        .expect("impact_damage");
    assert!(damage.catch_fire.0 && damage.destroy_on_hit.0);
    assert_eq!(damage.max_critical_damage.0, 9);
    assert_eq!(damage.power_multiplier.0, 3.0, "the canonical member wins");
    assert!(has(
        &d,
        |c| matches!(c, DiagnosticCode::UnrecognizedField { .. }),
        Severity::Warning,
        "on_hit/impact_damage/powermultiplier"
    ));
}

#[test]
fn should_bounce_reads_each_value() {
    for (text, expected) in [
        ("no", ShouldBounce::No),
        ("if_invulnerable", ShouldBounce::IfInvulnerable),
        ("if_no_damage_dealt", ShouldBounce::IfNoDamageDealt),
    ] {
        let (value, _) = strict::<Projectile>(&format!(r#"{{"should_bounce": "{text}"}}"#));
        assert_eq!(value.should_bounce, expected);
    }
}

#[test]
fn projectile_offset_reads_the_object_form_and_rejects_a_short_array() {
    let value = round_trip::<Projectile>(r#"{"offset": {"x": 0.5, "y": 1}}"#);
    let ProjectileOffset::Object(xyz) = &value.offset else {
        panic!("the object form: {:?}", value.offset)
    };
    assert_eq!((xyz.x.0, xyz.y.0, xyz.z.0), (0.5, 1.0, 0.0));
    let (short, d) = read_as::<Projectile>(r#"{"offset": [1, 2]}"#, ConvertMode::Strict);
    assert!(short.is_none() && d.has_errors());
}

#[test]
fn the_lenient_conversion_reads_a_legacy_single_mob_effect() {
    // Vanilla's 1.26.0 `shulker_bullet.json` writes the effect at `mob_effect`'s root; the
    // 1.26.50 copy writes `effects: [{…}]`.
    let (value, d) = lenient::<Projectile>(
        r#"{"on_hit": {"mob_effect": {"amplifier": 1, "durationhard": 200, "effect": "levitation", "durationeasy": 200, "durationnormal": 200}}}"#,
    );
    assert!(d.is_empty(), "{d:?}");
    let effects = value
        .on_hit
        .and_then(|o| o.mob_effect)
        .expect("mob_effect")
        .effects;
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].effect.0, "levitation");
    assert_eq!(effects[0].amplifier.0, 1);
}

/// The legacy spawn chance reader reads `*_spawn_percent_chance` (float,
/// default −1, ÷ 100) and, when that is negative, `*_spawn_chance` as an
/// integer N (default 100) for a chance of 1/N; the second group reads the same way.
/// Vanilla's 1.26.40 `egg.json` writes `first_spawn_chance: 8` (one in
/// eight) and `ender_pearl.json` `first_spawn_percent_chance: 5.0`.
#[test]
fn legacy_spawn_chances_are_read_as_the_game_reads_them() {
    let chances = |json: &str| {
        let (value, d) = lenient::<Projectile>(json);
        let spawn = value
            .on_hit
            .and_then(|o| o.spawn_chance)
            .expect("spawn_chance");
        (
            spawn.first_spawn_chance.0,
            spawn.second_spawn_chance.0,
            spawn.second_spawn_count.0,
            d,
        )
    };
    let (first, second, count, d) = chances(
        r#"{"on_hit": {"spawn_chance": {"first_spawn_chance": 8, "first_spawn_count": 1, "second_spawn_chance": 32, "second_spawn_count": 4}}}"#,
    );
    assert_eq!((first, second, count), (0.125, 0.031_25, 4));
    assert!(
        d.iter().all(|f| f.severity < Severity::Warning
            && !matches!(f.code, DiagnosticCode::InvalidValue { .. })),
        "{d:?}"
    );
    // A percent chance is read and divided by 100; an absent pair is one in 100.
    let (first, second, _, d) = chances(
        r#"{"on_hit": {"spawn_chance": {"first_spawn_count": 1, "first_spawn_percent_chance": 5.0}}}"#,
    );
    assert_eq!((first, second), (0.05, 0.01));
    assert!(
        !d.iter()
            .any(|f| matches!(f.code, DiagnosticCode::UnrecognizedField { .. })),
        "{d:?}"
    );
    // A negative percent falls back to the one-in-N member, the camel spellings included.
    let (first, second, _, _) = chances(
        r#"{"on_hit": {"spawn_chance": {"firstSpawnPercentChance": -1, "firstSpawnChance": 4, "secondSpawnPercentChance": 50}}}"#,
    );
    assert_eq!((first, second), (0.25, 0.5));
}

/// The reader looks the member up and converts it to a float or an integer with a default,
/// so a value that is no number converts as the game's JSON conversion does:
/// `null` is the default, a bool 0 or 1, a string, array or object 0. A percent
/// of 0 is kept (0 % → chance 0); N = 0 gives 1/0, +inf in the game,
/// written 1.
#[test]
fn legacy_spawn_chances_convert_other_types_as_the_game_does() {
    let chances = |spawn: &str| {
        let (value, _) =
            lenient::<Projectile>(&format!(r#"{{"on_hit": {{"spawn_chance": {{{spawn}}}}}}}"#));
        let spawn = value
            .on_hit
            .and_then(|o| o.spawn_chance)
            .expect("spawn_chance");
        (spawn.first_spawn_chance.0, spawn.second_spawn_chance.0)
    };
    let rows: [(&str, (f32, f32)); 7] = [
        // A string or array percent is 0 %: the chance 0, the one-in-N member not read.
        (
            r#""first_spawn_percent_chance": "50", "first_spawn_chance": 4"#,
            (0.0, 0.01),
        ),
        (
            r#""first_spawn_percent_chance": [50], "second_spawn_percent_chance": {}"#,
            (0.0, 0.0),
        ),
        // A bool percent is 0 % or 1 %.
        (
            r#""first_spawn_percent_chance": true, "second_spawn_percent_chance": false"#,
            (0.01, 0.0),
        ),
        // A string or array N is 0: 1/0, written 1.
        (
            r#""first_spawn_chance": "8", "second_spawn_chance": [8]"#,
            (1.0, 1.0),
        ),
        // A bool N is 1 (always) or 0 (1/0, written 1).
        (
            r#""first_spawn_chance": true, "second_spawn_chance": false"#,
            (1.0, 1.0),
        ),
        // `null` is the member's default: −1 % (so N is read), N = 100.
        (
            r#""first_spawn_percent_chance": null, "first_spawn_chance": 4, "second_spawn_chance": null"#,
            (0.25, 0.01),
        ),
        // A real N is truncated toward zero.
        (
            r#""first_spawn_chance": 2.9, "second_spawn_chance": 4.5"#,
            (0.5, 0.25),
        ),
    ];
    let wrong: Vec<_> = rows
        .into_iter()
        .map(|(spawn, expected)| (spawn, chances(spawn), expected))
        .filter(|(_, read, expected)| read != expected)
        .collect();
    assert!(wrong.is_empty(), "(row, read, expected): {wrong:#?}");
}

/// A legacy spawn chance the reader takes as another number than the one written (a
/// value of another type, a real or out-of-range N, an N below 1) is told where it is written
/// with an information `ValueRewritten` (the game is silent); an integer N of 1 or
/// more and a number percent are not told.
#[test]
fn a_legacy_spawn_chance_read_as_another_number_is_told_as_rewritten() {
    let told = |spawn: &str| {
        let (_, d) =
            lenient::<Projectile>(&format!(r#"{{"on_hit": {{"spawn_chance": {{{spawn}}}}}}}"#));
        let mut at: Vec<_> = d
            .iter()
            .filter(|f| matches!(f.code, DiagnosticCode::ValueRewritten { .. }))
            .map(|f| (f.location.json_path.to_string(), f.severity))
            .collect();
        at.sort();
        at
    };
    let info = |member: &str| (format!("on_hit/spawn_chance/{member}"), Severity::Info);
    let rows: Vec<(&str, Vec<(String, Severity)>)> = vec![
        (
            r#""first_spawn_chance": "8", "second_spawn_chance": [8]"#,
            vec![info("first_spawn_chance"), info("second_spawn_chance")],
        ),
        // The camel spelling is renamed before the read, the rename told as a move (the
        // document maps the finding back to the file's spelling).
        (
            r#""first_spawn_chance": true, "secondSpawnChance": {}"#,
            vec![info("first_spawn_chance"), info("second_spawn_chance")],
        ),
        (
            r#""first_spawn_chance": 0.08, "second_spawn_chance": 1e39"#,
            vec![info("first_spawn_chance"), info("second_spawn_chance")],
        ),
        (
            r#""first_spawn_chance": 0, "second_spawn_chance": -4"#,
            vec![info("first_spawn_chance"), info("second_spawn_chance")],
        ),
        (
            r#""first_spawn_percent_chance": "50", "second_spawn_percent_chance": false"#,
            vec![
                info("first_spawn_percent_chance"),
                info("second_spawn_percent_chance"),
            ],
        ),
        (
            r#""first_spawn_chance": 8, "second_spawn_chance": 1"#,
            vec![],
        ),
        (
            r#""first_spawn_percent_chance": 5.5, "second_spawn_percent_chance": -1"#,
            vec![],
        ),
        // `null` has its own information, not a second one.
        (
            r#""first_spawn_chance": null"#,
            vec![info("first_spawn_chance")],
        ),
    ];
    let wrong: Vec<_> = rows
        .into_iter()
        .map(|(spawn, expected)| (spawn, told(spawn), expected))
        .filter(|(_, told, expected)| told != expected)
        .collect();
    assert!(wrong.is_empty(), "(row, told, expected): {wrong:#?}");
}

/// `projectile`: `spread_fire` (bool, default `true`, ctor
/// offset 0x35), which the 1.26.50 type binds and the crate lacked.
#[test]
fn projectile_spread_fire_defaults_to_true() {
    let (value, _) = strict::<Projectile>("{}");
    let written = write(&value);
    assert_eq!(
        written
            .as_object()
            .and_then(|o| o.get("spread_fire"))
            .cloned(),
        Some(json("true"))
    );
    let (value, d) = strict::<Projectile>(r#"{"spread_fire": false}"#);
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(
        write(&value)
            .as_object()
            .and_then(|o| o.get("spread_fire"))
            .cloned(),
        Some(json("false"))
    );
}

/// `FreezeOnHitSubcomponentDefinition`: `size` defaults to 1
/// (not 0).
#[test]
fn freeze_on_hit_size_defaults_to_one() {
    let (value, _) = strict::<Projectile>(r#"{"on_hit": {"freeze_on_hit": {}}}"#);
    let freeze = value
        .on_hit
        .as_ref()
        .and_then(|on_hit| on_hit.freeze_on_hit.as_ref())
        .expect("freeze_on_hit");
    assert_eq!(freeze.size.0, 1.0);
}

/// The game has no reader of
/// `owner_launch_immunity_ticks`, typed or legacy; it is no member of `projectile`, kept as
/// written and told as unknown in both reads.
#[test]
fn owner_launch_immunity_ticks_is_no_member() {
    for mode in [ConvertMode::Strict, ConvertMode::Lenient] {
        let (value, d) = read_as::<Projectile>(r#"{"owner_launch_immunity_ticks": 5}"#, mode);
        assert!(value.is_some(), "{mode:?}: {d:?}");
        assert!(
            has(
                &d,
                |c| matches!(c, DiagnosticCode::UnrecognizedField { .. }),
                Severity::Warning,
                "owner_launch_immunity_ticks"
            ),
            "{mode:?}: {d:?}"
        );
        let written = write_with(&value.expect("kept"), d.unknown());
        assert_eq!(
            written
                .as_object()
                .and_then(|o| o.get("owner_launch_immunity_ticks"))
                .cloned(),
            Some(json("5")),
            "{mode:?}"
        );
    }
}

/// the legacy on-hit readers take the
/// camel-case spellings of `catch_fire`, `hurt_owner`, `spawn_chance` and `definition_event`;
/// below 1.26.50 they are the members, with no finding; the typed 1.26.50 reader binds none.
#[test]
fn the_legacy_on_hit_camel_case_spellings_are_the_members_below_1_26_50() {
    // The chances are one in 2 and one in 4: the legacy reader reads integer one-in-N chances.
    let text = r#"{"on_hit": {
        "catch_fire": {"onFireTime": 3, "fireAffectedByGriefing": true},
        "hurt_owner": {"ownerDamage": 2},
        "spawn_chance": {"firstSpawnChance": 2, "firstSpawnCount": 2, "secondSpawnChance": 4,
                         "secondSpawnCount": 3, "spawnDefinition": "minecraft:chicken", "spawnBaby": true},
        "definition_event": {"affectShooter": true, "affectProjectile": true, "affectTarget": true,
                             "affectSplashArea": true, "splashArea": 2, "eventTrigger": {"event": "demo:hit"}}
    }}"#;
    let (value, d) = lenient::<Projectile>(text);
    assert!(!d.iter().any(|f| f.severity >= Severity::Warning), "{d:?}");
    let on_hit = value.on_hit.as_ref().expect("on_hit");
    let fire = on_hit.catch_fire.as_ref().expect("catch_fire");
    assert_eq!(fire.on_fire_time.0, 3.0);
    assert!(fire.fire_affected_by_griefing.0);
    assert_eq!(
        on_hit
            .hurt_owner
            .as_ref()
            .expect("hurt_owner")
            .owner_damage
            .0,
        2.0
    );
    let spawn = on_hit.spawn_chance.as_ref().expect("spawn_chance");
    assert_eq!(
        (spawn.first_spawn_chance.0, spawn.first_spawn_count.0),
        (0.5, 2)
    );
    assert_eq!(
        (spawn.second_spawn_chance.0, spawn.second_spawn_count.0),
        (0.25, 3)
    );
    assert_eq!(spawn.spawn_definition.0, "minecraft:chicken");
    assert!(spawn.spawn_baby.0);
    let event = on_hit.definition_event.as_ref().expect("definition_event");
    assert!(event.affect_shooter.0 && event.affect_projectile.0 && event.affect_target.0);
    assert!(event.affect_splash_area.0);
    assert_eq!(event.splash_area.0, 2.0);
    assert!(event.event_trigger.is_some());
    let (_, d) = read_as::<Projectile>(text, ConvertMode::Strict);
    assert!(
        has(
            &d,
            |c| matches!(c, DiagnosticCode::UnrecognizedField { .. }),
            Severity::Warning,
            "on_hit/catch_fire/onFireTime"
        ),
        "{d:?}"
    );
}

/// Below 1.26.50 the legacy `mob_effect` reader takes `effects` as one
/// object as well as a list.
#[test]
fn a_lone_mob_effect_object_is_one_effect_below_1_26_50() {
    let (value, d) = lenient::<Projectile>(
        r#"{"on_hit": {"mob_effect": {"effects": {"effect": "poison", "durationeasy": 100}}}}"#,
    );
    assert!(!d.iter().any(|f| f.severity >= Severity::Warning), "{d:?}");
    let effects = &value
        .on_hit
        .as_ref()
        .and_then(|o| o.mob_effect.as_ref())
        .expect("mob_effect")
        .effects;
    assert_eq!(effects.len(), 1);
    assert_eq!(effects[0].effect.0, "poison");
}

/// The ≥ 1.26.50 side: the typed reader takes `effects` as a list only,
/// so a lone object is an error at `effects`.
#[test]
fn a_lone_mob_effect_object_is_an_error_at_1_26_50() {
    let (_, d) = read_as::<Projectile>(
        r#"{"on_hit": {"mob_effect": {"effects": {"effect": "poison", "durationeasy": 100}}}}"#,
        ConvertMode::Strict,
    );
    assert!(
        has(
            &d,
            |c| matches!(c, DiagnosticCode::InvalidType { .. }),
            Severity::Error,
            "on_hit/mob_effect/effects"
        ),
        "{d:?}"
    );
}
