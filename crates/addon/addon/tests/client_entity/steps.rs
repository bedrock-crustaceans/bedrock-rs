//! The two `Upgrade` impls of `families::client_entity::history`, the chain's shape, and
//! `s01`–`s02d`.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::document::JsonDocument as _;
use bedrock_addon::families::client_entity::ClientEntity;
use bedrock_addon::families::client_entity::history::{
    self, Any, ClientEntityFamily, v1_8_0, v1_10_0,
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

const FAMILY: &str = "client_entity";

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

const DESCRIPTION: &str = "minecraft:client_entity/description";

fn checks() -> Checks<'static> {
    Checks {
        load: cx(),
        identifier: "demo:x",
        actor_type: ActorType::Mob,
    }
}

/// The 1.8.0 step on `text`.
fn run_1_8(text: &str) -> StepOutput {
    step_output::<v1_8_0::ClientEntity>(&json(text), Run::default(), &checks())
        .expect("the 1.8.0 step runs")
}

/// The 1.10.0 step on `text`, as a document written at `version`.
fn run_1_10_at(text: &str, version: FormatVersion) -> StepOutput {
    let run = Run {
        original_version: Some(version),
        skip_section_a: false,
    };
    step_output::<v1_10_0::ClientEntity>(&json(text), run, &checks()).expect("the 1.10.0 step runs")
}

// ---- the 1.8 step ----

#[test]
fn the_1_8_step_moves_the_controller_list_into_animations_and_animate() {
    let out = run_1_8(&ghost(
        "1.8.0",
        r#""animations": {"walk": "animation.x.walk"}, "animation_controllers": [{"setup": "controller.animation.x.setup"}, {"move": "controller.animation.x.move"}]"#,
    ));
    let p = &out.payload;
    assert!(
        text_at(p, "format_version") == Some("1.10.0"),
        "the document is stamped 1.10.0",
    );
    assert!(
        text_at(p, &format!("{DESCRIPTION}/animations/controller__setup"))
            == Some("controller.animation.x.setup"),
        "each controller becomes an animation named controller__<name>",
    );
    assert!(
        text_at(p, &format!("{DESCRIPTION}/scripts/animate/1")) == Some("controller__move"),
        "each controller is appended to scripts.animate, in order",
    );
    assert!(
        at(p, &format!("{DESCRIPTION}/animation_controllers")).is_none(),
        "the list is removed",
    );
    assert!(out.diagnostics.is_empty(), "nothing is reported");
    // Gate: with `animations` and no controllers nothing is inserted.
    let out = run_1_8(&ghost(
        "1.8.0",
        r#""animations": {"walk": "animation.x.walk"}"#,
    ));
    assert!(
        at(&out.payload, &format!("{DESCRIPTION}/scripts")).is_none(),
        "no scripts.animate without controllers",
    );
    // An empty list moves nothing and creates nothing.
    let out = run_1_8(&ghost(
        "1.8.0",
        r#""animations": {"walk": "animation.x.walk"}, "animation_controllers": []"#,
    ));
    assert!(
        at(&out.payload, &format!("{DESCRIPTION}/scripts")).is_none()
            && at(
                &out.payload,
                &format!("{DESCRIPTION}/animation_controllers"),
            )
            .is_none(),
        "an empty list is removed and adds no animate",
    );
}

#[test]
fn a_document_without_a_version_is_read_as_1_8_0() {
    let text = r#"{"minecraft:client_entity": {"description": {"identifier": "demo:ghost"}}}"#;
    let document = loaded(entity(text, &cx()));
    assert!(
        document.version().parsed_as == FormatVersion::new(1, 8, 0),
        "it is read as 1.8.0",
    );
}

// ---- the 1.10 step ----

#[test]
fn the_1_10_step_keeps_the_string_elements_of_the_script_lists_and_drops_the_rest() {
    let out = run_1_10_at(
        &ghost(
            "1.20.0",
            r#""scripts": {"initialize": ["variable.A = 1;", 2.5], "pre_animation": ["variable.B = 2;", true, "variable.C = 3;"]}"#,
        ),
        FormatVersion::new(1, 20, 0),
    );
    let p = &out.payload;
    assert!(
        text_at(p, "format_version") == Some("1.26.0"),
        "the document is stamped 1.26.0",
    );
    let strings = |name: &str| -> Option<Vec<String>> {
        at(p, &format!("{DESCRIPTION}/scripts/{name}"))?
            .as_array()?
            .iter()
            .map(|v| v.as_string().map(|s| s.as_str().to_owned()))
            .collect()
    };
    assert!(
        strings("initialize") == Some(vec!["variable.A = 1;".to_owned()]),
        "a string element is kept verbatim and a number is dropped",
    );
    assert!(
        strings("pre_animation")
            == Some(vec![
                "variable.B = 2;".to_owned(),
                "variable.C = 3;".to_owned(),
            ]),
        "a boolean element is dropped",
    );
    assert!(
        out.diagnostics
            .iter()
            .filter(|f| f.code.name() == "DroppedField" && f.severity == Severity::Info)
            .count()
            == 2,
        "each dropped element is reported",
    );
}

const SCRIPTS: &str = r#""animations": {"Walk": "animation.x.walk"}, "render_controllers": [{"controller.render.X": "Query.Is_Baby"}], "scripts": {"variables": {"variable.Foo": "public"}, "initialize": ["Variable.A = 'Foo';"], "pre_animation": ["Variable.Foo = Query.Is_Baby;"], "scale": "Query.Scale", "scaleX": "Q.X", "parent_setup": "Variable.P = 1;", "should_update_effects_offscreen": "Query.Is_Baby", "animate": [{"Walk": "Query.Is_Baby"}]}"#;

#[test]
fn the_1_10_step_lower_cases_the_named_scripts_below_1_18_20() {
    let script = |out: &StepOutput, name: &str| {
        at(&out.payload, &format!("{DESCRIPTION}/scripts/{name}"))
            .and_then(|v| v.as_string().map(|s| s.as_str().to_owned()))
    };
    // Just below 1.18.20 every named script is folded, quoted literals included.
    let below = run_1_10_at(&ghost("1.18.10", SCRIPTS), FormatVersion::new(1, 18, 10));
    assert!(
        script(&below, "pre_animation/0").as_deref() == Some("variable.foo = query.is_baby;"),
        "pre_animation is lower-cased",
    );
    assert!(
        script(&below, "initialize/0").as_deref() == Some("variable.a = 'foo';"),
        "initialize is lower-cased, its literal included",
    );
    assert!(
        script(&below, "scale").as_deref() == Some("query.scale")
            && script(&below, "scaleX").as_deref() == Some("q.x")
            && script(&below, "parent_setup").as_deref() == Some("variable.p = 1;")
            && script(&below, "should_update_effects_offscreen").as_deref()
                == Some("query.is_baby"),
        "every other named script is lower-cased",
    );
    assert!(
        script(&below, "animate/0/Walk").as_deref() == Some("Query.Is_Baby")
            && text_at(
                &below.payload,
                &format!("{DESCRIPTION}/render_controllers/0/controller.render.X"),
            ) == Some("Query.Is_Baby")
            && at(
                &below.payload,
                &format!("{DESCRIPTION}/scripts/variables/variable.Foo"),
            )
            .is_some(),
        "conditions and variable names stay as written",
    );
    // At 1.18.20 nothing is folded.
    let at_gate = run_1_10_at(&ghost("1.18.20", SCRIPTS), FormatVersion::new(1, 18, 20));
    assert!(
        script(&at_gate, "pre_animation/0").as_deref() == Some("Variable.Foo = Query.Is_Baby;")
            && script(&at_gate, "initialize/0").as_deref() == Some("Variable.A = 'Foo';")
            && script(&at_gate, "scale").as_deref() == Some("Query.Scale"),
        "from 1.18.20 the scripts stay as written",
    );
}

// ---- the chain ----

#[test]
fn the_chain_is_contiguous_and_ends_at_1_26_0() {
    use bedrock_addon::versioning::Family;

    const HAS_BETA: bool = <Any as VersionChain<ClientEntityFamily>>::HAS_BETA;

    check_chain::<ClientEntityFamily>().expect("the chain is well formed");
    assert_eq!(
        <Any as VersionChain<ClientEntityFamily>>::VERSIONS,
        [
            FormatVersion::new(1, 8, 0),
            FormatVersion::new(1, 10, 0),
            FormatVersion::new(1, 26, 0)
        ]
    );
    let has_beta = HAS_BETA;
    assert!(!has_beta, "Beta = Infallible");
    assert_eq!(ClientEntityFamily::RANGE_MODEL, RangeModel::ClosedToEngine);
    assert_eq!(ClientEntityFamily::FLOOR, FormatVersion::new(1, 8, 0));
}

#[test]
fn each_range_selects_its_struct() {
    let pick = |a, b, c| {
        select::<ClientEntityFamily>(&FormatVersionInput {
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

// ---- steps ----

/// Parses `text` as written and runs the one step from its struct: the next struct written,
/// and what the step reported.
fn one_step(text: &str, cx: &LoadContext) -> (Value, Diagnostics) {
    let written = loaded(ClientEntity::parse_as_written(
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
        diagnostics: &mut diagnostics,
    };
    let value = match written.into_value() {
        Any::V1_8_0(entity) => entity.upgrade(&mut upgrade).write(),
        Any::V1_10_0(entity) => entity.upgrade(&mut upgrade).write(),
        Any::V1_26_0(entity) => entity.write(),
    };
    (value.unwrap(), diagnostics)
}

#[test]
fn s01_1_8_controllers_to_animate() {
    let before = fixture(FAMILY, "s01_1_8_controllers_to_animate.before.json");
    let after = fixture(FAMILY, "s01_1_8_controllers_to_animate.after.json");
    let (stepped, diagnostics) = one_step(&before, &cx());
    assert_eq!(stepped, json(&after));
    silent(&diagnostics);
    // The whole load goes on to 1.26.0 and keeps the moved controllers.
    let document = loaded(entity(&before, &cx()));
    assert!(document.version().upgraded);
    assert_eq!(document.format_version, FormatVersion::new(1, 26, 0));
    assert_eq!(
        document.description.animations.as_ref().unwrap()["controller__move"].as_str(),
        "controller.animation.pig.move"
    );
}

#[test]
fn s01b_existing_animate_is_no_member_of_1_8() {
    // s01b has a 1.8.0 `scripts.animate`; says the v1_8 schema does not
    // declare it and fails the file. The rule wins.
    let members = r#""animations": {"walk": "animation.x.walk"}, "animation_controllers": [{"move": "controller.animation.x.move"}], "scripts": {"animate": ["walk"]}"#;
    let d = rejected(entity(&ghost("1.8.0", members), &cx()));
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'animate' not valid here.",
    );
}

#[test]
fn s01c_name_collision() {
    let members = r#""animations": {"controller__move": "animation.x.old", "walk": "animation.x.walk"}, "animation_controllers": [{"move": "controller.animation.x.move"}]"#;
    let (stepped, _) = one_step(&ghost("1.8.0", members), &cx());
    assert_eq!(
        text_at(
            &stepped,
            &format!("{DESCRIPTION}/animations/controller__move")
        ),
        Some("controller.animation.x.move")
    );
    // Overwritten in place: still two animations.
    let animations = at(&stepped, &format!("{DESCRIPTION}/animations")).unwrap();
    assert_eq!(animations.as_object().unwrap().len(), 2);
}

#[test]
fn s01d_no_animations_deviation() {
    let members = r#""animation_controllers": [{"move": "controller.animation.x.move"}]"#;
    let d = rejected(entity(&ghost("1.8.0", members), &cx()));
    expect(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "child 'animation_controllers' not valid here.",
    );
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "minecraft:client_entity/description/animation_controllers",
    );
}

#[test]
fn s01e_multi_member_element_deviation() {
    // Written `b` first: the member read is the smallest key by bytes, not the first written.
    let members = r#""animations": {"walk": "animation.x.walk"}, "animation_controllers": [{"b": "controller.animation.x.b", "a": "controller.animation.x.a"}]"#;
    let document = loaded(entity(&ghost("1.8.0", members), &cx()));
    let animations = document.description.animations.as_ref().unwrap();
    assert_eq!(
        animations["controller__a"].as_str(),
        "controller.animation.x.a"
    );
    assert!(!animations.contains_key("controller__b"));
    expect_at(
        document.diagnostics(),
        "DroppedField",
        Severity::Info,
        "minecraft:client_entity/description/animation_controllers/0/b",
    );
    assert_eq!(
        document
            .diagnostics()
            .iter()
            .filter(|f| f.code.name() == "DroppedField")
            .count(),
        1
    );
}

#[test]
fn s01g_bad_1_8_schema() {
    let members = r#""render_controllers": [{"controller.render.x": "q.is_baby"}]"#;
    let d = rejected(entity(&ghost("1.8.0", members), &cx()));
    expect_prefix(
        &d,
        "InvalidType",
        Severity::Error,
        "unknown child schema option type.  Allowed types: ",
    );
    expect_at(
        &d,
        "InvalidType",
        Severity::Error,
        "minecraft:client_entity/description/render_controllers/0",
    );
    // The same list is the object form of 1.10.0.
    loaded(entity(&ghost("1.10.0", members), &cx()));
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
    let document = loaded(entity(&ghost("1.16.0", UPPER), &cx()));
    let scripts = document.description.scripts.as_ref().unwrap();
    assert_eq!(
        scripts.pre_animation.as_ref().unwrap()[0].0,
        "variable.foo = query.is_baby;"
    );
    let condition = scripts.animate.as_ref().unwrap()[0].condition().unwrap();
    assert_eq!(condition.source().unwrap().as_str(), "Query.Is_Baby");
    clean(document.diagnostics());
}

#[test]
fn s02c_no_lowercase_at_1_18_20() {
    let document = loaded(entity(&ghost("1.18.20", UPPER), &cx()));
    let scripts = document.description.scripts.as_ref().unwrap();
    assert_eq!(
        scripts.pre_animation.as_ref().unwrap()[0].0,
        "Variable.Foo = Query.Is_Baby;"
    );
}

#[test]
fn s02d_1_8_path_lowercases() {
    let members = r#""scripts": {"pre_animation": ["Variable.X = 1;"], "scale": "Variable.X"}"#;
    let document = loaded(entity(&ghost("1.8.0", members), &cx()));
    let scripts = document.description.scripts.as_ref().unwrap();
    assert_eq!(
        scripts.pre_animation.as_ref().unwrap()[0].0,
        "variable.x = 1;"
    );
    assert_eq!(
        scripts.scale.as_ref().unwrap().source().unwrap().as_str(),
        "variable.x"
    );
    // So does a document without a version (vanilla's 1.0.0 is below 1.18.20 too).
    let text = r#"{"minecraft:client_entity": {"description": {"identifier": "demo:ghost", "scripts": {"pre_animation": ["Variable.X = 1;"]}}}}"#;
    let document = loaded(entity(text, &cx()));
    let scripts = document.description.scripts.as_ref().unwrap();
    assert_eq!(
        scripts.pre_animation.as_ref().unwrap()[0].0,
        "variable.x = 1;"
    );
}

// ---- as written, and upgraded from there ----

#[test]
fn a_document_parses_as_written_and_upgrades_to_what_parse_gives() {
    let before = fixture(FAMILY, "s01_1_8_controllers_to_animate.before.json");
    let path = PackPath::from_static(PATH);
    let written = loaded(ClientEntity::parse_as_written(
        &path,
        before.as_bytes(),
        &cx(),
    ));
    assert!(matches!(written.value(), Any::V1_8_0(_)));
    assert_eq!(written.parsed_version(), FormatVersion::new(1, 8, 0));
    // Written back in its own version, it is the document it was read from.
    assert_eq!(json(&written.to_json().unwrap()), json(&before));
    let upgraded = loaded(history::upgrade(written, &cx()));
    let direct = loaded(entity(&before, &cx()));
    assert_eq!(upgraded.value(), direct.value());
    assert_eq!(upgraded.version(), direct.version());
}

#[test]
fn every_version_round_trips_as_written() {
    for name in [
        "s01_1_8_controllers_to_animate.before.json",
        "s02_1_10_to_1_26_types.before.json",
        "s02_1_10_to_1_26_types.after.json",
    ] {
        let text = fixture(FAMILY, name);
        let path = PackPath::from_static(PATH);
        let written = loaded(ClientEntity::parse_as_written(
            &path,
            text.as_bytes(),
            &cx(),
        ));
        let out = written.to_json().unwrap();
        assert_eq!(json(&out), json(&text), "{name}");
        let again = loaded(ClientEntity::parse_as_written(&path, out.as_bytes(), &cx()));
        assert_eq!(again.value(), written.value(), "{name}");
    }
}

// ---- the payload of the 1.8 step's refusal ----

/// Without `animations` the 1.8 step leaves the controller list where it is, refuses it and
/// converts nothing: no `animations`, no `scripts`.
#[test]
fn the_1_8_step_converts_nothing_of_a_controller_list_without_animations() {
    let out = run_1_8(&ghost(
        "1.8.0",
        r#""animation_controllers": [{"move": "controller.animation.x.move"}]"#,
    ));
    assert!(
        out.diagnostics.iter().any(|f| {
            f.code.name() == "UnrecognizedField"
                && f.severity == Severity::Error
                && f.code.to_string() == "child 'animation_controllers' not valid here."
                && f.location.json_path.to_string()
                    == "minecraft:client_entity/description/animation_controllers"
        }),
        "the list that stays is an undeclared member of 1.10.0",
    );
    assert!(
        at(&out.payload, &format!("{DESCRIPTION}/animations")).is_none()
            && at(&out.payload, &format!("{DESCRIPTION}/scripts")).is_none(),
        "nothing moved",
    );
}

/// an animation `controller__<name>` the file already has
/// is written over by the controller of that name (the game's step inserts under the name
/// whatever is there); the file's is noted where it is. The same value says nothing.
#[test]
fn an_animation_of_the_file_under_a_controller_s_name_is_noted_as_replaced() {
    let text = |own: &str| {
        ghost(
            "1.8.0",
            &format!(
                r#""animations": {{"walk": "animation.x.walk", "controller__move": "{own}"}}, "animation_controllers": [{{"move": "controller.animation.pig.move"}}]"#
            ),
        )
    };
    let document = loaded(entity(&text("animation.x.mine"), &cx()));
    expect_at(
        document.diagnostics(),
        "DroppedField",
        Severity::Info,
        "minecraft:client_entity/description/animations/controller__move",
    );
    assert_eq!(
        document.description.animations.as_ref().unwrap()["controller__move"].as_str(),
        "controller.animation.pig.move"
    );
    let document = loaded(entity(&text("controller.animation.pig.move"), &cx()));
    none(document.diagnostics(), "DroppedField");
}

/// two elements of `animation_controllers` with one
/// first key make one `controller__<key>`, the later one writing over the earlier. The
/// controller that goes is the earlier ELEMENT's, told where the file has it, not at
/// `animations/controller__<key>`, a member the file does not have. The same controller in
/// both says nothing.
#[test]
fn a_controller_a_later_element_of_the_same_key_replaces_is_noted_at_its_element() {
    let text = |first: &str| {
        ghost(
            "1.8.0",
            &format!(
                r#""animations": {{"walk": "animation.x.walk"}}, "animation_controllers": [{{"move": "{first}"}}, {{"move": "controller.animation.x.b"}}]"#
            ),
        )
    };
    let document = loaded(entity(&text("controller.animation.x.a"), &cx()));
    expect_at(
        document.diagnostics(),
        "DroppedField",
        Severity::Info,
        "minecraft:client_entity/description/animation_controllers/0/move",
    );
    assert!(
        document.diagnostics().iter().all(|f| !f
            .location
            .json_path
            .to_string()
            .ends_with("controller__move")),
        "{:#?}",
        document.diagnostics()
    );
    assert_eq!(
        document.description.animations.as_ref().unwrap()["controller__move"].as_str(),
        "controller.animation.x.b"
    );
    let document = loaded(entity(&text("controller.animation.x.b"), &cx()));
    none(document.diagnostics(), "DroppedField");
}

/// below 1.18.20 the loader lower-cases a script's
/// whole text, quoted literals included, and the document is written so. A literal whose case
/// that changes is another value than the author's: noted at the script. Names alone (Molang
/// reads them in any case) and a literal already in lower case say nothing.
#[test]
fn a_script_literal_the_lower_casing_changes_is_noted_at_the_script() {
    let members = r#""scripts": {"pre_animation": ["Variable.Foo = 1;", "Variable.Title = 'Big Boss';", "Variable.Tag = 'small';"], "scale": "Query.Get_Name == 'Steve' ? 2 : 1", "parent_setup": "Variable.Hidden = 0.0;"}"#;
    let told = |version: &str| {
        let document = loaded(entity(&ghost(version, members), &cx()));
        let scripts = document.description.scripts.as_ref().unwrap();
        let second = scripts.pre_animation.as_ref().unwrap()[1].0.clone();
        let told: Vec<String> = document
            .diagnostics()
            .iter()
            .filter(|f| f.code.name() == "ValueRewritten")
            .map(|f| format!("{:?} {}", f.severity, f.location.json_path))
            .collect();
        (second, told)
    };
    for version in ["1.10.0", "1.8.0"] {
        let (second, told) = told(version);
        assert_eq!(second, "variable.title = 'big boss';", "{version}");
        assert_eq!(
            told,
            [
                "Info minecraft:client_entity/description/scripts/pre_animation/1",
                "Info minecraft:client_entity/description/scripts/scale",
            ],
            "the literal the lower-casing changes is not told: {version}"
        );
    }
    // From 1.18.20 nothing is lower-cased and nothing is said.
    let (second, told) = told("1.18.20");
    assert_eq!(second, "Variable.Title = 'Big Boss';");
    assert_eq!(told, [""; 0]);
}

/// two elements of a 1.8 `animation_controllers`
/// with one key become one animation `controller__<key>`, the later's. The earlier one is
/// noted where the FILE has it, the element of the list, and not at the name the step makes,
/// which the file does not have. The same controller twice says nothing.
#[test]
fn a_controller_a_later_one_of_its_key_writes_over_is_noted_at_its_element() {
    let dropped = |second: &str| -> Vec<String> {
        let document = loaded(entity(
            &ghost(
                "1.8.0",
                &format!(
                    r#""animations": {{"walk": "animation.x.walk"}}, "animation_controllers": [{{"move": "controller.animation.x.a"}}, {{"move": "{second}"}}]"#
                ),
            ),
            &cx(),
        ));
        assert_eq!(
            document.description.animations.as_ref().unwrap()["controller__move"].as_str(),
            second
        );
        document
            .diagnostics()
            .iter()
            .filter(|f| f.code.name() == "DroppedField")
            .map(|f| format!("{:?} {}", f.severity, f.location.json_path))
            .collect()
    };
    assert_eq!(
        dropped("controller.animation.x.b"),
        ["Info minecraft:client_entity/description/animation_controllers/0/move"],
        "the controller that is written over is not told where the file has it"
    );
    assert_eq!(dropped("controller.animation.x.a"), [""; 0]);
}
