//! What the upgrade steps of the attachable chain do: the 1.8 step's refusals, the 1.10 step's
//! conversion and lower-casing, the chain's shape, and documents read as written and upgraded.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::document::JsonDocument as _;
use bedrock_addon::families::attachable::Attachable;
use bedrock_addon::families::attachable::history::{
    self, AttachableFamily, AttachableHistory, v1_8_0, v1_10_0,
};
use bedrock_addon::json::Value;
use bedrock_addon::load::LoadContext;
use bedrock_addon::storage::PackPath;
use bedrock_addon::vanilla::ActorType;
use bedrock_addon::version::{ENGINE_VERSION, FormatVersion, FormatVersionInput};
use bedrock_addon::versioning::testing::{Checks, Run, StepOutput, step_output};
use bedrock_addon::versioning::{
    RangeModel, Selection, Upgrade, UpgradeCx, VersionChain, VersionStruct, check_chain, select,
};

use super::common::*;

const FAMILY: &str = "attachable";

fn json(text: &str) -> Value {
    facet_json::from_str(text).expect("test JSON")
}

/// The value at a `/`-separated path.
fn at<'v>(value: &'v Value, path: &str) -> Option<&'v Value> {
    let mut value = value;
    for step in path.split('/') {
        value = match step.parse::<usize>() {
            Ok(index) if value.is_array() => value.as_array()?.get(index)?,
            _ => value.as_object()?.get(step)?,
        };
    }
    Some(value)
}

fn text_at<'v>(value: &'v Value, path: &str) -> Option<&'v str> {
    at(value, path)?
        .as_string()
        .map(facet_value::VString::as_str)
}

const DESCRIPTION: &str = "minecraft:attachable/description";

fn checks() -> Checks<'static> {
    Checks {
        load: cx(),
        identifier: "demo:x",
        actor_type: ActorType::Mob,
    }
}

/// Runs the 1.10 step on `text` as a document of `version`.
fn step_1_10(text: &str, version: FormatVersion) -> StepOutput {
    step_output::<v1_10_0::Attachable>(
        &json(text),
        Run {
            original_version: Some(version),
            skip_section_a: false,
        },
        &checks(),
    )
    .expect("the step runs")
}

/// Runs the 1.8 step on `text` as a 1.8.0 document.
fn step_1_8(text: &str) -> StepOutput {
    step_output::<v1_8_0::Attachable>(&json(text), Run::default(), &checks())
        .expect("the step runs")
}

#[test]
fn the_1_10_step_keeps_the_string_elements_of_a_script_list_and_drops_the_rest() {
    let out = step_1_10(
        &ghost(
            "1.20.0",
            r#""scripts": {"initialize": ["variable.A = 1;", 2.5], "pre_animation": ["variable.B = 2;", true, "variable.C = 3;"]}"#,
        ),
        FormatVersion::new(1, 20, 0),
    );
    let p = &out.payload;
    assert_eq!(text_at(p, "format_version"), Some("1.26.0"));
    let strings = |name: &str| -> Option<Vec<String>> {
        at(p, &format!("{DESCRIPTION}/scripts/{name}"))?
            .as_array()?
            .iter()
            .map(|v| v.as_string().map(|s| s.as_str().to_owned()))
            .collect()
    };
    assert_eq!(
        strings("initialize"),
        Some(vec!["variable.A = 1;".to_owned()]),
        "a string element is kept verbatim and a number is dropped"
    );
    assert_eq!(
        strings("pre_animation"),
        Some(vec![
            "variable.B = 2;".to_owned(),
            "variable.C = 3;".to_owned(),
        ]),
        "a boolean element is dropped"
    );
    assert_eq!(
        out.diagnostics
            .iter()
            .filter(|f| f.code.name() == "DroppedField" && f.severity == Severity::Info)
            .count(),
        2,
        "each dropped element is reported"
    );
}

const SCRIPTS: &str = r#""animations": {"Walk": "animation.x.walk"}, "render_controllers": [{"controller.render.X": "Query.Is_Baby"}], "scripts": {"variables": {"variable.Foo": "public"}, "initialize": ["Variable.A = 'Foo';"], "pre_animation": ["Variable.Foo = Query.Is_Baby;"], "scale": "Query.Scale", "scaleX": "Q.X", "parent_setup": "Variable.P = 1;", "should_update_effects_offscreen": "Query.Is_Baby", "animate": [{"Walk": "Query.Is_Baby"}]}"#;

#[test]
fn the_1_10_step_lowercases_every_named_script_below_1_18_20_and_nothing_else() {
    let script = |out: &StepOutput, name: &str| {
        at(&out.payload, &format!("{DESCRIPTION}/scripts/{name}"))
            .and_then(|v| v.as_string().map(|s| s.as_str().to_owned()))
    };
    // Just below 1.18.20 every named script is folded, quoted literals included.
    let below = step_1_10(&ghost("1.18.10", SCRIPTS), FormatVersion::new(1, 18, 10));
    assert_eq!(
        script(&below, "pre_animation/0").as_deref(),
        Some("variable.foo = query.is_baby;")
    );
    assert_eq!(
        script(&below, "initialize/0").as_deref(),
        Some("variable.a = 'foo';"),
        "a literal is folded too"
    );
    assert_eq!(script(&below, "scale").as_deref(), Some("query.scale"));
    assert_eq!(script(&below, "scaleX").as_deref(), Some("q.x"));
    assert_eq!(
        script(&below, "parent_setup").as_deref(),
        Some("variable.p = 1;")
    );
    assert_eq!(
        script(&below, "should_update_effects_offscreen").as_deref(),
        Some("query.is_baby")
    );
    // Conditions and variable names stay as written.
    assert_eq!(
        script(&below, "animate/0/Walk").as_deref(),
        Some("Query.Is_Baby")
    );
    assert_eq!(
        text_at(
            &below.payload,
            &format!("{DESCRIPTION}/render_controllers/0/controller.render.X"),
        ),
        Some("Query.Is_Baby")
    );
    assert!(
        at(
            &below.payload,
            &format!("{DESCRIPTION}/scripts/variables/variable.Foo"),
        )
        .is_some()
    );
    // At 1.18.20 nothing is folded.
    let at_gate = step_1_10(&ghost("1.18.20", SCRIPTS), FormatVersion::new(1, 18, 20));
    assert_eq!(
        script(&at_gate, "pre_animation/0").as_deref(),
        Some("Variable.Foo = Query.Is_Baby;")
    );
    assert_eq!(
        script(&at_gate, "initialize/0").as_deref(),
        Some("Variable.A = 'Foo';")
    );
    assert_eq!(script(&at_gate, "scale").as_deref(), Some("Query.Scale"));
}

#[test]
fn the_1_10_step_carries_item_over_as_written() {
    for (item, path, expected) in [
        (r#""item": "Demo:Wand""#, "item", "Demo:Wand"),
        (
            r#""item": {"Demo:Wand": "Query.Is_Sneaking"}"#,
            "item/Demo:Wand",
            "Query.Is_Sneaking",
        ),
    ] {
        // Below 1.18.20, where the named scripts are lower-cased: `item` is not.
        let out = step_1_10(&ghost("1.10.0", item), FormatVersion::new(1, 10, 0));
        assert_eq!(
            text_at(&out.payload, &format!("{DESCRIPTION}/{path}")),
            Some(expected)
        );
        assert!(out.diagnostics.is_empty(), "nothing is reported");
    }
}

#[test]
fn the_1_8_step_rejects_a_document_with_item_in_either_form() {
    for item in [r#""item": "demo:wand""#, r#""item": {"demo:wand": "1.0"}"#] {
        for version in ["1.8.0", "1.9.9"] {
            let d = rejected(attachable(&ghost(version, item), &cx()));
            expect(
                &d,
                "UnrecognizedField",
                Severity::Error,
                "child 'item' not valid here.",
            );
            expect_at(
                &d,
                "UnrecognizedField",
                Severity::Error,
                "minecraft:attachable/description/item",
            );
        }
    }
}

#[test]
fn the_1_8_step_stamps_the_version_and_reports_nothing_without_a_controller_list() {
    let out = step_1_8(&ghost(
        "1.8.0",
        r#""animations": {"hold": "animation.x.hold"}"#,
    ));
    assert_eq!(text_at(&out.payload, "format_version"), Some("1.10.0"));
    assert_eq!(
        text_at(&out.payload, &format!("{DESCRIPTION}/animations/hold")),
        Some("animation.x.hold")
    );
    assert!(out.diagnostics.is_empty(), "nothing is reported");
}

// ---- the chain ----

#[test]
fn the_chain_is_contiguous_and_ends_at_1_26_0() {
    use bedrock_addon::versioning::Family;

    const HAS_BETA: bool = <AttachableHistory as VersionChain<AttachableFamily>>::HAS_BETA;

    check_chain::<AttachableFamily>().expect("the chain is well formed");
    assert_eq!(
        <AttachableHistory as VersionChain<AttachableFamily>>::VERSIONS,
        [
            FormatVersion::new(1, 8, 0),
            FormatVersion::new(1, 10, 0),
            FormatVersion::new(1, 26, 0)
        ]
    );
    let has_beta = HAS_BETA;
    assert!(!has_beta, "Beta = Infallible");
    assert_eq!(AttachableFamily::RANGE_MODEL, RangeModel::ClosedToEngine);
    assert_eq!(AttachableFamily::FLOOR, FormatVersion::new(1, 8, 0));
}

#[test]
fn each_range_selects_its_struct() {
    let pick = |a, b, c| {
        select::<AttachableFamily>(&FormatVersionInput {
            version: Some(FormatVersion::new(a, b, c)),
            beta_literal: false,
            prerelease: false,
        })
    };
    assert_eq!(pick(1, 7, 9), Selection::BelowFloor);
    assert_eq!(pick(1, 8, 0), Selection::Struct(0));
    assert_eq!(pick(1, 9, 9), Selection::Struct(0));
    assert_eq!(pick(1, 10, 0), Selection::Struct(1));
    assert_eq!(pick(1, 25, 9), Selection::Struct(1));
    assert_eq!(pick(1, 26, 0), Selection::Struct(2));
    let e = ENGINE_VERSION;
    assert_eq!(pick(e.major, e.minor, e.patch), Selection::Struct(2));
    assert_eq!(pick(e.major, e.minor, e.patch + 1), Selection::Unsupported);
}

// ---- single steps on the fixtures ----

/// Parses `text` as written and runs the one step from its struct: the next struct written,
/// and what the step reported.
fn one_step(text: &str, cx: &LoadContext) -> (Value, Diagnostics) {
    let written = loaded(Attachable::parse_as_written(
        &PackPath::from_static(PATH),
        text.as_bytes(),
        cx,
    ));
    let original_version = written.version().effective;
    let mut diagnostics = Diagnostics::default();
    let mut upgrade = UpgradeCx {
        load: cx,
        original_version,
        identifier: "",
        actor_type: ActorType::Mob,
        skip_section_a: false,
        reading: cx.reading,
        diagnostics: &mut diagnostics,
    };
    let value = match written.into_value() {
        AttachableHistory::V1_8_0(entity) => entity.upgrade(&mut upgrade).write(),
        AttachableHistory::V1_10_0(entity) => entity.upgrade(&mut upgrade).write(),
        AttachableHistory::V1_26_0(entity) => entity.write(),
    };
    (value.unwrap(), diagnostics)
}

#[test]
fn s02_1_10_to_1_26_types() {
    let before = fixture(FAMILY, "s02_1_10_to_1_26_types.before.json");
    let after = fixture(FAMILY, "s02_1_10_to_1_26_types.after.json");
    let (stepped, diagnostics) = one_step(&before, &cx());
    assert_eq!(stepped, json(&after));
    silent(&diagnostics);
}

const UPPER: &str = r#""animations": {"walk": "animation.x.walk"}, "scripts": {"pre_animation": ["Variable.Foo = Query.Is_Baby;"], "animate": [{"walk": "Query.Is_Baby"}]}"#;

#[test]
fn s02b_lowercase_before_1_18_20() {
    let document = loaded(attachable(&ghost("1.16.0", UPPER), &cx()));
    let scripts = document.description.slots.scripts.as_ref().unwrap();
    assert_eq!(
        scripts.pre_animation.as_ref().unwrap()[0],
        "variable.foo = query.is_baby;"
    );
    let condition = scripts.animate.as_ref().unwrap()[0].condition().unwrap();
    assert_eq!(condition.source().unwrap().as_str(), "Query.Is_Baby");
    clean(document.diagnostics());
}

#[test]
fn s02c_no_lowercase_at_1_18_20() {
    let document = loaded(attachable(&ghost("1.18.20", UPPER), &cx()));
    let scripts = document.description.slots.scripts.as_ref().unwrap();
    assert_eq!(
        scripts.pre_animation.as_ref().unwrap()[0],
        "Variable.Foo = Query.Is_Baby;"
    );
}

#[test]
fn s02d_1_8_path_lowercases() {
    let members = r#""scripts": {"pre_animation": ["Variable.X = 1;"], "scale": "Variable.X"}"#;
    let document = loaded(attachable(&ghost("1.8.0", members), &cx()));
    let scripts = document.description.slots.scripts.as_ref().unwrap();
    assert_eq!(
        scripts.pre_animation.as_ref().unwrap()[0],
        "variable.x = 1;"
    );
    assert_eq!(
        scripts.scale.as_ref().unwrap().source().unwrap().as_str(),
        "variable.x"
    );
    // So does a document without a version (vanilla's 1.0.0 is below 1.18.20 too).
    let text = r#"{"minecraft:attachable": {"description": {"identifier": "demo:ghost", "scripts": {"pre_animation": ["Variable.X = 1;"]}}}}"#;
    let document = loaded(attachable(text, &cx()));
    let scripts = document.description.slots.scripts.as_ref().unwrap();
    assert_eq!(
        scripts.pre_animation.as_ref().unwrap()[0],
        "variable.x = 1;"
    );
}

// ---- as written, and upgraded from there ----

#[test]
fn a_document_parses_as_written_and_upgrades_to_what_parse_gives() {
    // Copied from bedrock-samples v1.26.50.4 `resource_pack/attachables/diamond_helmet.json`.
    let before = fixture(FAMILY, "vanilla/diamond_helmet.json");
    let path = PackPath::from_static(PATH);
    let written = loaded(Attachable::parse_as_written(
        &path,
        before.as_bytes(),
        &cx(),
    ));
    assert!(matches!(written.value(), AttachableHistory::V1_8_0(_)));
    assert_eq!(written.parsed_version(), FormatVersion::new(1, 8, 0));
    // Written back in its own version, it is the document it was read from.
    assert_eq!(json(&written.to_json().unwrap()), json(&before));
    let upgraded = loaded(history::upgrade(written, &cx()));
    let direct = loaded(attachable(&before, &cx()));
    assert_eq!(upgraded.value(), direct.value());
    assert_eq!(upgraded.version(), direct.version());
}

#[test]
fn every_version_round_trips_as_written() {
    for name in [
        "vanilla/diamond_helmet.json",
        "vanilla/diamond_helmet.player.json",
        "vanilla/trident.entity.json",
        "s02_1_10_to_1_26_types.before.json",
        "s02_1_10_to_1_26_types.after.json",
    ] {
        let text = fixture(FAMILY, name);
        let path = PackPath::from_static(PATH);
        let written = loaded(Attachable::parse_as_written(&path, text.as_bytes(), &cx()));
        let out = written.to_json().unwrap();
        // The version is written in full: vanilla's trident says "1.10".
        let text = text.replace(
            r#""format_version": "1.10","#,
            r#""format_version": "1.10.0","#,
        );
        assert_eq!(json(&out), json(&text), "{name}");
        let again = loaded(Attachable::parse_as_written(&path, out.as_bytes(), &cx()));
        assert_eq!(again.value(), written.value(), "{name}");
    }
}
