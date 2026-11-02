//! The atomic client entity rows: the version gate `v01`–`v09`, the schema
//! `s01`–`s14`, the stack `k01`–`k04` and `r08`. The render-controller lint `r01`–`r07` is the
//! link pass's (`link.rs`).

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::document::{Content, Placed};
use bedrock_addon::families::atomic_client_entity::{
    AabbDimensions, AtomicClientEntity, CompanionBlockAvoidance, CompanionBlockClipping,
    Components, VisibilityBoundary,
};
use bedrock_addon::id::AtomicClientEntityId;
use bedrock_addon::json::{StrictFloat, StrictInt};
use bedrock_addon::load::LoadContext;
use bedrock_addon::version::FormatVersion;
use bedrock_addon::versioning::BETA_VERSION;
use bedrock_addon::visual::{ControllerUse, HasVisuals};

use super::common::*;

const FAMILY: &str = "atomic_client_entity";
const NO_PARSER: &str = "unsupported version: no parser available for version ";
const DESCRIPTION: &str = "minecraft:atomic_client_entity/description";

/// A beta document whose payload is `payload` (a JSON object).
fn beta(payload: &str) -> String {
    format!(r#"{{"format_version": "beta", "minecraft:atomic_client_entity": {payload}}}"#)
}

/// A beta `demo:wisp` with `members` after the identifier and `rest` after the description.
fn wisp(members: &str, rest: &str) -> String {
    let comma = if members.is_empty() { "" } else { ", " };
    let rest_comma = if rest.is_empty() { "" } else { ", " };
    beta(&format!(
        r#"{{"description": {{"identifier": "demo:wisp"{comma}{members}}}{rest_comma}{rest}}}"#
    ))
}

fn id(text: &str) -> AtomicClientEntityId {
    AtomicClientEntityId::parse(text)
}

// ---- version gate ----

#[test]
fn v01_beta_minimal() {
    let document = loaded(atomic(&wisp("", ""), &cx()));
    silent(document.diagnostics());
    assert_eq!(document.value(), &AtomicClientEntity::new(id("demo:wisp")));
    let d = &document.description;
    assert!(d.materials.is_none() && d.textures.is_none() && d.geometry.is_none());
    assert!(d.animations.is_none() && d.render_controllers.is_none());
    assert_eq!((&d.particle_effects, &d.sound_effects), (&None, &None));
    assert_eq!(d.scripts, None);
    assert_eq!(document.components, None);
    assert_eq!(document.aabb_dimensions(), None);
    // The only struct is the beta one.
    assert!(document.version().beta);
    assert_eq!(document.version().parsed_as, BETA_VERSION);
    assert!(!document.version().upgraded);
}

#[test]
fn v02_beta_normal_pack_no_experiments() {
    // Any pack, the Beta APIs toggle off: vanilla hard-codes the permission.
    let cx: LoadContext = cx();
    assert!(!cx.beta_apis && !cx.is_base_game_pack);
    let document = loaded(atomic(&wisp("", ""), &cx));
    none(document.diagnostics(), "BetaFeaturesNotEnabled");
}

#[test]
fn v03_numeric_version() {
    for version in ["1.26.45", "1.26.50", "1.0.0"] {
        let text = wisp("", "").replace("\"beta\"", &format!("\"{version}\""));
        let d = rejected(atomic(&text, &cx()));
        expect(
            &d,
            "UnsupportedVersion",
            Severity::Error,
            &format!("{NO_PARSER}{version}"),
        );
    }
}

#[test]
fn v04_prerelease_version() {
    let text = wisp("", "").replace("\"beta\"", "\"1.26.40-beta.1\"");
    let d = rejected(atomic(&text, &cx()));
    expect(
        &d,
        "UnsupportedVersion",
        Severity::Error,
        &format!("{NO_PARSER}1.26.40-beta.1"),
    );
    none(&d, "PrereleaseFormatVersion");
}

#[test]
fn v05_missing_version() {
    // A root that holds only the payload fails the two-member rule first.
    let text =
        r#"{"minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}}}"#;
    let d = rejected(atomic(text, &cx()));
    expect(
        &d,
        "MalformedInput",
        Severity::Error,
        "A Minecraft Document must have exactly two members: \"format_version\" and a payload member",
    );
    none(&d, "MissingFormatVersion");
    // Two members, neither of them the version: the Puv loader's text.
    let text = r#"{"other": 1, "minecraft:atomic_client_entity": {"description": {"identifier": "demo:wisp"}}}"#;
    let d = rejected(atomic(text, &cx()));
    expect(
        &d,
        "MissingFormatVersion",
        Severity::Error,
        "no valid version passed by the caller or found in the input",
    );
    // Three members with the version: the two-member rule again (V4).
    let text = wisp("", "").replacen('{', r#"{"other": 1, "#, 1);
    let d = rejected(atomic(&text, &cx()));
    expect(
        &d,
        "MalformedInput",
        Severity::Error,
        "A Minecraft Document must have exactly two members: \"format_version\" and a payload member",
    );
}

#[test]
fn v06_malformed_version() {
    // The game's document input reads a number as its text: `1.26` is the
    // version 1.26.0, which has no parser.
    let text = wisp("", "").replace("\"beta\"", "1.26");
    let d = rejected(atomic(&text, &cx()));
    let codes: Vec<&str> = d.iter().map(|f| f.code.name()).collect();
    assert_eq!(codes, ["UnsupportedVersion"]);
    expect_prefix(&d, "UnsupportedVersion", Severity::Error, NO_PARSER);
    for version in ["true", "{}", "\"soon\""] {
        let text = wisp("", "").replace("\"beta\"", version);
        let d = rejected(atomic(&text, &cx()));
        expect(
            &d,
            "MalformedFormatVersion",
            Severity::Error,
            "Malformed format version",
        );
        expect(
            &d,
            "MissingFormatVersion",
            Severity::Error,
            "no valid version passed by the caller or found in the input",
        );
        let texts: Vec<&str> = d.iter().map(|f| f.code.name()).collect();
        assert_eq!(texts, ["MalformedFormatVersion", "MissingFormatVersion"]);
    }
}

#[test]
fn v07_wrong_payload_key() {
    let text = r#"{"format_version": "beta", "minecraft:client_entity": {"description": {"identifier": "demo:wisp"}}}"#;
    let d = rejected(atomic(text, &cx()));
    // The game builds its input with the key `minecraft:atomic_client_entity`, so the document input refuses the other key.
    expect(
        &d,
        "MissingPayload",
        Severity::Error,
        "The payload key does not match the expected one. Expected: 'minecraft:atomic_client_entity'. Actual: 'minecraft:client_entity'.",
    );
}

#[test]
fn v08_non_json_extension_and_v09_empty_file() {
    let wisp = wisp("", "");
    let files = [
        ("atomic_client_entities/wisp.txt", wisp.as_str()),
        ("atomic_client_entities/wisp.JSON", wisp.as_str()),
        ("atomic_client_entities/empty.json", ""),
    ];
    let set = stack(&[pack(&files)]);
    let entities = set.resource.atomic_client_entities();
    // The two files whose extension is not exactly `json` are never read.
    assert!(entities.is_empty());
    assert_eq!(entities.rejected().len(), 1);
    let found: Vec<String> = entities.diagnostics().map(line).collect();
    assert_eq!(found, ["Info EmptyDocument  empty document"]);
    // Read directly, an empty file is not loaded, with the info only.
    let d = rejected(atomic("", &cx()));
    assert_eq!(lines(&d), ["Info EmptyDocument  empty document"]);
}

#[test]
fn rule_v3_text_that_is_no_json_is_malformed_input() {
    let d = rejected(atomic(r#"{"format_version": "beta", "#, &cx()));
    assert!(find(&d, "MalformedInput", Severity::Error).is_some());
}

#[test]
fn the_beta_sentinel_spelled_out_is_the_literal() {
    // vanilla compares the version with its sentinel 9999.9999.9999-beta, so the
    // spelled-out sentinel loads as the literal does.
    let text = wisp("", "").replace("\"beta\"", "\"9999.9999.9999-beta\"");
    let document = loaded(atomic(&text, &cx()));
    assert!(document.version().beta);
}

// ---- schema ----

#[test]
fn s01_full() {
    let text = fixture(FAMILY, "s01_full.json");
    let document = loaded(atomic(&text, &cx()));
    silent(document.diagnostics());
    let d = &document.description;
    assert_eq!(d.identifier.as_str(), "demo:wisp");
    let visuals = document.visuals();
    assert_eq!(
        visuals.material("default").unwrap().as_str(),
        "entity_alphatest"
    );
    assert_eq!(
        visuals.texture("default").unwrap().as_str(),
        "textures/entity/wisp"
    );
    assert_eq!(
        visuals.geometry("default").unwrap().as_str(),
        "geometry.wisp"
    );
    assert_eq!(
        visuals.animation("idle").unwrap().as_str(),
        "animation.wisp.idle"
    );
    assert_eq!(d.particle_effects.as_ref().map(|m| m.len()), Some(0));
    assert_eq!(d.sound_effects.as_ref().map(|m| m.len()), Some(0));
    let scripts = d.scripts.as_ref().unwrap();
    assert_eq!(scripts.initialize.as_deref(), Some(&[][..]));
    assert_eq!(scripts.animate.as_ref().unwrap()[0].0, "idle");
    assert_eq!(
        d.render_controllers.as_ref().unwrap()[0]
            .controller()
            .unwrap()
            .as_str(),
        "controller.render.wisp"
    );
    assert_eq!(
        document.aabb_dimensions(),
        Some(&AabbDimensions::new(0.5, 0.5))
    );
    assert_eq!(visuals.render_controllers.len(), 1);
    assert_eq!(visuals.geometry.unwrap().len(), 1);
    // The box vanilla builds: half the width to each side, from the ground up.
    let (min, max) = document.aabb_dimensions().unwrap().bounds();
    assert_eq!((min, max), ([-0.25, 0.0, -0.25], [0.25, 0.5, 0.25]));
    // Value round trip: the members with a presence of their own keep their empty forms.
    let written = round_trip(&document, |text| atomic(text, &cx()));
    let json = |text: &str| {
        bedrock_addon::json::from_str_jsonc::<bedrock_addon::json::Value>(text).unwrap()
    };
    assert_eq!(json(&written), json(&text));
}

#[test]
fn s02_unknown_description_member() {
    for member in [
        r#""min_engine_version": "1.26.0""#,
        r#""spawn_egg": {}"#,
        r#""queryable_geometry": "default""#,
        r#""enable_attachables": true"#,
        r#""hide_armor": true"#,
        r#""held_item_scale": 1.0"#,
        r#""particle_emitters": {}"#,
    ] {
        let d = rejected(atomic(&wisp(member, ""), &cx()));
        assert_eq!(d.len(), 1, "{member}: {}", lines(&d).join("\n"));
        let name = member.split('"').nth(1).unwrap();
        expect_at(
            &d,
            "UnrecognizedField",
            Severity::Error,
            &format!("{DESCRIPTION}/{name}"),
        );
    }
}

#[test]
fn s03_unknown_script_member() {
    for member in [
        r#""variables": {"variable.x": "public"}"#,
        r#""scale": "1.0""#,
        r#""parent_setup": "1.0""#,
    ] {
        let d = rejected(atomic(
            &wisp(&format!(r#""scripts": {{{member}}}"#), ""),
            &cx(),
        ));
        let name = member.split('"').nth(1).unwrap();
        expect_at(
            &d,
            "UnrecognizedField",
            Severity::Error,
            &format!("{DESCRIPTION}/scripts/{name}"),
        );
    }
}

#[test]
fn s04_conditional_render_controller() {
    let members = r#""render_controllers": ["controller.render.ok", {"controller.render.wisp": "query.is_baby"}]"#;
    let d = rejected(atomic(&wisp(members, ""), &cx()));
    expect_at(
        &d,
        "InvalidType",
        Severity::Error,
        &format!("{DESCRIPTION}/render_controllers/1"),
    );
    assert_eq!(d.len(), 1);
}

#[test]
fn s05_conditional_animate() {
    let d = rejected(atomic(
        &wisp(r#""scripts": {"animate": [{"idle": "1.0"}]}"#, ""),
        &cx(),
    ));
    expect_at(
        &d,
        "InvalidType",
        Severity::Error,
        &format!("{DESCRIPTION}/scripts/animate/0"),
    );
    assert_eq!(d.len(), 1, "{}", lines(&d).join("\n"));
}

#[test]
fn rule_d6_wrong_json_types() {
    const BOX: &str = "minecraft:atomic_client_entity/components/minecraft:aabb_dimensions";
    // Each wrong type is reported once, at the member or element that has it.
    for (members, rest, at) in [
        (
            r#""materials": {"default": 3}"#,
            "",
            format!("{DESCRIPTION}/materials/default"),
        ),
        (
            r#""geometry": "geometry.wisp""#,
            "",
            format!("{DESCRIPTION}/geometry"),
        ),
        // An object where a string is declared: the finding is the value's, not a path
        // into it.
        (
            r#""textures": {"default": {"path": "textures/wisp"}}"#,
            "",
            format!("{DESCRIPTION}/textures/default"),
        ),
        (
            r#""scripts": {"initialize": [{"a": {"b": 1}}]}"#,
            "",
            format!("{DESCRIPTION}/scripts/initialize/0"),
        ),
        (
            r#""render_controllers": "controller.render.wisp""#,
            "",
            format!("{DESCRIPTION}/render_controllers"),
        ),
        (
            r#""scripts": {"initialize": "variable.x = 1;"}"#,
            "",
            format!("{DESCRIPTION}/scripts/initialize"),
        ),
        (
            r#""scripts": {"pre_animation": [1]}"#,
            "",
            format!("{DESCRIPTION}/scripts/pre_animation/0"),
        ),
        (
            r#""scripts": {"pre_animation": ["a", 1]}"#,
            "",
            format!("{DESCRIPTION}/scripts/pre_animation/1"),
        ),
        (
            "",
            r#""components": {"minecraft:aabb_dimensions": {"width": "1", "height": 1}}"#,
            format!("{BOX}/width"),
        ),
        (
            "",
            r#""components": {"minecraft:aabb_dimensions": {"width": 1, "height": true}}"#,
            format!("{BOX}/height"),
        ),
        (
            "",
            r#""components": []"#,
            "minecraft:atomic_client_entity/components".to_owned(),
        ),
    ] {
        let d = rejected(atomic(&wisp(members, rest), &cx()));
        expect_at(&d, "InvalidType", Severity::Error, &at);
        assert_eq!(d.len(), 1, "{members} {rest}: {}", lines(&d).join("\n"));
    }
    let d = rejected(atomic(
        &beta(r#"{"description": {"identifier": 5}}"#),
        &cx(),
    ));
    expect_at(
        &d,
        "InvalidType",
        Severity::Error,
        &format!("{DESCRIPTION}/identifier"),
    );
    assert_eq!(d.len(), 1, "{}", lines(&d).join("\n"));
}

#[test]
fn s06_unknown_component() {
    let rest = r#""components": {"minecraft:collision_box": {"width": 1, "height": 1}}"#;
    let d = rejected(atomic(&wisp("", rest), &cx()));
    expect_at(
        &d,
        "UnrecognizedComponent",
        Severity::Error,
        "minecraft:atomic_client_entity/components/minecraft:collision_box",
    );
    none(&d, "UnrecognizedField");
    // An unknown member inside the known component is a member, not a component.
    let rest =
        r#""components": {"minecraft:aabb_dimensions": {"width": 1, "height": 1, "depth": 1}}"#;
    let d = rejected(atomic(&wisp("", rest), &cx()));
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/depth",
    );
    none(&d, "UnrecognizedComponent");
}

#[test]
fn s07_missing_identifier_and_rule_d3_missing_description() {
    let d = rejected(atomic(&beta(r#"{"description": {}}"#), &cx()));
    expect_at(
        &d,
        "MissingRequiredField",
        Severity::Error,
        &format!("{DESCRIPTION}/identifier"),
    );
    let d = rejected(atomic(&beta("{}"), &cx()));
    expect_at(&d, "MissingRequiredField", Severity::Error, DESCRIPTION);
}

#[test]
fn s08_identifier_no_namespace() {
    let d = rejected(atomic(
        &beta(r#"{"description": {"identifier": "wisp"}}"#),
        &cx(),
    ));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "string must be prefixed with a namespace (eg. namespace:value)",
    );
    expect_at(
        &d,
        "InvalidValue",
        Severity::Error,
        &format!("{DESCRIPTION}/identifier"),
    );
}

#[test]
fn s13_identifier_namespace_edges() {
    for bad in [":wisp", "demo:", ":", ""] {
        let text = beta(&format!(r#"{{"description": {{"identifier": "{bad}"}}}}"#));
        let d = rejected(atomic(&text, &cx()));
        expect(
            &d,
            "InvalidValue",
            Severity::Error,
            "string must be prefixed with a namespace (eg. namespace:value)",
        );
    }
    for good in ["minecraft:wisp", "a:b:c", "a:b"] {
        let text = beta(&format!(r#"{{"description": {{"identifier": "{good}"}}}}"#));
        let document = loaded(atomic(&text, &cx()));
        assert_eq!(document.description.identifier.as_str(), good);
    }
}

fn boxed(width: &str, height: &str) -> String {
    wisp(
        "",
        &format!(
            r#""components": {{"minecraft:aabb_dimensions": {{"width": {width}, "height": {height}}}}}"#
        ),
    )
}

#[test]
fn s09_negative_width() {
    let at = "minecraft:atomic_client_entity/components/minecraft:aabb_dimensions";
    let d = rejected(atomic(&boxed("-1", "1"), &cx()));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "the input must be a number >= 0",
    );
    expect_at(&d, "InvalidValue", Severity::Error, &format!("{at}/width"));
    assert_eq!(d.len(), 1);
    // The height has the same bound, and both are reported.
    let d = rejected(atomic(&boxed("1", "-0.5"), &cx()));
    expect_at(&d, "InvalidValue", Severity::Error, &format!("{at}/height"));
    assert_eq!(rejected(atomic(&boxed("-1", "-1"), &cx())).len(), 2);
}

#[test]
fn s10_missing_height() {
    let at = "minecraft:atomic_client_entity/components/minecraft:aabb_dimensions";
    let rest = r#""components": {"minecraft:aabb_dimensions": {"width": 1}}"#;
    let d = rejected(atomic(&wisp("", rest), &cx()));
    expect_at(
        &d,
        "MissingRequiredField",
        Severity::Error,
        &format!("{at}/height"),
    );
    let rest = r#""components": {"minecraft:aabb_dimensions": {"height": 1}}"#;
    let d = rejected(atomic(&wisp("", rest), &cx()));
    expect_at(
        &d,
        "MissingRequiredField",
        Severity::Error,
        &format!("{at}/width"),
    );
}

#[test]
fn s11_zero_box() {
    let document = loaded(atomic(&boxed("0", "0"), &cx()));
    silent(document.diagnostics());
    assert_eq!(
        document.aabb_dimensions(),
        Some(&AabbDimensions::new(0.0, 0.0))
    );
    // An empty `components` is allowed and kept.
    let document = loaded(atomic(&wisp("", r#""components": {}"#), &cx()));
    assert_eq!(document.components, Some(Components::default()));
}

#[test]
fn s12_unknown_root_member() {
    let d = rejected(atomic(&wisp("", r#""events": {}"#), &cx()));
    expect_at(
        &d,
        "UnrecognizedField",
        Severity::Error,
        "minecraft:atomic_client_entity/events",
    );
}

#[test]
fn s14_partial_scripts() {
    let document = loaded(atomic(
        &wisp(r#""scripts": {"animate": ["idle"]}"#, ""),
        &cx(),
    ));
    let scripts = document.description.scripts.as_ref().unwrap();
    assert_eq!(scripts.initialize, None);
    assert_eq!(scripts.pre_animation, None);
    assert_eq!(scripts.animate.as_ref().unwrap().len(), 1);
    // Written back as written: no member is invented.
    let written = round_trip(&document, |text| atomic(text, &cx()));
    assert!(!written.contains("initialize"), "{written}");
}

#[test]
fn r08_scripts_not_compiled() {
    let members = r#""scripts": {"pre_animation": ["variable.x = query.not_a_query;", "1 +"], "initialize": ["{"]}"#;
    let document = loaded(atomic(&wisp(members, ""), &cx()));
    silent(document.diagnostics());
    let scripts = document.description.scripts.as_ref().unwrap();
    assert_eq!(scripts.pre_animation.as_ref().unwrap()[1].0, "1 +");
}

#[test]
fn names_are_kept_as_written() {
    // no lower-casing.
    let members = r#""materials": {"Default": "Entity_AlphaTest"}, "render_controllers": ["Controller.Render.Wisp"]"#;
    let text = beta(&format!(
        r#"{{"description": {{"identifier": "Demo:Wisp", {members}}}}}"#
    ));
    let document = loaded(atomic(&text, &cx()));
    assert_eq!(document.description.identifier.as_str(), "Demo:Wisp");
    assert!(document.visuals().material("Default").is_some());
    assert!(matches!(
        &document.visuals().render_controllers[0],
        ControllerUse::Always(id) if id.as_str() == "Controller.Render.Wisp"
    ));
}

#[test]
fn a_built_entity_writes_a_beta_document_at_its_default_path() {
    let wisp = AtomicClientEntity {
        components: Some(Components {
            aabb_dimensions: Some(AabbDimensions::new(0.5, 1.0)),
            ..Components::default()
        }),
        ..AtomicClientEntity::new(id("demo:wisp"))
    };
    assert_eq!(
        wisp.default_path().as_str(),
        "atomic_client_entities/demo/wisp.json"
    );
    let text = wisp.to_json().unwrap();
    assert!(text.contains(r#""format_version": "beta""#), "{text}");
    let again = loaded(atomic(&text, &cx()));
    assert_eq!(again.value(), &wisp);
    assert_eq!(again.to_json().unwrap(), text);
}

// ---- the chain ----

#[test]
fn the_chain_is_the_beta_struct_alone() {
    // No stable struct: a stable version of any height has no parser (v03), and the document
    // that loads is not upgraded (v01).
    let text = wisp("", "").replace("\"beta\"", "\"9999.0.0\"");
    assert!(atomic(&text, &cx()).is_err());
    let document = loaded(atomic(&wisp("", ""), &cx()));
    assert_eq!(document.version().effective, BETA_VERSION);
    assert_ne!(document.version().effective, FormatVersion::new(0, 0, 0));
}

// ---- stack ----

const FILE: &str = "atomic_client_entities/wisp.json";

#[test]
fn k01_top_pack_wins() {
    let top = wisp(r#""geometry": {"default": "geometry.top"}"#, "");
    let bottom = wisp(r#""geometry": {"default": "geometry.bottom"}"#, "");
    let set = stack(&[pack(&[(FILE, &top)]), pack(&[(FILE, &bottom)])]);
    let entities = set.resource.atomic_client_entities();
    let entity = entities.get(&id("demo:wisp")).unwrap();
    assert_eq!(
        entity.visuals().geometry("default").unwrap().as_str(),
        "geometry.top"
    );
    assert_eq!(entity.origin().pack.name(), "pack 1");
    let shadowed: Vec<_> = entities
        .diagnostics()
        .filter(|f| f.code.name() == "AtomicClientEntityShadowed")
        .collect();
    assert_eq!(shadowed.len(), 1);
    assert_eq!(shadowed[0].severity, Severity::Info);
    assert_eq!(shadowed[0].location.pack.as_ref().unwrap().name(), "pack 2");
    assert_eq!(entities.len(), 1);
}

#[test]
fn k01_inside_a_pack_the_last_file_wins() {
    let a = wisp(r#""geometry": {"default": "geometry.a"}"#, "");
    let b = wisp(r#""geometry": {"default": "geometry.b"}"#, "");
    let files = [
        ("atomic_client_entities/a.json", a.as_str()),
        ("atomic_client_entities/b.json", b.as_str()),
    ];
    let set = stack(&[pack(&files)]);
    let entities = set.resource.atomic_client_entities();
    let entity = entities.get(&id("demo:wisp")).unwrap();
    assert_eq!(
        entity.visuals().geometry("default").unwrap().as_str(),
        "geometry.b"
    );
}

#[test]
fn k02_replace_not_merge() {
    let top = wisp(r#""geometry": {"default": "geometry.top"}"#, "");
    let bottom = wisp(
        r#""textures": {"default": "textures/bottom"}, "geometry": {"default": "geometry.bottom"}"#,
        "",
    );
    let set = stack(&[pack(&[(FILE, &top)]), pack(&[(FILE, &bottom)])]);
    let entities = set.resource.atomic_client_entities();
    let entity = entities.get(&id("demo:wisp")).unwrap();
    assert!(entity.description.textures.is_none());
    assert!(entity.origin().contributors.is_empty());
}

#[test]
fn k03_failed_top_keeps_lower() {
    let top = boxed("-1", "1");
    let bottom = wisp(r#""geometry": {"default": "geometry.bottom"}"#, "");
    let set = stack(&[pack(&[(FILE, &top)]), pack(&[(FILE, &bottom)])]);
    let entities = set.resource.atomic_client_entities();
    let entity = entities.get(&id("demo:wisp")).unwrap();
    assert_eq!(entity.origin().pack.name(), "pack 2");
    assert_eq!(entities.rejected().len(), 1);
    assert_eq!(entities.rejected()[0].origin.pack.name(), "pack 1");
    assert!(
        entities
            .diagnostics()
            .all(|f| f.code.name() != "AtomicClientEntityShadowed")
    );
}

#[cfg(feature = "client_entity")]
#[test]
fn k04_same_id_as_client_entity() {
    use bedrock_addon::id::EntityId;
    let client = r#"{"format_version": "1.10.0", "minecraft:client_entity": {"description": {"identifier": "demo:wisp"}}}"#;
    let atomic = wisp("", "");
    let files = [("entity/wisp.json", client), (FILE, atomic.as_str())];
    let set = stack(&[pack(&files)]);
    assert!(
        set.resource
            .client_entities()
            .contains(&EntityId::parse("demo:wisp"))
    );
    assert!(
        set.resource
            .atomic_client_entities()
            .contains(&id("demo:wisp"))
    );
    assert!(
        set.resource
            .client_entities()
            .diagnostics()
            .next()
            .is_none()
    );
    assert!(
        set.resource
            .atomic_client_entities()
            .diagnostics()
            .next()
            .is_none()
    );
}

#[test]
fn files_in_sub_folders_are_read() {
    // the directory is read with its sub-folders.
    let wisp = wisp("", "");
    let files = [("atomic_client_entities/demo/wisp.json", wisp.as_str())];
    let set = stack(&[pack(&files)]);
    assert_eq!(set.resource.atomic_client_entities().len(), 1);
}

// ---- further cases ----

#[test]
fn a_box_size_beyond_f32_is_read_as_the_largest_finite_float_deviation() {
    let at = "minecraft:atomic_client_entity/components/minecraft:aabb_dimensions";
    let document = loaded(atomic(&boxed("1e39", "1e300"), &cx()));
    assert_eq!(
        document.aabb_dimensions(),
        Some(&AabbDimensions::new(f32::MAX, f32::MAX))
    );
    expect_at(
        document.diagnostics(),
        "InvalidValue",
        Severity::Info,
        &format!("{at}/width"),
    );
    expect_at(
        document.diagnostics(),
        "InvalidValue",
        Severity::Info,
        &format!("{at}/height"),
    );
    let written = round_trip(&document, |text| atomic(text, &cx()));
    silent(loaded(atomic(&written, &cx())).diagnostics());
    // A number no double holds refuses the document, as the game's JSON parser does.
    let d = rejected(atomic(&boxed("1e39", "1e999"), &cx()));
    expect(
        &d,
        "MalformedInput",
        Severity::Error,
        "Failed to parse the Minecraft Document JSON: Number too big to be stored in double.",
    );
    // Below zero it is still below zero: the constraint refuses it.
    let d = rejected(atomic(&boxed("-1e39", "1"), &cx()));
    expect(
        &d,
        "InvalidValue",
        Severity::Error,
        "the input must be a number >= 0",
    );
}

// ---- further cases ----

/// The description members of `text`, as JSON.
fn description_of(text: &str) -> bedrock_addon::json::Value {
    let json: bedrock_addon::json::Value = bedrock_addon::json::from_str_jsonc(text).unwrap();
    json.as_object().unwrap()["minecraft:atomic_client_entity"]
        .as_object()
        .unwrap()["description"]
        .clone()
}

#[test]
fn an_authored_empty_visuals_member_is_written_back() {
    // The four `Visuals` maps and
    // `render_controllers` keep their presence, so an empty one the author wrote is written
    // as it was read. One member at a time, then all five.
    let each = [
        r#""materials": {}"#,
        r#""textures": {}"#,
        r#""geometry": {}"#,
        r#""animations": {}"#,
        r#""render_controllers": []"#,
    ];
    let all = each.join(", ");
    for members in each.iter().copied().chain([all.as_str()]) {
        let document = loaded(atomic(&wisp(members, ""), &cx()));
        silent(document.diagnostics());
        let written = round_trip(&document, |text| atomic(text, &cx()));
        assert_eq!(
            description_of(&written),
            description_of(&wisp(members, "")),
            "{members}"
        );
    }
    // A document without the members gets none, and is the value built by hand.
    let document = loaded(atomic(&wisp("", ""), &cx()));
    assert_eq!(document.value(), &AtomicClientEntity::new(id("demo:wisp")));
    let written = round_trip(&document, |text| atomic(text, &cx()));
    assert_eq!(description_of(&written), description_of(&wisp("", "")));
    // The members that always had a presence of their own keep their empty forms beside
    // them.
    let kept = r#""particle_effects": {}, "sound_effects": {}, "scripts": {}"#;
    let document = loaded(atomic(&wisp(&format!("{all}, {kept}"), ""), &cx()));
    let written = round_trip(&document, |text| atomic(text, &cx()));
    assert_eq!(
        description_of(&written),
        description_of(&wisp(&format!("{all}, {kept}"), ""))
    );
    // A member that is not empty is written.
    let full = r#""materials": {"default": "m"}, "textures": {"default": "t"}, "geometry": {"default": "g"}, "animations": {"idle": "a"}, "render_controllers": ["c"]"#;
    let document = loaded(atomic(&wisp(full, ""), &cx()));
    let written = round_trip(&document, |text| atomic(text, &cx()));
    assert_eq!(description_of(&written), description_of(&wisp(full, "")));
}

// ---- the clamp changes no other finding ----

const WIDTH_AT: &str = "minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/width";
const HEIGHT_AT: &str =
    "minecraft:atomic_client_entity/components/minecraft:aabb_dimensions/height";

fn atomic_findings(text: &str) -> bedrock_addon::diagnostic::Diagnostics {
    findings(atomic(text, &cx()))
}

#[test]
fn two_clamped_sides_leave_every_later_finding_where_it_is() {
    for later in [
        r#""render_controllers": [1]"#,
        r#""scripts": {"animate": ["idle", {"idle": "1.0"}]}"#,
        r#""materials": {"default": 3}"#,
        r#""hide_armor": true"#,
        r#""render_controllers": ["a", {"b": "1.0"}]"#,
        r#""geometry": {"default": "geometry.wisp"}"#,
    ] {
        let text = format!(
            r#"{{
  "format_version": "beta",
  "minecraft:atomic_client_entity": {{
    "components": {{
      /* the box,
         over two lines: 1e40 */
      "minecraft:aabb_dimensions": {{ "width": 1e39, // wide
        "height": 1e300 }}
    }},
    "description": {{
      "identifier": "demo:wisp",
      {later}
    }}
  }}
}}"#
        );
        clamp_changes_no_other_finding(
            &text,
            &[("1e39", "1e30", WIDTH_AT), ("1e300", "1e-09", HEIGHT_AT)],
            atomic_findings,
        );
    }
}

#[test]
fn a_clamped_side_leaves_a_finding_between_and_after_the_sides_where_it_is() {
    // One side beyond `f32`, the other of the wrong type or below zero, in either order.
    for (sides, beyond, within, at) in [
        (
            r#""width": 1e39, "height": "tall""#,
            "1e39",
            "1e30",
            WIDTH_AT,
        ),
        (r#""width": [], "height": 1e39"#, "1e39", "1e30", HEIGHT_AT),
        (r#""width": 1e39, "height": -1"#, "1e39", "1e30", WIDTH_AT),
        (
            r#""height": -1e39, "width": -2"#,
            "-1e39",
            "-1e30",
            HEIGHT_AT,
        ),
        (r#""width": 1e39"#, "1e39", "1e30", WIDTH_AT),
    ] {
        let text = wisp(
            "",
            &format!(r#""components": {{"minecraft:aabb_dimensions": {{{sides}}}}}"#),
        );
        clamp_changes_no_other_finding(&text, &[(beyond, within, at)], atomic_findings);
    }
}

// ---- the clamped number is found by its member, not by its text ----

#[test]
fn clamped_sides_whose_text_occurs_earlier_are_found_at_their_members() {
    // Both sides are the same characters, which also stand in a comment, in string values
    // and in a key before them: each warning is on its own member's token.
    for token in ["1e39", "1e300"] {
        let text = format!(
            r#"{{"format_version": "beta", "minecraft:atomic_client_entity": {{
  "description": {{
    "identifier": "demo:wisp", // "width": {token}
    "materials": {{"{token}": "{token}"}},
    "textures": {{"default": "textures/{token}"}},
    "scripts": {{"initialize": ["variable.width = {token};"]}}
  }},
  /* "height": {token} */
  "components": {{"minecraft:aabb_dimensions": {{"height": {token}, "width": {token}}}}}
}}}}"#
        );
        let position = |member: &str| text.rfind(member).unwrap() + member.len();
        let (height, width) = (position("\"height\": "), position("\"width\": "));
        assert!(text.find(token).unwrap() < height && height < width);
        let document = loaded(atomic(&text, &cx()));
        clamp_warning_is_at(document.diagnostics(), HEIGHT_AT, height, token.len());
        clamp_warning_is_at(document.diagnostics(), WIDTH_AT, width, token.len());
        assert_eq!(document.diagnostics().len(), 2);
        assert_eq!(
            document.aabb_dimensions(),
            Some(&AabbDimensions::new(f32::MAX, f32::MAX))
        );
        // Every other occurrence is as written, in the value and in what is written back.
        let d = &document.description;
        assert_eq!(d.materials.as_ref().unwrap()[token].as_str(), token);
        let written = document.to_json().unwrap();
        for kept in [
            format!(r#""{token}": "{token}""#),
            format!(r#""default": "textures/{token}""#),
            format!(r#""variable.width = {token};""#),
        ] {
            assert!(written.contains(&kept), "{kept} in {written}");
        }
        assert_eq!(written.matches(token).count(), 4, "{written}");
    }
}

// ---- 1.26.60 ----

const NOT_NEGATIVE: &str = "the input must be a number >= 0";
const COMPONENTS: &str = "minecraft:atomic_client_entity/components";

/// A `demo:wisp` whose `components` are `components` (the members of the JSON object).
fn with_components(components: &str) -> String {
    wisp("", &format!(r#""components": {{{components}}}"#))
}

/// The 1.26.45 key is no component of 1.26.60: no alias of it is bound, so under the
/// strict mask it is an unknown component (`0x400`) that refuses the file.
#[test]
fn a01_the_1_26_45_collision_box_is_an_unknown_component() {
    let text = with_components(r#""minecraft:collision_box": {"width": 1, "height": 1}"#);
    let d = rejected(atomic(&text, &cx()));
    expect_at(
        &d,
        "UnrecognizedComponent",
        Severity::Error,
        &format!("{COMPONENTS}/minecraft:collision_box"),
    );
    // The same members under the new key load.
    let text = with_components(r#""minecraft:aabb_dimensions": {"width": 1, "height": 2}"#);
    let document = loaded(atomic(&text, &cx()));
    silent(document.diagnostics());
    assert_eq!(
        document.aabb_dimensions(),
        Some(&AabbDimensions::new(1.0, 2.0))
    );
    round_trip(&document, |text| atomic(text, &cx()));
}

/// `minecraft:companion_block_avoidance`: `backup_anchor_offsets` required,
/// `require_hidden_before_fallback` false and `anchor_switch_cooldown` 200 when left out.
#[test]
fn a02_companion_block_avoidance_reads_its_offsets_and_defaults() {
    let text = with_components(
        r#""minecraft:companion_block_avoidance": {"backup_anchor_offsets": [[0, 0.6, -0.8], [1, 2, 3]]}"#,
    );
    let document = loaded(atomic(&text, &cx()));
    silent(document.diagnostics());
    let avoidance = document.companion_block_avoidance().unwrap();
    assert_eq!(
        avoidance.to_offsets(),
        vec![[0.0, 0.6, -0.8], [1.0, 2.0, 3.0]]
    );
    assert_eq!(avoidance.require_hidden_before_fallback, None);
    assert!(!avoidance.require_hidden_before_fallback());
    assert_eq!(avoidance.anchor_switch_cooldown, None);
    assert_eq!(avoidance.anchor_switch_cooldown(), 200);
    assert_eq!(CompanionBlockAvoidance::DEFAULT_ANCHOR_SWITCH_COOLDOWN, 200);
    let written = round_trip(&document, |text| atomic(text, &cx()));
    assert!(!written.contains("anchor_switch_cooldown"), "{written}");
    // Written values win; the cooldown is a u16.
    let text = with_components(
        r#""minecraft:companion_block_avoidance": {"backup_anchor_offsets": [], "require_hidden_before_fallback": true, "anchor_switch_cooldown": 65535}"#,
    );
    let document = loaded(atomic(&text, &cx()));
    let avoidance = document.companion_block_avoidance().unwrap();
    assert!(avoidance.require_hidden_before_fallback());
    assert_eq!(avoidance.anchor_switch_cooldown(), 65535);
    round_trip(&document, |text| atomic(text, &cx()));
    let at = format!("{COMPONENTS}/minecraft:companion_block_avoidance");
    for (bad, code) in [
        ("65536", "InvalidType"),
        ("-1", "InvalidType"),
        ("1.5", "InvalidType"),
        ("\"200\"", "InvalidType"),
    ] {
        let text = with_components(&format!(
            r#""minecraft:companion_block_avoidance": {{"backup_anchor_offsets": [], "anchor_switch_cooldown": {bad}}}"#
        ));
        let d = rejected(atomic(&text, &cx()));
        expect_at(
            &d,
            code,
            Severity::Error,
            &format!("{at}/anchor_switch_cooldown"),
        );
    }
    // The offsets are required.
    let text = with_components(r#""minecraft:companion_block_avoidance": {}"#);
    let d = rejected(atomic(&text, &cx()));
    expect_at(
        &d,
        "MissingRequiredField",
        Severity::Error,
        &format!("{at}/backup_anchor_offsets"),
    );
    // An offset is three numbers.
    let text = with_components(
        r#""minecraft:companion_block_avoidance": {"backup_anchor_offsets": [[0, 1]]}"#,
    );
    let d = rejected(atomic(&text, &cx()));
    // The typed read fails on the list of three-number arrays: a wrong type at the list.
    expect_at(
        &d,
        "InvalidType",
        Severity::Error,
        &format!("{at}/backup_anchor_offsets"),
    );
}

/// `minecraft:companion_block_clipping` and `CompanionVisibilityBoundary`: every member may be left out, with the constructor's defaults.
#[test]
fn a03_companion_block_clipping_defaults() {
    let text = with_components(r#""minecraft:companion_block_clipping": {}"#);
    let document = loaded(atomic(&text, &cx()));
    silent(document.diagnostics());
    let clipping = document.companion_block_clipping().unwrap();
    assert_eq!(clipping, &CompanionBlockClipping::default());
    let boundary = VisibilityBoundary::default();
    assert_eq!(
        (
            boundary.margin(),
            boundary.lookahead_per_speed(),
            boundary.lookahead_max()
        ),
        (0.5, 0.1, 1.0)
    );
    assert_eq!(clipping.reveal_boundary(), boundary);
    assert_eq!(clipping.hide_boundary(), boundary);
    assert_eq!(clipping.min_hidden_seconds(), 2.0);
    assert_eq!(clipping.min_visible_seconds(), 0.0);
    assert_eq!(clipping.hide_delay_seconds(), 0.0);
    assert_eq!(clipping.hide_transition_seconds(), 0.25);
    let written = round_trip(&document, |text| atomic(text, &cx()));
    assert!(
        written.contains(r#""minecraft:companion_block_clipping": {}"#),
        "{written}"
    );
    // A boundary that writes one member keeps the defaults of the others.
    let text = with_components(
        r#""minecraft:companion_block_clipping": {"hide_boundary": {"margin": 2}, "min_hidden_seconds": 0}"#,
    );
    let document = loaded(atomic(&text, &cx()));
    let clipping = document.companion_block_clipping().unwrap();
    let hide = clipping.hide_boundary();
    assert_eq!(
        (
            hide.margin(),
            hide.lookahead_per_speed(),
            hide.lookahead_max()
        ),
        (2.0, 0.1, 1.0)
    );
    assert_eq!(clipping.min_hidden_seconds(), 0.0);
    round_trip(&document, |text| atomic(text, &cx()));
}

/// Every companion float has an inclusive minimum of 0.0: 0 loads,
/// below 0 is an error with the constraint's text.
#[test]
fn a04_the_companion_floats_are_at_least_zero() {
    let clipping = format!("{COMPONENTS}/minecraft:companion_block_clipping");
    for (members, at) in [
        (
            r#""min_hidden_seconds": VALUE"#,
            "min_hidden_seconds".to_owned(),
        ),
        (
            r#""min_visible_seconds": VALUE"#,
            "min_visible_seconds".to_owned(),
        ),
        (
            r#""hide_delay_seconds": VALUE"#,
            "hide_delay_seconds".to_owned(),
        ),
        (
            r#""hide_transition_seconds": VALUE"#,
            "hide_transition_seconds".to_owned(),
        ),
        (
            r#""reveal_boundary": {"margin": VALUE}"#,
            "reveal_boundary/margin".to_owned(),
        ),
        (
            r#""reveal_boundary": {"lookahead_per_speed": VALUE}"#,
            "reveal_boundary/lookahead_per_speed".to_owned(),
        ),
        (
            r#""reveal_boundary": {"lookahead_max": VALUE}"#,
            "reveal_boundary/lookahead_max".to_owned(),
        ),
        (
            r#""hide_boundary": {"margin": VALUE}"#,
            "hide_boundary/margin".to_owned(),
        ),
        (
            r#""hide_boundary": {"lookahead_per_speed": VALUE}"#,
            "hide_boundary/lookahead_per_speed".to_owned(),
        ),
        (
            r#""hide_boundary": {"lookahead_max": VALUE}"#,
            "hide_boundary/lookahead_max".to_owned(),
        ),
    ] {
        let text = |value: &str| {
            with_components(&format!(
                r#""minecraft:companion_block_clipping": {{{}}}"#,
                members.replace("VALUE", value)
            ))
        };
        loaded(atomic(&text("0"), &cx()));
        let d = rejected(atomic(&text("-0.001"), &cx()));
        expect(&d, "InvalidValue", Severity::Error, NOT_NEGATIVE);
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            &format!("{clipping}/{at}"),
        );
    }
    // The AABB keeps its two constraints under the new key: 0 loads, below 0 is refused.
    let text = with_components(r#""minecraft:aabb_dimensions": {"width": 0, "height": 0}"#);
    let document = loaded(atomic(&text, &cx()));
    silent(document.diagnostics());
    assert_eq!(
        document.aabb_dimensions(),
        Some(&AabbDimensions::new(0.0, 0.0))
    );
    for (members, at) in [
        (r#""width": -0.001, "height": 0"#, "width"),
        (r#""width": 0, "height": -0.001"#, "height"),
    ] {
        let text = with_components(&format!(r#""minecraft:aabb_dimensions": {{{members}}}"#));
        let d = rejected(atomic(&text, &cx()));
        expect(&d, "InvalidValue", Severity::Error, NOT_NEGATIVE);
        expect_at(
            &d,
            "InvalidValue",
            Severity::Error,
            &format!("{COMPONENTS}/minecraft:aabb_dimensions/{at}"),
        );
    }
}

/// Rule N2: a member written `null` is present and fails its type, the
/// optional members of the description and the companions included.
#[test]
fn a05_a_null_member_is_a_wrong_type() {
    for (members, rest, at) in [
        (
            r#""materials": null"#,
            "",
            format!("{DESCRIPTION}/materials"),
        ),
        (
            r#""scripts": {"animate": null}"#,
            "",
            format!("{DESCRIPTION}/scripts/animate"),
        ),
        ("", r#""components": null"#, COMPONENTS.to_owned()),
        (
            "",
            r#""components": {"minecraft:companion_block_avoidance": {"backup_anchor_offsets": [], "require_hidden_before_fallback": null}}"#,
            format!(
                "{COMPONENTS}/minecraft:companion_block_avoidance/require_hidden_before_fallback"
            ),
        ),
        (
            "",
            r#""components": {"minecraft:companion_block_clipping": {"hide_delay_seconds": null}}"#,
            format!("{COMPONENTS}/minecraft:companion_block_clipping/hide_delay_seconds"),
        ),
    ] {
        let d = rejected(atomic(&wisp(members, rest), &cx()));
        expect_at(&d, "InvalidType", Severity::Error, &at);
    }
}

/// A hand-written document with all three components loads, round-trips, and is the value
/// built by hand.
#[test]
fn a06_a_document_with_the_three_components_round_trips() {
    let text = fixture(FAMILY, "a06_companions.json");
    let document = loaded(atomic(&text, &cx()));
    silent(document.diagnostics());
    let components = document.components.as_ref().unwrap();
    let expected = Components {
        aabb_dimensions: Some(AabbDimensions::new(0.6, 0.8)),
        companion_block_avoidance: Some(CompanionBlockAvoidance::new(vec![
            [0.0, 0.6, -0.8],
            [0.5, 0.6, 0.0],
        ])),
        companion_block_clipping: Some(CompanionBlockClipping {
            reveal_boundary: Some(VisibilityBoundary {
                margin: Some(StrictFloat(0.25)),
                ..VisibilityBoundary::default()
            }),
            hide_transition_seconds: Some(StrictFloat(0.5)),
            ..CompanionBlockClipping::default()
        }),
    };
    let mut expected = expected;
    if let Some(avoidance) = &mut expected.companion_block_avoidance {
        avoidance.anchor_switch_cooldown = Some(StrictInt(100));
    }
    assert_eq!(components, &expected);
    round_trip(&document, |text| atomic(text, &cx()));
}
