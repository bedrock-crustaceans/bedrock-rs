//! The block deltas of 1.26.60: the
//! slices' input kind, null rule and error mask, the 1.26.20 → 1.26.60 step and the newest
//! document, the new components and the changed rules. The game's own test
//! documents under `fixtures/block/block/engine/` are the data.

use bedrock_addon::diagnostic::{Code, Diagnostics, Kind, Severity};
use bedrock_addon::load::LoadContext;

use super::common::{context, document, fixture, load, loads, rejected, ucf};

/// The findings at `path` (below `minecraft:block`) whose code passes `code`.
fn at<'d>(
    d: &'d Diagnostics,
    code: fn(&Code) -> bool,
    path: &str,
) -> Vec<&'d bedrock_addon::diagnostic::Diagnostic> {
    d.iter()
        .filter(|f| {
            code(&f.code) && f.location.json_path.to_string() == format!("minecraft:block/{path}")
        })
        .collect()
}

fn unrecognized_field(c: &Code) -> bool {
    // By legacy name: the variant and the definitions that were it.
    matches!(c.id(), "core/unrecognized_field")
}

fn unrecognized_component(c: &Code) -> bool {
    // By legacy name: the variant and the definitions that were it.
    matches!(
        c.id(),
        "block/child_not_valid"
            | "core/component_not_in_schema"
            | "core/unrecognized_component"
            | "core/unrecognized_component_worded"
            | "loot/unknown_condition"
            | "loot/unsupported_function"
            | "particle/unrecognized_component"
    )
}

fn invalid_type(c: &Code) -> bool {
    // By legacy name: the variant and the definitions that were it.
    matches!(c.id(), "core/invalid_type")
}

/// A base-game context (the game's test blocks are `minecraft:` blocks).
fn base_game() -> LoadContext {
    let mut cx = context();
    cx.is_base_game_pack = true;
    cx
}

/// A test document of `fixtures/block/block/engine/`.
fn gtest(name: &str) -> String {
    fixture(&format!("engine/{name}.json"))
}

// ---- the slices (legacy, v1_26_20, v1_26_60, all 0xfffc) ---------

/// The cereal slices are registered with `StrictErrorCodes`; an unknown member of the
/// document is an error (`0x4`) and the block is not loaded.
#[test]
fn an_unknown_member_of_a_cereal_block_rejects_it() {
    let text = document(
        "1.26.20",
        r#"{"description": {"identifier": "demo:x", "colour": 1}, "components": {}}"#,
    );
    let d = rejected(&text, &context());
    let found = at(&d, unrecognized_field, "description/colour");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
}

/// An unknown component (`0x400`) of a cereal block rejects it too, in a permutation
/// as in the base.
#[test]
fn an_unknown_component_of_a_cereal_block_rejects_it() {
    let base = document(
        "1.26.20",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:no_such": {}}}"#,
    );
    let d = rejected(&base, &context());
    let found = at(&d, unrecognized_component, "components/minecraft:no_such");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
}

/// The legacy slice is the hand-written parser's, with the crate's reading: an unknown member
/// below 1.26.20 stays a warning and the block loads.
#[test]
fn an_unknown_member_of_a_legacy_block_stays_a_warning() {
    let text = document(
        "1.21.0",
        r#"{"description": {"identifier": "demo:x", "colour": 1}, "components": {}}"#,
    );
    let block = loads(&text, &context());
    let d = block.diagnostics();
    assert!(
        d.iter()
            .any(|f| unrecognized_field(&f.code) && f.severity == Severity::Warning),
        "{d:#?}"
    );
}

/// Rule N2 on every read of the cereal document, once: a `null` member of a component is a
/// type error at the member, and the block is not loaded; a `null` description member is told
/// once (the slice's rule, no second hand-wired finding).
#[test]
fn a_null_member_of_a_cereal_block_is_one_type_error() {
    let component = document(
        "1.26.20",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:tick": {"interval_range": [1, 2], "looping": null}}}"#,
    );
    let d = rejected(&component, &context());
    let found = at(&d, invalid_type, "components/minecraft:tick/looping");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(found[0].severity, Severity::Error);
    let description = document(
        "1.26.20",
        r#"{"description": {"identifier": "minecraft:x", "internal_vanilla_data": {"requires_correct_tool_for_drops": null}}, "components": {}}"#,
    );
    let d = rejected(&description, &base_game());
    let found = at(
        &d,
        invalid_type,
        "description/internal_vanilla_data/requires_correct_tool_for_drops",
    );
    assert_eq!(found.len(), 1, "{d:#?}");
}

/// Below 1.26.20 the legacy reader keeps its own reading of `null`: a
/// `null` optional member of a component is read as absent, as the first run read it.
#[test]
fn a_null_member_of_a_legacy_block_is_the_legacy_reader_s() {
    let text = document(
        "1.21.0",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:tick": {"interval_range": [1, 2], "looping": null}}}"#,
    );
    let block = loads(&text, &context());
    let d = block.diagnostics();
    assert!(!d.iter().any(|f| invalid_type(&f.code)), "{d:#?}");
}

/// The members of ours are no members of the cereal document: `deprecated` and
/// `deprecated_event_handlers` are unknown (`0x4`), `ordered_v1_custom_components` is bound
/// and deprecated by name (`0x200`); each rejects a cereal block, in a permutation as at the
/// root.
#[test]
fn the_members_of_ours_reject_a_cereal_block() {
    let deprecated_member =
        // By legacy name: the variant and the definitions that were it.
        |c: &Code| matches!(c.id(), "core/member_deprecated");
    for (member, code, path) in [
        (
            r#""deprecated": {}"#,
            unrecognized_field as fn(&Code) -> bool,
            "deprecated",
        ),
        (
            r#""deprecated_event_handlers": {}"#,
            unrecognized_field,
            "deprecated_event_handlers",
        ),
        (
            r#""ordered_v1_custom_components": []"#,
            deprecated_member,
            "ordered_v1_custom_components",
        ),
    ] {
        let root = document(
            "1.26.20",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{}}, {member}}}"#
            ),
        );
        let d = rejected(&root, &context());
        let found = at(&d, code, path);
        assert_eq!(found.len(), 1, "{member}: {d:#?}");
        assert_eq!(found[0].severity, Severity::Error, "{member}");
        if path == "deprecated_event_handlers" {
            continue;
        }
        let permutation = document(
            "1.26.20",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{}}, "permutations": [{{"condition": "1", "components": {{}}, {member}}}]}}"#
            ),
        );
        let d = rejected(&permutation, &context());
        let found = at(&d, code, &format!("permutations/0/{path}"));
        assert_eq!(found.len(), 1, "{member}: {d:#?}");
    }
    let _ = (ucf, gtest);
}

/// The writer never puts an unknown member into a cereal document, which the strict slice
/// refuses: a legacy block that holds one is written at its own version, where the
/// legacy reader warns of it; a block of the newest format that holds one has no text.
#[test]
fn an_unknown_member_is_never_written_into_a_cereal_document() {
    use bedrock_addon::families::block::Block;
    use bedrock_addon::id::BlockId;
    use bedrock_addon::json::{EncodeError, Value};
    let legacy = loads(
        &document(
            "1.26.0",
            r#"{"description": {"identifier": "demo:x", "note": 1}, "components": {}}"#,
        ),
        &context(),
    );
    let written = legacy.to_json().expect("a text");
    assert!(
        written.contains(r#""format_version": "1.26.0""#),
        "{written}"
    );
    let again = loads(&written, &context());
    assert_eq!(again.value(), legacy.value());
    assert_eq!(again.unknown(), legacy.unknown());
    let mut newest = legacy.map(|_| Block::new(BlockId::parse("demo:x".to_owned())));
    *newest.unknown_mut() = bedrock_addon::document::Unknown::new();
    newest.unknown_mut().put(
        &bedrock_addon::diagnostic::JsonPath::of_keys(["minecraft:block", "description"]),
        "note",
        Value::from(1),
    );
    match newest.to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => {
            assert_eq!(at.to_string(), "minecraft:block/description/note");
        }
        other => panic!("{other:?}"),
    }
}

// ---- the 1.26.20 → 1.26.60 step; the 1.26.60 document is the newest ----------------

use bedrock_addon::families::block::component::{CollisionBoundingBox, CollisionBox, vec3};

/// A fmt template of the game's tests filled as `fmt::format` fills it: `{}` with `version`, each
/// `{:.2f}` with the next of `numbers` at two decimals, `{{` / `}}` as braces.
fn render(template: &str, version: &str, numbers: &[f32]) -> String {
    let mut out = template.replace("{{", "\u{1}").replace("}}", "\u{2}");
    out = out.replacen("{}", version, 1);
    for number in numbers {
        out = out.replacen("{:.2f}", &format!("{number:.2}"), 1);
    }
    out.replace('\u{1}', "{").replace('\u{2}', "}")
}

/// The collision box of the base components of a loaded block.
fn collision(block: &bedrock_addon::families::block::Block) -> CollisionBox {
    block
        .components
        .get::<CollisionBox>()
        .cloned()
        .expect("a collision box")
}

/// A box by its corners.
fn min_max(min: [f32; 3], max: [f32; 3]) -> CollisionBoundingBox {
    CollisionBoundingBox {
        min: Some(vec3(min[0], min[1], min[2])),
        max: Some(vec3(max[0], max[1], max[2])),
        ..CollisionBoundingBox::default()
    }
}

/// The game's `BlockWithLegacyOriginSizeObject` load test: the 1.26.20 box `(0,0,0)+(8,12,8)` loads as an array of one
/// box with `min` (0, 0, 8) and `max` (8, 12, 16).
#[test]
fn the_gtest_box_is_upgraded_to_a_min_max_array() {
    let block = loads(&gtest("loader-8"), &base_game());
    assert_eq!(
        collision(&block),
        CollisionBox::Boxes(vec![min_max([0.0, 0.0, 8.0], [8.0, 12.0, 16.0])])
    );
    assert!(block.version().upgraded);
}

/// Every `vanilla_1.26.50` block runs the step: `shelf_mushroom_block` in the base
/// and in its permutations; `red_shrub`'s `false` passes through.
#[test]
fn the_vanilla_1_26_50_blocks_are_upgraded() {
    let shelf = loads(
        &fixture("vanilla_1_26_50/shelf_mushroom_block.json"),
        &base_game(),
    );
    assert_eq!(
        collision(&shelf),
        CollisionBox::Boxes(vec![min_max([3.0, 4.0, 9.0], [13.0, 11.0, 16.0])])
    );
    let boxed: Vec<_> = shelf
        .permutations
        .iter()
        .filter_map(|p| p.components.get::<CollisionBox>())
        .collect();
    assert_eq!(boxed.len(), 4, "the four permutation boxes");
    for b in boxed {
        assert_eq!(
            b,
            &CollisionBox::Boxes(vec![min_max([1.0, 4.0, 6.0], [15.0, 11.0, 16.0])])
        );
    }
    let shrub = loads(
        &fixture("vanilla_1_26_50/red_shrub.block.json"),
        &base_game(),
    );
    assert_eq!(collision(&shrub), CollisionBox::Enabled(false));
}

/// The chain: `[1.26.20, 1.26.60)` is the 1.26.20 struct, `[1.26.60, engine]` the newest.
#[test]
fn the_newest_struct_is_1_26_60() {
    use bedrock_addon::document::JsonDocument;
    use bedrock_addon::families::block::history::BlockFamily;
    use bedrock_addon::version::FormatVersionInput;
    use bedrock_addon::versioning::{Selection, select};
    let selects =
        |v: &str| select::<BlockFamily>(&FormatVersionInput::parse(v).expect("a version"));
    assert_eq!(selects("1.26.20"), Selection::Struct(14));
    assert_eq!(selects("1.26.59"), Selection::Struct(14));
    assert_eq!(selects("1.26.60"), Selection::Struct(15));
    assert_eq!(selects("1.26.61"), Selection::Unsupported);
    let text = gtest("loader-8");
    let written = bedrock_addon::families::block::Block::parse_as_written(
        &bedrock_addon::storage::PackPath::from_static("blocks/x.json"),
        text.as_bytes(),
        &base_game(),
    )
    .expect("parses");
    assert!(matches!(
        written.value(),
        bedrock_addon::families::block::history::BlockHistory::V1_26_20(_)
    ));
    // As written: the one object, with `origin` and `size`.
    let again = written.to_json().expect("its own shape");
    assert!(
        again.contains(r#""origin""#) && !again.contains(r#""min""#),
        "{again}"
    );
}

/// A 1.26.60 document: `min`/`max` boxes load (with Upcoming Creator Features); the single
/// object of 1.26.20 is no alternative of the 1.26.60 component.
#[test]
fn a_1_26_60_collision_box_is_a_bool_or_an_array() {
    let boxes = document(
        "1.26.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:collision_box": [{"min": [0, 0, 0], "max": [16, 8, 16]}, {"origin": [-8, 8, -8], "size": [16, 8, 16]}]}}"#,
    );
    let block = loads(&boxes, &ucf());
    assert_eq!(
        collision(&block),
        CollisionBox::Boxes(vec![
            min_max([0.0, 0.0, 0.0], [16.0, 8.0, 16.0]),
            CollisionBoundingBox {
                origin: Some(vec3(-8.0, 8.0, -8.0)),
                size: Some(vec3(16.0, 8.0, 16.0)),
                ..CollisionBoundingBox::default()
            },
        ])
    );
    let single = document(
        "1.26.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:collision_box": {"origin": [-8, 0, -8], "size": [16, 8, 16]}}}"#,
    );
    let d = rejected(&single, &ucf());
    assert_eq!(
        at(&d, invalid_type, "components/minecraft:collision_box").len(),
        1,
        "{d:#?}"
    );
    let flag = document(
        "1.26.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:collision_box": true}}"#,
    );
    assert_eq!(
        collision(&loads(&flag, &context())),
        CollisionBox::Enabled(true)
    );
}

/// The 1.26.60 constraints: `origin`/`size` with `min`/`max`; the area of the
/// boxes; a box whose bounds are no box; at most 16 boxes; the member ranges.
#[test]
fn the_1_26_60_box_constraints_reject_the_block() {
    let one = |boxes: &str| {
        document(
            "1.26.60",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:collision_box": {boxes}}}}}"#
            ),
        )
    };
    let text_of = |d: &Diagnostics| -> Vec<String> {
        d.iter()
            .filter(|f| f.severity == Severity::Error)
            .map(|f| f.code.to_string())
            .collect()
    };
    let d = rejected(
        &one(r#"[{"origin": [-8, 0, -8], "max": [16, 16, 16]}]"#),
        &ucf(),
    );
    assert!(
        text_of(&d)
            .iter()
            .any(|t| t == r#""origin"/"size" cannot be combined with "min"/"max""#),
        "{d:#?}"
    );
    let d = rejected(
        &one(r#"[{"origin": [8, 0, -8], "size": [16, 16, 16]}]"#),
        &ucf(),
    );
    assert!(
        text_of(&d)
            .iter()
            .any(|t| t == "Invalid box boundsThe area covered by all boxes must be within (0, 0, 0) to (16, 24, 16)"),
        "{d:#?}"
    );
    // A `min` above its `max` is a box: loads.
    let _ = loads(&one(r#"[{"min": [8, 0, 0], "max": [4, 16, 16]}]"#), &ucf());
    let d = rejected(&one(r#"[{"min": [0, 0, 0], "max": [17, 16, 16]}]"#), &ucf());
    assert!(!text_of(&d).is_empty(), "{d:#?}");
    let seventeen = vec![r#"{"min": [0, 0, 0], "max": [1, 1, 1]}"#; 17].join(", ");
    let d = rejected(&one(&format!("[{seventeen}]")), &ucf());
    assert!(!text_of(&d).is_empty(), "{d:#?}");
}

/// A 1.26.20 box has no `min`/`max` (the 1.26.20 `BoundingBoxData` binds `origin` and `size`
/// alone): an unknown member, which the strict slice makes an error.
#[test]
fn min_and_max_are_unknown_in_a_1_26_20_box() {
    let text = document(
        "1.26.20",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:collision_box": {"min": [0, 0, 0], "max": [16, 16, 16]}}}"#,
    );
    let d = rejected(&text, &context());
    assert_eq!(
        at(
            &d,
            unrecognized_field,
            "components/minecraft:collision_box/min"
        )
        .len(),
        1,
        "{d:#?}"
    );
}

/// Below 1.26.20 the game's collision box description binds origin and size
/// alone, and an unknown member is silent in the lenient mask: `min` is kept as unknown JSON and told
/// as an information of ours.
#[test]
fn min_in_a_legacy_box_is_an_information() {
    let text = document(
        "1.21.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:collision_box": {"origin": [-8, 0, -8], "size": [16, 16, 16], "min": [0, 0, 0]}}}"#,
    );
    let block = loads(&text, &context());
    let found = at(
        block.diagnostics(),
        unrecognized_field,
        "components/minecraft:collision_box/min",
    );
    assert_eq!(found.len(), 1, "{:#?}", block.diagnostics());
    assert_eq!(found[0].severity, Severity::Info);
}

/// `collision_box-1/-2` (fmt templates): the
/// out-of-bounds box at 1.26.20 is not loaded ("Modern collision box constraints should reject
/// out-of-bounds boxes"); at 1.19.10 it loads, clamped; the extended Y and the array
/// load at 1.26.20.
#[test]
fn an_out_of_bounds_1_26_20_box_fails_the_block() {
    let single = gtest("collision_box-1");
    let overflow = [-8.0, 0.0, -8.0, 25.0, 8.0, 25.0];
    let d = rejected(&render(&single, "1.26.20", &overflow), &context());
    assert!(
        d.iter().any(|f| f.severity == Severity::Error
            && f.code.to_string()
                == r#""origin" + "size", must be inside the following limit: (0, 0, 0) and (16, 24, 16)"#),
        "{d:#?}"
    );
    // The legacy path (the game's test world; `collision_box` is released at 1.19.50 behind the
    // legacy creator features in the crate's registry row, so an allow-listed pack's).
    let _ = loads(
        &render(&single, "1.19.10", &overflow),
        &super::common::allow_listed(),
    );
    let extended = [-8.0, 5.0, -8.0, 16.0, 19.0, 16.0];
    let tall = loads(&render(&single, "1.26.20", &extended), &context());
    assert_eq!(
        collision(&tall),
        CollisionBox::Boxes(vec![min_max([0.0, 5.0, 0.0], [16.0, 24.0, 16.0])])
    );
    let array = gtest("collision_box-2");
    let block = loads(
        &render(&array, "1.26.20", &[-8.0, 5.0, -8.0, 16.0, 1.0, 16.0]),
        &context(),
    );
    assert!(matches!(collision(&block), CollisionBox::Boxes(b) if b.len() == 2));
}

/// The other collision fixtures of 1.26.20 load: non-symmetric boxes, boxes in permutations,
/// the piston template with either flag.
#[test]
fn the_collision_gtests_of_1_26_20_load() {
    for name in [
        "collision_box-3",
        "collision_box-4",
        "collision_box-5",
        "collision_box-6",
    ] {
        let _ = loads(&gtest(name), &context());
    }
    let piston = gtest("collision_box-7");
    for flag in ["true", "false"] {
        let _ = loads(&render(&piston, flag, &[]), &context());
    }
}

/// A block of 1.26.20 to 1.26.50 holds its boxes by `min`/`max` after the step; it is written
/// at its own version in the 1.26.20 reading (`origin`/`size`), which reads back as it, never
/// with `min`/`max` (which 1.26.20 does not bind).
#[test]
fn an_upgraded_block_is_written_at_its_own_version_by_origin_and_size() {
    let shelf = loads(
        &fixture("vanilla_1_26_50/shelf_mushroom_block.json"),
        &base_game(),
    );
    let written = shelf.to_json().expect("a text");
    assert!(
        written.contains(r#""format_version": "1.26.50""#),
        "{written}"
    );
    assert!(!written.contains(r#""min""#), "{written}");
    assert_eq!(loads(&written, &base_game()).value(), shelf.value());
    // A 1.26.60 block keeps its own shape.
    let newest = loads(
        &document(
            "1.26.60",
            r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:collision_box": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}"#,
        ),
        &ucf(),
    );
    let written = newest.to_json().expect("a text");
    assert!(written.contains(r#""min""#), "{written}");
    assert_eq!(loads(&written, &ucf()).value(), newest.value());
}

/// `min`/`max` in a document of 1.26.60 or later need Upcoming Creator Features
/// `origin`/`size` do not.
#[test]
fn min_and_max_need_upcoming_creator_features_from_1_26_60() {
    let min_max_box = document(
        "1.26.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:collision_box": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}"#,
    );
    let d = rejected(&min_max_box, &context());
    assert!(
        d.iter().any(|f| f.severity == Severity::Error
            && f.code.to_string()
                == r#"Upcoming Creator Features experiment is required to use "min"/"max" in "minecraft:collision_box" from version 1.26.60 and after"#),
        "{d:#?}"
    );
    let origin_size = document(
        "1.26.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:collision_box": [{"origin": [-8, 0, -8], "size": [16, 8, 16]}]}}"#,
    );
    let _ = loads(&origin_size, &context());
    // An upgraded box is no box written by `min`/`max` in a 1.26.60 file.
    let _ = loads(&gtest("loader-8"), &base_game());
}

// ---- enchantment_power_transmitter, vibration_properties, neighbor_change ----------

use bedrock_addon::families::block::component::{
    EnchantmentPowerTransmitter, NeighborChange, NeighborChangeDirection, VibrationProperties,
};

/// The texts of the errors of `d`.
fn error_texts(d: &Diagnostics) -> Vec<String> {
    d.iter()
        .filter(|f| f.severity == Severity::Error)
        .map(|f| f.code.to_string())
        .collect()
}

/// `power_transmitter-1..3`: with the experiment all
/// three load; `true` and `false` are the component's value; without the experiment the
/// component, whatever its value, needs Upcoming Creator Features.
#[test]
fn enchantment_power_transmitter() {
    for (n, value) in [(1, Some(true)), (2, Some(false)), (3, None)] {
        let text = gtest(&format!("power_transmitter-{n}"));
        let block = loads(&text, &ucf());
        assert_eq!(
            block
                .components
                .get::<EnchantmentPowerTransmitter>()
                .map(|c| c.0),
            value,
            "{n}"
        );
        if value.is_some() {
            let d = rejected(&text, &context());
            assert!(
                error_texts(&d).iter().any(|t| t
                    == "To use 'minecraft:enchantment_power_transmitter', experimental creator features are required"),
                "{n}: {d:#?}"
            );
        }
    }
}

/// `vibration-1/-2`: both members, each `false` when
/// absent; the component needs Upcoming Creator Features (`$_2`).
#[test]
fn vibration_properties() {
    let text = gtest("vibration-1");
    let block = loads(&text, &ucf());
    let v = block
        .components
        .get::<VibrationProperties>()
        .expect("the component");
    assert!(v.dampen_vibrations() && v.occlude_vibrations());
    assert!(!VibrationProperties::default().dampen_vibrations());
    assert!(!VibrationProperties::default().occlude_vibrations());
    let d = rejected(&text, &context());
    assert!(
        error_texts(&d).iter().any(|t| t
            == "To use 'minecraft:vibration_properties', experimental creator features are required"),
        "{d:#?}"
    );
    let _ = loads(&gtest("vibration-2"), &context());
}

/// `neighbor_changed-1..6`: the directions, allowed in
/// permutations; `{all}` when absent; `all` must be alone; 1 to 6 entries; the
/// component needs Upcoming Creator Features (`$_3`).
#[test]
fn neighbor_change() {
    for n in 1..=6 {
        let _ = loads(&gtest(&format!("neighbor_changed-{n}")), &ucf());
    }
    let down = loads(&gtest("neighbor_changed-2"), &ucf());
    assert_eq!(
        down.components
            .get::<NeighborChange>()
            .expect("the component")
            .neighbor_directions(),
        [NeighborChangeDirection::Down]
    );
    assert_eq!(
        NeighborChange::default().neighbor_directions(),
        [NeighborChangeDirection::All]
    );
    let d = rejected(&gtest("neighbor_changed-2"), &context());
    assert!(
        error_texts(&d).iter().any(|t| t
            == "To use 'minecraft:neighbor_change', experimental creator features are required"),
        "{d:#?}"
    );
    let with = |directions: &str| {
        document(
            "1.26.60",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:neighbor_change": {{"neighbor_directions": {directions}}}}}}}"#
            ),
        )
    };
    let d = rejected(&with(r#"["all", "up"]"#), &ucf());
    assert!(
        error_texts(&d)
            .iter()
            .any(|t| t == "The 'all' neighbor direction must be used alone"),
        "{d:#?}"
    );
    let d = rejected(&with("[]"), &ucf());
    assert!(!error_texts(&d).is_empty(), "{d:#?}");
    let _ = loads(
        &with(r#"["up", "down", "north", "south", "west", "east"]"#),
        &ucf(),
    );
}

/// No legacy description exists for `enchantment_power_transmitter` or `neighbor_change`
/// a document below 1.26.20 does not read them (an unknown component, a
/// warning of the legacy reader); `vibration_properties` has a legacy row released at 1.26.60
/// behind Upcoming Creator Features.
#[test]
fn the_legacy_reader_and_the_new_components() {
    let legacy = |components: &str| {
        document(
            "1.21.0",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}"#
            ),
        )
    };
    // The game's schema for the legacy document has no child for them: it logs
    // "child '%s' not valid here." at level 3, validation fails and the block is refused.
    for name in [
        "minecraft:enchantment_power_transmitter\": true",
        "minecraft:neighbor_change\": {}",
    ] {
        let d = rejected(&legacy(&format!("\"{name}")), &ucf());
        let key = name.split('"').next().unwrap();
        assert!(
            d.iter().any(|f| unrecognized_component(&f.code)
                && f.severity == Severity::Error
                && f.code.to_string() == format!("child '{key}' not valid here.")
                && f.location.json_path.to_string().ends_with(key)),
            "{name}: {d:#?}"
        );
    }
    let vibration = legacy(r#""minecraft:vibration_properties": {}"#);
    let _ = rejected(&vibration, &context());
    assert!(
        loads(&vibration, &ucf())
            .components
            .get::<VibrationProperties>()
            .is_some()
    );
}

// ---- the gates that 1.26.60 does not have ------------------------------------------------

/// The UCF validator of 1.26.60 does not check `block_entity` as a whole, `sound` or
/// `instrument_sound` ("Gone"; the two instrument texts are no strings of the
/// binary): a cereal block with them loads without the toggle.
#[test]
fn block_entity_sound_and_instrument_sound_need_no_toggle_from_1_26_20() {
    for components in [
        r#""minecraft:block_entity": {"dynamic_properties": true}"#,
        r#""minecraft:sound": "stone""#,
        r#""minecraft:instrument_sound": {"up": "note.bit"}"#,
    ] {
        for version in ["1.26.20", "1.26.60"] {
            let text = document(
                version,
                &format!(
                    r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}"#
                ),
            );
            let _ = loads(&text, &context());
        }
    }
}

/// Below 1.26.20 the legacy registry decides: `sound` is released at 1.26.40 behind Upcoming
/// Creator Features; `block_entity` and `instrument_sound` have no legacy
/// description, so the legacy reader does not read them (unknown components, a warning).
#[test]
fn the_legacy_registry_gates_sound_and_reads_no_block_entity() {
    let legacy = |components: &str| {
        document(
            "1.21.0",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}"#
            ),
        )
    };
    let sound = legacy(r#""minecraft:sound": "stone""#);
    let d = rejected(&sound, &context());
    assert!(
        error_texts(&d).iter().any(|t| t
            == "To use component 'minecraft:sound' in json formats earlier than 1.26.40, turn on the 'upcoming_creator_features' toggle in settings"),
        "{d:#?}"
    );
    let _ = loads(&sound, &ucf());
    for name in ["minecraft:block_entity", "minecraft:instrument_sound"] {
        let value = if name.ends_with("entity") {
            "{}"
        } else {
            r#"{"up": "note.bit"}"#
        };
        // No child of the game's schema either: the block is refused.
        let d = rejected(&legacy(&format!(r#""{name}": {value}"#)), &context());
        assert!(
            d.iter().any(|f| unrecognized_component(&f.code)
                && f.severity == Severity::Error
                && f.location.json_path.to_string().ends_with(name)),
            "{name}: {d:#?}"
        );
    }
}

// ---- block_entity's container and entity_storage ------------------------------------

use bedrock_addon::families::block::component::BlockEntity;

/// A 1.26.60 block with `components`.
fn newest(components: &str) -> String {
    document(
        "1.26.60",
        &format!(
            r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{components}}}}}"#
        ),
    )
}

/// `entity_storage-1`: with Upcoming Creator Features the block loads
/// with `entity_storage`; without, it is not registered ("To use '{}' in '{}',
/// experimental creator features are required").
#[test]
fn entity_storage_needs_upcoming_creator_features() {
    let text = gtest("entity_storage-1");
    let block = loads(&text, &ucf());
    let entity = block
        .components
        .get::<BlockEntity>()
        .expect("the component");
    assert!(entity.entity_storage());
    assert!(!BlockEntity::default().entity_storage());
    let d = rejected(&text, &context());
    assert!(
        error_texts(&d).iter().any(|t| t
            == "To use 'entity_storage' in 'minecraft:block_entity', experimental creator features are required"),
        "{d:#?}"
    );
    // `entity_storage` is no member of the 1.26.20 component.
    let older = document(
        "1.26.20",
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:block_entity": {"entity_storage": true}}}"#,
    );
    let d = rejected(&older, &ucf());
    assert_eq!(
        at(
            &d,
            unrecognized_field,
            "components/minecraft:block_entity/entity_storage"
        )
        .len(),
        1,
        "{d:#?}"
    );
}

/// `container {slot_count 1..=54 required, title}` (in the 1.26.20 component already,
/// the game's schema), behind Upcoming Creator Features; not with a `crafting_table`
/// (the game rejects that pairing).
#[test]
fn a_container_and_its_rules() {
    let container = |value: &str| {
        newest(&format!(
            r#""minecraft:block_entity": {{"container": {value}}}"#
        ))
    };
    let block = loads(
        &container(r#"{"slot_count": 27, "title": "demo.chest"}"#),
        &ucf(),
    );
    let entity = block
        .components
        .get::<BlockEntity>()
        .expect("the component");
    let data = entity.container.as_ref().expect("a container");
    assert_eq!(data.slot_count, 27);
    let d = rejected(&container(r#"{"slot_count": 27}"#), &context());
    assert!(
        error_texts(&d).iter().any(|t| t
            == "To use 'container' in 'minecraft:block_entity', experimental creator features are required"),
        "{d:#?}"
    );
    for bad in [
        r#"{"slot_count": 0}"#,
        r#"{"slot_count": 55}"#,
        r#"{"slot_count": 9, "title": ""}"#,
    ] {
        let d = rejected(&container(bad), &ucf());
        assert!(!error_texts(&d).is_empty(), "{bad}: {d:#?}");
    }
    // Without its required `slot_count` the component's typed read fails: the strict reader
    // rejects the block.
    let d = rejected(&container("{}"), &ucf());
    assert!(
        d.iter().any(
            // By legacy name: the variant and the definitions that were it.
            |f| matches!(f.code.id(), "core/missing_required_field")
                && f.severity == Severity::Error
                && f.location
                    .json_path
                    .to_string()
                    .ends_with("minecraft:block_entity/container/slot_count")
        ),
        "{d:#?}"
    );
    let d = rejected(
        &newest(
            r#""minecraft:block_entity": {"container": {"slot_count": 9}}, "minecraft:crafting_table": {"crafting_tags": ["demo"]}"#,
        ),
        &ucf(),
    );
    assert!(
        error_texts(&d).iter().any(|t| t
            == r#"Block cannot have both a "container" in "minecraft:block_entity" and a "minecraft:crafting_table" component."#),
        "{d:#?}"
    );
}

// ---- the multi-block rules of 1.26.60 -----------------------------------------------

/// `multi_block-1..4`: a vertical 1.26.20 block needs no experiment;
/// a horizontal 1.26.40 one does (below `HORIZONTAL_MULTI_BLOCK_RELEASE_VERSION` 1.26.50); a
/// horizontal 1.26.50 one with a `block_entity` loads without it (the multi-block +
/// `block_entity` rule is gone); the snow-loggable vertical block loads.
#[test]
fn the_multi_block_gtests() {
    let _ = loads(&gtest("multi_block-1"), &context());
    let horizontal = gtest("multi_block-2");
    let d = rejected(&horizontal, &context());
    assert!(
        error_texts(&d).iter().any(|t| t
            == "Upcoming Creator Features experiment is required for trait 'minecraft:multi_block' for horizontal directions"),
        "{d:#?}"
    );
    let _ = loads(&horizontal, &ucf());
    let _ = loads(&gtest("multi_block-3"), &context());
    let _ = loads(&gtest("multi_block-4"), &context());
}

/// A 1.26.60 multi-block block with `direction` and the components / permutations given.
fn multi_block(direction: &str, components: &str, permutations: &str) -> String {
    document(
        "1.26.60",
        &format!(
            r#"{{"description": {{"identifier": "demo:x", "traits": {{"minecraft:multi_block": {{"enabled_states": ["minecraft:multi_block_part"], "direction": "{direction}", "parts": 2}}}}}},
            "components": {{"minecraft:movable": {{"movement_type": "popped"}}{components}}},
            "permutations": [{permutations}]}}"#
        ),
    )
}

/// The game's multi-block trait validation: a horizontal `direction` with a base
/// `random_offset` is refused; the permutation constraints refuse
/// `precipitation_interactions` in a permutation when the direction is vertical, with the
/// whole text.
#[test]
fn the_new_multi_block_rules() {
    let offset = r#", "minecraft:random_offset": {"x": {"range": {"min": 0, "max": 0}}}"#;
    let d = rejected(&multi_block("west", offset, ""), &ucf());
    assert!(
        error_texts(&d).iter().any(|t| t
            == "Blocks with trait 'minecraft:multi_block' and a horizontal 'direction' cannot define component 'minecraft:random_offset'"),
        "{d:#?}"
    );
    let _ = loads(&multi_block("up", offset, ""), &ucf());
    let snow = r#"{"condition": "1", "components": {"minecraft:precipitation_interactions": {"precipitation_behavior": "obstruct_rain"}}}"#;
    let d = rejected(&multi_block("up", "", snow), &ucf());
    assert!(
        error_texts(&d).iter().any(|t| t
            == "Component 'minecraft:precipitation_interactions' is not valid in multi block permutations, only in the base BlockType components"),
        "{d:#?}"
    );
    let _ = loads(&multi_block("west", "", snow), &ucf());
    let movable = r#"{"condition": "1", "components": {"minecraft:movable": {"movement_type": "immovable"}}}"#;
    let d = rejected(&multi_block("up", "", movable), &ucf());
    assert!(
        error_texts(&d).iter().any(|t| t
            == "Component 'minecraft:movable' is not valid in multi block permutations, only in the base BlockType components"),
        "{d:#?}"
    );
}

// ---- internal_vanilla_data -------------------------------------------------------

use bedrock_addon::families::block::{MaterialType, VanillaBlockData};

/// `loader-10..14` (families numbering): base-game only, the load
/// fails elsewhere; one archetype at most; `requires_correct_tool_for_drops` defaults to
/// `false`; `translucency` in `0..=1`; the constructor's `block_id` −1 and `material` dirt.
#[test]
fn internal_vanilla_data() {
    let d = rejected(&gtest("loader-10"), &context());
    assert!(
        at(&d, unrecognized_field, "description/internal_vanilla_data")
            .iter()
            .any(|f| f.severity == Severity::Error),
        "{d:#?}"
    );
    let one = loads(&gtest("loader-11"), &base_game());
    assert!(one.description.internal_vanilla_data.is_some());
    let truthy = loads(&gtest("loader-12"), &base_game());
    let data = truthy
        .description
        .internal_vanilla_data
        .clone()
        .expect("data");
    assert!(data.requires_correct_tool_for_drops());
    let absent = loads(&gtest("loader-13"), &base_game());
    let data = absent
        .description
        .internal_vanilla_data
        .clone()
        .expect("data");
    assert!(!data.requires_correct_tool_for_drops());
    let _ = rejected(&gtest("loader-14"), &base_game());
    let translucent = document(
        "1.26.20",
        r#"{"description": {"identifier": "minecraft:x", "internal_vanilla_data": {"translucency": 1.5}}, "components": {}}"#,
    );
    let _ = rejected(&translucent, &base_game());
    let defaults = VanillaBlockData::default();
    assert_eq!(defaults.block_id(), -1);
    assert_eq!(defaults.material(), MaterialType::Dirt);
}

/// `minecraft:vertical_connections` is bound for base-game packs only: base-game
/// packs only; elsewhere the state is refused (the consequence is inferred).
#[test]
fn vertical_connections_is_base_game_only() {
    let block = |id: &str| {
        document(
            "1.26.60",
            &format!(
                r#"{{"description": {{"identifier": "{id}", "traits": {{"minecraft:connection": {{"enabled_states": ["minecraft:vertical_connections"]}}}}}}, "components": {{}}}}"#
            ),
        )
    };
    let mut vanilla = base_game();
    vanilla.min_engine_version = Some(bedrock_addon::version::MinEngineVersion::new(1, 26, 60));
    let _ = loads(&block("minecraft:x"), &vanilla);
    let _ = rejected(&block("demo:x"), &context());
}

// ---- a rejected key fails the block ---------------------------------------------------

/// `loader-9` (families numbering): `support` in a permutation, a
/// `rejectKeys` hit of the cereal document, fails the load ("Expected loading to fail when
/// support component is used in permutation"); a base `rejectKeys` hit likewise.
#[test]
fn a_rejected_key_fails_the_block() {
    let d = rejected(&gtest("loader-9"), &base_game());
    assert!(
        error_texts(&d)
            .iter()
            .any(|t| t == "key minecraft:support is not allowed"),
        "{d:#?}"
    );
    let base = newest(r#""minecraft:unit_cube": {}"#);
    let d = rejected(&base, &context());
    assert!(
        error_texts(&d)
            .iter()
            .any(|t| t == "key minecraft:unit_cube is not allowed"),
        "{d:#?}"
    );
}

// ---- the constructor defaults --------------------------------------------------------

use bedrock_addon::families::block::component::{
    AcceptsConnections, ChestObstruction, ConnectionRule, Direction, InstrumentSound,
    ObstructionRule,
};

/// `connection_rule.accepts_connections_from` is `all`,
/// `enabled_directions` all four, `chest_obstruction.obstruction_rule` `shape`;
/// `instrument_sound` holds at most two faces (`maxSize(2)`, implied by its `up`/`down`
/// keys).
#[test]
fn the_defaults_of_1_26_60() {
    let rule = ConnectionRule::default();
    assert_eq!(rule.accepts_connections_from(), AcceptsConnections::All);
    assert_eq!(
        rule.enabled_directions(),
        [
            Direction::South,
            Direction::North,
            Direction::East,
            Direction::West
        ]
    );
    assert_eq!(
        ChestObstruction::default().obstruction_rule(),
        ObstructionRule::Shape
    );
    let three = newest(
        r#""minecraft:instrument_sound": {"up": "note.bit", "down": "note.bell", "left": "note.bit"}"#,
    );
    // A third face is no key of the regex: the constraint fails the block.
    let _ = rejected(&three, &context());
    let _ = InstrumentSound::default();
}

// ---- a Cereal constraint of a component fails a cereal block -------------------------

/// Each rule that the 1.26.20 component binds as a Cereal constraint (0x40, an error in
/// both masks) fails a cereal block, at the member, both sides of each bound:
/// `(components that break it, the path, components at the bound that keep it)`. Where the
/// game's text is the generic constraint template (asserted by a separate test below)
/// no text is asserted; the component's own texts are.
#[test]
fn a_cereal_constraint_fails_the_block_at_the_member() {
    let mi = r#""minecraft:material_instances": {"*": {"texture": "x"}}"#;
    let filters = vec![r#""demo:a""#; 65].join(", ");
    let filters64 = vec![r#""demo:a""#; 64].join(", ");
    let conditions = vec!["{}"; 65].join(", ");
    let conditions64 = vec!["{}"; 64].join(", ");
    let tags = (0..65)
        .map(|i| format!("\"t{i}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let bones = (0..65)
        .map(|i| format!("\"b{i}\": true"))
        .collect::<Vec<_>>()
        .join(", ");
    let instances = (0..65)
        .map(|i| format!("\"i{i}\": {{\"texture\": \"x\"}}"))
        .collect::<Vec<_>>()
        .join(", ");
    let at_most =
        |n: usize, each: &dyn Fn(usize) -> String| (0..n).map(each).collect::<Vec<_>>().join(", ");
    let tags64 = at_most(64, &|i| format!("\"t{i}\""));
    let bones64 = at_most(64, &|i| format!("\"b{i}\": true"));
    let instances64 = at_most(64, &|i| format!("\"i{i}\": {{\"texture\": \"x\"}}"));
    let half = r#""minecraft:collision_box": [{"origin": [0, 0, -8], "size": [8, 16, 16]}], "#;
    let cases: Vec<(String, &str, String)> = vec![
        (r#""minecraft:friction": 0.95"#.into(), "components/minecraft:friction", r#""minecraft:friction": 0.9"#.into()),
        (r#""minecraft:friction": -0.1"#.into(), "components/minecraft:friction", r#""minecraft:friction": 0"#.into()),
        (r#""minecraft:light_dampening": 16"#.into(), "components/minecraft:light_dampening", r#""minecraft:light_dampening": 15"#.into()),
        (r#""minecraft:light_emission": 16"#.into(), "components/minecraft:light_emission", r#""minecraft:light_emission": 15"#.into()),
        (r#""minecraft:redstone_producer": {"power": 16}"#.into(), "components/minecraft:redstone_producer/power", r#""minecraft:redstone_producer": {"power": 15}"#.into()),
        (r#""minecraft:redstone_consumer": {"min_power": 16}"#.into(), "components/minecraft:redstone_consumer/min_power", r#""minecraft:redstone_consumer": {"min_power": 15}"#.into()),
        (r#""minecraft:transformation": {"rotation": [0, 45, 0]}"#.into(), "components/minecraft:transformation/rotation", r#""minecraft:transformation": {"rotation": [0, 90, 0]}"#.into()),
        (r#""minecraft:destructible_by_explosion": {"explosion_resistance": -1}"#.into(), "components/minecraft:destructible_by_explosion/explosion_resistance", r#""minecraft:destructible_by_explosion": {"explosion_resistance": 0}"#.into()),
        (r#""minecraft:destructible_by_mining": {"seconds_to_destroy": -1}"#.into(), "components/minecraft:destructible_by_mining/seconds_to_destroy", r#""minecraft:destructible_by_mining": {"seconds_to_destroy": 0}"#.into()),
        (r#""minecraft:connection_rule": {"enabled_directions": ["north", "north"]}"#.into(), "components/minecraft:connection_rule/enabled_directions", r#""minecraft:connection_rule": {"enabled_directions": ["north", "south"]}"#.into()),
        (r#""minecraft:instrument_sound": {}"#.into(), "components/minecraft:instrument_sound", r#""minecraft:instrument_sound": {"up": "note.bit"}"#.into()),
        (r#""minecraft:instrument_sound": {"left": "note.bit"}"#.into(), "components/minecraft:instrument_sound", r#""minecraft:instrument_sound": {"down": "note.bit"}"#.into()),
        (format!(r#""minecraft:geometry": {{"identifier": "geometry.x", "culling_shape": "minecraft:stairs"}}, {mi}"#), "components/minecraft:geometry/culling_shape", format!(r#""minecraft:geometry": {{"identifier": "geometry.x", "culling_shape": "minecraft:unit_cube"}}, {mi}"#)),
        (format!(r#""minecraft:geometry": {{"identifier": "geometry.x", "culling_layer": "minecraft:culling_layer.glass"}}, {mi}"#), "components/minecraft:geometry/culling_layer", format!(r#""minecraft:geometry": {{"identifier": "geometry.x", "culling_layer": "minecraft:culling_layer.leaves"}}, {mi}"#)),
        (r#""minecraft:geometry": "geometry.x", "minecraft:material_instances": {"*": {"texture": "x", "ambient_occlusion": 10.5}}"#.into(), "components/minecraft:material_instances", r#""minecraft:geometry": "geometry.x", "minecraft:material_instances": {"*": {"texture": "x", "ambient_occlusion": 10}}"#.into()),
        (format!(r#""minecraft:crafting_table": {{"crafting_tags": [{tags}]}}"#), "components/minecraft:crafting_table/crafting_tags", format!(r#""minecraft:crafting_table": {{"crafting_tags": [{tags64}]}}"#)),
        (r#""minecraft:liquid_detection": {"detection_rules": [{}, {}]}"#.into(), "components/minecraft:liquid_detection/detection_rules", r#""minecraft:liquid_detection": {"detection_rules": [{}]}"#.into()),
        (r#""minecraft:liquid_detection": {"detection_rules": []}"#.into(), "components/minecraft:liquid_detection/detection_rules", r#""minecraft:liquid_detection": {"detection_rules": [{}]}"#.into()),
        (r#""minecraft:tick": {"interval_range": [10, 5]}"#.into(), "components/minecraft:tick/interval_range", r#""minecraft:tick": {"interval_range": [5, 5]}"#.into()),
        (r#""minecraft:selection_box": {"origin": [-8, 0, -8], "size": [16, 17, 16]}"#.into(), "components/minecraft:selection_box", r#""minecraft:selection_box": {"origin": [-8, 0, -8], "size": [16, 16, 16]}"#.into()),
        (format!(r#""minecraft:geometry": {{"identifier": "geometry.x", "bone_visibility": {{{bones}}}}}, {mi}"#), "components/minecraft:geometry/bone_visibility", format!(r#""minecraft:geometry": {{"identifier": "geometry.x", "bone_visibility": {{{bones64}}}}}, {mi}"#)),
        (format!(r#""minecraft:geometry": {{"identifier": "minecraft:geometry.full_block", "bone_visibility": {{"b": true}}}}, {mi}"#), "components/minecraft:geometry/bone_visibility", format!(r#""minecraft:geometry": {{"identifier": "minecraft:geometry.full_block"}}, {mi}"#)),
        (format!(r#""minecraft:geometry": "geometry.x", "minecraft:material_instances": {{{instances}}}"#), "components/minecraft:material_instances", format!(r#""minecraft:geometry": "geometry.x", "minecraft:material_instances": {{{instances64}}}"#)),
        (r#""minecraft:geometry": "geometry.x", "minecraft:material_instances": {"bad key": {"texture": "x"}}"#.into(), "components/minecraft:material_instances", format!(r#""minecraft:geometry": "geometry.x", {mi}"#)),
        (format!(r#""minecraft:placement_filter": {{"conditions": [{conditions}]}}"#), "components/minecraft:placement_filter", format!(r#""minecraft:placement_filter": {{"conditions": [{conditions64}]}}"#)),
        (format!(r#""minecraft:placement_filter": {{"conditions": [{{"block_filter": [{filters}]}}]}}"#), "components/minecraft:placement_filter", format!(r#""minecraft:placement_filter": {{"conditions": [{{"block_filter": [{filters64}]}}]}}"#)),
        (format!(r#"{half}"minecraft:random_offset": {{"x": {{"range": {{"min": -9, "max": 0}}}}}}"#), "components/minecraft:random_offset/x/range/min", format!(r#"{half}"minecraft:random_offset": {{"x": {{"range": {{"min": -8, "max": 0}}}}}}"#)),
        (r#""minecraft:geometry": "geometry.x", "minecraft:material_instances": {"*": {"texture": "x", "alpha_masked_tint": true}}"#.into(), "components/minecraft:material_instances/*/alpha_masked_tint", r#""minecraft:geometry": "geometry.x", "minecraft:material_instances": {"*": {"texture": "x", "alpha_masked_tint": true, "tint_method": "grass"}}"#.into()),
    ];
    for (bad, path, good) in cases {
        let d = rejected(&newest(&bad), &ucf());
        assert!(
            d.iter()
                // By legacy name: the variant and the definitions that were it.
                .any(|f| f.code.kind() == Kind::InvalidValue
                    && f.severity == Severity::Error
                    && f.location.json_path.to_string() == format!("minecraft:block/{path}")),
            "{bad}: {d:#?}"
        );
        let _ = loads(&newest(&good), &ucf());
    }
    // A legacy document's friction is bound by the same range: the block is refused there as
    // well. A rule only 1.26.20 binds is no error in a legacy document.
    let legacy = |component: &str| {
        document(
            "1.21.0",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{{component}}}}}"#
            ),
        )
    };
    let _ = rejected(&legacy(r#""minecraft:friction": 0.95"#), &ucf());
    let block = loads(&legacy(r#""minecraft:light_dampening": 16"#), &ucf());
    assert!(
        block
            .diagnostics()
            .iter()
            .all(|f| f.severity < Severity::Error)
    );
    // `minecraft:tags` has no Cereal constraint : dropped with a
    // warning.
    let block = loads(&newest(r#""minecraft:tags": []"#), &ucf());
    assert!(block.components.is_empty());
}

// ---- the block family ---------------------------------------------------------------

/// The findings of `d` with `code`, severity error, at `path` below `minecraft:block`.
fn errors_at<'d>(
    d: &'d Diagnostics,
    code: fn(&Code) -> bool,
    path: &str,
) -> Vec<&'d bedrock_addon::diagnostic::Diagnostic> {
    at(d, code, path)
        .into_iter()
        .filter(|f| f.severity == Severity::Error)
        .collect()
}

fn invalid_value(c: &Code) -> bool {
    // By legacy name: the variant and the definitions that were it.
    c.kind() == Kind::InvalidValue
}

fn missing_toggles(c: &Code) -> bool {
    // By legacy name: the variant and the definitions that were it.
    c.kind() == Kind::Toggle
}

fn deprecated(c: &Code) -> bool {
    // By legacy name: the variant and the definitions that were it.
    c.kind() == Kind::Deprecated
}

/// The game's box type sorts each axis, so a box whose `min` is
/// above its `max` loads, and is written back as written; the one area message is
/// "Invalid box bounds" inserted before the area text.
#[test]
fn a_min_above_max_is_a_box_and_the_area_text_has_the_prefix() {
    let text = newest(r#""minecraft:collision_box": [{"min": [8, 0, 0], "max": [4, 16, 16]}]"#);
    let block = loads(&text, &ucf());
    let written = block.to_json().expect("a text");
    assert!(
        written.contains("8.0") && written.contains("4.0"),
        "{written}"
    );
    assert_eq!(loads(&written, &ucf()).value(), block.value());
    let d = rejected(
        &newest(r#""minecraft:collision_box": [{"origin": [8, 0, -8], "size": [16, 16, 16]}]"#),
        &ucf(),
    );
    let found = errors_at(&d, invalid_value, "components/minecraft:collision_box");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(
        found[0].code.to_string(),
        "Invalid box boundsThe area covered by all boxes must be within (0, 0, 0) to (16, 24, 16)"
    );
    // A combined box is left out of the union and the union is still checked.
    let d = rejected(
        &newest(
            r#""minecraft:collision_box": [{"origin": [-8, 0, -8], "max": [16, 16, 16]}, {"origin": [8, 0, -8], "size": [16, 16, 16]}]"#,
        ),
        &ucf(),
    );
    assert_eq!(
        errors_at(&d, invalid_value, "components/minecraft:collision_box/0").len(),
        1,
        "{d:#?}"
    );
    assert_eq!(
        errors_at(&d, invalid_value, "components/minecraft:collision_box").len(),
        1,
        "{d:#?}"
    );
}

/// The 1.26.60 box constraints at their paths, and the in-bound side.
#[test]
fn the_1_26_60_box_constraints_at_their_paths_and_bounds() {
    let boxes = |list: &str| newest(&format!(r#""minecraft:collision_box": {list}"#));
    let d = rejected(
        &boxes(r#"[{"origin": [-8, 0, -8], "max": [16, 16, 16]}]"#),
        &ucf(),
    );
    let found = errors_at(&d, invalid_value, "components/minecraft:collision_box/0");
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(
        found[0].code.to_string(),
        r#""origin"/"size" cannot be combined with "min"/"max""#
    );
    for (list, path) in [
        (
            r#"[{"min": [0, 0, 0], "max": [17, 16, 16]}]"#,
            "components/minecraft:collision_box/0/max",
        ),
        (
            r#"[{"min": [-1, 0, 0], "max": [16, 16, 16]}]"#,
            "components/minecraft:collision_box/0/min",
        ),
        (
            r#"[{"min": [0, 0, 0], "max": [16, 25, 16]}]"#,
            "components/minecraft:collision_box/0/max",
        ),
        (
            r#"[{"origin": [-9, 0, -8], "size": [16, 16, 16]}]"#,
            "components/minecraft:collision_box/0/origin",
        ),
        (
            r#"[{"origin": [-8, 0, -8], "size": [17, 16, 16]}]"#,
            "components/minecraft:collision_box/0/size",
        ),
    ] {
        let d = rejected(&boxes(list), &ucf());
        assert!(
            !errors_at(&d, invalid_value, path).is_empty(),
            "{list}: {d:#?}"
        );
    }
    let seventeen = vec![r#"{"min": [0, 0, 0], "max": [1, 1, 1]}"#; 17].join(", ");
    let d = rejected(&boxes(&format!("[{seventeen}]")), &ucf());
    assert_eq!(
        errors_at(&d, invalid_value, "components/minecraft:collision_box").len(),
        1,
        "{d:#?}"
    );
    let sixteen = vec![r#"{"min": [0, 0, 0], "max": [1, 1, 1]}"#; 16].join(", ");
    for list in [
        format!("[{sixteen}]"),
        r#"[{"min": [0, 0, 0], "max": [16, 24, 16]}]"#.to_owned(),
        r#"[{"origin": [-8, 0, -8], "size": [16, 24, 16]}]"#.to_owned(),
        r#"[{"origin": [8, 24, 8], "size": [-16, -24, -16]}]"#.to_owned(),
    ] {
        let _ = loads(&boxes(&list), &ucf());
    }
}

/// The multi-block trait is released at 1.26.40 for the legacy path;
/// a cereal document needs no toggle.
#[test]
fn the_multi_block_trait_gate_says_1_26_40_and_stops_at_1_26_20() {
    let trait_block = |version: &str| {
        document(
            version,
            r#"{"description": {"identifier": "demo:x", "traits": {"minecraft:multi_block": {"enabled_states": ["minecraft:multi_block_part"], "direction": "up"}}}, "components": {"minecraft:movable": {"movement_type": "popped"}}}"#,
        )
    };
    let d = rejected(&trait_block("1.26.19"), &context());
    let found = errors_at(
        &d,
        missing_toggles,
        "description/traits/minecraft:multi_block",
    );
    assert_eq!(found.len(), 1, "{d:#?}");
    assert_eq!(
        found[0].code.to_string(),
        "To use trait 'minecraft:multi_block' in json formats earlier than 1.26.40, turn on the 'upcoming_creator_features' toggle in settings"
    );
    let _ = loads(&trait_block("1.26.20"), &context());
}

/// The 1.26.19 / 1.26.20 bound of the cereal rules, and 1.26.49 / 1.26.50 of the
/// horizontal gate.
#[test]
fn the_bounds_of_the_cereal_rules() {
    let with = |version: &str, payload: &str| document(version, payload);
    // The strict mask.
    let colour = r#"{"description": {"identifier": "demo:x", "colour": 1}, "components": {}}"#;
    let block = loads(&with("1.26.19", colour), &context());
    assert!(
        block
            .diagnostics()
            .iter()
            .any(|f| unrecognized_field(&f.code) && f.severity == Severity::Warning)
    );
    let d = rejected(&with("1.26.20", colour), &context());
    assert_eq!(
        errors_at(&d, unrecognized_field, "description/colour").len(),
        1
    );
    // Rule N2.
    let null = r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:tick": {"interval_range": [1, 2], "looping": null}}}"#;
    let block = loads(&with("1.26.19", null), &context());
    assert!(!block.diagnostics().iter().any(|f| invalid_type(&f.code)));
    let d = rejected(&with("1.26.20", null), &context());
    assert_eq!(
        errors_at(&d, invalid_type, "components/minecraft:tick/looping").len(),
        1
    );
    // The `sound` gate: the legacy row below 1.26.20, none from it.
    let sound =
        r#"{"description": {"identifier": "demo:x"}, "components": {"minecraft:sound": "stone"}}"#;
    let d = rejected(&with("1.26.19", sound), &context());
    assert_eq!(
        errors_at(&d, missing_toggles, "components/minecraft:sound").len(),
        1,
        "{d:#?}"
    );
    let _ = loads(&with("1.26.20", sound), &context());
    // The members of ours.
    let ordered = r#"{"description": {"identifier": "demo:x"}, "components": {}, "ordered_v1_custom_components": []}"#;
    let _ = loads(&with("1.26.19", ordered), &context());
    let d = rejected(&with("1.26.20", ordered), &context());
    assert_eq!(
        errors_at(&d, deprecated, "ordered_v1_custom_components").len(),
        1
    );
    let handlers = r#"{"description": {"identifier": "demo:x"}, "components": {}, "deprecated_event_handlers": {}}"#;
    let block = loads(&with("1.26.19", handlers), &context());
    assert!(
        block
            .diagnostics()
            .iter()
            .any(|f| unrecognized_field(&f.code) && f.severity == Severity::Info)
    );
    let d = rejected(&with("1.26.20", handlers), &context());
    assert_eq!(
        errors_at(&d, unrecognized_field, "deprecated_event_handlers").len(),
        1
    );
    // The horizontal gate.
    let horizontal = r#"{"description": {"identifier": "demo:x", "traits": {"minecraft:multi_block": {"enabled_states": ["minecraft:multi_block_part"], "direction": "west"}}}, "components": {"minecraft:movable": {"movement_type": "popped"}}}"#;
    let d = rejected(&with("1.26.49", horizontal), &context());
    assert_eq!(
        errors_at(
            &d,
            missing_toggles,
            "description/traits/minecraft:multi_block/direction"
        )
        .len(),
        1,
        "{d:#?}"
    );
    let _ = loads(&with("1.26.50", horizontal), &context());
}

/// The gates and rules at their paths and codes, the
/// in-bound edges, every rejected key, each UCF gate in a permutation, and the validator
/// stopping at its first failure.
#[test]
fn the_rules_at_their_paths_codes_and_edges() {
    // The UCF gates, in the base and in a permutation.
    for (key, value) in [
        ("minecraft:enchantment_power_transmitter", "true"),
        ("minecraft:vibration_properties", "{}"),
        ("minecraft:neighbor_change", "{}"),
    ] {
        let d = rejected(&newest(&format!(r#""{key}": {value}"#)), &context());
        assert_eq!(
            errors_at(&d, missing_toggles, &format!("components/{key}")).len(),
            1,
            "{key}: {d:#?}"
        );
        let permutation = document(
            "1.26.60",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{}}, "permutations": [{{"condition": "1", "components": {{"{key}": {value}}}}}]}}"#
            ),
        );
        let d = rejected(&permutation, &context());
        assert_eq!(
            errors_at(
                &d,
                missing_toggles,
                &format!("permutations/0/components/{key}")
            )
            .len(),
            1,
            "{key}: {d:#?}"
        );
    }
    let min_max = document(
        "1.26.60",
        r#"{"description": {"identifier": "demo:x"}, "components": {}, "permutations": [{"condition": "1", "components": {"minecraft:collision_box": [{"min": [0, 0, 0], "max": [16, 8, 16]}]}}]}"#,
    );
    let d = rejected(&min_max, &context());
    assert_eq!(
        errors_at(
            &d,
            missing_toggles,
            "permutations/0/components/minecraft:collision_box"
        )
        .len(),
        1,
        "{d:#?}"
    );
    // The validator's first failure alone.
    let two = newest(
        r#""minecraft:enchantment_power_transmitter": true, "minecraft:neighbor_change": {}"#,
    );
    let d = rejected(&two, &context());
    assert_eq!(
        d.iter().filter(|f| missing_toggles(&f.code)).count(),
        1,
        "{d:#?}"
    );
    let horizontal = document(
        "1.26.40",
        r#"{"description": {"identifier": "demo:x", "traits": {"minecraft:multi_block": {"enabled_states": ["minecraft:multi_block_part"], "direction": "west"}}}, "components": {"minecraft:movable": {"movement_type": "popped"}, "minecraft:enchantment_power_transmitter": true}}"#,
    );
    let d = rejected(&horizontal, &context());
    let toggles: Vec<_> = d.iter().filter(|f| missing_toggles(&f.code)).collect();
    assert_eq!(toggles.len(), 1, "{d:#?}");
    assert!(
        toggles[0]
            .location
            .json_path
            .to_string()
            .ends_with("direction")
    );
}

#[test]
fn neighbor_change_directions_are_checked() {
    // neighbor_change.
    for directions in [r#"["all", "up"]"#, "[]"] {
        let d = rejected(
            &newest(&format!(
                r#""minecraft:neighbor_change": {{"neighbor_directions": {directions}}}"#
            )),
            &ucf(),
        );
        assert_eq!(
            errors_at(
                &d,
                invalid_value,
                "components/minecraft:neighbor_change/neighbor_directions"
            )
            .len(),
            1,
            "{directions}"
        );
    }
    let _ = loads(
        &newest(r#""minecraft:neighbor_change": {"neighbor_directions": ["all"]}"#),
        &ucf(),
    );
}

#[test]
fn block_entity_members_are_gated_and_ranged() {
    // block_entity.
    for (member, value) in [
        ("container", r#"{"slot_count": 9}"#),
        ("entity_storage", "true"),
    ] {
        let d = rejected(
            &newest(&format!(
                r#""minecraft:block_entity": {{"{member}": {value}}}"#
            )),
            &context(),
        );
        assert_eq!(
            errors_at(
                &d,
                missing_toggles,
                &format!("components/minecraft:block_entity/{member}")
            )
            .len(),
            1,
            "{member}"
        );
    }
    let container = |value: &str| {
        newest(&format!(
            r#""minecraft:block_entity": {{"container": {value}}}"#
        ))
    };
    for bad in [r#"{"slot_count": 0}"#, r#"{"slot_count": 55}"#] {
        let d = rejected(&container(bad), &ucf());
        assert_eq!(
            errors_at(
                &d,
                invalid_value,
                "components/minecraft:block_entity/container/slot_count"
            )
            .len(),
            1,
            "{bad}"
        );
    }
    let long = "t".repeat(257);
    for bad in [
        r#"{"slot_count": 9, "title": ""}"#.to_owned(),
        format!(r#"{{"slot_count": 9, "title": "{long}"}}"#),
    ] {
        let d = rejected(&container(&bad), &ucf());
        assert_eq!(
            errors_at(
                &d,
                invalid_value,
                "components/minecraft:block_entity/container/title"
            )
            .len(),
            1
        );
    }
    let edge = "t".repeat(256);
    for good in [
        r#"{"slot_count": 1}"#.to_owned(),
        r#"{"slot_count": 54}"#.to_owned(),
        format!(r#"{{"slot_count": 9, "title": "{edge}"}}"#),
    ] {
        let _ = loads(&container(&good), &ucf());
    }
    let d = rejected(
        &newest(
            r#""minecraft:block_entity": {"container": {"slot_count": 9}}, "minecraft:crafting_table": {"crafting_tags": ["demo"]}"#,
        ),
        &ucf(),
    );
    assert_eq!(
        errors_at(&d, invalid_value, "components").len(),
        1,
        "{d:#?}"
    );
}

#[test]
fn multi_block_rejects_random_offset_and_precipitation() {
    // Multi-block.
    let multi = |direction: &str, components: &str, permutations: &str| {
        document(
            "1.26.60",
            &format!(
                r#"{{"description": {{"identifier": "demo:x", "traits": {{"minecraft:multi_block": {{"enabled_states": ["minecraft:multi_block_part"], "direction": "{direction}"}}}}}}, "components": {{"minecraft:movable": {{"movement_type": "popped"}}{components}}}, "permutations": [{permutations}]}}"#
            ),
        )
    };
    let d = rejected(
        &multi(
            "west",
            r#", "minecraft:random_offset": {"x": {"range": {"min": 0, "max": 0}}}"#,
            "",
        ),
        &ucf(),
    );
    assert_eq!(
        errors_at(&d, invalid_value, "components/minecraft:random_offset").len(),
        1,
        "{d:#?}"
    );
    let d = rejected(
        &multi(
            "down",
            "",
            r#"{"condition": "1", "components": {"minecraft:precipitation_interactions": {"precipitation_behavior": "obstruct_rain"}}}"#,
        ),
        &ucf(),
    );
    assert_eq!(
        errors_at(
            &d,
            invalid_value,
            "permutations/0/components/minecraft:precipitation_interactions"
        )
        .len(),
        1,
        "{d:#?}"
    );
}

#[test]
fn internal_vanilla_data_is_one_archetype_and_a_unit_translucency() {
    // internal_vanilla_data.
    let d = rejected(&gtest("loader-14"), &base_game());
    let archetype = errors_at(
        &d,
        invalid_value,
        "description/internal_vanilla_data/block_archetype",
    );
    assert_eq!(archetype.len(), 1);
    // `BlockArchetypeMaxComponentConstraint`'s text.
    assert_eq!(
        archetype[0].code.to_string(),
        "\"block_archetype\" can only have a maximum of one component."
    );
    let translucent = |t: &str| {
        document(
            "1.26.20",
            &format!(
                r#"{{"description": {{"identifier": "minecraft:x", "internal_vanilla_data": {{"translucency": {t}}}}}, "components": {{}}}}"#
            ),
        )
    };
    for bad in ["1.5", "-0.1", "1.000001"] {
        let d = rejected(&translucent(bad), &base_game());
        let found = errors_at(
            &d,
            invalid_value,
            "description/internal_vanilla_data/translucency",
        );
        assert_eq!(found.len(), 1, "{bad}");
        // The range text of the cereal constraint.
        assert_eq!(
            found[0].code.to_string(),
            "the input is outside of the accepted range [0, 1]",
            "{bad}"
        );
    }
    for good in ["1.0", "0.0"] {
        let _ = loads(&translucent(good), &base_game());
    }
}

#[test]
fn the_base_and_permutation_reject_their_keys() {
    const PERMUTATION: [&str; 6] = [
        "minecraft:block_entity",
        "minecraft:embedded_visual",
        "minecraft:flower_pottable",
        "minecraft:item_visual",
        "minecraft:redstone_consumer",
        "minecraft:support",
    ];

    const BASE: [&str; 11] = [
        "minecraft:breathability",
        "minecraft:unit_cube",
        "minecraft:on_fall_on",
        "minecraft:on_interact",
        "minecraft:on_placed",
        "minecraft:on_player_destroyed",
        "minecraft:on_player_placing",
        "minecraft:on_step_off",
        "minecraft:on_step_on",
        "minecraft:queued_ticking",
        "minecraft:random_ticking",
    ];

    // Every rejected key: the base's eleven, the permutation's
    // seventeen.
    let key_text = |key: &str| format!("key {key} is not allowed");
    for key in BASE {
        let d = rejected(&newest(&format!(r#""{key}": {{}}"#)), &ucf());
        let found = errors_at(&d, invalid_value, &format!("components/{key}"));
        assert!(
            found.iter().any(|f| f.code.to_string() == key_text(key)),
            "{key}: {d:#?}"
        );
    }
    for key in BASE.iter().chain(PERMUTATION.iter()) {
        let text = document(
            "1.26.60",
            &format!(
                r#"{{"description": {{"identifier": "demo:x"}}, "components": {{}}, "permutations": [{{"condition": "1", "components": {{"{key}": {{}}}}}}]}}"#
            ),
        );
        let d = rejected(&text, &ucf());
        let found = errors_at(
            &d,
            invalid_value,
            &format!("permutations/0/components/{key}"),
        );
        assert!(
            found.iter().any(|f| f.code.to_string() == key_text(key)),
            "{key}: {d:#?}"
        );
    }
}

/// Fixture gap: `multi_block-5` is no block document but the loot
/// table the multi-block helper writes (`loot_tables/blocks/multiblock_loot.json`): rendered,
/// it is a loot table of one item pool, and the block reader does not take it.
#[test]
fn fixture_multi_block_5_is_the_helpers_loot_table() {
    let rendered = render(&gtest("multi_block-5"), "minecraft:stone", &[]);
    let json: bedrock_addon::json::Value = facet_json::from_str(&rendered).expect("JSON");
    let pools = json
        .as_object()
        .and_then(|o| o.get("pools"))
        .and_then(|p| p.as_array())
        .expect("pools");
    assert_eq!(pools.len(), 1);
    assert!(load(&rendered, &context()).is_err());
}

/// The loader's legacy ↔ modern pairs: the legacy file and its modern twin give the same block, but
/// for the source version and what the legacy steps synthesise; a seventh (an allow-listed pack's
/// deprecated syntax) keeps its two events and its legacy components.
#[test]
fn fixture_the_legacy_and_modern_pairs_give_the_same_block() {
    use bedrock_addon::document::ComponentRef;
    // The legacy files use components released after their version: the game's test world has
    // the experiments on (a base-game pack with Upcoming Creator Features here).
    let mut world = ucf();
    world.is_base_game_pack = true;
    let same = |legacy: &str, modern: &str| {
        let a = loads(&gtest(legacy), &world);
        let b = loads(&gtest(modern), &world);
        assert_eq!(a.description, b.description, "{legacy}");
        assert_eq!(a.permutations.len(), b.permutations.len(), "{legacy}");
        let authored = |map: &bedrock_addon::document::ComponentMap<
            bedrock_addon::families::block::component::BlockComponent,
        >|
         -> Vec<String> {
            map.iter()
                .filter(|e| {
                    !matches!(
                        e,
                        ComponentRef::Typed {
                            synthesized: true,
                            ..
                        }
                    )
                })
                .map(|e| format!("{e:?}"))
                .collect()
        };
        assert_eq!(authored(&a.components), authored(&b.components), "{legacy}");
        for (pa, pb) in a.permutations.iter().zip(&b.permutations) {
            assert_eq!(
                authored(&pa.components),
                authored(&pb.components),
                "{legacy}"
            );
        }
    };
    same("loader-1", "loader-2");
    same("loader-3", "loader-4");
    same("loader-5", "loader-6");
    let mut cx = world.clone();
    cx.pack_uuid = uuid::Uuid::from_u128(0xb4a0_f58d_fccb_4198_86f5_b44a_4fb1_62cd);
    let legacy = loads(&gtest("loader-7"), &cx);
    assert_eq!(legacy.deprecated_event_handlers.len(), 2, "two events");
    assert!(legacy.deprecated.queued_ticking.is_some());
    assert!(legacy.deprecated.random_ticking.is_some());
    assert!(legacy.deprecated.breathability.is_some());
}

// ---- the strict component read of the cereal slices ----------------------------

/// In a cereal block (1.26.20 and later, a
/// `MinecraftDocumentInput` slice) a component whose typed read fails (`0x8`) or whose required
/// member is missing (`0x80`) rejects the block, at the member, with an error; the legacy
/// hand-written slice (below 1.26.20) rejects it as well: the game's lenient component loader
/// fails the group and the block is refused, not dropped with a warning.
#[test]
fn a_failed_component_read_rejects_a_block_under_both_slices() {
    let payload = |components: &str| {
        format!(r#"{{"description": {{"identifier": "demo:x"}}, "components": {components}}}"#)
    };
    for (components, code_name, at) in [
        (
            r#"{"minecraft:flammable": {"catch_chance_modifier": "x"}, "minecraft:friction": 0.3}"#,
            "core/invalid_type",
            "minecraft:block/components/minecraft:flammable/catch_chance_modifier",
        ),
        (
            r#"{"minecraft:movable": {}, "minecraft:friction": 0.3}"#,
            "core/missing_required_field",
            "minecraft:block/components/minecraft:movable/movement_type",
        ),
    ] {
        // The legacy reader loads each component with its cereal description under the lenient
        // mask too, and 0x8 / 0x80 fail it: "Failed to parse block '%s' " at level 3, no
        // definition. The block is refused under both slices.
        for version in ["1.19.40", "1.26.20", "1.26.60"] {
            let d = rejected(&document(version, &payload(components)), &ucf());
            let errors: Vec<(String, String)> = d
                .iter()
                .filter(|f| f.severity == Severity::Error)
                .map(|f| (code_name_of(&f.code), f.location.json_path.to_string()))
                .collect();
            assert_eq!(
                errors,
                [(code_name.to_owned(), at.to_owned())],
                "{version} {components}"
            );
        }
    }
}

fn code_name_of(code: &Code) -> String {
    code.id().to_owned()
}

// The Cereal constraints carry the game's generic texts.
#[test]
fn the_cereal_constraints_carry_the_game_texts() {
    let mi = r#""minecraft:material_instances": {"*": {"texture": "x"}}"#;
    let many =
        |n: usize, each: &dyn Fn(usize) -> String| (0..n).map(each).collect::<Vec<_>>().join(", ");
    let tags = many(65, &|i| format!("\"t{i}\""));
    let bones = many(65, &|i| format!("\"b{i}\": true"));
    let uv_bones = many(65, &|i| format!("\"b{i}\""));
    let instances = many(65, &|i| format!("\"i{i}\": {{\"texture\": \"x\"}}"));
    let conditions = many(65, &|_| "{}".to_owned());
    let filters = many(65, &|_| r#""demo:a""#.to_owned());
    let range = |text: &str| format!("the input is outside of the accepted range {text}");
    let regex = |re: &str| format!("expected string to match the \"{re}\" regex");
    let cases: Vec<(String, &str, String)> = vec![
        // Both bounds.
        (r#""minecraft:friction": 0.95"#.into(), "components/minecraft:friction", range("[0, 0.900000]")),
        (r#""minecraft:geometry": "geometry.x", "minecraft:material_instances": {"*": {"texture": "x", "ambient_occlusion": 10.5}}"#.into(), "components/minecraft:material_instances", range("[0, 10]")),
        // An upper bound.
        (r#""minecraft:light_dampening": 16"#.into(), "components/minecraft:light_dampening", "the input must be a number <= 15".into()),
        (r#""minecraft:light_emission": 16"#.into(), "components/minecraft:light_emission", "the input must be a number <= 15".into()),
        (r#""minecraft:redstone_producer": {"power": 16}"#.into(), "components/minecraft:redstone_producer/power", "the input must be a number <= 15".into()),
        (r#""minecraft:redstone_consumer": {"min_power": 16}"#.into(), "components/minecraft:redstone_consumer/min_power", "the input must be a number <= 15".into()),
        // A lower bound.
        (r#""minecraft:destructible_by_explosion": {"explosion_resistance": -1}"#.into(), "components/minecraft:destructible_by_explosion/explosion_resistance", "the input must be a number >= 0".into()),
        (r#""minecraft:destructible_by_mining": {"seconds_to_destroy": -0.5}"#.into(), "components/minecraft:destructible_by_mining/seconds_to_destroy", "the input must be a number >= 0".into()),
        // A step (`multipleOf`).
        (r#""minecraft:transformation": {"rotation": [0, 45, 0]}"#.into(), "components/minecraft:transformation/rotation", "45 is not multiple of 90".into()),
        // No duplicates.
        (r#""minecraft:connection_rule": {"enabled_directions": ["north", "north"]}"#.into(), "components/minecraft:connection_rule/enabled_directions", "No duplicates allowed".into()),
        // An object and an array below their `minSize`.
        (r#""minecraft:instrument_sound": {}"#.into(), "components/minecraft:instrument_sound", "expected object with at least 1 elements".into()),
        (r#""minecraft:liquid_detection": {"detection_rules": []}"#.into(), "components/minecraft:liquid_detection/detection_rules", "expected array with at least 1 elements".into()),
        // A key or a string outside its regex.
        (r#""minecraft:instrument_sound": {"left": "note.bit"}"#.into(), "components/minecraft:instrument_sound", regex("^(up|down)$")),
        (r#""minecraft:geometry": "geometry.x", "minecraft:material_instances": {"bad key": {"texture": "x"}}"#.into(), "components/minecraft:material_instances", regex("^[a-zA-Z0-9_.:*]+$")),
        (format!(r#""minecraft:geometry": {{"identifier": "geometry.x", "culling_shape": "minecraft:stairs"}}, {mi}"#), "components/minecraft:geometry/culling_shape", regex("^(?:minecraft:empty|minecraft:unit_cube|(?!minecraft:)[a-z0-9].*[a-z0-9])$")),
        (format!(r#""minecraft:geometry": {{"identifier": "geometry.x", "culling_layer": "minecraft:culling_layer.glass"}}, {mi}"#), "components/minecraft:geometry/culling_layer", regex("^(?:minecraft:culling_layer.undefined|minecraft:culling_layer.leaves|(?!minecraft:)[a-z0-9].*[a-z0-9])$")),
        // A list and a map above their `maxSize` (the reader's input cap).
        (format!(r#""minecraft:crafting_table": {{"crafting_tags": [{tags}]}}"#), "components/minecraft:crafting_table/crafting_tags", "too many input elements, expected no more than 64".into()),
        (r#""minecraft:liquid_detection": {"detection_rules": [{}, {}]}"#.into(), "components/minecraft:liquid_detection/detection_rules", "too many input elements, expected no more than 1".into()),
        (format!(r#""minecraft:placement_filter": {{"conditions": [{conditions}]}}"#), "components/minecraft:placement_filter", "too many input elements, expected no more than 64".into()),
        (format!(r#""minecraft:placement_filter": {{"conditions": [{{"block_filter": [{filters}]}}]}}"#), "components/minecraft:placement_filter", "too many input elements, expected no more than 64".into()),
        (format!(r#""minecraft:geometry": {{"identifier": "geometry.x", "uv_lock": [{uv_bones}]}}, {mi}"#), "components/minecraft:geometry/uv_lock", "too many input elements, expected no more than 64".into()),
        (format!(r#""minecraft:geometry": {{"identifier": "geometry.x", "bone_visibility": {{{bones}}}}}, {mi}"#), "components/minecraft:geometry/bone_visibility", "too many input elements for an associative container".into()),
        (format!(r#""minecraft:geometry": "geometry.x", "minecraft:material_instances": {{{instances}}}"#), "components/minecraft:material_instances", "too many input elements for an associative container".into()),
    ];
    for (bad, path, text) in cases {
        let d = rejected(&newest(&bad), &ucf());
        let at = format!("minecraft:block/{path}");
        let found: Vec<_> = d
            .iter()
            .filter(|f| f.location.json_path.to_string() == at && f.severity == Severity::Error)
            .map(|f| (f.code.kind(), f.severity, f.code.to_string()))
            .collect();
        assert_eq!(
            found,
            [(Kind::InvalidValue, Severity::Error, text)],
            "{bad}: {d:#?}"
        );
    }
}

/// A legacy (below 1.26.20) horizontal
/// multi-block block without Upcoming Creator Features is refused by the trait registry's gate
/// alone (the registry releases the trait at 1.26.40):
/// the legacy read fails there, so the UCF validator's horizontal check does not run on it and
/// the block has one diagnostic, not two. From 1.26.20 (no registry gate on the cereal path) the
/// validator's horizontal check is the one.
#[test]
fn a_legacy_horizontal_multi_block_without_ucf_has_one_diagnostic() {
    let block = |version: &str| {
        document(
            version,
            r#"{"description": {"identifier": "demo:door", "traits": {"minecraft:multi_block": {"enabled_states": ["minecraft:multi_block_part"], "direction": "north", "parts": 2}}},
            "components": {"minecraft:movable": {"movement_type": "popped"}}}"#,
        )
    };
    let d = rejected(&block("1.21.0"), &context());
    // (`minecraft:movable` has its own legacy gate; the findings about the trait are counted.)
    let errors: Vec<_> = d
        .iter()
        .filter(|f| {
            f.severity == Severity::Error
                && f.location
                    .json_path
                    .to_string()
                    .contains("minecraft:multi_block")
        })
        .collect();
    assert_eq!(errors.len(), 1, "{d:#?}");
    assert_eq!(errors[0].code.id(), "core/turn_on_toggle_before");
    assert_eq!(
        errors[0].location.json_path.to_string(),
        "minecraft:block/description/traits/minecraft:multi_block"
    );
    let d = rejected(&block("1.26.20"), &context());
    let errors: Vec<_> = d.iter().filter(|f| f.severity == Severity::Error).collect();
    assert_eq!(errors.len(), 1, "{d:#?}");
    assert_eq!(
        errors[0].code.to_string(),
        "Upcoming Creator Features experiment is required for trait 'minecraft:multi_block' for horizontal directions"
    );
}

/// In 1.26.60 the crafting table component gives each crafting tag a string
/// `maxSize(64)` and the list a `maxSize(64)`; the localisation-id string (`maxSize(256)`)
/// is `table_name`'s. `culling_layer` is the cereal regex
/// `^(?:minecraft:culling_layer.undefined|minecraft:culling_layer.leaves|(?!minecraft:)[a-z0-9].*[a-z0-9])$`:
/// a custom or namespace-less layer starts and ends with a lower-case letter or a
/// digit. Each bound on both sides.
#[test]
fn the_crafting_table_and_culling_layer_constraints_as_the_game_reads_them() {
    let mi = r#""minecraft:material_instances": {"*": {"texture": "x"}}"#;
    let name = |n: usize| "a".repeat(n);
    let refused = |components: String, path: &str, text: &str| {
        let d = rejected(&newest(&components), &ucf());
        assert!(
            d.iter()
                // By legacy name: the variant and the definitions that were it.
                .any(|f| f.code.kind() == Kind::InvalidValue
                    && f.severity == Severity::Error
                    && f.location.json_path.to_string()
                        == format!("minecraft:block/components/{path}")
                    && f.code.to_string() == text),
            "{components}: {d:#?}"
        );
    };
    // A tag of 65 characters; 64 loads.
    refused(
        format!(
            r#""minecraft:crafting_table": {{"crafting_tags": ["{}"]}}"#,
            name(65)
        ),
        "minecraft:crafting_table/crafting_tags/0",
        "too many characters, expected no more than 64",
    );
    let _ = loads(
        &newest(&format!(
            r#""minecraft:crafting_table": {{"crafting_tags": ["{}"]}}"#,
            name(64)
        )),
        &ucf(),
    );
    // A table name of 257 characters; 256 loads.
    refused(
        format!(
            r#""minecraft:crafting_table": {{"crafting_tags": ["t"], "table_name": "{}"}}"#,
            name(257)
        ),
        "minecraft:crafting_table/table_name",
        "too many characters, expected no more than 256",
    );
    let _ = loads(
        &newest(&format!(
            r#""minecraft:crafting_table": {{"crafting_tags": ["t"], "table_name": "{}"}}"#,
            name(256)
        )),
        &ucf(),
    );
    // The culling layer's regex: the start and end characters of a custom layer.
    for layer in ["demo:layer.", "Demo:layer", "_demo:layer", "a"] {
        let d = rejected(
            &newest(&format!(
                r#""minecraft:geometry": {{"identifier": "geometry.x", "culling_layer": "{layer}"}}, {mi}"#
            )),
            &ucf(),
        );
        assert!(
            d.iter().any(|f| f.location.json_path.to_string()
                == "minecraft:block/components/minecraft:geometry/culling_layer"),
            "{layer}: {d:#?}"
        );
    }
    for layer in ["ab", "demo:layer_1", "minecraft:culling_layer.leaves"] {
        let _ = loads(
            &newest(&format!(
                r#""minecraft:geometry": {{"identifier": "geometry.x", "culling_layer": "{layer}"}}, {mi}"#
            )),
            &ucf(),
        );
    }
}

/// The game's string constraint works on bytes: it compares the byte length of the string
/// and runs a byte-wise regex search whose ECMAScript `.` matches any byte but `\n` and `\r`. So a crafting tag of 33 `é`
/// (66 bytes) is too long though 33 characters, and the culling layer regex treats `\r` like
/// `\n` and a two-byte character as two `.`s.
#[test]
fn the_string_constraints_count_bytes_and_the_regex_dot_is_a_byte() {
    let mi = r#""minecraft:material_instances": {"*": {"texture": "x"}}"#;
    let tag = |tag: &str| {
        newest(&format!(
            r#""minecraft:crafting_table": {{"crafting_tags": ["{tag}"]}}"#
        ))
    };
    let _ = loads(&tag(&"é".repeat(32)), &ucf());
    let d = rejected(&tag(&"é".repeat(33)), &ucf());
    assert!(
        d.iter().any(|f| f.location.json_path.to_string()
            == "minecraft:block/components/minecraft:crafting_table/crafting_tags/0"),
        "{d:#?}"
    );
    let layer = |layer: &str| {
        newest(&format!(
            r#""minecraft:geometry": {{"identifier": "geometry.x", "culling_layer": "{layer}"}}, {mi}"#
        ))
    };
    for refused in [
        r"demo:a\rb",
        r"demo:a\nb",
        r"minecraft:culling_layer\rleaves",
        "minecraft:culling_layer\u{e9}leaves",
    ] {
        let d = rejected(&layer(refused), &ucf());
        assert!(
            d.iter().any(|f| f.location.json_path.to_string()
                == "minecraft:block/components/minecraft:geometry/culling_layer"),
            "{refused:?}: {d:#?}"
        );
    }
    for accepted in ["demo:a\u{2028}b", "minecraft:culling_layerXleaves"] {
        let _ = loads(&layer(accepted), &ucf());
    }
}
