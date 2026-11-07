//! Beta items.

use bedrock_addon_core::document::Content;
use bedrock_addon_core::pack_identity::PackPath;
use bedrock_addon_family_item::component::{
    AllowOffHand, DurabilitySensor, Record, SwingSounds, UseModifiers,
};
use bedrock_addon_family_item::{Beta, Item};

use super::common::*;

/// A document whose sound members name `sound`.
fn sounds(version: &str, sound: &str) -> String {
    format!(
        r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x"}}, "components": {{
            "minecraft:use_modifiers": {{"use_duration": 1.0, "start_sound": "{sound}"}},
            "minecraft:swing_sounds": {{"attack_miss": "{sound}", "attack_hit": "{sound}", "attack_critical_hit": "{sound}"}},
            "minecraft:record": {{"sound_event": "{sound}", "duration": 5.0}},
            "minecraft:durability": {{"max_durability": 10}},
            "minecraft:durability_sensor": {{"durability_thresholds": [{{"durability": 5, "sound_event": "{sound}"}}]}},
            "minecraft:allow_off_hand": true,
            "minecraft:max_stack_size": 1
        }}}}}}"#
    )
}

/// The beta context: Beta APIs on.
fn beta_apis() -> bedrock_addon_core::load::LoadContext {
    let mut cx = third_party();
    cx.beta_apis = true;
    cx
}

#[test]
fn beta_sound_members_take_event_and_definition_names() {
    // an enum name (`record.cat`) and a sound_definitions.json name (`demo.custom`); both are
    // kept as written (the LevelSoundEvent names are not accepted here)
    for sound in ["record.cat", "demo.custom"] {
        let document = Beta::parse(
            &PackPath::from_static("items/x.json"),
            sounds("beta", sound).as_bytes(),
            &beta_apis(),
        )
        .unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
        let c = &document.components;
        assert_eq!(
            c.get::<UseModifiers>()
                .unwrap()
                .start_sound
                .as_ref()
                .unwrap()
                .as_str(),
            sound
        );
        let swing = c.get::<SwingSounds>().unwrap();
        for name in [
            &swing.attack_miss,
            &swing.attack_hit,
            &swing.attack_critical_hit,
        ] {
            assert_eq!(name.as_ref().unwrap().as_str(), sound);
        }
        assert_eq!(
            c.get::<Record>()
                .unwrap()
                .sound_event
                .as_ref()
                .unwrap()
                .as_str(),
            sound
        );
        let sensor = c.get::<DurabilitySensor>().unwrap();
        assert_eq!(
            sensor.durability_thresholds[0]
                .sound_event
                .as_ref()
                .unwrap()
                .as_str(),
            sound
        );
        // allow_off_hand is the 1.20.50 type
        assert!(c.get::<AllowOffHand>().unwrap().value);
    }
    // the same content at 1.26.30 stops at the newest stable struct
    let stable = Item::parse(
        &PackPath::from_static("items/x.json"),
        sounds("1.26.30", "record.cat").as_bytes(),
        &beta_apis(),
    )
    .unwrap();
    assert!(!stable.version().beta);
}

#[test]
fn a_beta_document_needs_the_beta_apis() {
    let d = match Beta::parse(
        &PackPath::from_static("items/x.json"),
        sounds("beta", "record.cat").as_bytes(),
        &third_party(),
    ) {
        Ok(_) => panic!("loaded"),
        Err(rejected) => rejected.diagnostics,
    };
    assert!(has(&d, |c| matches!(c.id(), "core/beta_documents")));
    // a base-game pack may not either: the JSON path gates on the toggle alone
    // `a_json_beta_item_needs_the_beta_apis_even_in_a_base_game_pack`)
}

/// The finding codes above info of a load, as names.
/// The `UnrecognizedField` findings of `d`: their paths and severities.
fn unknown(
    d: &bedrock_addon_core::diagnostic::Diagnostics,
) -> Vec<(String, bedrock_addon_core::diagnostic::Severity)> {
    d.iter()
        .filter(|f| f.code.id() == "core/unrecognized_field")
        .map(|f| (f.location.json_path.to_string(), f.severity))
        .collect()
}

fn told(d: &bedrock_addon_core::diagnostic::Diagnostics) -> Vec<(String, &'static str)> {
    above_info(d)
        .into_iter()
        .map(|f| (f.location.json_path.to_string(), f.code.id()))
        .collect()
}

/// `use_beta_features` is in no 1.26.60 string; the item path never
/// counts the root's members. A root `use_beta_features: true` neither gates the item
/// nor refuses it: it is a root member nothing reads, told where it is.
#[test]
fn a_root_use_beta_features_neither_gates_nor_rejects() {
    for flag in ["true", "false", "7"] {
        let text = item("1.21.50", "demo:x", r#""minecraft:icon": "stick""#).replacen(
            '{',
            &format!(r#"{{"use_beta_features": {flag}, "#),
            1,
        );
        let document = loads(&text, &third_party());
        // An information of ours.
        assert_eq!(told(document.diagnostics()), [], "{flag}");
        assert_eq!(
            unknown(document.diagnostics()),
            [(
                "use_beta_features".to_owned(),
                bedrock_addon_core::diagnostic::Severity::Info
            )],
            "{flag}"
        );
    }
}

/// Any third root member neither refuses nor gates an item.
#[test]
fn a_third_root_member_does_not_reject_an_item() {
    let text = item("1.26.30", "demo:x", r#""minecraft:icon": "stick""#).replacen(
        '{',
        r#"{"demo:note": {"a": 1}, "#,
        1,
    );
    let document = loads(&text, &third_party());
    // An information of ours.
    assert_eq!(told(document.diagnostics()), []);
    assert_eq!(
        unknown(document.diagnostics()),
        [(
            "demo:note".to_owned(),
            bedrock_addon_core::diagnostic::Severity::Info
        )]
    );
}

/// A JSON item's beta `format_version` passes `validateParsedVersion`
/// with the Beta APIs toggle alone; a base-game pack without it is refused with the version text.
#[test]
fn a_json_beta_item_needs_the_beta_apis_even_in_a_base_game_pack() {
    let mut base = third_party();
    base.is_base_game_pack = true;
    let d = match Beta::parse(
        &PackPath::from_static("items/x.json"),
        sounds("beta", "record.cat").as_bytes(),
        &base,
    ) {
        Ok(_) => panic!("loaded"),
        Err(rejected) => rejected.diagnostics,
    };
    assert!(has(&d, |c| matches!(c.id(), "core/beta_documents")));
    assert!(
        has_text(
            &d,
            "Turn on the Beta APIs toggle in settings to allow beta documents."
        ),
        "{d:#?}"
    );
}

/// A binary (MCB) item skips `upgradeJson` and meets the version check of
/// `CerealParser<Beta::ItemDocument>::doParse`: a base-game pack or the toggle.
#[test]
fn an_mcb_beta_item_takes_the_version_gate() {
    let mcb = |cx: &mut bedrock_addon_core::load::LoadContext| {
        cx.source = bedrock_addon_core::load::DocumentSource::Mcb;
    };
    let mut plain = third_party();
    mcb(&mut plain);
    let d = match Beta::parse(
        &PackPath::from_static("items/x.json"),
        sounds("beta", "record.cat").as_bytes(),
        &plain,
    ) {
        Ok(_) => panic!("loaded"),
        Err(rejected) => rejected.diagnostics,
    };
    assert!(
        has_text(
            &d,
            "Using a beta format_version is only allowed for base packs or when the BetaApis toggle is enabled."
        ),
        "{d:#?}"
    );
    let mut base = third_party();
    base.is_base_game_pack = true;
    mcb(&mut base);
    let mut toggle = beta_apis();
    mcb(&mut toggle);
    for cx in [base, toggle] {
        Beta::parse(
            &PackPath::from_static("items/x.json"),
            sounds("beta", "record.cat").as_bytes(),
            &cx,
        )
        .unwrap_or_else(|r| panic!("{:#?}", r.diagnostics));
    }
}

/// The beta `allow_off_hand` defaults to `value: true`
/// (`TypeSchema<Beta::AllowOffHandItemComponent>::doLoad`); the stable one to false. A bare
/// value or a written `value` is read as written in both.
#[test]
fn a_beta_allow_off_hand_defaults_to_true() {
    let document = |version: &str, value: &str| {
        format!(
            r#"{{"format_version": "{version}", "minecraft:item": {{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:allow_off_hand": {value}, "minecraft:max_stack_size": 1}}}}}}"#
        )
    };
    let beta = |value: &str| {
        Beta::parse(
            &PackPath::from_static("items/x.json"),
            document("beta", value).as_bytes(),
            &beta_apis(),
        )
        .unwrap_or_else(|r| panic!("{:#?}", r.diagnostics))
        .components
        .get::<AllowOffHand>()
        .unwrap()
        .value
    };
    let stable = |value: &str| {
        loads(&document("1.26.50", value), &third_party())
            .components
            .get::<AllowOffHand>()
            .unwrap()
            .value
    };
    assert!(beta("{}"));
    assert!(!stable("{}"));
    for (value, expected) in [
        ("true", true),
        ("false", false),
        (r#"{"value": false}"#, false),
        (r#"{"value": true}"#, true),
    ] {
        assert_eq!(beta(value), expected, "{value}");
        assert_eq!(stable(value), expected, "{value}");
    }
}

/// The values the game gives a `record` or a durability threshold
/// member the document leaves out; the crate keeps the member unset (the writer leaves it out
/// again) and states the value.
#[test]
fn the_defaults_of_record_and_of_a_durability_threshold() {
    use bedrock_addon_family_item::component::{DurabilityThreshold, Record};
    let record = Record::default();
    assert_eq!(record.effective_comparator_signal(), 1);
    assert_eq!(record.effective_duration(), 0.0);
    assert_eq!(record.effective_beta_sound_event(), "");
    assert_eq!(Record::STABLE_DEFAULT_SOUND_EVENT, 615);
    let threshold = DurabilityThreshold::default();
    assert_eq!(threshold.effective_sound_event(), "");
    assert_eq!(DurabilityThreshold::DEFAULT_PARTICLE_TYPE, 0);
    // written members are read as written
    let written = Beta::parse(
        &PackPath::from_static("items/x.json"),
        sounds("beta", "record.cat").as_bytes(),
        &beta_apis(),
    )
    .unwrap();
    let record = written.components.get::<Record>().unwrap();
    assert_eq!(record.effective_duration(), 5.0);
    assert_eq!(record.effective_beta_sound_event(), "record.cat");
}

/// The modern documents bind `components` required with
/// `ComponentStorageConstraint::minSize(1)` (1.26.30, 1.26.50, beta). A stable document without a registered component is routed
/// to the legacy item format before the binding is reached (routing wins); a beta document is not routed, and an
/// empty or absent `components` refuses it.
#[test]
fn routing_wins_over_the_components_min_size_and_beta_meets_it() {
    for version in ["1.26.30", "1.26.50"] {
        let document = loads(&item(version, "demo:x", ""), &third_party());
        assert_eq!(
            document.legacy.format,
            bedrock_addon_family_item::ItemFormat::Legacy,
            "{version}"
        );
    }
    let empty = r#"{"format_version": "beta", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {}}}"#;
    let d = match Beta::parse(
        &PackPath::from_static("items/x.json"),
        empty.as_bytes(),
        &beta_apis(),
    ) {
        Ok(_) => panic!("loaded"),
        Err(rejected) => rejected.diagnostics,
    };
    assert!(
        d.iter().any(
            |f| f.severity == bedrock_addon_core::diagnostic::Severity::Error
                && f.code.to_string() == "too few input components, expected at least 1"
                && f.location.json_path.to_string() == "minecraft:item/components"
        ),
        "{d:#?}"
    );
    let absent = r#"{"format_version": "beta", "minecraft:item": {"description": {"identifier": "demo:x"}}}"#;
    let d = match Beta::parse(
        &PackPath::from_static("items/x.json"),
        absent.as_bytes(),
        &beta_apis(),
    ) {
        Ok(_) => panic!("loaded"),
        Err(rejected) => rejected.diagnostics,
    };
    assert!(
        d.iter().any(
            |f| f.severity == bedrock_addon_core::diagnostic::Severity::Error
                && matches!(f.code.id(), "core/missing_required_field")
                && f.location.json_path.to_string() == "minecraft:item/components"
        ),
        "{d:#?}"
    );
}

/// A beta file read as an `Item` (`Content::parse`, which keeps only a stable item) is refused
/// with the shared "no parser available" for `beta`, told in its file, as every single-type
/// read refuses one.
#[test]
fn a_beta_file_read_as_an_item_is_refused_in_its_file() {
    use bedrock_addon_core::pack_identity::PackRef;

    let path = PackPath::new("items/x.json").unwrap();
    let text = r#"{"format_version": "beta", "minecraft:item": {"description": {"identifier": "demo:x"}, "components": {"minecraft:max_stack_size": 2}}}"#;
    let refused = <Item as Content>::parse(&path, text.as_bytes(), &beta_apis()).unwrap_err();
    let refusal = refused
        .diagnostics
        .iter()
        .find(|d| matches!(d.code.id(), "core/no_parser_for_version"))
        .unwrap_or_else(|| panic!("{:#?}", refused.diagnostics));
    assert_eq!(
        refusal.code,
        bedrock_addon_core::diagnostic::codes::NO_PARSER_FOR_VERSION.with(["beta"]),
        "{refusal:?}"
    );
    assert_eq!(refusal.location.pack, Some(PackRef::app_data()));
    assert_eq!(refusal.location.path, Some(path));
}
