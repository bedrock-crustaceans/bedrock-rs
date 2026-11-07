//! 20 R30 (and B8) and R31: a block's `minecraft:geometry` `culling` against the culling
//! rules of the resource stack, and the rules it names against the block's geometry and its
//! culling layer.

use bedrock_addon::diagnostic::{
    Diagnostic, DiagnosticDetail, Diagnostics, Phase, ReferenceHint, Severity,
};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::LinkOptions;

#[cfg(feature = "molang")]
use crate::common::{behavior_pack, resource_pack, stacks_of};
use crate::common::{hosted, link, link_with};

const BLOCK: &str = "blocks/lamp.json";
const RULES: &str = "block_culling/lamp.json";
const GEOMETRY_FILE: &str = "models/blocks/lamp.geo.json";
const CULLING: &str = "minecraft:block/components/minecraft:geometry/culling";

/// A block `demo:lamp` whose `minecraft:geometry` is `geometry.lamp` with `members` added,
/// and with `more` after the block's components.
fn block_with(members: &str, more: &str) -> String {
    format!(
        r#"{{"format_version": "1.26.20", "minecraft:block": {{"description": {{"identifier": "demo:lamp"}}, "components": {{"minecraft:geometry": {{"identifier": "geometry.lamp"{members}}}, "minecraft:material_instances": {{"*": {{"texture": "lamp"}}}}}}{more}}}}}"#
    )
}

fn block(members: &str) -> String {
    block_with(members, "")
}

/// `geometry.lamp`: the bone `body` with two cubes and the bone `Arm` with none.
const GEOMETRY: &str = r#"{"format_version": "1.21.0", "minecraft:geometry": [{"description": {"identifier": "geometry.lamp", "texture_width": 16, "texture_height": 16}, "bones": [{"name": "body", "cubes": [{"origin": [0, 0, 0], "size": [1, 1, 1], "uv": [0, 0]}, {"origin": [1, 0, 0], "size": [1, 1, 1], "uv": [0, 0]}]}, {"name": "Arm"}]}]}"#;

/// The culling rules `id` at 1.21.80 with `rules`.
fn culling_of(id: &str, rules: &str) -> String {
    format!(
        r#"{{"format_version": "1.21.80", "minecraft:block_culling_rules": {{"description": {{"identifier": "{id}"}}, "rules": [{rules}]}}}}"#
    )
}

fn culling(rules: &str) -> String {
    culling_of("demo:lamp_culling", rules)
}

const WHOLE_BODY: &str = r#"{"geometry_part": {"bone": "body"}, "direction": "up"}"#;
const NAMED: &str = r#", "culling": "demo:lamp_culling""#;

/// Everything a finding says: code, severity, phase, pack, file, JSON path, and the kind and
/// the name of its reference (empty without one).
type Said = (
    &'static str,
    Severity,
    Phase,
    String,
    String,
    String,
    &'static str,
    String,
);

fn said(finding: &Diagnostic) -> Said {
    let (kind, id) = match finding.detail.as_deref() {
        Some(DiagnosticDetail::Reference { kind, id, .. }) => (*kind, id.clone()),
        _ => ("", String::new()),
    };
    (
        finding.code.id(),
        finding.severity,
        finding.phase,
        finding
            .location
            .pack
            .as_ref()
            .map(|pack| pack.name().to_owned())
            .unwrap_or_default(),
        finding
            .location
            .path
            .as_ref()
            .map(|path| path.as_str().to_owned())
            .unwrap_or_default(),
        finding.location.json_path.to_string(),
        kind,
        id,
    )
}

fn hint(finding: &Diagnostic) -> Option<&ReferenceHint> {
    match finding.detail.as_deref()? {
        DiagnosticDetail::Reference { hint, .. } => hint.as_ref(),
        _ => None,
    }
}

/// The findings whose reference is of `kind`.
fn of_kind(found: &Diagnostics, kind: &str) -> Vec<Said> {
    found
        .iter()
        .map(said)
        .filter(|said| said.6 == kind)
        .collect()
}

/// The findings of the world with `block` and, in its resource pack, the geometry and the
/// culling rules `rules`.
fn linked(block: &str, rules: &str) -> Diagnostics {
    link(
        &[(BLOCK, block)],
        &[(GEOMETRY_FILE, GEOMETRY), (RULES, rules)],
    )
}

// ---- R30 --------------------------------------------------------------------------------------

#[test]
fn r30_culling_rules_a_pack_defines_are_no_finding() {
    let found = linked(&block(NAMED), &culling(WHOLE_BODY));
    assert_eq!(of_kind(&found, "culling rules"), []);
}

#[test]
fn r30_a_name_no_rules_document_has_is_a_kept_error_at_the_member() {
    let found = linked(&block(r#", "culling": "demo:nope""#), &culling(WHOLE_BODY));
    // The game logs the miss at level 3 and the
    // block loads without a schematic: a kept error.
    assert_eq!(
        of_kind(&found, "culling rules"),
        [(
            "core/unresolved_reference_worded",
            Severity::Error,
            Phase::Link,
            "bp 1".to_owned(),
            BLOCK.to_owned(),
            CULLING.to_owned(),
            "culling rules",
            "demo:nope".to_owned()
        )]
    );
    let missing: Vec<&Diagnostic> = found
        .iter()
        .filter(|finding| said(finding).6 == "culling rules")
        .collect();
    assert_eq!(
        missing[0].code.to_string(),
        " -> components -> minecraft:geometry -> culling:  Missing referenced asset demo:nope"
    );
    assert_eq!(hint(missing[0]), None);
    assert!(missing[0].kept, "the block loads");
    // A kept error is no lint.
    let found = link_with(
        &[(BLOCK, &block(r#", "culling": "demo:nope""#))],
        &[],
        LoadOptions::default(),
        LinkOptions {
            lints: false,
            ..Default::default()
        },
    );
    assert_eq!(of_kind(&found, "culling rules").len(), 1, "{found:#?}");
}

#[test]
fn r30_the_name_is_compared_as_written_with_the_near_miss_as_a_hint() {
    let found = linked(
        &block(r#", "culling": "demo:Lamp_culling""#),
        &culling(WHOLE_BODY),
    );
    let missing: Vec<&Diagnostic> = found
        .iter()
        .filter(|finding| said(finding).6 == "culling rules")
        .collect();
    assert_eq!(missing.len(), 1, "{found:#?}");
    assert_eq!(
        hint(missing[0]),
        Some(&ReferenceHint::CaseMismatch("demo:lamp_culling".to_owned()))
    );
}

#[test]
fn r30_culling_rules_of_the_game_are_a_host_s_to_know() {
    let lamp = block(r#", "culling": "minecraft:leaves_culling""#);
    // The crate has no list of the game's culling rules: without a host the name misses.
    let found = link(&[(BLOCK, &lamp)], &[]);
    assert_eq!(of_kind(&found, "culling rules").len(), 1, "{found:#?}");
    let found = link_with(
        &[(BLOCK, &lamp)],
        &[],
        hosted(&[("culling rules", "minecraft:leaves_culling")]),
        LinkOptions::default(),
    );
    assert_eq!(of_kind(&found, "culling rules"), []);
    let found = link_with(&[(BLOCK, &lamp)], &[], hosted(&[]), LinkOptions::default());
    assert_eq!(of_kind(&found, "culling rules").len(), 1, "{found:#?}");
}

#[test]
fn r30_a_permutation_s_geometry_is_checked_at_its_own_path() {
    let lamp = block_with(
        NAMED,
        r#", "permutations": [{"condition": "true", "components": {"minecraft:geometry": {"identifier": "geometry.lamp", "culling": "demo:nope"}, "minecraft:material_instances": {"*": {"texture": "lamp"}}}}]"#,
    );
    let found = linked(&lamp, &culling(WHOLE_BODY));
    let told: Vec<String> = of_kind(&found, "culling rules")
        .into_iter()
        .map(|said| said.5)
        .collect();
    assert_eq!(
        told,
        ["minecraft:block/permutations/0/components/minecraft:geometry/culling"],
        "{found:#?}"
    );
}

/// One of the packs allowed the legacy creator features: `minecraft:geometry` (released
/// 1.19.40, `requires = [LCF]`) loads in a file below its release.
#[cfg(feature = "molang")]
const ALLOW_LISTED: u128 = 0xda09_e399_6f5e_4287_a013_4cf1_3de3_2166;

/// A block of format 1.18.0 of an allow-listed pack whose `minecraft:geometry` is the object
/// form (`{"geometry": …}`) in its own components and in a permutation behind one the load
/// takes out (a condition the block-state rule rejects and the default rule compiles
/// (d); a `minecraft:sound` there refuses the block now): the 1.19.10 unwrap moves the geometry to the component itself, and R30 names
/// the `culling` the file has, `…/minecraft:geometry/geometry/culling`, at the permutation's
/// index in the file.
#[cfg(feature = "molang")]
#[test]
fn r30_a_culling_an_unwrap_moved_is_named_at_the_member_the_file_has() {
    let lamp = r#"{"format_version": "1.18.0", "minecraft:block": {
        "description": {"identifier": "demo:lamp", "properties": {"demo:lit": [false, true]}},
        "components": {"minecraft:geometry": {"geometry": {"identifier": "geometry.lamp", "culling": "demo:nope"}}},
        "permutations": [
            {"condition": "q.is_baby", "components": {"minecraft:friction": 0.4}},
            {"condition": "q.block_property('demo:lit') == true", "components": {"minecraft:geometry": {"geometry": {"identifier": "geometry.lamp", "culling": "demo:none"}}}}
        ]}}"#;
    let set = stacks_of(
        vec![behavior_pack(ALLOW_LISTED, &[(BLOCK, lamp)])],
        vec![resource_pack(
            2,
            &[(GEOMETRY_FILE, GEOMETRY), (RULES, &culling(WHOLE_BODY))],
        )],
        LoadOptions {
            molang: Some(std::sync::Arc::new(
                bedrock_addon::molang::MolangCheck::new(),
            )),
            ..LoadOptions::default()
        },
    );
    let blocks = set.behavior.blocks();
    let block = blocks
        .iter()
        .next()
        .unwrap_or_else(|| {
            panic!(
                "{:#?}",
                blocks
                    .diagnostics()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        })
        .1;
    assert_eq!(block.permutations.len(), 1, "{:#?}", block.diagnostics());
    let found = set.link_with(&LinkOptions::default());
    let told: Vec<String> = of_kind(&found, "culling rules")
        .into_iter()
        .map(|said| said.5)
        .collect();
    assert_eq!(
        told,
        [
            "minecraft:block/components/minecraft:geometry/geometry/culling",
            "minecraft:block/permutations/1/components/minecraft:geometry/geometry/culling",
        ],
        "{found:#?}"
    );
}

#[test]
fn r30_a_geometry_without_a_culling_name_is_no_reference() {
    // No `culling`, an empty one, and the short form of the component.
    for lamp in [
        block(""),
        block(r#", "culling": """#),
        r#"{"format_version": "1.26.20", "minecraft:block": {"description": {"identifier": "demo:lamp"}, "components": {"minecraft:geometry": "geometry.lamp", "minecraft:material_instances": {"*": {"texture": "lamp"}}}}}"#.to_owned(),
    ] {
        let found = linked(&lamp, &culling(WHOLE_BODY));
        assert_eq!(of_kind(&found, "culling rules"), [], "{lamp}");
    }
}

// ---- R31: the culling layer -------------------------------------------------------------------

const SAME_LAYER: &str =
    r#"{"condition": "same_culling_layer", "geometry_part": {"bone": "body"}, "direction": "up"}"#;
const NO_LAYER: &str =
    "culling condition specified is same_culling_layer, but geometry has no culling layer defined";

fn no_layer(found: &Diagnostics) -> Vec<Said> {
    found
        .iter()
        .filter(|finding| finding.code.to_string() == NO_LAYER)
        .map(said)
        .collect()
}

#[test]
fn r31_same_culling_layer_without_a_layer_is_told_at_the_rule_s_condition() {
    // Two rules for the same part would refuse the file: the second rule hides another bone.
    let rules = culling(&format!(
        "{WHOLE_BODY}, {}",
        SAME_LAYER.replace("body", "arm")
    ));
    let found = linked(&block(NAMED), &rules);
    assert_eq!(
        no_layer(&found),
        [(
            "core/no_effect_worded",
            Severity::Warning,
            Phase::Link,
            "rp 2".to_owned(),
            RULES.to_owned(),
            "minecraft:block_culling_rules/rules/1/condition".to_owned(),
            "",
            String::new()
        )]
    );
}

#[test]
fn r31_a_rule_two_blocks_use_is_told_once() {
    let second = block(NAMED).replace("demo:lamp\"}", "demo:lantern\"}");
    assert!(second.contains("demo:lantern"));
    let found = link(
        &[(BLOCK, &block(NAMED)), ("blocks/lantern.json", &second)],
        &[(GEOMETRY_FILE, GEOMETRY), (RULES, &culling(SAME_LAYER))],
    );
    assert_eq!(no_layer(&found).len(), 1, "{found:#?}");
}

#[test]
fn r31_rules_no_block_names_are_not_checked() {
    let found = linked(&block(""), &culling(SAME_LAYER));
    assert_eq!(no_layer(&found), []);
}

// ---- R31: the bone and the cube ---------------------------------------------------------------

#[cfg(feature = "geometry")]
mod parts {
    use super::*;

    fn cube(bone: &str, cube: u32) -> String {
        format!(
            r#"{{"geometry_part": {{"bone": "{bone}", "cube": {cube}, "face": "up"}}, "direction": "up"}}"#
        )
    }

    #[test]
    fn r31_a_bone_of_the_geometry_is_no_finding_whatever_its_case() {
        // The geometry keeps its bones by their lower-cased names.
        let rules = culling(
            r#"{"geometry_part": {"bone": "body"}, "direction": "up"}, {"geometry_part": {"bone": "BODY", "cube": 1, "face": "up"}, "direction": "down"}, {"geometry_part": {"bone": "arm"}, "direction": "north"}"#,
        );
        let found = linked(&block(NAMED), &rules);
        assert_eq!(of_kind(&found, "bone"), []);
        assert_eq!(of_kind(&found, "cube"), []);
    }

    #[test]
    fn r31_a_bone_the_geometry_lacks_is_a_kept_error_at_the_rule() {
        let rules = culling(&format!(
            r#"{WHOLE_BODY}, {{"geometry_part": {{"bone": "ghost"}}, "direction": "down"}}"#
        ));
        let found = linked(&block(NAMED), &rules);
        // The game logs the miss at level 3
        // and goes on with the next rule: a kept error.
        assert_eq!(
            of_kind(&found, "bone"),
            [(
                "core/unresolved_reference_worded",
                Severity::Error,
                Phase::Link,
                "rp 2".to_owned(),
                RULES.to_owned(),
                "minecraft:block_culling_rules/rules/1/geometry_part/bone".to_owned(),
                "bone",
                "ghost".to_owned()
            )]
        );
        let text: Vec<String> = found
            .iter()
            .filter(|finding| said(finding).6 == "bone")
            .map(|finding| finding.code.to_string())
            .collect();
        assert_eq!(text, ["Did not find any bones with name `ghost`"]);
        assert!(found.iter().filter(|f| said(f).6 == "bone").all(|f| f.kept));
    }

    #[test]
    fn r31_a_cube_beyond_the_bone_s_cubes_is_a_kept_error_at_the_rule() {
        // `body` has two cubes: 1 is the last one inside, 2 the first one beyond; `Arm` has
        // none, so its cube 0 is beyond.
        let rules = culling(&format!(
            "{}, {}, {}",
            cube("body", 1),
            cube("arm", 0),
            r#"{"geometry_part": {"bone": "body", "cube": 2, "face": "down"}, "direction": "down"}"#
        ));
        let found = linked(&block(NAMED), &rules);
        // Level 3, then the next rule: a kept error.
        assert_eq!(
            of_kind(&found, "cube"),
            [
                (
                    "core/invalid_value_worded",
                    Severity::Error,
                    Phase::Link,
                    "rp 2".to_owned(),
                    RULES.to_owned(),
                    "minecraft:block_culling_rules/rules/1/geometry_part/cube".to_owned(),
                    "cube",
                    "0".to_owned()
                ),
                (
                    "core/invalid_value_worded",
                    Severity::Error,
                    Phase::Link,
                    "rp 2".to_owned(),
                    RULES.to_owned(),
                    "minecraft:block_culling_rules/rules/2/geometry_part/cube".to_owned(),
                    "cube",
                    "2".to_owned()
                )
            ]
        );
        assert!(found.iter().filter(|f| said(f).6 == "cube").all(|f| f.kept));
        let text: Vec<String> = found
            .iter()
            .filter(|finding| said(finding).6 == "cube")
            .map(|finding| finding.code.to_string())
            .collect();
        assert_eq!(
            text,
            [
                "Cube index `0` is out of bounds for schematic part `arm`",
                "Cube index `2` is out of bounds for schematic part `body`"
            ]
        );
    }

    #[test]
    fn r31_a_geometry_no_pack_defines_has_no_bones_to_compare_with() {
        // The game's own geometry, with a host and without: its bones are not known.
        let lamp = block(NAMED).replace("geometry.lamp", "minecraft:geometry.full_block");
        let rules = culling(r#"{"geometry_part": {"bone": "ghost"}, "direction": "up"}"#);
        let found = link(&[(BLOCK, &lamp)], &[(RULES, &rules)]);
        assert_eq!(of_kind(&found, "bone"), []);
        let found = link_with(
            &[(BLOCK, &lamp)],
            &[(RULES, &rules)],
            hosted(&[("geometry", "minecraft:geometry.full_block")]),
            LinkOptions::default(),
        );
        assert_eq!(of_kind(&found, "bone"), []);
    }

    #[test]
    fn r31_a_rule_two_blocks_use_is_told_once_and_each_permutation_s_geometry_is_checked() {
        // The base geometry has `body`; the permutation's geometry is another one without it.
        let other = GEOMETRY
            .replace("geometry.lamp", "geometry.other")
            .replace(r#""name": "body""#, r#""name": "trunk""#);
        let lamp = block_with(
            NAMED,
            r#", "permutations": [{"condition": "true", "components": {"minecraft:geometry": {"identifier": "geometry.other", "culling": "demo:lamp_culling"}, "minecraft:material_instances": {"*": {"texture": "lamp"}}}}]"#,
        );
        let second = lamp.replace("demo:lamp\"}", "demo:lantern\"}");
        let found = link(
            &[(BLOCK, &lamp), ("blocks/lantern.json", &second)],
            &[
                (GEOMETRY_FILE, GEOMETRY),
                ("models/blocks/other.geo.json", &other),
                (RULES, &culling(WHOLE_BODY)),
            ],
        );
        let told = of_kind(&found, "bone");
        assert_eq!(told.len(), 1, "{found:#?}");
        assert_eq!(told[0].7, "body");
    }

    #[test]
    fn r31_the_rules_of_the_highest_pack_are_the_ones_checked() {
        let set = stacks_of(
            vec![behavior_pack(1, &[(BLOCK, &block(NAMED))])],
            vec![
                resource_pack(3, &[(RULES, &culling(WHOLE_BODY))]),
                resource_pack(
                    2,
                    &[
                        (GEOMETRY_FILE, GEOMETRY),
                        (
                            RULES,
                            &culling(r#"{"geometry_part": {"bone": "ghost"}, "direction": "up"}"#),
                        ),
                    ],
                ),
            ],
            LoadOptions::default(),
        );
        let found = set.link();
        assert_eq!(of_kind(&found, "bone"), []);
    }
}

/// Without the `geometry` feature a block's geometry is not in view: no bone and no cube is
/// checked.
#[cfg(not(feature = "geometry"))]
#[test]
fn r31_without_the_geometry_feature_no_bone_is_reported() {
    let rules = culling(r#"{"geometry_part": {"bone": "ghost"}, "direction": "up"}"#);
    let found = linked(&block(NAMED), &rules);
    assert_eq!(of_kind(&found, "bone"), []);
}
