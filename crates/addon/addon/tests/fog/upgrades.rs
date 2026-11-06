//! The upgrades of the fog chain (`u01`–`u08`) and the history API.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::JsonDocument as _;
use bedrock_addon::families::fog::history::{self, FogHistory, v1_16_100};
use bedrock_addon::families::fog::{ColorNorm, DistanceType, FogColor, FogSettings, Keyframed};
use bedrock_addon::storage::PackPath;

use super::common::{
    clean, count, expect, expect_at, find, fixture, lines, loaded, rejected, round_trip,
};
use super::support::{
    AIR, FAMILY, V1_16_100, V1_21_0, V1_21_90, V1_26_20, air, base_game, density, doc, pack, parse,
    refused,
};

fn as_written(text: &str) -> bedrock_addon::document::Document<FogHistory> {
    loaded(FogSettings::parse_as_written(
        &PackPath::from_static("fogs/x.json"),
        text.as_bytes(),
        &base_game(),
    ))
}

#[test]
fn u01_identity_1_16_100() {
    let text = fixture(FAMILY, "default_fog_setting.json");
    let written = as_written(&text);
    assert_eq!(written.parsed_version(), V1_16_100);
    let FogHistory::V1_16_100(old) = written.value() else {
        panic!("{:?}", written.value());
    };
    assert_eq!(old.format_version, "1.16.100");
    assert_eq!(old.description.identifier, "minecraft:fog_default");
    let old_air = old.distance.as_ref().unwrap().air.as_ref().unwrap();
    assert_eq!(
        old_air.fog_color,
        v1_16_100::FogColorJu::Hex("#ABD2FF".to_owned())
    );
    assert_eq!(old_air.render_distance_type, DistanceType::Render);

    // The document in its own shape reads back as the same value.
    let own = written.to_json().unwrap();
    assert_eq!(as_written(&own).value(), written.value());
    assert_eq!(as_written(&own).to_json().unwrap(), own);

    // Upgrading the document gives what loading the file gives.
    let upgraded = loaded(history::upgrade(written, &base_game()));
    let direct = loaded(parse(&text, &base_game()));
    assert_eq!(upgraded.value(), direct.value());
    assert_eq!(upgraded.version(), direct.version());
    clean(upgraded.diagnostics());
    // Every member the old file wrote is there, unchanged.
    let air = direct.distance.as_ref().unwrap().air.as_ref().unwrap();
    assert_eq!(
        (air.fog_start, air.fog_end, air.render_distance_type),
        (Some(0.92), Some(1.0), Some(DistanceType::Render))
    );
    assert_eq!(air.fog_color, Some(FogColor::Hex("#ABD2FF".to_owned())));
    round_trip(&direct, |t| parse(t, &base_game()));
    identity_of_a_vanilla_1_16_100_file("default_fog_setting.json", &text);
}

/// The payload member of a fog file's text, as a JSON value.
fn payload_of(text: &str) -> bedrock_addon::json::Value {
    let root = bedrock_addon::json::from_str_jsonc::<bedrock_addon::json::Value>(text)
        .unwrap_or_else(|e| panic!("{e}: {text}"));
    root.as_object()
        .and_then(|root| root.get("minecraft:fog_settings"))
        .unwrap_or_else(|| panic!("no payload: {text}"))
        .clone()
}

/// Row u01 for one file of the game: the 1.16.100 document, taken through both identity steps
/// and the last one, is what loading the file gives, without a finding; and the payload that
/// is written is the payload the file has, member for member ("before == after").
fn identity_of_a_vanilla_1_16_100_file(name: &str, text: &str) {
    let written = as_written(text);
    assert_eq!(written.parsed_version(), V1_16_100, "{name}");
    assert!(
        matches!(written.value(), FogHistory::V1_16_100(_)),
        "{name}"
    );
    // The document in its own shape reads back as the same value.
    let own = written.to_json().unwrap();
    assert_eq!(as_written(&own).value(), written.value(), "{name}");
    assert_eq!(payload_of(&own), payload_of(text), "{name}");

    let upgraded = loaded(history::upgrade(written, &base_game()));
    let direct = loaded(parse(text, &base_game()));
    assert_eq!(upgraded.value(), direct.value(), "{name}");
    assert_eq!(upgraded.version(), direct.version(), "{name}");
    assert!(
        upgraded.diagnostics().is_empty() && direct.diagnostics().is_empty(),
        "{name}: {:?} {:?}",
        lines(upgraded.diagnostics()),
        lines(direct.diagnostics())
    );
    // Before == after: the newest shape has every member of the old file, unchanged.
    let after = direct.to_json().unwrap();
    assert_eq!(payload_of(&after), payload_of(text), "{name}");
    round_trip(&direct, |t| parse(t, &base_game()));
}

/// Row u01, "every vanilla 1.16.100 file": each `fogs/*.json` at 1.16.100 that the corpus has
/// in the game's sample packs and in the packs of the retail build (the vanilla files are
/// read from the corpus outside the repository; without it the test skips).
#[test]
fn u01_identity_of_every_vanilla_1_16_100_file_of_the_corpus() {
    let Some(corpus) = std::env::var_os("BEDROCK_ADDON_CORPUS") else {
        eprintln!("BEDROCK_ADDON_CORPUS is not set: skipping the vanilla fog files of row u01");
        return;
    };
    let corpus = std::path::PathBuf::from(corpus);
    let mut folders = vec![corpus.join("bedrock-samples/resource_pack/fogs")];
    let mut packs: Vec<_> = std::fs::read_dir(corpus.join("gdk-retail/resource_packs"))
        .map(|packs| {
            packs
                .map(|pack| pack.unwrap().path().join("fogs"))
                .collect()
        })
        .unwrap_or_default();
    packs.sort();
    folders.extend(packs);
    let (mut checked, mut newer) = (0, 0);
    for folder in folders.iter().filter(|folder| folder.is_dir()) {
        let mut files: Vec<_> = std::fs::read_dir(folder)
            .unwrap()
            .map(|file| file.unwrap().path())
            .filter(|file| file.extension().is_some_and(|e| e == "json"))
            .collect();
        files.sort();
        for file in files {
            let name = file.display().to_string();
            let text = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{name}: {e}"));
            if as_written(&text).parsed_version() == V1_16_100 {
                identity_of_a_vanilla_1_16_100_file(&name, &text);
                checked += 1;
            } else {
                newer += 1;
            }
        }
    }
    eprintln!("u01: {checked} vanilla files at 1.16.100 are identities; {newer} of a later format");
    // The folder counts 30 in the 1.16.100 pack alone.
    assert!(checked >= 30, "{checked} files at 1.16.100");
}

#[test]
fn u01_each_older_struct_writes_its_own_shape() {
    let weather = r##""volumetric": {"density": {"weather": {"max_density": 0.5, "uniform": true}}, "media_coefficients": {"cloud": {"scattering": [0.5, 2, 0], "absorption": "#FFffFF"}}}"##;
    let hg = r#""volumetric": {"henyey_greenstein_g": {"air": {"henyey_greenstein_g": -0.5}}}"#;
    for (version, members, parsed_as) in [
        (
            "1.20.80",
            density(r#""max_density": 0.5, "uniform": true"#),
            V1_16_100,
        ),
        ("1.21.0", weather.to_owned(), V1_21_0),
        ("1.21.90", hg.to_owned(), V1_21_90),
        ("1.26.20", air(AIR), V1_26_20),
    ] {
        let text = doc(version, &members);
        let written = loaded(FogSettings::parse_as_written(
            &PackPath::from_static("fogs/x.json"),
            text.as_bytes(),
            &pack(),
        ));
        assert_eq!(written.parsed_version(), parsed_as, "{version}");
        let own = written.to_json().unwrap();
        assert!(
            own.contains(&format!("\"format_version\": \"{version}\"")),
            "{own}"
        );
        let again = loaded(FogSettings::parse_as_written(
            &PackPath::from_static("fogs/x.json"),
            own.as_bytes(),
            &pack(),
        ));
        assert_eq!(again.value(), written.value(), "{own}");
        let upgraded = loaded(history::upgrade(written, &pack()));
        assert_eq!(upgraded.value(), loaded(parse(&text, &pack())).value());
    }
}

#[test]
fn u02_to_cereal_all_required() {
    let before = loaded(parse(
        &fixture(FAMILY, "u02_to_cereal_all_required.before.json"),
        &pack(),
    ));
    let after = loaded(parse(
        &fixture(FAMILY, "u02_to_cereal_all_required.after.json"),
        &pack(),
    ));
    assert_eq!(before.value(), after.value());
    assert!(before.version().upgraded && !after.version().upgraded);
    // Every member of a converted entry is `Some`.
    let distance = before.distance.as_ref().unwrap();
    let air = distance.air.as_ref().unwrap();
    assert!(air.fog_start.is_some() && air.fog_end.is_some());
    assert_eq!(air.render_distance_type, Some(DistanceType::Render));
    let transition = distance
        .water
        .as_ref()
        .unwrap()
        .transition_fog
        .as_ref()
        .unwrap();
    let init = transition.init_fog.as_ref().unwrap();
    assert_eq!(
        (init.fog_start, init.fog_end, init.render_distance_type),
        (Some(0.0), Some(0.01), Some(DistanceType::Fixed))
    );
    assert_eq!(
        [
            transition.min_percent,
            transition.mid_seconds,
            transition.mid_percent,
            transition.max_seconds
        ],
        [Some(0.25), Some(5.0), Some(0.6), Some(30.0)]
    );
    // The one finding of the pair: the transition of `air`.
    assert_eq!(
        count(before.diagnostics(), "DroppedField", Severity::Info),
        1
    );
    clean(before.diagnostics());
    assert!(
        after.diagnostics().is_empty(),
        "{:?}",
        lines(after.diagnostics())
    );
    round_trip(&before, |t| parse(t, &pack()));
}

/// TOML `vanilla = "kept in FogSetting::mTransitionSetting for every distance type"`.
#[test]
fn u03_nonwater_transition_dropped_deviation() {
    let transition = r##""transition_fog": {"init_fog": {"fog_start": 0, "fog_end": 1, "fog_color": "#000000", "render_distance_type": "fixed"}, "min_percent": 0.2, "mid_seconds": 2, "mid_percent": 0.5, "max_seconds": 4}"##;
    for entry in ["air", "weather", "lava", "lava_resistance", "powder_snow"] {
        let members = format!(r#""distance": {{"{entry}": {{{AIR}, {transition}}}}}"#);
        let document = loaded(parse(&doc("1.21.90", &members), &pack()));
        let dropped = find(document.diagnostics(), "DroppedField", Severity::Info).unwrap();
        assert_eq!(
            dropped.location.json_path.to_string(),
            format!("minecraft:fog_settings/distance/{entry}/transition_fog")
        );
        assert_eq!(
            count(document.diagnostics(), "DroppedField", Severity::Info),
            1
        );
        let text = document.to_json().unwrap();
        assert!(!text.contains("transition_fog"), "{text}");
    }
    // Water keeps it, without a word.
    let members = format!(r#""distance": {{"water": {{{AIR}, {transition}}}}}"#);
    let document = loaded(parse(&doc("1.21.90", &members), &pack()));
    assert!(document.diagnostics().is_empty());
    assert!(
        document
            .distance
            .as_ref()
            .unwrap()
            .water
            .as_ref()
            .unwrap()
            .transition_fog
            .is_some()
    );
}

fn air_color(document: &FogSettings) -> FogColor {
    document
        .distance
        .as_ref()
        .unwrap()
        .air
        .as_ref()
        .unwrap()
        .fog_color
        .clone()
        .unwrap()
}

#[test]
fn u04_int_array_colour() {
    let members = air(&AIR.replace("\"#ABD2FF\"", "[128, 64, 255]"));
    let old = loaded(parse(&doc("1.21.90", &members), &pack()));
    assert!(
        old.diagnostics().is_empty(),
        "{:?}",
        lines(old.diagnostics())
    );
    assert_eq!(air_color(&old), FogColor::Array([128, 64, 255]));
    assert_eq!(
        air_color(&old).unit_rgb(),
        [128.0 / 255.0, 64.0 / 255.0, 1.0]
    );
    let new = loaded(parse(&doc("1.26.20", &members), &pack()));
    assert_eq!(old.value(), new.value());
}

/// TOML `vanilla = "Expected Integral Number. (LogArea Json), component 255, file loaded"`.
#[test]
fn u05_colour_array_old_era_deviation() {
    let with = |color: &str| doc("1.16.100", &air(&AIR.replace("\"#ABD2FF\"", color)));
    // Clamped when read; vanilla says nothing, we note each clamped component.
    let document = loaded(parse(&with("[300, -5, 128]"), &pack()));
    assert_eq!(air_color(&document), FogColor::Array([255, 0, 128]));
    assert_eq!(air_color(&document).unit_rgb(), [1.0, 0.0, 128.0 / 255.0]);
    assert_eq!(
        count(document.diagnostics(), "ClampedValue", Severity::Info),
        2
    );
    clean(document.diagnostics());
    // The bounds themselves are not clamped.
    let document = loaded(parse(&with("[0, 255, 1]"), &pack()));
    assert!(document.diagnostics().is_empty());

    // A real is no integral number: 255, with vanilla's text, and the file loads. The line
    // is level 3 (as are the other elements'): a kept
    // error.
    let document = loaded(parse(&with("[0.5, 1.0, 0]"), &pack()));
    assert_eq!(air_color(&document), FogColor::Array([255, 255, 0]));
    assert_eq!(air_color(&document).unit_rgb(), [1.0, 1.0, 0.0]);
    assert_eq!(
        count(document.diagnostics(), "InvalidValue", Severity::Error),
        2
    );
    assert!(document.diagnostics().iter().all(|f| f.kept));
    expect(
        document.diagnostics(),
        "InvalidValue",
        Severity::Error,
        "Expected Integral Number.",
    );
    // A bool is integral, a fourth element is ignored (and noted).
    let document = loaded(parse(&with("[true, 7, false, 9]"), &pack()));
    assert_eq!(air_color(&document), FogColor::Array([1, 7, 0]));
    // A bool is read as vanilla reads it, and said at the element (ours): the colour
    // is written with the numbers. The game reads it in silence, so each is an information.
    let told: Vec<_> = document
        .diagnostics()
        .iter()
        .map(|f| (f.code.name(), f.severity, f.location.json_path.to_string()))
        .collect();
    let at = |index: usize| format!("minecraft:fog_settings/distance/air/fog_color/{index}");
    assert_eq!(
        told,
        [
            ("DroppedField", Severity::Info, at(3)),
            ("InvalidType", Severity::Info, at(0)),
            ("InvalidType", Severity::Info, at(2)),
        ]
    );
    // The schema's colour validator reads each element as a float, which fails
    // a `null` in silence and a string with "Expected Number." (level 3); the failed validation refuses the file with "Failed to validate fog
    // schema" before it is registered.
    let d = rejected(parse(&with("[0, null, 0]"), &pack()));
    refused(&d);
    assert!(
        d.iter().all(|f| f.code.to_string() != "Expected Number."),
        "{:?}",
        lines(&d)
    );
    let d = rejected(parse(&with(r#"[0, "1", 0]"#), &pack()));
    refused(&d);
    expect(&d, "InvalidType", Severity::Error, "Expected Number.");

    // A coefficient is clamped to 0–1 when it is used, silently.
    let media = r#""volumetric": {"media_coefficients": {"air": {"scattering": [2.0, -1.0, 0.5], "absorption": [0, 0, 0]}}}"#;
    let document = loaded(parse(&doc("1.16.100", media), &pack()));
    assert!(document.diagnostics().is_empty());
    let media = document
        .volumetric
        .as_ref()
        .unwrap()
        .media_coefficients
        .as_ref()
        .unwrap();
    let scattering = media.air.as_ref().unwrap().scattering.as_ref().unwrap();
    assert_eq!(
        scattering,
        &Keyframed::Single(ColorNorm::Array([2.0, -1.0, 0.5]))
    );
    assert_eq!(scattering.single().unwrap().unit_rgb(), [1.0, 0.0, 0.5]);
}

#[test]
fn u06_hex_colour_both_eras() {
    let members = air(&AIR.replace("#ABD2FF", "#44AFF5"));
    let old = loaded(parse(&doc("1.21.90", &members), &pack()));
    assert_eq!(air_color(&old), FogColor::Hex("#44AFF5".to_owned()));
    let new = loaded(parse(&doc("1.26.20", &members), &pack()));
    assert_eq!(old.value(), new.value());
    assert_eq!(
        air_color(&new).unit_rgb(),
        [68.0 / 255.0, 175.0 / 255.0, 245.0 / 255.0]
    );
}

#[test]
fn u07_density_rules_refuse_an_older_file_too() {
    // V19 and V19a are rules of the older schemas too: the density setting's validation logs them at level 3
    // and fails the validation, so the file is refused.
    let d = rejected(parse(
        &doc("1.21.90", &density(r#""max_density": 0.1"#)),
        &pack(),
    ));
    let entry = "minecraft:fog_settings/volumetric/density/air";
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value for 'max_density_height' is required unless 'uniform' equals true.",
    );
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value for 'zero_density_height' is required unless 'uniform' equals true.",
    );
    assert_eq!(
        d.iter()
            .filter(|f| f.code.name() == "InvalidValue")
            .map(|f| f.location.json_path.to_string())
            .collect::<Vec<_>>(),
        [entry, entry]
    );
    refused(&d);
    // So is V18: `uniform` with a height.
    let d = rejected(parse(
        &doc(
            "1.21.90",
            &density(
                r#""max_density": 0.1, "uniform": true, "max_density_height": 10, "zero_density_height": 5"#,
            ),
        ),
        &pack(),
    ));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Values for 'max_density_height' and 'zero_density_height' are not used when 'uniform' equals true.",
    );
    refused(&d);
}

#[test]
fn u08_negative_fog_end_both_eras() {
    let negative = air(&AIR.replace("\"fog_end\": 1.0", "\"fog_end\": -5"));
    let d = rejected(parse(&doc("1.21.90", &negative), &pack()));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "Value '-5.000000' outside valid range [0.000000, 340282346638528859811704183484516925440.000000]",
    );
    refused(&d);
    let d = rejected(parse(&doc("1.26.20", &negative), &pack()));
    assert_eq!(
        count(&d, "InvalidValue", Severity::Error),
        1,
        "{:?}",
        lines(&d)
    );
    refused(&d);

    let seconds = |mid: &str| {
        format!(
            r#""distance": {{"water": {{{AIR}, "transition_fog": {{"init_fog": {{{AIR}}}, "min_percent": 0.2, "mid_seconds": {mid}, "mid_percent": 0.5, "max_seconds": 4}}}}}}"#
        )
    };
    for version in ["1.21.90", "1.26.20"] {
        let d = rejected(parse(&doc(version, &seconds("-1")), &pack()));
        assert!(
            count(&d, "InvalidValue", Severity::Error) >= 1,
            "{version}: {:?}",
            lines(&d)
        );
        refused(&d);
        // Zero is allowed in both.
        loaded(parse(&doc(version, &seconds("0")), &pack()));
    }
}

/// Settings upgraded from an older file may break a rule only the 1.26.20 parser has. Written
/// at 1.26.20 the game and this loader would refuse them, so the writer writes them in the
/// older schema, where they read back as themselves; the document in its own shape
/// (`history::FogHistory`) writes too.
#[test]
fn settings_the_newest_parser_refuses_are_refused_in_the_older_schema_too() {
    // The rules V15 and V17–V20 are the typed nodes' own in every era: a file
    // that breaks one is refused whatever its format version, and its settings have no text.
    for (members, at, text) in [
        (
            density(r#""max_density": 0.1"#),
            "minecraft:fog_settings/volumetric/density/air",
            "Value for 'max_density_height' is required unless 'uniform' equals true.",
        ),
        (
            density(r#""max_density": 0.1, "uniform": true, "max_density_height": 10"#),
            "minecraft:fog_settings/volumetric/density/air",
            "Values for 'max_density_height' and 'zero_density_height' are not used when 'uniform' equals true.",
        ),
        (
            air(&AIR.replace("0.9", "10").replace("1.0", "5")),
            "minecraft:fog_settings/distance/air",
            "Value for 'fog_end' was less than 'fog_start', 'fog_start' has to be less than or equal to 'fog_end'.",
        ),
    ] {
        for version in ["1.16.100", "1.21.90", "1.26.20"] {
            let d = rejected(parse(&doc(version, &members), &pack()));
            expect(&d, "InvalidValue", Severity::Error, text);
            assert!(
                d.iter()
                    .any(|f| f.code.name() == "InvalidValue"
                        && f.location.json_path.to_string() == at),
                "{version} {members}: {:?}",
                lines(&d)
            );
            refused(&d);
        }
    }
    // A render-range line is level 2 in every era (the typed node's validation and the 1.26.20 loader alike): a warning, no refusal, and the file is written at 1.26.20 and reads back.
    let warned = |version: &str| {
        doc(
            version,
            &air(&AIR.replace("0.9", "1.5").replace("1.0", "2")),
        )
    };
    for version in ["1.21.90", "1.26.20"] {
        let document = loaded(parse(&warned(version), &pack()));
        let told: Vec<_> = document
            .diagnostics()
            .iter()
            .map(|f| (f.severity, f.code.to_string()))
            .collect();
        assert_eq!(
            told,
            [
                (
                    Severity::Warning,
                    "Value for 'fog_start' should not be greater than 1 when using 'render' distance type.".to_owned()
                ),
                (
                    Severity::Warning,
                    "Value for 'fog_end' should not be greater than 1 when using 'render' distance type.".to_owned()
                ),
            ],
            "{version}"
        );
        let text = document.to_json().unwrap();
        assert!(text.contains(r#""format_version": "1.26.20""#), "{text}");
        let again = loaded(parse(&text, &pack()));
        assert_eq!(again.value(), document.value());
    }
}

/// H4: each older era's read refuses a member a later era adds (an unknown member fails the
/// strict schema), on the history path too, so a document an era holds never has one and the
/// one shape all three eras are read in loses nothing between them.
#[test]
fn h4_an_era_refuses_the_members_a_later_era_adds() {
    let weather =
        r#""volumetric": {"density": {"weather": {"max_density": 0.5, "uniform": true}}}"#;
    let hg = r#""volumetric": {"henyey_greenstein_g": {"air": {"henyey_greenstein_g": -0.5}}}"#;
    for (version, members, member, path) in [
        (
            "1.20.80",
            weather,
            "weather",
            "minecraft:fog_settings/volumetric/density/weather",
        ),
        (
            "1.16.100",
            hg,
            "henyey_greenstein_g",
            "minecraft:fog_settings/volumetric/henyey_greenstein_g",
        ),
        (
            "1.21.80",
            hg,
            "henyey_greenstein_g",
            "minecraft:fog_settings/volumetric/henyey_greenstein_g",
        ),
    ] {
        let d = rejected(FogSettings::parse_as_written(
            &PackPath::from_static("fogs/x.json"),
            doc(version, members).as_bytes(),
            &pack(),
        ));
        expect_at(
            &d,
            "UnrecognizedField",
            Severity::Error,
            &format!("child '{member}' not valid here."),
            path,
        );
    }
    // The era that adds each member reads it.
    for (version, members, parsed_as) in [("1.21.0", weather, V1_21_0), ("1.21.90", hg, V1_21_90)] {
        let written = loaded(FogSettings::parse_as_written(
            &PackPath::from_static("fogs/x.json"),
            doc(version, members).as_bytes(),
            &pack(),
        ));
        assert_eq!(written.parsed_version(), parsed_as, "{version}");
    }
}
