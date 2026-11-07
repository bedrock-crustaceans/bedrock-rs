//! The goal rows of and the goal rules of.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Diagnostics, Severity};
use bedrock_addon_core::document::Component;
use bedrock_addon_core::versioning::ConvertMode;
use bedrock_addon_family_entity::component::EntityComponent;
use bedrock_addon_family_entity::goal;

use super::{assert_finding, errors, finding, fixture_member, goal, strict};

/// The goal of type `T` the strict parse reads from fixture `name`'s member `key`.
fn strict_fixture<T: Component<Set = EntityComponent>>(name: &str, key: &str) -> (T, Diagnostics) {
    let (value, d) = goal::<T>(key, &fixture_member(name, key), ConvertMode::Strict);
    (value.unwrap_or_else(|| panic!("{key} is kept: {d:?}")), d)
}

/// `v07_strict_vs_lenient`: the same bad `melee_attack` is a warning and kept in a
/// lenient conversion (1.21.130), an error and dropped in a strict parse (1.26.0).
#[test]
fn v07_a_bad_melee_attack_is_kept_leniently_and_dropped_strictly() {
    let json = r#"{"priority": 2, "speed_multiplier": "fast", "track_target": true}"#;
    let (kept, d) = goal::<goal::MeleeAttack>(
        "minecraft:behavior.melee_attack",
        json,
        ConvertMode::Lenient,
    );
    let kept = kept.expect("the lenient conversion keeps the goal");
    assert_eq!(
        kept.speed_multiplier, None,
        "the bad member takes its default"
    );
    assert_eq!(kept.speed_multiplier(), 1.0);
    assert_eq!(kept.priority(), 2);
    assert!(kept.track_target());
    assert_finding(&d, "InvalidValue", "speed_multiplier", Severity::Info);
    assert!(errors(&d).is_empty());

    let (dropped, d) = strict("minecraft:behavior.melee_attack", json);
    assert!(dropped.is_none(), "the strict parse drops the goal");
    assert_finding(&d, "InvalidType", "speed_multiplier", Severity::Error);
}

/// `v13_deprecated_inherited_names`: `delayed_attack.cooldown_time`,
/// `stomp_attack.melee_fov` and `hurt_by_target.must_see` are `DeprecatedMember` with vanilla's
/// text and are dropped: errors in a typed read (in both masks), which
/// reject the document; `move_outdoors.search_count`, deprecated and re-bound, is an ordinary
/// member.
#[test]
fn v13_deprecated_inherited_names_are_reported_and_dropped() {
    const NAME: &str = "v13_deprecated_inherited_names";
    let deprecated = |d: &Diagnostics, member: &str, text: &'static str| {
        let found =
            finding(d, "DeprecatedMember", member).unwrap_or_else(|| panic!("{member}: {d:?}"));
        assert_eq!(found.severity, Severity::Error);
        assert_eq!(
            found.code,
            core_codes::DEPRECATED_MEMBER_WORDED.with([text])
        );
        assert_eq!(d.len(), 1, "only the deprecated member is reported: {d:?}");
    };

    let (delayed, d) =
        strict_fixture::<goal::DelayedAttack>(NAME, "minecraft:behavior.delayed_attack");
    deprecated(
        &d,
        "cooldown_time",
        "member \"cooldown_time\" is deprecated",
    );
    assert!(
        d.unknown()
            .members_at(&bedrock_addon_core::diagnostic::JsonPath::new())
            .next()
            .is_none(),
        "the deprecated member is dropped"
    );
    assert_eq!(delayed.attack_duration(), 0.5);

    let (stomp, d) = strict_fixture::<goal::StompAttack>(NAME, "minecraft:behavior.stomp_attack");
    deprecated(&d, "melee_fov", "member \"melee_fov\" is deprecated");
    let _ = stomp;
    assert!(
        d.unknown()
            .members_at(&bedrock_addon_core::diagnostic::JsonPath::new())
            .next()
            .is_none()
    );
    assert_eq!(stomp.stomp_range_multiplier(), 2.5);

    let (hurt, d) = strict_fixture::<goal::HurtByTarget>(NAME, "minecraft:behavior.hurt_by_target");
    deprecated(&d, "must_see", "member \"must_see\" is deprecated");
    let _ = hurt;
    assert!(
        d.unknown()
            .members_at(&bedrock_addon_core::diagnostic::JsonPath::new())
            .next()
            .is_none()
    );
    assert!(hurt.hurt_owner());

    let (outdoors, d) =
        strict_fixture::<goal::MoveOutdoors>(NAME, "minecraft:behavior.move_outdoors");
    assert!(d.is_empty(), "a re-bound name is not reported: {d:?}");
    assert_eq!(outdoors.search_count(), 12);
}
