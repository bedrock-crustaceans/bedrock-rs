//! Numbers no member can hold ("a JSON number beyond f32 must never
//! load as infinity and then fail the writer"):
//! vanilla's schema takes any number where a member is Molang (`ExpressionNode` accepts
//! numeric), and `min_engine_version` is kept as written. A number
//! beyond the member is read as the largest finite one of its sign, with a warning at the
//! number; what was read is what is written. An integer token beyond 64 bits is the number
//! it spells (core's reader), in each of those places.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::families::attachable::Attachable;
use bedrock_addon::json::Value;
use bedrock_addon::molang::Molang;

use super::common::*;

const DESCRIPTION: &str = "minecraft:attachable/description";

/// How many clamp warnings `d` has at `path`: ours, without a vanilla text, each on a token.
fn clamp_warnings(d: &Diagnostics, path: &str) -> usize {
    d.iter()
        .filter(|f| {
            f.code.name() == "InvalidValue"
                && f.severity == Severity::Info
                && f.code.to_string() == "invalid value"
                && f.location.json_path.to_string() == path
                && f.location.span.is_some()
        })
        .count()
}

/// Loads `members` at `version`, requires one clamp warning at each of `paths` (below the
/// description) whose span is a number token of the text, and that the written text reads
/// back to the same value without a clamp warning. The loaded value and its written text.
#[track_caller]
fn clamped(version: &str, members: &str, paths: &[&str]) -> (Attachable, String) {
    let text = ghost(version, members);
    let document = loaded(attachable(&text, &cx()));
    for path in paths {
        let at = format!("{DESCRIPTION}/{path}");
        let found: Vec<_> = document
            .diagnostics()
            .iter()
            .filter(|f| {
                f.code.name() == "InvalidValue"
                    && f.severity == Severity::Info
                    && f.location.json_path.to_string() == at
                    && f.location.span.is_some()
            })
            .collect();
        assert_eq!(
            found.len(),
            1,
            "{version} {members}: {at} in\n{}",
            lines(document.diagnostics()).join("\n")
        );
        let span = found[0].location.span.unwrap();
        let covered = &text[span.offset as usize..][..span.len as usize];
        assert!(
            covered.parse::<f64>().is_ok_and(f64::is_infinite)
                || covered.parse::<f32>().is_ok_and(f32::is_infinite),
            "{version} {members}: the span covers {covered:?}"
        );
    }
    let written = document
        .to_json()
        .unwrap_or_else(|e| panic!("{version} {members}: a loaded document is not written: {e:?}"));
    let again = loaded(attachable(&written, &cx()));
    assert_eq!(again.value(), document.value(), "{written}");
    assert_eq!(again.to_json().unwrap(), written);
    for path in paths {
        let at = format!("{DESCRIPTION}/{path}");
        assert!(
            clamp_warnings(again.diagnostics(), &at) == 0,
            "{written}\n{}",
            lines(again.diagnostics()).join("\n")
        );
    }
    (document.value().clone(), written)
}

fn is_const(molang: Option<&Molang>, expected: f32) -> bool {
    matches!(molang, Some(Molang::Const(x)) if x.to_bits() == expected.to_bits())
}

// ---- Molang members: an `f32` constant ----

#[test]
fn a_named_script_beyond_f32_is_read_as_the_largest_finite_float_deviation() {
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        for (token, expected) in [
            ("1e39", f32::MAX),
            ("1e999", f32::MAX),
            ("-1e999", f32::MIN),
        ] {
            let members = format!(r#""scripts": {{"scale": {token}, "parent_setup": "1"}}"#);
            let (attachable, written) = clamped(version, &members, &["scripts/scale"]);
            let scripts = attachable.description.slots.scripts.as_ref().unwrap();
            assert!(
                is_const(scripts.scale.as_ref(), expected),
                "{version} {token}: {:?}",
                scripts.scale
            );
            // A number is written, not its digits as a string.
            let json: Value = facet_json::from_str(&written).unwrap();
            let scale = &json.as_object().unwrap()["minecraft:attachable"]
                .as_object()
                .unwrap()["description"]
                .as_object()
                .unwrap()["scripts"]
                .as_object()
                .unwrap()["scale"];
            assert!(scale.as_number().is_some(), "{written}");
        }
    }
    // The largest float itself is no deviation, at either token kind.
    for token in ["3.4028235e38", "-3.4028235e38"] {
        let members = format!(r#""scripts": {{"scale": {token}}}"#);
        let document = loaded(attachable(&ghost("1.26.0", &members), &cx()));
        silent(document.diagnostics());
    }
}

#[test]
fn a_condition_beyond_f32_is_read_as_the_largest_finite_float_deviation() {
    for version in ["1.10.0", "1.26.0"] {
        let (attachable, _) = clamped(
            version,
            r#""render_controllers": ["controller.render.a", {"controller.render.b": 1e39}, {"controller.render.c": -1e999, "controller.render.d": 1e999}]"#,
            &[
                "render_controllers/1/controller.render.b",
                "render_controllers/2/controller.render.c",
                "render_controllers/2/controller.render.d",
            ],
        );
        assert_eq!(
            attachable
                .description
                .slots
                .render_controllers
                .as_ref()
                .unwrap()
                .len(),
            3
        );
        clamped(
            version,
            r#""animations": {"a": "animation.a"}, "scripts": {"animate": ["a", {"a": 1e999}]}"#,
            &["scripts/animate/1/a"],
        );
    }
}

#[test]
fn an_item_condition_beyond_f32_is_read_as_the_largest_finite_float_deviation() {
    for version in ["1.10.0", "1.26.0"] {
        // The member that does not bind: clamped, and dropped as any second member.
        let (attachable, _) = clamped(
            version,
            r#""item": {"demo:a": "1", "demo:b": -1e39}"#,
            &["item/demo:b"],
        );
        assert!(attachable.binding().is_some());
        // The member that binds: the clamp's warning, and rule 18's for a number (it reads
        // as the empty text whatever the number is).
        let text = ghost(version, r#""item": {"demo:a": 1e999}"#);
        let document = loaded(attachable_of(&text));
        let at = format!("{DESCRIPTION}/item/demo:a");
        let warnings: Vec<_> = document
            .diagnostics()
            .iter()
            .filter(|f| {
                // the clamp's warning and the a16 information
                f.code.name() == "InvalidValue" && f.location.json_path.to_string() == at
            })
            .collect();
        assert_eq!(
            warnings.len(),
            2,
            "{}",
            lines(document.diagnostics()).join("\n")
        );
        assert_eq!(
            warnings
                .iter()
                .filter(|f| f.location.span.is_some())
                .count(),
            1
        );
        let (_, condition) = document.binding().unwrap();
        assert!(condition.is_never());
        let written = document.to_json().unwrap();
        let again = loaded(attachable_of(&written));
        assert_eq!(again.value(), document.value());
        // Rule 18's warning stays; the clamp's is gone.
        assert_eq!(
            again
                .diagnostics()
                .iter()
                .filter(|f| f.code.name() == "InvalidValue")
                .count(),
            1,
            "{written}"
        );
    }
}

fn attachable_of(
    text: &str,
) -> Result<bedrock_addon::document::Document<Attachable>, bedrock_addon::document::Rejected> {
    attachable(text, &cx())
}

#[test]
fn a_statement_beyond_f32_is_dropped_as_any_number_before_1_26_deviation() {
    // Before 1.26.0 an `initialize` / `pre_animation` element is Molang: a number passes the
    // schema and the 1.10 step drops it. At 1.26.0 the element is a string.
    for (version, names) in [
        ("1.8.0", &["pre_animation"][..]),
        ("1.10.0", &["initialize", "pre_animation"][..]),
    ] {
        for name in names {
            let name = *name;
            let members = format!(r#""scripts": {{"{name}": ["variable.a = 1;", 1e999]}}"#);
            let path = format!("scripts/{name}/1");
            let (attachable, _) = clamped(version, &members, &[&path]);
            let scripts = attachable.description.slots.scripts.as_ref().unwrap();
            let kept = if name == "initialize" {
                scripts.initialize.as_ref()
            } else {
                scripts.pre_animation.as_ref()
            };
            assert_eq!(kept.unwrap().len(), 1, "{version} {name}");
        }
    }
    // 1.8.x declares no `initialize`: the member's finding alone, whatever it holds.
    let d = rejected(attachable(
        &ghost("1.8.0", r#""scripts": {"initialize": [1e999]}"#),
        &cx(),
    ));
    assert_eq!(
        lines(&d),
        [format!(
            "Error UnrecognizedField {DESCRIPTION}/scripts/initialize child 'initialize' not valid here."
        )]
    );
    let d = rejected(attachable(
        &ghost("1.26.0", r#""scripts": {"initialize": [1e999]}"#),
        &cx(),
    ));
    assert_eq!(
        lines(&d),
        [format!(
            "Error InvalidType {DESCRIPTION}/scripts/initialize/0 unknown child schema option type.  Allowed types: string"
        )]
    );
}

// ---- where no float is declared ----

#[test]
fn a_number_beyond_f32_where_no_number_is_declared_is_the_wrong_type_alone() {
    for (version, members, at, allowed) in [
        (
            "1.26.0",
            r#""textures": {"default": 1e999}"#,
            "textures/default",
            "string",
        ),
        (
            "1.26.0",
            r#""queryable_geometry": 1e39"#,
            "queryable_geometry",
            "string",
        ),
        (
            "1.26.0",
            r#""spawn_egg": {"texture_index": 1e39}"#,
            "spawn_egg/texture_index",
            "int",
        ),
        (
            "1.26.0",
            r#""scripts": {"variables": {"variable.a": 1e999}}"#,
            "scripts/variables/variable.a",
            "string",
        ),
        (
            "1.26.0",
            r#""min_engine_version": 1e999"#,
            "min_engine_version",
            "string, array",
        ),
        ("1.26.0", r#""hide_armor": 1e39"#, "hide_armor", "bool"),
        (
            "1.8.0",
            r#""render_controllers": [1e39]"#,
            "render_controllers/0",
            "string",
        ),
        (
            "1.8.0",
            r#""render_controllers": [{"a": 1e39}]"#,
            "render_controllers/0",
            "string",
        ),
        (
            "1.8.0",
            r#""animation_controllers": [{"a": 1e39}]"#,
            "animation_controllers/0/a",
            "string",
        ),
    ] {
        let d = rejected(attachable(&ghost(version, members), &cx()));
        assert_eq!(
            lines(&d),
            [format!(
                "Error InvalidType {DESCRIPTION}/{at} unknown child schema option type.  Allowed types: {allowed}"
            )],
            "{version} {members}"
        );
    }
}

// ---- `min_engine_version`: kept as written, a double ----

#[test]
fn a_number_no_double_holds_in_min_engine_version_is_kept_as_the_largest_finite_double_deviation() {
    for (members, paths) in [
        (
            r#""min_engine_version": [1e999, 0, 0]"#,
            &["min_engine_version/0"][..],
        ),
        (
            r#""min_engine_version": [1.5, {"a": [-1e999, 2]}, 1e999]"#,
            &["min_engine_version/1/a/0", "min_engine_version/2"][..],
        ),
    ] {
        for version in ["1.8.0", "1.10.0", "1.26.0"] {
            let (attachable, written) = clamped(version, members, paths);
            assert!(attachable.description.min_engine_version.is_some());
            assert!(written.contains("1.7976931348623157e308"), "{written}");
        }
    }
    // A number a double holds is kept as it is, and nothing is said.
    let text = ghost("1.26.0", r#""min_engine_version": [1e39, -1e300, 0]"#);
    let document = loaded(attachable(&text, &cx()));
    silent(document.diagnostics());
    round_trip(&document, |text| attachable(text, &cx()));
}

// ---- the clamp changes no other finding ----

fn findings_of(text: &str) -> Diagnostics {
    findings(attachable(text, &cx()))
}

#[test]
fn clamped_numbers_leave_every_other_finding_where_it_is() {
    const REJECTED: &str = "/*rejected*/";
    for later in [
        r#"/*rejected*/"spawn_egg": {"texture_index": 1.5}"#,
        r#"/*rejected*/"materials": {"default": {"x": 1}}"#,
        r#"/*rejected*/"unknown_member": 1"#,
        r#"/*rejected*/"hide_armor": 3"#,
        r#""hide_armor": true"#,
    ] {
        let later = later.replace(REJECTED, "");
        let text = format!(
            r#"{{"format_version": "1.26.0", "minecraft:attachable": {{
  "description": {{
    "identifier": "demo:ghost", // 1e41
    "min_engine_version": [1, /* 1e42 */ 2e999, 0],
    "scripts": {{
      "scale": 3e999,
      "scaleX": "1",
      "animate": ["walk", {{"walk": -4e999}}]
    }},
    "animations": {{"walk": "animation.walk"}},
    "render_controllers": [{{"controller.render.a": 5e39}}],
    "held_item_scale": 6e39,
    "item": {{"demo:a": "1", "demo:b": 7e39}},
    {later}
  }}
}}}}"#
        );
        clamp_changes_no_other_finding(
            &text,
            &[
                (
                    "2e999",
                    "2e-09",
                    "minecraft:attachable/description/min_engine_version/1",
                ),
                (
                    "3e999",
                    "3e-09",
                    "minecraft:attachable/description/scripts/scale",
                ),
                (
                    "-4e999",
                    "-4e-09",
                    "minecraft:attachable/description/scripts/animate/1/walk",
                ),
                (
                    "5e39",
                    "5e30",
                    "minecraft:attachable/description/render_controllers/0/controller.render.a",
                ),
                (
                    "6e39",
                    "6e30",
                    "minecraft:attachable/description/held_item_scale",
                ),
                (
                    "7e39",
                    "7e30",
                    "minecraft:attachable/description/item/demo:b",
                ),
            ],
            findings_of,
        );
    }
}

#[test]
fn a_number_of_a_member_written_twice_is_clamped_where_the_last_stands() {
    // of a member written twice the last stands; the first is not read, and not
    // clamped.
    let text = ghost(
        "1.26.0",
        r#""scripts": {"scale": 1e999, "scale": 2}, "held_item_scale": 1, "held_item_scale": 1e39"#,
    );
    let document = loaded(attachable(&text, &cx()));
    let scripts = document.description.slots.scripts.as_ref().unwrap();
    assert!(is_const(scripts.scale.as_ref(), 2.0));
    let at = format!("{DESCRIPTION}/held_item_scale");
    assert_eq!(clamp_warnings(document.diagnostics(), &at), 1);
    let at = format!("{DESCRIPTION}/scripts/scale");
    assert_eq!(clamp_warnings(document.diagnostics(), &at), 0);
    round_trip(&document, |text| attachable(text, &cx()));
    // The number that stands is written after a later member's: each is replaced where it is.
    let text = ghost(
        "1.26.0",
        r#""held_item_scale": 1, "scripts": {"scale": -1e999}, "held_item_scale": 1e39"#,
    );
    clamp_changes_no_other_finding(
        &text,
        &[
            (
                "-1e999",
                "-1e-09",
                "minecraft:attachable/description/scripts/scale",
            ),
            (
                "1e39",
                "1e30",
                "minecraft:attachable/description/held_item_scale",
            ),
        ],
        findings_of,
    );
    let document = loaded(attachable(&text, &cx()));
    let scripts = document.description.slots.scripts.as_ref().unwrap();
    assert!(is_const(scripts.scale.as_ref(), f32::MIN));
    assert_eq!(document.held_item_scale().to_bits(), f32::MAX.to_bits());
}

// ---- a number within `f32` that no `f32` is exactly ----

#[test]
fn a_constant_no_f32_holds_exactly_is_written_as_the_float_it_was_read_as() {
    // a Molang constant is an `f32`, and the token
    // written is that float's: nothing is said at the load, the value read is the value
    // written, and only the spelling of the author's number is lost.
    for (token, float, written) in [
        ("0.123456789012", 0.123_456_79_f32, "0.12345679"),
        ("16777217", 16_777_216.0, "16777216.0"),
        ("1e-50", 0.0, "0"),
        ("-1e-50", -0.0, "-0.0"),
    ] {
        for version in ["1.10.0", "1.26.0"] {
            let members = format!(
                r#""scripts": {{"scale": {token}}}, "render_controllers": [{{"controller.render.a": {token}}}]"#
            );
            let document = loaded(attachable(&ghost(version, &members), &cx()));
            clean(document.diagnostics());
            let scripts = document.description.slots.scripts.as_ref().unwrap();
            assert!(
                is_const(scripts.scale.as_ref(), float),
                "{version} {token}: {:?}",
                scripts.scale
            );
            let text = round_trip(&document, |text| attachable(text, &cx()));
            for member in ["scale", "controller.render.a"] {
                assert!(
                    text.contains(&format!(r#""{member}": {written}"#))
                        && !text.contains(&format!(r#""{member}": {written}0"#)),
                    "{version} {token} at {member}: {text}"
                );
            }
        }
    }
}

// ---- an integer token beyond 64 bits ----
//
// jsoncpp reads an integer token that fits neither 64-bit integer as the double it spells,
// and so does core's reader: to this family such a token is the number,
// whatever its length. It is a wrong type where a string is declared, a constant where the
// member is Molang, and clamped as any number where it is beyond the member.

/// 23 digits: beyond 64 bits, within `f32`. The number it spells is 1e23.
const WIDE: &str = "99999999999999999999999";
/// 39 digits between `f32::MAX` (340282346638528859811704183484516925440) and `u128::MAX`
/// (340282366920938463463374607431768211455): an integer of 128 bits that no `f32` holds.
const ABOVE_F32: &str = "340282360000000000000000000000000000000";

/// 40 digits: beyond 128 bits, a double (1e40), beyond `f32`.
fn forty_digits() -> String {
    "9".repeat(40)
}

/// 400 digits: beyond a double.
fn beyond_a_double() -> String {
    "9".repeat(400)
}

/// Requires `text` with its `{n}` replaced by `wide` to be read as with `plain` in its
/// place: loaded or refused alike, with the same findings (severity, code, path and text),
/// and, loaded, to an equal value. The document of `wide` when it loads.
#[track_caller]
fn reads_as(
    text: &str,
    wide: &str,
    plain: &str,
) -> Option<bedrock_addon::document::Document<Attachable>> {
    assert!(text.contains("{n}"), "{text}");
    let of_wide = attachable(&text.replace("{n}", wide), &cx());
    let of_plain = attachable(&text.replace("{n}", plain), &cx());
    match (of_wide, of_plain) {
        (Ok(of_wide), Ok(of_plain)) => {
            assert_eq!(
                lines(of_wide.diagnostics()),
                lines(of_plain.diagnostics()),
                "{wide} against {plain} in {text}"
            );
            assert_eq!(
                of_wide.value(),
                of_plain.value(),
                "{wide} against {plain} in {text}"
            );
            Some(of_wide)
        }
        (Err(of_wide), Err(of_plain)) => {
            assert_eq!(
                lines(&of_wide.diagnostics),
                lines(&of_plain.diagnostics),
                "{wide} against {plain} in {text}"
            );
            None
        }
        (Ok(of_wide), Err(of_plain)) => panic!(
            "{wide} loads where {plain} is refused, in {text}:\n{}\nagainst\n{}",
            lines(of_wide.diagnostics()).join("\n"),
            lines(&of_plain.diagnostics).join("\n")
        ),
        (Err(of_wide), Ok(_)) => panic!(
            "{wide} is refused where {plain} loads, in {text}:\n{}",
            lines(&of_wide.diagnostics).join("\n")
        ),
    }
}

/// The wide integers of every length, of either sign where the sign changes nothing.
fn wide_integers() -> Vec<String> {
    vec![
        WIDE.to_owned(),
        format!("-{WIDE}"),
        ABOVE_F32.to_owned(),
        forty_digits(),
        beyond_a_double(),
    ]
}

#[test]
fn an_integer_beyond_64_bits_where_a_string_is_declared_is_the_wrong_type() {
    // A number where a string is declared: the file is refused, as for `5`.
    for wide in wide_integers() {
        for version in ["1.8.0", "1.10.0", "1.26.0"] {
            for (members, at) in [
                (r#""textures": {"default": {n}}"#, "textures/default"),
                (r#""materials": {"default": {n}}"#, "materials/default"),
                (r#""geometry": {"default": {n}}"#, "geometry/default"),
                (r#""animations": {"hold": {n}}"#, "animations/hold"),
            ] {
                let text = ghost(version, members);
                assert!(reads_as(&text, &wide, "5").is_none(), "{version} {members}");
                let d = rejected(attachable(&text.replace("{n}", &wide), &cx()));
                assert_eq!(
                    lines(&d),
                    [format!(
                        "Error InvalidType {DESCRIPTION}/{at} unknown child schema option type.  Allowed types: string"
                    )],
                    "{version} {members} {wide}"
                );
            }
            // The identifier.
            let text = doc(version, r#"{"identifier": {n}}"#);
            assert!(reads_as(&text, &wide, "5").is_none(), "{version}");
            let d = rejected(attachable(&text.replace("{n}", &wide), &cx()));
            expect(
                &d,
                "InvalidType",
                Severity::Error,
                "unknown child schema option type.  Allowed types: string",
            );
            expect_at(
                &d,
                "InvalidType",
                Severity::Error,
                &format!("{DESCRIPTION}/identifier"),
            );
        }
        // `item` is a string or an object: rule 15's error, as for `"item": 5`.
        for version in ["1.10.0", "1.26.0"] {
            let text = ghost(version, r#""item": {n}"#);
            assert!(reads_as(&text, &wide, "5").is_none(), "{version}");
            let d = rejected(attachable(&text.replace("{n}", &wide), &cx()));
            assert_eq!(
                lines(&d),
                [format!(
                    "Error InvalidValue {DESCRIPTION}/item unknown child schema option type.  Allowed types: string, object"
                )],
                "{version} {wide}"
            );
        }
    }
}

#[test]
fn an_integer_beyond_64_bits_in_a_number_s_place_is_the_number_it_spells() {
    // Within `f32` the token is the constant it spells, as the real token `1e23` is.
    for (wide, plain, float) in [
        (WIDE.to_owned(), "1e23", 1e23_f32),
        (format!("-{WIDE}"), "-1e23", -1e23_f32),
    ] {
        for version in ["1.8.0", "1.10.0", "1.26.0"] {
            let text = ghost(version, r#""scripts": {"scale": {n}, "parent_setup": "1"}"#);
            let document = reads_as(&text, &wide, plain).unwrap();
            clean(document.diagnostics());
            let scripts = document.description.slots.scripts.as_ref().unwrap();
            assert!(
                is_const(scripts.scale.as_ref(), float),
                "{version} {wide}: {:?}",
                scripts.scale
            );
            // A number is written, not its digits as a string, and it reads back.
            let written = round_trip(&document, |text| attachable(text, &cx()));
            assert!(!written.contains(&format!("\"{wide}\"")), "{written}");
        }
        for version in ["1.10.0", "1.26.0"] {
            for members in [
                r#""render_controllers": ["controller.render.a", {"controller.render.b": {n}}]"#,
                r#""animations": {"a": "animation.a"}, "scripts": {"animate": ["a", {"a": {n}}]}"#,
                // The member that does not bind, and the one that does (rule 18: a number
                // reads as the empty text, whatever the number).
                r#""item": {"demo:a": "1", "demo:b": {n}}"#,
                r#""item": {"demo:a": {n}}"#,
            ] {
                let document = reads_as(&ghost(version, members), &wide, plain).unwrap();
                let written = round_trip(&document, |text| attachable(text, &cx()));
                assert!(!written.contains(&format!("\"{wide}\"")), "{written}");
            }
        }
        // `held_item_scale`, a float.
        let text = ghost("1.26.0", r#""held_item_scale": {n}"#);
        let document = reads_as(&text, &wide, plain).unwrap();
        if float > 0.0 {
            clean(document.diagnostics());
            assert_eq!(document.held_item_scale().to_bits(), float.to_bits());
        }
        let written = round_trip(&document, |text| attachable(text, &cx()));
        assert!(!written.contains(&format!("\"{wide}\"")), "{written}");
    }
    // `min_engine_version` is kept as written, a double: nothing is said of a number a
    // double holds, 128 bits or more.
    for (wide, plain) in [
        ("100000000000000000000000".to_owned(), "1e23"),
        (forty_digits(), "1e40"),
    ] {
        for version in ["1.8.0", "1.10.0", "1.26.0"] {
            let text = ghost(version, r#""min_engine_version": [{n}, 0, 0]"#);
            let document = reads_as(&text, &wide, plain).unwrap();
            silent(document.diagnostics());
            let written = round_trip(&document, |text| attachable(text, &cx()));
            assert!(!written.contains(&format!("\"{wide}\"")), "{written}");
        }
    }
}

#[test]
fn an_integer_beyond_f32_is_read_as_the_largest_finite_float_deviation() {
    // An integer token: 39 digits between `f32::MAX` and `u128::MAX`, 40 digits
    // (a double), 400 digits (beyond a double, as `1e999`). Each is clamped with the warning
    // at its token, and read as the real token beyond `f32` is.
    for (wide, plain, expected) in [
        (ABOVE_F32.to_owned(), "1e39", f32::MAX),
        (format!("-{ABOVE_F32}"), "-1e39", f32::MIN),
        (forty_digits(), "1e40", f32::MAX),
        (beyond_a_double(), "1e999", f32::MAX),
        (format!("-{}", beyond_a_double()), "-1e999", f32::MIN),
    ] {
        for version in ["1.8.0", "1.10.0", "1.26.0"] {
            let members = r#""scripts": {"scale": {n}, "parent_setup": "1"}"#;
            let (attachable, _) =
                clamped(version, &members.replace("{n}", &wide), &["scripts/scale"]);
            let scripts = attachable.description.slots.scripts.as_ref().unwrap();
            assert!(
                is_const(scripts.scale.as_ref(), expected),
                "{version} {wide}: {:?}",
                scripts.scale
            );
            assert!(reads_as(&ghost(version, members), &wide, plain).is_some());
        }
        for version in ["1.10.0", "1.26.0"] {
            let members = r#""render_controllers": ["controller.render.a", {"controller.render.b": {n}}], "animations": {"a": "animation.a"}, "scripts": {"animate": ["a", {"a": {n}}]}, "item": {"demo:a": "1", "demo:b": {n}}"#;
            clamped(
                version,
                &members.replace("{n}", &wide),
                &[
                    "render_controllers/1/controller.render.b",
                    "scripts/animate/1/a",
                    "item/demo:b",
                ],
            );
            assert!(reads_as(&ghost(version, members), &wide, plain).is_some());
        }
        // `held_item_scale`.
        let members = r#""held_item_scale": {n}"#;
        let (attachable, _) = clamped(
            "1.26.0",
            &members.replace("{n}", &wide),
            &["held_item_scale"],
        );
        if expected > 0.0 {
            assert_eq!(attachable.held_item_scale().to_bits(), expected.to_bits());
        }
        assert!(reads_as(&ghost("1.26.0", members), &wide, plain).is_some());
    }
    // `min_engine_version`: beyond a double only.
    for version in ["1.8.0", "1.10.0", "1.26.0"] {
        let members = r#""min_engine_version": [{n}, 0, 0]"#;
        let (attachable, written) = clamped(
            version,
            &members.replace("{n}", &beyond_a_double()),
            &["min_engine_version/0"],
        );
        assert!(attachable.description.min_engine_version.is_some());
        assert!(written.contains("1.7976931348623157e308"), "{written}");
        assert!(reads_as(&ghost(version, members), &beyond_a_double(), "1e999").is_some());
    }
}

#[test]
fn a_clamped_wide_integer_leaves_every_other_finding_where_it_is() {
    // The spans of findings after a wide integer, clamped or not: each is where it is with
    // an integer of the same length that needs no clamp in its place.
    let forty = forty_digits();
    let within_39 = format!("1{}", "0".repeat(38));
    let within_40 = format!("1.{}", "0".repeat(38));
    for later in [
        r#""spawn_egg": {"texture_index": 1.5}"#,
        r#""unknown_member": 1"#,
        r#""hide_armor": 3"#,
        r#""hide_armor": true"#,
    ] {
        let text = format!(
            r#"{{"format_version": "1.26.0", "minecraft:attachable": {{
  "description": {{
    "identifier": "demo:ghost",
    "min_engine_version": [1, {WIDE}, 0],
    "scripts": {{
      "scale": {ABOVE_F32},
      "scaleX": {WIDE}
    }},
    "held_item_scale": {forty},
    {later}
  }}
}}}}"#
        );
        clamp_changes_no_other_finding(
            &text,
            &[
                (
                    ABOVE_F32,
                    &within_39,
                    "minecraft:attachable/description/scripts/scale",
                ),
                (
                    &forty,
                    &within_40,
                    "minecraft:attachable/description/held_item_scale",
                ),
            ],
            findings_of,
        );
    }
}
