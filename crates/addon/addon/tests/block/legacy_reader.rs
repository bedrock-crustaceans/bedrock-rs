//! How the legacy block reader takes a scalar of another JSON type than the member's.
//!
//! The game reads the `components` of a document below 1.26.20 through its non-strict
//! schema reader: every cereal-registered description, the material instances among them, is
//! registered with the strict flag clear, so its members go to the non-strict loader. That
//! reader's float read takes a JSON bool as `0.0` or `1.0` and fails every other type but a
//! number with `errc` 22; the strict reader's float read takes numbers only. The game's own tests load `"ambient_occlusion": false` in a
//! `1.20.0` block .

use bedrock_addon::diagnostic::Kind;
use bedrock_addon::families::block::component::{LightEmission, MaterialInstances};

use super::common::{context, document, loads, rejected};

/// A block `demo:x` of `version` whose instance `*` has `instance`.
fn with_instance(version: &str, instance: &str) -> String {
    document(
        version,
        &format!(
            r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"minecraft:geometry": "geometry.x", "minecraft:material_instances": {{"*": {instance}}}}}}}"#
        ),
    )
}

#[test]
fn a_legacy_ambient_occlusion_bool_reads_as_zero_or_one() {
    for version in ["1.19.40", "1.21.40", "1.21.120", "1.26.0"] {
        for (written, expected) in [("false", 0.0), ("true", 1.0)] {
            let text = with_instance(
                version,
                &format!(r#"{{"texture": "x", "ambient_occlusion": {written}}}"#),
            );
            let block = loads(&text, &context());
            let read = block
                .components
                .get::<MaterialInstances>()
                .and_then(|m| m.resolve("*"))
                .and_then(|i| i.ambient_occlusion);
            assert_eq!(
                read,
                Some(expected),
                "{version} {written}: {:#?}",
                block.diagnostics()
            );
            assert!(
                block
                    .diagnostics()
                    .at_least(bedrock_addon::diagnostic::Severity::Warning)
                    .next()
                    .is_none(),
                "{version} {written}: {:#?}",
                block.diagnostics()
            );
        }
    }
}

#[test]
fn a_legacy_ambient_occlusion_string_still_rejects_the_block() {
    let text = with_instance("1.21.40", r#"{"texture": "x", "ambient_occlusion": "1"}"#);
    let d = rejected(&text, &context());
    assert!(
        d.iter()
            // By legacy name: the variant and the definitions that were it.
            .any(|f| matches!(f.code.id(), "core/invalid_type")),
        "{d:#?}"
    );
}

#[test]
fn a_cereal_ambient_occlusion_bool_is_not_taken() {
    let text = with_instance("1.26.20", r#"{"texture": "x", "ambient_occlusion": false}"#);
    let d = rejected(&text, &context());
    assert!(
        d.iter()
            // By legacy name: the variant and the definitions that were it.
            .any(|f| matches!(f.code.id(), "core/invalid_type")),
        "{d:#?}"
    );
}

/// A block `demo:x` of `version` with the one component `name`.
fn with_component(version: &str, name: &str, value: &str) -> String {
    document(
        version,
        &format!(
            r#"{{"description": {{"identifier": "demo:x"}}, "components": {{"{name}": {value}}}}}"#
        ),
    )
}

/// The non-strict reader's bool read takes a number as `value != 0` (an integer
/// token by its 64 bits, a double by `fabs(x) != 0.0`) and a bool as itself.
#[test]
fn a_legacy_bool_member_takes_a_number_as_nonzero() {
    for (written, expected) in [
        ("0", false),
        ("1", true),
        ("2", true),
        ("-1", true),
        ("0.0", false),
        ("0.5", true),
    ] {
        let text = with_instance(
            "1.21.40",
            &format!(r#"{{"texture": "x", "face_dimming": {written}}}"#),
        );
        let block = loads(&text, &context());
        let read = block
            .components
            .get::<MaterialInstances>()
            .and_then(|m| m.resolve("*"))
            .and_then(|i| i.face_dimming);
        assert_eq!(
            read,
            Some(expected),
            "{written}: {:#?}",
            block.diagnostics()
        );
    }
}

/// The non-strict reader's unsigned 8-bit read: a bool is `0` or `1`, an integer token its
/// low 8 bits (no range check), a double the truncation toward zero.
#[test]
fn a_legacy_integer_member_wraps_and_truncates() {
    for (written, expected) in [
        ("true", 1),
        ("false", 0),
        ("7", 7),
        ("257", 1),
        ("7.9", 7),
        ("3.0", 3),
    ] {
        let text = with_component("1.21.40", "minecraft:light_emission", written);
        let block = loads(&text, &context());
        assert_eq!(
            block.components.get::<LightEmission>().map(|l| l.0),
            Some(expected),
            "{written}: {:#?}",
            block.diagnostics()
        );
    }
}

#[test]
fn a_cereal_integer_member_takes_the_integer_tokens_only() {
    for written in ["true", "3.0", "257"] {
        let text = with_component("1.26.20", "minecraft:light_emission", written);
        let d = rejected(&text, &context());
        assert!(
            // By legacy name: the variant and the definitions that were it.
            d.iter()
                .any(|f| matches!(f.code.kind(), Kind::WrongType | Kind::InvalidValue)),
            "{written}: {d:#?}"
        );
    }
}

#[test]
fn a_legacy_integer_member_still_rejects_a_string() {
    let text = with_component("1.21.40", "minecraft:light_emission", r#""3""#);
    let d = rejected(&text, &context());
    assert!(
        d.iter()
            // By legacy name: the variant and the definitions that were it.
            .any(|f| matches!(f.code.id(), "core/invalid_type")),
        "{d:#?}"
    );
}

/// What the non-strict reader reads is then held to the member's range: `Brightness` is
/// `[0, 15]`, and the game's numeric range constraint logs "the input
/// is outside of the accepted range [0, 15]" with code 0x40, which is in the lenient mask 0xfbf8:
/// the load fails and the block is refused. No clamp.
/// `-1`, `1e39` and `-2147483649` read as 255.
#[test]
fn a_legacy_light_emission_out_of_range_after_the_read_rejects_the_block() {
    for written in ["-1", "1e39", "-2147483649", "16"] {
        let text = with_component("1.21.40", "minecraft:light_emission", written);
        let d = rejected(&text, &context());
        assert!(
            d.iter()
                // By legacy name: the variant and the definitions that were it.
                .any(|f| f.code.kind() == Kind::InvalidValue
                    && f.location
                        .json_path
                        .to_string()
                        .ends_with("minecraft:light_emission")),
            "{written}: {d:#?}"
        );
    }
}

/// `friction` is `range(0.0, 0.9f, inclusive)`: `true` reads as 1.0 and is out of
/// it, which refuses the block as above.
#[test]
fn a_legacy_friction_out_of_range_after_the_read_rejects_the_block() {
    for written in ["true", "0.95"] {
        let text = with_component("1.21.40", "minecraft:friction", written);
        let d = rejected(&text, &context());
        assert!(
            d.iter()
                // By legacy name: the variant and the definitions that were it.
                .any(|f| f.code.kind() == Kind::InvalidValue
                    && f.location
                        .json_path
                        .to_string()
                        .ends_with("minecraft:friction")),
            "{written}: {d:#?}"
        );
    }
}
