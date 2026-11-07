//! `experience_reward` and `anger_level`: their Molang members read, versioned by the pack and
//! compiled, the lenient conversion's reader, the 1.26.0 fold and the
//! deprecated array names of 1.21.110.

use bedrock_addon_core::diagnostic::{Diagnostics, JsonPath, Location, Severity};
use bedrock_addon_core::json::JsonSlice;
use bedrock_addon_core::molang::{
    ExperimentMask, Molang, MolangContext, MolangFields, MolangSource, pass_one,
};
use bedrock_addon_core::version::{FormatVersionInput, MinEngineVersion};
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::{
    AngerLevel, BlockMovementSlowdownImmunity, EntityComponent, ExperienceReward,
};

use super::{fixture, has, json, lenient, read_as, round_trip, strict, write};

/// The Molang context of a document at 1.26.0 in a pack at `min_engine_version`.
fn pack(min_engine_version: [u16; 3]) -> MolangContext {
    let [major, minor, patch] = min_engine_version;
    MolangContext::new(
        Some(&MinEngineVersion::new(major, minor, patch)),
        ExperimentMask::empty(),
        &FormatVersionInput::parse("1.26.0").expect("a version"),
        None,
    )
}

/// Pass 1 (reader check, context version) and, with the `molang` feature, pass 2 (compile).
fn passes<T: MolangFields>(value: &mut T, cx: &MolangContext) -> Diagnostics {
    let base = Location::new(None, None, JsonPath::new(), None);
    let mut d = Diagnostics::default();
    pass_one(value, cx, &base, &mut d);
    #[cfg(feature = "molang")]
    bedrock_addon_core::molang::check_fields(
        value,
        &bedrock_addon_core::molang::MolangCheck::new(),
        bedrock_addon_core::molang::Side::Server,
        cx,
        &base,
        &mut d,
    );
    d
}

/// The compiler's codes behind the `InvalidMolang` / `MolangLint` findings, with severities.
#[cfg(feature = "molang")]
fn molang_codes(d: &Diagnostics) -> Vec<(Severity, bedrock_addon_core::molang::DiagCode)> {
    use bedrock_addon_core::diagnostic::DiagnosticDetail;
    d.iter()
        .filter_map(|f| match f.detail.as_deref() {
            Some(DiagnosticDetail::Molang(m)) => Some((f.severity, m.code())),
            _ => None,
        })
        .collect()
}

#[test]
fn m01_experience_reward_number_is_a_constant_without_a_diagnostic() {
    // m01: 1.26.0 `experience_reward {on_death: 5}`.
    let (mut reward, d) = strict::<ExperienceReward>(&fixture("m01_experience_reward_float.json"));
    assert!(d.is_empty());
    assert_eq!(reward.on_death, Some(Molang::Const(5.0)));
    let d = passes(&mut reward, &pack([1, 21, 100]));
    assert!(d.is_empty(), "not compiled: {d:?}");
}

#[test]
fn m02_experience_reward_string_takes_the_pack_molang_version() {
    // m02: `on_death: "'a' + 1"`; Molang 0 in a [1, 16, 0] pack, 13 in [1, 21, 100].
    let text = fixture("m02_experience_reward_string_pack_version.json");
    let (mut old_pack, _) = strict::<ExperienceReward>(&text);
    assert!(passes(&mut old_pack, &pack([1, 16, 0])).is_empty());
    let (mut new_pack, _) = strict::<ExperienceReward>(&text);
    let d = passes(&mut new_pack, &pack([1, 21, 100]));
    #[cfg(feature = "molang")]
    {
        assert!(has(
            &d,
            |c| matches!(c.id(), "core/invalid_molang"),
            Severity::Error,
            "on_death"
        ));
        assert_eq!(
            molang_codes(&d),
            vec![(
                Severity::Error,
                bedrock_addon_core::molang::DiagCode::StringMisuse
            )]
        );
    }
    #[cfg(not(feature = "molang"))]
    assert!(d.is_empty());
    let kept = new_pack
        .on_death
        .as_ref()
        .and_then(Molang::source)
        .map(MolangSource::as_str);
    assert_eq!(kept, Some("'a' + 1"), "the value is kept");
}

#[test]
fn m03_experience_reward_object_form_keeps_its_own_version() {
    // m03: `{"expression": "'a' + 1", "version": 2}` compiles at 2 in a
    // [1, 21, 100] pack.
    let text = fixture("m03_experience_reward_object_version.json");
    let (mut reward, _) = strict::<ExperienceReward>(&text);
    let d = passes(&mut reward, &pack([1, 21, 100]));
    assert!(d.is_empty(), "{d:?}");
}

#[test]
fn m03_experience_reward_object_form_in_the_lenient_conversion_is_invalid_type() {
    // m03: the same object in a 1.21.100 document (lenient conversion,
    // `SchemaValidated`): `InvalidType` warning, field default.
    let (reward, d) =
        lenient::<ExperienceReward>(&fixture("m03_experience_reward_object_version.json"));
    assert_eq!(reward.on_death, None);
    assert!(has(
        &d,
        |c| matches!(c.id(), "core/invalid_type"),
        Severity::Info,
        "on_death"
    ));
}

#[cfg(feature = "molang")]
#[test]
fn m04_experience_reward_object_versions_outside_the_range() {
    use bedrock_addon_core::molang::DiagCode;
    // m04.
    let cases: [(&str, &[(Severity, DiagCode)]); 3] = [
        (
            r#"{"on_death": {"expression": "query.is_baby ? 2 : 1", "version": -1}}"#,
            &[
                (Severity::Info, DiagCode::InvalidVersion),
                (Severity::Info, DiagCode::QueryDeprecated),
                (Severity::Error, DiagCode::UnknownQuery),
                (Severity::Error, DiagCode::Syntax),
            ],
        ),
        (
            r#"{"on_death": {"expression": "query.is_baby ? 2 : 1", "version": 14}}"#,
            &[
                (Severity::Info, DiagCode::InvalidVersion),
                (Severity::Info, DiagCode::QueryDeprecated),
                (Severity::Error, DiagCode::UnknownQuery),
                (Severity::Error, DiagCode::Syntax),
            ],
        ),
        (
            r#"{"on_death": {"expression": "1 + 1", "version": 14}}"#,
            &[(Severity::Info, DiagCode::InvalidVersion)],
        ),
    ];
    for (text, expected) in cases {
        let (mut reward, _) = strict::<ExperienceReward>(text);
        let d = passes(&mut reward, &pack([1, 21, 100]));
        let mut found = molang_codes(&d);
        let mut expected = expected.to_vec();
        found.sort_by_key(|(s, c)| (format!("{s:?}"), format!("{c:?}")));
        expected.sort_by_key(|(s, c)| (format!("{s:?}"), format!("{c:?}")));
        assert_eq!(found, expected, "{text}");
        assert!(reward.on_death.is_some(), "the value is kept");
    }
}

#[test]
fn experience_reward_bool_is_rejected_by_the_strict_reader_of_pass_one() {
    // `StrictVersioned` in documents ≥ 1.21.110 (no bool).
    let (mut reward, _) = strict::<ExperienceReward>(r#"{"on_death": true}"#);
    let d = passes(&mut reward, &pack([1, 21, 100]));
    assert!(
        d.iter().any(|f| matches!(f.code.id(), "core/invalid_type")),
        "{d:?}"
    );
}

#[test]
fn experience_reward_bool_is_kept_by_the_lenient_conversion() {
    let (reward, d) = lenient::<ExperienceReward>(r#"{"on_death": true}"#);
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(reward.on_death, Some(Molang::Bool(true)));
}

#[test]
fn e29c_experience_reward_fold_moves_a_single_element_into_an_empty_scalar() {
    // e29c: empty scalar + 1-element array folds; 2-element array and non-empty
    // scalar carried unchanged.
    let (mut empty_scalar, _) = strict::<ExperienceReward>(
        r#"{"on_death": "", "on_death_array": ["query.last_hit_by_player ? 5 : 0"]}"#,
    );
    empty_scalar.fold_single_element_arrays();
    assert_eq!(
        empty_scalar
            .on_death
            .as_ref()
            .and_then(Molang::source)
            .map(MolangSource::as_str),
        Some("query.last_hit_by_player ? 5 : 0")
    );
    assert_eq!(empty_scalar.on_death_array, None);

    let (mut absent_scalar, _) = strict::<ExperienceReward>(r#"{"on_bred_array": [7]}"#);
    absent_scalar.fold_single_element_arrays();
    assert_eq!(absent_scalar.on_bred, Some(Molang::Const(7.0)));

    let two = r#"{"on_death": "", "on_death_array": [1, 2]}"#;
    let (mut unchanged, _) = strict::<ExperienceReward>(two);
    let before = unchanged.clone();
    unchanged.fold_single_element_arrays();
    assert_eq!(unchanged, before);

    let scalar = r#"{"on_death": "3", "on_death_array": [1]}"#;
    let (mut unchanged, _) = strict::<ExperienceReward>(scalar);
    let before = unchanged.clone();
    unchanged.fold_single_element_arrays();
    assert_eq!(unchanged, before);
}

#[test]
fn v13_experience_reward_arrays_are_deprecated_names_in_1_21_110() {
    // v13: `experience_reward.on_bred_array` hidden-deprecated.
    let text = r#"{"on_death": 1, "on_bred_array": ["1"]}"#;
    let mut d = Diagnostics::default();
    let reward = ExperienceReward::read_v1_21_110(
        JsonSlice { text, offset: 0 },
        ConvertMode::Strict,
        &bedrock_addon_core::load::LoadContext::detached(),
        &mut d,
    )
    .expect("reads");
    assert_eq!(reward.on_bred_array, None, "dropped");
    // An error: it rejects the document in both masks.
    assert!(has(
        &d,
        |c| matches!(c.id(), "core/member_deprecated"),
        Severity::Error,
        "on_bred_array"
    ));
    assert_eq!(
        d.iter().next().map(|f| f.code.to_string()).as_deref(),
        Some("member \"on_bred_array\" is deprecated")
    );
    // The lenient conversion keeps them.
    let mut d = Diagnostics::default();
    let reward = ExperienceReward::read_v1_21_110(
        JsonSlice { text, offset: 0 },
        ConvertMode::Lenient,
        &bedrock_addon_core::load::LoadContext::detached(),
        &mut d,
    )
    .expect("reads");
    assert_eq!(reward.on_bred_array.map(|a| a.len()), Some(1));
    // From 1.26.0 the names are bound again: a plain member.
    let (reward, d) = strict::<ExperienceReward>(text);
    assert!(d.is_empty() && reward.on_bred_array.is_some());
}

#[test]
fn experience_reward_round_trips_each_form() {
    let reward = round_trip::<ExperienceReward>(
        r#"{"on_bred": "Math.Random(1,7)", "on_death": {"expression": "query.last_hit_by_player ? 5 : 0", "version": 12}}"#,
    );
    assert_eq!(
        write(&reward),
        json(
            r#"{"on_bred": "Math.Random(1,7)", "on_death": {"expression": "query.last_hit_by_player ? 5 : 0", "version": 12}}"#
        )
    );
}

#[test]
fn anger_level_reads_its_sounds_and_compiles_their_conditions() {
    let (mut anger, d) = strict::<AngerLevel>(
        r#"{"max_anger": 150, "angry_threshold": 80, "remove_targets_below_angry_threshold": true,
            "angry_boost": 20, "anger_decrement_interval": 1,
            "default_annoyingness": 35, "default_projectile_annoyingness": 10,
            "on_increase_sounds": [{"sound": "listening", "condition": "query.anger_level(this) >= 40"}],
            "nuisance_filter": {"all_of": [{"test": "is_family", "subject": "other", "operator": "not", "value": "warden"}]}}"#,
    );
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(anger.on_increase_sounds.len(), 1);
    assert_eq!(anger.on_increase_sounds[0].sound, "listening");
    let d = passes(&mut anger, &pack([1, 21, 100]));
    assert!(!d.has_errors(), "{d:?}");
    let mut component = EntityComponent::from(anger);
    let d = passes(&mut component, &pack([1, 21, 100]));
    assert!(!d.has_errors(), "the enum forwards the walk: {d:?}");
}

#[test]
fn anger_level_negative_annoyingness_is_an_error() {
    let (value, d) = read_as::<AngerLevel>(r#"{"default_annoyingness": -1}"#, ConvertMode::Strict);
    assert!(value.is_none() && d.has_errors());
}

#[test]
fn anger_level_sound_with_an_object_condition_is_dropped_by_the_lenient_conversion() {
    let (anger, d) = lenient::<AngerLevel>(
        r#"{"on_increase_sounds": [{"sound": "a", "condition": {"expression": "1", "version": 2}}, {"sound": "b", "condition": "1"}]}"#,
    );
    assert_eq!(anger.on_increase_sounds.len(), 1);
    assert_eq!(anger.on_increase_sounds[0].sound, "b");
    assert!(has(
        &d,
        |c| matches!(c.id(), "core/invalid_type"),
        Severity::Info,
        "on_increase_sounds/0/condition"
    ));
}

#[test]
fn e00d_experience_reward_strings_of_the_1_13_0_step_compile_at_the_pack_version() {
    // e00d "zombie baby / adult `experience_reward`": the
    // inserted strings are compiled like authored ones.
    for on_death in [
        "query.last_hit_by_player ? 12 + (query.equipment_count * Math.Random(1,3)) : 0",
        "query.last_hit_by_player ? 5 + (query.equipment_count * Math.Random(1,3)) : 0",
        "query.last_hit_by_player ? (query.is_baby ? 12 : 5) + (Math.die_roll(query.equipment_count,1,3)) : 0",
        "!query.is_baby && query.last_hit_by_player ? Math.Random(1,3) : 0",
        "Math.Min(query.player_level * 7, 100)",
    ] {
        let mut reward = ExperienceReward {
            on_death: Some(Molang::Expr(MolangSource::string_without_context(on_death))),
            on_bred: Some(Molang::Expr(MolangSource::string_without_context(
                "Math.Random(1,7)",
            ))),
            ..ExperienceReward::default()
        };
        for version in [[1, 13, 0], [1, 21, 100]] {
            let d = passes(&mut reward, &pack(version));
            assert!(!d.has_errors(), "{on_death} at {version:?}: {d:?}");
        }
    }
}

#[test]
fn m08_descriptor_tags_compile_at_13_with_the_tag_queries_only() {
    // m08: a block descriptor `{"tags": "query.any_tag('stone')"}` in a component
    // typed at the document's version compiles at 13, no diagnostic, in a [1, 16, 0] pack;
    // `{"tags": "query.is_baby"}` is `UnknownQuery` + `Syntax`, the descriptor kept: warnings
    // (the game loads the entity).
    let (mut good, _) = strict::<BlockMovementSlowdownImmunity>(
        r#"{"blocks": [{"tags": "query.any_tag('stone')"}]}"#,
    );
    let d = passes(&mut good, &pack([1, 16, 0]));
    assert!(d.is_empty(), "{d:?}");
    let (mut bad, _) =
        strict::<BlockMovementSlowdownImmunity>(r#"{"blocks": [{"tags": "query.is_baby"}]}"#);
    let d = passes(&mut bad, &pack([1, 16, 0]));
    assert_eq!(bad.blocks.len(), 1, "the descriptor is kept");
    #[cfg(feature = "molang")]
    {
        use bedrock_addon_core::molang::DiagCode;
        let codes: Vec<DiagCode> = molang_codes(&d)
            .into_iter()
            .filter(|(s, _)| *s == Severity::Error)
            .map(|(_, c)| c)
            .collect();
        assert_eq!(
            codes,
            vec![DiagCode::UnknownQuery, DiagCode::Syntax],
            "{d:?}"
        );
        assert!(!d.has_errors(), "{d:?}");
        assert!(has(
            &d,
            |c| matches!(c.id(), "core/invalid_molang"),
            Severity::Error,
            "blocks/0/tags"
        ));
    }
    #[cfg(not(feature = "molang"))]
    assert!(d.is_empty());
}

/// A detached load context with the compile check, the pack at Molang 13.
#[cfg(feature = "molang")]
fn checking() -> bedrock_addon_core::load::LoadContext {
    let mut cx = bedrock_addon_core::load::LoadContext::detached();
    cx.molang = Some(std::sync::Arc::new(
        bedrock_addon_core::molang::MolangCheck::new(),
    ));
    cx
}

#[cfg(feature = "molang")]
#[test]
fn a_lenient_experience_reward_string_that_does_not_compile_is_a_warning_and_the_default() {
    // `experience_reward` row: "lenient conversion: `InvalidMolang` warning
    // and the field default 0".
    let cx = checking();
    let (value, d) = super::read_with(
        "minecraft:experience_reward",
        r#"{"on_death": "1 +", "on_bred": "Math.Random(1,7)"}"#,
        ConvertMode::Lenient,
        &cx,
    );
    let Some(EntityComponent::ExperienceReward(reward)) = value else {
        panic!("kept: {d:?}")
    };
    assert_eq!(reward.on_death, None, "the field default");
    assert!(reward.on_bred.is_some(), "a compiling string is kept");
    assert!(!d.has_errors(), "{d:?}");
    assert!(has(
        &d,
        |c| matches!(c.id(), "core/invalid_molang"),
        Severity::Error,
        "on_death"
    ));
    // A strict read keeps the value; pass 2 reports the error there.
    let (value, d) = super::read_with(
        "minecraft:experience_reward",
        r#"{"on_death": "1 +"}"#,
        ConvertMode::Strict,
        &cx,
    );
    assert!(d.is_empty() && value.is_some(), "{d:?}");
}

#[cfg(feature = "molang")]
#[test]
fn a_lenient_anger_sound_whose_condition_does_not_compile_is_dropped_with_a_warning() {
    let cx = checking();
    let (value, d) = super::read_with(
        "minecraft:anger_level",
        r#"{"on_increase_sounds": [{"sound": "a", "condition": "1 +"}, {"sound": "b", "condition": "1"}]}"#,
        ConvertMode::Lenient,
        &cx,
    );
    let Some(EntityComponent::AngerLevel(anger)) = value else {
        panic!("kept: {d:?}")
    };
    assert_eq!(anger.on_increase_sounds.len(), 1);
    assert!(has(
        &d,
        |c| matches!(c.id(), "core/invalid_molang"),
        Severity::Error,
        "on_increase_sounds/0/condition"
    ));
}
